// ============================================================================
// MODULE KIND: PHÂN LOẠI CÁC THÔNG ĐIỆP HỆ THỐNG CQRS (MESSAGE DISPATCH KIND)
// ============================================================================
// Định nghĩa enum `Kind` để phân định 3 loại thông điệp cơ bản trong kiến trúc CQRS-ES:
// - `Command`: Lệnh điều khiển làm thay đổi trạng thái hệ thống.
// - `Query`: Lệnh truy vấn dữ liệu đọc (Read-only query).
// - `Event`: Sự kiện đã xảy ra cần phát thông báo và lưu vết vào Event Store.
// ============================================================================

/// Enum `Kind` phân loại bản chất tác vụ của thông điệp CQRS.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Thông điệp dạng Lệnh điều khiển (Command)
    Command,
    /// Thông điệp dạng Truy vấn dữ liệu (Query)
    Query,
    /// Thông điệp dạng Sự kiện hệ thống (Event)
    Event,
}

