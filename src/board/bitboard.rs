// ============================================================================
// MODULE BITBOARD: MẶT NẠ NGUYÊN TỬ 128-BIT ĐẠI DIỆN TẬP HỢP Ô BÀN CỜ
// ============================================================================
// Bàn cờ Cờ Tướng gồm 90 ô (từ ô 0 đến ô 89). Để biểu diễn các tập hợp vị trí
// (ví dụ: vị trí tất cả quân Xe, quân Mã, ô bị tấn công) bằng thao tác Bitwise
// song song tốc độ cực cao trên CPU, chúng ta sử dụng một số nguyên 128-bit `u128`.
//
// Đặc tính căn lề bộ nhớ `#[repr(C, align(16))]`:
// - Căn chỉnh địa chỉ bộ nhớ theo bội số của 16 bytes (128 bits).
// - Cho phép các lệnh SIMD (SSE2/AVX2/AVX-512/NEON) nạp dữ liệu trực tiếp
//   vào thanh ghi vector 128-bit (`__m128i` / `uint64x2_t`) với zero latency alignment overhead.
// ============================================================================

use super::square::Square;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

/// Struct `Bitboard` bao bọc một số nguyên không dấu 128-bit (`u128`).
/// Bit thứ $k$ (từ 0 đến 89) bật giá trị 1 thể hiện ô $k$ đang thuộc tập hợp.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Bitboard(pub u128);

impl Bitboard {
    /// Khởi tạo một Bitboard rỗng (tất cả 128 bit đều bằng 0).
    #[inline(always)]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Khởi tạo Bitboard chỉ có duy nhất 1 bit được bật tại vị trí ô `square`.
    /// Sử dụng phép dịch trái bit `1u128 << square.index()`.
    #[inline(always)]
    pub const fn mask(square: Square) -> Self {
        Self(1u128 << square.index())
    }

    /// Kiểm tra xem bit tại ô `square` có đang ở trạng thái bật (1) hay không.
    /// Trả về `true` nếu ô đó thuộc tập hợp, `false` nếu không.
    #[inline(always)]
    pub const fn test(self, square: Square) -> bool {
        (self.0 & (1u128 << square.index())) != 0
    }

    /// Bật bit tại vị trí ô `square` lên 1 (phép toán `bitor_assign` bít).
    #[inline(always)]
    pub fn set(&mut self, square: Square) {
        self.0 |= 1u128 << square.index();
    }

    /// Tắt bit tại vị trí ô `square` về 0 (phép toán `bitand_assign` với mặt nạ đảo).
    #[inline(always)]
    pub fn clear(&mut self, square: Square) {
        self.0 &= !(1u128 << square.index());
    }

    /// Đảo trạng thái bit tại ô `square` (0 thành 1, 1 thành 0 nhờ phép toán XOR `^`).
    #[inline(always)]
    pub fn toggle(&mut self, square: Square) {
        self.0 ^= 1u128 << square.index();
    }

    /// Trích xuất vị trí ô cờ của bit 1 thấp nhất (Least Significant Bit - LSB),
    /// đồng thời tắt bit đó đi khỏi Bitboard (thao tác pop bit tối ưu O(1)).
    ///
    /// Phép toán `self.0 & (self.0 - 1)` xóa bit 1 thấp nhất chỉ trong 1 chu kỳ CPU.
    /// Phép toán `trailing_zeros()` biên dịch trực tiếp thành lệnh phần cứng `TZCNT` / `CTZ`.
    #[inline(always)]
    pub fn pop(&mut self) -> Option<Square> {
        if self.0 == 0 {
            None
        } else {
            // Đếm số bit 0 đứng trước bit 1 đầu tiên từ phải sang trái
            let index = self.0.trailing_zeros() as u8;
            // Xóa bit 1 thấp nhất khỏi mặt nạ bitboard
            self.0 &= self.0 - 1;
            Some(Square(index))
        }
    }

    /// Trả về vị trí ô cờ của bit 1 thấp nhất (LSB) mà không sửa đổi dữ liệu Bitboard.
    #[inline(always)]
    pub fn lsb(self) -> Option<Square> {
        if self.0 == 0 {
            None
        } else {
            Some(Square(self.0.trailing_zeros() as u8))
        }
    }

    /// Trả về vị trí ô cờ của bit 1 cao nhất (Most Significant Bit - MSB).
    /// Phép toán `leading_zeros()` biên dịch thành lệnh phần cứng `LZCNT` / `CLZ`.
    #[inline(always)]
    pub fn msb(self) -> Option<Square> {
        if self.0 == 0 {
            None
        } else {
            // Vị trí MSB = 127 - số bit 0 ở đầu
            Some(Square(127 - self.0.leading_zeros() as u8))
        }
    }

    /// Đếm tổng số lượng bit 1 đang bật trong Bitboard (Popcount / Population Count).
    /// Biên dịch trực tiếp thành lệnh vi xử lý `POPCNT` trên x86_64 hoặc `VCNT` trên ARM NEON.
    #[inline(always)]
    pub fn count(self) -> u32 {
        self.0.count_ones()
    }

    /// Kiểm tra xem Bitboard có chứa bất kỳ bit 1 nào không (khác 0).
    #[inline(always)]
    pub const fn active(self) -> bool {
        self.0 != 0
    }
}

// ----------------------------------------------------------------------------
// TRIỂN KHAI CÁC TOÁN TỬ BITWISE CHO BITBOARD
// ----------------------------------------------------------------------------

/// Phép toán GIAO hai tập hợp Bitboard (`self & rhs`)
impl BitAnd for Bitboard {
    type Output = Self;
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

/// Phép toán HỢP hai tập hợp Bitboard (`self | rhs`)
impl BitOr for Bitboard {
    type Output = Self;
    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// Phép toán HIỆU BẤT ĐỐI XỨNG / XOR hai tập hợp Bitboard (`self ^ rhs`)
impl BitXor for Bitboard {
    type Output = Self;
    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}

/// Phép toán ĐẢO BÍT / PHỦ ĐỊNH tập hợp Bitboard (`!self`)
impl Not for Bitboard {
    type Output = Self;
    #[inline(always)]
    fn not(self) -> Self {
        Self(!self.0)
    }
}

/// Phép gán kết hợp GIAO BÍT (`self &= rhs`)
impl BitAndAssign for Bitboard {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

/// Phép gán kết hợp HỢP BÍT (`self |= rhs`)
impl BitOrAssign for Bitboard {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Phép gán kết hợp XOR BÍT (`self ^= rhs`)
impl BitXorAssign for Bitboard {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

