// ============================================================================
// VÍ DỤ 08: ĐO LƯỜNG HIỆU QUẢ TỐI ƯU HÓA CẮT TỈA CÂY TÌM KIẾM
// ============================================================================
// Benchmark đo hiệu quả thực tế của các kỹ thuật cắt tỉa (Pruning Efficiency):
// - Futility Pruning: Cắt nước đi yên lặng vô vọng ở depth <= 2
// - History Malus: Phạt nước đi thất bại để cải thiện move ordering
// - MVV-LVA Ordering: Sắp xếp nước ăn quân trong QSearch theo giá trị
// - Delta Pruning: Loại bỏ nước ăn quân vô vọng trong QSearch
// Metric: Số nút duyệt (nodes) và thời gian (ms) tại các depth khác nhau.
// ============================================================================

// Nhập module Parser từ thư viện xiangrust để phân tích chuỗi FEN
use xiangrust::board::Parser;
// Nhập module Limits và Search từ thư viện xiangrust
use xiangrust::search::{Limits, Search};
// Nhập module Format để mã hóa nước đi sang chuỗi UCI
use xiangrust::uci::format::Format;

// Hàm chính benchmark đo hiệu quả cắt tỉa
fn main() {
    // In tiêu đề chương trình benchmark
    println!("===============================================================================");
    println!("  XIANGRUST — ĐO LƯỜNG HIỆU QUẢ TỐI ƯU HÓA CẮT TỈA (PRUNING EFFICIENCY)");
    println!("===============================================================================");
    println!();

    // Khởi tạo Search Engine với 32 MB Transposition Table
    let mb = 32usize;
    let mut search = Search::new(mb);
    println!("[INFO] Search Engine khởi tạo với {} MB TT", mb);
    println!();

    // Danh sách các vị trí FEN kiểm thử
    let positions: [(&str, &str); 3] = [
        ("Ban đầu (Initial)", Parser::DEFAULT),
        ("Trung cuộc (Midgame)", "r1bakab1r/9/1cn4c1/p1p1p1p1p/9/2P6/P3P1P1P/1C2C1N2/9/R1BAKAB1R w - - 0 5"),
        ("Tàn cuộc (Endgame)", "2bak4/4a4/4b4/9/9/3R5/9/4B4/4A4/2B1K4 w - - 0 1"),
    ];

    // Danh sách các depth kiểm thử
    let depths: [u8; 4] = [4, 6, 8, 10];

    // In tiêu đề bảng kết quả
    println!("{:<24} {:>6} {:>14} {:>12} {:>10} {:>8}", "Vị trí", "Depth", "Nodes", "Time(ms)", "NPS", "Best");
    println!("{}", "-".repeat(80));

    // Duyệt từng vị trí FEN
    for (name, fen) in &positions {
        // Duyệt từng depth
        for depth in &depths {
            // Phân tích chuỗi FEN thành đối tượng Position
            let pos = Parser::parse(fen);

            // Làm sạch bộ nhớ Engine trước mỗi thử nghiệm
            search.clear();

            // Cấu hình Limits với depth cố định
            let mut limit = Limits::new();
            limit.depth = *depth;

            // Thực thi tìm kiếm PVS
            let res = search.go(&pos, &limit);

            // Tính NPS (Nodes Per Second)
            let nps = if res.time > 0 { res.nodes * 1000 / res.time } else { 0 };

            // Mã hóa nước đi tốt nhất sang UCI
            let best = Format::encode(res.best);

            // In kết quả từng dòng
            println!(
                "{:<24} {:>6} {:>14} {:>12} {:>10} {:>8}",
                name, depth, res.nodes, res.time, nps, best
            );
        }
        // In dòng phân cách giữa các vị trí
        println!("{}", "-".repeat(80));
    }

    // In chú thích giải thích kết quả
    println!();
    println!("===============================================================================");
    println!("  CHÚ THÍCH PHÂN TÍCH KẾT QUẢ:");
    println!("  - Nodes: Tổng số nút cây cờ đã duyệt (ít hơn = cắt tỉa tốt hơn)");
    println!("  - NPS: Tốc độ xử lý nút trên giây (cao hơn = engine nhanh hơn)");
    println!("  - Futility Pruning: Giảm nodes ở depth 1-2 (nước đi yên lặng vô vọng)");
    println!("  - MVV-LVA + Delta: Giảm nodes QSearch (sắp xếp + loại ăn quân vô vọng)");
    println!("  - History Malus: Cải thiện thứ tự duyệt (move ordering chính xác hơn)");
    println!("  - MaybeUninit: Bỏ zero-init 512 bytes/nút cho NNUE Transform");
    println!("  - SIMD ClipReLU: Vectorize hidden layer 32 phần tử i32→i8");
    println!("===============================================================================");
}
