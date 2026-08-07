// ============================================================================
// MODULE CONFIG: THÔNG SỐ CẤU HÌNH NGƯỠNG AN TOÀN CHO MÁY NGẮT MẠCH (ALIGN 64)
// ============================================================================
// Cấu trúc `Config` chứa các ngưỡng ngắt và khôi phục hoạt động cho CircuitBreaker.
// Căn lề bộ nhớ `#[repr(C, align(64))]` chiếm đúng 64 bytes (1 L1 Cache line).
// ============================================================================

/// Struct `Config` quản lý cấu hình ranh giới điểm số và tần suất ngắt mạch.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct Config {
    /// Giới hạn số lần lỗi tối đa cho phép trước khi ngắt mạch (Mặc định: 5 lần liên tiếp) [4 bytes]
    pub limit: u32,
    /// Khoảng thời gian chờ ngắt mạch tính bằng mili-giây (Mặc định: 10,000 ms = 10 giây) [8 bytes]
    pub span: u64,
    /// Số lượng phép thử nghiệm thành công cần thiết để phục hồi mạch (Mặc định: 100 lần) [4 bytes]
    pub probe: u32,
    /// Điểm sàn tối thiểu hợp lệ của bàn cờ (-29,999 centipawns) [4 bytes]
    pub floor: i32,
    /// Điểm trần tối đa hợp lệ của bàn cờ (+29,999 centipawns) [4 bytes]
    pub ceiling: i32,
    /// Mảng đệm padding đảm bảo kích thước struct tròn đúng 64 bytes (1 L1 Cache Line) [36 bytes]
    pub pad: [u8; 36],
}

impl Default for Config {
    /// Khởi tạo cấu hình mặc định.
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    /// Khởi tạo hằng số đối tượng `Config` mới với các thông số an toàn chuẩn.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            limit: 5,
            span: 10000,
            probe: 100,
            floor: -29999,
            ceiling: 29999,
            pad: [0; 36],
        }
    }
}

