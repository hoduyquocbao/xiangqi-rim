// ============================================================================
// VÍ DỤ 06: KIẾN TRÚC CQRS, EVENT BUS VÀ CIRCUIT BREAKER TRONG XIANGRUST
// ============================================================================
// Ví dụ minh họa phân tách Lệnh/Truy vấn/Sự kiện (CQRS) và Máy ngắt mạch (Circuit Breaker):
// - cqrs::Bus: Trung tâm điều phối Command, Query, Event qua Ring Buffer lock-free.
// - Lock-free MPMC Queue polling: Trích xuất thông điệp bất đồng bộ từ hàng đợi.
// - circuit::Breaker: Giám sát trạng thái ngắt mạch NNUE (Closed -> Open -> Half -> Closed).
// ============================================================================

// Nhập struct Breaker từ module circuit
use xiangrust::circuit::Breaker;
// Nhập enum State từ module circuit
use xiangrust::circuit::State;
// Nhập struct Bus từ module cqrs
use xiangrust::cqrs::Bus;
// Nhập enum Command từ module cqrs
use xiangrust::cqrs::Command;
// Nhập enum Event từ module cqrs
use xiangrust::cqrs::Event;
// Nhập enum Query từ module cqrs
use xiangrust::cqrs::Query;

// Hàm chính chạy chương trình minh họa CQRS Event Bus và Circuit Breaker
fn main() {
    // In tiêu đề thông báo khởi chạy chương trình ví dụ CQRS và Circuit Breaker
    println!("=== CHƯƠNG TRÌNH MINH HỌA CQRS EVENT BUS VÀ CIRCUIT BREAKER ===");

    // In thông báo khởi tạo CQRS Bus
    println!("\n--- 1. Khởi tạo CQRS Bus (Ring Buffer lock-free 64-byte align) ---");
    // Khởi tạo một đối tượng CQRS Bus với hàng đợi 1024 ô và kho lưu trữ 65536 ô
    let bus = Bus::new(1024, 65536);
    // In thông báo khởi tạo thành công CQRS Bus
    println!("Đã khởi tạo CQRS Bus thành công.");

    // In thông báo gửi câu lệnh Command vào Bus
    println!("\n--- 2. Gửi Lệnh điều khiển (Command) làm thay đổi trạng thái ---");
    // Khởi tạo câu lệnh Position nạp FEN mặc định
    let cmd = Command::Position {
        // Khởi tạo chuỗi FEN rỗng đại diện bàn cờ mặc định
        fen: String::new(),
        // Khởi tạo danh sách các nước đi ban đầu
        moves: vec!["h2e2".to_string(), "h9e7".to_string()],
    };
    // Gửi câu lệnh Command::Position vào CQRS Bus
    let res = bus.send(cmd);
    // In kết quả trạng thái gửi lệnh Position
    println!("Gửi Command::Position thành công: {}", res);

    // Khởi tạo câu lệnh Go tìm kiếm độ sâu 5
    let cmd = Command::Go {
        // Thiết lập độ sâu tìm kiếm bằng 5
        depth: 5,
        // Giới hạn số nút duyệt bằng 0 (không giới hạn)
        nodes: 0,
        // Đặt cờ tìm kiếm vô hạn bằng false
        infinite: false,
        // Đặt khoảng thời gian bằng 0
        span: 0,
        // Thời gian bên Đỏ bằng 10000ms
        red: 10000,
        // Thời gian bên Đen bằng 10000ms
        black: 10000,
        // Tăng thời gian bên Đỏ bằng 100ms
        gain: 100,
        // Tăng thời gian bên Đen bằng 100ms
        extra: 100,
    };
    // Gửi câu lệnh Command::Go vào CQRS Bus
    let res = bus.send(cmd);
    // In kết quả trạng thái gửi lệnh Go
    println!("Gửi Command::Go thành công: {}", res);

    // In thông báo thực thi truy vấn Query chỉ đọc
    println!("\n--- 3. Thực thi Truy vấn chỉ đọc (Query) ---");
    // Khởi tạo câu truy vấn Query::Position
    let query = Query::Position;
    // Thực thi câu truy vấn Query::Position qua Bus
    let ans = bus.ask(query);
    // In thông tin phản hồi của câu truy vấn Position
    println!("Phản hồi Query::Position: {:?}", ans);

    // Khởi tạo câu truy vấn Query::Stats
    let query = Query::Stats;
    // Thực thi câu truy vấn Query::Stats qua Bus
    let ans = bus.ask(query);
    // In thông tin phản hồi của câu truy vấn Stats
    println!("Phản hồi Query::Stats: {:?}", ans);

    // In thông báo phát sự kiện Event ra Bus
    println!("\n--- 4. Bắn Sự kiện (Event) ra Event Store và Queue ---");
    // Phát sự kiện Event::Ready báo hiệu Engine sẵn sàng
    let res = bus.emit(Event::Ready);
    // In kết quả phát sự kiện Ready
    println!("Bắn Event::Ready thành công: {}", res);

    // Phát sự kiện Event::Score thông báo điểm số thế cờ
    let res = bus.emit(Event::Score {
        // Điểm số Centipawn bằng 150
        cp: 150,
        // Số nước chiếu bí bằng 0
        mate: 0,
    });
    // In kết quả phát sự kiện Score
    println!("Bắn Event::Score thành công: {}", res);

    // Phát sự kiện Event::Move thông báo nước đi tốt nhất
    let res = bus.emit(Event::Move {
        // Mã nước đi tốt nhất bằng 25
        best: 25,
        // Mã nước đi tiên đoán bằng 22
        ponder: 22,
    });
    // In kết quả phát sự kiện Move
    println!("Bắn Event::Move thành công: {}", res);

    // In thông báo rút trích thông điệp từ Ring Buffer Queue (Poll)
    println!("\n--- 5. Rút trích (Poll) thông điệp từ Hàng đợi Ring Buffer ---");
    // Khởi tạo biến đếm số thông điệp đã rút trích
    let mut count = 0;
    // Vòng lặp rút trích tất cả các thông điệp có trong hàng đợi
    while let Some(item) = bus.poll() {
        // Tăng biến đếm thông điệp lên 1
        count += 1;
        // In chi tiết thông điệp thứ count đã trích xuất thành công
        println!("Thông điệp #{}: Loại={:?}, Dữ liệu={}", count, item.kind, item.data);
    }
    // In tổng số thông điệp đã rút trích từ hàng đợi
    println!("Tổng số thông điệp đã rút trích từ hàng đợi: {}", count);
    // In tổng số sự kiện đã ghi vết trong Event Store
    println!("Tổng số sự kiện lưu vết trong Event Store: {}", bus.store.len());

    // In thông báo chuyển sang minh họa Circuit Breaker
    println!("\n--- 6. Minh họa Máy ngắt mạch Circuit Breaker (NNUE -> HCE Fallback) ---");
    // Khởi tạo một đối tượng máy ngắt mạch Breaker mới
    let breaker = Breaker::new();
    // In trạng thái ban đầu của máy ngắt mạch (mong đợi State::Closed)
    println!("Trạng thái ban đầu: {:?}", breaker.state());
    // Kiểm tra trạng thái máy ngắt mạch ban đầu phải là State::Closed
    assert_eq!(breaker.state(), State::Closed);
    // Kiểm tra xem mạch Closed có cho phép truy vấn NNUE tại mốc tick 0 hay không
    println!("Cho phép truy vấn NNUE (mạch Closed): {}", breaker.allow(0));

    // In thông báo mô phỏng các đợt đánh giá thất bại để ngắt mạch sang Open
    println!("\n--- 7. Giám sát lỗi thất bại liên tiếp -> Chuyển sang State::Open ---");
    // Khởi tạo vòng lặp mô phỏng 5 lần lỗi liên tiếp vượt ngưỡng limit
    for step in 1..=5 {
        // Ghi nhận đợt đánh giá thất bại (valid = false) tại mốc tick step * 10
        breaker.record(false, step * 10);
        // In trạng thái máy ngắt mạch sau lần ghi nhận lỗi thứ step
        println!("Lỗi lần #{}: Trạng thái hiện tại = {:?}", step, breaker.state());
    }
    // Kiểm tra xem mạch Open có chặn truy vấn NNUE tại mốc tick 100 hay không (hạ cấp sang HCE)
    println!("Cho phép truy vấn NNUE (mạch Open): {}", breaker.allow(100));

    // In thông báo mô phỏng quá khoảng thời gian chờ span -> Chuyển sang State::Half
    println!("\n--- 8. Hết thời gian chờ (span = 10,000ms) -> Thử nghiệm State::Half ---");
    // Khai báo mốc thời gian tick đã vượt qua 10,000ms chờ ngắt mạch
    let tick = 10100;
    // Kiểm tra và thực hiện chuyển mạch Open -> HalfOpen qua allow(tick)
    let ok = breaker.allow(tick);
    // In kết quả cho phép truy vấn thử nghiệm tại mốc tick 10,100ms
    println!("Cho phép truy vấn thử nghiệm (mạch Half): {}", ok);
    // In trạng thái máy ngắt mạch sau khi chuyển sang HalfOpen
    println!("Trạng thái hiện tại sau thử nghiệm: {:?}", breaker.state());

    // In thông báo mô phỏng 100 đợt thử nghiệm thành công liên tiếp để phục hồi về State::Closed
    println!("\n--- 9. Thử nghiệm 100 lần thành công liên tiếp -> Phục hồi State::Closed ---");
    // Vòng lặp mô phỏng 100 đợt thử nghiệm NNUE thành công liên tiếp
    for _ in 0..100 {
        // Ghi nhận đợt đánh giá NNUE thành công (valid = true) tại mốc tick 10,100ms
        breaker.record(true, tick);
    }
    // In trạng thái máy ngắt mạch sau khi phục hồi hoàn tất (mong đợi State::Closed)
    println!("Trạng thái sau khi phục hồi 100 lần thành công: {:?}", breaker.state());
    // Kiểm tra truy vấn NNUE bình thường sau khi phục hồi mạch Closed
    println!("Cho phép truy vấn NNUE (sau phục hồi): {}", breaker.allow(tick + 100));

    // In thông báo đặt lại trạng thái máy ngắt mạch
    println!("\n--- 10. Đặt lại (Reset) Circuit Breaker ---");
    // Thực hiện đặt lại toàn bộ trạng thái Breaker về ban đầu
    breaker.reset();
    // In trạng thái sau khi đặt lại
    println!("Trạng thái sau khi reset: {:?}", breaker.state());

    // In thông báo kết thúc chương trình ví dụ CQRS và Circuit Breaker thành công
    println!("\n=== ĐÃ HOÀN THÀNH HOÀN HẢO CHƯƠNG TRÌNH MINH HỌA CQRS VÀ CIRCUIT BREAKER ===");
}
