// ============================================================================
// MODULE BOUND: CỜ CẬN ĐIỂM SỐ ALPHA-BETA NGUYÊN TỬ (TRANSPOSITION BOUND FLAG)
// ============================================================================
// Định nghĩa enum `Bound` biểu diễn 4 loại cờ ranh giới điểm số lưu trong Transposition Table:
// - `None = 0`: Chưa có dữ liệu ranh giới.
// - `Exact = 1`: Điểm chính xác (PV Node score - nằm trong khoảng Alpha < score < Beta).
// - `Lower = 2`: Cận dưới (Fail-High / Beta Cutoff score - score >= Beta).
// - `Upper = 3`: Cận trên (Fail-Low / Alpha Cutoff score - score <= Alpha).
// ============================================================================

/// Enum `Bound` chiếm 1 byte (`#[repr(u8)]`) đại diện cho cờ ranh giới điểm số Alpha-Beta.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bound {
    /// Rỗng (None = 0): Ô nhớ chưa chứa điểm số ranh giới hợp lệ
    None = 0,
    /// Chính xác (Exact = 1): Điểm số chính xác thu được từ PV Node
    Exact = 1,
    /// Cận dưới (Lower = 2): Điểm cắt Beta (Beta Cutoff - điểm số thực tế >= score)
    Lower = 2,
    /// Cận trên (Upper = 3): Điểm thất bại Alpha (Alpha Cutoff - điểm số thực tế <= score)
    Upper = 3,
}

impl Bound {
    /// Trả về số nguyên 8-bit (`u8`) của cờ ranh giới để nạp vào trường bitwise TTEntry.
    #[inline(always)]
    pub const fn raw(self) -> u8 {
        self as u8
    }

    /// Giải mã số nguyên `val: u8` thành đối tượng enum `Bound` tương ứng.
    #[inline(always)]
    pub const fn parse(val: u8) -> Self {
        match val {
            1 => Self::Exact,
            2 => Self::Lower,
            3 => Self::Upper,
            _ => Self::None,
        }
    }
}

