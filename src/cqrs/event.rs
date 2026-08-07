// ============================================================================
// MODULE EVENT: PHÁT SỰ KIỆN NÂNG CAO TRONG EVENT SOURCING (CQRS EVENT)
// ============================================================================
// Định nghĩa enum `Event` biểu diễn các sự kiện xảy ra trong Engine được phát tới Event Store:
// - `Move`: Sự kiện nước đi tốt nhất (`best`) và nước đi ponder (`ponder`).
// - `Score`: Sự kiện cập nhật điểm đánh giá Centipawn (`cp`) hoặc điểm Mate (`mate`).
// - `State`: Sự kiện thay đổi trạng thái tìm kiếm (`running`).
// - `Info`: Sự kiện tiến trình thông tin UCI (`depth`, `score`, `nodes`, `nps`, `time`, `pv`).
// - `Depth`: Sự kiện chuyển độ sâu tìm kiếm mới.
// - `Ready`: Sự kiện Engine đã sẵn sàng.
// ============================================================================

/// Enum `Event` đại diện cho các sự kiện không thể đảo ngược phát ra từ Engine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// Sự kiện tìm thấy nước đi tốt nhất `Move`
    Move {
        /// Mã hóa 16-bit của nước đi tốt nhất (Best Move)
        best: u16,
        /// Mã hóa 16-bit của nước đi tiên đoán (Ponder Move)
        ponder: u16,
    },
    /// Sự kiện cập nhật điểm số thế cờ `Score`
    Score {
        /// Điểm số tính theo Centipawn
        cp: i32,
        /// Số nước còn lại để chiếu bí (Mate in N moves)
        mate: i32,
    },
    /// Sự kiện thay đổi trạng thái thực thi `State`
    State {
        /// Trạng thái Engine đang chạy (true) hay tạm dừng (false)
        running: bool,
    },
    /// Sự kiện tiến trình tìm kiếm chi tiết `Info` phục vụ phản hồi UCI
    Info {
        /// Độ sâu tìm kiếm đạt được
        depth: u8,
        /// Điểm số thế cờ
        score: i32,
        /// Tổng số nút cây cờ đã duyệt
        nodes: u64,
        /// Tốc độ duyêt nút (Nodes Per Second)
        nps: u64,
        /// Khoảng thời gian đã tiêu tốn tính bằng ms
        time: u64,
        /// Chuỗi biến thể chính (Principal Variation path string)
        pv: String,
    },
    /// Sự kiện đạt đến mốc độ sâu tìm kiếm mới `Depth`
    Depth {
        /// Giá trị độ sâu mới
        val: u8,
    },
    /// Sự kiện Engine đã khởi tạo xong và sẵn sàng nhận lệnh `Ready`
    Ready,
}

