// ============================================================================
// MODULE BOARD: HỆ THỐNG CẤU TRÚC BÀN CỜ VÀ ĐỢI BẢO TRÌ BÀN CỜ CỜ TƯỚNG
// ============================================================================
// Module `board` xuất bản các kiểu dữ liệu nền tảng cho toàn bộ Engine:
// - `Square`: Tọa độ ô bàn cờ (0..89) bọc kiểu `u8`.
// - `Piece`: Mã hóa loại quân (0..13, 14 là ô trống) bọc kiểu `u8`.
// - `Bitboard`: Mặt nạ tập hợp ô bàn cờ bọc kiểu `u128` (align 16).
// - `Position`: Trạng thái toàn cục bàn cờ Cờ Tướng 448 bytes (align 64).
// - `Zobrist`: Bảng giá trị băm Zobrist hash 64-bit ngẫu nhiên hằng số.
// - `State`: Đối tượng lưu vết trạng thái 16 bytes (align 16) cho MakeMove/UndoMove.
// - `Parser` / `Serializer`: Bộ phân tích và xuất định dạng FEN Cờ Tướng.
// ============================================================================

/// Module con `bitboard` quản lý các thao tác bitwise 128-bit
pub mod bitboard;
/// Module con `magic` quản lý Magic Bitboards PEXT Lookup O(1)
pub mod magic;
/// Module con `fen` quản lý đọc/ghi định dạng FEN Cờ Tướng
pub mod fen;
/// Module con `piece` quản lý mã hóa quân cờ, phe chơi và loại quân
pub mod piece;
/// Module con `position` quản lý trạng thái tổng thể bàn cờ Cờ Tướng (align 64)
pub mod position;
/// Module con `square` quản lý tọa độ 90 ô bàn cờ
pub mod square;
/// Module con `state` quản lý lưu vết MakeMove/UndoMove
pub mod state;
/// Module con `zobrist` quản lý mảng băm ngẫu nhiên Zobrist Hash
pub mod zobrist;

// Xuất bản công khai (re-export) các cấu trúc dữ liệu cốt lõi để các module khác dễ dàng truy cập
pub use bitboard::Bitboard;
pub use fen::{Parser, Serializer};
pub use piece::{Color, Piece, Role};
pub use position::Position;
pub use square::Square;
pub use state::State;
pub use zobrist::{KEYS, Zobrist};

// ----------------------------------------------------------------------------
// KHU VỰC BÀI KIỂM THỬ ĐƠN VỊ (UNIT TESTS) & KIỂM THỬ ĐỐI KHÁNG (ADVERSARIAL TESTS)
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, size_of};

    /// Kiểm thử kích thước vật lý (size_of) và căn lề bộ nhớ (align_of) của các cấu trúc dữ liệu cốt lõi.
    /// Đảm bảo tính tương thích phần cứng và triệt tiêu False Sharing.
    #[test]
    fn alignments() {
        // Kiểm tra Bitboard chiếm đúng 16 bytes và căn lề 16-byte
        assert_eq!(size_of::<Bitboard>(), 16);
        assert_eq!(align_of::<Bitboard>(), 16);

        // Kiểm tra State chiếm đúng 16 bytes và căn lề 16-byte
        assert_eq!(size_of::<State>(), 16);
        assert_eq!(align_of::<State>(), 16);

        // Kiểm tra Position chiếm đúng 448 bytes (7x64B L1 Cache Lines) và căn lề 64-byte
        assert_eq!(size_of::<Position>(), 448);
        assert_eq!(align_of::<Position>(), 64);
    }

    /// Kiểm thử phân tích (parse) và xuất chuỗi (export) FEN chuẩn vị trí xuất phát mặc định.
    #[test]
    fn default() {
        let fen = Parser::DEFAULT;
        let pos = Parser::parse(fen);
        let out = Serializer::export(&pos);
        assert_eq!(fen, out);
        assert_eq!(pos.side, 0); // Đỏ đi trước
        assert_eq!(pos.rule, 0); // rule50 = 0
        assert_eq!(pos.ply, 1);  // ply = 1
        assert_eq!(pos.hash, pos.compute()); // Zobrist hash trùng khớp tuyệt đối
    }

    /// Kiểm thử tính toàn vẹn 2 chiều FEN -> Position -> FEN (roundtrip test).
    #[test]
    fn roundtrip() {
        let text = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1";
        let pos = Parser::parse(text);
        let out = Serializer::export(&pos);
        assert_eq!(text, out);
    }

    /// Kiểm thử các phép toán Bitboard: bật/tắt bit, đếm bit (`count`), trích xuất LSB (`pop`, `lsb`).
    #[test]
    fn bitboard() {
        let mut bb = Bitboard::empty();
        assert!(!bb.active());
        assert_eq!(bb.count(), 0);

        let sq = Square::new(4, 0); // Ô e1 (index 4)
        bb.set(sq);
        assert!(bb.active());
        assert!(bb.test(sq));
        assert_eq!(bb.count(), 1);
        assert_eq!(bb.lsb(), Some(sq));

        let popped = bb.pop();
        assert_eq!(popped, Some(sq));
        assert!(!bb.active());
    }

    /// Kiểm thử tính nhất quán của mã băm Zobrist Hash khi MakeMove và UndoMove.
    #[test]
    fn zobrist() {
        let fen = Parser::DEFAULT;
        let mut pos = Parser::parse(fen);
        let orig = pos.hash;
        assert_eq!(orig, pos.compute());

        // Di chuyển Pháo Đỏ từ ô 19 (c2) lên ô 28 (c5)
        let from = 19u8;
        let to = 28u8;
        let state = pos.apply(from, to);

        assert_ne!(pos.hash, orig);
        assert_eq!(pos.hash, pos.compute());

        // Hoàn tác nước đi, khôi phục Zobrist hash cũ
        pos.revert(from, to, &state);
        assert_eq!(pos.hash, orig);
        assert_eq!(pos.hash, pos.compute());
    }

    /// Kiểm thử thực hiện (apply) và hoàn tác (revert) nước đi trực tiếp trên đối tượng Position.
    #[test]
    fn state() {
        let fen = Parser::DEFAULT;
        let mut pos = Parser::parse(fen);
        let orig = pos;

        let from = 19u8;
        let to = 28u8;
        let state = pos.apply(from, to);

        assert_eq!(pos.at(from), 14); // Ô từ đến trở thành ô trống (14)
        assert_eq!(pos.at(to), 5);    // Ô đích chứa Pháo Đỏ (mã 5)
        assert_eq!(pos.side, 1);      // Lượt đi đổi sang bên Đen

        pos.revert(from, to, &state);

        assert_eq!(pos.at(from), 5);
        assert_eq!(pos.at(to), 14);
        assert_eq!(pos.side, 0);
        assert_eq!(pos, orig);        // Phôi phục 100% bàn cờ gốc ban đầu
    }

    /// Kiểm thử trôi lệch băm (Hash Drift Test) với 1,000 nước đi và hoàn tác ngẫu nhiên liên tiếp.
    #[test]
    fn drift() {
        let text = Parser::DEFAULT;
        let mut pos = Parser::parse(text);
        let orig = pos;

        let mut seed = 0xDEADBEEF12345678u64;
        let mut history = Vec::with_capacity(1000);

        for _ in 0..1000 {
            let mut from = 0u8;
            let mut found = false;
            for _ in 0..90 {
                seed = seed.wrapping_add(0x9E3779B97F4A7C15);
                let idx = ((seed >> 24) % 90) as u8;
                if pos.at(idx) < 14 {
                    from = idx;
                    found = true;
                    break;
                }
            }
            if !found {
                continue;
            }

            seed = seed.wrapping_add(0x9E3779B97F4A7C15);
            let to = ((seed >> 16) % 90) as u8;

            if from != to {
                let state = pos.apply(from, to);
                assert_eq!(pos.hash, pos.compute(), "Phát hiện trôi lệch băm Zobrist hash khi apply!");
                history.push((from, to, state));
            }
        }

        while let Some((from, to, state)) = history.pop() {
            pos.revert(from, to, &state);
            assert_eq!(pos.hash, pos.compute(), "Phát hiện trôi lệch băm Zobrist hash khi revert!");
        }

        assert_eq!(pos, orig, "Bàn cờ không khớp bàn cờ gốc sau chuỗi 1000 nước đi hoàn tác!");
        assert_eq!(pos.hash, orig.hash, "Khóa băm không khớp khóa băm gốc sau hoàn tác!");
    }

    /// Kiểm thử đối kháng 1: Nước đi ô đi trùng ô đến (Same square move handling).
    #[test]
    fn same() {
        let fen = Parser::DEFAULT;
        let mut pos = Parser::parse(fen);
        let orig = pos;
        let sq = 19u8; // Pháo Đỏ
        let state = pos.apply(sq, sq);

        // Phải duy trì mã băm Zobrist đồng bộ
        assert_eq!(pos.hash, pos.compute(), "LỖI HASH DESYNC: pos.apply(sq, sq) làm lệch hash!");

        // Hoàn tác
        pos.revert(sq, sq, &state);

        assert_eq!(pos.counts[5], orig.counts[5], "LỖI TĂNG SỐ LƯỢNG QUÂN: revert(sq, sq) làm phình counts!");
        assert_eq!(pos, orig, "LỖI KHÔI PHỤC BÀN CỜ: revert(sq, sq) làm hỏng bàn cờ!");
    }

    /// Kiểm thử đối kháng 2: Ghi đè quân mới lên ô đang chứa quân cờ cũ (`put` occupied square).
    #[test]
    fn occupy() {
        let mut pos = Position::empty();
        pos.put(4, 0); // Đặt Xe Đỏ (4) vào ô 0
        assert_eq!(pos.counts[4], 1);
        assert!(pos.piece[4].test(Square(0)));

        pos.put(5, 0); // Đặt đè Pháo Đỏ (5) vào ô 0

        // Bitboard quân cũ phải được dọn dẹp sạch sẽ
        assert!(!pos.piece[4].test(Square(0)), "LỖI RÁC BITBOARD: put đè không dọn bitboard quân cũ!");
        assert_eq!(pos.counts[4], 0, "LỖI SỐ LƯỢNG QUÂN: put đè không giảm số lượng quân cũ!");
    }

    /// Kiểm thử đối kháng 3: Tránh lỗi tràn số u8 underflow khi flip ô cờ.
    #[test]
    fn underflow() {
        let sq = Square(90);
        let _flipped = sq.flip();
    }

    /// Kiểm thử đối kháng 4: Truy xuất ô vượt biên `at(90)` an toàn không bị panic.
    #[test]
    fn bounds() {
        let pos = Position::empty();
        let _p = pos.at(90);
    }
}




