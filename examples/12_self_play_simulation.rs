// ============================================================================
// VÍ DỤ 12: MÔ PHỎNG TỰ ĐẤU CỜ TƯỚNG TỰ ĐỘNG (SELF-PLAY MATCH SIMULATION)
// ============================================================================
// Chương trình ví dụ minh họa tiến trình mô phỏng tự đấu giữa hai AI Engine:
// 1. Cấu hình các thông số ván đấu như độ sâu tìm kiếm, thời gian, giới hạn nước.
// 2. Khởi chạy ván tự đấu bằng `Runner::play` điều phối hai lượt đi Đỏ và Đen.
// 3. Thu thập và hiển thị chi tiết các thông số hiệu năng như Nodes, NPS, Time/move.
// 4. Kiểm tra và in ra kết quả chung cuộc (Thắng, Thua, Hòa, Lặp nước, Giới hạn).
// 5. Tái tạo bàn cờ và xuất chuỗi FEN đại diện cho thế cờ kết thúc ván đấu.
// 6. Xuất toàn bộ dữ liệu biên bản ván đấu ra định dạng PGN chuẩn Cờ Tướng.
// 100% chú thích tiếng Việt từng dòng & 100% định danh đơn từ tiếng Anh.
// ============================================================================

// Nhập module Parser phân tích vị trí bàn cờ từ thư viện xiangrust
use xiangrust::board::Parser;
// Nhập các kiểu dữ liệu và module tự đấu từ thư viện xiangrust
use xiangrust::selfplay::{Config, Fen, Outcome, Pgn, Runner, Side, Stats};

// Hàm chính main thực thi ví dụ mô phỏng tự đấu Cờ Tướng
fn main() {
    // In tiêu đề trang trí khởi đầu chương trình minh họa tự đấu
    println!("============================================================");
    // In dòng chữ giới thiệu ví dụ 12 về mô phỏng ván tự đấu
    println!("  XIANGRUST AI ENGINE - VÍ DỤ 12: SELF-PLAY SIMULATION      ");
    // In dòng phân cách trang trí
    println!("============================================================");

    // 1. CẤU HÌNH THÔNG SỐ VÁN TỰ ĐẤU (CONFIG INITIALIZATION)
    // Thiết lập độ sâu tìm kiếm tối đa cho mỗi nước đi (depth = 3)
    let depth = 3u8;
    // Thiết lập giới hạn thời gian tính toán cho mỗi nước đi tính bằng ms (time = 500ms)
    let time = 500u64;
    // Thiết lập giới hạn số nước đi tối đa của ván đấu (limit = 20 nước)
    let limit = 20u32;

    // Khởi tạo đối tượng cấu hình Config từ các tham số đã định nghĩa
    let config = Config::new(depth, time, limit);
    // In thông báo đã thiết lập cấu hình tự đấu ra màn hình
    println!("\n[1] ĐÃ KHỞI TẠO CẤU HÌNH TỰ ĐẤU:");
    // In chi tiết độ sâu tìm kiếm cấu hình
    println!(" -> Độ sâu tìm kiếm (depth): {}", config.depth);
    // In chi tiết giới hạn thời gian cấu hình
    println!(" -> Thời gian/nước đi (time): {} ms", config.time);
    // In chi tiết giới hạn số nước đi cấu hình
    println!(" -> Giới hạn nước đi (limit): {}", config.limit);

    // 2. THỰC THI VÁN TỰ ĐẤU BẰNG RUNNER (RUNNING SELF-PLAY MATCH)
    // In thông báo bắt đầu khởi chạy ván tự đấu
    println!("\n[2] BẮT ĐẦU MÔ PHỎNG VÁN TỰ ĐẤU...");
    // Khởi chạy tiến trình tự đấu thông qua phương thức static Runner::play
    let game = Runner::play(&config);
    // In thông báo hoàn thành ván tự đấu
    println!(" -> Ván tự đấu đã hoàn tất thành công!");

    // 3. THỐNG KÊ VÀ HIỂN THỊ CHỈ SỐ HIỆU NĂNG (PERFORMANCE METRICS)
    // Trích xuất đối tượng Stats lưu trữ thống kê từ kết quả ván đấu
    let stats: Stats = game.stats;
    // Lấy tổng số nút cây cờ đã duyệt trong suốt ván đấu
    let nodes = stats.nodes;
    // Lấy tổng thời gian tính toán thực tế của toàn bộ ván đấu (ms)
    let time = stats.time;
    // Lấy tốc độ duyệt nút trên giây NPS (Nodes Per Second)
    let nps = stats.nps;
    // Lấy tổng số nước đi đã được thực hiện trong ván đấu
    let moves = stats.moves;
    // Tính trung bình số nút duyệt trên mỗi nước đi bằng hàm mean
    let mean = stats.mean();
    // Tính trung bình thời gian tính toán trên mỗi nước đi bằng hàm span
    let span = stats.span();

    // In mục 3 tiêu đề hiển thị các chỉ số hiệu năng thực thi
    println!("\n[3] CHỈ SỐ HIỆU NĂNG VÁN ĐẤU (MATCH STATS):");
    // In tổng số nước đi thực tế đã di chuyển
    println!(" -> Tổng số nước đi đã thực hiện : {}", moves);
    // In tổng số nút cây cờ đã duyệt
    println!(" -> Tổng số nút đã duyệt (nodes)   : {}", nodes);
    // In tổng thời gian tính toán thực tế
    println!(" -> Tổng thời gian tính toán (time): {} ms", time);
    // In tốc độ duyệt nút trung bình NPS
    println!(" -> Tốc độ duyệt nút (NPS)         : {} nodes/sec", nps);
    // In số nút trung bình trên mỗi nước đi
    println!(" -> Số nút trung bình/nước (mean)  : {} nodes/move", mean);
    // In thời gian trung bình trên mỗi nước đi
    println!(" -> Thời gian trung bình/nước(span): {} ms/move", span);

    // 4. KIỂM TRA VÀ HIỂN THỊ KẾT QUẢ CHUNG CUỘC (MATCH OUTCOME)
    // Trích xuất trạng thái kết quả chung cuộc Outcome từ ván đấu
    let outcome = game.outcome;
    // In mục 4 tiêu đề hiển thị kết quả chung cuộc ván đấu
    println!("\n[4] KẾT QUẢ CHUNG CUỘC (MATCH OUTCOME):");
    // Phân tích từng trường hợp kết quả outcome bằng biểu thức match
    match outcome {
        // Trường hợp bên Side thắng tuyệt đối do chiếu bí hoặc đối thủ hết nước đi
        Outcome::Win(side) => match side {
            // Trường hợp Bên Đỏ (Red) giành chiến thắng
            Side::Red => println!(" -> Kết quả: BÊN ĐỎ THẮNG (Red Win)!"),
            // Trường hợp Bên Đen (Black) giành chiến thắng
            Side::Black => println!(" -> Kết quả: BÊN ĐEN THẮNG (Black Win)!"),
        },
        // Trường hợp Hòa cờ tiêu chuẩn theo quy định
        Outcome::Draw => println!(" -> Kết quả: HÒA CỜ (Draw)!"),
        // Trường hợp Hòa lặp nước 3 lần (3-fold repetition loop)
        Outcome::Loop => println!(" -> Kết quả: HÒA LẶP NƯỚC (3-fold Repetition Loop)!"),
        // Trường hợp Hòa do chạm mốc giới hạn số nước đi cấu hình
        Outcome::Limit => println!(" -> Kết quả: HÒA GIỚI HẠN NƯỚC ĐỊ (Move Limit Reached)!"),
    }

    // 5. TÁI TẠO BÀN CỜ VÀ XUẤT FEN THẾ CỜ CUỐI CÙNG (FINAL FEN EXPORT)
    // Khởi tạo đối tượng bàn cờ từ chuỗi FEN xuất phát mặc định
    let mut pos = Parser::parse(Parser::DEFAULT);
    // Lặp qua từng nước đi đã thực hiện trong danh sách moves của ván đấu
    for mv in &game.moves {
        // Áp dụng từng nước đi lên bàn cờ để tái tạo thế cờ cuối cùng
        pos.apply(mv.from, mv.to);
    }
    // Xuất chuỗi FEN đại diện cho thế cờ cuối cùng thông qua struct Fen
    let fen = Fen::export(&pos);
    // In mục 5 tiêu đề hiển thị chuỗi FEN thế cờ cuối cùng
    println!("\n[5] CHUỖI FEN THẾ CỜ CUỐI CÙNG (FINAL FEN):");
    // In chuỗi FEN kết quả ra màn hình console
    println!(" -> FEN: {}", fen);

    // 6. XUẤT BIÊN BẢN VÁN ĐẤU RA ĐỊNH DẠNG PGN (PGN EXPORT)
    // Xuất toàn bộ dữ liệu ván đấu ra chuỗi văn bản định dạng PGN bằng Pgn::export
    let pgn = Pgn::export(&game);
    // In mục 6 tiêu đề hiển thị biên bản PGN Cờ Tướng
    println!("\n[6] BIÊN BẢN VÁN ĐẤU PGN CỜ TƯỚNG (PGN FORMAT):");
    // In dòng phân cách đầu chuỗi PGN
    println!("------------------------------------------------------------");
    // In toàn bộ chuỗi văn bản PGN đã xuất
    println!("{}", pgn);
    // In dòng phân cách kết thúc chuỗi PGN
    println!("------------------------------------------------------------");

    // In thông báo kết thúc thành công toàn bộ chương trình ví dụ 12
    println!("\n=> HOÀN THÀNH CHƯƠNG TRÌNH VÍ DỤ 12 MÔ PHỎNG TỰ ĐẤU!");
}
