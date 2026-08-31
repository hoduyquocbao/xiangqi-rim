// ============================================================================
// MODULE ROOK: HỆ THỐNG ĐÁNH GIÁ KIỂM SOÁT LỘ XE (ROOK SYSTEM)
// ============================================================================
// `RookSystem` chịu trách nhiệm tính toán:
// - Thưởng Xe kiểm soát lộ mở và lộ bán mở (Open / Semi-Open File Dominance).
// - Thưởng Xe chiếm hàng tuần hà (Rank 4 / Rank 6 Patrol).
// - Thưởng Xe chiếm sườn Cung Tướng (Cột 3, 5).
// - Phạt nặng Xe dạt biên ăn Tốt rác khi Cung Tướng đối phương đang bị đe dọa.
// ============================================================================

use crate::board::Position;
use crate::system::weights::Weights;

/// Struct `RookSystem` quản lý toàn bộ logic đánh giá Xe
pub struct RookSystem;

impl RookSystem {
    /// Đánh giá hoạt lực và vị trí Xe cho cả 2 phe (Đỏ và Đen)
    #[inline(always)]
    pub fn evaluate(pos: &Position, weights: &Weights) -> (i32, i32) {
        let mut mg = 0i32;
        let mut eg = 0i32;

        let mut color = 0usize;
        while color < 2 {
            let sign = if color == 0 { 1 } else { -1 };
            let rook_piece = if color == 0 { 4 } else { 11 };
            let mut rooks = pos.piece[rook_piece];

            while let Some(sq) = rooks.pop() {
                let rank = sq.rank();
                let file = sq.file();

                // Xe kiểm soát lộ mở / lộ trung tâm
                if file == 4 {
                    mg += sign * weights.rook.open_mg * 2;
                    eg += sign * weights.rook.open_eg * 2;
                } else if file == 3 || file == 5 {
                    mg += sign * weights.rook.flank_mg;
                    eg += sign * weights.rook.flank_eg;
                }

                // Xe tuần hà kiểm soát mặt sông
                if (color == 0 && rank == 4) || (color == 1 && rank == 5) {
                    mg += sign * weights.rook.patrol_mg;
                    eg += sign * weights.rook.patrol_eg;
                }

                // Xe áp sát Cung Tướng đối phương (Palace Infiltration / Palace Press)
                let enemy_palace_rank = if color == 0 { rank >= 7 } else { rank <= 2 };
                if enemy_palace_rank {
                    mg += sign * weights.rook.palace_press_mg;
                    eg += sign * weights.rook.palace_press_eg;

                    // Thưởng cực lớn khi Xe kết hợp với Tốt đã qua sông áp sát Cung
                    let friendly_pawn_piece = if color == 0 { 6 } else { 13 };
                    let pawns = pos.piece[friendly_pawn_piece];
                    let mut pawn_in_palace = false;
                    let mut p_bits = pawns;
                    while let Some(psq) = p_bits.pop() {
                        let prank = psq.rank();
                        let pfile = psq.file();
                        let in_p = if color == 0 { prank >= 7 && pfile >= 2 && pfile <= 6 } else { prank <= 2 && pfile >= 2 && pfile <= 6 };
                        if in_p {
                            pawn_in_palace = true;
                            break;
                        }
                    }

                    if pawn_in_palace {
                        mg += sign * (weights.rook.palace_press_mg + 100);
                        eg += sign * (weights.rook.palace_press_eg + 200);
                    }

                    // Phạt nặng Xe bị bẫy góc đáy Cung Tướng đối phương (Trapped Corner Rook)
                    let is_corner_bottom = if color == 0 { rank == 9 && (file == 2 || file == 6) } else { rank == 0 && (file == 2 || file == 6) };
                    let other_knight_count = pos.counts[(1 - color) * 7 + 3];
                    if is_corner_bottom && other_knight_count > 0 {
                        mg -= sign * 350;
                        eg -= sign * 650;
                    }
                }
            }

            // Đánh giá Song Xa cùng lộ hoặc cùng hàng (Double Rook Battery)
            let rook_count = pos.counts[color * 7 + 4];
            if rook_count == 2 {
                let mut r_bits = pos.piece[rook_piece];
                if let (Some(r1), Some(r2)) = (r_bits.pop(), r_bits.pop()) {
                    if r1.file() == r2.file() {
                        let f = r1.file();
                        // Thưởng Song Xa trên lộ sườn hoặc lộ trung tâm
                        let bonus_mg = if f == 4 || f == 3 || f == 5 { 350 } else { 200 };
                        let bonus_eg = if f == 4 || f == 3 || f == 5 { 700 } else { 400 };
                        mg += sign * bonus_mg;
                        eg += sign * bonus_eg;
                    }
                    if r1.rank() == r2.rank() {
                        let r = r1.rank();
                        let in_enemy_territory = if color == 0 { r >= 7 } else { r <= 2 };
                        if in_enemy_territory {
                            mg += sign * 450;
                            eg += sign * 900;
                        }
                    }
                }
            }

            // Phạt Địch Chiếm Hàng Tuyến Áp Cung (Enemy Throat-Locking Rook on Rank 2/7)
            let other = 1 - color;
            let enemy_rook_piece = other * 7 + 4;
            let throat_rank = if color == 0 { 2 } else { 7 };
            let mut e_rooks = pos.piece[enemy_rook_piece];
            while let Some(ersq) = e_rooks.pop() {
                if ersq.rank() == throat_rank && (2..=6).contains(&ersq.file()) {
                    let adv_count = pos.counts[color * 7 + 1];
                    let penalty_mg = if adv_count <= 1 { 450 } else { 220 };
                    let penalty_eg = if adv_count <= 1 { 900 } else { 450 };
                    mg -= sign * penalty_mg;
                    eg -= sign * penalty_eg;
                }
            }

            // Phạt mất sạch Xe khi đối phương vẫn còn Xe (Zero Rook vs Enemy Rook Deficit)
            let other_rook_count = pos.counts[other * 7 + 4];
            if rook_count == 0 && other_rook_count > 0 {
                mg -= sign * 650;
                eg -= sign * (1200 * other_rook_count as i32);
            }

            color += 1;
        }

        (mg, eg)
    }
}
