// ============================================================================
// MODULE STATE: TRẠNG THÁI MÁY NGẮT MẠCH BẢO VỆ (CIRCUIT BREAKER STATE)
// ============================================================================
// Định nghĩa enum `State` biểu diễn 3 trạng thái ngắt mạch điện tử:
// - `Closed = 0`: Mạch đóng - NNUE hoạt động bình thường.
// - `Half = 1`: Mạch bán mở (Half-Open) - Đang trong chu kỳ thử nghiệm phục hồi NNUE.
// - `Open = 2`: Mạch ngắt - NNUE gặp sự cố, tự động fallback ngắt sang HCE.
// ============================================================================

/// Enum `State` chiếm đúng 1 byte (`#[repr(u8)]`) trong bộ nhớ.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// Mạch đóng (Closed = 0): Hệ thống NNUE đang vận hành an toàn và ổn định
    Closed = 0,
    /// Mạch thử nghiệm (Half = 1): Hệ thống đang trong giai đoạn chạy thử nghiệm phục hồi
    Half = 1,
    /// Mạch ngắt (Open = 2): Hệ thống phát hiện lỗi bất thường và kích hoạt chế độ dự phòng HCE
    Open = 2,
}

impl State {
    /// Lấy giá trị nguyên 8-bit (`u8`) của trạng thái ngắt mạch để thao tác nguyên tử (Atomic operations).
    #[inline(always)]
    pub const fn raw(self) -> u8 {
        self as u8
    }

    /// Giải mã mã nguyên `val: u8` thành đối tượng enum `State` tương ứng.
    #[inline(always)]
    pub const fn parse(val: u8) -> Self {
        match val {
            0 => Self::Closed,
            1 => Self::Half,
            _ => Self::Open,
        }
    }
}

