// ============================================================================
// MODULE KING: HỆ THỐNG ĐÁNH GIÁ AN TOÀN CUNG TƯỚNG (KING SAFETY SYSTEM)
// ============================================================================
// `KingSystem` chịu trách nhiệm tính toán:
// - Phạt nặng Tướng rời hàng đáy lên lầu 2/lầu 3 (High-Palace King Penalty).
// - Thưởng bảo tồn cấu trúc Sĩ Tượng (Palace Guard Preservation).
// - Kiểm tra nguy cơ bị đòn chiếu đâm đáy và chiếu rút từ xa.
// ============================================================================

use crate::board::Position;
use crate::system::weights::Weights;

/// Struct `KingSystem` quản lý toàn bộ logic an toàn Tướng
pub struct KingSystem;

impl KingSystem {
    /// Đánh giá an toàn Cung Tướng cho cả 2 phe (Đỏ và Đen)
    #[inline(always)]
    pub fn evaluate(pos: &Position, weights: &Weights) -> (i32, i32) {
        let mut mg = 0i32;
        let mut eg = 0i32;

        let mut color = 0usize;
        while color < 2 {
            let sign = if color == 0 { 1 } else { -1 };
            let other = 1 - color;
            let enemy_attackers = pos.counts[other * 7 + 4] + pos.counts[other * 7 + 5] + pos.counts[other * 7 + 3];

            // Vị trí Tướng
            let king_sq = pos.king[color];
            let rank = king_sq / 9;

            // Phạt Tướng rời hàng đáy lên lầu 2 / lầu 3 khi đối phương còn quân tấn công
            if enemy_attackers > 0 {
                if color == 0 {
                    if rank == 1 {
                        mg += sign * weights.king.floor_two_mg;
                        eg += sign * weights.king.floor_two_eg;
                    } else if rank == 2 {
                        mg += sign * weights.king.floor_three_mg;
                        eg += sign * weights.king.floor_three_eg;
                    }
                } else {
                    if rank == 8 {
                        mg += sign * weights.king.floor_two_mg;
                        eg += sign * weights.king.floor_two_eg;
                    } else if rank == 7 {
                        mg += sign * weights.king.floor_three_mg;
                        eg += sign * weights.king.floor_three_eg;
                    }
                }

                // Phạt Cực Nặng khi Cung Tướng bị uy hiếp bởi nhiều quân tấn công đối phương
                let advisor_count = pos.counts[color * 7 + 1] as i32;
                if enemy_attackers >= 3 && advisor_count <= 1 {
                    mg -= sign * 450;
                    eg -= sign * 850;
                } else if enemy_attackers >= 2 && advisor_count <= 1 {
                    mg -= sign * 250;
                    eg -= sign * 500;
                }
            }

            // Thưởng bảo tồn Sĩ Tượng
            let advisor_count = pos.counts[color * 7 + 1] as i32;
            let bishop_count = pos.counts[color * 7 + 2] as i32;

            mg += sign * advisor_count * weights.king.advisor_mg;
            eg += sign * advisor_count * weights.king.advisor_eg;

            mg += sign * bishop_count * weights.king.bishop_mg;
            eg += sign * bishop_count * weights.king.bishop_eg;

            // Phạt Tượng Tâm bị Xe đè (Pinned Central Elephant) khi Xe đối phương ở hàng 2/7
            let enemy_rook_piece = other * 7 + 4;
            let mut e_rooks = pos.piece[enemy_rook_piece];
            while let Some(rsq) = e_rooks.pop() {
                let rrank = rsq.rank();
                let is_on_defense_line = if color == 0 { rrank == 2 } else { rrank == 7 };
                if is_on_defense_line {
                    let center_elephant_sq = if color == 0 { 22usize } else { 67usize }; // e2 = 22, e7 = 67
                    let expected_bishop = (color * 7 + 2) as u8;
                    if pos.grid[center_elephant_sq] == expected_bishop {
                        mg -= sign * 280;
                        eg -= sign * 550;
                    }
                }
            }

            // Thưởng khai thác khi đối phương bị Khuyết Sĩ / Khuyết Tượng
            let my_attackers = pos.counts[color * 7 + 4] + pos.counts[color * 7 + 5] + pos.counts[color * 7 + 3] + pos.counts[color * 7 + 6];
            if my_attackers > 0 {
                let enemy_advisors = pos.counts[other * 7 + 1];
                let enemy_bishops = pos.counts[other * 7 + 2];

                if enemy_advisors == 1 {
                    mg += sign * weights.king.advisor_lost_one_mg;
                    eg += sign * weights.king.advisor_lost_one_eg;
                } else if enemy_advisors == 0 {
                    mg += sign * weights.king.advisor_lost_all_mg;
                    eg += sign * weights.king.advisor_lost_all_eg;
                }

                if enemy_bishops == 1 {
                    mg += sign * weights.king.bishop_lost_one_mg;
                    eg += sign * weights.king.bishop_lost_one_eg;
                } else if enemy_bishops == 0 {
                    mg += sign * weights.king.bishop_lost_all_mg;
                    eg += sign * weights.king.bishop_lost_all_eg;
                }

                // Thưởng Giam Cầm Tướng đối phương (King Confinement)
                let enemy_king_sq = pos.king[other];
                let enemy_king_file = enemy_king_sq % 9;
                let enemy_king_rank = enemy_king_sq / 9;
                let is_enemy_king_displaced = enemy_king_file != 4 || (if other == 0 { enemy_king_rank > 0 } else { enemy_king_rank < 9 });
                if is_enemy_king_displaced {
                    mg += sign * weights.king.confinement_mg;
                    eg += sign * weights.king.confinement_eg;
                }

                // Đánh giá Mật độ Quân Địch Tấn Công Cung Tướng (Attacker Concentration Threat)
                let mut attacker_points = 0i32;
                let enemy_pieces = [other * 7 + 4, other * 7 + 5, other * 7 + 3]; // Rook, Cannon, Knight
                let our_king_sq = pos.king[color];
                let our_king_rank = (our_king_sq / 9) as i32;
                let our_king_file = (our_king_sq % 9) as i32;

                for &ep in &enemy_pieces {
                    let mut e_bits = pos.piece[ep];
                    while let Some(esq) = e_bits.pop() {
                        let erank = esq.rank() as i32;
                        let efile = esq.file() as i32;
                        let in_our_half = if color == 0 { erank <= 4 } else { erank >= 5 };
                        if in_our_half {
                            let k_dist = (erank - our_king_rank).abs() + (efile - our_king_file).abs();
                            if k_dist <= 4 {
                                let piece_weight = if ep % 7 == 4 { 4 } else { 3 };
                                attacker_points += piece_weight * (5 - k_dist);
                            }
                        }
                    }
                }

                // Nếu mật độ công kích lớn (>= 15 điểm), phạt nặng nguy cơ bị sát cục
                if attacker_points >= 15 {
                    let danger_mg = (attacker_points - 10) * 80;
                    let danger_eg = (attacker_points - 10) * 160;
                    mg -= sign * danger_mg;
                    eg -= sign * danger_eg;
                }

                // Thưởng Mạng Lưới Sát Cục (Mating Net Pattern) khi đối phương hở Cung và ta có quân nặng
                let has_major_attackers = pos.counts[color * 7 + 4] >= 1 && (pos.counts[color * 7 + 5] >= 1 || pos.counts[color * 7 + 3] >= 1);
                if has_major_attackers && (enemy_advisors <= 1 || is_enemy_king_displaced) {
                    mg += sign * weights.king.mating_net_mg;
                    eg += sign * weights.king.mating_net_eg;
                }
            }

            // Phạt Đòn Mã Ngọa Tào / Điếu Ngư đối phương uy hiếp Cung Tướng (Enemy Knight Infiltration Danger)
            let enemy_knight_piece = other * 7 + 3;
            let mut e_knights = pos.piece[enemy_knight_piece];
            while let Some(ksq) = e_knights.pop() {
                let krank = ksq.rank();
                let kfile = ksq.file();
                let in_danger_zone = if color == 0 {
                    krank <= 3 && (kfile >= 2 && kfile <= 6)
                } else {
                    krank >= 6 && (kfile >= 2 && kfile <= 6)
                };
                if in_danger_zone {
                    let adv_cnt = pos.counts[color * 7 + 1];
                    let penalty_mg = if adv_cnt <= 1 { 350 } else { 180 };
                    let penalty_eg = if adv_cnt <= 1 { 700 } else { 350 };
                    mg -= sign * penalty_mg;
                    eg -= sign * penalty_eg;
                }
            }

            // Phạt Đòn Xe/Pháo đâm đáy Cung Tướng (Enemy Baseline Infiltration Danger)
            let base_rank = if color == 0 { 0 } else { 9 };
            let enemy_rook_piece = other * 7 + 4;
            let enemy_cannon_piece = other * 7 + 5;
            let mut base_attackers = 0;
            let mut r_bits = pos.piece[enemy_rook_piece];
            while let Some(rsq) = r_bits.pop() {
                if rsq.rank() == base_rank {
                    base_attackers += 1;
                }
            }
            let mut c_bits = pos.piece[enemy_cannon_piece];
            while let Some(csq) = c_bits.pop() {
                if csq.rank() == base_rank {
                    base_attackers += 1;
                }
            }
            if base_attackers > 0 {
                let adv_cnt = pos.counts[color * 7 + 1];
                let penalty_mg = if base_attackers >= 2 || adv_cnt <= 1 { 500 } else { 250 };
                let penalty_eg = if base_attackers >= 2 || adv_cnt <= 1 { 1000 } else { 500 };
                mg -= sign * penalty_mg;
                eg -= sign * penalty_eg;
            }

            color += 1;
        }

        (mg, eg)
    }
}
