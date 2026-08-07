// ============================================================================
// MODULE BREAKER: BỘ MÁY NGẮT MẠCH BẤT ĐỒNG BỘ NGUYÊN TỬ (LOCK-FREE ATOMIC BREAKER)
// ============================================================================
// `Breaker` quản lý máy trạng thái chuyển đổi giữa NNUE và HCE tự động:
// - Sử dụng các biến nguyên tử `AtomicU8`, `AtomicU32`, `AtomicU64` lock-free
//   cho phép nhiều luồng kiểm tra và ghi nhận kết quả đánh giá song song không khóa.
// - Căn lề bộ nhớ 64-byte `#[repr(C, align(64))]` chiếm đúng 64 bytes (1 L1 Cache line).
// ============================================================================

use std::sync::atomic::{AtomicI32, AtomicU32, AtomicU64, AtomicU8, Ordering};
use super::config::Config;
use super::state::State;

/// Struct `Breaker` quản lý trạng thái ngắt mạch an toàn với thuộc tính căn lề 64-byte.
#[repr(C, align(64))]
#[derive(Debug)]
pub struct Breaker {
    /// Trạng thái ngắt mạch hiện tại (Closed = 0, Half = 1, Open = 2) [AtomicU8]
    pub state: AtomicU8,
    /// Bộ đếm số lần đánh giá thất bại liên tiếp [AtomicU32]
    pub fails: AtomicU32,
    /// Tổng số lần đánh giá đã thực hiện [AtomicU32]
    pub total: AtomicU32,
    /// Ngưỡng số lần lỗi tối đa được phép (Mặc định: 5) [AtomicU32]
    pub limit: AtomicU32,
    /// Mốc thời gian (ticks) xảy ra vụ ngắt mạch gần nhất [AtomicU64]
    pub ticks: AtomicU64,
    /// Khoảng thời gian chờ ngắt mạch tính bằng mili-giây (Mặc định: 10,000 ms) [AtomicU64]
    pub span: AtomicU64,
    /// Bộ đếm số lần chạy thử nghiệm thành công trong trạng thái HalfOpen [AtomicU32]
    pub probe: AtomicU32,
    /// Ranh giới điểm sàn tối thiểu hợp lệ (-29,999) [AtomicI32]
    pub floor: AtomicI32,
    /// Ranh giới điểm trần tối đa hợp lệ (+29,999) [AtomicI32]
    pub ceiling: AtomicI32,
    /// Mảng đệm padding để tổng dung lượng struct tròn đúng 64 bytes (1 L1 Cache Line) [16 bytes]
    pub pad: [u8; 16],
}

impl Default for Breaker {
    /// Khởi tạo mặc định đối tượng Breaker.
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Breaker {
    /// Sao chép trạng thái Breaker với thứ tự truy cập nguyên tử `Ordering::Relaxed`.
    fn clone(&self) -> Self {
        Self {
            state: AtomicU8::new(self.state.load(Ordering::Relaxed)),
            fails: AtomicU32::new(self.fails.load(Ordering::Relaxed)),
            total: AtomicU32::new(self.total.load(Ordering::Relaxed)),
            limit: AtomicU32::new(self.limit.load(Ordering::Relaxed)),
            ticks: AtomicU64::new(self.ticks.load(Ordering::Relaxed)),
            span: AtomicU64::new(self.span.load(Ordering::Relaxed)),
            probe: AtomicU32::new(self.probe.load(Ordering::Relaxed)),
            floor: AtomicI32::new(self.floor.load(Ordering::Relaxed)),
            ceiling: AtomicI32::new(self.ceiling.load(Ordering::Relaxed)),
            pad: [0; 16],
        }
    }
}

impl Breaker {
    /// Khởi tạo một máy ngắt mạch `Breaker` mới từ thông số `Config`.
    pub const fn new() -> Self {
        let config = Config::new();
        Self {
            state: AtomicU8::new(State::Closed.raw()),
            fails: AtomicU32::new(0),
            total: AtomicU32::new(0),
            limit: AtomicU32::new(config.limit),
            ticks: AtomicU64::new(0),
            span: AtomicU64::new(config.span),
            probe: AtomicU32::new(0),
            floor: AtomicI32::new(config.floor),
            ceiling: AtomicI32::new(config.ceiling),
            pad: [0; 16],
        }
    }

    /// Lấy trạng thái hiện tại (`State::Closed`, `State::Half`, `State::Open`).
    #[inline(always)]
    pub fn state(&self) -> State {
        State::parse(self.state.load(Ordering::Relaxed))
    }

    /// Kiểm tra xem có cho phép truy vấn NNUE tại mốc thời gian `tick` hay không.
    pub fn allow(&self, tick: u64) -> bool {
        let curr = self.state();
        match curr {
            State::Closed => true,
            State::Half => true,
            State::Open => {
                let last = self.ticks.load(Ordering::Relaxed);
                let span = self.span.load(Ordering::Relaxed);
                // Nếu đã qua hết khoảng thời gian chờ `span`, thử chuyển từ Open -> HalfOpen qua Compare-and-Swap
                if tick >= last && tick - last >= span {
                    if self.state.compare_exchange(
                        State::Open.raw(),
                        State::Half.raw(),
                        Ordering::AcqRel,
                        Ordering::Relaxed,
                    ).is_ok() {
                        self.probe.store(0, Ordering::Relaxed);
                        return true;
                    }
                }
                false
            }
        }
    }

    /// Ghi nhận kết quả đánh giá thành công/thất bại (`valid: bool`) tại mốc thời gian `tick`.
    pub fn record(&self, valid: bool, tick: u64) {
        self.total.fetch_add(1, Ordering::Relaxed);
        let curr = self.state();

        if valid {
            if curr == State::Half {
                let count = self.probe.fetch_add(1, Ordering::Relaxed) + 1;
                // Nếu chạy thử 100 lần thành công liên tiếp, chuyển HalfOpen -> Closed qua CAS
                if count >= 100 {
                    if self.state.compare_exchange(
                        State::Half.raw(),
                        State::Closed.raw(),
                        Ordering::AcqRel,
                        Ordering::Relaxed,
                    ).is_ok() {
                        self.fails.store(0, Ordering::Relaxed);
                    }
                }
            }
        } else {
            let fails = self.fails.fetch_add(1, Ordering::Relaxed) + 1;
            let limit = self.limit.load(Ordering::Relaxed);

            // Nếu thất bại khi đang ở HalfOpen hoặc vượt quá số lần lỗi cho phép, ngắt mạch lập tức sang Open
            if curr == State::Half || fails >= limit {
                if self.state.swap(State::Open.raw(), Ordering::AcqRel) != State::Open.raw() {
                    self.ticks.store(tick, Ordering::Relaxed);
                }
            }
        }
    }

    /// Đặt lại toàn bộ bộ đếm và trạng thái Breaker về trạng thái ban đầu (`Closed`).
    pub fn reset(&self) {
        self.state.store(State::Closed.raw(), Ordering::Relaxed);
        self.fails.store(0, Ordering::Relaxed);
        self.total.store(0, Ordering::Relaxed);
        self.probe.store(0, Ordering::Relaxed);
        self.ticks.store(0, Ordering::Relaxed);
    }
}

