// ============================================================================
// MODULE FORMAT: CHUYỂN ĐỔI ĐỊNH DẠNG NƯỚC ĐI VÀ CHUẨN ĐẠI SỐ UCI (UCI MOVE FORMATTER)
// ============================================================================
// `format.rs` cung cấp các phương thức chuyển đổi mã hóa/giải mã 2 chiều:
// - `encode(Move)`: Chuyển đổi nước đi 16-bit `Move` sang chuỗi đại số chuẩn UCI (ví dụ: `Move(from=25, to=22)` -> `"h2e2"`).
// - `decode(&str)`: Giải mã chuỗi đại số chuẩn UCI 4 ký tự sang nước đi 16-bit `Move`.
// - Tích hợp các unit test kiểm thử áp lực 8010 cặp tọa độ và thẩm định tốc độ thực thi nanosecond.
// ============================================================================

use crate::movegen::types::Move;

/// Struct `Format` chứa các hàm tĩnh chuyển đổi định dạng nước đi UCI.
pub struct Format;

impl Format {
    /// Chuyển đổi nước đi 16-bit `Move` sang chuỗi đại số UCI 4 ký tự (ví dụ: `"h2e2"`).
    pub fn encode(item: Move) -> String {
        if !item.valid() {
            return "0000".to_string();
        }
        let sf = (item.from % 9) as u8 + b'a';
        let sr = (item.from / 9) as u8 + b'0';
        let df = (item.to % 9) as u8 + b'a';
        let dr = (item.to / 9) as u8 + b'0';

        format!(
            "{}{}{}{}",
            sf as char, sr as char, df as char, dr as char
        )
    }

    /// Chuyển đổi chuỗi đại số UCI (ví dụ: `"h2e2"`) sang nước đi 16-bit `Move`.
    pub fn decode(text: &str) -> Move {
        let bytes = text.as_bytes();
        if bytes.len() != 4 {
            return Move::none();
        }

        // Thẩm định ranh giới cột 'a'..'i' và hàng '0'..'9'
        if bytes[0] < b'a' || bytes[0] > b'i' {
            return Move::none();
        }
        if bytes[1] < b'0' || bytes[1] > b'9' {
            return Move::none();
        }
        if bytes[2] < b'a' || bytes[2] > b'i' {
            return Move::none();
        }
        if bytes[3] < b'0' || bytes[3] > b'9' {
            return Move::none();
        }

        let sf = bytes[0] - b'a';
        let sr = bytes[1] - b'0';
        let df = bytes[2] - b'a';
        let dr = bytes[3] - b'0';

        let from = sr * 9 + sf;
        let to = dr * 9 + df;

        Move::new(from, to)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Kiểm thử mã hóa và giải mã 2 chiều cho nước đi cụ thể "h2e2"
    #[test]
    fn conversion() {
        let m = Move::new(25, 22);
        let text = Format::encode(m);
        assert_eq!(text, "h2e2");

        let decoded = Format::decode("h2e2");
        assert_eq!(decoded, m);
    }

    /// Kiểm thử áp lực chuyển đổi cho toàn bộ 8010 cặp ô hợp lệ trên bàn cờ 90 ô
    #[test]
    fn stress() {
        let mut count = 0;
        for from in 0..90 {
            for to in 0..90 {
                if from == to {
                    continue;
                }
                let m = Move::new(from, to);
                let text = Format::encode(m);
                assert_eq!(text.len(), 4);
                let decoded = Format::decode(&text);
                assert_eq!(decoded.from, from);
                assert_eq!(decoded.to, to);
                count += 1;
            }
        }
        assert_eq!(count, 8010);
    }

    /// Kiểm thử thẩm định chuỗi rác không hợp lệ
    #[test]
    fn invalid() {
        assert_eq!(Format::decode("a0"), Move::none());
        assert_eq!(Format::decode("z9z9"), Move::none());
        assert_eq!(Format::decode("a-1a0"), Move::none());
        assert_eq!(Format::decode("h2e2extra"), Move::none());
    }

    /// Kiểm thử đo lường tốc độ mã hóa/giải mã nanosecond
    #[test]
    fn speed() {
        let start = std::time::Instant::now();
        let iterations = 100;
        for _ in 0..iterations {
            for from in 0..90 {
                for to in 0..90 {
                    if from == to {
                        continue;
                    }
                    let m = Move::new(from, to);
                    let text = Format::encode(m);
                    let decoded = Format::decode(&text);
                    assert_eq!(decoded.from, from);
                }
            }
        }
        let elapsed = start.elapsed();
        let total = iterations * 8010;
        let rate = elapsed.as_nanos() / total as u128;
        println!("Format encode+decode 8,010x100 ops: Total = {:?}, Per Op = {}ns", elapsed, rate);
    }
}


