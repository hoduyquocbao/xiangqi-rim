// ============================================================================
// MODULE BOOK: THƯ VIỆN KHAI CUỘC VÀ CƠ SỞ TRI THỨC TÀN CUỘC THỰC DỤNG
// ============================================================================
// Module `book` quản lý hai phân hệ tri thức cơ bản của Engine Cờ Tướng:
// 1. `opening`: Thư viện nước đi khai cuộc (Opening Book) với tra cứu băm Zobrist O(log N) ~ 0ms.
// 2. `endgame`: Cơ sở tri thức tàn cuộc thực dụng (Endgame Knowledge Base) nhận diện các thế cờ lý thuyết.
//
// 100% Clean Room Design std-only và tuân thủ Quy tắc Định danh Đơn Từ (Single-word principle).
// ============================================================================

/// Phân hệ thư viện nước đi khai cuộc
pub mod opening;

/// Phân hệ cơ sở tri thức tàn cuộc thực dụng
pub mod endgame;

pub use endgame::{Endgame, Rule};
pub use opening::{Book, Entry};
