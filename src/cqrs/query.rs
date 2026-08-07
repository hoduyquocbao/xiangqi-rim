// ============================================================================
// MODULE QUERY: BIỂU DIỄN CÁC TRUY VẤN ĐỌC DỮ LIỆU ENGINE (CQRS QUERY)
// ============================================================================
// Định nghĩa enum `Query` biểu diễn các yêu cầu chỉ đọc (Read-only query) không làm thay đổi
// trạng thái Engine: lấy thông tin vị trí (`Position`), thống kê (`Stats`), điểm đánh giá (`Eval`),
// tra cứu tùy chọn (`Option`), và lấy trạng thái máy ngắt mạch (`State`).
// ============================================================================

/// Enum `Query` chứa các lệnh truy vấn thông tin an toàn không gây tác dụng phụ.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Query {
    /// Truy vấn chuỗi FEN biểu diễn vị trí thế cờ hiện tại
    Position,
    /// Truy vấn chỉ số thống kê hiệu năng (NPS, Nodes, Time)
    Stats,
    /// Truy vấn điểm số đánh giá static evaluation của vị trí hiện tại
    Eval,
    /// Truy vấn giá trị của một tùy chọn theo tên `name`
    Option {
        /// Tên của tùy chọn cần đọc giá trị
        name: String,
    },
    /// Truy vấn trạng thái ngắt mạch hiện tại của NNUE Circuit Breaker
    State,
}

