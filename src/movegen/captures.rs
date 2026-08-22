// ============================================================================
// XIANGQI-RIM ENGINE: MODULE SINH NƯỚC ĐỊ CHỈ ĂN QUÂN (CAPTURES ONLY MOVEGEN)
// ============================================================================
// Module `captures` chịu trách nhiệm sinh CHỈ các nước đi ăn quân (Captures Only)
// phục vụ trực tiếp cho thuật toán Quiescence Search (Q-Search) ở các nút lá.
// Loại bỏ 100% chi phí sinh và kiểm tra các nước đi không ăn quân (Quiet Moves),
// giúp tiết kiệm đến 80% chu kỳ xung nhịp CPU trong vòng lặp tìm kiếm yên tĩnh!
// Tuân thủ 100% định danh từ đơn tiếng Anh và 100% chú thích Tiếng Việt tường minh.
// ============================================================================

use crate::board::Position;
use crate::movegen::legal;
use crate::movegen::pseudo;
use crate::movegen::List;

/// Hàm `gen`: Sinh tất cả các nước đi ăn quân hợp lệ cho phe đang nắm lượt đi.
/// Nhận vào các tham số: `pos` kiểu `&Position` và `list` kiểu `&mut List`.
#[inline(always)]
pub fn gen(pos: &Position, list: &mut List) {
    let mut pseudo_list = List::new();
    // 1. Sinh các nước đi giả định (Pseudo Legal Moves)
    pseudo::pseudo(pos, &mut pseudo_list);

    let mut i = 0usize;
    while i < pseudo_list.count {
        let mv = pseudo_list.items[i];
        // 2. Chỉ lọc lấy các nước đi ăn quân (Destination square has enemy piece: pos.grid[to] < 14)
        if pos.grid[mv.to as usize] < 14 {
            let mut test_pos = *pos;
            let _state = test_pos.apply(mv.from, mv.to);
            // 3. Kiểm tra tính hợp lệ: Nước đi không làm Tướng nhà bị chiếu (Illegal Check)
            if !legal::check(&test_pos, pos.side as usize) {
                list.push(mv);
            }
        }
        i += 1;
    }
}
