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

/// Bảng giá trị cơ bản của 7 loại quân cờ cho cả 2 bên Đỏ và Đen cùng ô trống (Centipawn)
pub const VALUES: [i32; 15] = [
    20000, 200, 200, 400, 900, 450, 100, // Red: King, Advisor, Elephant, Knight, Rook, Cannon, Pawn
    20000, 200, 200, 400, 900, 450, 100, // Black: King, Advisor, Elephant, Knight, Rook, Cannon, Pawn
    0,                                    // Empty
];

/// Struct `History` quản lý bảng lịch sử tích lũy trọng số, phản đòn và tiếp diễn (Continuation History), căn lề 64-byte.
#[repr(C, align(64))]
#[derive(Debug)]
pub struct History {
    /// Bảng ma trận 90x90 lưu điểm số lịch sử từ ô `from` tới ô `to`
    pub table: Box<[[i32; 90]; 90]>,
    /// Bảng phản đòn Countermove Table 90x90 lưu nước đi đối ứng tốt nhất sau nước `prev` của đối phương
    pub counter: Box<[[Move; 90]; 90]>,
    /// Bảng lịch sử tiếp diễn Continuation History: 14 loại quân x 90 ô trước x 90 ô nay
    pub follow: Box<[[[i32; 90]; 90]; 14]>,
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
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.table.as_ptr() as *const u8,
                h.table.as_mut_ptr() as *mut u8,
                std::mem::size_of::<[[i32; 90]; 90]>(),
            );
            std::ptr::copy_nonoverlapping(
                self.counter.as_ptr() as *const u8,
                h.counter.as_mut_ptr() as *mut u8,
                std::mem::size_of::<[[Move; 90]; 90]>(),
            );
            std::ptr::copy_nonoverlapping(
                self.follow.as_ptr() as *const u8,
                h.follow.as_mut_ptr() as *mut u8,
                std::mem::size_of::<[[[i32; 90]; 90]; 14]>(),
            );
        }
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
        let follow = unsafe {
            let layout = std::alloc::Layout::new::<[[[i32; 90]; 90]; 14]>();
            let ptr = std::alloc::alloc_zeroed(layout) as *mut [[[i32; 90]; 90]; 14];
            Box::from_raw(ptr)
        };
        Self { table, counter, follow }
    }

    /// Lấy điểm số lịch sử của nước đi `mv`.
    #[inline(always)]
    pub fn get(&self, mv: Move) -> i32 {
        if mv.valid() {
            unsafe { *self.table.get_unchecked(mv.from as usize).get_unchecked(mv.to as usize) }
        } else {
            0
        }
    }

    /// Cập nhật điểm thưởng lịch sử cho nước đi `mv` thành công tại độ sâu `depth` (Smooth History Gravity).
    #[inline(always)]
    pub fn update(&mut self, mv: Move, depth: i32) {
        if !mv.valid() {
            return;
        }
        let bonus = (depth * depth * 32).min(2000);
        let entry = unsafe { self.table.get_unchecked_mut(mv.from as usize).get_unchecked_mut(mv.to as usize) };
        *entry += bonus - (*entry * bonus.abs()) / 16384;
    }

    /// Phạt điểm lịch sử cho nước đi yên lặng `mv` KHÔNG gây ra Beta Cutoff (History Malus Gravity).
    #[inline(always)]
    pub fn penalize(&mut self, mv: Move, depth: i32) {
        if !mv.valid() {
            return;
        }
        let malus = (depth * depth * 32).min(2000);
        let entry = unsafe { self.table.get_unchecked_mut(mv.from as usize).get_unchecked_mut(mv.to as usize) };
        *entry -= malus + (*entry * malus) / 16384;
    }

    /// Phạt điểm lịch sử hàng loạt cho danh sách các nước đi yên lặng thất bại `quiet` (History Malus Batch).
    #[inline(always)]
    pub fn penalize_batch(&mut self, quiet: &[Move], depth: i32) {
        if quiet.is_empty() {
            return;
        }
        let malus = (depth * depth * 32).min(2000);
        for &mv in quiet {
            if mv.valid() {
                let entry = unsafe { self.table.get_unchecked_mut(mv.from as usize).get_unchecked_mut(mv.to as usize) };
                *entry -= malus + (*entry * malus) / 16384;
            }
        }
    }

    /// Trừ bớt 50% tất cả các giá trị lịch sử để giảm nhiễu khi vượt ngưỡng (Age Decay).
    /// Sử dụng lặp phẳng 8,100 phần tử mở rộng 4-way cho phép CPU tự động SIMD hóa (AVX2/NEON).
    #[inline(always)]
    pub fn decay(&mut self) {
        let flat: &mut [i32; 8100] = unsafe { &mut *(self.table.as_mut_ptr() as *mut [i32; 8100]) };
        let mut i = 0;
        while i < 8100 {
            flat[i] >>= 1;
            flat[i + 1] >>= 1;
            flat[i + 2] >>= 1;
            flat[i + 3] >>= 1;
            i += 4;
        }
        let flat_f: &mut [i32; 113400] = unsafe { &mut *(self.follow.as_mut_ptr() as *mut [i32; 113400]) };
        let mut f = 0;
        while f < 113400 {
            flat_f[f] >>= 1;
            flat_f[f + 1] >>= 1;
            flat_f[f + 2] >>= 1;
            flat_f[f + 3] >>= 1;
            f += 4;
        }
    }

    /// Lấy nước đi phản đòn (Countermove) tốt nhất ứng với nước đi trước đó `prev`.
    #[inline(always)]
    pub fn get_counter(&self, prev: Move) -> Move {
        if prev.valid() {
            unsafe { *self.counter.get_unchecked(prev.from as usize).get_unchecked(prev.to as usize) }
        } else {
            Move::none()
        }
    }

    /// Cập nhật nước đi phản đòn (Countermove) `curr` khi đối phương vừa đi `prev`.
    #[inline(always)]
    pub fn update_counter(&mut self, prev: Move, curr: Move) {
        if prev.valid() && curr.valid() {
            unsafe { *self.counter.get_unchecked_mut(prev.from as usize).get_unchecked_mut(prev.to as usize) = curr; }
        }
    }

    /// Lấy điểm số Continuation History của nước đi `curr` khi quân `piece` đi tiếp nối sau nước `prev`.
    #[inline(always)]
    pub fn get_follow(&self, piece: u8, prev: Move, curr: Move) -> i32 {
        if (piece as usize) < 14 && prev.valid() && curr.valid() {
            unsafe {
                *self
                    .follow
                    .get_unchecked(piece as usize)
                    .get_unchecked(prev.to as usize)
                    .get_unchecked(curr.to as usize)
            }
        } else {
            0
        }
    }

    /// Cập nhật điểm thưởng Continuation History khi nước đi `curr` gây ra Cutoff sau nước `prev` (Smooth Gravity).
    #[inline(always)]
    pub fn update_follow(&mut self, piece: u8, prev: Move, curr: Move, depth: i32) {
        if (piece as usize) < 14 && prev.valid() && curr.valid() {
            let bonus = (depth * depth * 16).min(1000);
            let entry = unsafe {
                self.follow
                    .get_unchecked_mut(piece as usize)
                    .get_unchecked_mut(prev.to as usize)
                    .get_unchecked_mut(curr.to as usize)
            };
            *entry += bonus - (*entry * bonus.abs()) / 16384;
        }
    }

    /// Phạt điểm Continuation History khi nước đi `curr` thất bại sau nước `prev` (Smooth Gravity).
    #[inline(always)]
    pub fn penalize_follow(&mut self, piece: u8, prev: Move, curr: Move, depth: i32) {
        if (piece as usize) < 14 && prev.valid() && curr.valid() {
            let malus = (depth * depth * 16).min(1000);
            let entry = unsafe {
                self.follow
                    .get_unchecked_mut(piece as usize)
                    .get_unchecked_mut(prev.to as usize)
                    .get_unchecked_mut(curr.to as usize)
            };
            *entry -= malus + (*entry * malus) / 16384;
        }
    }

    /// Đặt lại toàn bộ bảng lịch sử, phản đòn và tiếp diễn về 0.
    #[inline(always)]
    pub fn clear(&mut self) {
        let flat_t: &mut [i32; 8100] = unsafe { &mut *(self.table.as_mut_ptr() as *mut [i32; 8100]) };
        flat_t.fill(0);
        let flat_c: &mut [Move; 8100] = unsafe { &mut *(self.counter.as_mut_ptr() as *mut [Move; 8100]) };
        flat_c.fill(Move::none());
        let flat_f: &mut [i32; 113400] = unsafe { &mut *(self.follow.as_mut_ptr() as *mut [i32; 113400]) };
        flat_f.fill(0);
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

/// Enum `Stage` đại diện cho từng giai đoạn của bộ chọn nước đi làm biếng Lazy Staged Picker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// Giai đoạn 1: Nước đi từ Transposition Table (TT Move)
    Tt,
    /// Giai đoạn 2: Sinh danh sách các nước ăn quân (Captures Only)
    CapturesGen,
    /// Giai đoạn 3: Duyệt và trả về từng nước ăn quân thắng thế (Good Captures SEE >= 0)
    CapturesYield,
    /// Giai đoạn 4: Sinh danh sách các nước yên lặng (Quiet Moves Only)
    QuietGen,
    /// Giai đoạn 5: Duyệt và trả về từng nước yên lặng (Killers/Counter/History)
    QuietYield,
    /// Giai đoạn 6: Duyệt và trả về các nước ăn quân thua thế còn lại (Bad Captures SEE < 0)
    BadCapturesYield,
    /// Giai đoạn 7: Đã hoàn tất danh sách nước đi
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
    /// Nước đi trước đó của đối phương để tra cứu Continuation History
    pub prev: Move,
    /// Danh sách các nước đi đã sinh
    pub moves: List,
    /// Mảng đệm điểm số tiền tính toán O(N) cho từng nước đi
    pub scores: [i32; 128],
    /// Con trỏ chỉ số nước đi tiếp theo
    pub index: usize,
    /// Mảng đệm lưu các nước ăn quân thua thế (Bad Captures)
    pub bads: [Move; 32],
    /// Số lượng nước ăn quân thua thế
    pub bad_count: usize,
    /// Con trỏ duyệt nước ăn quân thua thế
    pub bad_index: usize,
}

impl Picker {
    /// Khởi tạo một đối tượng Picker mới với nước đi TT và Killer.
    #[inline(always)]
    pub fn new(tt: Move, killers: [Move; 2]) -> Self {
        Self::with_context(tt, killers, Move::none(), Move::none())
    }

    /// Khởi tạo Picker bổ sung nước đi phản đòn Countermove.
    #[inline(always)]
    pub fn with_counter(tt: Move, killers: [Move; 2], counter: Move) -> Self {
        Self::with_context(tt, killers, counter, Move::none())
    }

    /// Khởi tạo Picker đầy đủ ngữ cảnh bao gồm cả Countermove và nước đi trước `prev`.
    #[inline(always)]
    pub fn with_context(tt: Move, killers: [Move; 2], counter: Move, prev: Move) -> Self {
        Self {
            stage: Stage::Tt,
            tt,
            killers,
            counter,
            prev,
            moves: List::new(),
            scores: [0; 128],
            index: 0,
            bads: [Move::none(); 32],
            bad_count: 0,
            bad_index: 0,
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
                    self.stage = Stage::CapturesGen;
                    if self.tt.valid() && crate::movegen::legal::valid(pos, self.tt) {
                        return Some(self.tt);
                    }
                    self.tt = Move::none();
                }
                Stage::CapturesGen => {
                    crate::movegen::pseudo::captures(pos, &mut self.moves);
                    self.index = 0;
                    self.bad_count = 0;
                    self.bad_index = 0;
                    let mut write = 0;
                    let mut i = 0;
                    while i < self.moves.count {
                        let mv = self.moves.items[i];
                        if mv == self.tt {
                            i += 1;
                            continue;
                        }
                        let captured = pos.grid[mv.to as usize];
                        let moving = pos.grid[mv.from as usize];
                        let v = if (captured as usize) < 14 { VALUES[captured as usize] } else { 0 };
                        let a = if (moving as usize) < 14 { VALUES[moving as usize] } else { 0 };

                        // Tối ưu hóa phân loại SEE: Nếu quân bị ăn có giá trị >= quân tấn công (v >= a),
                        // nước ăn quân hiển nhiên là Good Capture mà không cần gọi hàm See::evaluate.
                        let is_good = if v >= a {
                            true
                        } else {
                            crate::search::see::See::evaluate(pos, mv, 0)
                        };

                        if is_good {
                            self.moves.items[write] = mv;
                            let simplify_bonus = if v >= 45 && a >= 45 { 50_000 } else { 0 };
                            self.scores[write] = 1_000_000 + 10 * v - a + simplify_bonus;
                            write += 1;
                        } else if self.bad_count < 32 {
                            self.bads[self.bad_count] = mv;
                            self.bad_count += 1;
                        }
                        i += 1;
                    }
                    self.moves.count = write;
                    self.stage = Stage::CapturesYield;
                }
                Stage::CapturesYield => {
                    if self.index >= self.moves.count {
                        self.stage = Stage::QuietGen;
                        continue;
                    }

                    let mut best = self.index;
                    let mut best_score = self.scores[self.index];
                    let mut i = self.index + 1;
                    while i < self.moves.count {
                        let s = self.scores[i];
                        if s > best_score {
                            best_score = s;
                            best = i;
                        }
                        i += 1;
                    }

                    if best != self.index {
                        self.moves.items.swap(self.index, best);
                        self.scores.swap(self.index, best);
                    }

                    let mv = self.moves.items[self.index];
                    self.index += 1;
                    return Some(mv);
                }
                Stage::QuietGen => {
                    crate::movegen::pseudo::quiets(pos, &mut self.moves);
                    self.index = 0;
                    let mut write = 0;
                    let mut i = 0;
                    while i < self.moves.count {
                        let mv = self.moves.items[i];
                        if mv == self.tt {
                            i += 1;
                            continue;
                        }
                        self.moves.items[write] = mv;
                        let moving = pos.grid[mv.from as usize];
                        self.scores[write] = if mv == self.killers[0] {
                            900_000
                        } else if mv == self.counter {
                            850_000
                        } else if mv == self.killers[1] {
                            800_000
                        } else {
                            let base = history.get(mv);
                            let follow = history.get_follow(moving, self.prev, mv);
                            let mut total = base + follow;

                            // 1. Phạt nước đi Tướng vô cớ
                            if moving == 0 || moving == 7 {
                                total -= 50_000;
                            }

                            // 2. Thưởng Mã tiến công sang trận địa đối phương (Knight Advance & Infiltration Bonus)
                            let to_rank = mv.to / 9;
                            let to_file = mv.to % 9;
                            if moving == 3 {
                                // Mã Đỏ qua sông
                                if to_rank >= 5 { total += 15_000; }
                                if to_rank >= 7 && to_file >= 2 && to_file <= 6 { total += 35_000; }
                            } else if moving == 10 {
                                // Mã Đen qua sông
                                if to_rank <= 4 { total += 15_000; }
                                if to_rank <= 2 && to_file >= 2 && to_file <= 6 { total += 35_000; }
                            }

                            // 3. Thưởng Xe chiếm giữ Trung Lộ (Cột 4) và phạt Xe bỏ sang biên (Cột 0, 8)
                            if moving == 4 || moving == 11 {
                                if to_file == 4 {
                                    total += 25_000;
                                } else if to_file == 3 || to_file == 5 {
                                    total += 12_000;
                                } else if to_file == 0 || to_file == 8 {
                                    total -= 30_000;
                                }
                            }

                            if let Some(div) = diversity {
                                div.scale(total)
                            } else {
                                total
                            }
                        };
                        write += 1;
                        i += 1;
                    }
                    self.moves.count = write;
                    self.stage = Stage::QuietYield;
                }
                Stage::QuietYield => {
                    if self.index >= self.moves.count {
                        self.stage = Stage::BadCapturesYield;
                        continue;
                    }

                    let mut best = self.index;
                    let mut best_score = self.scores[self.index];
                    let mut i = self.index + 1;
                    while i < self.moves.count {
                        let s = self.scores[i];
                        if s > best_score {
                            best_score = s;
                            best = i;
                        }
                        i += 1;
                    }

                    if best != self.index {
                        self.moves.items.swap(self.index, best);
                        self.scores.swap(self.index, best);
                    }

                    let mv = self.moves.items[self.index];
                    self.index += 1;
                    return Some(mv);
                }
                Stage::BadCapturesYield => {
                    if self.bad_index >= self.bad_count {
                        self.stage = Stage::Done;
                        return None;
                    }
                    let mv = self.bads[self.bad_index];
                    self.bad_index += 1;
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

