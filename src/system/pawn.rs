// ============================================================================
// MODULE PAWN: HỆ THỐNG ĐÁNH GIÁ TỐT QUA SÔNG & TÀN CUỘC (PAWN SYSTEM)
// ============================================================================
// `PawnSystem` chịu trách nhiệm tính toán:
// - Thưởng Tốt qua sông và tiến sâu vào Cung Tướng đối phương.
// - Thưởng Tốt qua sông có Xe/Pháo/Mã yểm trợ phía sau (Escorted Passed Pawn).
// - Phạt nặng khi để Tốt đối phương tự do tiến sâu không có quân chốt chặn (Enemy Blockade).
// ============================================================================

use crate::board::Position;
use crate::system::weights::Weights;

/// Struct `PawnSystem` quản lý toàn bộ logic đánh giá Tốt
pub struct PawnSystem;

impl PawnSystem {
    /// Đánh giá cấu trúc Tốt cho cả 2 phe (Đỏ và Đen)
    #[inline(always)]
    pub fn evaluate(pos: &Position, weights: &Weights) -> (i32, i32) {
        let mut mg = 0i32;
        let mut eg = 0i32;

        let mut color = 0usize;
        while color < 2 {
            let sign = if color == 0 { 1 } else { -1 };
            let pawn_piece = if color == 0 { 6 } else { 13 };
            let mut pawns = pos.piece[pawn_piece];

            while let Some(sq) = pawns.pop() {
                let rank = sq.rank();
                let file = sq.file();
                let crossed_river = if color == 0 { rank >= 5 } else { rank <= 4 };

                if crossed_river {
                    mg += sign * weights.pawn.river_mg;
                    eg += sign * weights.pawn.river_eg;

                    // Thưởng Tốt tiến sâu vào hàng 3/4 (áp sát Cung Tướng)
                    let is_deep = if color == 0 { rank >= 7 } else { rank <= 2 };
                    if is_deep {
                        mg += sign * weights.pawn.deep_mg;
                        eg += sign * weights.pawn.deep_eg;

                        // Thưởng thêm nếu Tốt ở cột trung tâm hoặc sườn Cung (Cột 3, 4, 5)
                        if file >= 3 && file <= 5 {
                            mg += sign * 40;
                            eg += sign * 80;
                        }

                        // Thưởng cực lớn khi Tốt chính thức nhập Cung Tướng đối phương (Palace Breach)
                        let in_palace = if color == 0 { rank >= 7 && file >= 3 && file <= 5 } else { rank <= 2 && file >= 3 && file <= 5 };
                        if in_palace {
                            mg += sign * weights.pawn.palace_breach_mg;
                            eg += sign * weights.pawn.palace_breach_eg;
                        }
                    }

                    // Thưởng Rút Ngắn Khoảng Cách Tấn Công Tướng (King Hunter Distance Incentive)
                    let other = 1 - color;
                    let enemy_king_sq = pos.king[other];
                    let enemy_king_rank = (enemy_king_sq / 9) as i32;
                    let enemy_king_file = (enemy_king_sq % 9) as i32;
                    let dist = (rank as i32 - enemy_king_rank).abs() + (file as i32 - enemy_king_file).abs();
                    if dist <= 4 {
                        let hunter_bonus = (5 - dist) * 120;
                        eg += sign * hunter_bonus;
                        if dist <= 2 {
                            eg += sign * 350; // Kẹp nách / áp sát trực tiếp Tướng
                        }
                    }

                    // Kiểm tra yểm trợ: Có quân ta phía sau yểm trợ đường tiến của Tốt
                    let behind_sq = if color == 0 {
                        if rank > 0 { Some(sq.0 - 9) } else { None }
                    } else {
                        if rank < 9 { Some(sq.0 + 9) } else { None }
                    };

                    if let Some(bsq) = behind_sq {
                        let behind_piece = pos.grid[bsq as usize];
                        if behind_piece < 14 {
                            let is_friendly = if color == 0 { behind_piece < 7 } else { behind_piece >= 7 };
                            if is_friendly {
                                mg += sign * weights.pawn.escorted_mg;
                                eg += sign * weights.pawn.escorted_eg;
                            }
                        }
                    }
                }
            }

            color += 1;
        }

        (mg, eg)
    }
}
