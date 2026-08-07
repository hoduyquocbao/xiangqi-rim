// ============================================================================
// MODULE TYPES: KIỂU DỮ LIỆU NƯỚC ĐỊ (MOVE) VÀ DANH SÁCH NƯỚC ĐỊ CỐ ĐỊNH (LIST)
// ============================================================================
// Trong Engine Cờ Tướng:
// - Cấu trúc `Move` gói gọn ô từ (`from: u8`) và ô tới (`to: u8`), chiếm đúng 2 bytes (16-bit).
// - Cấu trúc `List` chứa tối đa 128 nước đi (vượt trên số nước đi hợp lệ tối đa thực tế là 120),
//   được căn lề 64-byte (`#[repr(C, align(64))]`) để tối ưu hóa việc phân bổ tĩnh trên Stack.
// ============================================================================

use std::ops::Index;

/// Struct `Move` đại diện cho một nước đi gồm ô xuất phát (`from`) và ô đích (`to`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Move {
    /// Vị trí tuyến tính ô xuất phát (0..89) [1 byte]
    pub from: u8,
    /// Vị trí tuyến tính ô đích đến (0..89) [1 byte]
    pub to: u8,
}

impl Move {
    /// Khởi tạo một nước đi mới từ ô xuất phát `from` đến ô đích `to`.
    #[inline(always)]
    pub const fn new(from: u8, to: u8) -> Self {
        Self { from, to }
    }

    /// Khởi tạo một nước đi rỗng / không hợp lệ (đánh dấu ô 255).
    #[inline(always)]
    pub const fn none() -> Self {
        Self { from: 255, to: 255 }
    }

    /// Kiểm tra nước đi có nằm trong giới hạn bàn cờ 90 ô hợp lệ hay không.
    #[inline(always)]
    pub const fn valid(self) -> bool {
        self.from < 90 && self.to < 90 && self.from != self.to
    }

    /// Trả về số nguyên 16-bit (`u16`) mã hóa nước đi: `(from << 8) | to`.
    #[inline(always)]
    pub const fn raw(self) -> u16 {
        ((self.from as u16) << 8) | (self.to as u16)
    }
}

/// Struct `List` chứa danh sách các nước đi được sinh ra, căn lề bộ nhớ 64-byte.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct List {
    /// Mảng lưu trữ tối đa 128 nước đi cố định (không cấp phát bộ nhớ động Heap)
    pub items: [Move; 128],
    /// Số lượng nước đi thực tế đang có trong danh sách
    pub count: usize,
}

impl Default for List {
    /// Khởi tạo danh sách mặc định.
    fn default() -> Self {
        Self::new()
    }
}

impl List {
    /// Khởi tạo danh sách nước đi rỗng với bộ đếm = 0.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            items: [Move { from: 255, to: 255 }; 128],
            count: 0,
        }
    }

    /// Thêm một nước đi `item` vào cuối danh sách.
    #[inline(always)]
    pub fn push(&mut self, item: Move) {
        if self.count < 128 {
            self.items[self.count] = item;
            self.count += 1;
        }
    }

    /// Rút nước đi cuối cùng ra khỏi danh sách.
    #[inline(always)]
    pub fn pop(&mut self) -> Option<Move> {
        if self.count == 0 {
            None
        } else {
            self.count -= 1;
            Some(self.items[self.count])
        }
    }

    /// Xóa sạch toàn bộ nước đi trong danh sách (đặt lại bộ đếm về 0).
    #[inline(always)]
    pub fn clear(&mut self) {
        self.count = 0;
    }

    /// Trả về số lượng nước đi hiện có trong danh sách.
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.count
    }

    /// Kiểm tra danh sách có rỗng hay không.
    #[inline(always)]
    pub const fn empty(&self) -> bool {
        self.count == 0
    }

    /// Truy xuất nước đi tại vị trí chỉ số `index`.
    #[inline(always)]
    pub fn get(&self, index: usize) -> Move {
        if index < self.count {
            self.items[index]
        } else {
            Move::none()
        }
    }

    /// Gán lại giá trị nước đi tại chỉ số `index`.
    #[inline(always)]
    pub fn set(&mut self, index: usize, item: Move) {
        if index < self.count {
            self.items[index] = item;
        }
    }
}

/// Triển khai Trait `Index` cho phép truy xuất `list[index]` trực tiếp như mảng.
impl Index<usize> for List {
    type Output = Move;
    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}



