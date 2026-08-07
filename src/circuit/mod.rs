// ============================================================================
// MODULE CIRCUIT: MÁY TRẠNG THÁI NGẮT MẠCH BẢO VỆ DỰ PHÒNG (CIRCUIT BREAKER SYSTEM)
// ============================================================================
// Circuit Breaker giám sát độ ổn định của mạng nơ-ron NNUE trong suốt quá trình chạy:
// - `Closed` (Mạch đóng): NNUE hoạt động bình thường, tất cả truy vấn được chấp nhận.
// - `Open` (Mạch mở/ngắt): NNUE gặp lỗi hoặc quá ngưỡng thất bại, ngắt NNUE và hạ cấp tự động sang HCE.
// - `HalfOpen` (Mạch thử nghiệm): Thử nghiệm lại NNUE sau khoảng thời gian chờ, phục hồi về `Closed` nếu thành công.
// ============================================================================

/// Module con `breaker` bộ điều khiển ngắt mạch CircuitBreaker (align 64)
pub mod breaker;
/// Module con `check` kiểm tra tính hợp lệ của điểm số đánh giá
pub mod check;
/// Module con `config` cấu hình ngưỡng lỗi thất bại và thời gian chờ ngắt mạch
pub mod config;
/// Module con `state` Enum các trạng thái ngắt mạch (Closed, Open, Half)
pub mod state;

pub use breaker::Breaker;
pub use check::Check;
pub use config::Config;
pub use state::State;

// ----------------------------------------------------------------------------
// KHU VỰC BÀI KIỂM THỬ ĐƠN VỊ (UNIT TESTS) CHO CIRCUIT BREAKER
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// Kiểm thử căn lề bộ nhớ 64-byte align 64 cho `Breaker` và `Config`.
    #[test]
    fn alignments() {
        assert_eq!(std::mem::align_of::<Breaker>(), 64);
        assert_eq!(std::mem::align_of::<Config>(), 64);
    }

    /// Kiểm thử trạng thái ban đầu `Closed` cho phép truy vấn NNUE bình thường.
    #[test]
    fn closed() {
        let breaker = Breaker::new();
        assert_eq!(breaker.state(), State::Closed);
        assert!(breaker.allow(0));
    }

    /// Kiểm thử tự động ngắt mạch nhảy sang `Open` khi liên tục ghi nhận 5 lỗi thất bại.
    #[test]
    fn trip() {
        let breaker = Breaker::new();
        for _ in 0..5 {
            breaker.record(false, 10);
        }
        assert_eq!(breaker.state(), State::Open);
        assert!(!breaker.allow(10));
    }

    /// Kiểm thử chu trình phục hồi: `Open` -> `HalfOpen` -> `Closed` sau khi thử nghiệm thành công.
    #[test]
    fn recover() {
        let breaker = Breaker::new();
        for _ in 0..5 {
            breaker.record(false, 100);
        }
        assert_eq!(breaker.state(), State::Open);

        // Hết thời gian chờ 10,000ms -> Chuyển sang HalfOpen để thử nghiệm
        assert!(breaker.allow(10100));
        assert_eq!(breaker.state(), State::Half);

        // Thử nghiệm 100 lần thành công -> Phục hồi mạch về trạng thái Closed ban đầu
        for _ in 0..100 {
            breaker.record(true, 10100);
        }
        assert_eq!(breaker.state(), State::Closed);
    }
}

