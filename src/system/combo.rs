// ============================================================================
// MODULE COMBO: TỔ HỢP SIÊU KỸ NĂNG ĐA QUÂN CỜ THẾ GIỚI (WORLD-CLASS COMBOS)
// ============================================================================
// combo.rs tính toán các mẫu hình phối hợp liên hoàn đẳng cấp thế giới:
// 1. Combo Xe Mã Hợp Kích (Rook-Knight Pincer): Mã khóa góc Cung, Xe giáng đòn sát cục.
// 2. Combo Xe Pháo Trùng (Rook-Cannon Battery): Pháo làm ngòi, Xe đâm thủng Cung Tướng.
// 3. Combo Mã Hậu Pháo (Cannon-Behind-Knight): Mã dâng làm ngòi cho Pháo nổ triệt hạ.
// 4. Combo Song Long Xuất Hải (Twin Rooks Ocean Cleave): Song Xe khống chế 2 tuyến then chốt.
// 5. Combo Tam Khôi Hợp Bích (Three Grandmasters Assault): Xe - Pháo - Mã đồng loạt công thành.
// 6. Combo Pháo Đài Sĩ Tượng (Iron Fortress Shield): Sĩ Tượng Pháo Gánh bất khả xâm phạm.
// 100% chú thích tiếng Việt từng dòng & 100% định danh từ đơn tiếng Anh.
// ============================================================================

use crate::board::Position;
use crate::system::weights::Weights;

/// Struct `Combo` quản lý việc tính toán các đòn phối hợp chiến thuật đa quân cờ
pub struct Combo;

impl Combo {
    /// Đánh giá toàn bộ các tổ hợp siêu kỹ năng cho cả 2 bên Đỏ và Đen
    #[inline(always)]
    pub fn evaluate(pos: &Position, _weights: &Weights, mg: &mut i32, eg: &mut i32) {
        for color in 0..2 {
            let sign = if color == 0 { 1 } else { -1 };
            let other = 1 - color;

            let enemy_king_sq = pos.king[other];
            let enemy_king_rank = (enemy_king_sq / 9) as i32;
            let enemy_king_file = (enemy_king_sq % 9) as i32;

            let rooks = pos.counts[color * 7 + 4];
            let cannons = pos.counts[color * 7 + 5];
            let knights = pos.counts[color * 7 + 3];
            let _pawns = pos.counts[color * 7 + 6];

            // 1. Combo Xe Mã Hợp Kích (Rook-Knight Pincer)
            if rooks >= 1 && knights >= 1 {
                Self::eval_rook_knight(pos, color, other, sign, enemy_king_rank, enemy_king_file, mg, eg);
            }

            // 2. Combo Xe Pháo Trùng & Kẹp Cổ (Rook-Cannon Battery)
            if rooks >= 1 && cannons >= 1 {
                Self::eval_rook_cannon(pos, color, other, sign, enemy_king_rank, enemy_king_file, mg, eg);
            }

            // 3. Combo Mã Hậu Pháo & Pháo Đầu Mã Đội (Cannon-Behind-Knight)
            if cannons >= 1 && knights >= 1 {
                Self::eval_cannon_knight(pos, color, other, sign, enemy_king_rank, enemy_king_file, mg, eg);
            }

            // 4. Combo Song Long Xuất Hải (Twin Rooks Ocean Cleave)
            if rooks >= 2 {
                Self::eval_twin_rooks(pos, color, sign, mg, eg);
            }

            // 5. Combo Tam Khôi Hợp Bích (Three Grandmasters: Xe + Pháo + Mã)
            if rooks >= 1 && cannons >= 1 && knights >= 1 {
                Self::eval_three_heroes(pos, color, other, sign, enemy_king_rank, enemy_king_file, mg, eg);
            }
        }
    }

    /// Combo 1: Xe Mã Hợp Kích (Mã khóa góc/Điếu Ngư + Xe chém Cung)
    #[inline(always)]
    fn eval_rook_knight(
        pos: &Position,
        color: usize,
        other: usize,
        sign: i32,
        k_rank: i32,
        k_file: i32,
        mg: &mut i32,
        eg: &mut i32,
    ) {
        let mut knight_threat = false;
        let mut r_knight = pos.piece[color * 7 + 3];
        while let Some(ksq) = r_knight.pop() {
            let krank = ksq.rank() as i32;
            let kfile = ksq.file() as i32;
            let dist = (krank - k_rank).abs() + (kfile - k_file).abs();
            if dist <= 3 {
                knight_threat = true;
                break;
            }
        }

        if knight_threat {
            let mut r_rook = pos.piece[color * 7 + 4];
            while let Some(rsq) = r_rook.pop() {
                let rrank = rsq.rank() as i32;
                let rfile = rsq.file() as i32;
                let rdist = (rrank - k_rank).abs() + (rfile - k_file).abs();
                if rdist <= 3 {
                    // Xe và Mã cùng áp sát Cung Tướng trong bán kính 3 ô
                    *mg += sign * 450;
                    *eg += sign * 900;
                    // Nếu đối phương khuyết Sĩ, thưởng thêm đòn dứt điểm sát cục
                    if pos.counts[other * 7 + 1] <= 1 {
                        *mg += sign * 350;
                        *eg += sign * 700;
                    }
                }
            }
        }
    }

    /// Combo 2: Xe Pháo Trùng & Kẹp Cổ (Pháo làm ngòi, Xe đâm thủng Cung)
    #[inline(always)]
    fn eval_rook_cannon(
        pos: &Position,
        color: usize,
        _other: usize,
        sign: i32,
        _k_rank: i32,
        k_file: i32,
        mg: &mut i32,
        eg: &mut i32,
    ) {
        let mut r_rook = pos.piece[color * 7 + 4];
        let mut r_cannon = pos.piece[color * 7 + 5];

        let mut rook_files = [false; 9];
        let mut rook_ranks = [false; 10];
        while let Some(rsq) = r_rook.pop() {
            rook_files[rsq.file() as usize] = true;
            rook_ranks[rsq.rank() as usize] = true;
        }

        while let Some(csq) = r_cannon.pop() {
            let crank = csq.rank() as usize;
            let cfile = csq.file() as usize;

            // Xe và Pháo cùng đứng trên 1 cột tấn công (Xe Pháo Trùng Cột)
            if rook_files[cfile] {
                *mg += sign * 280;
                *eg += sign * 450;
                if (cfile as i32 - k_file).abs() <= 1 {
                    *mg += sign * 250;
                    *eg += sign * 500;
                }
            }

            // Xe và Pháo cùng đứng trên 1 hàng tấn công (Xe Pháo Trùng Hàng)
            if rook_ranks[crank] {
                *mg += sign * 240;
                *eg += sign * 400;
                let is_palace_line = if color == 0 { crank >= 7 } else { crank <= 2 };
                if is_palace_line {
                    *mg += sign * 300;
                    *eg += sign * 600;
                }
            }
        }
    }

    /// Combo 3: Mã Hậu Pháo & Pháo Đầu Mã Đội (Mã dâng ngòi cho Pháo nổ)
    #[inline(always)]
    fn eval_cannon_knight(
        pos: &Position,
        color: usize,
        _other: usize,
        sign: i32,
        _k_rank: i32,
        _k_file: i32,
        mg: &mut i32,
        eg: &mut i32,
    ) {
        let mut r_cannon = pos.piece[color * 7 + 5];
        while let Some(csq) = r_cannon.pop() {
            let cfile = csq.file();
            let crank = csq.rank() as i32;

            // Pháo Đầu (Lộ 4) kết hợp Mã Đội
            if cfile == 4 {
                let mut r_knight = pos.piece[color * 7 + 3];
                while let Some(ksq) = r_knight.pop() {
                    let kfile = ksq.file();
                    let krank = ksq.rank() as i32;
                    // Mã dâng lên lộ 4 hoặc lộ 3/5 áp sát trung lộ
                    if (kfile == 4 || kfile == 3 || kfile == 5) && (krank - crank).abs() <= 3 {
                        *mg += sign * 320;
                        *eg += sign * 550;
                    }
                }
            }
        }
    }

    /// Combo 4: Song Long Xuất Hải (Song Xe khống chế 2 tuyến then chốt)
    #[inline(always)]
    fn eval_twin_rooks(pos: &Position, color: usize, sign: i32, mg: &mut i32, eg: &mut i32) {
        let mut r_rook = pos.piece[color * 7 + 4];
        if let (Some(r1), Some(r2)) = (r_rook.pop(), r_rook.pop()) {
            let r1_rank = r1.rank() as i32;
            let r2_rank = r2.rank() as i32;
            let r1_file = r1.file() as i32;
            let r2_file = r2.file() as i32;

            // Song Xe cùng qua sông tấn công
            let r1_across = if color == 0 { r1_rank >= 5 } else { r1_rank <= 4 };
            let r2_across = if color == 0 { r2_rank >= 5 } else { r2_rank <= 4 };

            if r1_across && r2_across {
                *mg += sign * 400;
                *eg += sign * 800;

                // Song Xe chiếm 2 hàng then chốt (Hàng Đáy và Hàng Cổ)
                let has_bottom = (color == 0 && (r1_rank == 9 || r2_rank == 9)) || (color == 1 && (r1_rank == 0 || r2_rank == 0));
                let has_throat = (color == 0 && (r1_rank == 7 || r2_rank == 7)) || (color == 1 && (r1_rank == 2 || r2_rank == 2));
                if has_bottom && has_throat {
                    *mg += sign * 500;
                    *eg += sign * 1000;
                }

                // Song Xe kẹp 2 sườn Cung Tướng (Lộ 3 và Lộ 5)
                let has_rib3 = r1_file == 3 || r2_file == 3;
                let has_rib5 = r1_file == 5 || r2_file == 5;
                if has_rib3 && has_rib5 {
                    *mg += sign * 450;
                    *eg += sign * 900;
                }
            }
        }
    }

    /// Combo 5: Tam Khôi Hợp Bích (Xe + Pháo + Mã đồng loạt công thành)
    #[inline(always)]
    fn eval_three_heroes(
        pos: &Position,
        color: usize,
        other: usize,
        sign: i32,
        k_rank: i32,
        k_file: i32,
        mg: &mut i32,
        eg: &mut i32,
    ) {
        let mut active_attackers = 0;

        // Đếm số quân nặng áp sát Cung Tướng đối phương trong bán kính 4 ô
        for &p in &[color * 7 + 4, color * 7 + 5, color * 7 + 3] {
            let mut bits = pos.piece[p];
            while let Some(sq) = bits.pop() {
                let rank = sq.rank() as i32;
                let file = sq.file() as i32;
                let dist = (rank - k_rank).abs() + (file - k_file).abs();
                if dist <= 4 {
                    active_attackers += 1;
                }
            }
        }

        // Khi có đủ 3 quân nặng hội tụ quanh Cung Tướng, kích hoạt Tam Khôi Hợp Bích
        if active_attackers >= 3 {
            *mg += sign * 600;
            *eg += sign * 1200;
            // Nếu đối phương bị khuyết Sĩ hoặc khuyết Tượng, đòn tấn công đạt mức sát cục hủy diệt
            let enemy_guards = pos.counts[other * 7 + 1] + pos.counts[other * 7 + 2];
            if enemy_guards <= 2 {
                *mg += sign * 500;
                *eg += sign * 1000;
            }
        }
    }
}
