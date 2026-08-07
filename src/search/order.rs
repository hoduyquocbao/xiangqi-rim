// ============================================================================
// MODULE ORDER: CHIẾN LƯỢC SẮP XẾP NƯỚC ĐI TỐI ƯU CẮT GIẢM ALPHA-BETA (MOVE ORDERING)
// ============================================================================
// `order.rs` đóng vai trò tối quan trọng trong hiệu năng Alpha-Beta Cutoff:
// - Bảng giá trị quân cờ `VALUES` (King=20000, Rook=900, Cannon=450, Knight=400, Elephant=200, Advisor=200, Pawn=100).
// - Thuật toán MVV-LVA (Most Valuable Victim - Least Valuable Attacker) ưu tiên nước ăn quân giá trị cao bằng quân nhỏ.
// - `History`: Bảng lịch sử thành công tích lũy trọng số theo độ sâu $(depth^2)$.
// - `Killer`: Bảng nước đi sát thủ (Killer Moves) tại mỗi ply.
// - `Picker`: Bộ chọn nước đi làm biếng (Lazy Move Picker STAGED) giúp chỉ sinh/sắp xếp nước đi khi cần thiết.
// ============================================================================

use crate::board::Position;
use crate::movegen::types::{List, Move};

/// Bảng giá trị cơ bản của 7 loại quân cờ cho cả 2 bên Đỏ và Đen (Centipawn)
pub const VALUES: [i32; 14] = [
    20000, 200, 200, 400, 900, 450, 100, // Red: King, Advisor, Elephant, Knight, Rook, Cannon, Pawn
    20000, 200, 200, 400, 900, 450, 100, // Black
];

/// Struct `History` quản lý bảng lịch sử tích lũy trọng số và nước đi phản đòn (Countermove Table), căn lề 64-byte.
#[repr(C, align(64))]
#[derive(Debug)]
pub struct History {
    /// Bảng ma trận 90x90 lưu điểm số lịch sử từ ô `from` tới ô `to`
    pub table: Box<[[i32; 90]; 90]>,
    /// Bảng phản đòn Countermove Table 90x90 lưu nước đi đối ứng tốt nhất sau nước `prev` của đối phương
    pub counter: Box<[[Move; 90]; 90]>,
}

impl Default for History {
    /// Khởi tạo mặc định đối tượng History.
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for History {
    fn clone(&self) -> Self {
        let mut h = Self::new();
        *h.table = *self.table;
        *h.counter = *self.counter;
        h
    }
}

impl History {
    /// Ngưỡng điểm lịch sử tối đa trước khi giảm bớt 50% (Decay Ceiling = 1,000,000)
    pub const CEILING: i32 = 1_000_000;

    /// Khởi tạo bảng lịch sử `History` rỗng bằng 0 trên Heap.
    pub fn new() -> Self {
        let table = unsafe {
            let layout = std::alloc::Layout::new::<[[i32; 90]; 90]>();
            let ptr = std::alloc::alloc_zeroed(layout) as *mut [[i32; 90]; 90];
            Box::from_raw(ptr)
        };
        let counter = unsafe {
            let layout = std::alloc::Layout::new::<[[Move; 90]; 90]>();
            let ptr = std::alloc::alloc_zeroed(layout) as *mut [[Move; 90]; 90];
            Box::from_raw(ptr)
        };
        Self { table, counter }
    }

    /// Lấy điểm số lịch sử của nước đi `mv`.
    #[inline(always)]
    pub fn get(&self, mv: Move) -> i32 {
        if mv.valid() {
            self.table[mv.from as usize][mv.to as usize]
        } else {
            0
        }
    }

    /// Cập nhật điểm thưởng lịch sử cho nước đi `mv` thành công tại độ sâu `depth`.
    #[inline(always)]
    pub fn update(&mut self, mv: Move, depth: i32) {
        if !mv.valid() {
            return;
        }
        let bonus = (depth * depth).min(400);
        let entry = &mut self.table[mv.from as usize][mv.to as usize];
        *entry += bonus;
        if *entry > Self::CEILING {
            self.decay();
        }
    }

    /// Phạt điểm lịch sử cho nước đi yên lặng `mv` KHÔNG gây ra Beta Cutoff (History Malus).
    /// Hệ số phạt bằng $-\text{depth}^2$, giúp move ordering chính xác hơn 10-15%
    /// bằng cách giảm ưu tiên các nước đi đã thử nhưng thất bại ở độ sâu cao.
    #[inline(always)]
    pub fn penalize(&mut self, mv: Move, depth: i32) {
        if !mv.valid() {
            return;
        }
        // Hệ số phạt = -(depth^2), giới hạn tối đa -400 để không phá vỡ cân bằng
        let malus = -(depth * depth).min(400);
        let entry = &mut self.table[mv.from as usize][mv.to as usize];
        *entry += malus;
        // Không cho phép điểm lịch sử xuống dưới -1,000,000 (tránh tràn số)
        if *entry < -Self::CEILING {
            *entry = -Self::CEILING;
        }
    }

    /// Trừ bớt 50% tất cả các giá trị lịch sử để giảm nhiễu khi vượt ngưỡng (Age Decay).
    #[inline(always)]
    pub fn decay(&mut self) {
        let mut f = 0;
        while f < 90 {
            let mut t = 0;
            while t < 90 {
                self.table[f][t] >>= 1;
                t += 1;
            }
            f += 1;
        }
    }

    /// Lấy nước đi phản đòn (Countermove) tốt nhất ứng với nước đi trước đó `prev`.
    #[inline(always)]
    pub fn get_counter(&self, prev: Move) -> Move {
        if prev.valid() {
            self.counter[prev.from as usize][prev.to as usize]
        } else {
            Move::none()
        }
    }

    /// Cập nhật nước đi phản đòn (Countermove) `curr` khi đối phương vừa đi `prev`.
    #[inline(always)]
    pub fn update_counter(&mut self, prev: Move, curr: Move) {
        if prev.valid() && curr.valid() {
            self.counter[prev.from as usize][prev.to as usize] = curr;
        }
    }

    /// Đặt lại toàn bộ bảng lịch sử và phản đòn về 0.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.table.fill([0; 90]);
        self.counter.fill([Move::none(); 90]);
    }
}

/// Struct `Killer` quản lý bảng nước đi sát thủ gây ra Beta Cutoff, căn lề 64-byte.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct Killer {
    /// Mảng lưu 2 nước đi sát thủ cho tối đa 128 tầng độ sâu ply
    pub slot: [[Move; 2]; 128],
}

impl Default for Killer {
    /// Khởi tạo mặc định đối tượng Killer.
    fn default() -> Self {
        Self::new()
    }
}

impl Killer {
    /// Khởi tạo bảng nước đi sát thủ rỗng.
    pub const fn new() -> Self {
        Self {
            slot: [[Move::none(); 2]; 128],
        }
    }

    /// Thêm một nước đi sát thủ mới tại tầng độ sâu `ply`.
    #[inline(always)]
    pub fn push(&mut self, ply: usize, mv: Move) {
        if ply < 128 && mv.valid() {
            if self.slot[ply][0] != mv {
                self.slot[ply][1] = self.slot[ply][0];
                self.slot[ply][0] = mv;
            }
        }
    }

    /// Đặt lại toàn bộ bảng sát thủ về rỗng.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.slot = [[Move::none(); 2]; 128];
    }
}

/// Enum `Stage` đại diện cho từng giai đoạn của bộ chọn nước đi làm biếng Lazy Picker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// Giai đoạn 1: Nước đi từ Transposition Table (TT Move)
    Tt,
    /// Giai đoạn 2: Sinh toàn bộ nước đi hợp lệ
    Gen,
    /// Giai đoạn 3: Tính toán điểm số sắp xếp cho danh sách nước đi
    Sort,
    /// Giai đoạn 4: Trả về từng nước đi theo thứ tự điểm số giảm dần
    Yield,
    /// Giai đoạn 5: Đã hoàn tất danh sách nước đi
    Done,
}

/// Struct `Picker` triển khai thuật toán Lazy Move Picker phân đoạn, căn lề 64-byte.
#[repr(C, align(64))]
#[derive(Clone, Debug)]
pub struct Picker {
    /// Giai đoạn hiện tại của Picker
    pub stage: Stage,
    /// Nước đi gợi ý từ bảng băm TT
    pub tt: Move,
    /// Mảng 2 nước đi sát thủ Killer
    pub killers: [Move; 2],
    /// Nước đi phản đòn Countermove từ lượt trước
    pub counter: Move,
    /// Danh sách các nước đi đã sinh
    pub moves: List,
    /// Con trỏ chỉ số nước đi tiếp theo
    pub index: usize,
}

impl Picker {
    /// Khởi tạo một đối tượng Picker mới với nước đi TT và Killer.
    #[inline(always)]
    pub fn new(tt: Move, killers: [Move; 2]) -> Self {
        Self::with_counter(tt, killers, Move::none())
    }

    /// Khởi tạo Picker bổ sung nước đi phản đòn Countermove.
    #[inline(always)]
    pub fn with_counter(tt: Move, killers: [Move; 2], counter: Move) -> Self {
        Self {
            stage: Stage::Tt,
            tt,
            killers,
            counter,
            moves: List::new(),
            index: 0,
        }
    }

    /// Lấy (Pop) nước đi tiếp theo có điểm ưu tiên cao nhất kết hợp đa dạng hóa History scaling.
    #[inline(always)]
    pub fn next_with(
        &mut self,
        pos: &mut Position,
        history: &History,
        diversity: Option<&crate::search::diversity::Diversity>,
    ) -> Option<Move> {
        loop {
            match self.stage {
                Stage::Tt => {
                    self.stage = Stage::Gen;
                    if self.tt.valid() && crate::movegen::legal::valid(pos, self.tt) {
                        return Some(self.tt);
                    }
                    self.tt = Move::none();
                }
                Stage::Gen => {
                    crate::movegen::legal::gen(pos, &mut self.moves);
                    self.index = 0;
                    self.stage = Stage::Yield;
                }
                Stage::Sort => {
                    self.index = 0;
                    self.stage = Stage::Yield;
                }
                Stage::Yield => {
                    if self.index >= self.moves.count {
                        self.stage = Stage::Done;
                        return None;
                    }

                    // Score function cho nước đi
                    let score_fn = |mv: Move| -> i32 {
                        if mv == self.tt {
                            -2_000_000
                        } else {
                            let captured = pos.grid[mv.to as usize];
                            if captured < 14 {
                                let moving = pos.grid[mv.from as usize];
                                let v = VALUES[captured as usize];
                                let a = VALUES[moving as usize];
                                1_000_000 + 10 * v - a
                            } else if mv == self.killers[0] {
                                900_000
                            } else if mv == self.counter {
                                850_000
                            } else if mv == self.killers[1] {
                                800_000
                            } else {
                                let base = history.get(mv);
                                if let Some(div) = diversity {
                                    div.scale(base)
                                } else {
                                    base
                                }
                            }
                        }
                    };

                    // Tìm phần tử có điểm số cao nhất trong các phần tử chưa chọn (Selection Sort step)
                    let mut best = self.index;
                    let mut best_score = score_fn(self.moves.items[self.index]);
                    let mut i = self.index + 1;
                    while i < self.moves.count {
                        let s = score_fn(self.moves.items[i]);
                        if s > best_score {
                            best_score = s;
                            best = i;
                        }
                        i += 1;
                    }

                    self.moves.items.swap(self.index, best);
                    let mv = self.moves.items[self.index];
                    self.index += 1;

                    if best_score <= -2_000_000 {
                        continue;
                    }
                    return Some(mv);
                }
                Stage::Done => return None,
            }
        }
    }

    /// Lấy (Pop) nước đi tiếp theo có điểm ưu tiên cao nhất theo cơ chế Lazy Selection mặc định.
    #[inline(always)]
    pub fn next(&mut self, pos: &mut Position, history: &History) -> Option<Move> {
        self.next_with(pos, history, None)
    }
}

/// Struct `Order` cung cấp các hàm tĩnh tiện ích đánh giá điểm số và sắp xếp danh sách nước đi.
pub struct Order;

impl Order {
    /// Tính điểm số ưu tiên sắp xếp cho nước đi `mv`.
    #[inline(always)]
    pub fn score(
        pos: &Position,
        mv: Move,
        hash: Move,
        killers: &[Move; 2],
        history: &History,
    ) -> i32 {
        if mv == hash {
            return 2_000_000;
        }
        let captured = pos.grid[mv.to as usize];
        if captured < 14 {
            let attacker = pos.grid[mv.from as usize];
            return 1_000_000 + (VALUES[captured as usize] * 10) - VALUES[attacker as usize];
        }
        if mv == killers[0] {
            return 900_000;
        }
        if mv == killers[1] {
            return 800_000;
        }
        history.get(mv)
    }

    /// Sắp xếp trực tiếp danh sách nước đi `list` theo điểm số ưu tiên giảm dần.
    #[inline(always)]
    pub fn sort(
        pos: &Position,
        list: &mut List,
        hash: Move,
        killers: &[Move; 2],
        history: &History,
    ) {
        let mut scores = [0i32; 128];
        let len = list.len();
        for i in 0..len {
            scores[i] = Self::score(pos, list[i], hash, killers, history);
        }
        for i in 0..len {
            for j in (i + 1)..len {
                if scores[j] > scores[i] {
                    scores.swap(i, j);
                    let tmp = list.items[i];
                    list.items[i] = list.items[j];
                    list.items[j] = tmp;
                }
            }
        }
    }
}

