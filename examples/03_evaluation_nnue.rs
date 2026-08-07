// ============================================================================
// VÍ DỤ 03: BỘ ĐÁNH GIÁ MẠNG NƠ-RON NNUE VÀ DỰ PHÒNG HCE TRONG XIANGRUST
// ============================================================================
// Tệp ví dụ độc lập minh họa kiến trúc mạng nơ-ron NNUE HalfKAv2_hm, cập nhật
// gia tăng bộ tích lũy Accumulator O(1), trích xuất chỉ số đặc trưng Feature,
// và cơ chế hạ cấp dự phòng HCE kết hợp ngắt mạch Circuit Breaker.
// 100% chú thích tiếng Việt từng dòng & 100% định danh đơn từ tiếng Anh.
// ============================================================================

// Nhập module Parser từ thư viện xiangrust
use xiangrust::board::Parser;
// Nhập module Accum, Eval, Feature, Hce, Mode từ thư viện xiangrust
use xiangrust::eval::{Accum, Eval, Feature, Hce, Mode};

// Hàm chính main thực thi ví dụ minh họa NNUE và HCE
fn main() {
    // In tiêu đề trang trí khởi đầu chương trình minh họa
    println!("============================================================");
    // In tiêu đề giới thiệu ví dụ 03 về bộ đánh giá thế cờ NNUE
    println!("  XIANGRUST AI ENGINE - VÍ DỤ 03: EVALUATION & NNUE & HCE   ");
    // In dòng kẻ phân cách trang trí
    println!("============================================================");

    // Khởi tạo bàn cờ mặc định vị trí xuất phát từ chuỗi FEN chuẩn
    let mut pos = Parser::parse(Parser::DEFAULT);
    // In thông báo bàn cờ xuất phát đã được khởi tạo thành công
    println!("\n[1] Đã khởi tạo bàn cờ xuất phát thành công!");

    // Khởi tạo bộ đánh giá Eval chứa NNUE, HCE và Circuit Breaker
    let mut eval = Eval::new();
    // In thông báo bộ đánh giá Eval đã được khởi tạo thành công
    println!("[2] Đã khởi tạo bộ đánh giá Eval thành công!");

    // Đặt lại bộ tích lũy Accumulator từ đầu theo trạng thái bàn cờ hiện tại
    eval.reset(&pos);
    // In thông báo đã reset bộ tích lũy Accumulator ban đầu
    println!("[3] Đã nạp và reset bộ tích lũy Accumulator cho bàn cờ!");

    // Tính điểm thế cờ ở chế độ tự động Auto (NNUE kết hợp HCE fallback)
    let raw = eval.score(&pos);
    // In điểm số thế cờ ban đầu ở chế độ Auto
    println!(" -> Điểm đánh giá ban đầu (Auto Mode): {} centipawns", raw);

    // Chuyển sang chế độ ép buộc đánh giá bằng luật tĩnh thủ công HCE
    eval.mode(Mode::Hce);
    // Tính điểm thế cờ với chế độ HCE
    let hce = eval.score(&pos);
    // In điểm số thế cờ đánh giá bằng HCE
    println!(" -> Điểm đánh giá luật tĩnh (HCE Mode): {} centipawns", hce);

    // Khôi phục lại chế độ đánh giá tự động Auto
    eval.mode(Mode::Auto);
    // In thông báo đã đặt lại chế độ Auto
    println!(" -> Đã đặt lại chế độ đánh giá về Auto.");

    // In dòng phân cách mục kiểm thử tính toán chỉ số đặc trưng
    println!("\n------------------------------------------------------------");
    // In tiêu đề mục 4: Tính toán chỉ số đặc trưng Feature Index
    println!("[4] TRÍCH XUẤT CHỈ SỐ ĐẶC TRƯNG FEATURE INDEX (HalfKAv2_hm)");
    // In dòng phân cách
    println!("------------------------------------------------------------");

    // Lấy vị trí Tướng Đỏ trên bàn cờ xuất phát (ô 4 - rank 0, file 4)
    let red = pos.king[0];
    // Lấy vị trí Tướng Đen trên bàn cờ xuất phát (ô 85 - rank 9, file 4)
    let black = pos.king[1];
    // In vị trí Tướng Đỏ và Tướng Đen
    println!(" -> Vị trí Tướng Đỏ: ô {}, Tướng Đen: ô {}", red, black);

    // Khai báo quân cờ thử nghiệm: Chariot (Xe Đỏ = 4)
    let piece = 4u8;
    // Khai báo ô vị trí thử nghiệm của Xe Đỏ (ô 0 - Xe góc)
    let sq = 0u8;
    // Khai báo phe sở hữu quân cờ: Đỏ (0)
    let side = 0u8;
    // Khai báo góc nhìn đánh giá: Góc nhìn Đỏ (0)
    let view = 0u8;
    // Khai báo góc nhìn đánh giá: Góc nhìn Đen (1)
    let other = 1u8;

    // Tính toán chỉ số đặc trưng duy nhất theo góc nhìn phe Đỏ
    let idx = Feature::index(red, piece, sq, side, view);
    // In chỉ số đặc trưng trích xuất theo góc nhìn Đỏ
    println!(" -> Chỉ số đặc trưng (Góc nhìn Đỏ): {}", idx);

    // Tính toán chỉ số đặc trưng duy nhất theo góc nhìn phe Đen (tự động lật dọc)
    let alt = Feature::index(black, piece, sq, side, other);
    // In chỉ số đặc trưng trích xuất theo góc nhìn Đen
    println!(" -> Chỉ số đặc trưng (Góc nhìn Đen): {}", alt);

    // Thử nghiệm hàm lật dọc tọa độ bàn cờ flip (ngược hàng 0 <-> 9)
    let flip = Feature::flip(sq);
    // In kết quả lật dọc ô cờ 0
    println!(" -> Lật dọc ô {} thu được ô {}", sq, flip);

    // Thử nghiệm hàm lật đối xứng ngang mirror (ngược cột 0 <-> 8)
    let mirror = Feature::mirror(sq);
    // In kết quả lật ngang ô cờ 0
    println!(" -> Lật ngang ô {} thu được ô {}", sq, mirror);

    // In dòng phân cách mục kiểm thử bộ tích lũy Accumulator
    println!("\n------------------------------------------------------------");
    // In tiêu đề mục 5: Thử nghiệm cập nhật gia tăng O(1) Accumulator
    println!("[5] CẬP NHẬT GIA TĂNG ACCUMULATOR (Apply & Revert O(1))");
    // In dòng phân cách
    println!("------------------------------------------------------------");

    // Khai báo ô xuất phát của Pháo Đỏ (ô 19 - Cột 1 Hàng 2)
    let from = 19u8;
    // Khai báo ô đích đến của Pháo Đỏ (ô 22 - Pháo 2 thối 3 sang Pháo Trung Cung)
    let to = 22u8;
    // Lấy quân cờ di chuyển tại ô xuất phát
    let moving = pos.grid[from as usize];
    // Lấy quân cờ bị ăn tại ô đích đến (ô 22 rỗng = 14)
    let captured = pos.grid[to as usize];

    // Sao chép trạng thái bộ tích lũy trước khi đi nước cờ
    let initial = eval.accum;
    // In thông báo đã lưu bộ tích lũy ban đầu
    println!(" -> Trạng thái bộ tích lũy ban đầu đã được lưu.");

    // Cập nhật gia tăng bộ tích lũy Accumulator với nước đi apply O(1)
    eval.apply(&pos, from, to, moving, captured);
    // Thực thi nước đi trên bàn cờ để cập nhật trạng thái position
    let state = pos.apply(from, to);
    // In thông báo đã thực thi apply gia tăng
    println!(" -> Đã áp dụng nước đi ({}) từ ô {} đến ô {}.", moving, from, to);

    // Tạo một bộ tích lũy mới để đối chiếu kết quả reset từ đầu
    let mut fresh = Accum::new();
    // Tính toán tích lũy lại từ đầu cho toàn bộ bàn cờ mới
    fresh.reset(&pos, &eval.nnue.weight);
    // In thông báo đã reset tính toán lại từ đầu
    println!(" -> Đã tính toán bộ tích lũy mới từ đầu để đối chiếu.");

    // Kiểm tra tính nhất quán 100% giữa apply gia tăng O(1) và reset từ đầu
    let valid = eval.accum == fresh;
    // In kết quả kiểm tra khớp nhau
    println!(" -> Cập nhật gia tăng O(1) khớp 100% với Reset: {}", valid);
    // Thẩm định khẳng định hai bộ tích lũy bắt buộc phải bằng nhau
    assert!(valid, "Cập nhật gia tăng apply bắt buộc phải khớp với reset!");

    // Hoàn tác nước đi trên bàn cờ vị trí position
    pos.revert(from, to, &state);
    // Hoàn tác gia tăng bộ tích lũy Accumulator với revert O(1)
    eval.revert(&pos, from, to, moving, captured);
    // In thông báo đã hoàn tác revert nước đi
    println!(" -> Đã hoàn tác nước đi revert O(1).");

    // Kiểm tra tính nhất quán 100% sau khi revert quay về trạng thái ban đầu
    let exact = eval.accum == initial;
    // In kết quả kiểm tra khôi phục thành công
    println!(" -> Khôi phục bộ tích lũy revert khớp 100% với Ban đầu: {}", exact);
    // Thẩm định khẳng định bộ tích lũy sau revert phải khớp với trạng thái ban đầu
    assert!(exact, "Hoàn tác gia tăng revert bắt buộc phải khớp với ban đầu!");

    // In dòng phân cách mục HCE & Circuit Breaker
    println!("\n------------------------------------------------------------");
    // In tiêu đề mục 6: HCE EVALUATION & CƠ CHẾ BẢO VỆ CIRCUIT BREAKER
    println!("[6] HCE EVALUATION & CƠ CHẾ BẢO VỆ CIRCUIT BREAKER");
    // In dòng phân cách
    println!("------------------------------------------------------------");

    // Khởi tạo trực tiếp đối tượng đánh giá tĩnh thủ công HCE
    let calc = Hce::new();
    // Đánh giá trực tiếp điểm số vị trí bằng HCE
    let points = calc.evaluate(&pos);
    // In điểm số đánh giá trực tiếp từ HCE
    println!(" -> Đánh giá trực tiếp Hce::evaluate: {} centipawns", points);

    // Kiểm tra trạng thái máy ngắt mạch Circuit Breaker
    let allow = eval.circuit.allow(0);
    // In trạng thái cho phép của Circuit Breaker
    println!(" -> Trạng thái ngắt mạch Circuit Breaker cho phép: {}", allow);

    // In dòng hoàn thành ví dụ 03 thành công
    println!("\n============================================================");
    // In thông báo chạy thành công toàn bộ ví dụ 03
    println!("  HOÀN THÀNH VÍ DỤ 03: EVALUATION & NNUE THÀNH CÔNG 100%!  ");
    // In dòng kẻ phân cách kết thúc
    println!("============================================================");
}
