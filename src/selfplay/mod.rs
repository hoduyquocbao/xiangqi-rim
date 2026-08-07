// ============================================================================
// MODULE SELFPLAY: HỆ THỐNG TỰ ĐẤU VÀ ĐÁNH GIÁ CHỈ SỐ THỰC NGHIỆM CỜ TƯỚNG
// ============================================================================
// Module `selfplay` chịu trách nhiệm điều hành và quản lý các ván tự đấu (Self-Play):
// - `engine`: Điều hành tiến trình tự đấu giữa 2 phiên bản Engine Cờ Tướng.
// - `stats`: Thu thập và thống kê các chỉ số hiệu năng (Nodes, NPS, Time/move).
// - `pgn`: Xuất dữ liệu ván đấu ra định dạng PGN Cờ Tướng và FEN thế cờ.
// ============================================================================

/// Module con `engine` điều phối tự đấu
pub mod engine;
/// Module con `pgn` định dạng PGN và FEN
pub mod pgn;
/// Module con `stats` theo dõi chỉ số thống kê
pub mod stats;

// Xuất bản công khai (re-export) các cấu trúc dữ liệu cốt lõi
pub use engine::{Config, Match, Outcome, Runner, Side};
pub use pgn::{Fen, Pgn};
pub use stats::Stats;
