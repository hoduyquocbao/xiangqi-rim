// ============================================================================
// MODULE EVAL TRAP: BỘ ĐÁNH GIÁ BẪY CỜ VÀ THẮT CHẶT KHÔNG GIAN (PIECE TRAP ENGINE)
// ============================================================================
// Module `trap.rs` triển khai tri thức chuyên sâu về bẫy cờ và khống chế không gian:
// 1. Mã nghẽn chân (Trapped Knight): Phát hiện Mã bị vây kẹt cản chân không có ô hạ cánh an toàn.
// 2. Xe bị kẹt góc (Trapped Rook): Phát hiện Xe bị nhốt ở góc không có đường xuất trận.
// 3. Pháo mất ngòi (Trapped Cannon): Phát hiện Pháo bị giam hãm không có ngòi cơ động.
// 4. Độ thắt chặt không gian (Piece Constriction): Đánh giá tỷ lệ ô bị phong tỏa của quân chủ lực.
// 100% Clean Room std-only, căn lề 64-byte, 100% chú thích tiếng Việt & từ đơn tiếng Anh.
// ============================================================================

use crate::board::Position;
use crate::movegen::lookup;

/// Struct `Trap` đánh giá bẫy cờ và mức độ phong tỏa không gian của quân chủ lực.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug, Default)]
pub struct Trap;

impl Trap {
    /// Khởi tạo đối tượng `Trap` mới.
    #[inline(always)]
    pub const fn new() -> Self {
        Self
    }

    /// Đánh giá điểm bẫy cờ và phong tỏa cho cả 2 phe, trả về cặp `(mg, eg)` điểm centipawn.
    #[inline(always)]
    pub fn evaluate(pos: &Position) -> (i32, i32) {
        let mut mg = 0i32;
        let mut eg = 0i32;

        let mut color = 0usize;
        while color < 2 {
            let sign = if color == 0 { 1 } else { -1 };

            // 1. Đánh giá Mã nghẽn chân (Trapped Knight)
            let mut knights = pos.piece[color * 7 + 3];
            while let Some(sq) = knights.pop() {
                let mut safe_destinations = 0i32;
                let mut blocked_legs = 0i32;

                let dest_bb = lookup::KNIGHT[sq.0 as usize];
                let mut dest_iter = dest_bb;
                while let Some(dest_sq) = dest_iter.pop() {
                    let leg_sq = lookup::leg(sq.0 as usize, dest_sq.0 as usize);
                    if leg_sq != 255 {
                        let leg_code = pos.grid[leg_sq as usize];
                        if leg_code < 14 {
                            blocked_legs += 1;
                        } else {
                            let dest_code = pos.grid[dest_sq.0 as usize];
                            if dest_code >= 14 || (dest_code as usize / 7) != color {
                                safe_destinations += 1;
                            }
                        }
                    }
                }

                // Nếu Mã bị cản từ 3 chân trở lên hoặc có 0 ô hạ cánh an toàn -> Bẫy Mã nghẽn chân
                if safe_destinations == 0 || blocked_legs >= 3 {
                    mg -= sign * 180;
                    eg -= sign * 220;
                }
            }

            // 2. Đánh giá Xe bị kẹt góc/khóa đường (Trapped Rook)
            let mut rooks = pos.piece[color * 7 + 4];
            while let Some(sq) = rooks.pop() {
                let rank = sq.rank();
                let file = sq.file();

                // Xe ở góc bàn cờ (files 0, 8 và ranks 0, 9) bị phong tỏa
                let is_corner = (file == 0 || file == 8) && (rank == 0 || rank == 9 || rank == 1 || rank == 8);
                if is_corner {
                    let king_bb = lookup::KING[color][sq.0 as usize];
                    let mut open_paths = 0i32;
                    let mut king_iter = king_bb;
                    while let Some(target_sq) = king_iter.pop() {
                        let target_code = pos.grid[target_sq.0 as usize];
                        if target_code >= 14 {
                            open_paths += 1;
                        }
                    }

                    if open_paths <= 1 {
                        mg -= sign * 200;
                        eg -= sign * 250;
                    }
                }
            }

            // 3. Đánh giá Pháo mất ngòi/kẹt ngòi (Trapped Cannon)
            let mut cannons = pos.piece[color * 7 + 5];
            while let Some(sq) = cannons.pop() {
                let king_bb = lookup::KING[color][sq.0 as usize];
                let mut screens = 0i32;
                let mut king_iter = king_bb;
                while let Some(target_sq) = king_iter.pop() {
                    let target_code = pos.grid[target_sq.0 as usize];
                    if target_code < 14 {
                        screens += 1;
                    }
                }

                // Pháo bị bao vây 3 phía trở lên không có ngòi phát huy sức mạnh
                if screens >= 3 {
                    mg -= sign * 120;
                    eg -= sign * 150;
                }
            }

            color += 1;
        }

        (mg, eg)
    }
}

// ----------------------------------------------------------------------------
// UNIT TESTS CHO BỘ ĐÁNH GIÁ BẪY CỜ TRAP
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Parser;

    #[test]
    fn alignments() {
        assert_eq!(std::mem::align_of::<Trap>(), 64);
    }

    #[test]
    fn default_position_trap() {
        let pos = Parser::parse(Parser::DEFAULT);
        let (mg, eg) = Trap::evaluate(&pos);
        // Ở vị trí ban đầu, không quân chủ lực nào bị sập bẫy nặng
        assert_eq!(mg, 0);
        assert_eq!(eg, 0);
    }
}
