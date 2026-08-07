// ============================================================================
// MODULE LEGAL: THẨM ĐỊNH NƯỚC ĐI HỢP LỆ TUYỆT ĐỐI (STRICT LEGAL MOVES FILTER)
// ============================================================================
// `legal.rs` chịu trách nhiệm thẩm định và lọc các nước đi hợp lệ tuyệt đối (Legal Moves):
// - `fly()`: Kiểm tra quy tắc Lộ mặt Tướng (Flying General Rule - 2 Tướng nhìn thẳng mặt nhau không có quân cản).
// - `check()`: Thẩm định xem Tướng của bên `side` có đang bị chiếu bởi Tốt, Mã, Xe, Pháo đối phương hay không.
// - `gen()` / `legal()`: Sinh toàn bộ nước đi pseudo-legal, thử di chuyển `apply()`, kiểm tra `!check()` và `!fly()`,
//   sau đó khôi phục `revert()`. Chi phí $O(1)$ tuyệt đối!
// ============================================================================

use super::lookup;
use super::pseudo;
use super::types::List;
use crate::board::{Position, Square};

/// Kiểm tra quy tắc Lộ mặt Tướng (Flying General / Facing Kings).
/// Trả về `true` nếu 2 Tướng nằm trên cùng 1 cột và không có bất kỳ quân cản nào ở giữa!
#[inline(always)]
pub fn fly(pos: &Position) -> bool {
    let red = pos.king[0];
    let black = pos.king[1];
    if red >= 90 || black >= 90 {
        return false;
    }
    let first = Square(red);
    let second = Square(black);
    // Nếu không cùng Cột (File) -> Không vi phạm
    if first.file() != second.file() {
        return false;
    }
    let floor = if first.rank() < second.rank() {
        first.rank()
    } else {
        second.rank()
    };
    let roof = if first.rank() > second.rank() {
        first.rank()
    } else {
        second.rank()
    };
    let file = first.file();

    // Duyệt khoảng giữa 2 Tướng xem có quân cản hay không
    let mut r = floor + 1;
    while r < roof {
        let sq = r * 9 + file;
        if pos.occupied.test(Square(sq)) {
            return false;
        }
        r += 1;
    }
    true
}

/// Kiểm tra xem Tướng của phe `side` có đang bị phe đối phương (`side ^ 1`) chiếu hay không.
#[inline(always)]
pub fn check(pos: &Position, side: usize) -> bool {
    let king = pos.king[side];
    if king >= 90 {
        return false;
    }
    let enemy = side ^ 1;

    // 1. Kiểm tra Tốt đối phương chiếu Tướng (chiếu tiến hoặc chiếu ngang trong Cung)
    let pawns = pos.piece[enemy * 7 + 6];
    let k = king as usize;
    let f = (k % 9) as i8;
    let mut mask = 0u128;
    if side == 0 {
        if k + 9 < 90 {
            mask |= 1u128 << (k + 9);
        }
        if f > 0 {
            mask |= 1u128 << (k - 1);
        }
        if f < 8 {
            mask |= 1u128 << (k + 1);
        }
    } else {
        if k >= 9 {
            mask |= 1u128 << (k - 9);
        }
        if f > 0 {
            mask |= 1u128 << (k - 1);
        }
        if f < 8 {
            mask |= 1u128 << (k + 1);
        }
    }
    if (crate::board::Bitboard(mask) & pawns).active() {
        return true;
    }

    // 2. Kiểm tra Mã đối phương chiếu Tướng (truyền đúng ô nguồn origin và kiểm tra chân Mã leg)
    let knights = pos.piece[enemy * 7 + 3];
    let mut bb = lookup::knight(k) & knights;
    while let Some(origin) = bb.pop() {
        let leg = lookup::leg(origin.index(), k);
        if leg < 90 && !pos.occupied.test(Square(leg)) {
            return true;
        }
    }

    // 3. Kiểm tra Xe đối phương chiếu Tướng
    let rooks = pos.piece[enemy * 7 + 4];
    if (lookup::rook(king, pos.occupied, pos.color[side]) & rooks).active() {
        return true;
    }

    // 4. Kiểm tra Pháo đối phương chiếu Tướng qua ngòi
    let cannons = pos.piece[enemy * 7 + 5];
    let mut d = 0;
    while d < 4 {
        let r = lookup::ray(d, k);
        let block = r & pos.occupied;
        if block.active() {
            let mount = if d == 0 || d == 2 {
                block.lsb().unwrap().index()
            } else {
                block.msb().unwrap().index()
            };
            let behind = lookup::ray(d, mount) & pos.occupied;
            if behind.active() {
                let victim = if d == 0 || d == 2 {
                    behind.lsb().unwrap().index()
                } else {
                    behind.msb().unwrap().index()
                };
                if cannons.test(Square(victim as u8)) {
                    return true;
                }
            }
        }
        d += 1;
    }

    false
}

/// Thẩm định một nước đi `mv` có hợp lệ tuyệt đối hay không (Single move legal validation).
#[inline(always)]
pub fn valid(pos: &mut Position, mv: crate::movegen::types::Move) -> bool {
    if !mv.valid() {
        return false;
    }
    let from = mv.from as usize;
    let to = mv.to as usize;
    let piece = pos.grid[from];
    if piece >= 14 {
        return false;
    }
    let side = pos.side as usize;
    if (piece / 7) as usize != side {
        return false;
    }
    let dest = pos.grid[to];
    if dest < 14 && (dest / 7) as usize == side {
        return false;
    }

    let own = pos.color[side];
    let enemy = pos.color[side ^ 1];
    let occupied = pos.occupied;
    let role = (piece % 7) as usize;

    let target = match role {
        0 => lookup::king(side, from) & !own,
        1 => lookup::advisor(side, from) & !own,
        2 => {
            let eye = lookup::eye(from, to);
            if eye < 90 && occupied.test(Square(eye)) {
                return false;
            }
            lookup::elephant(side, from) & !own
        }
        3 => {
            let leg = lookup::leg(from, to);
            if leg < 90 && occupied.test(Square(leg)) {
                return false;
            }
            lookup::knight(from) & !own
        }
        4 => lookup::rook(mv.from, occupied, own),
        5 => lookup::cannon(mv.from, occupied, enemy),
        6 => lookup::pawn(side, from) & !own,
        _ => return false,
    };

    if !target.test(Square(mv.to)) {
        return false;
    }

    let state = pos.apply(mv.from, mv.to);
    let ok = !check(pos, side) && !fly(pos);
    pos.revert(mv.from, mv.to, &state);
    ok
}

/// Sinh và lọc tất cả nước đi hợp lệ tuyệt đối (Legal Moves) cho vị trí bàn cờ hiện tại.
#[inline(always)]
pub fn gen(pos: &mut Position, moves: &mut List) {
    moves.clear();
    let mut raw = List::new();
    pseudo::gen(pos, &mut raw);

    let side = pos.side as usize;
    let mut i = 0;
    while i < raw.count {
        let mv = raw.items[i];
        // Thử thực thi nước đi
        let state = pos.apply(mv.from, mv.to);
        // Thẩm định: Tướng không bị chiếu VÀ không bị lộ mặt Tướng
        if !check(pos, side) && !fly(pos) {
            moves.push(mv);
        }
        // Hoàn tác nước đi
        pos.revert(mv.from, mv.to, &state);
        i += 1;
    }
}

/// Bí danh (Alias) gọi hàm sinh nước đi hợp lệ tuyệt đối `gen`.
#[inline(always)]
pub fn legal(pos: &mut Position, moves: &mut List) {
    gen(pos, moves);
}


