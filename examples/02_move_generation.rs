// ============================================================================
// VÍ DỤ 02: MINH HỌA SINH NƯỚC ĐỊ PSEUDO, NƯỚC ĐỊ LEGAL VÀ CHẠY PERFT BENCHMARK
// ============================================================================
// Chương trình ví dụ minh họa toàn bộ các tính năng cốt lõi của module movegen:
// 1. Sinh danh sách các nước đi giả định pseudo-legal chưa qua kiểm tra chiếu Tướng.
// 2. Sinh và lọc danh sách các nước đi hợp lệ tuyệt đối legal-moves.
// 3. Thực thi thuật toán Perft benchmark đếm tổng số nút cây nước đi ở các độ sâu.
// 4. Xác minh kết quả Perft Depth 1 = 44 nodes và Depth 2 = 1,920 nodes trên vị trí FEN chuẩn.
// ============================================================================

// Nhập các mô-đun và cấu trúc dữ liệu từ thư viện xiangrust
use xiangrust::board::Parser;
use xiangrust::movegen::{legal, perft, pseudo, List};

// Khởi tạo điểm chạy chính cho chương trình ví dụ 02
fn main() {
    // 1. KHỞI TẠO BÀN CỜ VỊ TRÍ KHỞI ĐẦU VÀ NẠP NƯỚC ĐỊ GIẢ ĐỊNH (PSEUDO)
    // Khai báo chuỗi FEN vị trí ban đầu mặc định của Cờ Tướng
    let text = Parser::DEFAULT;
    // Phân tích chuỗi FEN để tạo đối tượng bàn cờ Position
    let mut pos = Parser::parse(text);
    // In vị trí FEN khởi đầu ra màn hình
    println!("FEN khởi đầu: {}", text);

    // Khởi tạo danh sách chứa các nước đi giả định (Pseudo-legal moves)
    let mut raw = List::new();
    // Sinh toàn bộ các nước đi giả định chưa qua lọc chiếu Tướng
    pseudo::gen(&pos, &mut raw);
    // Truy xuất số lượng nước đi giả định được sinh ra
    let count = raw.len();
    // In tổng số lượng nước đi giả định ra màn hình
    println!("Số nước đi giả định (pseudo): {}", count);
    // In thông tin chi tiết một vài nước đi giả định đầu tiên
    println!("Nước đi giả định đầu tiên: từ ô {} đến ô {}", raw[0].from, raw[0].to);

    // 2. SINH VÀ LỌC CÁC NƯỚC ĐỊ HỢP LỆ TUYỆT ĐỐI (LEGAL MOVES)
    // Khởi tạo danh sách chứa các nước đi hợp lệ tuyệt đối
    let mut moves = List::new();
    // Sinh và lọc các nước đi hợp lệ tuyệt đối (Legal moves)
    legal::gen(&mut pos, &mut moves);
    // Truy xuất số lượng nước đi hợp lệ tuyệt đối
    let valid = moves.len();
    // In tổng số lượng nước đi hợp lệ ra màn hình
    println!("Số nước đi hợp lệ (legal) : {}", valid);
    // In nước đi hợp lệ đầu tiên ra màn hình
    println!("Nước đi hợp lệ đầu tiên: từ ô {} đến ô {}", moves[0].from, moves[0].to);

    // Khẳng định vị trí ban đầu có đúng 44 nước đi hợp lệ
    assert_eq!(valid, 44);

    // 3. CHẠY THỬ NGHIỆM ĐẾM NÚT CÂY NƯỚC ĐỊ PERFT BENCHMARK
    // In thông báo bắt đầu đo Perft độ sâu 1
    println!("--- ĐANG CHẠY PERFT BENCHMARK ---");
    // Chạy hàm perft đếm tổng số nút cây nước đi ở độ sâu 1
    let nodes = perft::perft(&mut pos, 1);
    // In số lượng nút Perft thu được ở độ sâu 1
    println!("Perft ở độ sâu 1: {} nodes", nodes);
    // Khẳng định kết quả Perft độ sâu 1 BẮT BUỘC bằng đúng 44 nodes
    assert_eq!(nodes, 44);
    // In thông báo xác minh Perft độ sâu 1 thành công
    println!("Xác minh Perft Depth 1 = 44 nodes: THÀNH CÔNG!");

    // Chạy thử Perft ở độ sâu 2 để kiểm chứng cây đệ quy mở rộng
    let depth = perft::perft(&mut pos, 2);
    // In số lượng nút Perft thu được ở độ sâu 2
    println!("Perft ở độ sâu 2: {} nodes", depth);
    // Khẳng định kết quả Perft độ sâu 2 BẮT BUỘC bằng đúng 1,920 nodes
    assert_eq!(depth, 1920);
    // In thông báo xác minh Perft độ sâu 2 thành công
    println!("Xác minh Perft Depth 2 = 1920 nodes: THÀNH CÔNG!");
}
