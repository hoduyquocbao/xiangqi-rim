// ============================================================================
// VÍ DỤ 04: BỘ TÌM KIẾM CÂY NƯỚC ĐỊ PVS VÀ ASPIRATION WINDOW TRONG XIANGRUST
// ============================================================================
// Tệp ví dụ độc lập minh họa cách khởi tạo PVS Search Engine (`Search::go`),
// giải thích và áp dụng cửa sổ Aspiration Window, cấu hình tham số giới hạn
// `Limits` (depth, nodes, time), và in ra kết quả tìm kiếm (Best move, Score, PV line).
// 100% chú thích tiếng Việt từng dòng & 100% định danh đơn từ tiếng Anh.
// ============================================================================

// Nhập module Parser từ thư viện xiangrust
use xiangrust::board::Parser;
// Nhập module Limits và Search từ thư viện xiangrust
use xiangrust::search::{Limits, Search};
// Nhập module Format từ thư viện xiangrust
use xiangrust::uci::format::Format;

// Hàm chính main thực thi ví dụ minh họa Search Engine
fn main() {
    // In tiêu đề trang trí khởi đầu chương trình minh họa
    println!("============================================================");
    // In tiêu đề giới thiệu ví dụ 04 về PVS Search Engine
    println!("  XIANGRUST AI ENGINE - VÍ DỤ 04: PVS SEARCH ENGINE & LIMITS ");
    // In dòng kẻ phân cách trang trí
    println!("============================================================");

    // Khai báo dung lượng bảng băm Transposition Table (16 Megabytes)
    let mb = 16usize;
    // Khởi tạo Search Engine với dung lượng bảng băm đã cấu hình
    let mut search = Search::new(mb);
    // In thông báo Search Engine đã được khởi tạo thành công
    println!("\n[1] Đã khởi tạo PVS Search Engine thành công với {} MB TT!", mb);

    // Khởi tạo bàn cờ mặc định vị trí xuất phát từ chuỗi FEN chuẩn
    let pos = Parser::parse(Parser::DEFAULT);
    // In thông báo bàn cờ xuất phát đã được khởi tạo thành công
    println!("[2] Đã khởi tạo bàn cờ xuất phát từ FEN thành công!");

    // In dòng phân cách mục cấu hình giới hạn độ sâu
    println!("\n------------------------------------------------------------");
    // In tiêu đề mục 3: Thử nghiệm tìm kiếm theo giới hạn độ sâu (Depth Limits)
    println!("[3] THỬ NGHIỆM TÌM KIẾM THEO ĐỘ SÂU (Depth Limit = 5)");
    // In dòng phân cách
    println!("------------------------------------------------------------");

    // Khởi tạo đối tượng Limits chứa các tham số giới hạn tìm kiếm
    let mut limit = Limits::new();
    // Khai báo thiết lập giới hạn độ sâu tìm kiếm bằng 5
    limit.depth = 5;
    // In thông báo đã thiết lập độ sâu tìm kiếm = 5
    println!(" -> Cấu hình Limits: depth = {}", limit.depth);

    // Thực thi thuật toán tìm kiếm PVS qua phương thức go
    let res = search.go(&pos, &limit);
    // Trích xuất nước đi tốt nhất best từ kết quả tìm kiếm
    let best = res.best;
    // Mã hóa nước đi tốt nhất sang chuỗi đại số chuẩn UCI 4 ký tự
    let code = Format::encode(best);
    // Trích xuất điểm số thế cờ score tính theo centipawns
    let score = res.score;
    // Trích xuất số lượng nút cây cờ đã duyệt nodes
    let nodes = res.nodes;
    // Trích xuất thời gian thực thi time tính bằng miligiây
    let time = res.time;

    // In kết quả nước đi tốt nhất dưới dạng mã UCI
    println!(" -> Nước đi tốt nhất (Best Move): {} (from: {}, to: {})", code, best.from, best.to);
    // In điểm số thế cờ thu được từ tìm kiếm
    println!(" -> Điểm số thế cờ (Score): {} centipawns", score);
    // In số lượng nút cây cờ đã duyệt
    println!(" -> Số nút cờ đã duyệt (Nodes): {}", nodes);
    // In thời gian tìm kiếm thực tế
    println!(" -> Thời gian tìm kiếm (Time): {} ms", time);

    // In dòng phân cách mục Aspiration Window & Time Limits
    println!("\n------------------------------------------------------------");
    // In tiêu đề mục 4: Thử nghiệm tìm kiếm theo giới hạn thời gian (Time Limit)
    println!("[4] THỬ NGHIỆM TÌM KIẾM THEO THỜI GIAN (Time Limit = 300 ms)");
    // In dòng phân cách
    println!("------------------------------------------------------------");

    // Đặt lại đối tượng Limits rỗng bằng 0
    limit = Limits::new();
    // Cấu hình thời gian tối đa cho phép tìm kiếm là 300 ms
    limit.time = 300;
    // In thông báo đã cấu hình giới hạn thời gian 300 ms
    println!(" -> Cấu hình Limits: time = {} ms", limit.time);

    // Thực thi thuật toán tìm kiếm PVS với giới hạn thời gian 300 ms
    let alt = search.go(&pos, &limit);
    // Trích xuất nước đi tốt nhất thu được
    let step = alt.best;
    // Mã hóa nước đi tốt nhất sang chuẩn UCI
    let text = Format::encode(step);

    // In kết quả nước đi tốt nhất theo thời gian
    println!(" -> Nước đi tốt nhất thu được: {}", text);
    // In tổng số nút đã duyệt trong 300 ms
    println!(" -> Tổng số nút đã duyệt: {} nodes", alt.nodes);
    // In thời gian thực tế đã chạy
    println!(" -> Thời gian thực tế: {} ms", alt.time);

    // In dòng phân cách mục Aspiration Window Concept Explanation
    println!("\n------------------------------------------------------------");
    // In tiêu đề mục 5: Minh họa cơ chế cửa sổ Aspiration Window
    println!("[5] MINH HỌA CƠ CHẾ CỬA SỔ ASPIRATION WINDOW (Delta = 20)");
    // In dòng phân cách
    println!("------------------------------------------------------------");

    // Khai báo độ rộng delta ban đầu của cửa sổ Aspiration Window (20 centipawns)
    let delta = 20i32;
    // Khai báo biên dưới alpha ban đầu bằng điểm số trừ delta
    let alpha = score - delta;
    // Khai báo biên trên beta ban đầu bằng điểm số cộng delta
    let beta = score + delta;
    // In thông báo thông số cửa sổ Aspiration Window cho độ sâu tiếp theo
    println!(" -> Cửa sổ Aspiration Window cho độ sâu tiếp theo: [{}, {}]", alpha, beta);
    // In giải thích cơ chế nới rộng cửa sổ khi Fail-High hoặc Fail-Low
    println!(" -> Khi điểm số vượt ranh giới, Alpha/Beta tự nới rộng: +/- 50 centipawns.");

    // In dòng phân cách mục trích xuất tuyến biến thể chính PV Line
    println!("\n------------------------------------------------------------");
    // In tiêu đề mục 6: Trích xuất tuyến biến thể chính PV Line từ Transposition Table
    println!("[6] TRÍCH XUẤT TUYẾN NƯỚC ĐỊ BIẾN THỂ CHÍNH (PV Line)");
    // In dòng phân cách
    println!("------------------------------------------------------------");

    // Tạo bản sao bàn cờ tạm thời temp để mô phỏng duyệt tuyến PV
    let mut temp = pos;
    // Khởi tạo biến đếm độ dài nước đi trong tuyến PV
    let mut depth = 0usize;
    // In thông báo bắt đầu trích xuất tuyến PV
    print!(" -> PV Line: ");

    // Vòng lặp tra cứu bảng băm TT để khôi phục chuỗi nước đi PV
    while depth < 8 {
        // Tra cứu phần tử băm tương ứng với hash của bàn cờ hiện tại
        if let Some(entry) = search.tt.probe(temp.hash) {
            // Lấy nước đi lưu trong bảng băm TT
            let step = entry.step;
            // Kiểm tra nước đi có hợp lệ hay không
            if !step.valid() {
                // Nếu nước đi không hợp lệ thì ngắt vòng lặp
                break;
            }
            // Mã hóa nước đi sang dạng chuỗi đại số UCI
            let str = Format::encode(step);
            // In nước đi thuộc tuyến PV ra màn hình
            print!("{} ", str);

            // Kiểm tra bộ đánh giá có cần cập nhật bộ tích lũy không
            let active = search.eval.enabled();
            // Lấy quân cờ di chuyển tại ô từ
            let moving = temp.grid[step.from as usize];
            // Lấy quân cờ bị ăn tại ô tới
            let captured = temp.grid[step.to as usize];

            // Nếu NNUE hoạt động, cập nhật bộ tích lũy Accumulator
            if active {
                search.eval.apply(&temp, step.from, step.to, moving, captured);
            }
            // Thực thi nước đi trên bàn cờ tạm thời
            temp.apply(step.from, step.to);
            // Tăng bộ đếm độ sâu tuyến PV
            depth += 1;
        } else {
            // Nếu không tìm thấy trong TT thì ngắt vòng lặp
            break;
        }
    }
    // In dòng mới xuống hàng sau khi in xong tuyến PV
    println!();

    // In dòng phân cách mục làm sạch bộ nhớ Clear
    println!("\n------------------------------------------------------------");
    // In tiêu đề mục 7: Làm sạch bộ nhớ Engine Search Clear
    println!("[7] LÀM SẠCH BỘ NHỚ ENGINE (Search::clear)");
    // In dòng phân cách
    println!("------------------------------------------------------------");

    // Thực hiện làm sạch toàn bộ dữ liệu bàn cờ, lịch sử, TT
    search.clear();
    // In thông báo đã làm sạch bộ nhớ thành công
    println!(" -> Đã làm sạch toàn bộ dữ liệu trong Search Engine!");

    // In dòng hoàn thành ví dụ 04 thành công
    println!("\n============================================================");
    // In thông báo chạy thành công toàn bộ ví dụ 04
    println!("  HOÀN THÀNH VÍ DỤ 04: PVS SEARCH ENGINE THÀNH CÔNG 100%!  ");
    // In dòng kẻ phân cách kết thúc
    println!("============================================================");
}
