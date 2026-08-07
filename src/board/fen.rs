// ============================================================================
// MODULE FEN: BỘ PHÂN TÍCH (PARSER) VÀ XUẤT CHUỖI FEN (SERIALIZER) CỜ TƯỚNG
// ============================================================================
// Định dạng FEN (Forsyth-Edwards Notation) chuẩn Cờ Tướng gồm 6 trường phân tách:
// 1. Vị trí 90 ô cờ trên 10 hàng (Rank 9 down to Rank 0, phân tách bằng `/`)
// 2. Phe đến lượt đi (`w`: Đỏ / `b`: Đen)
// 3. Khả năng nhập thành (luôn là `-` trong Cờ Tướng)
// 4. Khả năng bắt Tốt qua đường (luôn là `-` trong Cờ Tướng)
// 5. Số nước đi chưa ăn quân / chưa đi Tốt (Halfmove clock / rule50)
// 6. Số ply (nửa nước đi) tính từ đầu trận (Fullmove number / ply)
// ============================================================================

use super::piece::Piece;
use super::position::Position;

/// Struct `Parser` cung cấp các hàm đọc và dựng đối tượng `Position` từ chuỗi FEN.
pub struct Parser;

impl Parser {
    /// Chuỗi FEN mặc định mô tả vị trí xuất phát ban đầu của bàn cờ Cờ Tướng.
    pub const DEFAULT: &'static str =
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1";

    /// Phân tích chuỗi FEN `text` và trả về cấu trúc `Position` hoàn chỉnh.
    pub fn parse(text: &str) -> Position {
        let mut pos = Position::empty();
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.is_empty() {
            return pos;
        }

        // 1. Phân tích các hàng bàn cờ (Rank 9 down to Rank 0)
        let ranks: Vec<&str> = parts[0].split('/').collect();
        let mut r = 0usize;
        while r < ranks.len() && r < 10 {
            let rank = (9 - r) as u8;
            let mut file = 0u8;
            for char in ranks[r].chars() {
                if let Some(digit) = char.to_digit(10) {
                    // Ký tự số đại diện cho số ô trống liên tiếp
                    file += digit as u8;
                } else {
                    // Ký tự chữ đại diện cho quân cờ
                    let piece = Piece::parse(char);
                    if piece.valid() && file < 9 {
                        let square = rank * 9 + file;
                        pos.put(piece.0, square);
                    }
                    file += 1;
                }
            }
            r += 1;
        }

        // 2. Phân tích phe đến lượt đi ('w'/'r': Đỏ, 'b': Đen)
        if parts.len() > 1 {
            let turn = parts[1];
            if turn == "b" || turn == "B" {
                pos.side = 1;
            } else {
                pos.side = 0;
            }
        }

        // 3. Phân tích bộ đếm 50 nước hòa (rule50)
        if parts.len() > 4 {
            if let Ok(rule) = parts[4].parse::<u16>() {
                pos.rule = rule;
            }
        }

        // 4. Phân tích số nửa nước đi (ply counter)
        if parts.len() > 5 {
            if let Ok(ply) = parts[5].parse::<u16>() {
                pos.ply = ply;
            }
        }

        // 5. Tính toán khóa băm Zobrist Hash ban đầu cho toàn bộ vị trí vừa dựng
        pos.hash = pos.compute();
        pos
    }
}

/// Struct `Serializer` đóng gói đối tượng `Position` thành chuỗi FEN chuẩn.
pub struct Serializer;

impl Serializer {
    /// Chuyển đổi đối tượng `Position` thành chuỗi FEN định dạng String.
    pub fn export(pos: &Position) -> String {
        let mut out = String::with_capacity(128);

        // 1. Xuất dữ liệu 10 hàng bàn cờ từ Rank 9 xuống Rank 0
        let mut r = 9i32;
        while r >= 0 {
            let rank = r as u8;
            let mut count = 0u32;
            let mut f = 0u8;
            while f < 9 {
                let square = rank * 9 + f;
                let piece = Piece::make(pos.grid[square as usize]);
                if piece.empty() {
                    count += 1; // Tích lũy số ô trống liên tiếp
                } else {
                    if count > 0 {
                        out.push_str(&count.to_string());
                        count = 0;
                    }
                    out.push(piece.char());
                }
                f += 1;
            }
            if count > 0 {
                out.push_str(&count.to_string());
            }
            if r > 0 {
                out.push('/');
            }
            r -= 1;
        }

        // 2. Xuất ký tự lượt đi ('w': Đỏ, 'b': Đen)
        out.push(' ');
        if pos.side == 0 {
            out.push('w');
        } else {
            out.push('b');
        }

        // 3. Xuất 2 trường cố định (- -)
        out.push_str(" - - ");

        // 4. Xuất giá trị rule50 và ply counter
        out.push_str(&pos.rule.to_string());
        out.push(' ');
        out.push_str(&pos.ply.to_string());

        out
    }
}

