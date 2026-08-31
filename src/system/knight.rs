// ============================================================================
// MODULE KNIGHT: HỆ THỐNG ĐÁNH GIÁ VẬN MÃ CHIẾN THUẬT (KNIGHT SYSTEM)
// ============================================================================
// `KnightSystem` chịu trách nhiệm tính toán:
// - Phạt nặng Mã dạt biên không bảo vệ (Edge Knight Trap Penalty).
// - Phạt Mã bị cản chân hoàn toàn / kiềm tỏa (Knight Pin / Immobilization).
// - Thưởng Mã qua sông và Mã tiền đồn áp sát Cung Tướng (Ngọa Tào / Điếu Ngư).
// ============================================================================

use crate::board::Position;
use crate::movegen::lookup;
use crate::system::weights::Weights;

/// Struct `KnightSystem` quản lý toàn bộ logic đánh giá Mã
pub struct KnightSystem;

impl KnightSystem {
    /// Đánh giá hoạt lực và vị trí Mã cho cả 2 phe (Đỏ và Đen)
    #[inline(always)]
    pub fn evaluate(pos: &Position, weights: &Weights) -> (i32, i32) {
        let mut mg = 0i32;
        let mut eg = 0i32;

        let mut color = 0usize;
        while color < 2 {
            let sign = if color == 0 { 1 } else { -1 };
            let knight_piece = if color == 0 { 3 } else { 10 };
            let mut knights = pos.piece[knight_piece];

            while let Some(sq) = knights.pop() {
                let attacks = lookup::knight(sq.0 as usize);
                let mut count = 0i32;
                let mut targets = attacks;
                while let Some(to) = targets.pop() {
                    let leg = lookup::leg(sq.0 as usize, to.0 as usize);
                    if leg < 90 && pos.grid[leg as usize] == 14 {
                        count += 1;
                    }
                }
                mg += sign * count * 5;
                eg += sign * count * 7;

                // Phạt Mã bị cản chân / tê liệt hoàn toàn
                if count == 0 {
                    mg += sign * weights.knight.pinned_mg;
                    eg += sign * weights.knight.pinned_eg;
                } else if count <= 2 {
                    mg += sign * weights.knight.restricted_mg;
                    eg += sign * weights.knight.restricted_eg;
                }

                // Phạt Mã nhập biên lộ (Edge Knight Trap Penalty) tại cột 0 (a), cột 8 (i) hoặc góc đáy
                let n_rank = sq.rank();
                let n_file = sq.file();
                let is_edge = n_file == 0 || n_file == 8 || (n_rank == 0 && (n_file == 1 || n_file == 7)) || (n_rank == 9 && (n_file == 1 || n_file == 7));
                let crossed_river = if color == 0 { n_rank >= 5 } else { n_rank <= 4 };
                if is_edge && !crossed_river {
                    mg += sign * weights.knight.edge_mg;
                    eg += sign * weights.knight.edge_eg;
                }

                // Mã qua sông chiếm lĩnh tiền đồn
                if crossed_river {
                    mg += sign * weights.knight.river_mg;
                    eg += sign * weights.knight.river_eg;
                }

                // Mã áp sát Cung Tướng đối phương (Ngọa Tào / Điếu Ngư / Mã Sườn / Khóa Cung)
                let is_deep_knight = if color == 0 { n_rank >= 5 && (2..=6).contains(&n_file) } else { n_rank <= 4 && (2..=6).contains(&n_file) };
                if is_deep_knight {
                    mg += sign * weights.knight.outpost_mg;
                    eg += sign * weights.knight.outpost_eg;

                    // Thưởng cực lớn khi Mã chiếm các điểm hiểm (Ngọa Tào c7/g7/c2/g2 hoặc Điếu Ngư c6/g6/c3/g3)
                    let is_palace_attack_spot = if color == 0 {
                        n_rank >= 6 && (n_file == 2 || n_file == 4 || n_file == 6)
                    } else {
                        n_rank <= 3 && (n_file == 2 || n_file == 4 || n_file == 6)
                    };
                    if is_palace_attack_spot {
                        mg += sign * (weights.knight.outpost_mg + 120);
                        eg += sign * (weights.knight.outpost_eg + 220);
                    }

                    // Thưởng Rút Ngắn Khoảng Cách Mã Tấn Công Tướng (Knight King Hunter Distance)
                    let other = 1 - color;
                    let enemy_king_sq = pos.king[other];
                    let enemy_king_rank = (enemy_king_sq / 9) as i32;
                    let enemy_king_file = (enemy_king_sq % 9) as i32;
                    let dist = (n_rank as i32 - enemy_king_rank).abs() + (n_file as i32 - enemy_king_file).abs();
                    if dist <= 4 {
                        let hunter_bonus = (5 - dist) * 100;
                        eg += sign * hunter_bonus;
                        if dist <= 2 {
                            eg += sign * 300; // Áp sát trực tiếp Cung Tướng
                        }
                    }
                }
            }

            // Phạt Địch Có Mã Hiểm Áp Sát Cung Tướng Ta (Enemy Infiltrating Knight Defense Penalty)
            let other = 1 - color;
            let enemy_knight_piece = other * 7 + 3;
            let our_king_sq = pos.king[color];
            let our_king_rank = (our_king_sq / 9) as i32;
            let our_king_file = (our_king_sq % 9) as i32;
            let mut e_knights = pos.piece[enemy_knight_piece];
            while let Some(eksq) = e_knights.pop() {
                let ek_rank = eksq.rank() as i32;
                let ek_file = eksq.file() as i32;
                let is_enemy_deep = if color == 0 { ek_rank <= 4 && (2..=6).contains(&ek_file) } else { ek_rank >= 5 && (2..=6).contains(&ek_file) };
                if is_enemy_deep {
                    let k_dist = (ek_rank - our_king_rank).abs() + (ek_file - our_king_file).abs();
                    if k_dist <= 4 {
                        let threat_mg = (5 - k_dist) * 160;
                        let threat_eg = (5 - k_dist) * 350;
                        mg -= sign * threat_mg;
                        eg -= sign * threat_eg;
                        if k_dist <= 2 {
                            mg -= sign * 300;
                            eg -= sign * 600;
                        }
                    }
                }
            }

            color += 1;
        }

        (mg, eg)
    }
}
