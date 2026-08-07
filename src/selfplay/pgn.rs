// ============================================================================
// MODULE PGN: ĐỊNH DẠNG XUẤT PGN VÀ FEN CỜ TƯỚNG (PGN & FEN EXPORTER)
// ============================================================================
// Module `pgn` cung cấp khả năng xuất dữ liệu ván đấu tự đấu:
// - `Pgn`: Chuyển đổi đối tượng `Match` thành chuỗi văn bản PGN chuẩn Cờ Tướng.
// - `Fen`: Chuyển đổi đối tượng `Position` thành chuỗi vị trí FEN chuẩn.
// - Căn lề bộ nhớ 64-byte (`#[repr(C, align(64))]`) đảm bảo tiêu chuẩn hệ thống.
// ============================================================================

use crate::board::{Position, Serializer};
use crate::selfplay::engine::{Match, Outcome, Side};
use crate::uci::format::Format as UciFormat;

/// Struct `Pgn` bọc hàm xuất chuỗi định dạng ván đấu PGN Cờ Tướng.
#[repr(C, align(64))]
pub struct Pgn {
    /// Trường đệm căn lề 64-byte
    _pad: [u8; 64],
}

impl Pgn {
    /// Khởi tạo đối tượng Pgn.
    pub const fn new() -> Self {
        Self { _pad: [0; 64] }
    }

    /// Xuất thông tin ván đấu `item` ra chuỗi văn bản định dạng PGN chuẩn.
    pub fn export(item: &Match) -> String {
        let result = match item.outcome {
            Outcome::Win(Side::Red) => "1-0",
            Outcome::Win(Side::Black) => "0-1",
            _ => "1/2-1/2",
        };

        let mut text = String::with_capacity(512);
        text.push_str("[Event \"Self-Play Match\"]\n");
        text.push_str("[Site \"Local Engine\"]\n");
        text.push_str("[Date \"2026.08.06\"]\n");
        text.push_str("[Round \"1\"]\n");
        text.push_str("[Red \"Xiangqi AI\"]\n");
        text.push_str("[Black \"Xiangqi AI\"]\n");
        text.push_str(&format!("[Result \"{}\"]\n", result));
        text.push_str(&format!("[TimeControl \"{}/0\"]\n\n", item.stats.time));

        let mut step = 0usize;
        while step < item.moves.len() {
            let turn = step / 2 + 1;
            if step % 2 == 0 {
                text.push_str(&format!("{}. ", turn));
            }
            let code = UciFormat::encode(item.moves[step]);
            text.push_str(&code);
            if step % 2 == 1 || step + 1 == item.moves.len() {
                text.push_str("\n");
            } else {
                text.push_str(" ");
            }
            step += 1;
        }

        text
    }
}

/// Struct `Fen` bọc hàm xuất chuỗi định dạng thế cờ FEN.
#[repr(C, align(64))]
pub struct Fen {
    /// Trường đệm căn lề 64-byte
    _pad: [u8; 64],
}

impl Fen {
    /// Khởi tạo đối tượng Fen.
    pub const fn new() -> Self {
        Self { _pad: [0; 64] }
    }

    /// Xuất vị trí bàn cờ `pos` ra chuỗi văn bản định dạng FEN.
    pub fn export(pos: &Position) -> String {
        Serializer::export(pos)
    }
}

// ----------------------------------------------------------------------------
// KHU VỰC BÀI KIỂM THỬ ĐƠN VỊ (UNIT TESTS) CHO PGN & FEN EXPORTER
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Parser;
    use std::mem::{align_of, size_of};

    /// Kiểm thử căn lề bộ nhớ 64-byte và kích thước của struct Pgn và Fen.
    #[test]
    fn alignments() {
        assert_eq!(size_of::<Pgn>(), 64);
        assert_eq!(align_of::<Pgn>(), 64);
        assert_eq!(size_of::<Fen>(), 64);
        assert_eq!(align_of::<Fen>(), 64);
    }

    /// Kiểm thử xuất chuỗi FEN từ vị trí mặc định ban đầu.
    #[test]
    fn fen() {
        let pos = Parser::parse(Parser::DEFAULT);
        let text = Fen::export(&pos);
        assert_eq!(text, Parser::DEFAULT);
    }

    /// Kiểm thử xuất chuỗi PGN từ một ván đấu mẫu.
    #[test]
    fn pgn() {
        let mut item = Match::new(10);
        item.moves.push(crate::movegen::Move::new(19, 28)); // Pháo c2-c5 (19 -> 28: b2b3)
        item.moves.push(crate::movegen::Move::new(64, 55)); // Pháo c7-c4 (64 -> 55: b7b6)
        item.outcome = Outcome::Win(Side::Red);
        item.stats.time = 150;
        item.stats.moves = 2;

        let text = Pgn::export(&item);
        assert!(text.contains("[Event \"Self-Play Match\"]"));
        assert!(text.contains("[Result \"1-0\"]"));
        assert!(text.contains("1. b2b3 b7b6"));
    }
}
