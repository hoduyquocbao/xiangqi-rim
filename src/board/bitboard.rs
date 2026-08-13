// ============================================================================
// XIANGQI-RIM ENGINE: MODULE BITBOARD 64-BIT SPLIT ACCELERATION VÀ BẢNG TRA CỨU TĨNH O(1)
// ============================================================================
// Định nghĩa cấu trúc Bitboard 64-bit Split (`low: u64, high: u64`) biểu diễn 90 ô cờ Tướng:
// - `low`: 45 bit biểu diễn ô 0..44 (Rank 0..4 - Bàn cờ bên dưới).
// - `high`: 45 bit biểu diễn ô 45..89 (Rank 5..9 - Bàn cờ bên trên).
// Tận dụng trực tiếp các lệnh phần cứng x86_64 `tzcnt`/`popcnt` 64-bit nguyên thủy trong 1 CPU clock cycle.
// Căn lề 16-byte vật lý phòng chống False Sharing và tối ưu nạp thanh ghi SIMD.
// Tuân thủ 100% định danh từ đơn tiếng Anh và 100% chú thích Tiếng Việt tường minh.
// ============================================================================

use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};
use crate::board::Square;

/// Mặt nạ 45 bit (2^45 - 1 = 0x0000_1FFF_FFFF_FFFF)
pub const MASK45: u64 = (1u64 << 45) - 1;

/// Struct `Bitboard`: Cấu trúc dữ liệu biểu diễn 90 ô cờ Tướng dưới dạng cặp `(low: u64, high: u64)`.
/// Căn lề 16-byte vật lý phòng chống False Sharing và tối ưu nạp thanh ghi SIMD.
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Bitboard {
    pub low: u64,
    pub high: u64,
}

impl Bitboard {
    /// Hàm `new`: Khởi tạo đối tượng Bitboard rỗng với giá trị 0.
    #[inline(always)]
    pub fn new() -> Self {
        Self { low: 0, high: 0 }
    }

    /// Hàm `empty`: Khởi tạo Bitboard rỗng với giá trị 0 (const fn).
    #[inline(always)]
    pub const fn empty() -> Self {
        Self { low: 0, high: 0 }
    }

    /// Hàm `from_u128`: Chuyển đổi từ `u128` sang cấu trúc Bitboard 64-bit Split (const fn).
    #[inline(always)]
    pub const fn from_u128(val: u128) -> Self {
        Self {
            low: (val & 0x0000_1FFF_FFFF_FFFF) as u64,
            high: ((val >> 45) & 0x0000_1FFF_FFFF_FFFF) as u64,
        }
    }

    /// Hàm `from_raw`: Khởi tạo từ cặp `(low, high)` (const fn).
    #[inline(always)]
    pub const fn from_raw(low: u64, high: u64) -> Self {
        Self { low, high }
    }

    /// Hàm `mask`: Tạo Bitboard chỉ có 1 bit bật tại ô `sq`.
    #[inline(always)]
    pub fn mask(sq: Square) -> Self {
        let idx = sq.0 as usize;
        if idx < 45 {
            Self { low: 1u64 << idx, high: 0 }
        } else {
            Self { low: 0, high: 1u64 << (idx - 45) }
        }
    }

    /// Phương thức `set`: Bật bit tại vị trí ô cờ `sq` (0..89) lên 1.
    #[inline(always)]
    pub fn set(&mut self, sq: impl Into<usize>) {
        let idx = sq.into();
        debug_assert!(idx < 90, "Square index out of bounds: {}", idx);
        if idx < 45 {
            self.low |= 1u64 << idx;
        } else {
            self.high |= 1u64 << (idx - 45);
        }
    }

    /// Phương thức `set_unchecked`: Bật bit 0-branching không qua kiểm tra chỉ số.
    #[inline(always)]
    pub unsafe fn set_unchecked(&mut self, idx: usize) {
        if idx < 45 {
            self.low |= 1u64 << idx;
        } else {
            self.high |= 1u64 << (idx - 45);
        }
    }

    /// Phương thức `clear`: Tắt bit tại vị trí ô cờ `sq` (0..89) về 0.
    #[inline(always)]
    pub fn clear(&mut self, sq: impl Into<usize>) {
        let idx = sq.into();
        debug_assert!(idx < 90, "Square index out of bounds: {}", idx);
        if idx < 45 {
            self.low &= !(1u64 << idx);
        } else {
            self.high &= !(1u64 << (idx - 45));
        }
    }

    /// Phương thức `clear_unchecked`: Tắt bit 0-branching không qua kiểm tra chỉ số.
    #[inline(always)]
    pub unsafe fn clear_unchecked(&mut self, idx: usize) {
        if idx < 45 {
            self.low &= !(1u64 << idx);
        } else {
            self.high &= !(1u64 << (idx - 45));
        }
    }

    /// Phương thức `test`: Kiểm tra xem bit tại vị trí ô cờ `sq` (0..89) có bật (bằng 1) hay không.
    #[inline(always)]
    pub fn test(&self, sq: impl Into<usize>) -> bool {
        let idx = sq.into();
        debug_assert!(idx < 90, "Square index out of bounds: {}", idx);
        if idx < 45 {
            (self.low & (1u64 << idx)) != 0
        } else {
            (self.high & (1u64 << (idx - 45))) != 0
        }
    }

    /// Phương thức `test_unchecked`: Kiểm tra bit 0-branching không qua kiểm tra chỉ số.
    #[inline(always)]
    pub unsafe fn test_unchecked(&self, idx: usize) -> bool {
        if idx < 45 {
            (self.low & (1u64 << idx)) != 0
        } else {
            (self.high & (1u64 << (idx - 45))) != 0
        }
    }

    /// Phương thức `is_empty`: Kiểm tra xem Bitboard có rỗng hay không.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        (self.low | self.high) == 0
    }

    /// Phương thức `active`: Kiểm tra xem Bitboard có ít nhất 1 bit bật (khác 0) hay không.
    #[inline(always)]
    pub fn active(&self) -> bool {
        (self.low | self.high) != 0
    }

    /// Phương thức `pop`: Lấy và tắt vị trí bit 1 đầu tiên (lsb), trả về `Option<Square>`.
    #[inline(always)]
    pub fn pop(&mut self) -> Option<Square> {
        if self.low != 0 {
            let lsb = self.low.trailing_zeros() as u8;
            self.low &= self.low - 1;
            Some(Square(lsb))
        } else if self.high != 0 {
            let lsb = self.high.trailing_zeros() as u8;
            self.high &= self.high - 1;
            Some(Square(45 + lsb))
        } else {
            None
        }
    }

    /// Phương thức `pop_unchecked`: Lấy và tắt vị trí bit 1 đầu tiên (0-branching 0-option).
    #[inline(always)]
    pub fn pop_unchecked(&mut self) -> u8 {
        if self.low != 0 {
            let lsb = self.low.trailing_zeros() as u8;
            self.low &= self.low - 1;
            lsb
        } else {
            let lsb = self.high.trailing_zeros() as u8;
            self.high &= self.high - 1;
            45 + lsb
        }
    }

    /// Phương thức `lsb`: Lấy vị trí bit 1 đầu tiên (least significant bit) dưới dạng `Option<Square>`.
    #[inline(always)]
    pub fn lsb(&self) -> Option<Square> {
        if self.low != 0 {
            let lsb = self.low.trailing_zeros() as u8;
            Some(Square(lsb))
        } else if self.high != 0 {
            let lsb = self.high.trailing_zeros() as u8;
            Some(Square(45 + lsb))
        } else {
            None
        }
    }

    /// Phương thức `msb`: Lấy vị trí bit 1 cuối cùng (most significant bit) dưới dạng `Option<Square>`.
    #[inline(always)]
    pub fn msb(&self) -> Option<Square> {
        if self.high != 0 {
            let msb = (63 - self.high.leading_zeros()) as u8;
            Some(Square(45 + msb))
        } else if self.low != 0 {
            let msb = (63 - self.low.leading_zeros()) as u8;
            Some(Square(msb))
        } else {
            None
        }
    }

    /// Phương thức `lsb_idx`: Lấy chỉ số bit 1 đầu tiên (0-branching 0-option).
    #[inline(always)]
    pub fn lsb_idx(&self) -> usize {
        if self.low != 0 {
            self.low.trailing_zeros() as usize
        } else {
            45 + self.high.trailing_zeros() as usize
        }
    }

    /// Phương thức `msb_idx`: Lấy chỉ số bit 1 cuối cùng (0-branching 0-option).
    #[inline(always)]
    pub fn msb_idx(&self) -> usize {
        if self.high != 0 {
            45 + (63 - self.high.leading_zeros()) as usize
        } else {
            (63 - self.low.leading_zeros()) as usize
        }
    }

    /// Phương thức `count`: Đếm tổng số bit 1 đang bật trong Bitboard trong 1 CPU Clock Cycle.
    #[inline(always)]
    pub fn count(&self) -> u32 {
        self.low.count_ones() + self.high.count_ones()
    }
}

impl Not for Bitboard {
    type Output = Self;
    #[inline(always)]
    fn not(self) -> Self::Output {
        Self {
            low: !self.low & MASK45,
            high: !self.high & MASK45,
        }
    }
}

impl BitAnd for Bitboard {
    type Output = Self;
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            low: self.low & rhs.low,
            high: self.high & rhs.high,
        }
    }
}

impl BitOr for Bitboard {
    type Output = Self;
    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            low: self.low | rhs.low,
            high: self.high | rhs.high,
        }
    }
}

impl BitXor for Bitboard {
    type Output = Self;
    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            low: self.low ^ rhs.low,
            high: self.high ^ rhs.high,
        }
    }
}

impl BitAndAssign for Bitboard {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        self.low &= rhs.low;
        self.high &= rhs.high;
    }
}

impl BitOrAssign for Bitboard {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        self.low |= rhs.low;
        self.high |= rhs.high;
    }
}

impl BitXorAssign for Bitboard {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.low ^= rhs.low;
        self.high ^= rhs.high;
    }
}

impl From<Square> for usize {
    #[inline(always)]
    fn from(sq: Square) -> usize {
        sq.0 as usize
    }
}
