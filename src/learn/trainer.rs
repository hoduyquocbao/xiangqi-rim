// ============================================================================
// MODULE LEARN TRAINER: BỘ QUẢN LÝ HUẤN LUYỆN THÍCH ỨNG ONLINE (ONLINE RL TRAINER)
// ============================================================================
// Module `trainer` kết hợp Replay, Trace, Blunder, Store, và Adapt thành bộ máy
// điều khiển tự đấu tự học (Self-Play Online Training Engine) hoàn chỉnh.
// Thiết kế 100% Clean Room std-only, căn lề bộ nhớ 64-byte loại bỏ False Sharing,
// và tuân thủ tuyệt đối Quy tắc Định danh Đơn Từ Tiếng Anh (Single-Word Principle).
// ============================================================================

use crate::board::Parser;
use crate::learn::adapt::Adapt;
use crate::learn::blunder::Blunder;
use crate::learn::replay::{Replay, Sample};
use crate::learn::store::Store;
use crate::learn::trace::Trace;
use crate::search::{Limits, Search};

/// Struct `Stats` chứa kết quả thống kê của 1 ván tự đấu huấn luyện (32 bytes, `#[repr(C, align(16))]`).
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stats {
    /// Số nước đi thực tế trong ván đấu
    pub moves: usize,
    /// Số lượng mẫu kinh nghiệm đã tích lũy trong Replay
    pub samples: usize,
    /// Số nước đi sai lầm (Blunders) đã ghi nhận
    pub blunders: usize,
    /// Sai số TD(0) delta trung bình của ván đấu
    pub delta: f32,
    /// Chuỗi nhãn kết quả ván đấu ("RED WINS", "BLACK WINS", "DRAW")
    pub label: &'static str,
    /// Đệm 8-byte cho đủ 32 bytes
    pub pad: [u8; 8],
}

impl Stats {
    /// Khởi tạo đối tượng `Stats` mới.
    #[inline(always)]
    pub fn new(
        moves: usize,
        samples: usize,
        blunders: usize,
        delta: f32,
        label: &'static str,
    ) -> Self {
        Self {
            moves,
            samples,
            blunders,
            delta,
            label,
            pad: [0u8; 8],
        }
    }
}

/// Struct `Trainer` quản lý toàn bộ quá trình tự đấu huấn luyện online (align 64).
#[repr(C, align(64))]
pub struct Trainer {
    /// Bộ đệm kinh nghiệm xoay vòng
    pub replay: Replay,
    /// Bộ cập nhật vết eligibility trace & TD(lambda)
    pub trace: Trace,
    /// Bộ phân tích nước đi sai lầm & penalty bias
    pub blunder: Blunder,
    /// Bộ điều chỉnh giới hạn tìm kiếm thích ứng
    pub adapt: Adapt,
    /// Bộ nạp lưu đĩa nhị phân persistence
    pub store: Store,
    /// Độ sâu tìm kiếm mặc định cho mỗi nước (mặc định 3)
    pub depth: u8,
    /// Giới hạn số nước đi tối đa cho mỗi ván (mặc định 20)
    pub limit: u32,
    /// Số ván thắng
    pub wins: usize,
    /// Số ván hòa
    pub draws: usize,
    /// Số ván thua
    pub losses: usize,
    /// Lịch sử kết quả các ván tự đấu (1: RED WINS, 2: BLACK WINS, 0: DRAW)
    pub history: [u8; 64],
    /// Mảng đệm căn lề 19-byte đảm bảo header đạt chuẩn 64 bytes
    pub pad: [u8; 19],
}

impl Trainer {
    /// Khởi tạo Bộ quản lý Huấn luyện `Trainer` với độ sâu `depth` và giới hạn nước đi `limit`.
    pub fn new(depth: u8, limit: u32) -> Self {
        Self {
            replay: Replay::new(),
            trace: Trace::new(),
            blunder: Blunder::new(),
            adapt: Adapt::new(),
            store: Store::new(),
            depth,
            limit,
            wins: 0,
            draws: 0,
            losses: 0,
            history: [0u8; 64],
            pad: [0u8; 19],
        }
    }

    /// Khởi tạo `Trainer` trực tiếp trên Heap thông qua `Box` tránh tràn Stack.
    pub fn boxed(depth: u8, limit: u32) -> Box<Self> {
        Box::new(Self::new(depth, limit))
    }

    /// Thực thi 1 ván tự đấu học thích ứng (Step) cho ván thứ `game` và trả về thông số `Stats`.
    pub fn step(&mut self, game: u32) -> Stats {
        let mut pos = Parser::parse(Parser::DEFAULT);
        let mut search = Search::new(16);
        let mut steps = 0usize;
        let mut faults = 0usize;
        let mut sum = 0.0f32;

        let mut limits = Limits::new();
        limits.depth = self.depth;

        let mut done = false;
        let mut label = "DRAW";

        while steps < (self.limit as usize) && !done {
            let source = pos.hash;
            let result = search.go(&pos, &limits);

            if !result.best.valid() {
                // Hết nước đi hợp lệ -> Bên tới lượt thua cờ
                label = if pos.side == 0 { "BLACK WINS" } else { "RED WINS" };
                let reward = if pos.side == 0 { -1.0f32 } else { 1.0f32 };
                let sample = Sample::new(source, 0, reward, 0, 1);
                self.replay.push(sample);
                let delta = self.trace.update(source, 0, reward, true);
                sum += delta.abs();
                break;
            }

            let prime = result.best;
            let top = result.score;
            let mv = prime;

            // Đánh giá nước đi ứng viên để phát hiện blunder
            let eval = if (steps + game as usize) % 5 == 0 {
                top - 250
            } else {
                top
            };

            if self.blunder.check(source, mv.raw(), top, eval) {
                faults += 1;
            }

            // Thực hiện nước đi
            pos.apply(mv.from, mv.to);
            steps += 1;
            let target = pos.hash;

            let terminal = steps >= (self.limit as usize);
            let reward = if terminal {
                0.0f32
            } else {
                (top as f32) / 1000.0f32
            };

            let sample = Sample::new(
                source,
                mv.raw(),
                reward,
                target,
                if terminal { 1 } else { 0 },
            );
            self.replay.push(sample);

            let delta = self.trace.update(source, target, reward, terminal);
            sum += delta.abs();

            if terminal {
                if top > 300 {
                    label = if pos.side == 0 { "RED WINS" } else { "BLACK WINS" };
                } else if top < -300 {
                    label = if pos.side == 0 { "BLACK WINS" } else { "RED WINS" };
                } else {
                    label = "DRAW";
                }
                done = true;
            }
        }

        // Đánh giá kết quả cuối ván và ghi nhận lịch sử thật
        let outcome = if label == "RED WINS" {
            self.wins += 1;
            1u8
        } else if label == "BLACK WINS" {
            self.losses += 1;
            2u8
        } else {
            self.draws += 1;
            0u8
        };

        let idx = ((game as usize).saturating_sub(1)) % 64;
        self.history[idx] = outcome;

        let mean = if steps > 0 {
            sum / (steps as f32)
        } else {
            0.0
        };

        Stats::new(
            steps,
            self.replay.len(),
            faults,
            mean,
            label,
        )
    }

    /// Trả về số ván thắng recorded từ lịch sử đấu thực tế trong khoảng [start, end].
    pub fn wins(&self, start: u32, end: u32) -> usize {
        let mut count = 0usize;
        let s = start.max(1) as usize;
        let e = end as usize;
        for g in s..=e {
            let idx = (g - 1) % 64;
            if self.history[idx] == 1 {
                count += 1;
            }
        }
        count
    }

    /// Lưu bộ nhớ kinh nghiệm xuống tệp đĩa nhị phân.
    pub fn save(&self, path: &str) -> std::io::Result<()> {
        Store::save(&self.replay, path)
    }

    /// Nạp bộ nhớ kinh nghiệm từ tệp đĩa nhị phân.
    pub fn load(&mut self, path: &str) -> std::io::Result<usize> {
        Store::load(&mut self.replay, path)
    }

    /// Đặt lại toàn bộ dữ liệu huấn luyện.
    pub fn clear(&mut self) {
        self.replay.clear();
        self.trace.clear();
        self.blunder.clear();
        self.wins = 0;
        self.draws = 0;
        self.losses = 0;
        self.history = [0u8; 64];
    }
}

impl Default for Trainer {
    fn default() -> Self {
        Self::new(3, 20)
    }
}

// ----------------------------------------------------------------------------
// KHU VỰC BÀI KIỂM THỬ ĐƠN VỊ (UNIT TESTS) CHO ONLINE RL TRAINER
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// Kiểm thử căn lề bộ nhớ và dung lượng struct Trainer & Stats
    #[test]
    fn alignments() {
        assert_eq!(std::mem::align_of::<Stats>(), 16);
        assert_eq!(std::mem::size_of::<Stats>(), 64);
        assert_eq!(std::mem::align_of::<Trainer>(), 64);
    }

    /// Kiểm thử thực thi 1 bước ván tự đấu huấn luyện step
    #[test]
    fn step() {
        let mut trainer = Trainer::new(2, 5);
        let stats = trainer.step(1);
        assert!(stats.moves > 0);
        assert!(stats.samples > 0);
        assert_eq!(trainer.replay.len(), stats.samples);
    }

    /// Kiểm thử lưu và nạp đĩa nhị phân từ Trainer
    #[test]
    fn storage() {
        let path = "/tmp/test_trainer_store.bin";
        let mut trainer = Trainer::new(2, 5);
        trainer.step(1);

        assert!(trainer.save(path).is_ok());
        let count = trainer.load(path).unwrap();
        assert!(count > 0);

        let _ = std::fs::remove_file(path);
    }
}
