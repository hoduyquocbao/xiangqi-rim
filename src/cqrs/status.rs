// ============================================================================
// MODULE STATUS: TRẠNG THÁI XỬ LÝ THÔNG ĐIỆP HỆ THỐNG (MESSAGE PROCESSING STATUS)
// ============================================================================
// Định nghĩa enum `Status` biểu diễn tiến trình xử lý thông điệp CQRS:
// - `Pending`: Đang chờ xử lý trong Hàng đợi Ring Buffer.
// - `Success`: Xử lý thành công hoàn tất.
// - `Failed`: Xử lý thất bại.
// - `Aborted`: Bị ngắt dừng chủ động.
// ============================================================================

/// Enum `Status` theo dõi trạng thái tiến trình xử lý tác vụ thông điệp.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// Đang chờ (Pending): Thông điệp nằm trong hàng đợi chờ luồng công nhân nạp xử lý
    Pending,
    /// Thành công (Success): Thông điệp đã được thực thi thành công
    Success,
    /// Thất bại (Failed): Thông điệp gặp lỗi trong quá trình thực thi
    Failed,
    /// Bị hủy (Aborted): Thông điệp bị ngắt ngưng chủ động trước khi hoàn tất
    Aborted,
}

