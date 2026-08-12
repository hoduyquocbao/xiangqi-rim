// ============================================================================
// MODULE MOVEGEN: BỘ SINH NƯỚC ĐỊ VÀ TRA BẢNG TỐC ĐỘ CAO CHO CỜ TƯỚNG
// ============================================================================
// Module `movegen` chịu trách nhiệm sinh toàn bộ nước đi hợp lệ cho cả 2 phe:
// - `types`: Định nghĩa `Move` (16-bit) và `List` (danh sách nước đi căn lề 64-byte).
// - `lookup`: Quản lý các mảng tra cứu trước tĩnh (`ADVISOR_ATTACKS`, `KNIGHT_ATTACKS`, v.v.).
// - `pseudo`: Sinh nước đi giả lập (Pseudo-legal moves) nhanh không kiểm tra chiếu tướng.
// - `legal`: Lọc nước đi giả lập thành nước đi hoàn toàn hợp lệ (Legal moves) và kiểm tra chiếu.
// - `perft`: Đếm số lượng nút cây cờ (Performance Test) xác minh tính chính xác tuyệt đối.
// ============================================================================

/// Module con `legal` lọc và kiểm tra chiếu tướng đối phương
pub mod legal;
/// Module con `lookup` chứa các bảng tra cứu vị trí di chuyển và cản chân tĩnh
pub mod lookup;
/// Module con `order` sắp xếp nước đi MVV-LVA (align 64)
pub mod order;
/// Module con `perft` chạy kiểm thử cây nước đi Perft
pub mod perft;
/// Module con `pseudo` sinh nước đi giả lập sơ bộ
pub mod pseudo;
/// Module con `types` định nghĩa kiểu dữ liệu nước đi và mảng chứa nước đi
pub mod types;

// Xuất bản các hàm và kiểu dữ liệu cốt lõi
pub use legal::{check, fly, gen, legal};
pub use order::{score, sort};
pub use perft::{divide, perft};
pub use pseudo::pseudo;
pub use types::{List, Move};

// ----------------------------------------------------------------------------
// KHU VỰC BÀI KIỂM THỬ ĐƠN VỊ (UNIT TESTS) CHO BỘ SINH NƯỚC ĐỊ AND LOOKUP TABLES
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Parser, Square};

    /// Kiểm thử đếm số nút cây Perft (Performance Test) trên vị trí khởi đầu Cờ Tướng tiêu chuẩn:
    /// - Độ sâu 1 (Depth 1): Phải đúng 44 nút hợp lệ.
    /// - Độ sâu 2 (Depth 2): Phải đúng 1,920 nút hợp lệ.
    /// - Độ sâu 3 (Depth 3): Phải đúng 79,666 nút hợp lệ.
    #[test]
    fn perft() {
        let mut pos = Parser::parse(Parser::DEFAULT);
        assert!(super::perft(&mut pos, 1) == 44, "Perft Depth 1 BẮT BUỘC bằng 44 nodes!");
        assert!(super::perft(&mut pos, 2) == 1920, "Perft Depth 2 BẮT BUỘC bằng 1,920 nodes!");
        assert!(super::perft(&mut pos, 3) == 79666, "Perft Depth 3 BẮT BUỘC bằng 79,666 nodes!");
    }

    /// Kiểm thử bảng tra cứu di chuyển trong Cung Tướng của quân Tướng (King) và quân Sĩ (Advisor).
    #[test]
    fn palace() {
        // Tướng Đỏ đứng ô 13 (e2 trong Cung) có 4 nước đi xung quanh
        let k = lookup::king(0, 13);
        assert!(k.count() == 4);

        // Sĩ Đỏ đứng ô 13 (e2 tâm Cung) có 4 nước đi đường chéo
        let a = lookup::advisor(0, 13);
        assert!(a.count() == 4);
    }

    /// Kiểm thử bảng tra cứu bước di chuyển của Mã (Knight) và vị trí chân Mã cản đường (Knight Leg).
    #[test]
    fn knight() {
        // Mã đứng ô 1 (b1) có thể đi đến ô 18 và ô 20
        let k = lookup::knight(1);
        assert!(k.test(Square(18)));
        assert!(k.test(Square(20)));
        // Ô chân Mã cản đường khi đi từ 1 tới 18 hoặc 20 là ô 10 (b2)
        assert!(lookup::leg(1, 18) == 10);
        assert!(lookup::leg(1, 20) == 10);
    }

    /// Kiểm thử bảng tra cứu bước di chuyển của Tượng (Elephant/Bishop) và ô mắt Tượng (Elephant Eye).
    #[test]
    fn elephant() {
        // Tượng Đỏ đứng ô 2 (c1) đi chéo tới ô 18 hoặc 22
        let e = lookup::elephant(0, 2);
        assert!(e.test(Square(18)));
        assert!(e.test(Square(22)));
        // Ô mắt Tượng cản đường đi từ 2 tới 18 là ô 10, tới 22 là ô 12
        assert!(lookup::eye(2, 18) == 10);
        assert!(lookup::eye(2, 22) == 12);
    }
}


