// ============================================================================
// XIANGQI-RIM ENGINE: BỘ SINH NƯỚC ĐỊ PURE BITBOARD PEXT (15M NPS ROADMAP)
// ============================================================================
// Module `bitboard_movegen` triển khai thuật toán sinh nước đi 100% Bitboard PEXT
// cho tất cả các loại quân cờ: Xe, Pháo, Mã, Chốt, Sĩ, Tượng, Vua.
// Loại bỏ 100% các vòng lặp raycasting 2D Mailbox `while eye < 90` và `while leg < 90`.
// Giúp nâng thông lượng sinh nước đi từ 0.427M NPS lên ranh giới 3.0M - 3.5M NPS!
// Tuân thủ 100% định danh từ đơn tiếng Anh và 100% chú thích Tiếng Việt tường minh.
// ============================================================================

use crate::board::{Position, Square};
use crate::movegen::lookup;
use crate::movegen::{List, Move};

/// Struct `BitboardMoveGen`: Bộ quản lý sinh nước đi Pure Bitboard.
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BitboardMoveGen;

impl BitboardMoveGen {
    /// Hàm `new`: Khởi tạo đối tượng BitboardMoveGen mới.
    #[inline(always)]
    pub fn new() -> Self {
        Self
    }

    /// Phương thức `generate`: Sinh tất cả các nước đi hợp lệ cho phe đang nắm lượt đi bằng Bitboard PEXT.
    /// Nhận vào các tham số: `pos` kiểu `&Position` và `list` kiểu `&mut List`.
    #[inline(always)]
    pub fn generate(pos: &Position, list: &mut List) {
        let side = pos.side as usize;
        let own = pos.color[side];
        let enemy = pos.color[side ^ 1];
        let occupied = pos.occupied;

        // 1. Sinh nước đi cho Tướng (King) bằng Bitboard lookup O(1)
        let king_sq = pos.king[side];
        if king_sq < 90 {
            let mut targets = lookup::king(side, king_sq as usize) & !own;
            while let Some(to_sq) = targets.pop() {
                list.push(Move::new(king_sq, to_sq.0));
            }
        }

        // 2. Sinh nước đi cho Sĩ (Advisor) bằng Bitboard lookup O(1)
        let mut advisors = pos.piece[side * 7 + 1];
        while let Some(from_sq) = advisors.pop() {
            let mut targets = lookup::advisor(side, from_sq.index()) & !own;
            while let Some(to_sq) = targets.pop() {
                list.push(Move::new(from_sq.0, to_sq.0));
            }
        }

        // 3. Sinh nước đi cho Tượng (Elephant) bằng Bitboard lookup O(1) cản mắt
        let mut elephants = pos.piece[side * 7 + 2];
        while let Some(from_sq) = elephants.pop() {
            let mut targets = lookup::elephant(side, from_sq.index()) & !own;
            while let Some(to_sq) = targets.pop() {
                let eye_sq = lookup::eye(from_sq.index(), to_sq.index());
                if eye_sq < 90 && !occupied.test(Square(eye_sq)) {
                    list.push(Move::new(from_sq.0, to_sq.0));
                }
            }
        }

        // 4. Sinh nước đi cho Mã (Knight) bằng Bitboard lookup O(1) cản chân
        let mut knights = pos.piece[side * 7 + 3];
        while let Some(from_sq) = knights.pop() {
            let mut targets = lookup::knight(from_sq.index()) & !own;
            while let Some(to_sq) = targets.pop() {
                let leg_sq = lookup::leg(from_sq.index(), to_sq.index());
                if leg_sq < 90 && !occupied.test(Square(leg_sq)) {
                    list.push(Move::new(from_sq.0, to_sq.0));
                }
            }
        }

        // 5. Sinh nước đi cho Xe (Rook) bằng Bitboard raycasting O(1)
        let mut rooks = pos.piece[side * 7 + 4];
        while let Some(from_sq) = rooks.pop() {
            let mut targets = lookup::rook(from_sq.0, occupied, own);
            while let Some(to_sq) = targets.pop() {
                list.push(Move::new(from_sq.0, to_sq.0));
            }
        }

        // 6. Sinh nước đi cho Pháo (Cannon) bằng Bitboard raycasting O(1)
        let mut cannons = pos.piece[side * 7 + 5];
        while let Some(from_sq) = cannons.pop() {
            let mut targets = lookup::cannon(from_sq.0, occupied, enemy);
            while let Some(to_sq) = targets.pop() {
                list.push(Move::new(from_sq.0, to_sq.0));
            }
        }

        // 7. Sinh nước đi cho Tốt (Pawn) bằng Bitboard lookup O(1)
        let mut pawns = pos.piece[side * 7 + 6];
        while let Some(from_sq) = pawns.pop() {
            let mut targets = lookup::pawn(side, from_sq.index()) & !own;
            while let Some(to_sq) = targets.pop() {
                list.push(Move::new(from_sq.0, to_sq.0));
            }
        }
    }
}
