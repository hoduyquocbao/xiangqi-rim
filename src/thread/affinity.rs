// ============================================================================
// MODULE AFFINITY: GÁN ĐỊNH TUYẾN LUỒNG ƯU TIÊN P-CORE (THREAD AFFINITY & QOS)
// ============================================================================
// `affinity.rs` chịu trách nhiệm định tuyến các luồng xử lý (Worker Threads)
// sang các nhân hiệu năng cao (P-Cores - Performance Cores) trên hệ điều hành macOS Apple Silicon.
// - Sử dụng hàm FFI chuẩn `pthread_set_qos_class_self_np` không phụ thuộc crate ngoài (Clean Room 0₫).
// - Thiết lập cấp độ chất lượng dịch vụ QoS: `QOS_CLASS_USER_INTERACTIVE` (0x21 / 33).
// - Cấu trúc `Affinity` căn lề bộ nhớ 64-byte (`#[repr(C, align(64))]`) triệt tiêu Cache Line Bouncing.
// ============================================================================

/// Struct `Affinity` quản lý định tuyến luồng ưu tiên P-Core, căn lề 64-byte.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct Affinity {
    /// Chỉ số nhận diện luồng worker trong hệ thống (0..N-1)
    pub index: usize,
    /// Mã cấp độ chất lượng dịch vụ QoS (macOS QOS_CLASS_USER_INTERACTIVE = 0x21)
    pub scope: u32,
    /// Trạng thái gán định tuyến luồng thành công (true nếu thành công)
    pub active: bool,
    /// Mảng đệm bảo tồn kích thước vừa khít dòng đệm L1 Cache line 64-byte
    pub pad: [u8; 47],
}

impl Affinity {
    /// Khởi tạo một đối tượng `Affinity` mới với chỉ số luồng `index`.
    #[inline(always)]
    pub fn new(index: usize) -> Self {
        Self {
            index,
            scope: 0x21, // QOS_CLASS_USER_INTERACTIVE
            active: false,
            pad: [0u8; 47],
        }
    }

    /// Thực thi gán định tuyến luồng hiện tại sang P-Cores thông qua FFI hệ điều hành.
    /// Trả về `true` nếu áp dụng QoS class thành công.
    #[inline(always)]
    pub fn apply(&mut self) -> bool {
        #[cfg(target_os = "macos")]
        unsafe {
            extern "C" {
                fn pthread_set_qos_class_self_np(qos: u32, relative_priority: i32) -> i32;
            }
            let ret = pthread_set_qos_class_self_np(self.scope, 0);
            self.active = ret == 0;
            self.active
        }
        #[cfg(not(target_os = "macos"))]
        {
            self.active = false;
            false
        }
    }
}

// ----------------------------------------------------------------------------
// KHU VỰC BÀI KIỂM THỬ ĐƠN VỊ (UNIT TESTS) CHO THREAD AFFINITY
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// Kiểm thử căn lề bộ nhớ 64-byte và kích thước struct `Affinity`.
    #[test]
    fn alignment() {
        assert_eq!(std::mem::align_of::<Affinity>(), 64);
        assert_eq!(std::mem::size_of::<Affinity>(), 64);
    }

    /// Kiểm thử phương thức khởi tạo và áp dụng QoS class trên luồng hiện tại.
    #[test]
    fn binding() {
        let mut affinity = Affinity::new(0);
        assert_eq!(affinity.index, 0);
        assert_eq!(affinity.scope, 0x21);

        let ok = affinity.apply();
        #[cfg(target_os = "macos")]
        assert!(ok, "Trên macOS, apply MUST trả về true khi đặt QoS USER_INTERACTIVE!");
        #[cfg(not(target_os = "macos"))]
        assert!(!ok, "Trên nền tảng khác macOS, apply trả về false!");
    }
}
