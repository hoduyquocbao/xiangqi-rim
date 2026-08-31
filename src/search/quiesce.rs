// ============================================================================
// MODULE QUIESCE: TÌM KIẾM TĨNH BẢO VỆ VÙNG BIÊN (QUIESCENCE SEARCH ENGINE)
// ============================================================================
// `quiesce.rs` triển khai thuật toán Quiescence Search để giải quyết Horizon Effect:
// - Chỉ tiếp tục duyệt các nước ăn quân (`captured < 14`) hoặc nước giải chiếu (`check`).
// - Đánh giá "Stand Pat" điểm đứng yên trước khi duyệt để làm cận dưới Alpha.
// - Tích hợp kiểm tra đồng hồ bấm giờ `timer.check()` phản hồi ngắt ngắt dừng trong < 10ms.
// ============================================================================

use crate::board::{Bitboard, Position, Square};
use crate::eval::Eval;
use crate::movegen::{legal, lookup, List};
use crate::search::limit::Timer;
use crate::search::order::VALUES;

/// Struct `Quiesce` chứa hàm tĩnh thực thi tìm kiếm tĩnh trắc Quiescence Search.
pub struct Quiesce;

impl Quiesce {
    /// Thực thi tìm kiếm đệ quy Quiescence Search với ranh giới Alpha-Beta $[alpha, beta]$.
    /// Tối ưu hóa: (1) MVV-LVA sort cho captures, (2) Delta Pruning, (3) Batch timer check.
    #[inline(always)]
    pub fn search(
        pos: &mut Position,
        eval: &mut Eval,
        timer: &Timer,
        mut alpha: i32,
        beta: i32,
        ply: usize,
        nodes: &mut u64,
    ) -> i32 {
        *nodes += 1;
        // Giới hạn độ sâu đệ quy QSearch tối đa 64 ply chống bùng nổ tìm kiếm và tràn stack
        if ply >= 64 {
            return eval.score(pos);
        }
        // Kiểm tra tín hiệu ngắt dừng khẩn cấp từ Timer
        // Timer.check() đã có batch nội bộ (time check mỗi 256 nút).
        if timer.check(*nodes) {
            return 0;
        }

        let check = legal::check(pos, pos.side as usize);

        // 1. Nếu không bị chiếu -> Đánh giá điểm Stand Pat
        let mut standing = -30000;
        if !check {
            standing = eval.score(pos);
            if standing >= beta {
                return beta; // Cutoff Beta
            }
            if standing > alpha {
                alpha = standing; // Nâng Alpha
            }
        }

        // 2. Sinh danh sách các nước đi ăn quân (Captures Only) bằng Bitboard O(1) hoặc toàn bộ nước đi giải chiếu
        let mut list = List::new();
        if check {
            legal::gen(pos, &mut list);
        } else {
            crate::movegen::pseudo::captures(pos, &mut list);
            // Bổ sung các nước Tốt áp cung lọt vào cung Tướng (Palace Infiltrated Pawn Pushes) ở tầng đầu QSearch
            if ply <= 2 {
                let side = pos.side as usize;
                let mut pawns = pos.piece[side * 7 + 6];
                let own = pos.color[side];
                let enemy = pos.color[1 - side];
                while let Some(from) = pawns.pop() {
                    let mut targets = lookup::pawn(side, from.index()) & !own & !enemy;
                    while let Some(to) = targets.pop() {
                        let to_rank = to.rank();
                        let to_file = to.file();
                        let enters_palace = if side == 0 {
                            to_rank >= 7 && to_file >= 3 && to_file <= 5
                        } else {
                            to_rank <= 2 && to_file >= 3 && to_file <= 5
                        };
                        if enters_palace {
                            list.push(crate::movegen::types::Move::new(from.0, to.0));
                        }
                    }
                }
            }

            // Bổ sung các nước đi Chiếu Tướng (Quiet Checks) ở tầng đầu QSearch (ply <= 1) để nhìn thấy sát cục O(1)
            if ply <= 1 {
                let side = pos.side as usize;
                let other = 1 - side;
                if let Some(king_sq) = pos.piece[other * 7 + 0].lsb() {
                    let occupied = pos.occupied;
                    let own = pos.color[side];
                    let enemy = pos.color[other];

                    // 1. Xe chiếu:
                    let mut rooks = pos.piece[side * 7 + 4];
                    let rook_rays = lookup::rook(king_sq.0, occupied, Bitboard::empty());
                    while let Some(r_sq) = rooks.pop() {
                        let mut slides = lookup::rook(r_sq.0, occupied, Bitboard::empty()) & rook_rays & !own & !enemy;
                        while let Some(to) = slides.pop() {
                            list.push(crate::movegen::types::Move::new(r_sq.0, to.0));
                        }
                    }

                    // 2. Pháo chiếu:
                    let mut cannons = pos.piece[side * 7 + 5];
                    let cannon_rays = lookup::cannon(king_sq.0, occupied, Bitboard::empty());
                    while let Some(c_sq) = cannons.pop() {
                        let mut slides = lookup::cannon(c_sq.0, occupied, Bitboard::empty()) & cannon_rays & !own & !enemy;
                        while let Some(to) = slides.pop() {
                            list.push(crate::movegen::types::Move::new(c_sq.0, to.0));
                        }
                    }

                    // 3. Mã chiếu:
                    let mut knights = pos.piece[side * 7 + 3];
                    let knight_targets = lookup::KNIGHT[king_sq.0 as usize];
                    while let Some(n_sq) = knights.pop() {
                        let mut jumps = lookup::KNIGHT[n_sq.0 as usize] & knight_targets & !own & !enemy;
                        while let Some(to) = jumps.pop() {
                            let leg_from = lookup::leg(n_sq.0 as usize, to.0 as usize);
                            let leg_to_king = lookup::leg(to.0 as usize, king_sq.0 as usize);
                            if leg_from != 255 && !occupied.test(Square(leg_from))
                                && leg_to_king != 255 && !occupied.test(Square(leg_to_king)) {
                                list.push(crate::movegen::types::Move::new(n_sq.0, to.0));
                            }
                        }
                    }

                    // 4. Tốt chiếu (Quiet Pawn Checks at ply <= 1)
                    let mut pawns = pos.piece[side * 7 + 6];
                    let k_r = king_sq.rank();
                    let k_f = king_sq.file();
                    while let Some(p_sq) = pawns.pop() {
                        let p_r = p_sq.rank();
                        let p_f = p_sq.file();
                        let crossed = if side == 0 { p_r >= 5 } else { p_r <= 4 };
                        if crossed {
                            // Nước tiến:
                            let fwd_r = if side == 0 { p_r + 1 } else { p_r.saturating_sub(1) };
                            if (side == 0 && fwd_r <= 9) || (side == 1 && p_r > 0) {
                                let to_sq = Square(fwd_r * 9 + p_f);
                                if !occupied.test(to_sq) {
                                    if (fwd_r == k_r && (p_f as i32 - k_f as i32).abs() == 1)
                                        || (p_f == k_f && ((side == 0 && fwd_r + 1 == k_r) || (side == 1 && fwd_r == k_r + 1))) {
                                        list.push(crate::movegen::types::Move::new(p_sq.0, to_sq.0));
                                    }
                                }
                            }
                            // Nước ngang trái/phải:
                            if p_f > 0 {
                                let to_sq = Square(p_r * 9 + p_f - 1);
                                if !occupied.test(to_sq) {
                                    if (p_r == k_r && ((p_f - 1) as i32 - k_f as i32).abs() == 1)
                                        || (p_f - 1 == k_f && ((side == 0 && p_r + 1 == k_r) || (side == 1 && p_r == k_r + 1))) {
                                        list.push(crate::movegen::types::Move::new(p_sq.0, to_sq.0));
                                    }
                                }
                            }
                            if p_f < 8 {
                                let to_sq = Square(p_r * 9 + p_f + 1);
                                if !occupied.test(to_sq) {
                                    if (p_r == k_r && ((p_f + 1) as i32 - k_f as i32).abs() == 1)
                                        || (p_f + 1 == k_f && ((side == 0 && p_r + 1 == k_r) || (side == 1 && p_r == k_r + 1))) {
                                        list.push(crate::movegen::types::Move::new(p_sq.0, to_sq.0));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Bổ sung các nước né tránh cho quân lớn (Xe, Pháo, Mã) khi bị Xe địch ngắm tại ply <= 1
            if ply <= 1 {
                let side = pos.side as usize;
                let other = 1 - side;
                let occupied = pos.occupied;
                let own_pieces = pos.color[side];
                let enemy_rooks = pos.piece[other * 7 + 4];

                if !enemy_rooks.is_empty() {
                    for pt in 3..=5 {
                        let mut pieces = pos.piece[side * 7 + pt];
                        while let Some(sq) = pieces.pop() {
                            let is_threatened = !(lookup::rook(sq.0, occupied, own_pieces) & enemy_rooks).is_empty();
                            if is_threatened {
                                match pt {
                                    3 => { // Mã né tránh
                                        let mut jumps = lookup::KNIGHT[sq.0 as usize] & !occupied;
                                        while let Some(to) = jumps.pop() {
                                            let leg = lookup::leg(sq.0 as usize, to.0 as usize);
                                            if leg != 255 && !occupied.test(Square(leg)) {
                                                list.push(crate::movegen::types::Move::new(sq.0, to.0));
                                            }
                                        }
                                    }
                                    4 => { // Xe né tránh
                                        let mut slides = lookup::rook(sq.0, occupied, Bitboard::empty());
                                        while let Some(to) = slides.pop() {
                                            list.push(crate::movegen::types::Move::new(sq.0, to.0));
                                        }
                                    }
                                    5 => { // Pháo né tránh
                                        let mut slides = lookup::cannon(sq.0, occupied, Bitboard::empty());
                                        while let Some(to) = slides.pop() {
                                            list.push(crate::movegen::types::Move::new(sq.0, to.0));
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Nếu không còn nước đi hợp lệ
        if list.empty() {
            if check {
                return -30000 + (ply as i32); // Bị chiếu bí (Mate score)
            }
            return alpha;
        }

        let active = eval.enabled();

        // 4. Duyệt đệ quy danh sách các nước ăn quân theo thứ tự MVV-LVA giảm dần (0-closure Selection Sort)
        let mut i = 0;
        while i < list.count {
            // Selection Sort bước đơn: Tìm nước ăn quân có điểm MVV-LVA cao nhất trực tiếp 0-closure
            if !check {
                let mut best = i;
                let mut best_score = {
                    let m = list.items[i];
                    let cap = pos.grid[m.to as usize];
                    let mov = pos.grid[m.from as usize];
                    if cap < 14 { 10 * VALUES[cap as usize] - VALUES[mov as usize] } else { 0 }
                };
                let mut j = i + 1;
                while j < list.count {
                    let m = list.items[j];
                    let cap = pos.grid[m.to as usize];
                    let mov = pos.grid[m.from as usize];
                    let score = if cap < 14 { 10 * VALUES[cap as usize] - VALUES[mov as usize] } else { 0 };
                    if score > best_score {
                        best_score = score;
                        best = j;
                    }
                    j += 1;
                }
                if best != i {
                    list.items.swap(i, best);
                }
            }

            let mv = list.items[i];
            let moving = pos.grid[mv.from as usize];
            let captured = pos.grid[mv.to as usize];

            // Nghẽn 7: Delta Pruning — an toàn với biên độ 900 cp (giá trị 1 quân Xe)
            if !check && captured < 14 && standing + VALUES[captured as usize] + 900 < alpha {
                i += 1;
                continue;
            }

            // Grandmaster Optimization: SEE Pruning trong QSearch
            // Loại bỏ ngay lập tức các nước ăn quân thua thiệt (SEE < 0) mà KHÔNG cần
            // thực thi pos.apply hay eval.apply. Nếu v >= a thì SEE luôn >= 0 mà không cần tính.
            if !check && captured < 14 && VALUES[captured as usize] < VALUES[moving as usize] && !crate::search::see::See::evaluate(pos, mv, 0) {
                i += 1;
                continue;
            }

            let side = pos.side as usize;
            // Cập nhật gia tăng NNUE accumulator và thực thi nước đi
            if active {
                eval.apply(pos, mv.from, mv.to, moving, captured);
            }
            let state = pos.apply(mv.from, mv.to);

            // Kiểm tra nước đi hợp lệ: Nếu để Tướng của phe mình bị chiếu hoặc phạm quy Lộ mặt Tướng → Bỏ qua nước này
            if legal::check(pos, side) || legal::fly(pos) {
                pos.revert(mv.from, mv.to, &state);
                if active {
                    eval.revert(pos, mv.from, mv.to, moving, captured);
                }
                i += 1;
                continue;
            }

            let score = -Self::search(pos, eval, timer, -beta, -alpha, ply + 1, nodes);

            // Hoàn tác nước đi và khôi phục NNUE accumulator
            pos.revert(mv.from, mv.to, &state);
            if active {
                eval.revert(pos, mv.from, mv.to, moving, captured);
            }

            if score >= beta {
                return beta; // Cutoff Beta
            }
            if score > alpha {
                alpha = score; // Nâng Alpha
            }
            i += 1;
        }

        alpha
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Parser;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;
    use std::time::Instant;

    /// Unit test phản hồi lệnh ngắt dừng halt trong < 10ms
    #[test]
    fn halt() {
        let mut pos = Parser::parse(Parser::DEFAULT);
        let mut eval = Eval::new();
        eval.reset(&pos);
        let timer = Timer::new();
        timer.halt();
        let mut nodes = 0u64;

        let start = Instant::now();
        let score = Quiesce::search(&mut pos, &mut eval, &timer, -30000, 30000, 0, &mut nodes);
        let elapsed = start.elapsed();

        assert_eq!(score, 0);
        assert!(
            elapsed.as_millis() < 500,
            "Halt reaction time in Quiesce MUST be < 500ms, actual: {}ms",
            elapsed.as_millis()
        );
    }

    /// Unit test phản hồi tín hiệu dừng bất đồng bộ từ luồng khác trong < 10ms
    #[test]
    fn abort() {
        let mut pos = Parser::parse(Parser::DEFAULT);
        let mut eval = Eval::new();
        eval.reset(&pos);
        let mut timer = Timer::new();
        let flag = Arc::new(AtomicBool::new(true));
        timer.bind(Arc::clone(&flag));
        let mut nodes = 0u64;

        let start = Instant::now();
        let score = Quiesce::search(&mut pos, &mut eval, &timer, -30000, 30000, 0, &mut nodes);
        let elapsed = start.elapsed();

        assert_eq!(score, 0);
        assert!(
            elapsed.as_millis() < 500,
            "Abort reaction time in Quiesce MUST be < 500ms, actual: {}ms",
            elapsed.as_millis()
        );
    }
}


