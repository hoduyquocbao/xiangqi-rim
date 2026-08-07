// ============================================================================
// MODULE LIMIT: QUẢN LÝ THỜI GIAN VÀ THAM SỐ GIỚI HẠN TÌM KIẾM (SEARCH LIMITS & TIMER)
// ============================================================================
// `limit.rs` chịu trách nhiệm điều phối toàn bộ thời gian và điều kiện dừng của Search Engine:
// - `Limits`: Chứa các thông số giới hạn từ câu lệnh UCI `go` (`depth`, `moves`, `time`, `inc`, `nodes`, `infinite`).
// - `Timer`: Quản lý mốc thời gian thực tế `start: Instant`, thời gian tối ưu `optimum`, thời gian tối đa `maximum`,
//   và cờ ngắt tín hiệu nguyên tử `AtomicBool`.
// - `Result`: Chứa kết quả nước đi tốt nhất (`best`), nước đi dự đoán (`ponder`), điểm số (`score`), và chuỗi biến thể chính (`pv`).
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn now() -> f64;
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instant {
    stamp: u64,
}

#[cfg(target_arch = "wasm32")]
impl Instant {
    #[inline(always)]
    pub fn now() -> Self {
        let val = unsafe { now() };
        Self { stamp: val as u64 }
    }

    #[inline(always)]
    pub fn elapsed(&self) -> std::time::Duration {
        let cur = unsafe { now() } as u64;
        let diff = cur.saturating_sub(self.stamp);
        std::time::Duration::from_millis(diff)
    }
}

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::movegen::types::{List, Move};

/// Struct `Limits` chứa các chỉ số giới hạn từ giao thức UCI, căn lề 64-byte.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// Giới hạn độ sâu tìm kiếm (u8)
    pub depth: u8,
    /// Giới hạn số nước đi còn lại
    pub moves: u16,
    /// Tổng thời gian còn lại của phe hiện tại tính bằng ms
    pub time: u64,
    /// Thời gian tăng thêm sau mỗi nước (Increment) tính bằng ms
    pub inc: u64,
    /// Thời gian dành riêng chính xác cho nước đi này tính bằng ms
    pub exact: u64,
    /// Giới hạn tổng số nút cây cờ tối đa được duyệt
    pub nodes: u64,
    /// Cờ đánh dấu tìm kiếm vô hạn cho đến khi nhận lệnh stop
    pub infinite: bool,
}

impl Default for Limits {
    /// Khởi tạo mặc định đối tượng Limits.
    fn default() -> Self {
        Self::new()
    }
}

impl Limits {
    /// Khởi tạo một đối tượng Limits rỗng bằng 0.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            depth: 0,
            moves: 0,
            time: 0,
            inc: 0,
            exact: 0,
            nodes: 0,
            infinite: false,
        }
    }
}

/// Struct `Timer` quản lý đồng hồ đếm ngược và kiểm tra tín hiệu ngắt dừng, căn lề 64-byte.
#[repr(C, align(64))]
#[derive(Debug)]
pub struct Timer {
    /// Mốc thời gian bắt đầu phiên tìm kiếm
    pub start: Instant,
    /// Thời gian lý tưởng nên dừng phiên tìm kiếm (ms)
    pub optimum: u64,
    /// Thời gian tối đa bắt buộc phải ngắt lập tức (ms)
    pub maximum: u64,
    /// Bộ đếm số nút đã duyệt
    pub nodes: u64,
    /// Độ sâu tìm kiếm hiện tại
    pub depth: u8,
    /// Cấu hình giới hạn Limits
    pub limit: Limits,
    /// Cờ nguyên tử báo ngắt dừng phiên tìm kiếm [AtomicBool]
    pub abort: AtomicBool,
    /// Con trỏ tín hiệu dừng từ luồng chính toàn cục (Global halt signal)
    pub signal: Option<Arc<AtomicBool>>,
}

impl Default for Timer {
    /// Khởi tạo mặc định đối tượng Timer.
    fn default() -> Self {
        Self::new()
    }
}

impl Timer {
    /// Khoảng thời gian an toàn trừ đi tránh quá giờ (Safety Margin = 20ms)
    pub const SAFETY: u64 = 20;

    /// Khởi tạo Timer mới với các giá trị mặc định vô hạn.
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            optimum: u64::MAX,
            maximum: u64::MAX,
            nodes: 0,
            depth: 0,
            limit: Limits::new(),
            abort: AtomicBool::new(false),
            signal: None,
        }
    }

    /// Đăng ký liên kết con trỏ tín hiệu nguyên tử từ luồng chính.
    #[inline(always)]
    pub fn bind(&mut self, signal: Arc<AtomicBool>) {
        self.signal = Some(signal);
    }

    /// Khởi tạo đồng hồ bấm giờ `Timer` trước khi bắt đầu phiên tìm kiếm mới.
    #[inline(always)]
    pub fn init(&mut self, limit: &Limits, _side: u8) {
        self.start = Instant::now();
        self.limit = *limit;
        self.nodes = 0;
        self.depth = 0;
        if let Some(ref sig) = self.signal {
            if sig.load(Ordering::Relaxed) {
                self.abort.store(true, Ordering::Relaxed);
            }
        }
        if !self.abort.load(Ordering::Relaxed) {
            self.abort.store(false, Ordering::Relaxed);
        }

        // Tính toán khoảng thời gian `optimum` (Soft Limit) và `maximum` (Hard Limit)
        if limit.exact > 0 {
            let hard = limit.exact.saturating_sub(Self::SAFETY);
            let soft = (hard * 7) / 10;
            self.optimum = soft;
            self.maximum = hard;
        } else if limit.time > 0 {
            let remaining = limit.time.saturating_sub(Self::SAFETY);
            let horizon = if limit.moves > 0 { limit.moves as u64 } else { 30 };
            let opt = (remaining / horizon) + (limit.inc * 3 / 4);
            let max = (remaining / 5) + limit.inc;
            self.optimum = opt;
            self.maximum = max;
        } else {
            self.optimum = u64::MAX;
            self.maximum = u64::MAX;
        }
    }

    /// Kiểm tra định kỳ xem có cần ngắt dừng phiên tìm kiếm hay không.
    /// Kiểm tra cờ nguyên tử ngắt khẩn cấp trên MỌI nút cờ (0 CPU clock overhead).
    /// Kiểm tra thời gian tối đa và giới hạn nút cờ mỗi 256 nút cờ (`nodes & 255 == 0`).
    #[inline(always)]
    pub fn check(&self, nodes: u64) -> bool {
        if self.abort.load(Ordering::Relaxed) {
            return true;
        }
        if let Some(ref sig) = self.signal {
            if sig.load(Ordering::Relaxed) {
                self.abort.store(true, Ordering::Relaxed);
                return true;
            }
        }
        if (nodes & 255) == 0 {
            if self.limit.nodes > 0 && nodes >= self.limit.nodes {
                self.abort.store(true, Ordering::Relaxed);
                return true;
            }
            if self.maximum != u64::MAX {
                let elapsed = self.start.elapsed().as_millis() as u64;
                if elapsed >= self.maximum {
                    self.abort.store(true, Ordering::Relaxed);
                    return true;
                }
            }
        }
        false
    }

    /// Kiểm tra thời gian đã vượt quá mốc thời gian lý tưởng `optimum` hay chưa.
    #[inline(always)]
    pub fn expired(&self) -> bool {
        if self.abort.load(Ordering::Relaxed) {
            return true;
        }
        if let Some(ref sig) = self.signal {
            if sig.load(Ordering::Relaxed) {
                self.abort.store(true, Ordering::Relaxed);
                return true;
            }
        }
        if self.maximum != u64::MAX {
            let elapsed = self.start.elapsed().as_millis() as u64;
            return elapsed >= self.optimum;
        }
        false
    }

    /// Phát lệnh ngắt dừng lập tức phiên tìm kiếm hiện tại.
    #[inline(always)]
    pub fn halt(&self) {
        self.abort.store(true, Ordering::Relaxed);
    }
}

/// Struct `Result` lưu trữ kết quả cuối cùng thu được từ Search Engine, căn lề 64-byte.
#[repr(C, align(64))]
#[derive(Clone, Debug)]
pub struct Result {
    /// Nước đi tốt nhất tìm được (Best Move)
    pub best: Move,
    /// Nước đi tiên đoán đối phương sẽ đi (Ponder Move)
    pub ponder: Move,
    /// Điểm số thế cờ (Centipawn Score)
    pub score: i32,
    /// Độ sâu tìm kiếm tối đa đạt được
    pub depth: u8,
    /// Tổng số nút cây cờ đã duyệt
    pub nodes: u64,
    /// Khoảng thời gian đã thực thi (ms)
    pub time: u64,
    /// Danh sách các nước đi thuộc biến thể chính (PV Line)
    pub pv: List,
}

impl Default for Result {
    /// Khởi tạo kết quả mặc định.
    fn default() -> Self {
        Self::new()
    }
}

impl Result {
    /// Khởi tạo một đối tượng Result rỗng.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            best: Move::none(),
            ponder: Move::none(),
            score: 0,
            depth: 0,
            nodes: 0,
            time: 0,
            pv: List::new(),
        }
    }
}

