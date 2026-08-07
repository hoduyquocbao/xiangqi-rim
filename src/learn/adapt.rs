// ============================================================================
// MODULE LEARN ADAPT: TỐI ƯU PHƯƠNG TRÌNH VÀ GIỚI HẠN TÌM KIẾM THÍCH ỨNG (ADAPTIVE SEARCH)
// ============================================================================
// Module `adapt` triển khai các công thức tự điều chỉnh giới hạn tìm kiếm (Search Limits):
// 1. Board Complexity (C_board): Tính độ rối rắm thế cờ dựa trên số nước đi/chiếu/ăn quân.
// 2. PV Stability (S_pv): Đo độ ổn định biến thể chính Principal Variation qua các độ sâu.
// 3. Dynamic Aspiration Window (Delta_asp): Nới rộng/thu hẹp biên độ cửa sổ Aspiration Window.
// 4. Adaptive LMR (R_adaptive): Tự điều chỉnh mức cắt giảm độ sâu LMR.
// Thiết kế 100% Clean Room std-only, căn lề bộ nhớ 64-byte loại bỏ False Sharing,
// và tuân thủ tuyệt đối Quy tắc Định danh Đơn Từ Tiếng Anh (Single-Word Principle).
// ============================================================================

/// Struct `Adapt` quản lý các thông số suy luận thích ứng (align 64).
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Adapt {
    /// Chỉ số độ phức tạp thế cờ C_board (1.0 đến 5.0)
    pub complexity: f32,
    /// Chỉ số độ ổn định tuyến PV S_pv (0.0 đến 1.0)
    pub stability: f32,
    /// Biên độ cửa sổ Aspiration Window hiện tại (16, 32, hoặc 64 centipawns)
    pub window: i32,
    /// Mảng đệm căn lề 52-byte đảm bảo struct đạt chuẩn 64 bytes
    pub pad: [u8; 52],
}

impl Adapt {
    /// Khởi tạo bộ thích ứng `Adapt` mặc định.
    pub fn new() -> Self {
        Self {
            complexity: 1.0,
            stability: 1.0,
            window: 16,
            pad: [0u8; 52],
        }
    }

    /// Tính độ phức tạp thế cờ C_board dựa trên số nước hợp lệ `legal`, nước chiếu `checks`, và nước ăn quân `captures`.
    pub fn board(legal: usize, checks: usize, captures: usize) -> f32 {
        let score = (legal as f32) / 12.0 + (checks as f32) / 2.0 + (captures as f32) / 4.0;
        score.clamp(1.0, 5.0)
    }

    /// Tính chỉ số độ ổn định tuyến PV S_pv qua mảng nước đi tốt nhất `moves` ở các độ sâu.
    pub fn pv(moves: &[u16]) -> f32 {
        if moves.len() <= 1 {
            return 1.0;
        }

        let mut matches = 0usize;
        for i in 1..moves.len() {
            if moves[i] == moves[i - 1] {
                matches += 1;
            }
        }

        (matches as f32) / ((moves.len() - 1) as f32)
    }

    /// Đánh giá biên độ cửa sổ Aspiration Window thích ứng dựa trên chỉ số ổn định `stability`.
    pub fn window(stability: f32) -> i32 {
        if stability >= 0.75 {
            16
        } else if stability >= 0.40 {
            32
        } else {
            64
        }
    }

    /// Tính mức cắt giảm độ sâu LMR thích ứng dựa trên mức LMR cơ bản `base`, độ ổn định `stability`, và điểm phạt `penalty`.
    pub fn lmr(base: i32, stability: f32, penalty: i32) -> i32 {
        let mut r = base;
        if stability < 0.75 {
            r -= ((1.0 - stability) * 2.0) as i32;
        }
        if penalty > 0 {
            r -= 1;
        }
        r.max(0)
    }

    /// Cập nhật tổng thể các chỉ số thích ứng.
    pub fn update(&mut self, legal: usize, checks: usize, captures: usize, moves: &[u16]) {
        self.complexity = Self::board(legal, checks, captures);
        self.stability = Self::pv(moves);
        self.window = Self::window(self.stability);
    }
}

impl Default for Adapt {
    fn default() -> Self {
        Self::new()
    }
}

// ----------------------------------------------------------------------------
// KHU VỰC BÀI KIỂM THỬ ĐƠN VỊ (UNIT TESTS) CHO ADAPTIVE SEARCH
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// Kiểm thử căn lề bộ nhớ và dung lượng struct
    #[test]
    fn alignments() {
        assert_eq!(std::mem::align_of::<Adapt>(), 64);
        assert_eq!(std::mem::size_of::<Adapt>(), 64);
    }

    /// Kiểm thử tính toán chỉ số độ phức tạp thế cờ
    #[test]
    fn board() {
        let c1 = Adapt::board(12, 0, 0);
        assert_eq!(c1, 1.0);

        let c2 = Adapt::board(36, 4, 8);
        assert!(c2 > 3.0);
    }

    /// Kiểm thử độ ổn định PV và window thích ứng
    #[test]
    fn window() {
        let moves = [42u16, 42u16, 42u16, 42u16];
        let s = Adapt::pv(&moves);
        assert_eq!(s, 1.0);
        assert_eq!(Adapt::window(s), 16);

        let unstable = [42u16, 100u16, 200u16, 300u16];
        let su = Adapt::pv(&unstable);
        assert_eq!(su, 0.0);
        assert_eq!(Adapt::window(su), 64);
    }
}
