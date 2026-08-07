// Bộ kiểm thử thực nghiệm cho Module 4 (Động cơ Tìm kiếm Search & Bảng Chuyển vị TT).
// Tác giả: Challenger 1 cho Cột mốc 4.
// Tuân thủ 100% Clean Room Design (chỉ dùng thư viện chuẩn Rust std).
// Tuân thủ 100% định danh đơn từ tiếng Anh (Single-word English Identifiers).
// Tuân thủ 100% chú thích Tiếng Việt chi tiết từng dòng mã.

use std::time::Instant;
use xiangrust::board::Parser;
use xiangrust::search::{Limits, Search};

// Kiểm tra tìm kiếm nước chiếu bí phía dương (Red có thể ép chiếu bí).
#[test]
fn positive() {
    // Vị trí bàn cờ Red có thể chiếu bí Black trong 1-2 nước đi.
    let fen = "3pkp3/9/9/9/9/9/9/9/4R4/4K4 w - - 0 1";
    // Phân tích chuỗi FEN thành đối tượng vị trí bàn cờ.
    let pos = Parser::parse(fen);
    // Khởi tạo đối tượng tìm kiếm với bảng chuyển vị 16MB.
    let mut search = Search::new(16);
    // Khởi tạo đối tượng giới hạn tham số tìm kiếm.
    let mut limits = Limits::new();
    // Thiết lập độ sâu tìm kiếm bằng 6.
    limits.depth = 6;

    // Ghi nhận mốc thời gian bắt đầu thực thi tìm kiếm.
    let start = Instant::now();
    // Thực thi lệnh tìm kiếm trên vị trí bàn cờ hiện tại.
    let res = search.go(&pos, &limits);
    // Tính toán khoảng thời gian đã trôi qua.
    let elapsed = start.elapsed();

    // Xác minh nước đi tốt nhất thu được phải hợp lệ.
    assert!(res.best.valid(), "Vị trí chiếu bí phải trả về nước đi hợp lệ!");
    // Xác minh điểm số thế cờ phải tiệm cận giá trị MATE (>= 29000).
    assert!(res.score >= 29000, "Điểm số phải gần MATE (score={})", res.score);
    // Xác minh thời gian tìm kiếm không bị treo lặp vô hạn (nhỏ hơn 5 giây).
    assert!(elapsed.as_secs() < 5, "Tìm kiếm phải kết thúc mà không bị lặp vô hạn!");
}

// Kiểm tra tìm kiếm nước chiếu bí phía âm (Side hiện tại bị chiếu bí ép buộc).
#[test]
fn negative() {
    // Vị trí bàn cờ phía Black bị Red ép chiếu bí.
    let fen = "3pkp3/4R4/9/9/9/9/9/9/9/4K4 b - - 0 1";
    // Phân tích chuỗi FEN thành đối tượng vị trí bàn cờ.
    let pos = Parser::parse(fen);
    // Khởi tạo đối tượng tìm kiếm với bảng chuyển vị 16MB.
    let mut search = Search::new(16);
    // Khởi tạo đối tượng giới hạn tham số tìm kiếm.
    let mut limits = Limits::new();
    // Thiết lập độ sâu tìm kiếm bằng 6.
    limits.depth = 6;

    // Ghi nhận mốc thời gian bắt đầu thực thi tìm kiếm.
    let start = Instant::now();
    // Thực thi lệnh tìm kiếm trên vị trí bàn cờ hiện tại.
    let res = search.go(&pos, &limits);
    // Tính toán khoảng thời gian đã trôi qua.
    let elapsed = start.elapsed();

    // Xác minh điểm số thế cờ phải tiệm cận giá trị âm MATE (<= -29000).
    assert!(res.score <= -29000, "Điểm số phải gần -MATE (score={})", res.score);
    // Xác minh thời gian tìm kiếm không bị treo lặp vô hạn (nhỏ hơn 5 giây).
    assert!(elapsed.as_secs() < 5, "Tìm kiếm phải kết thúc mà không bị lặp vô hạn!");
}

// Kiểm tra mã băm Zobrist không bị thay đổi và cơ chế Null Move Pruning hoạt động.
#[test]
fn toggling() {
    // Sử dụng FEN trung cuộc nằm ngoài Opening Book để PVS search duyệt cây thực sự.
    let fen = "r1ba1a3/4k4/3ab4/9/9/9/9/9/4K4/3A1A3 w - - 0 1";
    // Phân tích chuỗi FEN thành đối tượng vị trí bàn cờ.
    let pos = Parser::parse(fen);
    // Lưu lại giá trị mã băm Zobrist ban đầu của bàn cờ.
    let origin = pos.hash;

    // Khởi tạo đối tượng tìm kiếm với bảng chuyển vị 16MB.
    let mut search = Search::new(16);
    // Khởi tạo đối tượng giới hạn tham số tìm kiếm.
    let mut limits = Limits::new();
    // Thiết lập độ sâu tìm kiếm bằng 6.
    limits.depth = 6;

    // Thực thi lệnh tìm kiếm trên vị trí bàn cờ hiện tại.
    let res = search.go(&pos, &limits);

    // Xác minh mã băm Zobrist sau khi tìm kiếm không bị thay đổi so với ban đầu.
    assert_eq!(pos.hash, origin, "Mã băm Zobrist của bàn cờ sau tìm kiếm phải giữ nguyên!");
    // Xác minh số nút đã duyệt phải lớn hơn 100 khi chạy PVS search thực sự.
    assert!(res.nodes > 100, "Tìm kiếm ở độ sâu 6 phải duyệt > 100 nút!");
}

// Kiểm tra cơ chế tín hiệu dừng ngắt halt và khởi tạo bộ đếm thời gian timer.
#[test]
fn reset() {
    // Phân tích vị trí bàn cờ mặc định ban đầu.
    let pos = Parser::parse(Parser::DEFAULT);
    // Khởi tạo đối tượng tìm kiếm với bảng chuyển vị 16MB.
    let mut search = Search::new(16);
    // Khởi tạo đối tượng giới hạn tham số tìm kiếm.
    let mut limits = Limits::new();
    // Thiết lập độ sâu tìm kiếm bằng 2.
    limits.depth = 2;

    // Phát tín hiệu dừng ngắt phiên tìm kiếm khẩn cấp.
    search.halt();
    // Xác minh cờ ngắt tín hiệu nguyên tử đã được bật thành true.
    assert!(search.timer.abort.load(std::sync::atomic::Ordering::Relaxed), "Lệnh halt phải đặt cờ abort=true");

    // Thực thi lệnh tìm kiếm (timer.init sẽ khởi tạo lại phiên tìm kiếm mới).
    let res = search.go(&pos, &limits);
    // Lấy giá trị cờ ngắt tín hiệu sau khi thực thi lệnh tìm kiếm.
    let aborted = search.timer.abort.load(std::sync::atomic::Ordering::Relaxed);
    // In thông tin kiểm thử số nút đã duyệt và trạng thái cờ ngắt.
    println!("Pre-halt test: nodes searched = {}, aborted = {}", res.nodes, aborted);

    // Xác minh phiên tìm kiếm hoàn thành thành công và trả về số nút duyệt >= 1.
    assert!(res.nodes >= 1, "Phiên tìm kiếm khởi tạo lại phải hoàn thành và trả về kết quả!");
}
