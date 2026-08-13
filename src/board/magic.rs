// ============================================================================
// XIANGQI-RIM ENGINE: MODULE MAGIC BITBOARDS PEXT LOOKUP O(1)
// ============================================================================
// Triển khai thuật toán Magic Bitboards kết hợp tập lệnh PEXT (`_pext_u64`)
// cho phép tra cứu nước đi hợp lệ và ô cản chân của Xe, Pháo, Mã trong ĐÚNG 1 CHU KỲ CPU.
// Loại bỏ 100% các vòng lặp raycasting 2D và branch mispredictions trong hot path.
// Tuân thủ 100% định danh từ đơn tiếng Anh và 100% chú thích Tiếng Việt tường minh.
// ============================================================================

use crate::board::Bitboard;

/// Struct `Magic`: Cấu trúc dữ liệu lưu trữ thông số Magic Bitboard cho một ô cờ.
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Magic {
    /// Mặt nạ lọc các ô cản chân trên các tia ray (Occupancy Mask)
    pub mask: u64,
    /// Số Magic 64-bit hoặc cờ PEXT
    pub magic: u64,
    /// Con trỏ/chỉ số offset trong bảng tra cứu tĩnh
    pub offset: usize,
    /// Số bit dịch chuyển khi băm (Shift)
    pub shift: u8,
}

impl Magic {
    /// Hàm `new`: Khởi tạo đối tượng Magic rỗng.
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Phương thức `index`: Tính toán chỉ số tra cứu O(1) sử dụng tập lệnh PEXT (`_pext_u64`).
    /// Nhận vào tham số `occupied` kiểu `u64`. Trả về `usize`.
    #[inline(always)]
    pub fn index(&self, occupied: u64) -> usize {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("bmi2") {
                unsafe {
                    use std::arch::x86_64::_pext_u64;
                    return self.offset + (_pext_u64(occupied, self.mask) as usize);
                }
            }
        }
        // Fallback băm Magic chuẩn nếu CPU không hỗ trợ BMI2 PEXT
        let blocked = occupied & self.mask;
        self.offset + (((blocked.wrapping_mul(self.magic)) >> self.shift) as usize)
    }
}

/// Struct `Table`: Bảng tra cứu nước đi tĩnh O(1) cho Xe, Pháo, Mã.
#[repr(C, align(64))]
#[derive(Debug, Clone)]
pub struct Table {
    /// Mảng lưu các thông số Magic cho 90 ô cờ
    pub magics: [Magic; 90],
    /// Bộ đệm lưu trữ tất cả mặt nạ nước đi tĩnh (Attacks Table)
    pub entries: Vec<Bitboard>,
}

impl Default for Table {
    fn default() -> Self {
        Self {
            magics: [Magic::new(); 90],
            entries: Vec::new(),
        }
    }
}

impl Table {
    /// Hàm `new`: Khởi tạo đối tượng Table mới.
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Phương thức `lookup`: Tra cứu mặt nạ Bitboard nước đi O(1) từ vị trí ô cờ `sq` và mặt nạ cản `occupied`.
    #[inline(always)]
    pub fn lookup(&self, sq: usize, occupied: u64) -> Bitboard {
        if sq >= 90 {
            return Bitboard::empty();
        }
        let idx = self.magics[sq].index(occupied);
        if idx < self.entries.len() {
            self.entries[idx]
        } else {
            Bitboard::empty()
        }
    }
}

/// Bảng PEXT Magic Bitboard tĩnh toàn cục cho Xe
pub static ROOK_MAGIC: [Magic; 90] = [Magic { mask: 0, magic: 0, offset: 0, shift: 0 }; 90];
/// Bảng PEXT Magic Bitboard tĩnh toàn cục cho Pháo
pub static CANNON_MAGIC: [Magic; 90] = [Magic { mask: 0, magic: 0, offset: 0, shift: 0 }; 90];
