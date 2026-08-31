// Module quản lý ThreadPool Zero-Lock Lazy SMP cho đa luồng tìm kiếm.

use std::sync::{Arc, Mutex};
use std::thread;
use crate::board::Position;
use crate::learn::Harvest;
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
    pub eval: crate::eval::Eval,
    pub harvest: Option<Arc<Mutex<Harvest>>>,
    pub pad: [u8; 8],
}

impl Pool {
    /// Khởi tạo ThreadPool với số lượng luồng size và dung lượng TT mb (MB)
    pub fn new(size: usize, mb: usize) -> Self {
        let count = size.max(1);
        let tt = Arc::new(Table::new(mb));
        let signal = Arc::new(Signal::new());
        let mut eval = crate::eval::Eval::new();
        if std::path::Path::new("data/nnue_weights.bin").exists() {
            let _ = eval.load("data/nnue_weights.bin");
        }

        let auto_harvest = if cfg!(test) {
            std::env::var("HARVEST").map(|v| v == "1").unwrap_or(false)
        } else {
            std::env::var("HARVEST").map(|v| v != "0").unwrap_or(true)
        };
        let harvest = if auto_harvest {
            Some(Arc::new(Mutex::new(Harvest::default())))
        } else {
            None
        };

        Self {
            size: count,
            tt,
            signal,
            eval,
            harvest,
            pad: [0u8; 8],
        }
    }

    /// Thực thi quá trình tìm kiếm song song Lazy SMP trên tất cả các luồng worker
    #[inline(always)]
    pub fn go(&self, pos: &Position, limits: &Limits) -> Result {
        self.trace(pos, limits, &[])
    }

    /// Thực thi quá trình tìm kiếm song song Lazy SMP tích hợp mảng past hashes chống lặp cờ và chiếu dai
    pub fn trace(&self, pos: &Position, limits: &Limits, past: &[u64]) -> Result {
        let start = std::time::Instant::now();

        // 0. Pre-Search Vault O(1) Fast-Path: Tra cứu Kho Tri Thức Vĩnh Cửu trước khi duyệt
        if limits.depth >= 12 {
            if let Some((v_move, v_score, v_depth)) = crate::system::Vault::global().probe(pos, limits.depth) {
                if v_move.valid() {
                    let mut result = Result::new();
                    result.best = v_move;
                    result.score = v_score;
                    result.nodes = 1;
                    result.depth = v_depth;
                    result.time = start.elapsed().as_millis() as u64;
                    return result;
                }
            }
        }

        self.signal.reset();

        let mut handles = Vec::with_capacity(self.size.saturating_sub(1));

        if self.size > 1 {
            for index in 1..self.size {
                let board = *pos;
                let bound = *limits;
                let table = Arc::clone(&self.tt);
                let sig = Arc::clone(&self.signal);
                let eval_clone = self.eval.clone();
                let history = past.to_vec();

                if let Ok(handle) = thread::Builder::new()
                    .name(format!("worker-{}", index))
                    .stack_size(16 * 1024 * 1024)
                    .spawn(move || {
                        let mut worker = Worker::new_boxed(index);
                        worker.eval = eval_clone;
                        worker.search(&board, &bound, &table, &sig, Some(&history));
                    })
                {
                    handles.push(handle);
                }
            }
        }

        let board = *pos;
        let bound = *limits;
        let table = Arc::clone(&self.tt);
        let sig = Arc::clone(&self.signal);
        let eval_clone = self.eval.clone();
        let history = past.to_vec();

        let master_handle = thread::Builder::new()
            .name("master-worker".to_string())
            .stack_size(16 * 1024 * 1024)
            .spawn(move || {
                let mut master = Worker::new_boxed(0);
                master.eval = eval_clone;
                master.search(&board, &bound, &table, &sig, Some(&history));
                master
            });

        let master = if let Ok(handle) = master_handle {
            handle.join().unwrap_or_else(|_| {
                let mut w = Worker::new_boxed(0);
                w.eval = self.eval.clone();
                w
            })
        } else {
            let mut m = Worker::new_boxed(0);
            m.eval = self.eval.clone();
            m.search(pos, limits, &self.tt, &self.signal, Some(past));
            m
        };

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
        result.depth = if master.depth > 0 { master.depth } else { limits.depth };
        result.time = start.elapsed().as_millis() as u64;

        // Tự động bảo tồn thế cờ Depth cao (>= 12) vào Kho Tri Thức Vĩnh Cửu (Perpetual Vault)
        if result.depth >= 12 && result.best.valid() {
            crate::system::Vault::global().save(pos, result.depth, result.best, result.score, 0);
        }

        // Tự động thu hoạch và bảo tồn thế cờ tính toán vào kho tri thức vĩnh cửu
        if let Some(ref h_arc) = self.harvest {
            if result.best.valid() {
                if let Ok(mut h) = h_arc.lock() {
                    let ply = past.len();
                    h.push(pos, result.best, result.score, result.depth, "RIM", ply);
                }
            }
        }

        result
    }

    /// Xả toàn bộ tri thức trong bộ đệm xuống tệp nhị phân bitwise và Shards NVMe
    pub fn flush(&self, outcome: &str) -> usize {
        if let Some(ref h_arc) = self.harvest {
            if let Ok(mut h) = h_arc.lock() {
                return h.flush(outcome);
            }
        }
        0
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

impl Drop for Pool {
    /// Tự động xả toàn bộ tri thức còn lại khi Pool kết thúc vòng đời
    fn drop(&mut self) {
        if let Some(ref h_arc) = self.harvest {
            if Arc::strong_count(h_arc) == 1 {
                if let Ok(mut h) = h_arc.lock() {
                    let _ = h.flush("AutoSave");
                }
            }
        }
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
        limits.depth = 12;

        let sig = pool.clone();
        let thread = std::thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(move || {
                sig.go(&pos, &limits)
            })
            .unwrap();

        std::thread::sleep(std::time::Duration::from_millis(50));
        let start = std::time::Instant::now();
        pool.halt();
        let _ = thread.join();
        let elapsed = start.elapsed().as_millis();

        let limit = if cfg!(debug_assertions) { 5000 } else { 1500 };
        assert!(elapsed < limit, "Pool halt MUST stop search in < {}ms, took {}ms", limit, elapsed);
    }
}
