// ============================================================================
// VÍ DỤ 10: MÁY CHỦ MCP SERVER (MODEL CONTEXT PROTOCOL JSON-RPC 2.0 OVER STDIN/STDOUT)
// ============================================================================
// Chương trình minh họa khởi chạy MCP Server thuần Rust std (Clean Room Design 0đ):
// - Vận hành theo mô hình I/O dòng văn bản JSON-RPC 2.0 qua STDIN / STDOUT.
// - Hỗ trợ đầy đủ các phương thức chuẩn: "initialize", "tools/list", "tools/call".
// - Cung cấp bộ 5 MCP Tools cho Trợ lý AI (Claude, Gemini, Cursor):
//   1. get_best_move: Tìm nước đi tốt nhất bằng PVS Lazy SMP.
//   2. evaluate_position: Chấm điểm thế cờ bằng NNUE & HCE.
//   3. perft_test: Đếm tổng số nút lá perft và phân rã nhánh hợp lệ.
//   4. parse_fen: Thẩm định FEN, tính Zobrist hash & ma trận 90 ô cờ.
//   5. get_engine_info: Xuất thông tin định danh & capabilities của XiangRust.
// ============================================================================

// Nhập module mcp từ thư viện xiangrust
use xiangrust::mcp;

/// Hàm chính khởi chạy máy chủ MCP Server
fn main() {
    // Khởi tạo một đối tượng máy chủ MCP Server mới
    let mut server = mcp::Server::new();

    // In thông báo hướng dẫn khởi chạy máy chủ ra cổng log nội bộ (STDERR)
    eprintln!("============================================================================");
    eprintln!("XIANGRUST MCP SERVER — MODEL CONTEXT PROTOCOL (JSON-RPC 2.0)");
    eprintln!("============================================================================");
    eprintln!("- Máy chủ đang chạy ở chế độ STDIN / STDOUT line-delimited JSON-RPC 2.0.");
    eprintln!("- Đã đăng ký 5 MCP Tools: get_best_move, evaluate_position, perft_test, parse_fen, get_engine_info.");
    eprintln!("- Sẵn sàng nhận câu lệnh từ các Trợ lý AI hoặc Client...");
    eprintln!("============================================================================");

    // Kích hoạt vòng lặp lắng nghe STDIN và phản hồi STDOUT của máy chủ MCP
    server.run();
}
