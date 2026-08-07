// ============================================================================
// MODULE REQUEST: PHÂN TÍCH YÊU CẦU HTTP REST VÀ WEBSOCKET HANDSHAKE
// ============================================================================
// Triển khai cấu trúc và phương thức phân tích HTTP Request từ TcpStream.
// Hỗ trợ trích xuất đường dẫn, phương thức, header và JSON body payload.
// Tuân thủ 100% quy tắc từ đơn tiếng Anh cho định danh và chú thích tiếng Việt.
// ============================================================================

use std::io::Read;
use std::net::TcpStream;
use super::method::Method;

/// Struct `Request` biểu diễn thông tin một yêu cầu HTTP
#[derive(Debug, Clone)]
#[repr(C, align(64))]
pub struct Request {
    /// Phương thức HTTP method
    pub method: Method,
    /// Đường dẫn URI path
    pub path: String,
    /// Danh sách cặp header (khóa, giá trị)
    pub headers: Vec<(String, String)>,
    /// Dải byte dữ liệu thân yêu cầu body
    pub body: Vec<u8>,
    /// Cờ đánh dấu yêu cầu nâng cấp WebSocket
    pub websocket: bool,
}

impl Request {
    /// Đọc và phân tích yêu cầu HTTP từ luồng kết nối TCP stream
    pub fn parse(stream: &mut TcpStream) -> Result<Self, String> {
        let mut buf = [0u8; 4096];
        let count = stream.read(&mut buf).map_err(|e| e.to_string())?;
        if count == 0 {
            return Err("Dữ liệu socket rỗng".to_string());
        }

        let raw = String::from_utf8_lossy(&buf[..count]);
        let mut lines = raw.split("\r\n");

        // Đọc dòng yêu cầu đầu tiên (GET /api/v1/health HTTP/1.1)
        let line = lines.next().ok_or_else(|| "Dòng yêu cầu rỗng".to_string())?;
        let mut parts = line.split_whitespace();
        let method = Method::parse(parts.next().unwrap_or(""));
        let path = parts.next().unwrap_or("/").to_string();

        // Đọc danh sách các header
        let mut headers = Vec::new();
        let mut websocket = false;
        let mut len = 0usize;

        for line in lines {
            if line.is_empty() {
                break;
            }
            if let Some(pos) = line.find(':') {
                let key = line[..pos].trim().to_lowercase();
                let val = line[pos + 1..].trim().to_string();

                if key == "upgrade" && val.to_lowercase() == "websocket" {
                    websocket = true;
                }
                if key == "content-length" {
                    len = val.parse::<usize>().unwrap_or(0);
                }

                headers.push((key, val));
            }
        }

        // Đọc dữ liệu thân body dựa trên vị trí \r\n\r\n
        let body = if let Some(pos) = raw.find("\r\n\r\n") {
            let start = pos + 4;
            let bytes = raw.as_bytes();
            if start < bytes.len() {
                let avail = bytes.len() - start;
                let take = if len > 0 && len < avail { len } else { avail };
                bytes[start..start + take].to_vec()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        Ok(Self {
            method,
            path,
            headers,
            body,
            websocket,
        })
    }

    /// Trích xuất giá trị header theo tên khóa name
    pub fn header(&self, name: &str) -> Option<&str> {
        let key = name.to_lowercase();
        for (k, v) in &self.headers {
            if k == &key {
                return Some(v.as_str());
            }
        }
        None
    }
}
