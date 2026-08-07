# BẢN ĐẶC TẢ KIẾN TRÚC MÁY CHỦ BACKEND REST API VÀ WEBSOCKET REAL-TIME STREAMING

---

## 1. TỔNG QUAN KIẾN TRÚC MÁY CHỦ (BACKEND ARCHITECTURE OVERVIEW)

### 1.1 Triết lý Clean Room 0% Dependency (std-only Networking)
Máy chủ Backend của XiangRust Engine được thiết kế hoàn toàn dựa trên thư viện tiêu chuẩn **Rust `std-only`**:
- **0% crate ngoài**: Không sử dụng `tokio`, `actix-web`, `hyper`, `tungstenite`, `serde`, hay bất kỳ thư viện async/HTTP/WebSocket bên thứ ba nào.
- **Hạ tầng Mạng Thuần túy**: Xây dựng trực tiếp trên `std::net::TcpListener`, `std::net::TcpStream`, `std::thread`, và `std::sync`.
- **Căn Lề Bộ Nhớ Hardware Alignment 64-Byte**: Cấu trúc `Server`, `Sha1`, và `Frame` đều sử dụng chỉ thị `#[repr(C, align(64))]` nhằm tối ưu hóa CPU Cache Line, triệt tiêu vi phạm bộ nhớ và lỗi False Sharing giữa các luồng.

### 1.2 Hỗ trợ Đa Giao thức Kết hợp (HTTP REST & RFC 6455 WebSocket)
- **HTTP REST API**: Cung cấp các cổng giao tiếp chuẩn RESTful hỗ trợ cơ chế CORS Preflight (`Access-Control-Allow-Origin: *`) cho ứng dụng Web Client/Mobile.
- **WebSocket Streaming**: Tương thích hoàn toàn với RFC 6455, cho phép nâng cấp kết nối từ HTTP sang WebSocket song công toàn phần (Full-duplex real-time streaming).

---

## 2. LỆNH KHỞI CHẠY MÁY CHỦ & DÒNG LỆNH (SERVER INVOCATION)

### 2.1 Biên dịch & Chạy Tự Động Kiểm Thử Self-Test
```bash
cargo run --example 11_backend_server --release
```
Lệnh trên khởi chạy server tại `http://127.0.0.1:8080`, thực thi thành công bộ tự động kiểm thử 6/6 kịch bản REST API & WebSocket, sau đó kết thúc lệnh.

### 2.2 Khởi chạy Máy chủ Liên tục Chế độ Daemon Listening
```bash
cargo run --example 11_backend_server --release -- --serve
```
Truyền thêm tham số `--serve` để giữ server tiếp tục lắng nghe liên tục phục vụ các ứng dụng client bên ngoài.

---

## 3. ĐẶC TẢ CHI TIẾT 5 HTTP REST API ENDPOINTS

Tất cả các phản hồi HTTP REST đều đính kèm tiêu đề CORS Header:
`Access-Control-Allow-Origin: *`, `Access-Control-Allow-Methods: GET, POST, OPTIONS`, `Content-Type: application/json`.

---

### 3.1 Endpoint 1: `GET /api/v1/health`

- **Phương thức**: `GET`
- **Mục đích**: Kiểm tra trạng thái sống (Health Check) và thông tin phiên bản của AI Engine.
- **Header Yêu cầu**: Không có.
- **Mẫu Phản hồi HTTP 200 OK**:
```json
{
  "status": "ok",
  "engine": "xiangrust",
  "version": "0.1.0"
}
```

---

### 3.2 Endpoint 2: `POST /api/v1/position/parse`

- **Phương thức**: `POST`
- **Mục đích**: Phân tích FEN bàn cờ, tính toán mã băm 64-bit Zobrist Hash, xác định phe nắm lượt đi và kiểm tra tính chuẩn hóa.
- **Payload Yêu cầu (Request JSON)**:
```json
{
  "fen": "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1"
}
```

- **Mẫu Phản hồi HTTP 200 OK**:
```json
{
  "status": "ok",
  "fen": "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
  "hash": "0x0000000000000000",
  "turn": "red",
  "export": "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1"
}
```

---

### 3.3 Endpoint 3: `POST /api/v1/eval`

- **Phương thức**: `POST`
- **Mục đích**: Đánh giá vị trí bàn cờ bằng Mạng Nơ-ron NNUE + HCE, đồng thời kiểm tra trạng thái CircuitBreaker an toàn.
- **Payload Yêu cầu (Request JSON)**:
```json
{
  "fen": "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1"
}
```

- **Mẫu Phản hồi HTTP 200 OK**:
```json
{
  "status": "ok",
  "score": 0,
  "turn": "red",
  "breaker": "Closed"
}
```

---

### 3.4 Endpoint 4: `POST /api/v1/search`

- **Phương thức**: `POST`
- **Mục đích**: Thực thi phiên tìm kiếm PVS (Principal Variation Search) để tìm nước đi tối ưu nhất cho thế cờ.
- **Payload Yêu cầu (Request JSON)**:
```json
{
  "fen": "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
  "depth": 3
}
```

- **Mẫu Phản hồi HTTP 200 OK**:
```json
{
  "status": "ok",
  "bestmove": "h2e2",
  "score": 0,
  "nodes": 182040,
  "time": 15,
  "nps": 12136000
}
```

---

### 3.5 Endpoint 5: `POST /api/v1/perft`

- **Phương thức**: `POST`
- **Mục đích**: Chạy kiểm thử sinh nước đi Perft và trả về danh sách phân rã số lượng nút con cho từng nước đi hợp lệ.
- **Payload Yêu cầu (Request JSON)**:
```json
{
  "fen": "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
  "depth": 1
}
```

- **Mẫu Phản hồi HTTP 200 OK**:
```json
{
  "status": "ok",
  "depth": 1,
  "nodes": 44,
  "divide": [
    {"move": "b2e2", "nodes": 1},
    {"move": "b2a2", "nodes": 1},
    {"move": "b2c2", "nodes": 1},
    {"move": "h2e2", "nodes": 1},
    {"move": "h2g2", "nodes": 1},
    {"move": "h2i2", "nodes": 1}
  ]
}
```

---

## 4. ĐẶC TẢ GIAO THỨC REAL-TIME WEBSOCKET RFC 6455

Endpoint WebSocket hoạt động tại địa chỉ URI: `ws://127.0.0.1:8080/ws`.

### 4.1 Thuật toán Handshake RFC 6455
1. Client gửi HTTP GET với các header Upgrade:
   ```http
   GET /ws HTTP/1.1
   Host: 127.0.0.1:8080
   Upgrade: websocket
   Connection: Upgrade
   Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==
   Sec-WebSocket-Version: 13
   ```
2. Server trích xuất `Sec-WebSocket-Key`, nối với GUID chuẩn RFC 6455 `"258EAFA5-E914-47DA-95CA-C5AB0DC85B11"`:
   `concat = "dGhlIHNhbXBsZSBub25jZQ==258EAFA5-E914-47DA-95CA-C5AB0DC85B11"`.
3. Băm kết quả qua `Sha1::hash()` (20 bytes raw binary).
4. Mã hóa mảng 20 bytes thành chuỗi Base64 bằng `Base64::encode()`.
5. Phản hồi Handshake HTTP 101:
   ```http
   HTTP/1.1 101 Switching Protocols
   Upgrade: websocket
   Connection: Upgrade
   Sec-WebSocket-Accept: s3pPLsBiTxaQ9kYGzzhZRbK+xOo=
   ```

### 4.2 Giải Mã Khung Tin WebSocket (4-byte Mask Decoding)
Theo chuẩn RFC 6455, tất cả các khung tin gửi từ Client đến Server phải bật bit Mask (Mask bit = 1) và chứa 4 bytes Mask Key:
- `Frame::parse()` đọc header 2 byte, lấy 4 bytes `mask_key`.
- Giải mã mảng byte payload bằng phép toán XOR phần cứng:
  $$\text{payload}[i] = \text{masked\_payload}[i] \oplus \text{mask\_key}[i \pmod 4]$$

### 4.3 Quá trình Real-Time Search Progress Streaming
Khi Client gửi câu lệnh WebSocket Text Frame:
```json
{
  "action": "search",
  "fen": "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
  "depth": 3
}
```

Server lập tức stream từng độ sâu kết quả bằng các khung tin `type: info` theo thời gian thực trước khi chốt khung tin `type: bestmove`:

1. **Khung tin Real-time Progress (Depth 1)**:
   ```json
   {"type":"info","depth":1,"score":0,"nodes":44,"nps":44000,"pv":"h2e2"}
   ```
2. **Khung tin Real-time Progress (Depth 2)**:
   ```json
   {"type":"info","depth":2,"score":0,"nodes":1936,"nps":1936000,"pv":"h2e2"}
   ```
3. **Khung tin Real-time Progress (Depth 3)**:
   ```json
   {"type":"info","depth":3,"score":0,"nodes":182040,"nps":12136000,"pv":"h2e2"}
   ```
4. **Khung tin Kết quả Cuối cùng (`bestmove`)**:
   ```json
   {"type":"bestmove","best":"h2e2","score":0,"nodes":182040,"time":15}
   ```

---

## 5. MÔ HÌNH ĐA LUỒNG & QUẢN LÝ PHIÊN (THREADING MODEL)

### 5.1 Mô hình Luồng Kết Nối TCP (Connection Threads)
- **Vòng lặp Chờ Kết nối**: `Server::listen()` gọi `listener.incoming()`.
- **Phân chia Luồng Độc lập**: Khi có kết nối TCP stream mới, server thực thi `std::thread::spawn(move || { server.handle(&mut stream); })`.
- Mỗi kết nối client được xử lý riêng biệt trên một OS Thread độc lập, tránh tình trạng nghẽn cổ chai I/O.

### 5.2 Mô hình lazy SMP Engine Pool Integration
- Trong phiên xử lý lệnh `search`, server khởi tạo instance của `Search` hoặc `thread::Pool` (Lazy SMP).
- Bộ nhớ Transposition Table và mảng Bitboard bàn cờ được cô lập an toàn giữa các kết nối client.

---
*Tài liệu được biên soạn chuẩn đặc tả Backend Server XiangRust.*
