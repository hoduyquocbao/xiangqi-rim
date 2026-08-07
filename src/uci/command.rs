// ============================================================================
// MODULE COMMAND: ĐỊNH NGHĨA CÁC CÂU LỆNH CHUẨN GIAO THỨC UCI V2 (UCI COMMANDS)
// ============================================================================
// `command.rs` chứa enum `Command` biểu diễn toàn bộ các câu lệnh mà Engine nhận từ GUI
// qua luồng chuẩn STDIN trong giao thức UCI v2 (Universal Chess Interface):
// - `Uci`: Yêu cầu khởi tạo phản hồi thông tin Engine.
// - `Ready`: Lệnh kiểm tra Engine sẵn sàng (trả về `readyok`).
// - `Option`: Lệnh thiết lập giá trị tùy chọn (ví dụ: `setoption name Hash value 64`).
// - `Reset`: Lệnh làm mới Engine (`ucinewgame`).
// - `Position`: Lệnh nạp thế cờ từ FEN hoặc các nước đi tiếp nối (`position fen ... moves ...`).
// - `Go`: Lệnh thực thi tìm kiếm với các giới hạn thời gian/độ sâu.
// - `Stop`: Lệnh dừng khẩn cấp phiên tìm kiếm.
// - `Quit`: Lệnh thoát chương trình.
// - `Invalid`: Lệnh không hợp lệ.
// ============================================================================

/// Enum `Command` biểu diễn các câu lệnh chuẩn UCI v2 được phân tích từ cú pháp đầu vào.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    /// Lệnh khởi tạo giao thức `uci` (yêu cầu trả về `id name` và `uciok`)
    Uci,
    /// Lệnh kiểm tra sẵn sàng `isready` (yêu cầu trả về `readyok`)
    Ready,
    /// Lệnh cài đặt tùy chọn `setoption name <name> value <value>`
    Option {
        /// Tên tùy chọn
        name: String,
        /// Giá trị tùy chọn
        value: String,
    },
    /// Lệnh đặt lại Engine cho ván đấu mới `ucinewgame`
    Reset,
    /// Lệnh thiết lập vị trí bàn cờ `position [fen <fen>] [moves <m1> <m2> ...]`
    Position {
        /// Chuỗi FEN vị trí xuất phát
        fen: String,
        /// Danh sách nước đi tiếp nối
        moves: Vec<String>,
    },
    /// Lệnh bắt đầu tính toán nước đi `go [depth N] [nodes N] [wtime N] [btime N] [winc N] [binc N] [movetime N] [infinite]`
    Go {
        /// Độ sâu tìm kiếm
        depth: u8,
        /// Số nút tối đa
        nodes: u64,
        /// Cờ tìm kiếm vô hạn
        infinite: bool,
        /// Thời gian tính toán cố định (movetime ms)
        span: u64,
        /// Thời gian còn lại của phe Đỏ (wtime ms)
        red: u64,
        /// Thời gian còn lại của phe Đen (btime ms)
        black: u64,
        /// Tăng thời gian mỗi nước cho Đỏ (winc ms)
        gain: u64,
        /// Tăng thời gian mỗi nước cho Đen (binc ms)
        extra: u64,
    },
    /// Lệnh dừng tính toán lập tức `stop`
    Stop,
    /// Lệnh thoát Engine `quit`
    Quit,
    /// Cú pháp câu lệnh không hợp lệ `Invalid`
    Invalid,
}

