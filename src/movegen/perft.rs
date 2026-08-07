// ============================================================================
// MODULE PERFT: THUẬT TOÁN ĐẾM NÚT CÂY NƯỚC ĐỊ PERFT (PERFORMANCE TEST)
// ============================================================================
// Perft (Performance Test) là công cụ thẩm định tiêu chuẩn trong lập trình AI Cờ Tướng.
// Thuật toán duyệt toàn bộ các nước đi hợp lệ theo dạng cây đệ quy tới độ sâu `depth` chỉ định
// và đếm tổng số lượng thế cờ thu được.
// Nếu kết quả đếm trùng khớp với các mốc quy chuẩn (như FEN khởi đầu: Depth 1 = 44, Depth 2 = 1,920),
// điều đó chứng minh bộ sinh nước đi (MoveGen) hoàn toàn không có lỗi logic hay sút rác bit.
// ============================================================================

use super::legal;
use super::types::List;
use crate::board::Position;

/// Hàm `perft` đếm tổng số nút cây nước đi hợp lệ đệ quy từ vị trí `pos` ở độ sâu `depth`.
/// Ép buộc inlining `#[inline(always)]` triệt tiêu chi phí gọi đệ quy trên Perft hot loop.
#[inline(always)]
pub fn perft(pos: &mut Position, depth: usize) -> u64 {
    // 1. Trường hợp cơ sở: Độ sâu 0 tương ứng với 1 thế cờ hiện tại
    if depth == 0 {
        return 1;
    }

    // 2. Tạo danh sách tĩnh `List` trên Stack (căn lề 64-byte) để chứa nước đi hợp lệ
    let mut list = List::new();
    legal::gen(pos, &mut list);

    // 3. Tối ưu hóa điều kiện dừng: Độ sâu 1 trả về trực tiếp số nước đi vừa sinh
    if depth == 1 {
        return list.count as u64;
    }

    // 4. Duyệt đệ quy qua tất cả các nước đi hợp lệ
    let mut nodes = 0u64;
    let mut i = 0;
    while i < list.count {
        let mv = list.items[i];
        // Thực hiện nước đi và lưu trạng thái để hoàn tác
        let state = pos.apply(mv.from, mv.to);
        // Gọi đệ quy cho cây con ở độ sâu (depth - 1)
        nodes += perft(pos, depth - 1);
        // Hoàn tác nước đi trả bàn cờ về trạng thái cũ
        pos.revert(mv.from, mv.to, &state);
        i += 1;
    }

    nodes
}

/// Hàm `divide` phân rã đếm nút chi tiết cho từng nước đi hợp lệ tại vị trí hiện tại.
/// Thường dùng cho UCI debug để so sánh từng nhánh nước đi với engine chuẩn như Pikafish.
#[inline(always)]
pub fn divide(pos: &mut Position, depth: usize) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut list = List::new();
    legal::gen(pos, &mut list);

    let mut total = 0u64;
    let mut i = 0;
    while i < list.count {
        let mv = list.items[i];
        // Thực hiện nước đi
        let state = pos.apply(mv.from, mv.to);
        // Đếm số nút của nhánh con
        let count = if depth == 1 { 1 } else { perft(pos, depth - 1) };
        // Hoàn tác nước đi
        pos.revert(mv.from, mv.to, &state);

        total += count;
        i += 1;
    }

    total
}

