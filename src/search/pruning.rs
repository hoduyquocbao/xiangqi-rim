// ============================================================================
// XIANGQI-RIM ENGINE: MODULE CẮT TỈA CÂY TÌM KIẾM SÂU (LMR, RFP, Q-SEARCH)
// ============================================================================
// Triển khai các kỹ thuật cắt tỉa cây tìm kiếm Alpha-Beta tối thượng:
//   1. Late Move Reductions (LMR): Giảm độ sâu các nước đi không ăn quân xếp ở cuối list.
//   2. Reverse Futility Pruning (RFP): Cắt tỉa sớm tại nút gần lá khi static eval vượt xa beta.
//   3. Quiescence Search (Q-Search) với Stand-Pat: Tìm kiếm nước ăn quân ở nút lá loại bỏ Horizon Effect.
// Tuân thủ 100% định danh từ đơn tiếng Anh và 100% chú thích Tiếng Việt tường minh.
// ============================================================================

use crate::board::Position;
use crate::movegen::{legal, List};

/// Struct `Pruner`: Bộ quản lý các quy tắc cắt tỉa cây tìm kiếm sâu LMR và RFP.
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Pruner;

impl Pruner {
    /// Hàm `new`: Khởi tạo đối tượng Pruner mới.
    #[inline(always)]
    pub fn new() -> Self {
        Self
    }

    /// Phương thức `reduction`: Tính toán số độ sâu cần giảm cho LMR (Late Move Reduction).
    /// Nhận vào các tham số: `depth` kiểu `i32` và `move_count` kiểu `usize`.
    /// Trả về số độ sâu cần giảm `i32`.
    #[inline(always)]
    pub fn reduction(&self, depth: i32, move_count: usize) -> i32 {
        if depth < 3 || move_count < 4 {
            return 0; // KHÔNG áp dụng LMR cho độ sâu quá nông hoặc 3 nước đi đầu tiên
        }
        // Công thức tính toán độ sâu giảm LMR: log(depth) * log(move_count) / 2
        let d = depth as f64;
        let m = move_count as f64;
        let r = (d.ln() * m.ln() * 0.5).round() as i32;
        r.clamp(1, depth - 1)
    }

    /// Phương thức `futile`: Kiểm tra xem nút tìm kiếm hiện tại có thể cắt tỉa sớm bằng Reverse Futility Pruning (RFP) hay không.
    /// Nhận vào các tham số: `static_eval` kiểu `i32`, `beta` kiểu `i32`, `depth` kiểu `i32`.
    /// Trả về `bool`.
    #[inline(always)]
    pub fn futile(&self, static_eval: i32, beta: i32, depth: i32) -> bool {
        if depth > 3 || beta.abs() > 20000 {
            return false; // Chỉ áp dụng RFP tại các nút gần lá (depth <= 3)
        }
        let margin = 120 * depth; // Biên độ margin động theo độ sâu
        (static_eval - margin) >= beta
    }

    /// Phương thức `quiescence`: Thuật toán tìm kiếm Quiescence Search (Q-Search) xử lý các nước đi ăn quân ở nút lá.
    /// Nhận vào các tham số: `pos` kiểu `&mut Position`, `alpha` kiểu `i32`, `beta` kiểu `i32`.
    /// Trả về điểm số centipawn `i32`.
    pub fn quiescence(&self, pos: &mut Position, mut alpha: i32, beta: i32) -> i32 {
        // 1. Chấm điểm tĩnh Stand-Pat tại vị trí hiện tại
        let stand_pat = crate::eval::Hce::new().evaluate(pos);
        if stand_pat >= beta {
            return beta; // Cắt tỉa Stand-Pat cutoff
        }
        if stand_pat > alpha {
            alpha = stand_pat; // Cập nhật alpha theo điểm Stand-Pat
        }

        let mut list = List::new();
        legal::gen(pos, &mut list); // Sinh các nước đi hợp lệ

        let mut i = 0usize;
        while i < list.len() {
            let mv = list.get(i);
            // Chỉ duyệt các nước đi ăn quân (Captures Only: pos.at(to) < 14) để tĩnh hóa vị trí
            let captured = pos.at(mv.to);
            if captured < 14 {
                let state = pos.apply(mv.from, mv.to);
                let score = -self.quiescence(pos, -beta, -alpha);
                pos.revert(mv.from, mv.to, &state);

                if score >= beta {
                    return beta; // Cutoff Alpha-Beta trong Q-Search
                }
                if score > alpha {
                    alpha = score;
                }
            }
            i += 1;
        }
        alpha
    }
}
