// ============================================================================
// MODULE ENDGAME: CƠ SỞ TRI THỨC TÀN CUỘC THỰC DỤNG VÀ THẾ CỜ LÝ THUYẾT
// ============================================================================
// Module `endgame` chịu trách nhiệm nhận diện các thế cờ tàn cuộc lý thuyết:
// 1. Đơn Mã thắng Đơn Sĩ (+15000 / -15000)
// 2. Đơn Pháo vs Sĩ/Tượng hòa (0)
// 3. Xe Mã thắng Xe Sĩ Tượng (+15000 / -15000)
// 4. Hai Pháo thắng Khuyết Sĩ Tượng (+15000 / -15000)
// 5. Đơn Xe thắng Khuyết Sĩ Tượng (+15000 / -15000)
// 6. Đơn Mã hòa Đơn Tượng (0)
// 7. Hai Mã thắng Sĩ Tượng Toàn (+15000 / -15000)
// 8. Pháo Tốt qua sông thắng Khuyết Sĩ Tượng / hòa Sĩ Tượng Toàn (+15000 / 0 / -15000)
// 9. Xe Pháo thắng Xe (+15000 / -15000)
// 10. Không còn quân công hòa (0)
// Tích hợp AtomicUsize COUNT kiểm tra Fast-Path bỏ qua RwLock.read() khi rỗng.
// ============================================================================

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;
use crate::board::Position;

/// Điểm số thắng tuyệt đối cho thế cờ tàn cuộc (+15000 centipawns)
pub const WIN: i32 = 15000;
/// Điểm số hòa cân bằng cho thế cờ tàn cuộc (0 centipawns)
pub const DRAW: i32 = 0;
/// Điểm số thua tuyệt đối cho thế cờ tàn cuộc (-15000 centipawns)
pub const LOSS: i32 = -15000;

/// Struct `Count` đếm số lượng các loại quân cờ của 2 bên.
/// Căn lề bộ nhớ 16-byte (`#[repr(C, align(16))]`), kích thước đúng 16-byte.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Count {
    /// Số lượng 7 loại quân bên ta (King, Advisor, Bishop, Knight, Rook, Cannon, Pawn)
    pub hero: [u8; 7],
    /// Số lượng 7 loại quân bên địch (King, Advisor, Bishop, Knight, Rook, Cannon, Pawn)
    pub enemy: [u8; 7],
    /// Số Tốt đã qua sông của bên ta
    pub river: u8,
    /// Mảng đệm căn lề bộ nhớ 16 bytes (7 + 7 + 1 + 1 = 16B)
    pub pad: [u8; 1],
}

impl Count {
    /// Phân tích số lượng quân cờ trên bàn cờ `pos` cho bên nắm lượt đi (`hero`) và bên đối thủ (`enemy`).
    #[inline(always)]
    pub fn parse(pos: &Position) -> Self {
        let side = pos.side as usize;
        let foe = (1 - pos.side) as usize;

        let mut hero = [0u8; 7];
        let mut enemy = [0u8; 7];

        let mut i = 0;
        while i < 7 {
            hero[i] = pos.counts[side * 7 + i];
            enemy[i] = pos.counts[foe * 7 + i];
            i += 1;
        }

        let val = river(pos, pos.side);

        Self {
            hero,
            enemy,
            river: val,
            pad: [0; 1],
        }
    }
}

/// Tính số lượng Tốt đã qua sông của một bên (`side`: 0 - Đỏ, 1 - Đen).
#[inline(always)]
pub fn river(pos: &Position, side: u8) -> u8 {
    let mut total = 0u8;
    if side == 0 {
        let mut sq = 45;
        while sq < 90 {
            if pos.grid[sq] == 6 {
                total += 1;
            }
            sq += 1;
        }
    } else {
        let mut sq = 0;
        while sq <= 44 {
            if pos.grid[sq] == 13 {
                total += 1;
            }
            sq += 1;
        }
    }
    total
}

/// Struct `Rule` định nghĩa quy tắc tàn cuộc.
/// Căn lề bộ nhớ 16-byte (`#[repr(C, align(16))]`), kích thước 32-byte.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rule {
    /// Mã số quy tắc tàn cuộc
    pub code: u16,
    /// Điểm đánh giá thế cờ (+15000: Thắng, 0: Hòa, -15000: Thua)
    pub score: i32,
    /// Tên quy tắc tàn cuộc
    pub name: &'static str,
    /// Mảng đệm căn lề bộ nhớ 32 bytes (2B + 2B compiler pad + 4B + 16B + 8B = 32B)
    pub pad: [u8; 8],
}

impl Rule {
    /// Khởi tạo một đối tượng `Rule` mới.
    #[inline(always)]
    pub const fn new(code: u16, score: i32, name: &'static str) -> Self {
        Self {
            code,
            score,
            name,
            pad: [0; 8],
        }
    }
}

/// Struct `Endgame` bọc cơ sở tri thức tàn cuộc thực dụng, căn lề bộ nhớ 64-byte (`#[repr(C, align(64))]`).
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct Endgame {
    /// Số lượng quy tắc tàn cuộc được hỗ trợ
    pub total: usize,
    /// Mảng đệm căn lề bộ nhớ đạt đúng 64 bytes (8B + 56B = 64B)
    pub pad: [u8; 56],
}

impl Default for Endgame {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// Bảng bộ nhớ đệm động chứa các thế cờ tàn cuộc được đồng bộ tự động từ bộ đệm học máy.
static DYNAMIC: RwLock<Vec<(u64, i32)>> = RwLock::new(Vec::new());

/// Biến đếm nguyên tử số lượng phần tử động hiện có, cho phép Fast-Path bỏ qua RwLock.read() khi rỗng.
static COUNT: AtomicUsize = AtomicUsize::new(0);

impl Endgame {
    /// Khởi tạo bộ đánh giá tàn cuộc `Endgame` với 10 quy tắc cố định.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            total: 10,
            pad: [0; 56],
        }
    }

    /// Đồng bộ/lưu một thế cờ tàn cuộc xuất sắc vào dynamic endgame memory table.
    /// Trả về `true` nếu đồng bộ thành công.
    pub fn sync(hash: u64, score: i32) -> bool {
        if let Ok(mut guard) = DYNAMIC.write() {
            for item in guard.iter_mut() {
                if item.0 == hash {
                    item.1 = score;
                    return true;
                }
            }
            guard.push((hash, score));
            COUNT.store(guard.len(), Ordering::Release);
            return true;
        }
        false
    }

    /// Tra cứu trực tiếp điểm đánh giá tàn cuộc dựa trên khóa băm Zobrist `hash`.
    #[inline(always)]
    pub fn probe(hash: u64) -> Option<i32> {
        if COUNT.load(Ordering::Acquire) > 0 {
            if let Ok(guard) = DYNAMIC.read() {
                for &(h, s) in guard.iter().rev() {
                    if h == hash {
                        return Some(s);
                    }
                }
            }
        }
        None
    }

    /// Xóa toàn bộ dữ liệu trong dynamic endgame memory table.
    pub fn clear() {
        if let Ok(mut guard) = DYNAMIC.write() {
            guard.clear();
            COUNT.store(0, Ordering::Release);
        }
    }

    /// Trả về số lượng thế cờ tàn cuộc hiện có trong dynamic endgame memory table.
    pub fn count() -> usize {
        COUNT.load(Ordering::Acquire)
    }

    /// Nhận diện và chấm điểm các thế cờ tàn cuộc lý thuyết thực dụng.
    /// Trả về `Some(WIN)` (+15000), `Some(DRAW)` (0), hoặc `Some(LOSS)` (-15000).
    /// Trả về `None` nếu thế cờ nằm ngoài cơ sở tri thức tàn cuộc.
    #[inline(always)]
    pub fn eval(pos: &Position) -> Option<i32> {
        // 1. Kiểm tra dynamic endgame memory table trước
        if let Some(score) = Self::probe(pos.hash) {
            return Some(score);
        }

        // 2. Phân tích các quy tắc tàn cuộc lý thuyết chuẩn
        let cnt = Count::parse(pos);

        // Tổng số quân tấn công bên ta (Mã, Xe, Pháo, Tốt)
        let attack = cnt.hero[3] + cnt.hero[4] + cnt.hero[5] + cnt.hero[6];
        // Tổng số quân tấn công bên địch (Mã, Xe, Pháo, Tốt)
        let danger = cnt.enemy[3] + cnt.enemy[4] + cnt.enemy[5] + cnt.enemy[6];

        // Số Tốt qua sông của bên địch
        let reach = river(pos, 1 - pos.side);

        // 1. Không còn quân công ở cả 2 bên -> Hòa
        if attack == 0 && danger == 0 {
            return Some(DRAW);
        }

        // 2. Đơn Mã vs Tượng / Sĩ Tượng hoặc Đơn Sĩ
        if cnt.hero[3] == 1 && attack == 1 {
            if danger == 0 {
                if cnt.enemy[2] >= 1 {
                    return Some(DRAW);
                }
                if cnt.enemy[1] <= 1 && cnt.enemy[2] == 0 {
                    return Some(WIN);
                }
            }
        }
        if cnt.enemy[3] == 1 && danger == 1 {
            if attack == 0 {
                if cnt.hero[2] >= 1 {
                    return Some(DRAW);
                }
                if cnt.hero[1] <= 1 && cnt.hero[2] == 0 {
                    return Some(LOSS);
                }
            }
        }

        // 3. Đơn Pháo vs Sĩ / Tượng bất kỳ -> Hòa
        if cnt.hero[5] == 1 && attack == 1 {
            if danger == 0 && (cnt.enemy[1] >= 1 || cnt.enemy[2] >= 1) {
                return Some(DRAW);
            }
        }
        if cnt.enemy[5] == 1 && danger == 1 {
            if attack == 0 && (cnt.hero[1] >= 1 || cnt.hero[2] >= 1) {
                return Some(DRAW);
            }
        }

        // 4. Pháo Tốt chưa qua sông vs Sĩ / Tượng -> Hòa
        if cnt.hero[5] == 1 && cnt.river == 0 && attack == cnt.hero[5] + cnt.hero[6] {
            if danger == 0 {
                return Some(DRAW);
            }
        }
        if cnt.enemy[5] == 1 && reach == 0 && danger == cnt.enemy[5] + cnt.enemy[6] {
            if attack == 0 {
                return Some(DRAW);
            }
        }

        // 5. Pháo Tốt qua sông vs Sĩ Tượng Toàn (Hòa) / Khuyết Sĩ Tượng (Thắng)
        if cnt.hero[5] == 1 && cnt.river >= 1 && attack == cnt.hero[5] + cnt.hero[6] {
            if danger == 0 {
                if cnt.enemy[1] == 2 && cnt.enemy[2] == 2 {
                    return Some(DRAW);
                }
                if cnt.enemy[1] < 2 || cnt.enemy[2] < 2 {
                    return Some(WIN);
                }
            }
        }
        if cnt.enemy[5] == 1 && reach >= 1 && danger == cnt.enemy[5] + cnt.enemy[6] {
            if attack == 0 {
                if cnt.hero[1] == 2 && cnt.hero[2] == 2 {
                    return Some(DRAW);
                }
                if cnt.hero[1] < 2 || cnt.hero[2] < 2 {
                    return Some(LOSS);
                }
            }
        }

        // 6. Xe Mã vs Xe
        if cnt.hero[4] == 1 && cnt.hero[3] == 1 && attack == 2 {
            if cnt.enemy[4] == 1 && danger == 1 {
                return Some(WIN);
            }
        }
        if cnt.enemy[4] == 1 && cnt.enemy[3] == 1 && danger == 2 {
            if cnt.hero[4] == 1 && attack == 1 {
                return Some(LOSS);
            }
        }

        // 7. Hai Pháo vs Khuyết Sĩ Tượng
        if cnt.hero[5] == 2 && attack == 2 {
            if danger == 0 && (cnt.enemy[1] < 2 || cnt.enemy[2] < 2) {
                return Some(WIN);
            }
        }
        if cnt.enemy[5] == 2 && danger == 2 {
            if attack == 0 && (cnt.hero[1] < 2 || cnt.hero[2] < 2) {
                return Some(LOSS);
            }
        }

        // 8. Đơn Xe vs Khuyết Sĩ Tượng
        if cnt.hero[4] == 1 && attack == 1 {
            if danger == 0 && (cnt.enemy[1] < 2 || cnt.enemy[2] < 2) {
                return Some(WIN);
            }
        }
        if cnt.enemy[4] == 1 && danger == 1 {
            if attack == 0 && (cnt.hero[1] < 2 || cnt.hero[2] < 2) {
                return Some(LOSS);
            }
        }

        // 9. Hai Mã vs Sĩ Tượng Toàn
        if cnt.hero[3] == 2 && attack == 2 {
            if danger == 0 {
                return Some(WIN);
            }
        }
        if cnt.enemy[3] == 2 && danger == 2 {
            if attack == 0 {
                return Some(LOSS);
            }
        }

        // 10. Xe Pháo vs Xe
        if cnt.hero[4] == 1 && cnt.hero[5] == 1 && attack == 2 {
            if cnt.enemy[4] == 1 && danger == 1 {
                return Some(WIN);
            }
        }
        if cnt.enemy[4] == 1 && cnt.enemy[5] == 1 && danger == 2 {
            if cnt.hero[4] == 1 && attack == 1 {
                return Some(LOSS);
            }
        }

        None
    }
}

// ----------------------------------------------------------------------------
// KHU VỰC BÀI KIỂM THỬ ĐƠN VỊ (UNIT TESTS) CHO MODULE ENDGAME
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Parser;

    /// Kiểm thử căn lề bộ nhớ SIMD `align(16)` và `align(64)` cho các struct thuộc Endgame Module.
    #[test]
    fn alignments() {
        assert_eq!(std::mem::align_of::<Count>(), 16);
        assert_eq!(std::mem::size_of::<Count>(), 16);
        assert_eq!(std::mem::align_of::<Rule>(), 16);
        assert_eq!(std::mem::size_of::<Rule>(), 32);
        assert_eq!(std::mem::align_of::<Endgame>(), 64);
        assert_eq!(std::mem::size_of::<Endgame>(), 64);
    }

    /// Kiểm thử thế cờ không còn quân công hòa (Draw).
    #[test]
    fn bare() {
        let pos = Parser::parse("4k4/9/9/9/9/9/9/9/9/4K4 w - - 0 1");
        let score = Endgame::eval(&pos);
        assert_eq!(score, Some(DRAW));
    }

    /// Kiểm thử Đơn Mã thắng Đơn Sĩ (Win).
    #[test]
    fn knight() {
        let pos = Parser::parse("4ka3/9/9/9/9/9/9/4N4/9/4K4 w - - 0 1");
        let score = Endgame::eval(&pos);
        assert_eq!(score, Some(WIN));
    }

    /// Kiểm thử Xe Pháo thắng Xe (Win / Loss).
    #[test]
    fn rook() {
        let pos = Parser::parse("3k5/4r4/9/9/9/9/9/9/4C4/4K1R2 w - - 0 1");
        let score = Endgame::eval(&pos);
        assert_eq!(score, Some(WIN));

        let mut black = pos;
        black.side = 1;
        let enemy = Endgame::eval(&black);
        assert_eq!(enemy, Some(LOSS));
    }

    /// Kiểm thử tính năng đồng bộ động sync, probe, count, clear trong Endgame.
    #[test]
    fn dynamic() {
        Endgame::clear();
        assert_eq!(Endgame::count(), 0);

        let hash_val = 0x9988776655443322u64;
        let score = WIN;

        let synced = Endgame::sync(hash_val, score);
        assert!(synced);
        assert_eq!(Endgame::count(), 1);

        assert_eq!(Endgame::probe(hash_val), Some(score));

        let mut pos = Position::empty();
        pos.hash = hash_val;
        assert_eq!(Endgame::eval(&pos), Some(score));

        Endgame::clear();
        assert_eq!(Endgame::count(), 0);
    }
}
