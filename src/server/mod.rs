// ============================================================================
// MODULE SERVER: BỘ CHUYỂN ĐỔI GIAO THỨC SERVER REST VÀ WEBSOCKET BACKEND
// ============================================================================
// Module giao tiếp hạ tầng mạng đa luồng HTTP REST API và WebSocket RFC 6455.
// 100% Clean Room std-only không phụ thuộc bất kỳ thư viện bên ngoài nào.
// Tuân thủ 100% quy tắc từ đơn tiếng Anh cho định danh và chú thích tiếng Việt.
// ============================================================================

pub mod base64;
pub mod frame;
pub mod json;
pub mod method;
pub mod request;
pub mod response;
pub mod server;
pub mod sha1;
pub mod status;

pub use base64::Base64;
pub use frame::{Frame, Opcode};
pub use method::Method;
pub use request::Request;
pub use response::Response;
pub use server::Server;
pub use sha1::Sha1;
pub use status::Status;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha1_and_base64_rfc6455_vector() {
        // Kiểm thử RFC 6455 Sec-WebSocket-Key Handshake vector
        let key = "dGhlIHNhbXBsZSBub25jZQ==";
        let guid = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
        let concat = format!("{}{}", key, guid);
        let hash = Sha1::hash(concat.as_bytes());
        let accept = Base64::encode(&hash);
        assert_eq!(accept, "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
    }

    #[test]
    fn test_sha1_basic_vectors() {
        // Kiểm thử SHA-1 vector "abc"
        let hash = Sha1::hash(b"abc");
        let hex: String = hash.iter().map(|b| format!("{:02x}", b)).collect();
        assert_eq!(hex, "a9993e364706816aba3e25717850c26c9cd0d89d");
    }

    #[test]
    fn test_base64_basic_vectors() {
        // Kiểm thử Base64 vector chuẩn
        assert_eq!(Base64::encode(b""), "");
        assert_eq!(Base64::encode(b"f"), "Zg==");
        assert_eq!(Base64::encode(b"fo"), "Zm8=");
        assert_eq!(Base64::encode(b"foo"), "Zm9v");
    }

    #[test]
    fn test_websocket_frame_text_encoding() {
        // Kiểm thử mã hóa Text Frame WebSocket
        let frame = Frame::text("hello");
        assert_eq!(frame[0], 0x81);
        assert_eq!(frame[1], 5);
        assert_eq!(&frame[2..], b"hello");
    }

    #[test]
    fn test_rest_routes() {
        let server = Server::new("127.0.0.1", 8888);

        // Test POST /api/v1/config/hash
        let req = Request {
            method: Method::Post,
            path: "/api/v1/config/hash".to_string(),
            headers: Vec::new(),
            body: b"{\"mb\":512}".to_vec(),
            websocket: false,
        };
        let res = server.route(&req);
        assert_eq!(res.status, Status::Ok);
        assert!(String::from_utf8_lossy(&res.body).contains("\"mb\":512"));
        assert_eq!(server.hash.load(std::sync::atomic::Ordering::Relaxed), 512);

        // Test GET /api/v1/health
        let req = Request {
            method: Method::Get,
            path: "/api/v1/health".to_string(),
            headers: Vec::new(),
            body: Vec::new(),
            websocket: false,
        };
        let res = server.route(&req);
        assert_eq!(res.status, Status::Ok);
        assert!(String::from_utf8_lossy(&res.body).contains("status\":\"ok"));

        // Test POST /api/v1/position/parse
        let req = Request {
            method: Method::Post,
            path: "/api/v1/position/parse".to_string(),
            headers: Vec::new(),
            body: b"{\"fen\":\"rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1\"}".to_vec(),
            websocket: false,
        };
        let res = server.route(&req);
        assert_eq!(res.status, Status::Ok);
        assert!(String::from_utf8_lossy(&res.body).contains("hash"));

        // Test POST /api/v1/eval
        let req = Request {
            method: Method::Post,
            path: "/api/v1/eval".to_string(),
            headers: Vec::new(),
            body: b"{\"fen\":\"rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1\"}".to_vec(),
            websocket: false,
        };
        let res = server.route(&req);
        assert_eq!(res.status, Status::Ok);
        assert!(String::from_utf8_lossy(&res.body).contains("score"));

        // Test POST /api/v1/search
        let req = Request {
            method: Method::Post,
            path: "/api/v1/search".to_string(),
            headers: Vec::new(),
            body: b"{\"fen\":\"rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1\",\"depth\":2}".to_vec(),
            websocket: false,
        };
        let res = server.route(&req);
        assert_eq!(res.status, Status::Ok);
        assert!(String::from_utf8_lossy(&res.body).contains("bestmove"));

        // Test POST /api/v1/perft
        let req = Request {
            method: Method::Post,
            path: "/api/v1/perft".to_string(),
            headers: Vec::new(),
            body: b"{\"fen\":\"rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1\",\"depth\":1}".to_vec(),
            websocket: false,
        };
        let res = server.route(&req);
        assert_eq!(res.status, Status::Ok);
        assert!(String::from_utf8_lossy(&res.body).contains("divide"));
    }
}
