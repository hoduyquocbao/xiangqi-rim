// ============================================================================
// MODULE FEATURE: TRÍCH XUẤT CHỈ SỐ ĐẶC TRƯNG NNUE (HALF-KA-V2-HM FEATURE EXTRACTOR)
// ============================================================================
// `feature.rs` chịu trách nhiệm tính toán chỉ số đặc trưng (0..65535) cho mạng nơ-ron NNUE:
// - Kiến trúc `HalfKAv2_hm` sử dụng vị trí Tướng (`king`), loại quân (`piece`), và vị trí quân (`square`).
// - Áp dụng thuộc tính đối xứng ngang (`mirror`) để giảm một nửa số lượng vị trí Tướng trong Cung (từ 9 xuống 5 ô ngang).
// - Hỗ trợ lật cờ dọc (`flip`) khi đổi góc nhìn giữa phe Đỏ và Đen (`side != view`).
// ============================================================================

use crate::board::Square;

/// Tổng số lượng chỉ số đặc trưng không gian NNUE HalfKAv2_hm = 65,536
pub const TOTAL: usize = 65536;

/// Struct `Feature` chứa các hàm tĩnh trích xuất chỉ số đặc trưng vị trí.
pub struct Feature;

impl Feature {
    /// Lật vị trí ô bàn cờ theo chiều dọc (ngược hàng $0 \leftrightarrow 9$, $1 \leftrightarrow 8$,...).
    #[inline(always)]
    pub fn flip(square: u8) -> u8 {
        Square(square).flip().0
    }

    /// Lật vị trí ô bàn cờ theo chiều ngang (ngược cột $0 \leftrightarrow 8$, $1 \leftrightarrow 7$,...).
    #[inline(always)]
    pub fn mirror(square: u8) -> u8 {
        let file = square % 9;
        let rank = square / 9;
        rank * 9 + (8 - file)
    }

    /// Tính toán chỉ số đặc trưng duy nhất (0..65535) cho bộ 3 (king, piece, square) theo phe `side` và góc nhìn `view`.
    #[inline(always)]
    pub fn index(king: u8, piece: u8, square: u8, side: u8, view: u8) -> usize {
        let (king, piece, square) = if side == view {
            (king, piece, square)
        } else {
            // Đổi góc nhìn: Đổi mã loại quân (Red <-> Black) và lật dọc tọa độ
            let swapped = if piece < 7 { piece + 7 } else { piece - 7 };
            (Self::flip(king), swapped, Self::flip(square))
        };

        let file = king % 9;
        let rank = king / 9;
        let tf = square % 9;
        let tr = square / 9;

        // Chuẩn hóa đối xứng ngang: Nếu Tướng nằm ở nửa cột phải (file > 4), lật ngang cả Tướng và quân
        let (norm, target) = if file > 4 {
            let norm = rank * 9 + (8 - file);
            let target = tr * 9 + (8 - tf);
            (norm, target)
        } else {
            (king, square)
        };

        let col = norm % 9;
        let row = norm / 9;
        let base = row * 5 + col;

        // Công thức mã hóa băm đặc trưng HalfKAv2_hm
        (base as usize) * 1260 + (piece as usize) * 90 + (target as usize)
    }
}


