// ============================================================================
// MODULE TIME: HỆ THỐNG CẤP PHÁT THỜI GIAN THÍCH ỨNG ĐỘNG (TIME ALLOCATOR SYSTEM)
// ============================================================================
// `TimeSystem` chịu trách nhiệm:
// - Cấp phát thời gian thích ứng động theo trạng thái ván cờ (Dynamic Critical Time Allocator).
// - Nhận diện tình huống nguy cấp: đang bị chiếu tướng, biến động điểm số lớn, hoặc tàn cuộc then chốt.
// - Tự động nhân hệ số thời gian (x1.5 -> x2.2) cho phép Xiangqi-RIM bung sức ở nước đi sinh tử.
// ============================================================================

use crate::board::Position;
use crate::movegen::legal;
use crate::system::weights::Weights;

/// Struct `TimeSystem` quản lý toàn bộ thuật toán cấp phát thời gian
pub struct TimeSystem;

impl TimeSystem {
    /// Tính toán thời gian tìm kiếm tối ưu cho nước đi hiện tại (mili-giây)
    #[inline(always)]
    pub fn allocate(pos: &Position, weights: &Weights, prev_score: i32, curr_score: i32, ply: usize) -> u64 {
        let base_ms = weights.time.base_ms as f64;
        let mut multiplier = 1.0f64;

        // 1. Kiểm tra trạng thái đang bị chiếu tướng
        let in_check = legal::check(pos, pos.side as usize);
        if in_check {
            multiplier *= weights.time.check_mult;
        }

        // 2. Kiểm tra biến động điểm số chiến thuật lớn (Tactical Panic / Opportunity)
        let score_diff = (curr_score - prev_score).abs();
        if score_diff >= 150 {
            multiplier *= weights.time.tactical_mult;
        }

        // 3. Cơ hội dứt điểm sát cục khi đang dẫn thế lớn (Decisive Winning Conversion: score >= +300cp)
        if curr_score >= 300 {
            multiplier *= 1.6;
        }

        // 4. Kiểm tra giai đoạn tàn cuộc sâu (Endgame Critical Phase: Ply >= 40)
        if ply >= 40 {
            multiplier *= weights.time.endgame_mult;
        }

        let allocated = (base_ms * multiplier) as u64;
        allocated.clamp(weights.time.min_ms, weights.time.max_ms)
    }
}
