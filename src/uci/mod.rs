// ============================================================================
// MODULE UCI: GIAO DIỆN GIAO THỨC CHUẨN UCI V2 (UNIVERSAL CHESS INTERFACE)
// ============================================================================
// Giao thức UCI v2 giúp Engine kết nối tương thích 100% với các GUI Cờ Tướng nổi tiếng
// (như CCBridge, SharkChess, GuiXiangqi, Penguin):
// - `command`: Phân tích 8 loại câu lệnh UCI (`uci`, `isready`, `position`, `go`, `stop`, `setoption`, `ucinewgame`, `quit`).
// - `engine`: Vòng lặp lắng nghe bất đồng bộ và xử lý lệnh không chặn luồng I/O.
// - `format`: Chuyển đổi mã hóa nước đi giữa `Move` (16-bit) và chuỗi ký tự tiêu chuẩn (như `h2e2`).
// ============================================================================

/// Module con `command` biểu diễn các câu lệnh UCI dưới dạng Enum Command
pub mod command;
/// Module con `engine` quản lý trạng thái Engine và chạy vòng lặp sự kiện UCI Loop
pub mod engine;
/// Module con `format` mã hóa và giải mã định dạng nước đi văn bản UCI
pub mod format;
/// Module con `option` quản lý danh sách cấu hình tùy chỉnh (`Hash`, `Threads`, `Clear Hash`, v.v.)
pub mod option;
/// Module con `parser` đọc và phân tích cú pháp chuỗi văn bản từ stdin
pub mod parser;

pub use command::Command;
pub use engine::Engine;
pub use format::Format;
pub use option::{Kind, Option};
pub use parser::Parser;

// ----------------------------------------------------------------------------
// KHU VỰC BÀI KIỂM THỬ ĐƠN VỊ (UNIT TESTS) CHO BỘ PHÂN TÍCH VÀ ENGINE UCI
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::movegen::types::Move;

    /// Kiểm thử mã hóa và giải mã định dạng chuỗi nước đi UCI (Coordinate notation conversion e.g. "h2e2").
    #[test]
    fn conversion() {
        let m = Move::new(25, 22);
        let text = Format::encode(m);
        assert_eq!(text, "h2e2");

        let decoded = Format::decode("h2e2");
        assert_eq!(decoded, m);
    }

    /// Kiểm thử thực thi chuỗi lệnh UCI chuẩn (`uci` -> `isready` -> `position` -> `go` -> `quit`).
    #[test]
    fn execution() {
        let mut engine = Engine::new();
        assert!(engine.exec(Command::Uci));
        assert!(engine.exec(Command::Ready));
        assert!(engine.exec(Command::Position {
            fen: String::new(),
            moves: vec!["h2e2".to_string()]
        }));
        assert!(engine.exec(Command::Go {
            depth: 4,
            nodes: 0,
            infinite: false,
            span: 0,
            red: 0,
            black: 0,
            gain: 0,
            extra: 0,
        }));
        assert!(!engine.exec(Command::Quit)); // Lệnh quit trả về false để thoát vòng lặp
    }
}

