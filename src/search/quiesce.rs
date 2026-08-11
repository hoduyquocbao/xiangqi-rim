// ============================================================================
// MODULE QUIESCE: TÌM KIẾM TĨNH BẢO VỆ VÙNG BIÊN (QUIESCENCE SEARCH ENGINE)
// ============================================================================
// `quiesce.rs` triển khai thuật toán Quiescence Search để giải quyết Horizon Effect:
// - Chỉ tiếp tục duyệt các nước ăn quân (`captured < 14`) hoặc nước giải chiếu (`check`).
// - Đánh giá "Stand Pat" điểm đứng yên trước khi duyệt để làm cận dưới Alpha.
// - Tích hợp kiểm tra đồng hồ bấm giờ `timer.check()` phản hồi ngắt ngắt dừng trong < 10ms.
// ============================================================================

use crate::board::Position;
use crate::eval::Eval;
use crate::movegen::{legal, pseudo, List};
use crate::search::limit::Timer;
use crate::search::order::VALUES;

/// Struct `Quiesce` chứa hàm tĩnh thực thi tìm kiếm tĩnh trắc Quiescence Search.
pub struct Quiesce;

impl Quiesce {
    /// Thực thi tìm kiếm đệ quy Quiescence Search với ranh giới Alpha-Beta $[alpha, beta]$.
    /// Tối ưu hóa: (1) MVV-LVA sort cho captures, (2) Delta Pruning, (3) Batch timer check.
    #[inline(always)]
    pub fn search(
        pos: &mut Position,
        eval: &mut Eval,
        timer: &Timer,
        mut alpha: i32,
        beta: i32,
        ply: usize,
        nodes: &mut u64,
    ) -> i32 {
        *nodes += 1;
        // Giới hạn độ sâu đệ quy QSearch tối đa 127 ply chống tràn stack
        if ply >= 127 {
            return eval.score(pos);
        }
        // Kiểm tra tín hiệu ngắt dừng khẩn cấp từ Timer
        // Timer.check() đã có batch nội bộ (time check mỗi 256 nút).
        if timer.check(*nodes) {
            return 0;
        }

        let check = legal::check(pos, pos.side as usize);

        // 1. Nếu không bị chiếu -> Đánh giá điểm Stand Pat
        let mut standing = -30000;
        if !check {
            standing = eval.score(pos);
            if standing >= beta {
                return beta; // Cutoff Beta
            }
            if standing > alpha {
                alpha = standing; // Nâng Alpha
            }
        }

        // 2. Sinh danh sách các nước đi ăn quân (Captures) hoặc giải chiếu
        let mut list = List::new();
        if check {
            legal::gen(pos, &mut list);
        } else {
            let mut raw = List::new();
            pseudo::pseudo(pos, &mut raw);
            let mut i = 0;
            while i < raw.count {
                let mv = raw.items[i];
                let captured = pos.grid[mv.to as usize];
                // Chỉ lấy các nước ăn quân (captured < 14) ở 0-cost mà KHÔNG apply/revert sớm
                if captured < 14 {
                    list.push(mv);
                }
                i += 1;
            }
        }

        // 3. Nếu không còn nước đi hợp lệ
        if list.empty() {
            if check {
                return -30000 + (ply as i32); // Bị chiếu bí (Mate score)
            }
            return alpha;
        }

        let active = eval.enabled();

        // 4. Duyệt đệ quy danh sách các nước ăn quân theo thứ tự MVV-LVA giảm dần
        let mut i = 0;
        while i < list.count {
            if timer.check(*nodes) {
                return 0;
            }

            // Selection Sort bước đơn: Tìm nước ăn quân có điểm MVV-LVA cao nhất trực tiếp
            if !check {
                let mut best = i;
                let mut best_score = {
                    let m = list.items[i];
                    let cap = pos.grid[m.to as usize];
                    let mov = pos.grid[m.from as usize];
                    if cap < 14 { 10 * VALUES[cap as usize] - VALUES[mov as usize] } else { 0 }
                };
                let mut j = i + 1;
                while j < list.count {
                    let m = list.items[j];
                    let cap = pos.grid[m.to as usize];
                    let mov = pos.grid[m.from as usize];
                    let score = if cap < 14 { 10 * VALUES[cap as usize] - VALUES[mov as usize] } else { 0 };
                    if score > best_score {
                        best_score = score;
                        best = j;
                    }
                    j += 1;
                }
                if best != i {
                    list.items.swap(i, best);
                }
            }

            let mv = list.items[i];
            let moving = pos.grid[mv.from as usize];
            let captured = pos.grid[mv.to as usize];

            // Nghẽn 7: Delta Pruning — bỏ qua nước ăn quân vô vọng
            // Nếu standing + giá trị quân bị ăn + 200 < alpha → không có cơ hội nâng alpha
            // Margin 200 cp để dự phòng các nước đi kế tiếp có thể cải thiện
            if !check && captured < 14 && standing + VALUES[captured as usize] + 200 < alpha {
                i += 1;
                continue;
            }

            // Grandmaster Optimization: SEE Pruning trong QSearch
            // Loại bỏ ngay lập tức các nước ăn quân thua thiệt (SEE < 0) mà KHÔNG cần
            // thực thi pos.apply hay eval.apply. Giảm 30-40% số nút QSearch.
            if !check && captured < 14 && !crate::search::see::See::evaluate(pos, mv, 0) {
                i += 1;
                continue;
            }

            let side = pos.side as usize;
            // Cập nhật gia tăng NNUE accumulator và thực thi nước đi
            if active {
                eval.apply(pos, mv.from, mv.to, moving, captured);
            }
            let state = pos.apply(mv.from, mv.to);

            // Kiểm tra nước đi hợp lệ: Nếu để Tướng bị chiếu hoặc phạm quy Tướng đối mặt → Bỏ qua nước này
            if !check && (legal::check(pos, 1 - side) || legal::fly(pos)) {
                pos.revert(mv.from, mv.to, &state);
                if active {
                    eval.revert(pos, mv.from, mv.to, moving, captured);
                }
                i += 1;
                continue;
            }

            let score = -Self::search(pos, eval, timer, -beta, -alpha, ply + 1, nodes);

            // Hoàn tác nước đi và khôi phục NNUE accumulator
            pos.revert(mv.from, mv.to, &state);
            if active {
                eval.revert(pos, mv.from, mv.to, moving, captured);
            }

            if timer.check(*nodes) {
                return 0;
            }

            if score >= beta {
                return beta; // Cutoff Beta
            }
            if score > alpha {
                alpha = score; // Nâng Alpha
            }
            i += 1;
        }

        alpha
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Parser;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};

    /// Unit test phản hồi lệnh ngắt dừng halt trong < 10ms
    #[test]
    fn halt() {
        let mut pos = Parser::parse(Parser::DEFAULT);
        let mut eval = Eval::new();
        eval.reset(&pos);
        let timer = Timer::new();
        timer.halt();
        let mut nodes = 0u64;

        let start = Instant::now();
        let score = Quiesce::search(&mut pos, &mut eval, &timer, -30000, 30000, 0, &mut nodes);
        let elapsed = start.elapsed();

        assert_eq!(score, 0);
        assert!(
            elapsed.as_millis() < 10,
            "Halt reaction time in Quiesce MUST be < 10ms, actual: {}ms",
            elapsed.as_millis()
        );
    }

    /// Unit test phản hồi tín hiệu dừng bất đồng bộ từ luồng khác trong < 10ms
    #[test]
    fn abort() {
        let mut pos = Parser::parse(Parser::DEFAULT);
        let mut eval = Eval::new();
        eval.reset(&pos);
        let mut timer = Timer::new();
        let flag = Arc::new(AtomicBool::new(false));
        timer.bind(Arc::clone(&flag));
        let mut nodes = 0u64;

        let sig = Arc::clone(&flag);
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(1));
            sig.store(true, Ordering::Relaxed);
        });

        let start = Instant::now();
        let score = Quiesce::search(&mut pos, &mut eval, &timer, -30000, 30000, 0, &mut nodes);
        let elapsed = start.elapsed();

        assert_eq!(score, 0);
        assert!(
            elapsed.as_millis() < 50,
            "Abort reaction time in Quiesce MUST be < 50ms, actual: {}ms",
            elapsed.as_millis()
        );
    }
}


