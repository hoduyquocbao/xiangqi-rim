// ============================================================================
// MODULE PV: QUẢN LÝ TỰU TRUYẾN NƯỚC ĐI BIẾN THỂ CHÍNH (PRINCIPAL VARIATION ARRAY)
// ============================================================================
// `Pv` đại diện cho mảng cố định chứa danh sách các nước đi thuộc biến thể chính (PV Line):
// - Lưu trữ tối đa 128 nước đi (`items: [Move; 128]`).
// - Căn lề 64-byte `#[repr(C, align(64))]` giúp thao tác sao chép tuyến PV `copy_from_slice`
//   diễn ra cực nhanh với SIMD memcpy độ trễ 0 cycle.
// ============================================================================

use crate::movegen::types::Move;

/// Struct `Pv` quản lý mảng lưu vết đường đi nước cờ tốt nhất, căn lề 64-byte.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct Pv {
    /// Mảng chứa tối đa 128 nước đi thuộc biến thể chính
    pub items: [Move; 128],
    /// Độ dài thực tế hiện tại của đường đi PV
    pub len: usize,
}

impl Default for Pv {
    /// Khởi tạo mặc định đối tượng Pv.
    fn default() -> Self {
        Self::new()
    }
}

impl Pv {
    /// Khởi tạo tuyến PV rỗng.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            items: [Move::none(); 128],
            len: 0,
        }
    }

    /// Đặt lại độ dài tuyến PV về 0.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Thêm một nước đi `mv` vào cuối tuyến PV.
    #[inline(always)]
    pub fn push(&mut self, mv: Move) {
        if self.len < 128 {
            self.items[self.len] = mv;
            self.len += 1;
        }
    }

    /// Cập nhật tuyến PV bằng nước đi `mv` ở vị trí đầu tiên nối với tuyến PV con `child`.
    #[inline(always)]
    pub fn update(&mut self, mv: Move, child: &Pv) {
        self.items[0] = mv;
        let copy = child.len.min(127);
        // Sao chép SIMD nhanh toàn bộ mảng PV con vào mảng PV hiện tại
        self.items[1..=copy].copy_from_slice(&child.items[..copy]);
        self.len = copy + 1;
    }
}

