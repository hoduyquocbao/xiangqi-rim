// ============================================================================
// MODULE SEARCH: LÕI THUẬT TOÁN TÌM KIẾM CÂY NƯỚC ĐỊ PVS & ĐÁNH GIÁ ĐỘ SÂU (SEARCH ENGINE)
// ============================================================================
// Module `search` là bộ não tính toán chính của Engine Cờ Tướng:
// - `core`: Thuật toán PVS (Principal Variation Search) kết hợp Aspiration Window và Iterative Deepening.
// - `quiesce`: Quiescence Search (tìm kiếm yên tĩnh) loại bỏ hiện tượng Horizon Effect (hiệu ứng chân trời).
// - `order`: Bộ sắp xếp nước đi ưu tiên (MVV-LVA, Killer Moves, History Heuristics Table).
// - `prune`: Các kỹ thuật tỉa nhánh (Null Move Pruning, Futility Pruning, LMR - Late Move Reduction).
// - `limit`: Bộ quản lý thời gian (Time Manager) phản hồi ngắt lệnh < 1ms.
// - `stack`: Mảng Stack lưu vết dữ liệu qua các độ sâu tìm kiếm đệ quy (align 64).
// ============================================================================

/// Module con `core` thuật toán PVS và Iterative Deepening
pub mod core;
/// Module con `diversity` quản lý đa dạng hóa tìm kiếm nguyên tố và hệ số lịch sử
pub mod diversity;
/// Module con `hybrid` động cơ tìm kiếm kết hợp GPU+CPU tối ưu hóa tải
pub mod hybrid;
/// Module con `limit` quản lý thời gian và tín hiệu Abort
pub mod limit;
/// Module con `order` sắp xếp thứ tự ưu tiên nước đi
pub mod order;
/// Module con `prune` tỉa nhánh thuật toán
pub mod prune;
/// Module con `pruning` cắt tỉa LMR, RFP và Q-Search với Stand-Pat
pub mod pruning;
/// Module con `pv` lưu vết tuyến nước đi tốt nhất Principal Variation
pub mod pv;
/// Module con `quiesce` tìm kiếm ăn quân Quiescence Search
pub mod quiesce;
/// Module con `see` tính toán tĩnh chuỗi đổi quân Static Exchange Evaluation
pub mod see;
/// Module con `smp` động cơ tìm kiếm song song đa luồng Lazy SMP
pub mod smp;
/// Module con `stack` bộ nhớ đệm Stack đệ quy
pub mod stack;

pub use core::Core;
pub use diversity::{Diversity, PRIMES};
pub use hybrid::HybridEngine;
pub use limit::{Limits, Result, Timer};
pub use order::{History, Killer, Order, Picker, Stage, VALUES};
pub use pruning::Pruner;
pub use prune::Prune;
pub use pv::Pv;
pub use quiesce::Quiesce;
pub use see::See;
pub use smp::LazySmp;
pub use stack::Stack;


use std::sync::atomic::Ordering;
use crate::board::Position;
use crate::eval::Eval;
use crate::tt::Table;

/// Struct `Search` quản lý toàn bộ phiên tìm kiếm, căn lề bộ nhớ 64-byte (`#[repr(C, align(64))]`).
#[repr(C, align(64))]
pub struct Search {
    /// Bàn cờ đang được tìm kiếm
    pub pos: Position,
    /// Bộ đánh giá thế cờ NNUE + HCE
    pub eval: Eval,
    /// Bảng băm Transposition Table toàn cục
    pub tt: Table,
    /// Bảng lịch sử History Heuristics
    pub history: History,
    /// Bộ lưu trữ Killer Moves
    pub killer: Killer,
    /// Bộ đếm thời gian và quản lý tín hiệu dừng
    pub timer: Timer,
    /// Kết quả tìm kiếm (Best move, Score, Nodes, Time)
    pub result: Result,
    /// Mảng lưu vết Zobrist Hashes của toàn bộ các nước đi đã đấu trong ván cờ
    pub past_hashes: Vec<u64>,
}

impl Search {
    /// Khởi tạo một Engine Tìm kiếm mới với dung lượng bộ nhớ băm `mb` Megabytes.
    pub fn new(mb: usize) -> Self {
        Self {
            pos: Position::empty(),
            eval: Eval::new(),
            tt: Table::new(mb),
            history: History::new(),
            killer: Killer::new(),
            timer: Timer::new(),
            result: Result::new(),
            past_hashes: Vec::with_capacity(256),
        }
    }

    /// Khởi tạo một Engine Tìm kiếm mới trực tiếp trên Heap (Box) với 0-byte stack overhead.
    pub fn new_boxed(mb: usize) -> Box<Self> {
        unsafe {
            let layout = std::alloc::Layout::new::<Self>();
            let ptr = std::alloc::alloc_zeroed(layout) as *mut Self;
            std::ptr::write(&mut (*ptr).pos, Position::empty());
            std::ptr::write(&mut (*ptr).eval, Eval::new());
            std::ptr::write(&mut (*ptr).tt, Table::new(mb));
            std::ptr::write(&mut (*ptr).history, History::new());
            std::ptr::write(&mut (*ptr).killer, Killer::new());
            std::ptr::write(&mut (*ptr).timer, Timer::new());
            std::ptr::write(&mut (*ptr).result, Result::new());
            std::ptr::write(&mut (*ptr).past_hashes, Vec::with_capacity(256));
            Box::from_raw(ptr)
        }
    }

    /// Đưa Zobrist Hash của nước cờ vừa đấu vào mảng past_hashes lịch sử toàn ván.
    pub fn push_history(&mut self, hash: u64) {
        if !self.past_hashes.contains(&hash) {
            self.past_hashes.push(hash);
        }
    }

    /// Nạp trọng số NNUE từ tệp nhị phân XRNN.
    pub fn load_nnue(&mut self, path: &str) -> std::result::Result<(), String> {
        self.eval.load(path)
    }

    /// Tự động kiểm tra và nạp trọng số NNUE nếu có tệp nhị phân trong thư mục data/.
    pub fn auto_load(&mut self) -> bool {
        let candidates = [
            "data/nnue_weights_gpu.bin",
            "data/nnue_weights.bin",
        ];
        for path in &candidates {
            if std::path::Path::new(path).exists() {
                if self.eval.load(path).is_ok() {
                    return true;
                }
            }
        }
        false
    }

    /// Đặt lại toàn bộ dữ liệu bàn cờ, lịch sử, killer table và bảng băm.
    pub fn clear(&mut self) {
        self.pos.clear();
        self.history.clear();
        self.killer.clear();
        self.tt.clear();
        self.past_hashes.clear();
        self.result = Result::new();
    }

    /// Tra cứu nước đi khai cuộc từ Opening Book dựa trên vị trí bàn cờ `pos` (0ms CPU).
    #[inline(always)]
    pub fn probe(pos: &Position) -> Option<crate::movegen::Move> {
        crate::book::Book::probe(pos)
    }

    /// Thực thi lệnh tìm kiếm `go` từ vị trí bàn cờ `pos` với các giới hạn `limits`.
    pub fn go(&mut self, pos: &Position, limits: &Limits) -> Result {
        if !self.past_hashes.contains(&pos.hash) {
            self.past_hashes.push(pos.hash);
        }
        let past_slice = self.past_hashes.clone();
        self.go_with_history(pos, limits, &past_slice)
    }

    /// Thực thi lệnh tìm kiếm `go_with_history` tích hợp mảng past hashes lịch sử ván cờ ngăn lặp cờ toàn ván.
    pub fn go_with_history(&mut self, pos: &Position, limits: &Limits, past: &[u64]) -> Result {
        self.pos = *pos;
        self.eval.reset(&self.pos);
        self.timer.init(limits, self.pos.side);
        self.result = Result::new();

        // 0. Tra cứu nhanh Opening Book 0ms khai cuộc mà không tốn CPU search
        if let Some(mv) = Self::probe(pos) {
            self.result.best = mv;
            self.result.score = 0;
            self.result.nodes = 1;
            self.result.depth = 1;
            self.result.time = 0;
            return self.result.clone();
        }

        // 1. Chạy vòng lặp độ sâu lặp tăng dần (Iterative Deepening Search Loop)
        let (best, score, nodes, completed_depth) = Core::iterate(
            &mut self.pos,
            &mut self.eval,
            Some(&self.tt),
            &mut self.history,
            &mut self.killer,
            &self.timer,
            None,
            if past.is_empty() { None } else { Some(past) },
        );

        // 2. Ghi nhận kết quả nước đi tốt nhất thu được (Bổ sung fallback nếu best không hợp lệ)
        let mut final_best = best;
        if !final_best.valid() {
            let mut moves = crate::movegen::types::List::new();
            crate::movegen::gen(&mut self.pos, &mut moves);
            if moves.count > 0 {
                final_best = moves.items[0];
            }
        }

        self.result.best = final_best;
        self.result.score = score;
        self.result.nodes = nodes;
        self.result.depth = completed_depth;
        self.result.time = self.timer.start.elapsed().as_millis() as u64;
        self.timer.abort.store(false, Ordering::Relaxed);

        self.result.clone()
    }

    /// Phát tín hiệu ngắt dừng ngay lập tức phiên tìm kiếm hiện tại (`stop`).
    pub fn halt(&self) {
        self.timer.halt();
    }
}

// ----------------------------------------------------------------------------
// KHU VỰC BÀI KIỂM THỬ ĐƠN VỊ (UNIT TESTS) CHO MODULE SEARCH ENGINE
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Parser;

    /// Kiểm thử căn lề bộ nhớ SIMD 64-byte cho tất cả các struct thuộc Search Module.
    #[test]
    fn alignments() {
        assert_eq!(std::mem::align_of::<Search>(), 64);
        assert_eq!(std::mem::align_of::<Diversity>(), 64);
        assert_eq!(std::mem::align_of::<Stack>(), 64);
        assert_eq!(std::mem::align_of::<Limits>(), 64);
        assert_eq!(std::mem::align_of::<Timer>(), 64);
        assert_eq!(std::mem::align_of::<Result>(), 64);
        assert_eq!(std::mem::align_of::<Pv>(), 64);
        assert_eq!(std::mem::align_of::<History>(), 64);
        assert_eq!(std::mem::align_of::<Killer>(), 64);
        assert_eq!(std::mem::align_of::<Picker>(), 64);
    }

    /// Kiểm thử phiên tìm kiếm cơ bản trên vị trí khởi đầu ở độ sâu 4.
    #[test]
    fn initial() {
        let pos = Parser::parse(Parser::DEFAULT);
        let mut search = Search::new(16);
        let mut limits = Limits::new();
        limits.depth = 4;

        let result = search.go(&pos, &limits);
        assert!(result.best.valid(), "Search BẮT BUỘC trả về nước đi hợp lệ!");
        assert!(result.nodes > 0, "Search BẮT BUỘC đã duyệt > 0 nút!");
    }

    /// Kiểm thử tìm kiếm ăn quân Quiescence Search không bị nổ score hay văng lỗi.
    #[test]
    fn quiesce() {
        let mut pos = Parser::parse(Parser::DEFAULT);
        let mut eval = Eval::new();
        eval.reset(&pos);
        let timer = Timer::new();
        let mut nodes = 0u64;

        let score = Quiesce::search(&mut pos, &mut eval, &timer, -30000, 30000, 0, &mut nodes);
        assert!(score.abs() < 2000, "Điểm Quiesce BẮT BUỘC nằm trong ranh giới thực tế!");
    }
}

