// ============================================================================
// MODULE HUNTER: ĐỘNG CƠ SĂN SÁT CỤC CHIẾU BÍ ĐA TẦNG & DUAL-PRIORITY TASK QUEUE
// ============================================================================
// hunter.rs vận hành tiến trình tự động đào sâu và rẽ nhánh săn tìm sát cục:
// 1. Quét tầng nông phát hiện ưu thế chiến thuật (|score| >= 300cp).
// 2. Dual Priority Queue: Ưu tiên DeepenMate & ForcingCheck trên hàng đợi Urgent!
// 3. Tự động sinh Task đào sâu (DeepenMate) lên Depth 20..32 để chứng minh sát cục.
// 4. Tính toán chính xác khoảng cách sát cục chuẩn (Mate in N plies).
// 5. Lưu vĩnh viễn vào Kho Tri Thức Vault để phục vụ tra cứu tương lai O(1) tức thì!
// 100% chú thích tiếng Việt từng dòng & 100% định danh từ đơn tiếng Anh.
// ============================================================================

use std::collections::VecDeque;

use crate::board::{Position, Serializer};
use crate::circuit::Breaker;
use crate::eval::Sieve;
use crate::movegen::{legal, types::List};
use crate::system::{Counter, PnCounter, Vault};
use crate::uci::Format;

/// Loại nhiệm vụ trong hàng đợi Dynamic Task Queue
#[derive(Clone, Debug)]
pub enum Task {
    /// Nhiệm vụ mở rộng cây khai/trung cuộc thông thường
    Explore {
        /// Thế cờ hiện tại
        board: Position,
        /// Lịch sử nước đi
        path: Vec<String>,
        /// Độ sâu tìm kiếm
        depth: u8,
        /// Tầng ply hiện tại
        ply: usize,
    },
    /// Nhiệm vụ đào sâu săn tìm sát cục dứt điểm (Deepen Mate Pursuit)
    DeepenMate {
        /// Thế cờ hiện tại
        board: Position,
        /// Lịch sử nước đi
        path: Vec<String>,
        /// Độ sâu tìm kiếm cao
        depth: u8,
        /// Mục tiêu số nước chiếu bí
        target_mate: u8,
    },
    /// Nhiệm vụ rẽ nhánh vét cạn toàn bộ chuỗi nước Chiếu Bắt Buộc (Forcing Check)
    ForcingCheck {
        /// Thế cờ hiện tại
        board: Position,
        /// Lịch sử nước đi
        path: Vec<String>,
        /// Độ sâu tìm kiếm
        depth: u8,
    },
}

/// Cấu trúc lưu kết quả thẩm định sát cục chuẩn xác
#[derive(Clone, Debug, Default)]
pub struct MateInfo {
    /// Thế cờ bàn cờ
    pub fen: String,
    /// Nước đi tối thượng
    pub best: String,
    /// Điểm số Centipawn
    pub score: i32,
    /// Độ sâu đã giải
    pub depth: u8,
    /// Khoảng cách sát cục (+N: Thắng sau N plies, -N: Thua sau N plies, 0: Chưa thấy)
    pub mate: i8,
    /// Đã xác nhận sát cục 100%
    pub solved: bool,
}

impl MateInfo {
    /// Tạo đối tượng `MateInfo` từ kết quả thẩm định
    pub fn new(pos: &Position, best: String, score: i32, depth: u8) -> Self {
        let fen = Serializer::export(pos);
        let mut mate = 0i8;
        let mut solved = false;

        if score >= 29000 {
            mate = (30000 - score).clamp(1, 127) as i8;
            solved = true;
        } else if score <= -29000 {
            mate = (-30000 - score).clamp(-127, -1) as i8;
            solved = true;
        }

        Self {
            fen,
            best,
            score,
            depth,
            mate,
            solved,
        }
    }
}

/// Struct `Hunter` quản lý việc săn sát cục và điều phối hàng đợi nhiệm vụ động hai tầng ưu tiên
pub struct Hunter {
    /// Độ sâu tối thiểu khi quét
    pub min_depth: u8,
    /// Độ sâu tối đa khi đào sâu săn sát cục (ví dụ Depth 28..32)
    pub max_depth: u8,
    /// Giới hạn sức chứa hàng đợi thông thường chống OOM
    pub limit: usize,
    /// Hàng đợi nhiệm vụ khẩn cấp (DeepenMate, ForcingCheck) - Được ưu tiên xử lý trước 100%
    pub urgent: VecDeque<Task>,
    /// Hàng đợi nhiệm vụ mở rộng thông thường (Explore)
    pub normal: VecDeque<Task>,
    /// Bộ lọc Bloom Filter chống trùng FEN
    pub sieve: Sieve,
    /// HyperLogLog ước lượng Cardinality
    pub hll: Counter,
    /// CRDT đếm số mẫu sát cục tìm thấy
    pub mates_found: PnCounter,
    /// CRDT đếm số task đã hoàn thành
    pub tasks_done: PnCounter,
    /// Circuit Breaker bảo vệ C FFI / GPU
    pub breaker: Breaker,
}

impl Hunter {
    /// Khởi tạo `Hunter` mới với độ sâu tối thiểu và tối đa
    pub fn new(min_depth: u8, max_depth: u8) -> Self {
        Self {
            min_depth,
            max_depth,
            limit: 50000,
            urgent: VecDeque::with_capacity(5000),
            normal: VecDeque::with_capacity(20000),
            sieve: Sieve::new(),
            hll: Counter::new(),
            mates_found: PnCounter::new(),
            tasks_done: PnCounter::new(),
            breaker: Breaker::new(),
        }
    }

    /// Đặt giới hạn sức chứa hàng đợi thông thường
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Đẩy một nhiệm vụ mới vào Task Queue với phân luồng ưu tiên tự động
    pub fn push_task(&mut self, task: Task) {
        match task {
            Task::DeepenMate { .. } | Task::ForcingCheck { .. } => {
                self.urgent.push_back(task);
            }
            Task::Explore { .. } => {
                if self.normal.len() < self.limit {
                    self.normal.push_back(task);
                }
            }
        }
    }

    /// Lấy nhiệm vụ tiếp theo từ hàng đợi (Ưu tiên tuyệt đối hàng đợi `urgent` trước `normal`)
    pub fn pop_task(&mut self) -> Option<Task> {
        if let Some(task) = self.urgent.pop_front() {
            return Some(task);
        }
        self.normal.pop_front()
    }

    /// Tổng số nhiệm vụ đang chờ xử lý
    pub fn pending(&self) -> usize {
        self.urgent.len() + self.normal.len()
    }

    /// Xóa sạch hàng đợi nhiệm vụ để chuẩn bị khảo sát thế cờ mới
    pub fn clear(&mut self) {
        self.urgent.clear();
        self.normal.clear();
    }

    /// Phân tích một thế cờ và tự động quyết định có đẩy sâu săn sát cục hay không
    pub fn analyze_and_branch(
        &mut self,
        pos: &Position,
        path: &[String],
        score: i32,
        current_depth: u8,
        ply: usize,
        max_ply: usize,
    ) -> Option<MateInfo> {
        self.tasks_done.add(1);
        self.hll.add(pos.hash);

        // 1. Nếu đã phát hiện điểm số Sát Cục (|score| >= 29000cp)
        if score.abs() >= 29000 {
            self.mates_found.add(1);
            let info = MateInfo::new(pos, String::new(), score, current_depth);
            return Some(info);
        }

        // 2. Nếu có ưu thế chiến thuật lớn (|score| >= 300cp) và chưa đạt max_depth
        // -> Tự động sinh Task đào sâu DeepenMate vào hàng đợi Urgent để truy sát ngay!
        if score.abs() >= 300 && current_depth < self.max_depth {
            let next_depth = (current_depth + 4).min(self.max_depth);
            self.push_task(Task::DeepenMate {
                board: *pos,
                path: path.to_vec(),
                depth: next_depth,
                target_mate: (self.max_depth - next_depth) / 2 + 2,
            });
        }

        // 3. Nếu chưa vượt quá max_ply, tiếp tục rẽ nhánh thế cờ con
        if ply < max_ply {
            let mut moves = List::new();
            let mut board_clone = *pos;
            legal::gen(&mut board_clone, &mut moves);

            for i in 0..moves.len() {
                let mv = moves.items[i];
                let mut next_board = *pos;
                next_board.apply(mv.from, mv.to);

                // Lọc trùng lặp qua Bloom Filter
                if self.sieve.contains(next_board.hash) {
                    continue;
                }
                self.sieve.push(next_board.hash);

                let mut next_path = path.to_vec();
                next_path.push(Format::encode(mv));

                // Nếu nước đi tạo chiếu bắt buộc (Check), ưu tiên đẩy ForcingCheck vào Urgent
                if next_board.check != 0 {
                    self.push_task(Task::ForcingCheck {
                        board: next_board,
                        path: next_path,
                        depth: (current_depth + 2).min(self.max_depth),
                    });
                } else {
                    self.push_task(Task::Explore {
                        board: next_board,
                        path: next_path,
                        depth: self.min_depth,
                        ply: ply + 1,
                    });
                }
            }
        }

        None
    }

    /// Lưu kết quả thế cờ kèm thông tin sát cục Mate-in-N vào Kho Tri Thức Vĩnh Cửu Vault
    pub fn save_to_vault(&self, vault: &Vault, pos: &Position, best_uci: &str, score: i32, depth: u8, mate: i8) {
        let bm = Format::decode(best_uci);
        if bm.valid() {
            vault.save_mate(pos, depth, bm, score, mate, 0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Parser;

    #[test]
    fn test_hunter_dual_priority_queue() {
        let mut hunter = Hunter::new(10, 20);
        let pos = Parser::parse(Parser::DEFAULT);

        // Đẩy 1 task Explore (normal) và 1 task DeepenMate (urgent)
        hunter.push_task(Task::Explore {
            board: pos,
            path: vec![],
            depth: 10,
            ply: 0,
        });

        hunter.push_task(Task::DeepenMate {
            board: pos,
            path: vec![],
            depth: 16,
            target_mate: 3,
        });

        assert_eq!(hunter.pending(), 2);

        // Pop task phải trả về task DeepenMate (urgent) trước!
        let first = hunter.pop_task().unwrap();
        match first {
            Task::DeepenMate { depth, .. } => assert_eq!(depth, 16),
            _ => panic!("Phải ưu tiên pop task DeepenMate trước!"),
        }

        // Pop task thứ hai là Explore
        let second = hunter.pop_task().unwrap();
        match second {
            Task::Explore { depth, .. } => assert_eq!(depth, 10),
            _ => panic!("Task thứ hai phải là Explore!"),
        }

        assert_eq!(hunter.pending(), 0);
    }

    #[test]
    fn test_mate_info_score_mapping() {
        let pos = Parser::parse(Parser::DEFAULT);

        // Mate in 1 ply (Score 29999)
        let info1 = MateInfo::new(&pos, "e2e4".to_string(), 29999, 16);
        assert_eq!(info1.mate, 1);
        assert!(info1.solved);

        // Mate in 3 plies (Score 29997)
        let info3 = MateInfo::new(&pos, "e2e4".to_string(), 29997, 16);
        assert_eq!(info3.mate, 3);
        assert!(info3.solved);

        // Bị chiếu bí sau 2 plies (Score -29998)
        let info_neg2 = MateInfo::new(&pos, "e2e4".to_string(), -29998, 16);
        assert_eq!(info_neg2.mate, -2);
        assert!(info_neg2.solved);

        // Thế cờ cân bằng (Score 50)
        let info_eq = MateInfo::new(&pos, "e2e4".to_string(), 50, 16);
        assert_eq!(info_eq.mate, 0);
        assert!(!info_eq.solved);
    }
}
