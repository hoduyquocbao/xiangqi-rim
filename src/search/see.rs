// ============================================================================
// MODULE SEE: STATIC EXCHANGE EVALUATION (TÍNH TOÁN TĨNH CHUỖI ĐỔI QUÂN)
// ============================================================================
// `see.rs` chịu trách nhiệm ước lượng kết quả trao đổi quân tại một ô mục tiêu `to`
// mà KHÔNG cần cập nhật bàn cờ hay NNUE accumulator:
// - `See::evaluate`: Trả về true nếu điểm trao đổi >= `threshold`.
// - `See::score`: Trả về điểm trao đổi centipawn chính xác.
// - 100% chú thích tiếng Việt từng dòng & 100% định danh từ đơn tiếng Anh.
// ============================================================================

use crate::board::{Bitboard, Position, Square};
use crate::movegen::lookup::{self, RAY};
use crate::movegen::types::Move;
use crate::search::order::VALUES;

/// Struct `See` chứa các hàm tĩnh thực thi Static Exchange Evaluation, căn lề 64-byte.
#[repr(C, align(64))]
pub struct See;

impl See {
    /// Đánh giá xem nước đi ăn quân/di chuyển `mv` trên vị trí `pos` có đạt điểm SEE >= `threshold` hay không.
    /// Ép buộc inlining `#[inline(always)]` tối ưu hóa trên hot path tìm kiếm.
    #[inline(always)]
    pub fn evaluate(pos: &Position, mv: Move, threshold: i32) -> bool {
        if !mv.valid() {
            return false;
        }

        let from = mv.from as usize;
        let to = mv.to as usize;
        let moving = pos.grid[from] as usize;
        let victim = pos.grid[to] as usize;

        // Giá trị quân bị ăn tại ô mục tiêu
        let initial = if victim < 14 { VALUES[victim] } else { 0 };

        // Giá trị quân đang di chuyển
        let moving_val = if moving < 14 { VALUES[moving] } else { 0 };

        // Cắt tỉa nhanh: Nếu điểm thu được ban đầu trừ giá trị quân đi vẫn >= threshold -> Luôn đúng
        if initial - moving_val >= threshold {
            return true;
        }

        // Mảng lưu vết kết quả trao đổi qua từng tầng (Negamax swap array)
        let mut gain = [0i32; 32];
        gain[0] = initial;

        let mut occ = pos.occupied;
        occ.clear(Square(from as u8));

        let mut side = (pos.side as usize) ^ 1; // Phe đối phương chuẩn bị phản đòn ăn lại
        let mut depth = 1usize;
        let mut attacker_val = moving_val;

        loop {
            gain[depth] = attacker_val - gain[depth - 1];

            // Tìm quân tấn công ô `to` nhỏ nhất của phe `side` trên Bitboard `occ`
            let attacker = Self::smallest_attacker(pos, to, side, occ);
            if attacker.is_none() {
                break;
            }

            let (att_sq, _att_piece, val) = attacker.unwrap();
            occ.clear(Square(att_sq as u8));

            attacker_val = val;
            side ^= 1;
            depth += 1;

            if depth >= 32 {
                break;
            }
        }

        // Đảo ngược Minimax Negamax backward pass
        let mut i = depth - 1;
        while i > 0 {
            gain[i - 1] = gain[i - 1].min(-gain[i]);
            i -= 1;
        }

        gain[0] >= threshold
    }

    /// Đánh giá điểm SEE chính xác dưới dạng số nguyên centipawns.
    #[inline(always)]
    pub fn score(pos: &Position, mv: Move) -> i32 {
        if !mv.valid() {
            return 0;
        }

        let from = mv.from as usize;
        let to = mv.to as usize;
        let moving = pos.grid[from] as usize;
        let victim = pos.grid[to] as usize;

        let initial = if victim < 14 { VALUES[victim] } else { 0 };
        let moving_val = if moving < 14 { VALUES[moving] } else { 0 };

        let mut gain = [0i32; 32];
        gain[0] = initial;

        let mut occ = pos.occupied;
        occ.clear(Square(from as u8));

        let mut side = (pos.side as usize) ^ 1;
        let mut depth = 1usize;
        let mut attacker_val = moving_val;

        loop {
            gain[depth] = attacker_val - gain[depth - 1];

            let attacker = Self::smallest_attacker(pos, to, side, occ);
            if attacker.is_none() {
                break;
            }

            let (att_sq, _att_piece, val) = attacker.unwrap();
            occ.clear(Square(att_sq as u8));

            attacker_val = val;
            side ^= 1;
            depth += 1;

            if depth >= 32 {
                break;
            }
        }

        let mut i = depth - 1;
        while i > 0 {
            gain[i - 1] = gain[i - 1].min(-gain[i]);
            i -= 1;
        }

        gain[0]
    }

    /// Tìm quân tấn công có giá trị nhỏ nhất (Least Valuable Attacker) hướng vào ô `target_sq` của phe `side`.
    #[inline(always)]
    fn smallest_attacker(
        pos: &Position,
        target_sq: usize,
        side: usize,
        occ: Bitboard,
    ) -> Option<(usize, usize, i32)> {
        let side_offset = side * 7;

        // 1. Tốt (Pawn = 100 centipawns)
        let pawns = pos.piece[side_offset + 6] & occ;
        if pawns.active() {
            let cand = Self::pawn_attackers_to(target_sq, side);
            let valid_pawns = cand & pawns;
            if valid_pawns.active() {
                let p_sq = valid_pawns.lsb_idx();
                return Some((p_sq, side_offset + 6, 100));
            }
        }

        // 2. Sĩ (Advisor = 200 centipawns)
        let advisors = pos.piece[side_offset + 1] & occ;
        if advisors.active() {
            let valid_advisors = lookup::advisor(side, target_sq) & advisors;
            if valid_advisors.active() {
                let a_sq = valid_advisors.lsb_idx();
                return Some((a_sq, side_offset + 1, 200));
            }
        }

        // 3. Tượng (Bishop = 200 centipawns)
        let elephants = pos.piece[side_offset + 2] & occ;
        if elephants.active() {
            let mut cand = lookup::elephant(side, target_sq) & elephants;
            while let Some(sq) = cand.pop() {
                let eye_sq = lookup::eye(sq.index(), target_sq);
                if !occ.test(Square(eye_sq)) {
                    return Some((sq.index(), side_offset + 2, 200));
                }
            }
        }

        // 4. Mã (Knight = 400 centipawns)
        let knights = pos.piece[side_offset + 3] & occ;
        if knights.active() {
            let mut cand = lookup::knight(target_sq) & knights;
            while let Some(sq) = cand.pop() {
                let leg_sq = lookup::leg(sq.index(), target_sq);
                if !occ.test(Square(leg_sq)) {
                    return Some((sq.index(), side_offset + 3, 400));
                }
            }
        }

        // 5. Pháo (Cannon = 450 centipawns)
        let cannons = pos.piece[side_offset + 5] & occ;
        if cannons.active() {
            for dir in 0..4 {
                let r = lookup::ray(dir, target_sq);
                let b = r & occ;
                if b.active() {
                    let mount_idx = if dir == 0 || dir == 2 {
                        b.lsb_idx()
                    } else {
                        b.msb_idx()
                    };
                    let behind = RAY[dir][mount_idx] & occ;
                    if behind.active() {
                        let cannon_sq = if dir == 0 || dir == 2 {
                            behind.lsb_idx()
                        } else {
                            behind.msb_idx()
                        };
                        if cannons.test(Square(cannon_sq as u8)) {
                            return Some((cannon_sq, side_offset + 5, 450));
                        }
                    }
                }
            }
        }

        // 6. Xe (Rook = 900 centipawns)
        let rooks = pos.piece[side_offset + 4] & occ;
        if rooks.active() {
            for dir in 0..4 {
                let r = lookup::ray(dir, target_sq);
                let b = r & occ;
                if b.active() {
                    let rook_sq = if dir == 0 || dir == 2 {
                        b.lsb_idx()
                    } else {
                        b.msb_idx()
                    };
                    if rooks.test(Square(rook_sq as u8)) {
                        return Some((rook_sq, side_offset + 4, 900));
                    }
                }
            }
        }

        // 7. Tướng (King = 20000 centipawns)
        let king_bb = pos.piece[side_offset + 0] & occ;
        if king_bb.active() {
            let k_sq = king_bb.lsb_idx();
            if (lookup::king(side, target_sq) & Bitboard::mask(Square(k_sq as u8))).active() {
                return Some((k_sq, side_offset + 0, 20000));
            }
            // Flying General Check
            if (k_sq % 9) == (target_sq % 9) {
                let b = lookup::between(k_sq, target_sq) & occ;
                if !b.active() {
                    return Some((k_sq, side_offset + 0, 20000));
                }
            }
        }

        None
    }

    /// Lấy Bitboard các vị trí Tốt thuộc phe `side` có khả năng tấn công ô `target_sq`.
    #[inline(always)]
    fn pawn_attackers_to(target_sq: usize, side: usize) -> Bitboard {
        let mut bb = Bitboard::empty();
        if side == 0 {
            // Tốt Đỏ tiến lên (+9) hoặc đi ngang khi qua sông (>= 45)
            if target_sq >= 9 {
                bb.set(Square((target_sq - 9) as u8));
            }
            if target_sq % 9 > 0 && target_sq >= 46 {
                bb.set(Square((target_sq - 1) as u8));
            }
            if target_sq % 9 < 8 && target_sq >= 44 {
                bb.set(Square((target_sq + 1) as u8));
            }
        } else {
            // Tốt Đen lùi xuống (-9) hoặc đi ngang khi qua sông (< 45)
            if target_sq + 9 < 90 {
                bb.set(Square((target_sq + 9) as u8));
            }
            if target_sq % 9 > 0 && target_sq <= 45 {
                bb.set(Square((target_sq - 1) as u8));
            }
            if target_sq % 9 < 8 && target_sq < 44 {
                bb.set(Square((target_sq + 1) as u8));
            }
        }
        bb
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Parser;
    use crate::movegen::types::Move;

    #[test]
    fn test_see_evaluation() {
        let pos = Parser::parse(Parser::DEFAULT);
        // Nước đi h2e2 (Pháo 2 bình 5): move 19 -> 22, ô trống -> SEE score = 0
        let mv = Move::new(19, 22);
        let score = See::score(&pos, mv);
        assert_eq!(score, 0);
        assert!(See::evaluate(&pos, mv, 0));
    }
}


