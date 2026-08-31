// ============================================================================
// MODULE SKILL: HỆ THỐNG KỸ NĂNG QUÂN CỜ CHIẾN THUẬT TINH HOA (PIECE SKILLS)
// ============================================================================
// skill.rs xây dựng các mẫu hình kỹ năng độc lập cho từng loại quân cờ:
// 1. Xe (Rook): Khóa Cổ Tuyến 2/7, Chiếu Đáy Ghim Sĩ, Kẹp Nách Sườn Cung, Mở Tuyến.
// 2. Pháo (Cannon): Pháo Đầu Uy Lực, Song Pháo Trùng, Pháo Quá Hà, Pháo Gánh Lộ Tâm.
// 3. Mã (Knight): Mã Ngọa Tào Sát Cung, Điếu Ngư Hiểm Hóc, Mã Quỳ Thủ Hộ, Tiền Đồn Áp Sát.
// 4. Tốt (Pawn): Tốt Nhập Cung Kẹp Cổ, Trung Lộ Xung Phong, Phong Tỏa Biên Tượng.
// 5. Sĩ & Tượng (Advisor & Bishop): Pháo Đài Bất Hoại, Tượng Giác Hộ Tâm.
// 100% chú thích tiếng Việt từng dòng & 100% định danh từ đơn tiếng Anh.
// ============================================================================

use crate::board::Position;
use crate::system::weights::Weights;

/// Struct `Skill` quản lý việc tính toán và đánh giá các kỹ năng đặc thù của từng quân cờ
pub struct Skill;

impl Skill {
    /// Đánh giá toàn bộ các kỹ năng quân cờ trên bàn cờ cho cả 2 bên Đỏ và Đen
    #[inline(always)]
    pub fn evaluate(pos: &Position, weights: &Weights, mg: &mut i32, eg: &mut i32) {
        for color in 0..2 {
            let sign = if color == 0 { 1 } else { -1 };
            let other = 1 - color;

            // 1. Kỹ Năng Xe (Rook Skills)
            Self::eval_rook(pos, weights, color, other, sign, mg, eg);

            // 2. Kỹ Năng Pháo (Cannon Skills)
            Self::eval_cannon(pos, weights, color, other, sign, mg, eg);

            // 3. Kỹ Năng Mã (Knight Skills)
            Self::eval_knight(pos, weights, color, other, sign, mg, eg);

            // 4. Kỹ Năng Tốt (Pawn Skills)
            Self::eval_pawn(pos, weights, color, other, sign, mg, eg);

            // 5. Kỹ Năng Sĩ & Tượng (Advisor & Bishop Skills)
            Self::eval_guard(pos, weights, color, other, sign, mg, eg);
        }
    }

    /// Đánh giá kỹ năng chiến thuật của Xe (Rook Skills)
    #[inline(always)]
    fn eval_rook(pos: &Position, _weights: &Weights, color: usize, other: usize, sign: i32, mg: &mut i32, eg: &mut i32) {
        let piece = color * 7 + 4;
        let mut bits = pos.piece[piece];
        let enemy_king_sq = pos.king[other];
        let enemy_king_rank = (enemy_king_sq / 9) as i32;
        let enemy_king_file = (enemy_king_sq % 9) as i32;

        while let Some(sq) = bits.pop() {
            let rank = sq.rank() as i32;
            let file = sq.file() as i32;

            // Kỹ năng 1: Xe Khóa Cổ Tuyến 2 / Tuyến 7 (Throat-Lock Skill)
            let is_throat_line = if color == 0 { rank == 7 } else { rank == 2 };
            if is_throat_line {
                *mg += sign * 180;
                *eg += sign * 360;
                // Nếu đối phương khuyết Sĩ, tăng gấp đôi uy lực khóa cổ
                if pos.counts[other * 7 + 1] <= 1 {
                    *mg += sign * 220;
                    *eg += sign * 450;
                }
            }

            // Kỹ năng 2: Xe Chiếu Đáy Ghim Cung (Bottom-Rank Pinning Skill)
            let is_bottom_rank = if color == 0 { rank == 9 } else { rank == 0 };
            if is_bottom_rank {
                *mg += sign * 200;
                *eg += sign * 400;
                // Nếu Tướng đối phương đang ở đáy, tạo thế ghim chết Sĩ Tượng
                if (color == 0 && enemy_king_rank == 9) || (color == 1 && enemy_king_rank == 0) {
                    *mg += sign * 250;
                    *eg += sign * 500;
                }
            }

            // Kỹ năng 3: Xe Kẹp Nách Sườn Cung (Rib-Attack Skill)
            let is_rib_file = file == 3 || file == 5;
            let in_enemy_palace_zone = if color == 0 { rank >= 7 } else { rank <= 2 };
            if is_rib_file && in_enemy_palace_zone {
                *mg += sign * 240;
                *eg += sign * 480;
            }

            // Kỹ năng 4: Khoảng cách tiếp cận Tướng (King Hunting Proximity)
            let k_dist = (rank - enemy_king_rank).abs() + (file - enemy_king_file).abs();
            if k_dist <= 3 {
                *mg += sign * (4 - k_dist) * 90;
                *eg += sign * (4 - k_dist) * 180;
            }
        }
    }

    /// Đánh giá kỹ năng chiến thuật của Pháo (Cannon Skills)
    #[inline(always)]
    fn eval_cannon(pos: &Position, _weights: &Weights, color: usize, other: usize, sign: i32, mg: &mut i32, eg: &mut i32) {
        let piece = color * 7 + 5;
        let mut bits = pos.piece[piece];
        let enemy_king_sq = pos.king[other];
        let enemy_king_file = (enemy_king_sq % 9) as i32;

        while let Some(sq) = bits.pop() {
            let rank = sq.rank() as i32;
            let file = sq.file() as i32;

            // Kỹ năng 1: Pháo Đầu Công Phá Trung Lộ (Central Cannon Mastery)
            if file == 4 {
                *mg += sign * 260;
                *eg += sign * 320;
                // Nếu Tướng đối phương đang ở lộ 4 (đối đầu trực tiếp), tăng thêm điểm uy hiếp
                if enemy_king_file == 4 {
                    *mg += sign * 150;
                    *eg += sign * 200;
                }
            }

            // Kỹ năng 2: Pháo Quá Hà Phong Tỏa (Cross-River Sniper)
            let is_cross_river = if color == 0 { rank >= 5 } else { rank <= 4 };
            if is_cross_river {
                *mg += sign * 120;
                *eg += sign * 200;
            }

            // Kỹ năng 3: Pháo Biên Phản Kích (Flank Cannon Sniper)
            if file == 0 || file == 8 {
                *mg += sign * 80;
                *eg += sign * 140;
            }

            // Kỹ năng 4: Pháo Gánh Trung Tâm Bảo Vệ Cung (Mount Cannon Defensive Anchor)
            let is_mount_pos = if color == 0 { rank == 2 && (file == 1 || file == 7) } else { rank == 7 && (file == 1 || file == 7) };
            if is_mount_pos {
                *mg += sign * 140;
                *eg += sign * 180;
            }
        }
    }

    /// Đánh giá kỹ năng chiến thuật của Mã (Knight Skills)
    #[inline(always)]
    fn eval_knight(pos: &Position, _weights: &Weights, color: usize, other: usize, sign: i32, mg: &mut i32, eg: &mut i32) {
        let piece = color * 7 + 3;
        let mut bits = pos.piece[piece];
        let enemy_king_sq = pos.king[other];
        let enemy_king_rank = (enemy_king_sq / 9) as i32;
        let enemy_king_file = (enemy_king_sq % 9) as i32;

        while let Some(sq) = bits.pop() {
            let rank = sq.rank() as i32;
            let file = sq.file() as i32;

            // Kỹ năng 1: Mã Ngọa Tào Sát Cung (Corner Leap Palace Infiltration)
            // Ngọa tào tại (c7, g7 cho Đỏ) hoặc (c2, g2 cho Đen)
            let is_corner_leap = if color == 0 {
                (rank == 7 && (file == 2 || file == 6)) || (rank == 8 && (file == 1 || file == 7))
            } else {
                (rank == 2 && (file == 2 || file == 6)) || (rank == 1 && (file == 1 || file == 7))
            };
            if is_corner_leap {
                *mg += sign * 350;
                *eg += sign * 700;
            }

            // Kỹ năng 2: Điếu Ngư Mã Hiểm Hóc (Angler Knight Tactical Lock)
            // Điếu ngư tại (c8, g8 cho Đỏ) hoặc (c1, g1 cho Đen)
            let is_angler = if color == 0 {
                rank == 8 && (file == 2 || file == 6)
            } else {
                rank == 1 && (file == 2 || file == 6)
            };
            if is_angler {
                *mg += sign * 380;
                *eg += sign * 750;
            }

            // Kỹ năng 3: Mã Tiền Đồn Quá Hà (Outpost Cross-River Knight)
            let is_outpost = if color == 0 { rank >= 5 } else { rank <= 4 };
            if is_outpost {
                *mg += sign * 180;
                *eg += sign * 360;
            }

            // Kỹ năng 4: Mã Quỳ / Bình Phong Thủ Hộ (Screen Guard Knight)
            let is_screen = if color == 0 {
                rank == 2 && (file == 2 || file == 6)
            } else {
                rank == 7 && (file == 2 || file == 6)
            };
            if is_screen {
                *mg += sign * 120;
                *eg += sign * 160;
            }

            // Kỹ năng 5: Áp sát Cung Tướng (King Palace Threat Distance)
            let k_dist = (rank - enemy_king_rank).abs() + (file - enemy_king_file).abs();
            if k_dist <= 3 {
                *mg += sign * (4 - k_dist) * 110;
                *eg += sign * (4 - k_dist) * 220;
            }
        }
    }

    /// Đánh giá kỹ năng chiến thuật của Tốt / Binh (Pawn Skills)
    #[inline(always)]
    fn eval_pawn(pos: &Position, _weights: &Weights, color: usize, other: usize, sign: i32, mg: &mut i32, eg: &mut i32) {
        let piece = color * 7 + 6;
        let mut bits = pos.piece[piece];
        let enemy_king_sq = pos.king[other];
        let enemy_king_rank = (enemy_king_sq / 9) as i32;
        let enemy_king_file = (enemy_king_sq % 9) as i32;

        while let Some(sq) = bits.pop() {
            let rank = sq.rank() as i32;
            let file = sq.file() as i32;

            // Kỹ năng 1: Tốt Lọt Cung / Nhập Cung Kẹp Cổ (Palace Infiltration Pawn)
            let in_enemy_palace = if color == 0 {
                rank >= 7 && file >= 3 && file <= 5
            } else {
                rank <= 2 && file >= 3 && file <= 5
            };
            if in_enemy_palace {
                *mg += sign * 450;
                *eg += sign * 900;
                // Nếu Tốt kề sát Tướng (khoảng cách 1 ô), tạo đòn sát cục tức thì
                let k_dist = (rank - enemy_king_rank).abs() + (file - enemy_king_file).abs();
                if k_dist == 1 {
                    *mg += sign * 500;
                    *eg += sign * 1000;
                }
            }

            // Kỹ năng 2: Tốt Trung Lộ Xung Phong (Central Passed Pawn)
            if file == 4 {
                let cross_river = if color == 0 { rank >= 5 } else { rank <= 4 };
                if cross_river {
                    *mg += sign * 220;
                    *eg += sign * 440;
                }
            }

            // Kỹ năng 3: Tốt Phong Tỏa Biên / Đè Tượng (Flank Elephant Clamp)
            let is_elephant_clamp = if color == 0 {
                rank == 6 && (file == 2 || file == 6)
            } else {
                rank == 3 && (file == 2 || file == 6)
            };
            if is_elephant_clamp {
                *mg += sign * 160;
                *eg += sign * 320;
            }
        }
    }

    /// Đánh giá kỹ năng phòng thủ của Sĩ & Tượng (Advisor & Bishop Guard Skills)
    #[inline(always)]
    fn eval_guard(pos: &Position, _weights: &Weights, color: usize, _other: usize, sign: i32, mg: &mut i32, eg: &mut i32) {
        let advisors = pos.counts[color * 7 + 1];
        let bishops = pos.counts[color * 7 + 2];

        // Kỹ năng 1: Pháo Đài Sĩ Tượng Toàn Vẹn (Iron Fortress Harmony)
        if advisors == 2 && bishops == 2 {
            *mg += sign * 280;
            *eg += sign * 560;
        }

        // Kỹ năng 2: Sĩ Liên Hoàn Tâm Cung (Linked Advisors Palace Guard)
        if advisors == 2 {
            *mg += sign * 150;
            *eg += sign * 300;
        }

        // Kỹ năng 3: Tượng Giác Che Chắn Tuyến Sông (Wing Bishops Flank Control)
        if bishops == 2 {
            *mg += sign * 120;
            *eg += sign * 240;
        }
    }
}
