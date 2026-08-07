// ============================================================================
// MODULE RESPONSE: ĐÓNG GÓI PHẢN HỒI HTTP REST VÀ CORS
// ============================================================================
// Triển khai cấu trúc và phương thức đóng gói HTTP Response gửi qua TcpStream.
// Hỗ trợ tạo phản hồi JSON, định dạng header CORS và viết ra socket.
// Tuân thủ 100% quy tắc từ đơn tiếng Anh cho định danh và chú thích tiếng Việt.
// ============================================================================

use std::io::Write;
use std::net::TcpStream;
use super::status::Status;

/// Struct `Response` biểu diễn thông tin phản hồi HTTP
#[derive(Debug, Clone)]
#[repr(C, align(64))]
pub struct Response {
    /// Mã trạng thái HTTP status
    pub status: Status,
    /// Danh sách cặp header (khóa, giá trị)
    pub headers: Vec<(String, String)>,
    /// Dải byte dữ liệu thân phản hồi body
    pub body: Vec<u8>,
}

impl Response {
    /// Khởi tạo đối tượng `Response` với mã trạng thái status ban đầu
    pub fn new(status: Status) -> Self {
        let mut res = Self {
            status,
            headers: Vec::new(),
            body: Vec::new(),
        };
        // Thêm các header mặc định bao gồm CORS
        res = res.header("Server", "XiangRust/0.1.0");
        res = res.header("Access-Control-Allow-Origin", "*");
        res = res.header("Access-Control-Allow-Methods", "GET, POST, OPTIONS");
        res = res.header("Access-Control-Allow-Headers", "Content-Type");
        res
    }

    /// Thêm một cặp header (khóa key, giá trị val) vào phản hồi
    pub fn header(mut self, key: &str, val: &str) -> Self {
        self.headers.push((key.to_string(), val.to_string()));
        self
    }

    /// Khởi tạo nhanh phản hồi chuẩn JSON với chuỗi dữ liệu body
    pub fn json(status: Status, body: &str) -> Self {
        let bytes = body.as_bytes().to_vec();
        let len = bytes.len();
        Self::new(status)
            .header("Content-Type", "application/json")
            .header("Content-Length", &len.to_string())
            .body(bytes)
    }

    /// Thiết lập dữ liệu thân body cho phản hồi
    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    /// Gửi phản hồi HTTP hoàn chỉnh ra luồng TCP stream
    pub fn write(&self, stream: &mut TcpStream) -> Result<(), String> {
        let mut head = format!("HTTP/1.1 {}\r\n", self.status.text());
        for (k, v) in &self.headers {
            head.push_str(&format!("{}: {}\r\n", k, v));
        }
        head.push_str("\r\n");

        stream.write_all(head.as_bytes()).map_err(|e| e.to_string())?;
        if !self.body.is_empty() {
            stream.write_all(&self.body).map_err(|e| e.to_string())?;
        }
        stream.flush().map_err(|e| e.to_string())?;
        Ok(())
    }
}
