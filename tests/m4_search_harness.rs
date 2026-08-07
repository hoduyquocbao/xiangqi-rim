// Thử nghiệm thực nghiệm kiểm chứng cho Module 4 (Động cơ Tìm kiếm Search & Bảng Chuyển vị TT).
// Tuân thủ 100% Clean Room Design (chỉ dùng thư viện chuẩn Rust std).
// Tuân thủ 100% định danh mã nguồn từ đơn tiếng Anh (Single-Word English Identifiers).
// Tuân thủ 100% chú thích Tiếng Việt chi tiết từng dòng mã.

use std::time::Instant;

use xiangrust::board::Parser;
use xiangrust::movegen::{legal, List};
use xiangrust::search::{Limits, Search};

// Kiểm tra tìm kiếm cơ bản trên bàn cờ vị trí khởi đầu mặc định.
#[test]
fn initial() {
    // Phân tích vị trí bàn cờ mặc định ban đầu.
    let mut pos = Parser::parse(Parser::DEFAULT);
    // Khởi tạo đối tượng tìm kiếm với bảng chuyển vị 16MB.
    let mut search = Search::new(16);
    // Khởi tạo đối tượng giới hạn tham số tìm kiếm.
    let mut limits = Limits::new();
    // Thiết lập độ sâu tìm kiếm bằng 4.
    limits.depth = 4;

    // Thực thi lệnh tìm kiếm trên vị trí bàn cờ hiện tại.
    let res = search.go(&pos, &limits);
    // Xác minh nước đi tốt nhất thu được phải hợp lệ.
    assert!(res.best.valid(), "Tìm kiếm ban đầu phải trả về nước đi hợp lệ!");
    // Xác minh số nút duyệt phải lớn hơn 0.
    assert!(res.nodes > 0, "Tìm kiếm ban đầu phải duyệt > 0 nút!");
    // Xác minh điểm số bàn cờ ban đầu ở mức cân bằng (trị tuyệt đối nhỏ hơn 5000).
    assert!(res.score.abs() < 5000, "Điểm số ban đầu phải gần mức cân bằng!");

    // Khởi tạo danh sách chứa các nước đi hợp lệ.
    let mut list = List::new();
    // Sinh tất cả nước đi hợp lệ của vị trí bàn cờ hiện tại.
    legal::gen(&mut pos, &mut list);
    // Cờ đánh dấu tìm thấy nước đi tốt nhất trong danh sách.
    let mut found = false;
    // Biến chỉ số vòng lặp duyệt danh sách.
    let mut i = 0;
    // Vòng lặp kiểm tra từng nước đi trong danh sách hợp lệ.
    while i < list.count {
        if list.items[i] == res.best {
            found = true;
            break;
        }
        i += 1;
    }
    // Xác minh nước đi tốt nhất BẮT BUỘC nằm trong danh sách hợp lệ.
    assert!(found, "Nước đi tốt nhất phải nằm trong danh sách nước đi hợp lệ!");
}

// Kiểm tra tìm kiếm trên vị trí bàn cờ trung cuộc.
#[test]
fn midgame() {
    // Chuỗi FEN biểu diễn vị trí bàn cờ trung cuộc.
    let fen = "r1ba1a3/4k4/3ab4/9/9/9/9/9/4K4/3A1A3 w - - 0 1";
    // Phân tích chuỗi FEN thành đối tượng vị trí bàn cờ.
    let mut pos = Parser::parse(fen);
    // Khởi tạo đối tượng tìm kiếm với bảng chuyển vị 16MB.
    let mut search = Search::new(16);
    // Khởi tạo đối tượng giới hạn tham số tìm kiếm.
    let mut limits = Limits::new();
    // Thiết lập độ sâu tìm kiếm bằng 4.
    limits.depth = 4;

    // Thực thi lệnh tìm kiếm trên vị trí bàn cờ trung cuộc.
    let res = search.go(&pos, &limits);
    // Xác minh nước đi tốt nhất thu được phải hợp lệ.
    assert!(res.best.valid(), "Tìm kiếm trung cuộc phải trả về nước đi hợp lệ!");
    // Xác minh số nút duyệt phải lớn hơn 0.
    assert!(res.nodes > 0, "Tìm kiếm trung cuộc phải duyệt > 0 nút!");

    // Khởi tạo danh sách chứa các nước đi hợp lệ.
    let mut list = List::new();
    // Sinh tất cả nước đi hợp lệ của vị trí bàn cờ trung cuộc.
    legal::gen(&mut pos, &mut list);
    // Cờ đánh dấu tìm thấy nước đi tốt nhất trong danh sách.
    let mut found = false;
    // Biến chỉ số vòng lặp duyệt danh sách.
    let mut i = 0;
    // Vòng lặp kiểm tra từng nước đi trong danh sách hợp lệ.
    while i < list.count {
        if list.items[i] == res.best {
            found = true;
            break;
        }
        i += 1;
    }
    // Xác minh nước đi tốt nhất BẮT BUỘC nằm trong danh sách hợp lệ.
    assert!(found, "Nước đi tốt nhất trong trung cuộc phải hợp lệ!");
}

// Kiểm tra tìm kiếm nước chiếu bí (Mate Score).
#[test]
fn mate() {
    // Chuỗi FEN biểu diễn vị trí chiếu bí thế Red thắng.
    let fen = "3pkp3/9/9/9/9/9/9/9/4R4/4K4 w - - 0 1";
    // Phân tích chuỗi FEN thành đối tượng vị trí bàn cờ.
    let pos = Parser::parse(fen);
    // Khởi tạo đối tượng tìm kiếm với bảng chuyển vị 16MB.
    let mut search = Search::new(16);
    // Khởi tạo đối tượng giới hạn tham số tìm kiếm.
    let mut limits = Limits::new();
    // Thiết lập độ sâu tìm kiếm bằng 4.
    limits.depth = 4;

    // Thực thi lệnh tìm kiếm trên thế bàn cờ chiếu bí.
    let res = search.go(&pos, &limits);
    // In kết quả kiểm thử nước đi và điểm số chiếu bí.
    println!("MATE TEST RESULT: best from={}, to={}, score={}", res.best.from, res.best.to, res.score);
    // Xác minh nước đi tốt nhất thu được phải hợp lệ.
    assert!(res.best.valid(), "Vị trí chiếu bí phải trả về nước đi hợp lệ!");
    // Xác minh điểm số thu được phải tiệm cận MATE (lớn hơn 29000).
    assert!(res.score > 29000, "Điểm số chiếu bí phải tiệm cận giá trị MATE!");
}

// Kiểm tra tìm kiếm trên vị trí bàn cờ cờ tàn.
#[test]
fn endgame() {
    // Chuỗi FEN biểu diễn vị trí bàn cờ cờ tàn Xe thắng Đơn Tướng.
    let fen = "3k5/9/9/9/9/9/9/9/4K4/4R4 w - - 0 1";
    // Phân tích chuỗi FEN thành đối tượng vị trí bàn cờ.
    let pos = Parser::parse(fen);
    // Khởi tạo đối tượng tìm kiếm với bảng chuyển vị 16MB.
    let mut search = Search::new(16);
    // Khởi tạo đối tượng giới hạn tham số tìm kiếm.
    let mut limits = Limits::new();
    // Thiết lập độ sâu tìm kiếm bằng 4.
    limits.depth = 4;

    // Thực thi lệnh tìm kiếm trên thế cờ tàn.
    let res = search.go(&pos, &limits);
    // Xác minh nước đi tốt nhất thu được phải hợp lệ.
    assert!(res.best.valid(), "Tìm kiếm cờ tàn phải trả về nước đi hợp lệ!");
    // Xác minh số nút duyệt phải lớn hơn 0.
    assert!(res.nodes > 0, "Tìm kiếm cờ tàn phải duyệt > 0 nút!");
    // Xác minh điểm số Red có ưu thế quân vượt trội (lớn hơn 100).
    assert!(res.score > 100, "Red phải có ưu thế điểm số vượt trội!");
}

// Kiểm tra giới hạn thời gian cực ngắn (1ms).
#[test]
fn zero() {
    // Phân tích vị trí bàn cờ mặc định ban đầu.
    let pos = Parser::parse(Parser::DEFAULT);
    // Khởi tạo đối tượng tìm kiếm với bảng chuyển vị 16MB.
    let mut search = Search::new(16);
    // Khởi tạo đối tượng giới hạn tham số tìm kiếm.
    let mut limits = Limits::new();
    // Thiết lập giới hạn thời gian chính xác 1ms.
    limits.exact = 1;

    // Ghi nhận mốc thời gian bắt đầu thực thi tìm kiếm.
    let start = Instant::now();
    // Thực thi lệnh tìm kiếm với thời gian 1ms.
    let res = search.go(&pos, &limits);
    // Tính khoảng thời gian thực tế đã trôi qua (ms).
    let elapsed = start.elapsed().as_millis();

    // Xác minh thời gian chạy không vượt quá 500ms.
    assert!(elapsed < 500, "Tìm kiếm 1ms phải thoát nhanh lập tức!");
    // Xác minh số nút duyệt phải lớn hơn 0.
    assert!(res.nodes > 0, "Tìm kiếm phải thực thi và duyệt ít nhất một số nút!");
}

// Kiểm tra giới hạn số nút tìm kiếm tối đa (Node count limit).
#[test]
fn deep() {
    // Chuỗi FEN trung cuộc nằm ngoài Opening Book để PVS search thực hiện duyệt cây đạt giới hạn nodes.
    let fen = "r1ba1a3/4k4/3ab4/9/9/9/9/9/4K4/3A1A3 w - - 0 1";
    // Phân tích chuỗi FEN trung cuộc thành vị trí bàn cờ.
    let pos = Parser::parse(fen);
    // Khởi tạo đối tượng tìm kiếm với bảng chuyển vị 16MB.
    let mut search = Search::new(16);
    // Khởi tạo đối tượng giới hạn tham số tìm kiếm.
    let mut limits = Limits::new();
    // Thiết lập độ sâu tối đa 20.
    limits.depth = 20;
    // Thiết lập giới hạn số nút tối đa là 1000.
    limits.nodes = 1000;

    // Thực thi lệnh tìm kiếm PVS trên vị trí bàn cờ trung cuộc.
    let res = search.go(&pos, &limits);
    // Xác minh tổng số nút đã duyệt phải đạt >= 1000 theo đúng giới hạn.
    assert!(res.nodes >= 1000, "Giới hạn số nút tối đa phải được tôn trọng!");
}

// Kiểm tra phát lệnh dừng khẩn cấp trước khi tìm kiếm (Pre-halt search).
#[test]
fn halt() {
    // Phân tích vị trí bàn cờ mặc định ban đầu.
    let pos = Parser::parse(Parser::DEFAULT);
    // Khởi tạo đối tượng tìm kiếm với bảng chuyển vị 16MB.
    let mut search = Search::new(16);
    // Khởi tạo đối tượng giới hạn tham số tìm kiếm.
    let mut limits = Limits::new();
    // Thiết lập độ sâu tối đa 20.
    limits.depth = 20;

    // Phát lệnh ngắt dừng tìm kiếm khẩn cấp trước.
    search.halt();
    // Thực thi lệnh tìm kiếm.
    let res = search.go(&pos, &limits);
    // Xác minh thời gian trả về nhỏ hơn 500ms.
    assert!(res.time < 500, "Tìm kiếm bị ngắt dừng phải trả về nhanh chóng!");
}

// Kiểm tra trường hợp bên đi bị hết nước (Mated side score).
#[test]
fn nomoves() {
    // Chuỗi FEN đại diện thế cờ bên đi đã bị hết nước đi / bị chiếu bí.
    let fen = "3pkp3/4R4/9/9/9/9/9/9/9/4K4 b - - 0 1";
    // Phân tích chuỗi FEN thành vị trí bàn cờ.
    let pos = Parser::parse(fen);
    // Khởi tạo đối tượng tìm kiếm với bảng chuyển vị 16MB.
    let mut search = Search::new(16);
    // Khởi tạo đối tượng giới hạn tham số tìm kiếm.
    let mut limits = Limits::new();
    // Thiết lập độ sâu tìm kiếm bằng 2.
    limits.depth = 2;

    // Thực thi lệnh tìm kiếm.
    let res = search.go(&pos, &limits);
    // In kết quả kiểm thử điểm số hết nước.
    println!("NOMOVES TEST RESULT: best from={}, to={}, score={}", res.best.from, res.best.to, res.score);
    // Xác minh điểm số bên bị hết nước phải nhỏ hơn -29000.
    assert!(res.score < -29000, "Điểm số bên bị hết nước/bị chiếu bí phải tiệm cận -30000!");
}

// Kiểm tra tính bất biến của mã băm Zobrist hash trước và sau khi tìm kiếm.
#[test]
fn hash() {
    // Phân tích vị trí bàn cờ mặc định ban đầu.
    let pos = Parser::parse(Parser::DEFAULT);
    // Tính toán mã băm Zobrist kỳ vọng của vị trí bàn cờ.
    let expected = pos.compute();
    // Xác minh mã băm Zobrist ban đầu phải trùng khớp với mã băm tính toán.
    assert_eq!(pos.hash, expected, "Mã băm Zobrist ban đầu phải khớp với kết quả tính toán!");

    // Khởi tạo đối tượng tìm kiếm với bảng chuyển vị 16MB.
    let mut search = Search::new(16);
    // Khởi tạo đối tượng giới hạn tham số tìm kiếm.
    let mut limits = Limits::new();
    // Thiết lập độ sâu tìm kiếm bằng 4.
    limits.depth = 4;
    // Thực thi lệnh tìm kiếm.
    search.go(&pos, &limits);

    // Xác minh mã băm Zobrist của bàn cờ sau khi tìm kiếm không bị thay đổi.
    assert_eq!(pos.hash, expected, "Mã băm Zobrist sau khi tìm kiếm phải giữ nguyên!");
}

// Kiểm tra hiệu năng tốc độ tìm kiếm ở độ sâu 6.
#[test]
fn speed() {
    // Phân tích vị trí bàn cờ mặc định ban đầu.
    let pos = Parser::parse(Parser::DEFAULT);
    // Khởi tạo đối tượng tìm kiếm với bảng chuyển vị 16MB.
    let mut search = Search::new(16);
    // Khởi tạo đối tượng giới hạn tham số tìm kiếm.
    let mut limits = Limits::new();
    // Thiết lập độ sâu tìm kiếm bằng 6.
    limits.depth = 6;

    // Ghi nhận mốc thời gian bắt đầu thực thi tìm kiếm.
    let start = Instant::now();
    // Thực thi lệnh tìm kiếm.
    let res = search.go(&pos, &limits);
    // Tính khoảng thời gian đã trôi qua.
    let elapsed = start.elapsed();
    // Chuyển đổi sang đơn vị giây.
    let seconds = elapsed.as_secs_f64();

    // Tính toán thông lượng NPS.
    let nps = if seconds > 0.0 {
        (res.nodes as f64 / seconds) as u64
    } else {
        0
    };

    // In thông tin tốc độ tìm kiếm.
    println!("Search depth 6: nodes = {}, time = {:.3}s, NPS = {}", res.nodes, seconds, nps);
    // Xác minh số nút đã duyệt phải lớn hơn 0.
    assert!(res.nodes > 0, "Kiểm thử tốc độ phải duyệt số nút > 0!");
}
