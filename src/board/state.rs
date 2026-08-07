// ============================================================================
// MODULE STATE: LƯU VẾT TRẠNG THÁI PHỤC VỤ HOÀN TÁC NƯỚC ĐỊ (UNDO MOVE)
// ============================================================================
// Cấu trúc `State` chiếm đúng 16 bytes trong bộ nhớ với thuộc tính `#[repr(C, align(16))]`.
// Căn chỉnh 16 bytes cho phép các lệnh sao chép SIMD/Memory nạp xào dữ liệu cực nhanh
// trong quá trình tìm kiếm độ sâu lớn (Pruning / Extension / Quiescence Search).
// ============================================================================

/// Struct `State` lưu vết các thuộc tính đã bị thay đổi khi MakeMove để UndoMove phục hồi O(1).
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct State {
    /// Mã quân cờ bị ăn ở nước đi này (0..13, 14 là ô trống/không ăn quân) [1 byte]
    pub captured: u8,
    /// Mã quân cờ bị ăn ở nước đi liền trước đó [1 byte]
    pub prev: u8,
    /// Trạng thái chiếu tướng tại thời điểm này (1: Đang bị chiếu, 0: Bình thường) [1 byte]
    pub check: u8,
    /// Byte đệm padding đảm bảo căn lề trường dữ liệu u16 [1 byte]
    pub pad: u8,
    /// Giá trị bộ đếm rule50 trước nước đi [2 bytes]
    pub rule: u16,
    /// Số ply (nửa nước đi) từ đầu trận [2 bytes]
    pub ply: u16,
    /// Khóa băm Zobrist hash 64-bit của vị trí bàn cờ trước nước đi [8 bytes]
    pub hash: u64,
}

impl State {
    /// Khởi tạo một đối tượng `State` mới lưu vết với đầy đủ các tham số truyền vào.
    #[inline(always)]
    pub const fn new(captured: u8, prev: u8, check: u8, rule: u16, ply: u16, hash: u64) -> Self {
        Self {
            captured,
            prev,
            check,
            rule,
            ply,
            pad: 0,
            hash,
        }
    }

    /// Khởi tạo một đối tượng `State` rỗng mặc định.
    #[inline(always)]
    pub const fn empty() -> Self {
        Self {
            captured: 14,
            prev: 14,
            check: 0,
            rule: 0,
            ply: 1,
            pad: 0,
            hash: 0,
        }
    }
}

