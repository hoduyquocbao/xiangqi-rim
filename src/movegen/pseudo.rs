// ============================================================================
// MODULE PSEUDO: SINH NƯỚC ĐI GIẢ ĐỊNH CHO 7 LOẠI QUÂN CỜ TƯỚNG (PSEUDO-LEGAL GENERATOR)
// ============================================================================
// `pseudo.rs` chứa các hàm sinh nước đi giả định (Pseudo-Legal Moves) chưa qua thẩm định
// xem Tướng nhà có bị chiếu hay không.
// Các nước đi giả định tuân thủ đầy đủ quy tắc di chuyển vật lý của từng quân cờ:
// - Tướng: Đi trong Cung.
// - Sĩ: Đi chéo trong Cung.
// - Tượng: Đi chéo 2 ô, không bị cản mắt Tượng (`eye`).
// - Mã: Đi chữ Nhật, không bị cản chân Mã (`leg`).
// - Xe: Trượt trên 4 hướng, ăn quân đối phương.
// - Pháo: Trượt tự do và nhảy qua ngòi để ăn quân đối phương.
// - Tốt: Tiến 1 bước, qua sông được đi ngang.
// ============================================================================

use super::lookup;
use super::types::{List, Move};
use crate::board::{Position, Square};

/// Sinh tất cả nước đi giả định cho Tướng (King) của lượt đi hiện tại.
#[inline(always)]
pub fn king(pos: &Position, list: &mut List) {
    let side = pos.side as usize;
    let own = pos.color[side];
    let from = pos.king[side];
    if from >= 90 {
        return;
    }
    // Lấy Bitboard các ô tấn công hợp lệ của Tướng trong Cung trừ các ô có quân nhà
    let mut target = lookup::king(side, from as usize) & !own;
    while let Some(to) = target.pop() {
        list.push(Move::new(from, to.0));
    }
}

/// Sinh tất cả nước đi giả định cho Sĩ (Advisor) của lượt đi hiện tại.
#[inline(always)]
pub fn advisor(pos: &Position, list: &mut List) {
    let side = pos.side as usize;
    let own = pos.color[side];
    let mut bb = pos.piece[side * 7 + 1];
    while let Some(from) = bb.pop() {
        let mut target = lookup::advisor(side, from.index()) & !own;
        while let Some(to) = target.pop() {
            list.push(Move::new(from.0, to.0));
        }
    }
}

/// Sinh tất cả nước đi giả định cho Tượng (Elephant) của lượt đi hiện tại (Kiểm tra ô mắt Tượng).
#[inline(always)]
pub fn elephant(pos: &Position, list: &mut List) {
    let side = pos.side as usize;
    let own = pos.color[side];
    let occupied = pos.occupied;
    let mut bb = pos.piece[side * 7 + 2];
    while let Some(from) = bb.pop() {
        let mut target = lookup::elephant(side, from.index()) & !own;
        while let Some(to) = target.pop() {
            let eye = lookup::eye(from.index(), to.index());
            // Điều kiện Tượng không bị cản mắt: ô mắt Tượng không có quân đứng (!occupied.test)
            if eye < 90 && !occupied.test(Square(eye)) {
                list.push(Move::new(from.0, to.0));
            }
        }
    }
}

/// Sinh tất cả nước đi giả định cho Mã (Knight) của lượt đi hiện tại (Kiểm tra ô chân Mã).
#[inline(always)]
pub fn knight(pos: &Position, list: &mut List) {
    let side = pos.side as usize;
    let own = pos.color[side];
    let occupied = pos.occupied;
    let mut bb = pos.piece[side * 7 + 3];
    while let Some(from) = bb.pop() {
        let mut target = lookup::knight(from.index()) & !own;
        while let Some(to) = target.pop() {
            let leg = lookup::leg(from.index(), to.index());
            // Điều kiện Mã không bị cản chân: ô chân Mã không có quân đứng (!occupied.test)
            if leg < 90 && !occupied.test(Square(leg)) {
                list.push(Move::new(from.0, to.0));
            }
        }
    }
}

/// Sinh tất cả nước đi giả định cho Xe (Rook) của lượt đi hiện tại.
#[inline(always)]
pub fn rook(pos: &Position, list: &mut List) {
    let side = pos.side as usize;
    let own = pos.color[side];
    let occupied = pos.occupied;
    let mut bb = pos.piece[side * 7 + 4];
    while let Some(from) = bb.pop() {
        let mut target = lookup::rook(from.0, occupied, own);
        while let Some(to) = target.pop() {
            list.push(Move::new(from.0, to.0));
        }
    }
}

/// Sinh tất cả nước đi giả định cho Pháo (Cannon) của lượt đi hiện tại.
#[inline(always)]
pub fn cannon(pos: &Position, list: &mut List) {
    let side = pos.side as usize;
    let enemy = pos.color[side ^ 1];
    let occupied = pos.occupied;
    let mut bb = pos.piece[side * 7 + 5];
    while let Some(from) = bb.pop() {
        let mut target = lookup::cannon(from.0, occupied, enemy);
        while let Some(to) = target.pop() {
            list.push(Move::new(from.0, to.0));
        }
    }
}

/// Sinh tất cả nước đi giả định cho Tốt (Pawn) của lượt đi hiện tại.
#[inline(always)]
pub fn pawn(pos: &Position, list: &mut List) {
    let side = pos.side as usize;
    let own = pos.color[side];
    let mut bb = pos.piece[side * 7 + 6];
    while let Some(from) = bb.pop() {
        let mut target = lookup::pawn(side, from.index()) & !own;
        while let Some(to) = target.pop() {
            list.push(Move::new(from.0, to.0));
        }
    }
}

/// Hàm tổng quát sinh toàn bộ danh sách nước đi giả định (Pseudo-Legal Moves) cho 7 loại quân cờ.
#[inline(always)]
pub fn gen(pos: &Position, list: &mut List) {
    king(pos, list);
    advisor(pos, list);
    elephant(pos, list);
    knight(pos, list);
    rook(pos, list);
    cannon(pos, list);
    pawn(pos, list);
}

/// Bí danh (Alias) gọi hàm sinh nước đi giả định `gen`.
#[inline(always)]
pub fn pseudo(pos: &Position, list: &mut List) {
    gen(pos, list);
}

