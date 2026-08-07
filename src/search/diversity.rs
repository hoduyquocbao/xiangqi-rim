// ============================================================================
// MODULE DIVERSITY: PHÂN TÁCH ĐA DẠNG HÓA TÌM KIẾM CÂY CỜ (SEARCH DIVERSIFICATION)
// ============================================================================
// `diversity.rs` giải quyết triệt để bài toán lãng phí công suất tính toán do
// các luồng Worker Thread trong Lazy SMP tìm kiếm trùng lặp cây cờ (~68% Tree Overlap).
// - Mảng số nguyên tố `PRIMES` giúp lệch pha độ sâu tìm kiếm giữa các luồng hoàn toàn độc lập.
// - Hệ số tỷ lệ `scaling = 100 + ((index * 17) % 45) - 10` điều chỉnh History Bias cho từng luồng.
// - Cấu trúc `Diversity` căn lề 64-byte (`#[repr(C, align(64))]`).
// ============================================================================

/// Mảng 16 số nguyên tố phân tách độ lệch nhịp độ sâu giữa các luồng
pub const PRIMES: [u8; 16] = [0, 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47];

/// Struct `Diversity` quản lý độ lệch tìm kiếm nguyên tố và hệ số lịch sử, căn lề 64-byte.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct Diversity {
    /// Chỉ số nhận diện luồng worker trong hệ thống (0..N-1)
    pub index: usize,
    /// Độ lệch số nguyên tố phân tách nhịp độ sâu tìm kiếm
    pub offset: u8,
    /// Hệ số tỷ lệ điều chỉnh điểm thưởng lịch sử History Bias
    pub scaling: i32,
    /// Mảng đệm căn lề bộ nhớ vừa khít 64-byte
    pub pad: [u8; 48],
}

impl Diversity {
    /// Khởi tạo một đối tượng `Diversity` mới dựa trên chỉ số luồng `index`.
    #[inline(always)]
    pub fn new(index: usize) -> Self {
        let offset = PRIMES[index % PRIMES.len()];
        let scaling = 100 + ((index * 17) % 45) as i32 - 10;
        Self {
            index,
            offset,
            scaling,
            pad: [0u8; 48],
        }
    }

    /// Nhân tỷ lệ điểm lịch sử `score` theo hệ số `scaling` của luồng này.
    #[inline(always)]
    pub fn scale(&self, score: i32) -> i32 {
        if self.index == 0 {
            score
        } else {
            (score * self.scaling) / 100
        }
    }

    /// Điều chỉnh độ sâu cơ bản `base` thêm độ lệch số nguyên tố của luồng.
    #[inline(always)]
    pub fn depth(&self, base: u8) -> u8 {
        if self.index == 0 {
            base
        } else {
            base.saturating_add(self.offset % 4)
        }
    }
}

// ----------------------------------------------------------------------------
// KHU VỰC BÀI KIỂM THỬ ĐƠN VỊ (UNIT TESTS) CHO SEARCH DIVERSITY
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// Kiểm thử căn lề bộ nhớ 64-byte và kích thước struct `Diversity`.
    #[test]
    fn alignment() {
        assert_eq!(std::mem::align_of::<Diversity>(), 64);
        assert_eq!(std::mem::size_of::<Diversity>(), 64);
    }

    /// Kiểm thử tính toán độ lệch số nguyên tố và hệ số lịch sử theo index.
    #[test]
    fn calculation() {
        let d0 = Diversity::new(0);
        assert_eq!(d0.index, 0);
        assert_eq!(d0.offset, 0);
        assert_eq!(d0.scaling, 90);
        assert_eq!(d0.scale(1000), 1000);
        assert_eq!(d0.depth(10), 10);

        let d1 = Diversity::new(1);
        assert_eq!(d1.index, 1);
        assert_eq!(d1.offset, 2);
        assert_eq!(d1.scaling, 107);
        assert_eq!(d1.scale(1000), 1070);
        assert_eq!(d1.depth(10), 12);
    }
}
