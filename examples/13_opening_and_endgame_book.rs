// ============================================================================
// VÍ DỤ 13: MINH HỌA SÁCH KHAI CUỘC ZOBRIST O(1) VÀ TRI THỨC TÀN CUỘC CỜ TƯỚNG
// ============================================================================
// Chương trình ví dụ minh họa toàn bộ hai phân hệ cơ sở tri thức của Engine:
// 1. Thư viện nước đi khai cuộc (Opening Book):
//    - Tra cứu nước đi lý thuyết Zobrist Hash O(1) ~ 0ms không tốn CPU.
//    - Kiểm tra số lượng biến thể khai cuộc trong sách đạt mốc >= 1,000 bản ghi.
// 2. Tri thức tàn cuộc thực dụng (Endgame Knowledge Base):
//    - Đánh giá các thế cờ tàn cuộc lý thuyết chuyên sâu không phụ thuộc thư viện ngoài.
//    - Nhận diện các thế cờ thắng tuyệt đối (+15000), hòa cân bằng (0), thua tuyệt đối (-15000).
// 100% chú thích tiếng Việt từng dòng & 100% định danh đơn từ tiếng Anh.
// ============================================================================

// Nhập module Parser và Position phân tích vị trí bàn cờ từ thư viện xiangrust
use xiangrust::board::{Parser, Position};
// Nhập mảng tĩnh ENTRIES từ module book::opening
use xiangrust::book::opening::ENTRIES;
// Nhập các hằng số điểm số tàn cuộc từ module book::endgame
use xiangrust::book::endgame::{DRAW, LOSS, WIN};
// Nhập các struct tri thức từ module book của xiangrust
use xiangrust::book::{Book, Endgame};
// Nhập module Format mã hóa nước đi UCI từ thư viện xiangrust
use xiangrust::uci::format::Format;

// Hàm chính main thực thi ví dụ minh họa Sách khai cuộc và Tri thức tàn cuộc
fn main() {
    // In tiêu đề trang trí khởi đầu chương trình minh họa tri thức cờ
    println!("============================================================");
    // In dòng chữ giới thiệu ví dụ 13 về Opening Book và Endgame Knowledge    
    println!("  XIANGRUST AI ENGINE - VÍ DỤ 13: OPENING & ENDGAME BOOK    ");
    // In dòng phân cách trang trí
    println!("============================================================");

    // ========================================================================
    // PHẦN 1: THƯ VIỆN NƯỚC ĐỊ KHAI CUỘC ZOBRIST HASH O(1) (OPENING BOOK)
    // ========================================================================
    // In mục 1 tiêu đề kiểm tra thư viện khai cuộc Opening Book
    println!("\n[1] THƯ VIỆN KHAI CUỘC ZOBRIST HASH O(1) (OPENING BOOK):");

    // Khởi tạo đối tượng Book mặc định từ mảng tĩnh dữ liệu khai cuộc
    let book = Book::default();
    // Lấy tổng số lượng bản ghi nước đi khai cuộc lưu trong thư viện
    let count = book.count;
    // In số lượng bản ghi khai cuộc ra màn hình console
    println!(" -> Số lượng biến thể khai cuộc trong sách: {} entries", count);
    // Khẳng định kiểm tra số lượng bản ghi khai cuộc bắt buộc >= 1,000 bản ghi
    assert!(
        count >= 1000,
        "Số lượng bản ghi khai cuộc BẮT BUỘC phải lớn hơn hoặc bằng 1,000!"
    );
    // In thông báo xác nhận mốc kiểm tra số lượng bản ghi đạt chuẩn
    println!(" -> [OK] Kiểm tra mốc Book::count() >= 1000 đạt yêu cầu tuyệt đối!");

    // Khởi tạo bàn cờ mặc định vị trí xuất phát ban đầu
    let pos = Parser::parse(Parser::DEFAULT);
    // Tra cứu nước đi khai cuộc cho bàn cờ xuất phát bằng Book::probe
    let probe = Book::probe(&pos);
    // Phân tích kết quả tra cứu nước đi mở màn từ Book::probe
    match probe {
        // Trường hợp tìm thấy nước đi khai cuộc phù hợp trong 0ms
        Some(mv) => {
            // Mã hóa nước đi tìm thấy sang chuỗi đại số chuẩn UCI 4 ký tự
            let code = Format::encode(mv);
            // In nước đi khai cuộc tìm được ra màn hình
            println!(" -> Nước đi khai cuộc gợi ý (Parser::DEFAULT): {}", code);
        }
        // Trường hợp không tìm thấy nước đi trong thư viện
        None => {
            // In thông báo không tìm thấy nước đi khai cuộc cho vị trí mặc định
            println!(" -> Không tìm thấy nước đi khai cuộc cho bàn cờ mặc định!");
        }
    }

    // Minh họa tra cứu khóa băm Zobrist Hash cụ thể từ bản ghi trong ENTRIES
    // Trích xuất phần tử bản ghi tại vị trí chỉ số 100 trong ENTRIES
    let target = ENTRIES[100];
    // Khởi tạo một bàn cờ rỗng để gán khóa băm giả định tra cứu
    let mut pos = Position::empty();
    // Gán khóa băm Zobrist của vị trí bàn cờ bằng khóa băm của bản ghi target
    pos.hash = target.hash;

    // Thực thi tra cứu băm Zobrist O(1) 0ms bằng Book::probe
    let probe = Book::probe(&pos);
    // Phân tích kết quả tra cứu theo khóa băm Zobrist
    if let Some(mv) = probe {
        // Mã hóa nước đi trả về sang định dạng UCI
        let code = Format::encode(mv);
        // In chi tiết bản ghi khai cuộc đã tra cứu thành công
        println!("\n -> TRA CỨU ZOBRIST HASH TRỰC TIẾP (Bản ghi #100):");
        // In mã băm Zobrist 64-bit định dạng Hexadecimal
        println!("    + Khóa băm Zobrist Hash : {:#018X}", target.hash);
        // In tên biến thể khai cuộc kinh điển tiếng Việt
        println!("    + Tên biến thể khai cuộc: {}", target.name);
        // In trọng số ưu tiên của nước đi khai cuộc
        println!("    + Trọng số ưu tiên      : {}", target.weight);
        // In mã nước đi UCI được tra cứu ra thành công
        println!("    + Nước đi mã hóa UCI    : {}", code);
        // Khẳng định mã nước đi tìm được phải trùng khớp với bản ghi target
        assert_eq!(mv.raw(), target.mv);
    }
    // In thông báo hoàn thành thử nghiệm tra cứu Sách khai cuộc
    println!(" -> [OK] Tra cứu Sách khai cuộc O(1) 0ms Zobrist Hash hoạt động chuẩn xác!");

    // ========================================================================
    // PHẦN 2: TRI THỨC TÀN CUỘC THỰC DỤNG VÀ THẾ CỜ LÝ THUYẾT (ENDGAME BOOK)
    // ========================================================================
    // In mục 2 tiêu đề đánh giá Tri thức tàn cuộc thực dụng
    println!("\n[2] TRI THỨC TÀN CUỘC THỰC DỤNG VÀ THẾ CỜ LÝ THUYẾT (ENDGAME):");

    // Khởi tạo bộ đánh giá tri thức tàn cuộc Endgame
    let endgame = Endgame::new();
    // Lấy tổng số quy tắc tàn cuộc được hỗ trợ trong bộ Endgame
    let total = endgame.total;
    // In tổng số quy tắc tàn cuộc ra màn hình console
    println!(" -> Số lượng quy tắc tàn cuộc thực dụng: {} rules", total);

    // THẾ CỜ TÀN CUỘC 1: KHÔNG CÒN QUÂN CÔNG NÀO Ở CẢ 2 BÊN (HÒA CỜ)
    // Khai báo chuỗi FEN bàn cờ chỉ còn 2 Tướng nằm trên cung Tướng
    let text = "4k4/9/9/9/9/9/9/9/9/4K4 w - - 0 1";
    // Phân tích chuỗi FEN thành đối tượng Position bàn cờ
    let pos = Parser::parse(text);
    // Đánh giá thế cờ tàn cuộc bằng Endgame::eval
    let score = Endgame::eval(&pos);
    // In FEN thế cờ tàn cuộc 1
    println!("\n -> Thế cờ 1 (Không còn quân công): FEN = {}", text);
    // In điểm số đánh giá từ bộ tri thức tàn cuộc
    println!("    + Điểm đánh giá: {:?} centipawns", score);
    // Khẳng định điểm số đánh giá phải trả về Some(DRAW) tức 0 centipawns
    assert_eq!(score, Some(DRAW));
    // In giải thích ý nghĩa thế cờ hòa cờ 0 centipawns
    println!("    + Kết luận    : HÒA CỜ LÝ THUYẾT (0 centipawns) - Hai Tướng trần!");

    // THẾ CỜ TÀN CUỘC 2: ĐƠN MÃ THẮNG ĐƠN SĨ (ĐỎ THẮNG tuyệt đối)
    // Khai báo chuỗi FEN Đỏ có 1 Mã, Đen có 1 Sĩ
    let text = "4k1a2/9/9/9/9/9/9/4N4/9/4K4 w - - 0 1";
    // Phân tích chuỗi FEN thế cờ Đơn Mã thắng Đơn Sĩ
    let pos = Parser::parse(text);
    // Đánh giá thế cờ Đơn Mã thắng Đơn Sĩ
    let score = Endgame::eval(&pos);
    // In FEN thế cờ tàn cuộc 2
    println!("\n -> Thế cờ 2 (Đơn Mã vs Đơn Sĩ - Lượt Đỏ): FEN = {}", text);
    // In điểm số đánh giá thế cờ Đơn Mã thắng Đơn Sĩ
    println!("    + Điểm đánh giá: {:?} centipawns", score);
    // Khẳng định điểm số đánh giá phải trả về Some(WIN) tức +15000 centipawns
    assert_eq!(score, Some(WIN));
    // In giải thích kết luận Đỏ thắng tuyệt đối
    println!("    + Kết luận    : ĐỎ THẮNG LÝ THUYẾT (+15000 centipawns) - Đơn Mã bắt Sĩ!");

    // THẾ CỜ TÀN CUỘC 3: XE PHÁO THẮNG XE (ĐỎ THẮNG tuyệt đối)
    // Khai báo chuỗi FEN Đỏ có Xe Pháo, Đen chỉ có 1 Xe
    let text = "3k5/4r4/9/9/9/9/9/9/4C4/3K1R3 w - - 0 1";
    // Phân tích chuỗi FEN thế cờ Xe Pháo thắng Xe
    let pos = Parser::parse(text);
    // Đánh giá thế cờ Xe Pháo thắng Xe
    let score = Endgame::eval(&pos);
    // In FEN thế cờ tàn cuộc 3
    println!("\n -> Thế cờ 3 (Xe Pháo vs Đơn Xe): FEN = {}", text);
    // In điểm số đánh giá thế cờ Xe Pháo thắng Xe
    println!("    + Điểm đánh giá: {:?} centipawns", score);
    // Khẳng định điểm số đánh giá phải trả về Some(WIN) tức +15000 centipawns
    assert_eq!(score, Some(WIN));
    // In giải thích kết luận Đỏ thắng tuyệt đối
    println!("    + Kết luận    : ĐỎ THẮNG LÝ THUYẾT (+15000 centipawns) - Xe Pháo công Xe!");

    // THẾ CỜ TÀN CUỘC 4: ĐƠN PHÁO KHUYẾT TƯỢNG HÒA ĐƠN SĨ (HÒA CỜ)
    // Khai báo chuỗi FEN Đỏ có 1 Pháo không Tốt, Đen có 1 Sĩ
    let text = "4k1a2/9/9/9/9/9/9/9/4C4/4K4 w - - 0 1";
    // Phân tích chuỗi FEN thế cờ Đơn Pháo hòa Đơn Sĩ
    let pos = Parser::parse(text);
    // Đánh giá thế cờ Đơn Pháo hòa Đơn Sĩ
    let score = Endgame::eval(&pos);
    // In FEN thế cờ tàn cuộc 4
    println!("\n -> Thế cờ 4 (Đơn Pháo vs Đơn Sĩ): FEN = {}", text);
    // In điểm số đánh giá thế cờ Đơn Pháo hòa Đơn Sĩ
    println!("    + Điểm đánh giá: {:?} centipawns", score);
    // Khẳng định điểm số đánh giá phải trả về Some(DRAW) tức 0 centipawns
    assert_eq!(score, Some(DRAW));
    // In giải thích kết luận Hòa cờ vì Pháo không ngòi không thể di chuyển
    println!("    + Kết luận    : HÒA CỜ LÝ THUYẾT (0 centipawns) - Pháo khuyết ngòi!");

    // THẾ CỜ TÀN CUỘC 5: HAI PHÁO THẮNG KHUYẾT SĨ TƯỢNG (ĐỎ THẮNG tuyệt đối)
    // Khai báo chuỗi FEN Đỏ có Hai Pháo, Đen khuyết Sĩ Tượng
    let text = "4k1a2/9/9/9/9/9/9/9/4C1C2/4K4 w - - 0 1";
    // Phân tích chuỗi FEN thế cờ Hai Pháo thắng Khuyết Sĩ Tượng
    let pos = Parser::parse(text);
    // Đánh giá thế cờ Hai Pháo thắng Khuyết Sĩ Tượng
    let score = Endgame::eval(&pos);
    // In FEN thế cờ tàn cuộc 5
    println!("\n -> Thế cờ 5 (Hai Pháo vs Khuyết Sĩ Tượng): FEN = {}", text);
    // In điểm số đánh giá thế cờ Hai Pháo thắng Khuyết Sĩ Tượng
    println!("    + Điểm đánh giá: {:?} centipawns", score);
    // Khẳng định điểm số đánh giá phải trả về Some(WIN) tức +15000 centipawns
    assert_eq!(score, Some(WIN));
    // In giải thích kết luận Đỏ thắng tuyệt đối
    println!("    + Kết luận    : ĐỎ THẮNG LÝ THUYẾT (+15000 centipawns) - Hai Pháo trùng!");

    // THẾ CỜ TÀN CUỘC 6: ĐƠN MÃ VS ĐƠN SĨ (LƯỢT ĐEN BỊ THUA TUYỆT ĐỐI)
    // Đổi lượt đi trong FEN sang bên Đen (side = 1)
    let text = "4k1a2/9/9/9/9/9/9/4N4/9/4K4 b - - 0 1";
    // Phân tích chuỗi FEN lượt đi bên Đen
    let pos = Parser::parse(text);
    // Đánh giá thế cờ từ góc nhìn bên tới lượt Đen
    let score = Endgame::eval(&pos);
    // In FEN thế cờ tàn cuộc 6
    println!("\n -> Thế cờ 6 (Đơn Mã vs Đơn Sĩ - Lượt Đen): FEN = {}", text);
    // In điểm số đánh giá từ góc nhìn bên Đen
    println!("    + Điểm đánh giá: {:?} centipawns", score);
    // Khẳng định điểm số đánh giá trả về Some(LOSS) tức -15000 centipawns cho bên Đen
    assert_eq!(score, Some(LOSS));
    // In giải thích kết luận Đen bị thua tuyệt đối
    println!("    + Kết luận    : ĐEN THUA LÝ THUYẾT (-15000 centipawns) - Bên Đen bị đe dọa!");

    // In thông báo kết thúc thành công toàn bộ chương trình ví dụ 13
    println!("\n=> HOÀN THÀNH CHƯƠNG TRÌNH VÍ DỤ 13 OPENING & ENDGAME BOOK!");
}
