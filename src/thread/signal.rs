// Module định nghĩa tín hiệu chia sẻ trạng thái giữa các luồng trong ThreadPool.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, AtomicUsize, Ordering};

/// Trạng thái hoạt động của ThreadPool
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Idle = 0,
    Run = 1,
    Stop = 2,
    Quit = 3,
}

/// Trạng thái chia sẻ giữa các luồng (Căn lề 64 bytes chống False Sharing)
#[repr(C, align(64))]
pub struct Signal {
    pub abort: Arc<AtomicBool>,
    pub state: AtomicU8,
    pub epoch: AtomicUsize,
    pub nodes: AtomicU64,
    pub limit: AtomicU64,
    pub pad: [u8; 31],
}

impl Default for Signal {
    fn default() -> Self {
        Self::new()
    }
}

impl Signal {
    /// Khởi tạo instance mới cho Signal với các giá trị mặc định
    pub fn new() -> Self {
        Self {
            abort: Arc::new(AtomicBool::new(false)),
            state: AtomicU8::new(State::Idle as u8),
            epoch: AtomicUsize::new(0),
            nodes: AtomicU64::new(0),
            limit: AtomicU64::new(0),
            pad: [0u8; 31],
        }
    }

    /// Phát lệnh dừng khẩn cấp cho tất cả các luồng worker
    #[inline(always)]
    pub fn halt(&self) {
        self.abort.store(true, Ordering::Relaxed);
    }

    /// Đặt lại trạng thái tín hiệu cho lượt tìm kiếm mới
    #[inline(always)]
    pub fn reset(&self) {
        self.abort.store(false, Ordering::Relaxed);
        self.nodes.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alignment() {
        assert_eq!(std::mem::align_of::<Signal>(), 64);
    }

    #[test]
    fn state() {
        let signal = Signal::new();
        assert!(!signal.abort.load(Ordering::Relaxed));
        signal.halt();
        assert!(signal.abort.load(Ordering::Relaxed));
        signal.reset();
        assert!(!signal.abort.load(Ordering::Relaxed));
    }
}
