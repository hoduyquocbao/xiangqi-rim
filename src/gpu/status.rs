// ============================================================================
// XIANGTI ENGINE: TRẠNG THÁI TẦNG GPU VÀ QUẢN LÝ BỘ NHỚ VRAM (STATUS)
// ============================================================================
// Định nghĩa các mã trạng thái kết quả của bộ chuyển đổi GPU Adapter và Guard.
// Tuân thủ 100% định danh từ đơn tiếng Anh và 100% chú thích tiếng Việt.
// ============================================================================

/// Enum `Status`: Định nghĩa mã trạng thái kết quả hoạt động của GPU và bộ đệm VRAM.
#[repr(u8)] // Định dạng căn lề bộ nhớ 1 byte u8 tương thích FFI
#[derive(Debug, Clone, Copy, PartialEq, Eq)] // Tự động derive các trait cơ bản
pub enum Status { // Định nghĩa enum Status
    Ready = 0, // Trạng thái sẵn sàng tiếp nhận yêu cầu cấp phát bộ nhớ
    Active = 1, // Trạng thái đang hoạt động bình thường
    Exhausted = 2, // Trạng thái cạn kệt VRAM (chạm trần an toàn 409.6MB)
    Fault = 3, // Trạng thái lỗi bộ nhớ hoặc lỗi truy cập con trỏ
    Full = 4, // Trạng thái VRAM đầy cần chia nhỏ khối dữ liệu (chunking)
    Fail = 5, // Trạng thái tràn cứng VRAM (vượt quá 512MB)
    Cpu = 6, // Trạng thái đã hạ cấp về CPU SIMD fallback
} // Kết thúc định nghĩa enum Status

impl Status { // Khối triển khai các phương thức cho enum Status
    /// Phương thức `ok`: Kiểm tra xem trạng thái có phải là thành công (Ready hoặc Active) hay không.
    #[inline(always)] // Chỉ thị inline phương thức hot path
    pub fn ok(&self) -> bool { // Trả về true nếu không phải là trạng thái lỗi
        matches!(self, Self::Ready | Self::Active) // Khớp các biến thể thành công
    } // Kết thúc phương thức ok

    /// Phương thức `valid`: Kiểm tra xem trạng thái có hợp lệ để tiếp tục thao tác GPU hay không.
    #[inline(always)] // Inline phương thức kiểm tra hợp lệ
    pub fn valid(&self) -> bool { // Trả về giá trị boolean
        !matches!(self, Self::Fault | Self::Fail) // Trả về true nếu không gặp lỗi nghiêm trọng
    } // Kết thúc phương thức valid

    /// Phương thức `busy`: Kiểm tra xem trạng thái có đang ở mức Active hay không.
    #[inline(always)] // Inline phương thức kiểm tra bận
    pub fn busy(&self) -> bool { // Trả về true nếu đang Active
        matches!(self, Self::Active) // Khớp biến thể Active
    } // Kết thúc phương thức busy

    /// Phương thức `done`: Kiểm tra xem trạng thái có ở mức Ready hay không.
    #[inline(always)] // Inline phương thức kiểm tra hoàn thành
    pub fn done(&self) -> bool { // Trả về true nếu ở trạng thái Ready
        matches!(self, Self::Ready) // Khớp biến thể Ready
    } // Kết thúc phương thức done

    /// Phương thức `code`: Trả về mã số nguyên đại diện u8 của trạng thái.
    #[inline(always)] // Inline phương thức lấy mã số
    pub fn code(&self) -> u8 { // Trả về giá trị u8
        *self as u8 // Ép kiểu enum thành u8
    } // Kết thúc phương thức code

    /// Phương thức `name`: Trả về tên chuỗi hiển thị tĩnh của trạng thái.
    #[inline(always)] // Inline phương thức lấy tên hiển thị
    pub fn name(&self) -> &'static str { // Trả về tham chiếu chuỗi tĩnh
        match self { // Khớp mẫu biến thể enum
            Self::Ready => "Ready", // Chuỗi Ready
            Self::Active => "Active", // Chuỗi Active
            Self::Exhausted => "Exhausted", // Chuỗi Exhausted
            Self::Fault => "Fault", // Chuỗi Fault
            Self::Full => "Full", // Chuỗi Full
            Self::Fail => "Fail", // Chuỗi Fail
            Self::Cpu => "Cpu", // Chuỗi Cpu
        } // Kết thúc biểu thức match
    } // Kết thúc phương thức name
} // Kết thúc khối impl Status

#[cfg(test)] // Module kiểm thử unit tests cho Status
mod tests { // Cấu hình module tests
    use super::*; // Nhập các phần tử từ module cha

    #[test] // Đánh dấu hàm kiểm thử các phương thức helper của Status
    fn test_status_helpers() { // Hàm test các helper phương thức Status
        let status = Status::Ready; // Khởi tạo trạng thái Ready
        assert!(status.ok()); // Ready là ok
        assert!(status.valid()); // Ready là valid
        assert!(status.done()); // Ready là done
        assert!(!status.busy()); // Ready không busy
        assert_eq!(status.code(), 0); // Mã là 0
        assert_eq!(status.name(), "Ready"); // Tên là Ready

        let status = Status::Active; // Khởi tạo trạng thái Active
        assert!(status.ok()); // Active là ok
        assert!(status.valid()); // Active là valid
        assert!(status.busy()); // Active là busy
        assert!(!status.done()); // Active không done
        assert_eq!(status.code(), 1); // Mã là 1
        assert_eq!(status.name(), "Active"); // Tên là Active
    } // Kết thúc hàm test_status_helpers
} // Kết thúc module tests

