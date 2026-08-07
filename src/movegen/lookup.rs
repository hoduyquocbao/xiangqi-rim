// ============================================================================
// MODULE LOOKUP: BẢNG TRA CỨU TĨNH SỰ TẤN CÔNG VÀ NƯỚC ĐI (STATIC ATTACK LOOKUP TABLES)
// ============================================================================
// Module này khởi tạo toàn bộ các bảng tra cứu static `pub static` tại mốc biên dịch (`const fn`):
// - `KING`: Mảng Bitboard nước đi Tướng theo phe trong Cung (Palace: $3 \times 3$).
// - `ADVISOR`: Mảng Bitboard nước đi Sĩ chéo trong Cung.
// - `ELEPHANTS` & `EYE`: Bảng Bitboard bước nhảy Tượng và bảng ô mắt Tượng cản `EYE`.
// - `KNIGHTS` & `LEG`: Bảng Bitboard bước nhảy Mã và bảng ô chân Mã cản `LEG`.
// - `PAWN`: Bảng Bitboard nước đi Tốt theo mốc qua sông (River crossing).
// - `RAY`: Bảng Bitboard 4 tia chiếu hướng (North, South, East, West).
// - Phương thức `rook()` và `cannon()` tính nước đi Xe và Pháo dựa trên các bitwise LSB/MSB.
// ============================================================================

use crate::board::{Bitboard, Square};

/// Hàm `const fn` tính bảng tra cứu nước đi của Tướng (King) trong Cung cho 2 phe.
const fn kings() -> [[Bitboard; 90]; 2] {
    let mut table = [[Bitboard(0); 90]; 2];
    let mut side = 0;
    while side < 2 {
        let floor = if side == 0 { 0 } else { 7 };
        let roof = if side == 0 { 2 } else { 9 };
        let mut sq = 0;
        while sq < 90 {
            let f = (sq % 9) as i8;
            let r = (sq / 9) as i8;
            // Kiểm tra ô xuất phát có nằm trong Cung (Files 3..5)
            if f >= 3 && f <= 5 && r >= floor as i8 && r <= roof as i8 {
                let mut mask = 0u128;
                let dirs: [(i8, i8); 4] = [(0, 1), (0, -1), (-1, 0), (1, 0)];
                let mut d = 0;
                while d < 4 {
                    let nf = f + dirs[d].0;
                    let nr = r + dirs[d].1;
                    if nf >= 3 && nf <= 5 && nr >= floor as i8 && nr <= roof as i8 {
                        let target = (nr * 9 + nf) as u8;
                        mask |= 1u128 << target;
                    }
                    d += 1;
                }
                table[side][sq] = Bitboard(mask);
            }
            sq += 1;
        }
        side += 1;
    }
    table
}

/// Hàm `const fn` tính bảng tra cứu nước đi chéo của Sĩ (Advisor) trong Cung cho 2 phe.
const fn advisors() -> [[Bitboard; 90]; 2] {
    let mut table = [[Bitboard(0); 90]; 2];
    let mut side = 0;
    while side < 2 {
        let floor = if side == 0 { 0 } else { 7 };
        let roof = if side == 0 { 2 } else { 9 };
        let mut sq = 0;
        while sq < 90 {
            let f = (sq % 9) as i8;
            let r = (sq / 9) as i8;
            if f >= 3 && f <= 5 && r >= floor as i8 && r <= roof as i8 {
                let mut mask = 0u128;
                let dirs: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
                let mut d = 0;
                while d < 4 {
                    let nf = f + dirs[d].0;
                    let nr = r + dirs[d].1;
                    if nf >= 3 && nf <= 5 && nr >= floor as i8 && nr <= roof as i8 {
                        let target = (nr * 9 + nf) as u8;
                        mask |= 1u128 << target;
                    }
                    d += 1;
                }
                table[side][sq] = Bitboard(mask);
            }
            sq += 1;
        }
        side += 1;
    }
    table
}

/// Hàm `const fn` tính bảng tra cứu nước đi của Tượng (Elephant) và ô cản mắt Tượng (`EYE`).
const fn elephants() -> ([[Bitboard; 90]; 2], [[u8; 90]; 90]) {
    let mut table = [[Bitboard(0); 90]; 2];
    let mut eye = [[255u8; 90]; 90];
    let mut side = 0;
    while side < 2 {
        let floor = if side == 0 { 0 } else { 5 };
        let roof = if side == 0 { 4 } else { 9 };
        let mut sq = 0;
        while sq < 90 {
            let f = (sq % 9) as i8;
            let r = (sq / 9) as i8;
            if r >= floor as i8 && r <= roof as i8 {
                let mut mask = 0u128;
                let dirs: [(i8, i8); 4] = [(2, 2), (2, -2), (-2, 2), (-2, -2)];
                let mut d = 0;
                while d < 4 {
                    let nf = f + dirs[d].0;
                    let nr = r + dirs[d].1;
                    if nf >= 0 && nf <= 8 && nr >= floor as i8 && nr <= roof as i8 {
                        let target = (nr * 9 + nf) as usize;
                        mask |= 1u128 << target;

                        // Tính tọa độ trung điểm mắt Tượng cản
                        let ef = (f + nf) / 2;
                        let er = (r + nr) / 2;
                        let slot = (er * 9 + ef) as u8;
                        eye[sq][target] = slot;
                    }
                    d += 1;
                }
                table[side][sq] = Bitboard(mask);
            }
            sq += 1;
        }
        side += 1;
    }
    (table, eye)
}

/// Hàm `const fn` tính bảng tra cứu bước nhảy của Mã (Knight) và ô chân Mã cản (`LEG`).
const fn knights() -> ([Bitboard; 90], [[u8; 90]; 90]) {
    let mut table = [Bitboard(0); 90];
    let mut leg = [[255u8; 90]; 90];
    let mut sq = 0;
    while sq < 90 {
        let f = (sq % 9) as i8;
        let r = (sq / 9) as i8;
        let moves: [(i8, i8, i8, i8); 8] = [
            (1, 2, 0, 1),
            (-1, 2, 0, 1),
            (1, -2, 0, -1),
            (-1, -2, 0, -1),
            (2, 1, 1, 0),
            (2, -1, 1, 0),
            (-2, 1, -1, 0),
            (-2, -1, -1, 0),
        ];
        let mut mask = 0u128;
        let mut i = 0;
        while i < 8 {
            let (df, dr, lf, lr) = moves[i];
            let nf = f + df;
            let nr = r + dr;
            if nf >= 0 && nf <= 8 && nr >= 0 && nr <= 9 {
                let target = (nr * 9 + nf) as usize;
                mask |= 1u128 << target;
                // Vị trí chân Mã cản tương ứng
                let slot = ((r + lr) * 9 + (f + lf)) as u8;
                leg[sq][target] = slot;
            }
            i += 1;
        }
        table[sq] = Bitboard(mask);
        sq += 1;
    }
    (table, leg)
}

/// Hàm `const fn` tính bảng tra cứu nước đi của Tốt (Pawn) chưa qua sông và đã qua sông.
const fn pawns() -> [[Bitboard; 90]; 2] {
    let mut table = [[Bitboard(0); 90]; 2];
    let mut side = 0;
    while side < 2 {
        let mut sq = 0;
        while sq < 90 {
            let f = (sq % 9) as i8;
            let r = (sq / 9) as i8;
            let mut mask = 0u128;
            if side == 0 {
                if r + 1 <= 9 {
                    mask |= 1u128 << ((r + 1) * 9 + f);
                }
                // Nếu Đỏ đã qua sông (rank >= 5) -> được đi ngang
                if r >= 5 {
                    if f - 1 >= 0 {
                        mask |= 1u128 << (r * 9 + (f - 1));
                    }
                    if f + 1 <= 8 {
                        mask |= 1u128 << (r * 9 + (f + 1));
                    }
                }
            } else {
                if r - 1 >= 0 {
                    mask |= 1u128 << ((r - 1) * 9 + f);
                }
                // Nếu Đen đã qua sông (rank <= 4) -> được đi ngang
                if r <= 4 {
                    if f - 1 >= 0 {
                        mask |= 1u128 << (r * 9 + (f - 1));
                    }
                    if f + 1 <= 8 {
                        mask |= 1u128 << (r * 9 + (f + 1));
                    }
                }
            }
            table[side][sq] = Bitboard(mask);
            sq += 1;
        }
        side += 1;
    }
    table
}

/// Hàm `const fn` tính bảng tra cứu 4 tia chiếu Ray Bitboard (North=0, South=1, East=2, West=3).
const fn rays() -> [[Bitboard; 90]; 4] {
    let mut table = [[Bitboard(0); 90]; 4];
    let mut sq = 0;
    while sq < 90 {
        let f = (sq % 9) as i8;
        let r = (sq / 9) as i8;

        // Hướng 0: North (+9, rank tăng)
        let mut mask0 = 0u128;
        let mut nr = r + 1;
        while nr <= 9 {
            mask0 |= 1u128 << (nr * 9 + f);
            nr += 1;
        }
        table[0][sq] = Bitboard(mask0);

        // Hướng 1: South (-9, rank giảm)
        let mut mask1 = 0u128;
        nr = r - 1;
        while nr >= 0 {
            mask1 |= 1u128 << (nr * 9 + f);
            nr -= 1;
        }
        table[1][sq] = Bitboard(mask1);

        // Hướng 2: East (+1, file tăng)
        let mut mask2 = 0u128;
        let mut nf = f + 1;
        while nf <= 8 {
            mask2 |= 1u128 << (r * 9 + nf);
            nf += 1;
        }
        table[2][sq] = Bitboard(mask2);

        // Hướng 3: West (-1, file giảm)
        let mut mask3 = 0u128;
        nf = f - 1;
        while nf >= 0 {
            mask3 |= 1u128 << (r * 9 + nf);
            nf -= 1;
        }
        table[3][sq] = Bitboard(mask3);

        sq += 1;
    }
    table
}

static ELEPHANTS_DATA: ([[Bitboard; 90]; 2], [[u8; 90]; 90]) = elephants();
static KNIGHTS_DATA: ([Bitboard; 90], [[u8; 90]; 90]) = knights();

/// Bảng Bitboard tấn công của Tướng tĩnh toàn cục cho 2 phe
pub static KING: [[Bitboard; 90]; 2] = kings();
/// Bảng Bitboard tấn công của Sĩ tĩnh toàn cục cho 2 phe
pub static ADVISOR: [[Bitboard; 90]; 2] = advisors();
/// Bảng Bitboard di chuyển của Tượng tĩnh toàn cục cho 2 phe
pub static ELEPHANT: [[Bitboard; 90]; 2] = ELEPHANTS_DATA.0;
/// Bảng vị trí ô mắt Tượng cản
pub static EYE: [[u8; 90]; 90] = ELEPHANTS_DATA.1;
/// Bảng Bitboard di chuyển của Mã tĩnh toàn cục
pub static KNIGHT: [Bitboard; 90] = KNIGHTS_DATA.0;
/// Bảng vị trí ô chân Mã cản
pub static LEG: [[u8; 90]; 90] = KNIGHTS_DATA.1;
/// Bảng Bitboard di chuyển của Tốt tĩnh toàn cục cho 2 phe
pub static PAWN: [[Bitboard; 90]; 2] = pawns();
/// Bảng Bitboard 4 tia chiếu Rayleigh toàn cục
pub static RAY: [[Bitboard; 90]; 4] = rays();

/// Lấy Bitboard các nước đi tấn công của Tướng tại ô `sq` của phe `side`.
#[inline(always)]
pub fn king(side: usize, sq: usize) -> Bitboard {
    KING[side][sq]
}

/// Lấy Bitboard các nước đi tấn công của Sĩ tại ô `sq` của phe `side`.
#[inline(always)]
pub fn advisor(side: usize, sq: usize) -> Bitboard {
    ADVISOR[side][sq]
}

/// Lấy Bitboard các nước đi di chuyển của Tượng tại ô `sq` của phe `side`.
#[inline(always)]
pub fn elephant(side: usize, sq: usize) -> Bitboard {
    ELEPHANT[side][sq]
}

/// Lấy chỉ số ô mắt Tượng cản nằm giữa ô xuất phát `from` và ô đích `to`.
#[inline(always)]
pub fn eye(from: usize, to: usize) -> u8 {
    EYE[from][to]
}

/// Lấy Bitboard các nước đi di chuyển của Mã tại ô `sq`.
#[inline(always)]
pub fn knight(sq: usize) -> Bitboard {
    KNIGHT[sq]
}

/// Lấy chỉ số ô chân Mã cản nằm giữa ô xuất phát `from` và ô đích `to`.
#[inline(always)]
pub fn leg(from: usize, to: usize) -> u8 {
    LEG[from][to]
}

/// Lấy Bitboard các nước đi di chuyển của Tốt tại ô `sq` của phe `side`.
#[inline(always)]
pub fn pawn(side: usize, sq: usize) -> Bitboard {
    PAWN[side][sq]
}

/// Lấy Bitboard tia chiếu theo hướng `dir` xuất phát từ ô `sq`.
#[inline(always)]
pub fn ray(dir: usize, sq: usize) -> Bitboard {
    RAY[dir][sq]
}

/// Tính toán Bitboard các nước đi hợp lệ cho Xe tại ô `from` với Bitboard `occupied` và `own`.
#[inline(always)]
pub fn rook(from: u8, occupied: Bitboard, own: Bitboard) -> Bitboard {
    let mut target = Bitboard::empty();
    let sq = from as usize;
    let mut d = 0;
    while d < 4 {
        let r = RAY[d][sq];
        let block = r & occupied;
        if !block.active() {
            target |= r;
        } else {
            let hit = if d == 0 || d == 2 {
                block.lsb().unwrap().index()
            } else {
                block.msb().unwrap().index()
            };
            target |= r ^ RAY[d][hit];
        }
        d += 1;
    }
    target & !own
}

/// Tính toán Bitboard các nước đi và ăn quân hợp lệ cho Pháo tại ô `from`.
#[inline(always)]
pub fn cannon(from: u8, occupied: Bitboard, enemy: Bitboard) -> Bitboard {
    let mut target = Bitboard::empty();
    let sq = from as usize;
    let mut d = 0;
    while d < 4 {
        let r = RAY[d][sq];
        let block = r & occupied;
        if !block.active() {
            target |= r;
        } else {
            // Ngòi Pháo (Mount piece)
            let mount = if d == 0 || d == 2 {
                block.lsb().unwrap().index()
            } else {
                block.msb().unwrap().index()
            };
            let quiet = (r ^ RAY[d][mount]) ^ Bitboard::mask(Square(mount as u8));
            target |= quiet;

            // Tìm quân đối phương nằm sau ngòi Pháo để ăn
            let behind = RAY[d][mount] & occupied;
            if behind.active() {
                let victim = if d == 0 || d == 2 {
                    behind.lsb().unwrap().index()
                } else {
                    behind.msb().unwrap().index()
                };
                let v = Square(victim as u8);
                if enemy.test(v) {
                    target.set(v);
                }
            }
        }
        d += 1;
    }
    target
}

