// ============================================================================
// MODULE METHOD: ĐỊNH NGHĨA CÁC PHƯƠNG THỨC HTTP (RFC 7231)
// ============================================================================
// Định nghĩa enum `Method` đại diện cho các phương thức HTTP REST.
// Tuân thủ 100% quy tắc từ đơn tiếng Anh cho định danh và chú thích tiếng Việt.
// ============================================================================

/// Enum `Method` biểu diễn các phương thức HTTP
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// Phương thức Get
    Get,
    /// Phương thức Post
    Post,
    /// Phương thức Put
    Put,
    /// Phương thức Delete
    Delete,
    /// Phương thức Options cho CORS preflight
    Options,
    /// Phương thức Unknown không xác định
    Unknown,
}

impl Method {
    /// Phân tích chuỗi phương thức HTTP text thành enum `Method`
    pub fn parse(text: &str) -> Self {
        match text {
            "GET" => Self::Get,
            "POST" => Self::Post,
            "PUT" => Self::Put,
            "DELETE" => Self::Delete,
            "OPTIONS" => Self::Options,
            _ => Self::Unknown,
        }
    }
}
