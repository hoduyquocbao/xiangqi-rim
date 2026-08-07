// ============================================================================
// MODULE ITEM: CẤU TRÚC KẾT QUẢ TRA CỨU ĐÃ GIẢI MÃ (DECODED TRANSPOSITION ITEM)
// ============================================================================
// Cấu trúc `Item` chứa thông tin đầy đủ sau khi đã giải mã từ 2 trường nguyên tử
// 64-bit (`data` và `info`) của `TTEntry`. Được trả về từ thao tác `probe(key)`.
// ============================================================================

use crate::movegen::types::Move;
use crate::tt::bound::Bound;

/// Struct `Item` gói gọn các trường dữ liệu kết quả tra cứu Transposition Table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Item {
    /// Khóa băm Zobrist hash 64-bit đầy đủ của vị trí bàn cờ
    pub key: u64,
    /// Độ sâu tìm kiếm đã đạt được tại vị trí này (depth: 0..255)
    pub depth: u8,
    /// Loại cờ ranh giới điểm số Alpha-Beta (Exact, Lower, Upper, None)
    pub bound: Bound,
    /// Nước đi tốt nhất (Best Move) được ghi nhận tại vị trí này
    pub step: Move,
    /// Điểm số thế cờ được đánh giá (Centipawn score)
    pub score: i16,
    /// Tuổi của Entry (Age - dùng cho chính sách thay thế khi cụm bị đầy)
    pub age: u8,
}

impl Item {
    /// Khởi tạo một đối tượng `Item` mới với đầy đủ các trường dữ liệu.
    #[inline(always)]
    pub const fn new(
        key: u64,
        depth: u8,
        bound: Bound,
        step: Move,
        score: i16,
        age: u8,
    ) -> Self {
        Self {
            key,
            depth,
            bound,
            step,
            score,
            age,
        }
    }

    /// Khởi tạo một đối tượng `Item` rỗng mặc định đại diện cho việc tra cứu thất bại (Cache Miss).
    #[inline(always)]
    pub const fn empty() -> Self {
        Self {
            key: 0,
            depth: 0,
            bound: Bound::None,
            step: Move::none(),
            score: 0,
            age: 0,
        }
    }
}

