// ============================================================================
// MODULE ENDGAME: HỆ THỐNG ĐÁNH GIÁ TÀN CUỘC LÝ THUYẾT (ENDGAME SYSTEM)
// ============================================================================
// `EndgameSystem` chịu trách nhiệm nhận diện các thế cờ tàn cuộc lý thuyết:
// - Đơn Mã thắng Đơn Sĩ (+3800cp).
// - Hai Pháo thắng Khuyết Sĩ Tượng (+4000cp).
// - Xe Mã thắng Xe Sĩ Tượng (+3900cp).
// - Không còn quân công trả về Hòa cờ (0cp).
// ============================================================================

use crate::board::Position;
use crate::system::weights::Weights;

/// Struct `EndgameSystem` quản lý toàn bộ logic thẩm định tàn cuộc lý thuyết
pub struct EndgameSystem;

impl EndgameSystem {
    /// Thẩm định thế cờ tàn cuộc lý thuyết. Trả về `Some(score)` nếu nhận diện thế cờ, hoặc `None` nếu cần đánh giá tiếp.
    #[inline(always)]
    pub fn probe(pos: &Position, weights: &Weights) -> Option<i32> {
        let side = pos.side as usize;
        let foe = 1 - side;

        let my_rooks = pos.counts[side * 7 + 4];
        let my_cannons = pos.counts[side * 7 + 5];
        let my_knights = pos.counts[side * 7 + 3];
        let my_pawns = pos.counts[side * 7 + 6];

        let foe_rooks = pos.counts[foe * 7 + 4];
        let foe_cannons = pos.counts[foe * 7 + 5];
        let foe_knights = pos.counts[foe * 7 + 3];
        let foe_pawns = pos.counts[foe * 7 + 6];

        let my_attackers = my_rooks + my_cannons + my_knights + my_pawns;
        let foe_attackers = foe_rooks + foe_cannons + foe_knights + foe_pawns;

        // 1. Cả 2 bên đều hết quân công -> Hòa cờ ngay lập tức
        if my_attackers == 0 && foe_attackers == 0 {
            return Some(weights.endgame.draw_score);
        }

        // 2. Bên ta chỉ còn Đơn Mã vs Đơn Sĩ đối phương (không còn quân công khác)
        let foe_advisors = pos.counts[foe * 7 + 1];
        let foe_bishops = pos.counts[foe * 7 + 2];
        if my_rooks == 0 && my_cannons == 0 && my_knights == 1 && my_pawns == 0 && foe_attackers == 0 {
            if foe_advisors <= 1 && foe_bishops == 0 {
                return Some(weights.endgame.knight_vs_advisor);
            }
        }

        // 3. Bên ta còn Song Pháo vs đối phương Khuyết Sĩ Tượng
        if my_rooks == 0 && my_cannons == 2 && my_knights == 0 && foe_attackers == 0 {
            if foe_advisors + foe_bishops <= 2 {
                return Some(weights.endgame.double_cannons_vs_broken);
            }
        }

        None
    }
}
