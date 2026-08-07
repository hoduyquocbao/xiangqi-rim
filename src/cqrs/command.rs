// ============================================================================
// MODULE COMMAND: BIỂU DIỄN CÁC LỆNH ĐIỀU KHIỂN THAY ĐỔI TRẠNG THÁI ENGINE (CQRS COMMAND)
// ============================================================================
// Định nghĩa enum `Command` đại diện cho tất cả các tác vụ thay đổi trạng thái Engine
// trong mô hình CQRS: thiết lập bàn cờ (`Position`), thực thi tìm kiếm (`Go`), ngắt dừng (`Stop`),
// cài đặt tùy chọn (`Option`), đặt lại (`Reset`), và thoát chương trình (`Quit`).
// ============================================================================

/// Enum `Command` chứa danh sách các lệnh điều khiển làm biến đổi trạng thái nội tại Engine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    /// Lệnh thiết lập thế cờ `Position`: nạp chuỗi FEN và danh sách nước đi nối tiếp
    Position {
        /// Chuỗi FEN biểu diễn vị trí thế cờ
        fen: String,
        /// Danh sách các chuỗi nước đi tiếp theo (e.g. `["h2e2", "h9e7"]`)
        moves: Vec<String>,
    },
    /// Lệnh thực thi tìm kiếm nước đi `Go` với các tham số giới hạn thời gian và độ sâu
    Go {
        /// Giới hạn độ sâu tìm kiếm tối đa (depth: 0..255)
        depth: u8,
        /// Giới hạn tổng số nút cây cờ tối đa được duyệt
        nodes: u64,
        /// Cờ đánh dấu tìm kiếm vô hạn cho đến khi nhận lệnh Stop
        infinite: bool,
        /// Giới hạn khoảng thời gian tìm kiếm cố định tính bằng ms
        span: u64,
        /// Thời gian còn lại của bên Đỏ tính bằng ms
        red: u64,
        /// Thời gian còn lại của bên Đen tính bằng ms
        black: u64,
        /// Tăng thời gian sau mỗi nước đi của bên Đỏ tính bằng ms
        gain: u64,
        /// Tăng thời gian sau mỗi nước đi của bên Đen tính bằng ms
        extra: u64,
    },
    /// Lệnh ngắt dừng lập tức phiên tìm kiếm đang chạy `Stop`
    Stop,
    /// Lệnh cài đặt cấu hình tùy chọn `Option`: truyền tên tùy chọn và giá trị mới
    Option {
        /// Tên tùy chọn (e.g. `"Hash"`, `"Threads"`)
        name: String,
        /// Giá trị thiết lập dưới dạng chuỗi văn bản
        value: String,
    },
    /// Lệnh đặt lại toàn bộ Engine về vị trí bàn cờ mặc định `Reset`
    Reset,
    /// Lệnh thoát khỏi ứng dụng Engine `Quit`
    Quit,
}

