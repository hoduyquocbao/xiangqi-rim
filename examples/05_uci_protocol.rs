// ============================================================================
// VÍ DỤ 05: GIAO THỨC UCI V2 (UNIVERSAL CHESS INTERFACE) TRONG XIANGRUST
// ============================================================================
// Ví dụ minh họa toàn bộ quy trình phân tích và xử lý giao thức UCI v2:
// - Phân tích chuỗi lệnh từ STDIN bằng Parser::parse.
// - Khởi tạo và thực thi các câu lệnh UCI với uci::Engine.
// - Mã hóa và giải mã định dạng nước đi UCI bằng uci::Format.
// - Chạy vòng lặp điều khiển Engine xử lý các lệnh uci, isready, position, go, quit.
// ============================================================================

// Nhập module board từ thư viện xiangrust
use xiangrust::board;
// Nhập type Move từ module movegen::types
use xiangrust::movegen::types::Move;
// Nhập enum Command từ module uci
use xiangrust::uci::Command;
// Nhập struct Engine từ module uci
use xiangrust::uci::Engine;
// Nhập struct Format từ module uci
use xiangrust::uci::Format;
// Nhập struct Parser từ module uci
use xiangrust::uci::Parser;

// Hàm chính chạy chương trình minh họa giao thức UCI
fn main() {
    // In tiêu đề thông báo khởi chạy chương trình ví dụ UCI
    println!("=== CHƯƠNG TRÌNH MINH HỌA GIAO THỨC UCI V2 ===");

    // Khởi tạo một đối tượng Engine mới với cấu hình mặc định
    let mut engine = Engine::new();

    // In thông báo phân tích lệnh uci
    println!("\n--- 1. Phân tích và thực thi lệnh 'uci' ---");
    // Khai báo chuỗi lệnh văn bản uci
    let line = "uci";
    // Phân tích chuỗi văn bản thành enum Command
    let cmd: Command = Parser::parse(line);
    // In thông tin câu lệnh đã được phân tích
    println!("Lệnh đã phân tích: {:?}", cmd);
    // Thực thi câu lệnh uci trên Engine
    let res = engine.exec(cmd);
    // In kết quả trạng thái thực thi của lệnh uci
    println!("Kết quả thực thi lệnh uci: {}", res);

    // In thông báo kiểm tra câu lệnh isready
    println!("\n--- 2. Phân tích và thực thi lệnh 'isready' ---");
    // Khai báo chuỗi văn bản lệnh isready
    let line = "isready";
    // Phân tích chuỗi lệnh isready
    let cmd: Command = Parser::parse(line);
    // In kết quả phân tích lệnh isready
    println!("Lệnh đã phân tích: {:?}", cmd);
    // Thực thi câu lệnh isready trên Engine
    let res = engine.exec(cmd);
    // In kết quả thực thi của lệnh isready
    println!("Kết quả thực thi lệnh isready: {}", res);

    // In thông báo cài đặt tùy chọn setoption
    println!("\n--- 3. Phân tích và thực thi lệnh 'setoption' ---");
    // Khai báo chuỗi lệnh setoption cài đặt Threads
    let line = "setoption name Threads value 4";
    // Phân tích chuỗi lệnh setoption
    let cmd: Command = Parser::parse(line);
    // In kết quả phân tích lệnh setoption
    println!("Lệnh đã phân tích: {:?}", cmd);
    // Thực thi câu lệnh setoption trên Engine
    let res = engine.exec(cmd);
    // In kết quả thực thi lệnh setoption
    println!("Kết quả thực thi lệnh setoption: {}", res);
    // In số lượng luồng công nhân hiện tại của Engine
    println!("Số luồng hiện tại của Engine: {}", engine.threads);

    // In thông báo thiết lập vị trí bàn cờ position
    println!("\n--- 4. Phân tích và thực thi lệnh 'position' ---");
    // Khai báo chuỗi FEN bàn cờ tiêu chuẩn khởi đầu
    let fen = board::fen::Parser::DEFAULT;
    // Tạo chuỗi câu lệnh position với FEN và danh sách nước đi
    let line = format!("position fen {} moves h2e2 h9e7", fen);
    // Phân tích chuỗi câu lệnh position
    let cmd: Command = Parser::parse(&line);
    // In kết quả phân tích lệnh position
    println!("Lệnh đã phân tích: {:?}", cmd);
    // Thực thi câu lệnh position trên Engine
    let res = engine.exec(cmd);
    // In kết quả thực thi lệnh position
    println!("Kết quả thực thi lệnh position: {}", res);

    // In thông báo khởi chạy tìm kiếm go depth 5
    println!("\n--- 5. Phân tích và thực thi lệnh 'go depth 5' ---");
    // Khai báo chuỗi lệnh go với tham số độ sâu 5
    let line = "go depth 5";
    // Phân tích chuỗi lệnh go
    let cmd: Command = Parser::parse(line);
    // In kết quả phân tích lệnh go
    println!("Lệnh đã phân tích: {:?}", cmd);
    // Thực thi câu lệnh go tìm kiếm nước đi tốt nhất
    let res = engine.exec(cmd);
    // In kết quả khởi chạy lệnh go
    println!("Kết quả thực thi lệnh go: {}", res);

    // Dừng luồng hiện tại 100ms để chờ luồng tìm kiếm hoàn tất và in bestmove
    std::thread::sleep(std::time::Duration::from_millis(100));

    // In thông báo kiểm tra mã hóa và giải mã nước đi UCI Format
    println!("\n--- 6. Mã hóa và giải mã định dạng nước đi UCI Format ---");
    // Khởi tạo một đối tượng nước đi từ ô 25 sang ô 22
    let mv = Move::new(25, 22);
    // Mã hóa đối tượng nước đi thành chuỗi ký tự UCI
    let text = Format::encode(mv);
    // In chuỗi nước đi uci đã được mã hóa
    println!("Nước đi mã hóa: {}", text);
    // Giải mã chuỗi ký tự uci trở lại đối tượng Move
    let code = Format::decode(&text);
    // In giá trị nguyên 16-bit của nước đi sau khi giải mã
    println!("Mã nước đi giải mã: {}", code.raw());

    // In thông báo thử nghiệm lệnh ngắt dừng stop
    println!("\n--- 7. Phân tích và thực thi lệnh 'stop' ---");
    // Khai báo chuỗi lệnh stop
    let line = "stop";
    // Phân tích chuỗi lệnh stop
    let cmd: Command = Parser::parse(line);
    // In kết quả phân tích lệnh stop
    println!("Lệnh đã phân tích: {:?}", cmd);
    // Thực thi câu lệnh stop ngắt tìm kiếm
    let res = engine.exec(cmd);
    // In kết quả thực thi lệnh stop
    println!("Kết quả thực thi lệnh stop: {}", res);

    // In thông báo thử nghiệm câu lệnh quit
    println!("\n--- 8. Phân tích và thực thi lệnh 'quit' ---");
    // Khai báo chuỗi lệnh quit
    let line = "quit";
    // Phân tích chuỗi lệnh quit
    let cmd: Command = Parser::parse(line);
    // In kết quả phân tích lệnh quit
    println!("Lệnh đã phân tích: {:?}", cmd);
    // Thực thi câu lệnh quit trên Engine
    let res = engine.exec(cmd);
    // In kết quả thực thi lệnh quit (trả về false để báo hiệu thoát vòng lặp)
    println!("Kết quả thực thi lệnh quit (false nghĩa là thoát): {}", res);

    // In thông báo kiểm tra Engine::run với STDIN
    println!("\n--- 9. Minh họa Engine::run ---");
    // In giải thích cơ chế vòng lặp I/O không chặn STDIN của Engine::run
    println!("Hàm Engine::run() lắng nghe STDIN và xử lý câu lệnh bất đồng bộ.");

    // In thông báo kết thúc chương trình ví dụ UCI thành công
    println!("\n=== ĐÃ HOÀN THÀNH HOÀN HẢO CHƯƠNG TRÌNH MINH HỌA GIAO THỨC UCI V2 ===");
}
