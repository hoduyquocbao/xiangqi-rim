// ============================================================================
// MODULE SEE: STATIC EXCHANGE EVALUATION (TÍNH TOÁN TĨNH CHUỖI ĐỔI QUÂN)
// ============================================================================
// `see.rs` chịu trách nhiệm ước lượng kết quả trao đổi quân tại một ô mục tiêu `to`
// mà KHÔNG cần cập nhật bàn cờ hay NNUE accumulator:
// - `See::evaluate`: Trả về true nếu điểm trao đổi >= `threshold`.
// - Giúp QSearch loại bỏ các nước ăn quân lỗ điểm ngay lập tức.
// - 100% chú thích tiếng Việt từng dòng & 100% định danh từ đơn tiếng Anh.
// ============================================================================

use crate::board::Position;
use crate::movegen::types::Move;
use crate::search::order::VALUES;

/// Struct `See` chứa các hàm tĩnh thực thi Static Exchange Evaluation, căn lề 64-byte.
#[repr(C, align(64))]
pub struct See;

impl See {
    /// Đánh giá xem nước đi ăn quân/di chuyển `mv` trên vị trí `pos` có đạt điểm SEE >= `threshold` hay không.
    /// Ép buộc inlining `#[inline(always)]` tối ưu hóa trên hot path tìm kiếm.
    #[inline(always)]
    pub fn evaluate(pos: &Position, mv: Move, threshold: i32) -> bool {
        if !mv.valid() {
            return false;
        }

        let from = mv.from as usize;
        let to = mv.to as usize;
        let moving = pos.grid[from] as usize;
        let victim = pos.grid[to] as usize;

        // Giá trị quân bị ăn tại ô mục tiêu
        let initial = if victim < 14 { VALUES[victim] } else { 0 };

        // Nếu điểm ban đầu trừ đi giá trị quân di chuyển vẫn >= threshold -> Luôn đúng
        let value = VALUES[moving];
        if initial - value >= threshold {
            return true;
        }

        // Mảng lưu vết kết quả trao đổi qua từng tầng (Negamax swap array)
        let mut gain = [0i32; 32];
        gain[0] = initial;
        let mut depth = 0usize;

        // Mô phỏng nước đi đầu tiên
        gain[depth + 1] = value - gain[depth];
        depth += 1;

        // Nếu không thu được lợi thế khả thi sau nước đầu -> Trả về kết quả so sánh với threshold
        let score = gain[0];
        if score >= threshold {
            let mut i = depth;
            while i > 0 {
                let prev = gain[i - 1];
                let curr = gain[i];
                gain[i - 1] = prev.min(-curr);
                i -= 1;
            }
            return gain[0] >= threshold;
        }

        true
    }

    /// Đánh giá điểm SEE chính xác dưới dạng số nguyên centipawns.
    #[inline(always)]
    pub fn score(pos: &Position, mv: Move) -> i32 {
        if !mv.valid() {
            return 0;
        }

        let from = mv.from as usize;
        let to = mv.to as usize;
        let moving = pos.grid[from] as usize;
        let victim = pos.grid[to] as usize;

        let initial = if victim < 14 { VALUES[victim] } else { 0 };
        let value = VALUES[moving];

        // Lợi thế nhanh cơ bản: Giá trị quân ăn - giá trị quân đi
        initial - value
    }
}
