// ============================================================================
// MODULE PROVER: ĐỘNG CƠ VÉT CẠN CÂY QUYẾT ĐỊNH BACKTRACKING & UNDO SĂN SÁT CỤC
// ============================================================================
// prover.rs vận hành thuật toán vét cạn toàn bộ cây quyết định bằng cơ chế Undo:
// 1. Duyệt toàn bộ các nước đi hợp lệ ở mọi tầng rẽ nhánh (OR-nodes & AND-nodes).
// 2. Sử dụng `pos.apply(from, to)` đi sâu vào nhánh; khi gặp nhánh thua (-MATE) hoặc
//    đã chứng minh xong sát cục (+MATE), tự động `pos.revert(from, to, &state)` (Undo)
//    trở về nút quyết định trước đó để rẽ sang tất cả các biến thể khác!
// 3. Khai quật toàn bộ hàng trăm đường sát cục ẩn sâu trong một thế trận khai cuộc.
// 4. Lưu từng Trạm Sát Cục (Checkmate Waypoint) vào Kho Tri Thức Vault 1,024 Shards.
// 100% chú thích tiếng Việt từng dòng & 100% định danh từ đơn tiếng Anh.
// ============================================================================

use crate::board::{Position, State};
use crate::movegen::{legal, types::List, types::Move};
use crate::search::{Limits, Search};
use crate::system::{PnCounter, Vault};
use crate::uci::Format;

/// Cấu trúc lưu một bước đi trên ngăn xếp Backtracking Stack
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    /// Nước đi đã thực hiện
    pub step: Move,
    /// Trạng thái phục hồi vi phân Bitboard
    pub state: State,
    /// Điểm số đánh giá Centipawn
    pub score: i32,
    /// Độ sâu tìm kiếm
    pub depth: u8,
    /// Khoảng cách sát cục Mate-in-N
    pub mate: i8,
}

impl Frame {
    /// Khởi tạo một `Frame` mới
    #[inline(always)]
    pub fn new(step: Move, state: State, score: i32, depth: u8, mate: i8) -> Self {
        Self {
            step,
            state,
            score,
            depth,
            mate,
        }
    }
}

/// Cấu trúc lưu một đường chứng minh sát cục thắng trọn vẹn (Winning Checkmate Proof Path)
#[derive(Clone, Debug)]
pub struct Proof {
    /// Chuỗi các nước đi từ thế cờ gốc dẫn tới sát cục (UCI string)
    pub path: Vec<String>,
    /// Nước đi dứt điểm
    pub final_move: String,
    /// Điểm số sát cục
    pub score: i32,
    /// Độ sâu chứng minh
    pub depth: u8,
    /// Khoảng cách sát cục (Mate in N plies)
    pub mate: i8,
}

/// Struct `Prover` quản lý việc duyệt vét cạn cây quyết định và khôi phục trạng thái bằng Undo
pub struct Prover {
    /// Độ sâu tìm kiếm cho mỗi nút quyết định
    pub depth: u8,
    /// Số tầng ply tối đa cho cây quyết định
    pub max_ply: usize,
    /// Động cơ tìm kiếm Alpha-Beta/PVS khởi tạo duy nhất 1 lần
    pub search: Search,
    /// Danh sách các đường sát cục đã chứng minh thành công
    pub proofs: Vec<Proof>,
    /// Bộ đếm số sát cục tìm thấy
    pub mates_found: PnCounter,
    /// Bộ đếm số nút đã duyệt
    pub nodes_visited: PnCounter,
    /// Bộ đếm số lần Undo / Backtrack
    pub backtracks: PnCounter,
}

impl Prover {
    /// Khởi tạo Động cơ Chứng minh Sát cục `Prover` mới
    pub fn new(depth: u8, max_ply: usize) -> Self {
        Self {
            depth,
            max_ply,
            search: Search::new(4),
            proofs: Vec::with_capacity(1000),
            mates_found: PnCounter::new(),
            nodes_visited: PnCounter::new(),
            backtracks: PnCounter::new(),
        }
    }

    /// Khởi động tiến trình vét cạn toàn bộ cây quyết định từ thế cờ `pos`
    pub fn prove(&mut self, pos: &mut Position, vault: &Vault) -> usize {
        let mut path = Vec::with_capacity(32);
        let mut history = Vec::with_capacity(32);
        self.dfs(pos, 0, &mut path, &mut history, vault);
        self.proofs.len()
    }

    /// Hàm duyệt đệ quy DFS kết hợp MakeMove (`apply`) và Undo (`revert`)
    fn dfs(
        &mut self,
        pos: &mut Position,
        ply: usize,
        path: &mut Vec<String>,
        history: &mut Vec<u64>,
        vault: &Vault,
    ) {
        self.nodes_visited.add(1);

        // 1. Kiểm tra lặp cờ trên nhánh hiện tại (Exact Path Cycle Detection)
        if history.contains(&pos.hash) {
            return;
        }
        history.push(pos.hash);

        // 2. Nếu có nước Chiếu Tướng (pos.check != 0) hoặc ở nút gốc (ply == 0), kích hoạt Deep Search chứng minh Sát Cục
        if pos.check != 0 || ply == 0 {
            let mut limits = Limits::new();
            limits.depth = self.depth;
            let res = self.search.go(pos, &limits);

            let best_uci = Format::encode(res.best);

            // A. Nếu phát hiện Sát Cục Thắng (+MATE: Score >= 29000)
            if res.score >= 29000 {
                let mate_val = (30000 - res.score).clamp(1, 127) as i8;
                self.mates_found.add(1);

                let mut proof_path = path.clone();
                proof_path.push(best_uci.clone());

                self.proofs.push(Proof {
                    path: proof_path,
                    final_move: best_uci.clone(),
                    score: res.score,
                    depth: self.depth,
                    mate: mate_val,
                });

                // Bảo tồn Trạm Sát Cục vào Vault
                if res.best.valid() {
                    vault.save_mate(pos, self.depth, res.best, res.score, mate_val, 0);
                }

                // Nếu là nút con (ply > 0), đã dứt điểm sát cục ở nhánh này -> Dừng và Backtrack
                if ply > 0 {
                    history.pop();
                    return;
                }
            }

            // B. Nếu nhánh bị thua cờ hoặc đạt max_ply -> Dừng đào sâu, chuẩn bị Undo
            if (res.score <= -29000 || ply >= self.max_ply) && ply > 0 {
                history.pop();
                return;
            }

            // C. Lưu thế cờ trung gian vào Vault nếu điểm số đáng chú ý (|score| >= 300)
            if res.score.abs() >= 300 && res.best.valid() {
                vault.save_mate(pos, self.depth, res.best, res.score, 0, 0);
            }
        } else if ply >= self.max_ply {
            history.pop();
            return;
        }

        // 3. Sinh toàn bộ các nước đi hợp lệ ở nút quyết định này để vét cạn mọi khả năng
        let mut moves = List::new();
        legal::gen(pos, &mut moves);

        for i in 0..moves.len() {
            let mv = moves.items[i];
            let mv_uci = Format::encode(mv);

            // A. THỰC HIỆN NƯỚC ĐI (MakeMove / Apply)
            let state = pos.apply(mv.from, mv.to);
            path.push(mv_uci);

            // B. ĐI SÂU VÀO NHÁNH CON (Recurse)
            self.dfs(pos, ply + 1, path, history, vault);

            // C. QUAY LUI VÀ PHỤC HỒI TRẠNG THÁI (Undo / Revert Stack)
            path.pop();
            pos.revert(mv.from, mv.to, &state);
            self.backtracks.add(1);
        }

        history.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Parser;

    #[test]
    fn test_prover_backtracking_and_undo_invariants() {
        let mut pos = Parser::parse("4k4/4a4/4ba3/9/2r6/9/9/4C4/3N5/4K1R2 w - - 0 1");
        let original_hash = pos.hash;
        let original_grid = pos.grid;
        let original_side = pos.side;

        let vault = Vault::global();
        let mut prover = Prover::new(4, 1);

        let count = prover.prove(&mut pos, vault);

        // Sau khi vét cạn toàn bộ cây và Undo, bàn cờ PHẢI nguyên vẹn 100%
        assert_eq!(pos.hash, original_hash, "Hash phải bảo toàn sau khi Undo!");
        assert_eq!(pos.grid, original_grid, "Grid phải bảo toàn sau khi Undo!");
        assert_eq!(pos.side, original_side, "Side phải bảo toàn sau khi Undo!");
        assert!(prover.backtracks.get() > 0, "Phải thực hiện ít nhất 1 lần Backtrack Undo!");
    }
}
