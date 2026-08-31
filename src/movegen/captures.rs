// ============================================================================
// XIANGQI-RIM ENGINE: MODULE SINH NƯỚC ĐI CHỈ ĂN QUÂN (CAPTURES ONLY MOVEGEN)
// ============================================================================
// Module `captures` chịu trách nhiệm sinh CHỈ các nước đi ăn quân (Captures Only)
// phục vụ trực tiếp cho thuật toán Quiescence Search (Q-Search) ở các nút lá.
// - Triệt tiêu 100% chi phí sao chép struct bộ nhớ bằng cơ chế `apply/revert` in-place.
// - Kiểm tra đầy đủ 100% luật cờ tướng: Cấm tự chiếu (`!check`) VÀ Cấm lộ mặt Tướng (`!fly`).
// - Tuân thủ 100% định danh từ đơn tiếng Anh và 100% chú thích Tiếng Việt tường minh.
// ============================================================================

use crate::board::Position;
use crate::movegen::legal;
use crate::movegen::List;

/// Hàm `gen`: Sinh tất cả các nước đi ăn quân hợp lệ cho phe đang nắm lượt đi.
/// Nhận vào các tham số: `pos` kiểu `&mut Position` và `list` kiểu `&mut List`.
#[inline(always)]
pub fn gen(pos: &mut Position, list: &mut List) {
    let mut pseudo_list = List::new();
    // 1. Sinh các nước đi giả định CHỈ ĂN QUÂN bằng Bitboard PEXT O(1)
    crate::movegen::bitboard_movegen::BitboardMoveGen::generate_captures(pos, &mut pseudo_list);

    let side = pos.side as usize;
    let mut i = 0usize;
    while i < pseudo_list.count {
        let mv = pseudo_list.items[i];
        let state = pos.apply(mv.from, mv.to);
        // 2. Kiểm tra tính hợp lệ tuyệt đối: Cấm tự chiếu (Check) VÀ Cấm lộ mặt Tướng (Fly)
        if !legal::check(pos, side) && !legal::fly(pos) {
            list.push(mv);
        }
        pos.revert(mv.from, mv.to, &state);
        i += 1;
    }
}
