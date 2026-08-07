# BẢN ĐẶC TẢ GIAO THỨC & HƯỚNG DẪN TÍCH HỢP MODEL CONTEXT PROTOCOL (MCP) SERVER

---

## 1. TỔNG QUAN KIẾN TRÚC MCP SERVER (ARCHITECTURAL OVERVIEW)

### 1.1 Chuẩn Giao thức JSON-RPC 2.0 over STDIN/STDOUT
Model Context Protocol (MCP) Server của XiangRust Engine triển khai chuẩn truyền thông **JSON-RPC 2.0** tương tác trực tiếp thông qua hai đường dẫn dòng lệnh chuẩn:
- **STDIN**: Tiếp nhận các thông điệp yêu cầu JSON-RPC từ Trợ lý AI Client (Claude Desktop, Cursor IDE, Gemini Agent).
- **STDOUT**: Trả về các thông điệp phản hồi JSON-RPC theo đúng định dạng dòng (line-delimited JSON).
- **STDERR**: Toàn bộ nhật ký hệ thống, cảnh báo debug, và vết theo dõi nội bộ của Engine được đẩy về đường dẫn STDERR để tránh làm ô nhiễm đường dữ liệu STDOUT.

### 1.2 Triển khai Bộ Phân Tích JSON 0% Dependency (Clean Room Parser/Builder)
Server được thiết kế hoàn toàn bằng thư viện chuẩn Rust `std-only` (0% external crates như `serde` hay `json-rpc`):
- **Cấu trúc `Kind` & `Value`**: Biểu diễn cây cú pháp JSON linh hoạt trong bộ nhớ với các kiểu `Null`, `Bool`, `Number`, `String`, `Array`, `Object`.
- **Bộ dựng `Builder`**: Đóng gói cấu trúc `Value` thành chuỗi JSON hợp lệ kèm cơ chế mã hóa thoát ký tự (escaping) an toàn cho `\"`, `\\`, `\n`, `\r`, `\t`.
- **Bộ phân tích `Parser`**: Thuật toán phân tích đệ quy (Recursive Descent Parser) giải mã trực tiếp chuỗi JSON từ STDIN thành các nút `Value`, hỗ trợ đầy đủ chuỗi unicode `\uXXXX`, mảng động và đối tượng lồng nhau.

---

## 2. LỆNH KHỞI CHẠY MÁY CHỦ MCP SERVER (COMMAND LINE INVOCATION)

Máy chủ MCP Server được phát hành dưới dạng ứng dụng ví dụ nhị phân độc lập.

### 2.1 Biên dịch & Khởi chạy Chế độ Release
```bash
cargo run --example 10_mcp_server --release
```

### 2.2 Luồng xử lý Vòng lặp Yêu cầu (Request Loop)
Khi khởi chạy, server thực thi vòng lặp `Server::run()`:
1. Đọc từng dòng văn bản từ `io::stdin().lock().lines()`.
2. Phân tích gói tin yêu cầu `Request::parse(text)`.
3. Kiểm tra phương thức JSON-RPC 2.0:
   - `"initialize"`: Phản hồi thông tin phiên bản giao thức `2024-11-05` và tính năng máy chủ.
   - `"tools/list"`: Trả về danh sách đặc tả của 5 MCP Tools.
   - `"tools/call"`: Điều hướng thực thi Tool tương ứng và đóng gói phản hồi chuẩn `content: [{ type: "text", text: "..." }]`.
4. In phản hồi ra `STDOUT` bằng `println!` và gọi `io::stdout().flush()` lập tức.

---

## 3. ĐẶC TẢ CHI TIẾT & SCHEMA CỦA 5 MCP TOOLS

### 3.1 Tool 1: `get_best_move`

- **Mô tả**: Tìm kiếm nước đi tối ưu nhất cho vị trí FEN chỉ định bằng bộ tìm kiếm PVS (Principal Variation Search) kết hợp mô hình đa luồng Lazy SMP.
- **Tham số Đầu vào (`arguments`)**:
  - `fen` *(string, tùy chọn)*: Chuỗi FEN vị trí bàn cờ (mặc định là FEN vị trí ban đầu).
  - `depth` *(integer, tùy chọn)*: Độ sâu duyệt cây nước đi (mặc định: `8`).
  - `movetime` *(integer, tùy chọn)*: Thời gian giới hạn tìm kiếm tính bằng mili-giây.
  - `threads` *(integer, tùy chọn)*: Số luồng tính toán song song Lazy SMP (mặc định: `1`).
  - `hash` *(integer, tùy chọn)*: Dung lượngTransposition Table MB (mặc định: `64`).

- **Luồng xử lý (Workflow)**:
  1. Phân tích cú pháp chuỗi `fen` thành cấu trúc `Position`.
  2. Khởi tạo pool luồng `thread::Pool::new(threads, hash)`.
  3. Khởi chạy tìm kiếm `pool.go(&pos, &limits)`.
  4. Mã hóa nước đi tối ưu `best` và nước đi dự phòng `ponder` theo chuẩn UCI format.
  5. Tính toán tốc độ duyệt nút `nps = (nodes * 1000) / time_ms`.

- **Mẫu Yêu cầu JSON-RPC (`tools/call`)**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "get_best_move",
    "arguments": {
      "fen": "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
      "depth": 6,
      "threads": 4
    }
  }
}
```

- **Mẫu Phản hồi JSON-RPC**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"best_move\":\"h2e2\",\"ponder_move\":\"h8e8\",\"score\":25,\"depth\":6,\"nodes\":482910,\"time_ms\":38,\"nps\":12708157,\"pv\":[\"h2e2\",\"h8e8\",\"c2c5\",\"e7e5\"]}"
      }
    ]
  }
}
```

---

### 3.2 Tool 2: `evaluate_position`

- **Mô tả**: Đánh giá chi tiết điểm số thế cờ bằng Mạng Nơ-ron NNUE và Bộ luật Tĩnh HCE (Hand-Crafted Evaluation).
- **Tham số Đầu vào (`arguments`)**:
  - `fen` *(string, tùy chọn)*: Chuỗi FEN vị trí bàn cờ.
  - `mode` *(string, tùy chọn)*: Chế độ đánh giá (`"auto"`, `"nnue"`, `"hce"`).

- **Luồng xử lý (Workflow)**:
  1. Phân tích chuỗi `fen` thành `Position`.
  2. Khởi tạo đối tượng `Eval`.
  3. Gọi `eval.score(&pos)` lấy điểm tổng hợp và `eval.hce.score(&pos)` lấy điểm luật tĩnh HCE.
  4. Đếm số lượng 32 loại quân cờ đang có trên bàn cờ.

- **Mẫu Yêu cầu JSON-RPC**:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "evaluate_position",
    "arguments": {
      "fen": "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
      "mode": "auto"
    }
  }
}
```

- **Mẫu Phản hồi JSON-RPC**:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"score\":0,\"eval_mode\":\"auto\",\"hce_score\":0,\"side_to_move\":\"red\",\"piece_counts\":{\"red_king\":1,\"red_advisor\":2,\"red_bishop\":2,\"red_knight\":2,\"red_rook\":2,\"red_cannon\":2,\"red_pawn\":5,\"black_king\":1,\"black_advisor\":2,\"black_bishop\":2,\"black_knight\":2,\"black_rook\":2,\"black_cannon\":2,\"black_pawn\":5}}"
      }
    ]
  }
}
```

---

### 3.3 Tool 3: `perft_test`

- **Mô tả**: Chạy kiểm thử sinh nước đi hợp lệ Perft, đếm số lượng nút lá và phân rã chi tiết nhánh nước đi đầu tiên (Divide decomposition).
- **Tham số Đầu vào (`arguments`)**:
  - `fen` *(string, tùy chọn)*: Chuỗi FEN vị trí bàn cờ.
  - `depth` *(integer, tùy chọn)*: Độ sâu kiểm thử Perft (mặc định: `3`).

- **Luồng xử lý (Workflow)**:
  1. Phân tích chuỗi `fen` và tạo bàn cờ `pos`.
  2. Gọi `movegen::perft::perft(&mut pos, depth)` tính tổng số nút.
  3. Sinh danh sách nước đi hợp lệ ở tầng gốc, áp dụng thử từng nước đi và đếm số nút con tương ứng để dựng mảng `divide`.

- **Mẫu Yêu cầu JSON-RPC**:
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "perft_test",
    "arguments": {
      "fen": "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
      "depth": 1
    }
  }
}
```

- **Mẫu Phản hồi JSON-RPC**:
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"depth\":1,\"total_nodes\":44,\"time_ms\":0,\"nps\":0,\"divide\":[{\"move\":\"b2e2\",\"nodes\":1},{\"move\":\"b2a2\",\"nodes\":1},{\"move\":\"b2c2\",\"nodes\":1},{\"move\":\"h2e2\",\"nodes\":1},{\"move\":\"h2i2\",\"nodes\":1},{\"move\":\"h2g2\",\"nodes\":1}]}"
      }
    ]
  }
}
```

---

### 3.4 Tool 4: `parse_fen`

- **Mô tả**: Phân tích cú pháp FEN, kiểm tra số lượng 2 Tướng Đỏ/Đen, tính toán mã băm 64-bit Zobrist Hash và xuất ma trận 2D bàn cờ (10 hàng x 9 cột).
- **Tham số Đầu vào (`arguments`)**:
  - `fen` *(string, tùy chọn)*: Chuỗi FEN cần phân tích.

- **Luồng xử lý (Workflow)**:
  1. Gọi `board::fen::Parser::parse(fen)` để nạp bàn cờ.
  2. Kiểm tra `valid = pos.king[0] != 90 && pos.king[1] != 90`.
  3. Tính toán chuỗi Hex của `pos.hash` dạng `0x0000000000000000`.
  4. Quét 90 ô cờ từ hàng 9 xuống 0 để tạo ma trận ký tự `grid`.

- **Mẫu Yêu cầu JSON-RPC**:
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "parse_fen",
    "arguments": {
      "fen": "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1"
    }
  }
}
```

- **Mẫu Phản hồi JSON-RPC**:
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"fen\":\"rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1\",\"valid\":true,\"side\":\"red\",\"zobrist_hash\":\"0x0000000000000000\",\"grid\":[[\"r\",\"n\",\"b\",\"a\",\"k\",\"a\",\"b\",\"n\",\"r\"],[\".\",\".\",\".\",\".\",\".\",\".\",\".\",\".\",\".\"],[\".\",\"c\",\".\",\".\",\".\",\".\",\".\",\"c\",\".\"],[\"p\",\".\",\"p\",\".\",\"p\",\".\",\"p\",\".\",\"p\"],[\".\",\".\",\".\",\".\",\".\",\".\",\".\",\".\",\".\"],[\".\",\".\",\".\",\".\",\".\",\".\",\".\",\".\",\".\"],[\"P\",\".\",\"P\",\".\",\"P\",\".\",\"P\",\".\",\"P\"],[\".\",\"C\",\".\",\".\",\".\",\".\",\".\",\"C\",\".\"],[\".\",\".\",\".\",\".\",\".\",\".\",\".\",\".\",\".\"],[\"R\",\"N\",\"B\",\"A\",\"K\",\"A\",\"B\",\"N\",\"R\"]]}"
      }
    ]
  }
}
```

---

### 3.5 Tool 5: `get_engine_info`

- **Mô tả**: Trả về thông tin định danh hệ thống, phiên bản, tác giả, khả năng xử lý (capabilities) và danh sách 5 MCP Tools hỗ trợ.
- **Tham số Đầu vào (`arguments`)**:
  - Không có (`arguments: {}`).

- **Mẫu Yêu cầu JSON-RPC**:
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "tools/call",
  "params": {
    "name": "get_engine_info",
    "arguments": {}
  }
}
```

- **Mẫu Phản hồi JSON-RPC**:
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"name\":\"XiangRust\",\"version\":\"0.1.0\",\"author\":\"HDQB\",\"capabilities\":[\"pvs_search\",\"nnue_eval\",\"hce_eval\",\"perft_test\",\"fen_parser\",\"lazy_smp\",\"mcp_server\"],\"tools\":[\"get_best_move\",\"evaluate_position\",\"perft_test\",\"parse_fen\",\"get_engine_info\"],\"default_fen\":\"rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1\"}"
      }
    ]
  }
}
```

---

## 4. HƯỚNG DẪN TÍCH HỢP AI ASSISTANT (CLAUDE DESKTOP, CURSOR IDE, GEMINI AGENT)

Dưới đây là cấu hình tệp `mcpSettings.json` chuẩn để tích hợp XiangRust MCP Server vào các môi trường Trợ lý AI.

### 4.1 Cấu hình Tích hợp Claude Desktop (`claude_desktop_config.json`)
- **Đường dẫn cấu hình trên macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Nội dung tệp JSON**:
```json
{
  "mcpServers": {
    "xiangrust": {
      "command": "cargo",
      "args": [
        "run",
        "--example",
        "10_mcp_server",
        "--release",
        "--manifest-path",
        "/Users/hdqb/workspaces/backend-xiangqi-ai-debugger-v1/Cargo.toml"
      ]
    }
  }
}
```

### 4.2 Cấu hình Tích hợp Cursor IDE (`.cursor/mcp.json`)
- **Đường dẫn cấu hình**: `.cursor/mcp.json` trong dự án hoặc trong Cài đặt Cursor MCP Settings.
- **Nội dung tệp JSON**:
```json
{
  "mcpServers": {
    "xiangrust-ai": {
      "command": "cargo",
      "args": [
        "run",
        "--example",
        "10_mcp_server",
        "--release"
      ],
      "cwd": "/Users/hdqb/workspaces/backend-xiangqi-ai-debugger-v1"
    }
  }
}
```

### 4.3 Cấu hình Tích hợp Gemini Agent / Antigravity Agent
- **Đường dẫn cấu hình**: `~/.gemini/antigravity/mcp/xiangrust/config.json`
- **Nội dung tệp JSON**:
```json
{
  "mcpServers": {
    "xiangrust": {
      "command": "/Users/hdqb/workspaces/backend-xiangqi-ai-debugger-v1/target/release/examples/10_mcp_server",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

---
*Tài liệu được biên soạn chuẩn đặc tả MCP Server JSON-RPC 2.0.*
