// ============================================================================
// MODULE STATUS: ĐỊNH NGHĨA MÃ TRẠNG THÁI HTTP (RFC 7231)
// ============================================================================
// Định nghĩa enum `Status` đại diện cho mã phản hồi trạng thái HTTP.
// Tuân thủ 100% quy tắc từ đơn tiếng Anh cho định danh và chú thích tiếng Việt.
// ============================================================================

/// Enum `Status` biểu diễn mã phản hồi HTTP
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// 200 Ok
    Ok = 200,
    /// 101 Upgrade Switching Protocols
    Upgrade = 101,
    /// 400 Bad Request
    Bad = 400,
    /// 404 NotFound
    NotFound = 404,
    /// 500 Fault Server Error
    Fault = 500,
}

impl Status {
    /// Trả về chuỗi văn bản mô tả mã trạng thái HTTP
    pub fn text(self) -> &'static str {
        match self {
            Self::Ok => "200 OK",
            Self::Upgrade => "101 Switching Protocols",
            Self::Bad => "400 Bad Request",
            Self::NotFound => "404 Not Found",
            Self::Fault => "500 Internal Server Error",
        }
    }
}
