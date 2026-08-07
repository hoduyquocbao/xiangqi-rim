// ============================================================================
// MODULE CHECK: THẨM ĐỊNH TÍNH HỢP LỆ ĐIỂM SỐ ĐÁNH GIÁ (SCORE VALIDATOR)
// ============================================================================
// Cung cấp các phương thức kiểm tra điểm số thế cờ từ NNUE trả về nằm trong
// ranh giới cho phép $[floor, ceiling]$, loại bỏ các điểm số nổ dị thường (NaN/Overflow).
// ============================================================================

/// Struct `Check` chứa các hàm thẩm định logic không có trạng thái (Stateless helper).
pub struct Check;

impl Check {
    /// Kiểm tra điểm số `score` có nằm trong khoảng hợp lệ $[floor, ceiling]$ hay không.
    /// - Trả về `true`: Điểm hợp lệ.
    /// - Trả về `false`: Điểm dị thường, kích hoạt máy ngắt mạch CircuitBreaker.
    #[inline(always)]
    pub fn valid(score: i32, floor: i32, ceiling: i32) -> bool {
        score >= floor && score <= ceiling
    }
}

