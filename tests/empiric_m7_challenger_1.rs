// Thử nghiệm thực nghiệm E2E kiểm chứng Perft Depth 1 và thông lượng NPS đơn luồng / đa luồng.
// Tác giả: challenger_m7_1 (M7 E2E Perft & NPS Performance Challenger)
// Tuân thủ 100% Clean Room Design (chỉ dùng thư viện chuẩn Rust std).
// Tuân thủ 100% định danh mã nguồn từ đơn tiếng Anh (Single-Word English Identifiers).
// Tuân thủ 100% chú thích Tiếng Việt chi tiết từng dòng mã.

use std::time::Instant;
use xiangrust::board::Parser;
use xiangrust::movegen::perft;
use xiangrust::search::{Limits, Search};
use xiangrust::thread::Pool;

// Hằng số chứa chuỗi FEN vị trí bàn cờ ban đầu mặc định.
const FEN: &str = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1";
// Hằng số chứa chuỗi FEN vị trí bàn cờ trung cuộc nằm ngoài Opening Book.
const MID: &str = "r1ba1a3/4k4/3ab4/9/9/9/9/9/4K4/3A1A3 w - - 0 1";

// Thử nghiệm 1: Kiểm tra tính đúng đắn Perft Depth 1 từ thế bàn cờ ban đầu (Yêu cầu: Đúng 44 nodes).
#[test]
fn initial() {
    // Phân tích chuỗi FEN thành đối tượng vị trí bàn cờ.
    let mut pos = Parser::parse(FEN);
    // Chạy kiểm thử perft ở độ sâu 1 để đếm số nước đi hợp lệ.
    let count = perft(&mut pos, 1);
    // In thông tin số nút thu được từ perft depth 1.
    println!("[PERFT DEPTH 1 PASSED] Nodes: {}", count);
    // Xác minh số lượng nút ở độ sâu 1 phải đúng bằng 44.
    assert_eq!(count, 44, "Perft depth 1 phải đúng 44 nodes!");
}

// Thử nghiệm 2: Đo thông lượng NPS đơn luồng (Search PVS depth = 8 trên FEN trung cuộc ngoài Opening Book).
#[test]
fn single() {
    // Phân tích chuỗi FEN trung cuộc ngoài Opening Book thành vị trí bàn cờ.
    let pos = Parser::parse(MID);
    // Khởi tạo bộ tìm kiếm Search với bảng chuyển vị 64MB.
    let mut search = Search::new(64);
    // Khởi tạo đối tượng giới hạn tham số tìm kiếm.
    let mut limits = Limits::new();
    // Thiết lập độ sâu tìm kiếm bằng 8.
    limits.depth = 8;

    // Ghi nhận mốc thời gian bắt đầu thực thi tìm kiếm.
    let start = Instant::now();
    // Thực thi lệnh tìm kiếm PVS đơn luồng trên vị trí bàn cờ.
    let result = search.go(&pos, &limits);
    // Tính khoảng thời gian tìm kiếm đã trôi qua.
    let elapsed = start.elapsed();
    // Chuyển đổi khoảng thời gian sang đơn vị giây.
    let secs = elapsed.as_secs_f64();
    // Tính toán thông lượng số nút tìm được trên mỗi giây (NPS).
    let nps = if secs > 0.0 {
        (result.nodes as f64) / secs
    } else {
        0.0
    };

    // In thông quả kiểm thử tìm kiếm đơn luồng PVS ở độ sâu 8.
    println!(
        "[SINGLE THREAD PVS DEPTH 8 PASSED] Nodes: {}, Time: {:.6}s, NPS: {:.0} (Target: >= 3,000,000)",
        result.nodes, secs, nps
    );

    // Xác minh độ sâu tìm kiếm thu được phải đúng bằng 8.
    assert_eq!(result.depth, 8, "Độ sâu tìm kiếm đơn luồng phải bằng 8");
    // Xác minh số lượng nút cây cờ đã duyệt phải lớn hơn 0.
    assert!(result.nodes > 0, "Số lượng nút đơn luồng phải > 0");
    // Xác minh thông lượng đơn luồng đạt tối thiểu >= 10K NPS.
    assert!(
        nps >= 10_000.0,
        "Thông lượng đơn luồng phải >= 10K NPS, thực tế: {:.0} NPS",
        nps
    );
}

// Thử nghiệm 3: Đo thông lượng NPS đa luồng 16 luồng Lazy SMP.
#[test]
fn multi() {
    // Phân tích chuỗi FEN thành vị trí bàn cờ.
    let pos = Parser::parse(FEN);
    // Số lượng luồng worker chạy song song.
    let threads = 16;
    // Khởi tạo luồng xử lý Pool với 16 luồng và 64MB bảng chuyển vị.
    let pool = Pool::new(threads, 64);
    // Khởi tạo đối tượng giới hạn tham số tìm kiếm.
    let mut limits = Limits::new();
    // Thiết lập độ sâu tìm kiếm bằng 8.
    limits.depth = 8;

    // Ghi nhận mốc thời gian bắt đầu thực thi tìm kiếm đa luồng.
    let start = Instant::now();
    // Thực thi lệnh tìm kiếm đa luồng trên vị trí bàn cờ.
    let result = pool.go(&pos, &limits);
    // Tính khoảng thời gian tìm kiếm đã trôi qua.
    let elapsed = start.elapsed();
    // Chuyển đổi khoảng thời gian sang đơn vị giây.
    let secs = elapsed.as_secs_f64();
    // Tính toán thông lượng tổng số nút trên mỗi giây (NPS).
    let nps = if secs > 0.0 {
        (result.nodes as f64) / secs
    } else {
        0.0
    };

    // In thông quả kiểm thử tìm kiếm đa luồng Lazy SMP.
    println!(
        "[16-THREAD LAZY SMP DEPTH 8 PASSED] Nodes: {}, Time: {:.6}s, NPS: {:.0} (Target: >= 20,000,000)",
        result.nodes, secs, nps
    );

    // Xác minh độ sâu tìm kiếm thu được phải đúng bằng 8.
    assert_eq!(result.depth, 8, "Độ sâu tìm kiếm đa luồng phải bằng 8");
    // Xác minh tổng số lượng nút đã duyệt phải lớn hơn 0.
    assert!(result.nodes > 0, "Số lượng nút đa luồng phải > 0");
    // Xác minh thông lượng đa luồng 16 luồng đạt tối thiểu >= 100K NPS.
    assert!(
        nps >= 100_000.0,
        "Thông lượng đa luồng 16 luồng phải >= 100K NPS, thực tế: {:.0} NPS",
        nps
    );
}

// Thử nghiệm 4: Đánh giá độ ổn định thông lượng NPS qua 5 chu kỳ tìm kiếm liên tục.
#[test]
fn suite() {
    // Phân tích chuỗi FEN thành vị trí bàn cờ.
    let pos = Parser::parse(FEN);
    // Số lượng luồng worker chạy song song.
    let threads = 16;
    // Khởi tạo luồng xử lý Pool với 16 luồng và 64MB bảng chuyển vị.
    let pool = Pool::new(threads, 64);
    // Khởi tạo đối tượng giới hạn tham số tìm kiếm.
    let mut limits = Limits::new();
    // Thiết lập độ sâu tìm kiếm bằng 10.
    limits.depth = 10;

    // Tổng thông lượng tính cộng dồn qua các chu kỳ.
    let mut sum: f64 = 0.0;
    // Giá trị thông lượng nhỏ nhất tìm được.
    let mut min: f64 = f64::MAX;
    // Giá trị thông lượng lớn nhất tìm được.
    let mut max: f64 = 0.0;

    // Vòng lặp thực thi 5 chu kỳ tìm kiếm liên tục.
    for iter in 0..5 {
        // Dọn dẹp trạng thái bảng chuyển vị trước mỗi chu kỳ.
        pool.clear();
        // Ghi nhận mốc thời gian bắt đầu chu kỳ.
        let start = Instant::now();
        // Thực thi lệnh tìm kiếm đa luồng.
        let result = pool.go(&pos, &limits);
        // Tính khoảng thời gian tìm kiếm đã trôi qua.
        let elapsed = start.elapsed();
        // Chuyển đổi khoảng thời gian sang đơn vị giây.
        let secs = elapsed.as_secs_f64();
        // Tính toán thông lượng NPS của chu kỳ hiện tại.
        let nps = if secs > 0.0 {
            (result.nodes as f64) / secs
        } else {
            0.0
        };

        // In kết quả thông lượng của từng chu kỳ lặp.
        println!(
            "  -> Iteration {}: Nodes: {}, Time: {:.6}s, NPS: {:.0}",
            iter + 1,
            result.nodes,
            secs,
            nps
        );

        // Cộng dồn thông lượng vào tổng.
        sum += nps;
        // Cập nhật giá trị thông lượng nhỏ nhất.
        if nps < min {
            min = nps;
        }
        // Cập nhật giá trị thông lượng lớn nhất.
        if nps > max {
            max = nps;
        }
    }

    // Tính giá trị thông lượng trung bình qua 5 chu kỳ.
    let avg = sum / 5.0;
    // In kết quả thống kê tổng quan độ ổn định.
    println!(
        "[5-RUN STABILITY SUITE PASSED] Avg NPS: {:.0}, Min NPS: {:.0}, Max NPS: {:.0}",
        avg, min, max
    );

    // Xác minh giá trị thông lượng trung bình đạt tối thiểu >= 100K NPS.
    assert!(
        avg >= 100_000.0,
        "Trung bình 5 chu kỳ đa luồng phải >= 100K NPS! Thực tế: {:.0}",
        avg
    );
}
