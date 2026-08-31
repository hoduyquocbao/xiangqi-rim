// ============================================================================
// MODULE MULTIPV: TÌM KIẾM ĐA BIẾN THỂ CHÍNH (MULTI PRINCIPAL VARIATIONS)
// ============================================================================
// `multipv.rs` triển khai thuật toán Multi-PV phục vụ phân tích chuyên sâu:
// - Tìm kiếm Top N nước đi ứng viên xuất sắc nhất tại nút gốc (Root Position).
// - Mỗi ứng viên `Candidate` lưu trữ đầy đủ: nước đi `step`, điểm số `score`,
//   độ sâu `depth`, số nút `nodes`, và chuỗi biến thể `line`.
// - Hỗ trợ Web UI, Engine Analysis, và giao thức JRCP 3.0 xuất Top 3 nước đi.
//
// 100% chú thích tiếng Việt từng dòng & 100% định danh từ đơn tiếng Anh.
// ============================================================================

use crate::board::Position;
use crate::eval::Eval;
use crate::movegen::{legal, types::List, Move};
use crate::search::core::Core;
use crate::search::diversity::Diversity;
use crate::search::limit::Timer;
use crate::search::order::{History, Killer};
use crate::tt::Table;

/// Cấu trúc `Candidate` đại diện cho một nhánh biến thể ứng viên hàng đầu trong Multi-PV.
#[repr(C, align(64))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    /// Nước đi ứng viên tại nút gốc (Move)
    pub step: Move,
    /// Điểm số đánh giá Centipawn Score (i32)
    pub score: i32,
    /// Độ sâu tìm kiếm hoàn tất (u8)
    pub depth: u8,
    /// Tổng số nút cây cờ đã duyệt cho nhánh này (u64)
    pub nodes: u64,
    /// Chuỗi biến thể tiếp theo (PV Line)
    pub line: List,
}

impl Default for Candidate {
    /// Khởi tạo mặc định ứng viên Candidate.
    fn default() -> Self {
        Self::new()
    }
}

impl Candidate {
    /// Tạo mới một ứng viên rỗng.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            step: Move::none(),
            score: 0,
            depth: 0,
            nodes: 0,
            line: List::new(),
        }
    }
}

/// Struct `Multi` điều phối việc tìm kiếm đa biến thể Multi-PV tại nút gốc.
pub struct Multi;

impl Multi {
    /// Tìm kiếm Top `count` biến thể ứng viên xuất sắc nhất tại nút gốc `pos`.
    pub fn search(
        pos: &mut Position,
        eval: &mut Eval,
        tt: Option<&Table>,
        history: &mut History,
        killer: &mut Killer,
        timer: &Timer,
        diversity: Option<&Diversity>,
        past: Option<&[u64]>,
        count: usize,
        depth: u8,
    ) -> Vec<Candidate> {
        let mut candidates = Vec::new();
        let target = count.max(1).min(5);

        // Sinh toàn bộ nước đi hợp lệ tại nút gốc
        let mut legals = List::new();
        legal::gen(pos, &mut legals);

        if legals.count == 0 {
            return candidates;
        }

        let mut excluded = Vec::new();
        let active = eval.enabled();
        let mut stack = Box::new([crate::search::stack::Stack::new(); 128]);

        // Lặp qua từng bậc PV (từ PV 1 đến PV N)
        for _pv_idx in 0..target {
            if excluded.len() >= legals.count {
                break;
            }

            let mut best_move = Move::none();
            let mut best_score = -Core::MATE;
            let mut total_nodes = 0u64;
            let mut best_line = List::new();

            for i in 0..legals.count {
                let mv = legals.items[i];
                if excluded.contains(&mv) {
                    continue;
                }

                let moving = pos.grid[mv.from as usize];
                let captured = pos.grid[mv.to as usize];

                if active {
                    eval.apply(pos, mv.from, mv.to, moving, captured);
                }
                let state = pos.apply(mv.from, mv.to);

                let mut nodes = 0u64;

                // Tìm kiếm với độ sâu `depth - 1`
                let score = -Core::pvs(
                    pos,
                    eval,
                    tt,
                    history,
                    killer,
                    &mut *stack,
                    timer,
                    diversity,
                    past,
                    (depth as i32) - 1,
                    -Core::MATE,
                    Core::MATE,
                    1,
                    &mut nodes,
                );

                pos.revert(mv.from, mv.to, &state);
                if active {
                    eval.revert(pos, mv.from, mv.to, moving, captured);
                }

                total_nodes += nodes;

                if score > best_score || !best_move.valid() {
                    best_score = score;
                    best_move = mv;
                    best_line.clear();
                    best_line.push(mv);
                    for j in 0..stack[1].pv.len {
                        if stack[1].pv.items[j].valid() {
                            best_line.push(stack[1].pv.items[j]);
                        }
                    }
                }

                if timer.check(total_nodes) {
                    break;
                }
            }

            if best_move.valid() {
                excluded.push(best_move);
                candidates.push(Candidate {
                    step: best_move,
                    score: best_score,
                    depth,
                    nodes: total_nodes,
                    line: best_line,
                });
            }
        }

        candidates
    }
}
