// ============================================================================
// VÍ DỤ 11: BACKEND REST API VÀ WEBSOCKET REAL-TIME STREAMING SERVER XIANGRUST
// ============================================================================
// Tệp ví dụ độc lập minh họa khởi chạy Backend Server đa luồng phục vụ 5 endpoint HTTP
// REST API (/health, /position/parse, /eval, /search, /perft) và WebSocket real-time
// search progress streaming (RFC 6455 std-only SHA-1 & Base64 handshake & framing).
// 100% chú thích tiếng Việt từng dòng & 100% định danh từ đơn tiếng Anh.
// ============================================================================

use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use xiangrust::server::{Base64, Frame, Opcode, Server, Sha1};

// Hàm chính main thực thi ví dụ backend server và bộ kiểm thử tự động self-test
fn main() {
    // In dòng kẻ phân cách trang trí tiêu đề
    println!("============================================================");
    // In tiêu đề ứng dụng Backend Server XiangRust
    println!("  XIANGRUST AI ENGINE - VÍ DỤ 11: BACKEND REST & WEBSOCKET  ");
    // In dòng kẻ phân cách trang trí
    println!("============================================================");

    // Khai báo cổng port dịch vụ mặc định 8888
    let port = 8888u16;
    // Tạo địa chỉ bind chuỗi host 0.0.0.0 phục vụ kết nối từ bên ngoài & Cloudflare Tunnel
    let addr = format!("0.0.0.0:{}", port);

    // Khởi tạo đối tượng Server bind địa chỉ 127.0.0.1:8888
    let server = Server::bind(&addr).expect("Không thể bind địa chỉ server");
    // Tạo một bản sao cloned của server cho luồng lắng nghe TCP
    let cloned = server.clone();

    // Khởi chạy luồng background thread lắng nghe kết nối TCP incoming
    thread::spawn(move || {
        // Thực thi vòng lặp lắng nghe listen
        if let Err(e) = cloned.listen() {
            eprintln!("[Server] Lỗi lắng nghe: {}", e);
        }
    });

    // Tạm dừng 200ms chờ server khởi tạo socket TCP listener
    thread::sleep(Duration::from_millis(200));

    // In thông báo tiến hành kiểm thử tự động 5 REST endpoints và WebSocket
    println!("\n------------------------------------------------------------");
    // In tiêu đề mục 1: Tiến hành tự động kiểm thử REST API & WebSocket
    println!("[1] BẮT ĐẦU TỰ ĐỘNG KIỂM THỬ 5 REST ENDPOINTS & WEBSOCKET");
    // In dòng kẻ phân cách
    println!("------------------------------------------------------------");

    // 1. Kiểm thử REST Endpoint GET /api/v1/health
    println!("\n -> [Test 1/6] GET /api/v1/health");
    let res1 = get(&addr, "/api/v1/health");
    println!("    Phản hồi: {}", res1.trim());
    assert!(res1.contains("status\":\"ok"));

    // 2. Kiểm thử REST Endpoint POST /api/v1/position/parse
    println!("\n -> [Test 2/6] POST /api/v1/position/parse");
    let payload2 = "{\"fen\":\"rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1\"}";
    let res2 = post(&addr, "/api/v1/position/parse", payload2);
    println!("    Phản hồi: {}", res2.trim());
    assert!(res2.contains("hash"));

    // 3. Kiểm thử REST Endpoint POST /api/v1/eval
    println!("\n -> [Test 3/6] POST /api/v1/eval");
    let payload3 = "{\"fen\":\"rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1\"}";
    let res3 = post(&addr, "/api/v1/eval", payload3);
    println!("    Phản hồi: {}", res3.trim());
    assert!(res3.contains("score"));

    // 4. Kiểm thử REST Endpoint POST /api/v1/search
    println!("\n -> [Test 4/6] POST /api/v1/search");
    let payload4 = "{\"fen\":\"rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1\",\"depth\":3}";
    let res4 = post(&addr, "/api/v1/search", payload4);
    println!("    Phản hồi: {}", res4.trim());
    assert!(res4.contains("bestmove"));

    // 5. Kiểm thử REST Endpoint POST /api/v1/perft
    println!("\n -> [Test 5/6] POST /api/v1/perft");
    let payload5 = "{\"fen\":\"rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1\",\"depth\":1}";
    let res5 = post(&addr, "/api/v1/perft", payload5);
    println!("    Phản hồi: {}", res5.trim());
    assert!(res5.contains("divide"));

    // 6. Kiểm thử WebSocket Endpoint ws://127.0.0.1:8080/ws
    println!("\n -> [Test 6/6] WebSocket ws://{}/ws (Real-time Streaming Search)", addr);
    ws(&addr);

    // In thông báo hoàn tất kiểm thử tự động xuất sắc
    println!("\n============================================================");
    // In thông báo 100% kiểm thử REST & WebSocket PASSED
    println!("  HOÀN THÀNH 100% KIỂM THỬ BACKEND REST & WEBSOCKET PASSED! ");
    // In dòng kẻ phân cách
    println!("============================================================");

    // Giữ server chạy vô hạn ở chế độ daemon listening
    println!("\n[Server] Đang chạy liên tục ở chế độ daemon listening (0.0.0.0:8888)...");
    loop {
        thread::sleep(Duration::from_secs(3600));
    }
}

/// Hàm tiện ích gửi yêu cầu HTTP GET
fn get(addr: &str, path: &str) -> String {
    let mut stream = TcpStream::connect(addr).expect("Không thể kết nối TCP");
    let req = format!("GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n", path, addr);
    stream.write_all(req.as_bytes()).unwrap();
    stream.flush().unwrap();

    let mut buf = Vec::new();
    let _ = stream.read_to_end(&mut buf);
    let text = String::from_utf8_lossy(&buf);

    if let Some(pos) = text.find("\r\n\r\n") {
        text[pos + 4..].to_string()
    } else {
        text.to_string()
    }
}

/// Hàm tiện ích gửi yêu cầu HTTP POST
fn post(addr: &str, path: &str, body: &str) -> String {
    let mut stream = TcpStream::connect(addr).expect("Không thể kết nối TCP");
    let req = format!(
        "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        path,
        addr,
        body.len(),
        body
    );
    stream.write_all(req.as_bytes()).unwrap();
    stream.flush().unwrap();

    let mut buf = Vec::new();
    let _ = stream.read_to_end(&mut buf);
    let text = String::from_utf8_lossy(&buf);

    if let Some(pos) = text.find("\r\n\r\n") {
        text[pos + 4..].to_string()
    } else {
        text.to_string()
    }
}

/// Hàm tiện ích gửi và nhận WebSocket RFC 6455 streaming
fn ws(addr: &str) {
    let mut stream = TcpStream::connect(addr).expect("Không thể kết nối TCP cho WebSocket");

    // Khóa Handshake WebSocket giả lập
    let raw = "dGhlIHNhbXBsZSBub25jZQ==";
    let guid = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let concat = format!("{}{}", raw, guid);
    let expected = Base64::encode(&Sha1::hash(concat.as_bytes()));

    // Gửi HTTP GET Upgrade Request
    let req = format!(
        "GET /ws HTTP/1.1\r\nHost: {}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {}\r\nSec-WebSocket-Version: 13\r\n\r\n",
        addr, raw
    );
    stream.write_all(req.as_bytes()).unwrap();
    stream.flush().unwrap();

    // Đọc phản hồi Handshake 101 Switching Protocols
    let mut buf = [0u8; 1024];
    let count = stream.read(&mut buf).unwrap();
    let head = String::from_utf8_lossy(&buf[..count]);
    println!("    WebSocket Handshake Response:\n{}", head.trim());
    assert!(head.contains("101 Switching Protocols"));
    assert!(head.contains(&expected));

    // Gửi WebSocket Text Frame yêu cầu search
    let action = "{\"action\":\"search\",\"fen\":\"rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1\",\"depth\":3}";

    // Đóng gói Text Frame có 4-byte Mask từ Client (RFC 6455 required for client)
    let bytes = action.as_bytes();
    let len = bytes.len();
    let mask = [0x12, 0x34, 0x56, 0x78];

    let mut packet = Vec::new();
    // Byte 0: FIN = 1 (0x80) | Text (0x01) = 0x81
    packet.push(0x81);
    // Byte 1: Mask bit = 1 (0x80) | len
    packet.push(0x80 | (len as u8));
    // 4 bytes Mask key
    packet.extend_from_slice(&mask);

    // Payload đã được XOR với Mask key
    for i in 0..len {
        packet.push(bytes[i] ^ mask[i % 4]);
    }

    // Gửi packet frame tới server
    stream.write_all(&packet).unwrap();
    stream.flush().unwrap();

    // Đọc streaming frames nhận từ server (Depth 1, 2, 3 info frames và bestmove frame)
    println!("    Đang nhận WebSocket real-time search progress stream:");
    for _ in 0..4 {
        if let Ok(frame) = Frame::parse(&mut stream) {
            if frame.opcode == Opcode::Text {
                let payload = String::from_utf8_lossy(&frame.payload);
                println!("      [WS Stream] Frame nhận được: {}", payload);
            }
        }
    }

    // Gửi Close Frame kết thúc phiên WebSocket
    let close = vec![0x88, 0x80, 0x11, 0x22, 0x33, 0x44];
    let _ = stream.write_all(&close);
    let _ = stream.flush();
}
