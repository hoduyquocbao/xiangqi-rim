// ============================================================================
// MODULE FEN: BỘ PHÂN TÍCH (PARSER), THẨM ĐỊNH (VALIDATOR) VÀ XUẤT CHUỖI FEN
// ============================================================================
// Định dạng FEN (Forsyth-Edwards Notation) chuẩn Cờ Tướng gồm 6 trường phân tách:
// 1. Vị trí 90 ô cờ trên 10 hàng (Rank 9 down to Rank 0, phân tách bằng `/`)
// 2. Phe đến lượt đi (`w`/`r`: Đỏ, `b`: Đen)
// 3. Khả năng nhập thành (luôn là `-` trong Cờ Tướng)
// 4. Khả năng bắt Tốt qua đường (luôn là `-` trong Cờ Tướng)
// 5. Số nước đi chưa ăn quân / chưa đi Tốt (Halfmove clock / rule50)
// 6. Số ply (nửa nước đi) tính từ đầu trận (Fullmove number / ply)
// ============================================================================

use super::piece::Piece;
use super::position::Position;
use crate::movegen::fly;

/// Enum `Fault` đại diện cho các mã lỗi thẩm định FEN UCCI 10x9.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Fault {
    Format,  // Cú pháp trường tổng thể không hợp lệ
    Ranks,   // Số hàng khác 10 (số dấu '/' khác 9)
    Files,   // Tổng số ô trên 1 hàng khác 9
    Digits,  // Chữ số liền kề (111111111, 18, 45...) hoặc số 0
    Syntax,  // Ký tự không hợp lệ trong chuỗi bàn cờ
    Palace,  // Tướng hoặc Sĩ nằm ngoài Cung
    Kings,   // Số lượng Tướng khác 1 cho mỗi bên
    River,   // Tượng qua sông hoặc Tốt nằm sai vị trí
    Fly,     // Hai Tướng lộ mặt nhìn thẳng nhau
    Turn,    // Phe đến lượt đi không hợp lệ
}

/// Struct `Validator` thực hiện thẩm định tính toàn vẹn FEN UCCI 10x9.
pub struct Validator;

impl Validator {
    /// Thẩm định nhanh tính hợp lệ của chuỗi FEN `text` (trả về `true` nếu hợp lệ).
    #[inline(always)]
    pub fn check(text: &str) -> bool {
        Self::audit(text).is_ok()
    }

    /// Thẩm định chi tiết và trả về `Ok(())` hoặc mã lỗi `Fault`.
    pub fn audit(text: &str) -> Result<(), Fault> {
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.is_empty() {
            return Err(Fault::Format);
        }

        // 1. Kiểm tra cấu trúc 10 hàng (đúng 9 dấu '/')
        let ranks: Vec<&str> = parts[0].split('/').collect();
        if ranks.len() != 10 {
            return Err(Fault::Ranks);
        }

        let mut kings = [0u8; 2];
        let mut pos = Position::empty();

        // 2. Thẩm định từng hàng từ Rank 9 xuống Rank 0
        let mut r = 0usize;
        while r < 10 {
            let rank = (9 - r) as u8;
            let mut file = 0u8;
            let mut digit = false;

            for ch in ranks[r].chars() {
                if let Some(val) = ch.to_digit(10) {
                    if val == 0 {
                        return Err(Fault::Digits);
                    }
                    if digit {
                        // Triệt tiêu chữ số đứng liền kề (111111111, 18, 45...)
                        return Err(Fault::Digits);
                    }
                    file += val as u8;
                    digit = true;
                } else {
                    digit = false;
                    let piece = Piece::parse(ch);
                    if !piece.valid() {
                        return Err(Fault::Syntax);
                    }
                    if file >= 9 {
                        return Err(Fault::Files);
                    }

                    let square = rank * 9 + file;
                    let color = if piece.0 < 7 { 0 } else { 1 };
                    let role = piece.0 % 7;

                    // Thẩm định Cung Tướng (Palace: ranks 0..2/7..9, files 3..5)
                    if role == 0 {
                        // Tướng
                        kings[color] += 1;
                        if color == 0 {
                            if rank > 2 || file < 3 || file > 5 {
                                return Err(Fault::Palace);
                            }
                        } else {
                            if rank < 7 || file < 3 || file > 5 {
                                return Err(Fault::Palace);
                            }
                        }
                    } else if role == 1 {
                        // Sĩ
                        if color == 0 {
                            if rank > 2 || file < 3 || file > 5 {
                                return Err(Fault::Palace);
                            }
                        } else {
                            if rank < 7 || file < 3 || file > 5 {
                                return Err(Fault::Palace);
                            }
                        }
                    } else if role == 2 {
                        // Tượng (không qua sông: Red 0..4, Black 5..9)
                        if color == 0 {
                            if rank > 4 {
                                return Err(Fault::River);
                            }
                        } else {
                            if rank < 5 {
                                return Err(Fault::River);
                            }
                        }
                    } else if role == 6 {
                        // Tốt (không lùi sau hàng xuất phát)
                        if color == 0 {
                            if rank < 3 {
                                return Err(Fault::River);
                            }
                        } else {
                            if rank > 6 {
                                return Err(Fault::River);
                            }
                        }
                    }

                    pos.put(piece.0, square);
                    file += 1;
                }
            }

            if file != 9 {
                return Err(Fault::Files);
            }
            r += 1;
        }

        // 3. Thẩm định số lượng Tướng (mỗi bên đúng 1 Tướng)
        if kings[0] != 1 || kings[1] != 1 {
            return Err(Fault::Kings);
        }

        // 4. Thẩm định Lộ mặt Tướng (Flying General)
        if fly(&pos) {
            return Err(Fault::Fly);
        }

        // 5. Thẩm định lượt đi (w/r hoặc b)
        if parts.len() > 1 {
            let turn = parts[1];
            if turn != "w" && turn != "W" && turn != "r" && turn != "R" && turn != "b" && turn != "B" {
                return Err(Fault::Turn);
            }
        }

        Ok(())
    }
}

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

        // Thẩm định nhanh: Nếu FEN không hợp lệ, trả về bàn cờ rỗng an toàn
        if !Validator::check(text) {
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
                    file += digit as u8;
                } else {
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
                    count += 1;
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

    /// Chuyển đổi đối tượng `Position` trực tiếp vào mảng byte đệm `buf` không cấp phát bộ nhớ Heap (Zero Heap Allocation).
    #[inline(always)]
    pub fn export_bytes(pos: &Position, buf: &mut [u8; 96]) -> usize {
        let mut idx = 0usize;

        let mut r = 9i32;
        while r >= 0 {
            let rank = r as u8;
            let mut count = 0u8;
            let mut f = 0u8;
            while f < 9 {
                let square = rank * 9 + f;
                let piece = Piece::make(pos.grid[square as usize]);
                if piece.empty() {
                    count += 1;
                } else {
                    if count > 0 {
                        buf[idx] = b'0' + count;
                        idx += 1;
                        count = 0;
                    }
                    buf[idx] = piece.char() as u8;
                    idx += 1;
                }
                f += 1;
            }
            if count > 0 {
                buf[idx] = b'0' + count;
                idx += 1;
            }
            if r > 0 {
                buf[idx] = b'/';
                idx += 1;
            }
            r -= 1;
        }

        buf[idx] = b' ';
        idx += 1;
        buf[idx] = if pos.side == 0 { b'w' } else { b'b' };
        idx += 1;

        buf[idx..idx + 5].copy_from_slice(b" - - ");
        idx += 5;

        // rule50
        if pos.rule >= 100 {
            buf[idx] = b'0' + (pos.rule / 100) as u8;
            idx += 1;
            buf[idx] = b'0' + ((pos.rule / 10) % 10) as u8;
            idx += 1;
            buf[idx] = b'0' + (pos.rule % 10) as u8;
            idx += 1;
        } else if pos.rule >= 10 {
            buf[idx] = b'0' + (pos.rule / 10) as u8;
            idx += 1;
            buf[idx] = b'0' + (pos.rule % 10) as u8;
            idx += 1;
        } else {
            buf[idx] = b'0' + pos.rule as u8;
            idx += 1;
        }

        buf[idx] = b' ';
        idx += 1;

        // ply
        if pos.ply >= 100 {
            buf[idx] = b'0' + (pos.ply / 100) as u8;
            idx += 1;
            buf[idx] = b'0' + ((pos.ply / 10) % 10) as u8;
            idx += 1;
            buf[idx] = b'0' + (pos.ply % 10) as u8;
            idx += 1;
        } else if pos.ply >= 10 {
            buf[idx] = b'0' + (pos.ply / 10) as u8;
            idx += 1;
            buf[idx] = b'0' + (pos.ply % 10) as u8;
            idx += 1;
        } else {
            buf[idx] = b'0' + pos.ply as u8;
            idx += 1;
        }

        idx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_default_fen() {
        assert!(Validator::check(Parser::DEFAULT));
    }

    #[test]
    fn reject_collapsed_ranks() {
        let bad = "111111111/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1";
        assert_eq!(Validator::audit(bad), Err(Fault::Digits));
        assert!(!Validator::check(bad));
    }

    #[test]
    fn reject_adjacent_digits() {
        let bad1 = "rnbakabnr/18/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1";
        assert_eq!(Validator::audit(bad1), Err(Fault::Digits));

        let bad2 = "rnbakabnr/9/1c5c1/p1p1p1p1p/45/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1";
        assert_eq!(Validator::audit(bad2), Err(Fault::Digits));
    }

    #[test]
    fn reject_invalid_rank_counts() {
        let bad = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/RNBAKABNR w - - 0 1";
        assert_eq!(Validator::audit(bad), Err(Fault::Ranks));
    }

    #[test]
    fn reject_invalid_file_counts() {
        let bad = "rnbakabnr/8/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1";
        assert_eq!(Validator::audit(bad), Err(Fault::Files));
    }

    #[test]
    fn reject_king_outside_palace() {
        let bad = "rnbakabnr/9/1c5c1/p1p1p1p1p/4K4/9/P1P1P1P1P/1C5C1/9/RNBA1ABNR w - - 0 1";
        assert_eq!(Validator::audit(bad), Err(Fault::Palace));
    }

    #[test]
    fn reject_advisor_outside_palace() {
        let bad = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/ANBAKABNR w - - 0 1";
        assert_eq!(Validator::audit(bad), Err(Fault::Palace));
    }

    #[test]
    fn reject_flying_general() {
        let bad = "4k4/9/9/9/9/9/9/9/9/4K4 w - - 0 1";
        assert_eq!(Validator::audit(bad), Err(Fault::Fly));
    }
}


