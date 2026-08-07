// ============================================================================
// VÍ DỤ 01: MINH HỌA KHỞI TẠO BÀN CỜ, XỬ LÝ FEN, BĂM ZOBRIST VÀ THAO TÁC BITBOARD
// ============================================================================
// Chương trình ví dụ minh họa toàn bộ các tính năng cốt lõi của module board:
// 1. Đọc và phân tích chuỗi FEN Cờ Tướng thành đối tượng Position.
// 2. Xuất ngược đối tượng Position ra chuỗi FEN chuẩn.
// 3. Tính toán và xác minh mã băm ngẫu nhiên Zobrist Hashing.
// 4. Thao tác bật (set), tắt (clear), kiểm tra (test), và rút bit (pop) trên Bitboard.
// ============================================================================

// Nhập các kiểu dữ liệu và hằng số từ module board của thư viện xiangrust
use xiangrust::board::{Bitboard, Parser, Serializer, Square, KEYS};

// Khởi tạo điểm chạy chính cho chương trình ví dụ 01
fn main() {
    // 1. MINH HỌA KHỞI TẠO BÀN CỜ VÀ ĐỌC/XUẤT CHUỖI FEN
    // Khai báo chuỗi FEN mặc định vị trí khởi đầu bàn cờ Cờ Tướng
    let text = Parser::DEFAULT;
    // Phân tích chuỗi FEN để khởi tạo đối tượng bàn cờ Position
    let pos = Parser::parse(text);
    // Xuất ngược đối tượng Position thành chuỗi FEN chuẩn
    let out = Serializer::export(&pos);
    // In ra màn hình chuỗi FEN ban đầu
    println!("FEN ban đầu : {}", text);
    // In ra màn hình chuỗi FEN xuất ngược từ đối tượng Position
    println!("FEN xuất ra: {}", out);
    // Kiểm tra tính nhất quán 2 chiều giữa FEN ban đầu và FEN xuất ra
    assert_eq!(text, out);

    // Truy xuất thông tin phe đến lượt đi (0: Đỏ, 1: Đen)
    let side = pos.side;
    // In thông tin phe đến lượt đi ra màn hình
    println!("Phe đến lượt đi: {}", side);

    // Truy xuất bộ đếm 50 nước hòa rule50
    let rule = pos.rule;
    // In bộ đếm rule50 ra màn hình
    println!("Bộ đếm rule50: {}", rule);

    // Truy xuất số nửa nước đi ply
    let ply = pos.ply;
    // In số nửa nước đi ply ra màn hình
    println!("Số nửa nước đi ply: {}", ply);

    // 2. MINH HỌA TÍNH TOÁN VÀ KIỂM TRA MÃ BĂM ZOBRIST HASHING
    // Lấy khóa băm Zobrist Hash hiện tại đã được lưu trong đối tượng Position
    let hash = pos.hash;
    // Tính toán lại khóa băm Zobrist Hash từ trạng thái thực tế của bàn cờ
    let calc = pos.compute();
    // In khóa băm Zobrist Hash lưu trong bàn cờ
    println!("Zobrist Hash hiện tại : {:#018X}", hash);
    // In khóa băm Zobrist Hash tự tính lại
    println!("Zobrist Hash tính lại : {:#018X}", calc);
    // Khẳng định khóa băm lưu trữ và khóa băm tự tính toán hoàn toàn trùng khớp
    assert_eq!(hash, calc);

    // Truy xuất khóa băm Zobrist đại diện cho lượt đi của bên Đen từ KEYS
    let key = KEYS.side();
    // In khóa băm Zobrist lượt đi bên Đen
    println!("Zobrist Hash lượt Đen : {:#018X}", key);

    // 3. MINH HỌA THAO TÁC TRÊN MẶT NẠ BITBOARD (128-BIT)
    // Khởi tạo một đối tượng Bitboard rỗng không chứa bit 1 nào
    let mut bb = Bitboard::empty();
    // In trạng thái hoạt động ban đầu của Bitboard (rỗng -> false)
    println!("Bitboard ban đầu active: {}", bb.active());
    // Kiểm tra số lượng bit 1 ban đầu bằng 0
    assert_eq!(bb.count(), 0);

    // Khởi tạo ô cờ tại vị trí ô 4 (tương ứng Tướng Đỏ e1 / ô 4)
    let sq = Square(4);
    // Bật bit tại vị trí ô 4 trên Bitboard
    bb.set(sq);
    // In trạng thái active sau khi bật bit (true)
    println!("Bitboard sau set(4) active: {}", bb.active());
    // Kiểm tra xem bit tại ô 4 có đang bật hay không (test -> true)
    let test = bb.test(sq);
    // In kết quả kiểm tra test(4)
    println!("Bitboard test(4): {}", test);
    // Khẳng định kết quả test(4) là true
    assert!(test);
    // Đếm số bit 1 đang bật trong Bitboard (bằng 1)
    let count = bb.count();
    // In số lượng bit 1 đang bật
    println!("Số lượng bit 1: {}", count);
    // Khẳng định số lượng bit 1 đúng bằng 1
    assert_eq!(count, 1);

    // Rút bit 1 thấp nhất out khỏi Bitboard bằng hàm pop()
    let pop = bb.pop();
    // In vị trí ô cờ vừa rút ra khỏi Bitboard
    println!("Ô cờ vừa pop(): {:?}", pop);
    // Khẳng định ô cờ vừa pop chính là ô 4
    assert_eq!(pop, Some(sq));
    // In trạng thái active sau khi pop (rỗng -> false)
    println!("Bitboard sau pop() active: {}", bb.active());
    // Khẳng định Bitboard đã trở về rỗng sau khi pop
    assert!(!bb.active());

    // Tắt bit tại ô 4 trên Bitboard để minh họa phương thức clear()
    bb.clear(sq);
    // Đảm bảo bit tại ô 4 đã bị tắt
    assert!(!bb.test(sq));
}
