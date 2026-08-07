// ============================================================================
// MODULE WORKER: THỰC THI LUỒNG LÀM VIỆC WORKER THREAD TRONG LAZY SMP
// ============================================================================
// `worker.rs` quản lý trạng thái độc lập của từng luồng làm việc Worker:
// - Tự động kích hoạt gán định tuyến luồng `Affinity` sang nhân P-Core khi bắt đầu.
// - Tích hợp đa dạng hóa `Diversity` (độ lệch số nguyên tố và History scaling).
// - Chạy độc lập thuật toán PVS không dùng khóa Mutex.
// - Cấu trúc `Worker` căn lề 64-byte (`#[repr(C, align(64))]`).
// ============================================================================

use std::sync::Arc;
use crate::board::Position;
use crate::eval::Eval;
use crate::movegen::types::Move;
use crate::search::core::Core;
use crate::search::diversity::Diversity;
use crate::search::limit::{Limits, Timer};
use crate::search::order::{History, Killer};
use crate::thread::affinity::Affinity;
use crate::tt::Table;
use super::signal::Signal;

/// Đại diện cho 1 luồng Worker (Master hoặc Helper) trong Lazy SMP
#[repr(C, align(64))]
pub struct Worker {
    /// Chỉ số nhận diện luồng worker (0 = Master, >= 1 = Helper)
    pub index: usize,
    /// Trạng thái bàn cờ riêng biệt của luồng
    pub pos: Position,
    /// Bộ đánh giá điểm số NNUE/HCE riêng biệt
    pub eval: Eval,
    /// Bảng lịch sử History Heuristics riêng biệt
    pub history: History,
    /// Bộ lưu trữ Killer Moves riêng biệt
    pub killer: Killer,
    /// Bộ đếm thời gian và quản lý tín hiệu dừng
    pub timer: Timer,
    /// Tổng số nút đã tìm kiếm trong phiên
    pub nodes: u64,
    /// Nước đi tốt nhất thu được
    pub best: Move,
    /// Điểm số thế cờ thu được
    pub score: i32,
    /// Định tuyến luồng ưu tiên P-Core (macOS QoS)
    pub affinity: Affinity,
    /// Bộ đa dạng hóa độ sâu nguyên tố và History scaling
    pub diversity: Diversity,
    /// Mảng đệm căn lề 64-byte
    pub pad: [u8; 16],
}

impl Worker {
    /// Khởi tạo Worker mới theo chỉ số luồng `index`.
    #[inline(always)]
    pub fn new(index: usize) -> Self {
        Self {
            index,
            pos: Position::empty(),
            eval: Eval::new(),
            history: History::new(),
            killer: Killer::new(),
            timer: Timer::new(),
            nodes: 0,
            best: Move::none(),
            score: 0,
            affinity: Affinity::new(index),
            diversity: Diversity::new(index),
            pad: [0u8; 16],
        }
    }

    /// Thực hiện tìm kiếm PVS trên bàn cờ với các giới hạn quy định.
    #[inline(always)]
    pub fn search(
        &mut self,
        pos: &Position,
        limits: &Limits,
        tt: &Table,
        signal: &Signal,
    ) {
        // 1. Tự động gán luồng hiện tại vào P-Cores bằng Affinity
        self.affinity.apply();

        self.pos = *pos;
        self.eval.reset(&self.pos);

        let mut depth = limits.depth;
        if depth == 0 {
            depth = 64;
        }

        if self.index > 0 {
            depth = self.diversity.depth(depth);
        }

        let mut local = *limits;
        local.depth = depth;

        self.timer.bind(Arc::clone(&signal.abort));
        self.timer.init(&local, self.pos.side);
        self.history.clear();
        self.killer.clear();
        self.nodes = 0;

        let (best, score, nodes, _depth) = Core::iterate(
            &mut self.pos,
            &mut self.eval,
            Some(tt),
            &mut self.history,
            &mut self.killer,
            &self.timer,
            Some(&self.diversity),
            None,
        );

        self.best = best;
        self.score = score;
        self.nodes = nodes;
        signal.nodes.fetch_add(nodes, std::sync::atomic::Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alignment() {
        assert_eq!(std::mem::align_of::<Worker>(), 64);
    }
}
