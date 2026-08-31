// ============================================================================
// MODULE TRAP: HỆ THỐNG ĐÁNH GIÁ BẪY CỜ VÀ KHỐNG CHẾ KHÔNG GIAN (TRAP SYSTEM)
// ============================================================================
// `TrapSystem` chịu trách nhiệm tính toán:
// - Phát hiện Mã nghẽn chân nhốt trong góc (Trapped Knight).
// - Phát hiện Xe bị nhốt góc không có đường xuất trận (Trapped Rook).
// - Phát hiện Pháo mất ngòi cơ động (Trapped Cannon).
// - Đo lường độ thắt chặt không gian của quân đối phương (Constriction).
// ============================================================================

use crate::board::Position;
use crate::movegen::lookup;
use crate::system::weights::Weights;

/// Struct `TrapSystem` quản lý toàn bộ logic đánh giá bẫy cờ
pub struct TrapSystem;

impl TrapSystem {
    /// Đánh giá điểm bẫy cờ và phong tỏa cho cả 2 phe (Đỏ và Đen)
    #[inline(always)]
    pub fn evaluate(pos: &Position, weights: &Weights) -> (i32, i32) {
        let mut mg = 0i32;
        let mut eg = 0i32;

        let mut color = 0usize;
        while color < 2 {
            let sign = if color == 0 { 1 } else { -1 };

            // 1. Kiểm tra Mã bị nghẽn chân (Trapped Knight)
            let knight_piece = if color == 0 { 3 } else { 10 };
            let mut knights = pos.piece[knight_piece];
            while let Some(sq) = knights.pop() {
                let attacks = lookup::knight(sq.0 as usize);
                let mut moves_count = 0i32;
                let mut targets = attacks;
                while let Some(to) = targets.pop() {
                    let leg = lookup::leg(sq.0 as usize, to.0 as usize);
                    if leg < 90 && pos.grid[leg as usize] == 14 {
                        moves_count += 1;
                    }
                }
                if moves_count == 0 {
                    mg += sign * weights.trap.trapped_knight_mg;
                    eg += sign * weights.trap.trapped_knight_eg;
                }
            }

            // 2. Kiểm tra Xe bị nhốt góc (Trapped Rook)
            let rook_piece = if color == 0 { 4 } else { 11 };
            let mut rooks = pos.piece[rook_piece];
            while let Some(sq) = rooks.pop() {
                let file = sq.file();
                let rank = sq.rank();
                let is_corner = (file == 0 || file == 8) && (rank == 0 || rank == 9);
                if is_corner {
                    let mobility = lookup::rook(sq.0, pos.occupied, pos.color[1 - color]).count();
                    if mobility <= 1 {
                        mg += sign * weights.trap.trapped_rook_mg;
                        eg += sign * weights.trap.trapped_rook_eg;
                    }
                }
            }

            // 3. Kiểm tra Pháo mất ngòi (Trapped Cannon)
            let cannon_piece = if color == 0 { 5 } else { 12 };
            let mut cannons = pos.piece[cannon_piece];
            while let Some(sq) = cannons.pop() {
                let mobility = lookup::cannon(sq.0, pos.occupied, pos.color[1 - color]).count();
                if mobility == 0 {
                    mg += sign * weights.trap.trapped_cannon_mg;
                    eg += sign * weights.trap.trapped_cannon_eg;
                }
            }

            color += 1;
        }

        (mg, eg)
    }
}
