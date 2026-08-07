// Module quản lý ThreadPool Zero-Lock Lazy SMP cho đa luồng tìm kiếm.

use std::sync::Arc;
use std::thread;
use crate::board::Position;
use crate::search::limit::{Limits, Result};
use crate::tt::Table;
use super::signal::Signal;
use super::worker::Worker;

/// ThreadPool quản lý các luồng Lazy SMP
#[repr(C, align(64))]
#[derive(Clone)]
pub struct Pool {
    pub size: usize,
    pub tt: Arc<Table>,
    pub signal: Arc<Signal>,
    pub pad: [u8; 32],
}

impl Pool {
    /// Khởi tạo ThreadPool với số lượng luồng size và dung lượng TT mb (MB)
    pub fn new(size: usize, mb: usize) -> Self {
        let count = size.max(1);
        let tt = Arc::new(Table::new(mb));
        let signal = Arc::new(Signal::new());

        Self {
            size: count,
            tt,
            signal,
            pad: [0u8; 32],
        }
    }

    /// Thực thi quá trình tìm kiếm song song Lazy SMP trên tất cả các luồng worker
    pub fn go(&self, pos: &Position, limits: &Limits) -> Result {
        let start = std::time::Instant::now();

        let mut handles = Vec::with_capacity(self.size.saturating_sub(1));

        if self.size > 1 {
            for index in 1..self.size {
                let board = *pos;
                let bound = *limits;
                let table = Arc::clone(&self.tt);
                let sig = Arc::clone(&self.signal);

                let handle = thread::spawn(move || {
                    let mut worker = Worker::new(index);
                    worker.search(&board, &bound, &table, &sig);
                });
                handles.push(handle);
            }
        }

        let mut master = Worker::new(0);
        master.search(pos, limits, &self.tt, &self.signal);

        self.signal.halt();

        for handle in handles {
            let _ = handle.join();
        }

        let mut result = Result::new();
        result.best = master.best;
        result.score = master.score;
        result.nodes = self
            .signal
            .nodes
            .load(std::sync::atomic::Ordering::Relaxed)
            .max(master.nodes);
        result.depth = limits.depth;
        result.time = start.elapsed().as_millis() as u64;

        result
    }

    /// Phát lệnh ngắt dừng khẩn cấp cho tất cả các luồng trong pool
    pub fn halt(&self) {
        self.signal.halt();
    }

    /// Đặt lại trạng thái tín hiệu cho lượt tìm kiếm mới
    pub fn reset(&self) {
        self.signal.reset();
    }

    /// Làm sạch Transposition Table và đặt lại tín hiệu
    pub fn clear(&self) {
        self.tt.clear();
        self.signal.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Parser;

    #[test]
    fn alignment() {
        assert_eq!(std::mem::align_of::<Pool>(), 64);
    }

    #[test]
    fn execution() {
        let pos = Parser::parse(Parser::DEFAULT);
        let pool = Pool::new(2, 16);
        let mut limits = Limits::new();
        limits.depth = 4;

        let result = pool.go(&pos, &limits);
        assert!(result.best.valid(), "Pool MUST return valid move!");
        assert!(result.nodes > 0, "Pool MUST search > 0 nodes!");
    }

    #[test]
    fn halt() {
        let pos = Parser::parse(Parser::DEFAULT);
        let pool = Pool::new(2, 16);
        let mut limits = Limits::new();
        limits.depth = 64;

        let sig = pool.clone();
        let thread = std::thread::spawn(move || {
            sig.go(&pos, &limits)
        });

        std::thread::sleep(std::time::Duration::from_millis(50));
        let start = std::time::Instant::now();
        pool.halt();
        let _ = thread.join();
        let elapsed = start.elapsed().as_millis();

        assert!(elapsed < 500, "Pool halt MUST stop search in < 500ms, took {}ms", elapsed);
    }
}
