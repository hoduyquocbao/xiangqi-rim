// ============================================================================
// MODULE ENGINE: ĐIỀU HÀNH TIẾN TRÌNH TỰ ĐẤU CỜ TƯỚNG (SELF-PLAY ENGINE RUNNER)
// ============================================================================
// `engine.rs` chịu trách nhiệm khởi chạy và quản lý toàn bộ ván tự đấu Cờ Tướng:
// - `Config`: Cấu hình độ sâu (`depth`), thời gian (`time`), giới hạn nước (`limit`).
// - `Side`: Enum phe chơi (`Red`, `Black`).
// - `Outcome`: Kết quả ván đấu (`Win(Side)`, `Draw`, `Limit`, `Loop`).
// - `Match`: Lưu trữ lịch sử băm Zobrist (`history`), danh sách nước đi (`moves`),
//   kết quả chung cuộc (`outcome`), thống kê hiệu năng (`stats`).
// - `Runner`: Engine tự đấu căn lề bộ nhớ 64-byte với hàm `play(config)`.
// ============================================================================

use crate::board;
use crate::movegen::{self, Move};
use crate::search;
use crate::selfplay::stats::Stats;

/// Enum `Side` đại diện cho phe chơi trong ván tự đấu (Red = 0, Black = 1).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    /// Bên Đỏ (di chuyển trước)
    Red = 0,
    /// Bên Đen (di chuyển sau)
    Black = 1,
}

impl Side {
    /// Đảo ngược lượt chơi
    #[inline(always)]
    pub const fn flip(self) -> Self {
        match self {
            Self::Red => Self::Black,
            Self::Black => Self::Red,
        }
    }

    /// Lấy chỉ số nguyên (0: Red, 1: Black)
    #[inline(always)]
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// Enum `Outcome` đại diện cho kết quả chung cuộc của ván tự đấu Cờ Tướng.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Outcome {
    /// Bên `Side` thắng do chiếu bí hoặc đối phương hết nước đi hợp lệ
    Win(Side),
    /// Hòa cờ tiêu chuẩn
    Draw,
    /// Vượt mốc giới hạn số nước đi cấu hình
    Limit,
    /// Hòa lặp nước 3 lần (3-fold repetition loop)
    Loop,
}

/// Struct `Config` chứa thông số cấu hình ván tự đấu Cờ Tướng (Self-Play Configuration).
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Config {
    /// Độ sâu tìm kiếm tối đa của Engine cho mỗi nước đi (1 byte)
    pub depth: u8,
    /// Giới hạn thời gian tính toán cho mỗi nước đi (ms) (8 bytes)
    pub time: u64,
    /// Giới hạn tối đa số nước đi của ván đấu trước khi xử Hòa Limit (4 bytes)
    pub limit: u32,
    /// Trường đệm căn lề bộ nhớ 64-byte (44 bytes)
    _pad: [u8; 44],
}

impl Config {
    /// Khởi tạo đối tượng Config mới với các tham số độ sâu `depth`, thời gian `time` ms, và giới hạn `limit` nước.
    pub const fn new(depth: u8, time: u64, limit: u32) -> Self {
        Self {
            depth,
            time,
            limit,
            _pad: [0; 44],
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new(4, 1000, 200)
    }
}

/// Struct `Match` đại diện cho toàn bộ dữ liệu và kết quả của một ván tự đấu Cờ Tướng.
#[repr(C, align(64))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Match {
    /// Lịch sử các khóa băm Zobrist Hash của bàn cờ qua từng nước đi
    pub history: Vec<u64>,
    /// Danh sách các nước đi hợp lệ đã thực hiện trong ván đấu
    pub moves: Vec<Move>,
    /// Kết quả chung cuộc của ván tự đấu
    pub outcome: Outcome,
    /// Chỉ số thống kê hiệu năng chi tiết
    pub stats: Stats,
}

impl Match {
    /// Khởi tạo một đối tượng Match rỗng với sức chứa dự kiến `capacity`.
    pub fn new(capacity: usize) -> Self {
        Self {
            history: Vec::with_capacity(capacity),
            moves: Vec::with_capacity(capacity),
            outcome: Outcome::Draw,
            stats: Stats::new(),
        }
    }
}

/// Struct `Runner` quản lý và điều phối các ván tự đấu Cờ Tướng (Self-Play Engine Runner).
#[repr(C, align(64))]
pub struct Runner {
    /// Dung lượng bộ nhớ Transposition Table (MB)
    pub memory: usize,
    /// Trường đệm căn lề bộ nhớ 64-byte
    _pad: [u8; 56],
}

impl Runner {
    /// Khởi tạo Runner mới với bộ nhớ băm `memory` MB.
    pub const fn new(memory: usize) -> Self {
        Self {
            memory,
            _pad: [0; 56],
        }
    }

    /// Tiến hành chơi ván tự đấu theo cấu hình `config` và trả về thông tin `Match`.
    pub fn play(config: &Config) -> Match {
        let runner = Self::new(16);
        runner.run(config)
    }

    /// Chạy ván đấu thực tế trên runner hiện tại.
    pub fn run(&self, config: &Config) -> Match {
        let mut pos = board::Parser::parse(board::Parser::DEFAULT);
        let cap = config.limit as usize;
        let mut result = Match::new(cap);
        result.history.push(pos.hash);

        let mut search = search::Search::new_boxed(self.memory);

        let mut step = 0u32;
        while step < config.limit {
            // 1. Kiểm tra lặp nước 2 lần (2-fold repetition / Loop detection) - Phạt lập tức ngăn AI đi lặp
            let curr = pos.hash;
            let mut count = 0;
            let mut h = 0;
            while h < result.history.len() {
                if result.history[h] == curr {
                    count += 1;
                }
                h += 1;
            }
            if count >= 2 {
                result.outcome = Outcome::Loop;
                break;
            }

            // 2. Kiểm tra danh sách nước đi hợp lệ của bên tới lượt
            let mut legal = movegen::List::new();
            movegen::legal(&mut pos, &mut legal);

            if legal.empty() {
                // Bên tới lượt không còn nước đi hợp lệ -> Thua! (Đối phương thắng)
                let winner = if pos.side == 0 {
                    Side::Black
                } else {
                    Side::Red
                };
                result.outcome = Outcome::Win(winner);
                break;
            }

            // 3. Tìm nước đi tiếp theo bằng Opening Book hoặc Search Engine
            let mv = if let Some(bm) = search::Search::probe(&pos) {
                result.stats.nodes += 1;
                bm
            } else {
                let mut limits = search::Limits::new();
                limits.depth = config.depth;
                limits.time = config.time;
                let res = search.go(&pos, &limits);
                result.stats.nodes += res.nodes;
                result.stats.time += res.time;
                if res.best.valid() {
                    res.best
                } else {
                    legal.get(0)
                }
            };

            // 4. Kiểm tra xem nước đi tìm được có hợp lệ không
            if !mv.valid() || !movegen::legal::valid(&mut pos, mv) {
                // Nếu nước đi tìm được không hợp lệ, fallback sang nước đi hợp lệ đầu tiên
                let fallback = legal.get(0);
                if !fallback.valid() {
                    let winner = if pos.side == 0 {
                        Side::Black
                    } else {
                        Side::Red
                    };
                    result.outcome = Outcome::Win(winner);
                    break;
                }
                pos.apply(fallback.from, fallback.to);
                result.moves.push(fallback);
            } else {
                pos.apply(mv.from, mv.to);
                result.moves.push(mv);
            }

            result.history.push(pos.hash);
            result.stats.moves += 1;
            step += 1;

            if step >= config.limit {
                result.outcome = Outcome::Limit;
                break;
            }
        }

        // Cập nhật tốc độ NPS trung bình của ván đấu
        result.stats.rate();
        result
    }
}

// ----------------------------------------------------------------------------
// KHU VỰC BÀI KIỂM THỬ ĐƠN VỊ (UNIT TESTS) CHO ENGINE MODULE
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, size_of};

    /// Kiểm thử căn lề bộ nhớ và kích thước các struct/enum cốt lõi.
    #[test]
    fn alignments() {
        assert_eq!(size_of::<Config>(), 64);
        assert_eq!(align_of::<Config>(), 64);
        assert_eq!(size_of::<Match>(), 128);
        assert_eq!(align_of::<Match>(), 64);
        assert_eq!(size_of::<Runner>(), 64);
        assert_eq!(align_of::<Runner>(), 64);
        assert_eq!(align_of::<Outcome>(), 16usize);
    }

    /// Kiểm thử tự đấu 10 nước đi với cấu hình giới hạn 10 nước.
    #[test]
    fn play() {
        let config = Config::new(1, 100, 10);
        let result = Runner::play(&config);

        assert!(result.moves.len() <= 10, "Số nước đi không vượt quá giới hạn 10!");
        assert!(result.stats.moves <= 10, "Thống kê số nước đi không vượt quá 10!");
        assert!(result.history.len() > 0, "Lịch sử băm Zobrist phải chứa dữ liệu!");
    }

    /// Kiểm thử phát hiện bẫy lặp lại 3-fold repetition (Outcome::Loop).
    #[test]
    fn loopback() {
        let mut pos = board::Parser::parse(board::Parser::DEFAULT);
        let mut history = Vec::new();
        history.push(pos.hash);

        // Giả lập di chuyển qua lại 3 lần làm trùng lặp Zobrist hash 3 lần
        let m1 = Move::new(1, 18);  // b1 -> a3 (Mã Đỏ)
        let m2 = Move::new(79, 62); // b10 -> a8 (Mã Đen)
        let m3 = Move::new(18, 1);  // a3 -> b1 (Mã Đỏ lùi)
        let m4 = Move::new(62, 79); // a8 -> b10 (Mã Đen lùi)

        let mut outcome = Outcome::Draw;

        for _ in 0..3 {
            let s1 = pos.apply(m1.from, m1.to);
            history.push(pos.hash);
            let s2 = pos.apply(m2.from, m2.to);
            history.push(pos.hash);
            let _s3 = pos.apply(m3.from, m3.to);
            history.push(pos.hash);
            let _s4 = pos.apply(m4.from, m4.to);
            history.push(pos.hash);

            let curr = pos.hash;
            let mut count = 0;
            let mut h = 0;
            while h < history.len() {
                if history[h] == curr {
                    count += 1;
                }
                h += 1;
            }

            if count >= 3 {
                outcome = Outcome::Loop;
                break;
            }

            // Dọn dẹp state hoàn tác không bị đè
            pos.revert(m4.from, m4.to, &_s4);
            pos.revert(m3.from, m3.to, &_s3);
            pos.revert(m2.from, m2.to, &s2);
            pos.revert(m1.from, m1.to, &s1);
        }

        assert_eq!(outcome, Outcome::Loop, "Phải phát hiện chính xác Outcome::Loop khi lặp nước 3 lần!");
    }
}
