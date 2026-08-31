// ============================================================================
// MODULE ORDER: HỆ THỐNG SẮP XẾP NƯỚC ĐI TỐI ƯU (MOVE ORDERING SYSTEM)
// ============================================================================
// `OrderSystem` chịu trách nhiệm:
// - Thưởng điểm nước đi sát thủ Killer Moves 1 & 2.
// - Thưởng điểm nước đi phản kích Counter Moves.
// - Tính điểm MVV-LVA (Most Valuable Victim - Least Valuable Attacker) cho các nước ăn quân.
// ============================================================================

use crate::board::Position;
use crate::movegen::Move;
use crate::system::weights::Weights;

/// Struct `OrderSystem` quản lý toàn bộ thuật toán sắp xếp nước đi
pub struct OrderSystem;

impl OrderSystem {
    /// Bảng giá trị cơ bản của 7 loại quân cờ cho MVV-LVA
    pub const PIECE_VALUES: [i32; 15] = [
        0,    // 0: King Đỏ
        20,   // 1: Advisor Đỏ
        20,   // 2: Bishop Đỏ
        45,   // 3: Knight Đỏ
        90,   // 4: Rook Đỏ
        45,   // 5: Cannon Đỏ
        10,   // 6: Pawn Đỏ
        0,    // 7: King Đen
        20,   // 8: Advisor Đen
        20,   // 9: Bishop Đen
        45,   // 10: Knight Đen
        90,   // 11: Rook Đen
        45,   // 12: Cannon Đen
        10,   // 13: Pawn Đen
        0,    // 14: Ô trống
    ];

    /// Tính điểm sắp xếp cho nước đi `mv`
    #[inline(always)]
    pub fn score(pos: &Position, mv: Move, killer1: Move, killer2: Move, counter: Move, weights: &Weights) -> i32 {
        if !mv.valid() {
            return -1_000_000;
        }

        let capture = pos.grid[mv.to as usize];
        if capture < 14 {
            // Nước đi ăn quân: Áp dụng công thức MVV-LVA
            let victim_val = Self::PIECE_VALUES[capture as usize];
            let attacker = pos.grid[mv.from as usize];
            let attacker_val = if attacker < 14 { Self::PIECE_VALUES[attacker as usize] } else { 0 };
            return 1_000_000 + (victim_val * weights.order.mvv_lva_scale) - attacker_val;
        }

        // Nước đi sát thủ 1
        if mv == killer1 {
            return weights.order.killer_one;
        }

        // Nước đi sát thủ 2
        if mv == killer2 {
            return weights.order.killer_two;
        }

        // Nước đi phản kích
        if mv == counter {
            return weights.order.counter_move;
        }

        0
    }
}
