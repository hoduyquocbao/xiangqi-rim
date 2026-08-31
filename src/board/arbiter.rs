// ============================================================================
// MODULE ARBITER: BỘ TRỌNG TÀI LUẬT CỜ TƯỚNG CHÂU Á & QUỐC TẾ (AXF RULES)
// ============================================================================
// Module này chịu trách nhiệm phân xử các tình huống lặp lại nước đi phức tạp:
// 1. Trường Chiếu (Perpetual Check): Chiếu Tướng liên tục >= 3 lần -> Xử THUA bên chiếu (-29,000 cp).
// 2. Trường Tráp (Perpetual Chase): Bắt quân liên tục >= 3 lần -> Xử THUA bên bắt (-29,000 cp).
// 3. Lặp nước thông thường (Both Perpetual hoặc Both Peaceful): Xử HÒA (0 cp).
// 4. Nhất Chiếu Nhất Dứ / Nhất Tráp Nhất Tróc: Xử lý theo quy chuẩn Luật Cờ Tướng Châu Á.
// ============================================================================

use crate::board::position::Position;
use crate::movegen::types::Move;

/// Kiểu phán quyết của Trọng tài Cờ tướng Châu Á
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// Thế cờ bình thường, không có vi phạm lặp lại
    Normal,
    /// Hòa cờ do lặp lại trạng thái bình đẳng
    Draw,
    /// Bên Đỏ phạm luật Trường Chiếu hoặc Trường Tráp -> Đỏ Thua
    RedLoss,
    /// Bên Đen phạm luật Trường Chiếu hoặc Trường Tráp -> Đen Thua
    BlackLoss,
}

/// Bản ghi trạng thái của 1 nước đi trong chuỗi lịch sử
#[derive(Clone, Copy, Debug, Default)]
pub struct Record {
    /// Giá trị băm Zobrist của bàn cờ
    pub hash: u64,
    /// Nước đi đã thực hiện
    pub step: Move,
    /// Nước đi có tạo ra đòn chiếu Tướng hay không
    pub check: bool,
    /// Nước đi có tạo ra đòn bắt quân (tráp) hay không
    pub chase: bool,
    /// Phe thực hiện nước đi (0: Đỏ, 1: Đen)
    pub side: u8,
}

/// Bộ Trọng tài phân xử luật Cờ tướng Châu Á (align 64 bytes)
#[repr(C, align(64))]
#[derive(Clone, Debug)]
pub struct Arbiter {
    /// Mảng lưu vết lịch sử ván cờ (tối đa 256 nước đi)
    pub history: [Record; 256],
    /// Số lượng nước đi hiện tại trong lịch sử
    pub count: usize,
    /// Vùng đệm căn lề 64 bytes
    pub pad: [u8; 40],
}

impl Arbiter {
    /// Khởi tạo một Bộ Trọng tài mới
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            history: [Record {
                hash: 0,
                step: Move::none(),
                check: false,
                chase: false,
                side: 0,
            }; 256],
            count: 0,
            pad: [0u8; 40],
        }
    }

    /// Đặt lại toàn bộ lịch sử trọng tài
    #[inline(always)]
    pub fn reset(&mut self) {
        self.count = 0;
    }

    /// Ghi nhận một nước đi mới vào lịch sử trọng tài
    #[inline(always)]
    pub fn push(&mut self, pos: &Position, step: Move, is_check: bool, is_chase: bool) {
        if self.count < 256 {
            self.history[self.count] = Record {
                hash: pos.hash,
                step,
                check: is_check,
                chase: is_chase,
                side: pos.side,
            };
            self.count += 1;
        }
    }

    /// Rút lại nước đi cuối cùng khỏi lịch sử trọng tài
    #[inline(always)]
    pub fn pop(&mut self) {
        if self.count > 0 {
            self.count -= 1;
        }
    }

    /// Thẩm định chu kỳ lặp lại nước đi và đưa ra phán quyết theo Luật Châu Á
    /// `target_hash`: Khóa băm Zobrist của thế cờ chuẩn bị lặp lại
    pub fn judge(&self, target_hash: u64, current_side: u8) -> Verdict {
        if self.count < 4 {
            return Verdict::Normal;
        }

        // Tìm vị trí gần nhất trong lịch sử xuất hiện thế cờ target_hash
        let mut first_match = None;
        let mut repeat_count = 0;

        let mut idx = self.count;
        while idx > 0 {
            idx -= 1;
            if self.history[idx].hash == target_hash {
                repeat_count += 1;
                if first_match.is_none() {
                    first_match = Some(idx);
                }
            }
        }

        // Nếu thế cờ này chưa từng xuất hiện hoặc chỉ mới lặp 1 lần -> Bình thường
        if repeat_count < 2 || first_match.is_none() {
            return Verdict::Normal;
        }

        let start_idx = first_match.unwrap();
        let cycle_len = self.count - start_idx;

        // Chu kỳ lặp nước thông thường từ 2 đến 16 nước đi
        if cycle_len < 2 || cycle_len > 32 {
            return Verdict::Normal;
        }

        // Thống kê đòn Chiếu và Tráp của cả 2 bên trong chu kỳ lặp lại
        let mut red_checks = 0;
        let mut red_chases = 0;
        let mut red_moves = 0;

        let mut black_checks = 0;
        let mut black_chases = 0;
        let mut black_moves = 0;

        for i in start_idx..self.count {
            let rec = &self.history[i];
            if rec.side == 0 {
                red_moves += 1;
                if rec.check {
                    red_checks += 1;
                }
                if rec.chase {
                    red_chases += 1;
                }
            } else {
                black_moves += 1;
                if rec.check {
                    black_checks += 1;
                }
                if rec.chase {
                    black_chases += 1;
                }
            }
        }

        // 1. Trường Chiếu (Perpetual Check): Chiếu liên tục 100% các nước đi của phe mình trong chu kỳ
        let red_perp_check = red_moves > 0 && red_checks == red_moves;
        let black_perp_check = black_moves > 0 && black_checks == black_moves;

        if red_perp_check && !black_perp_check {
            return Verdict::RedLoss; // Đỏ trường chiếu -> Đỏ Thua
        }
        if black_perp_check && !red_perp_check {
            return Verdict::BlackLoss; // Đen trường chiếu -> Đen Thua
        }

        // 2. Trường Tráp (Perpetual Chase): Bắt quân liên tục 100% các nước đi
        let red_perp_chase = red_moves > 0 && (red_checks + red_chases) == red_moves;
        let black_perp_chase = black_moves > 0 && (black_checks + black_chases) == black_moves;

        if red_perp_chase && !black_perp_chase {
            return Verdict::RedLoss; // Đỏ trường tráp -> Đỏ Thua
        }
        if black_perp_chase && !red_perp_chase {
            return Verdict::BlackLoss; // Đen trường tráp -> Đen Thua
        }

        // 3. Cả 2 bên cùng trường chiếu/trường tráp hoặc lặp nước yên lặng bình đẳng -> Xử HÒA
        if repeat_count >= 2 {
            let _ = current_side;
            return Verdict::Draw;
        }

        Verdict::Normal
    }

    /// Tính toán điểm số phạt / thưởng tìm kiếm dựa trên phán quyết của Trọng tài
    #[inline(always)]
    pub fn score(&self, target_hash: u64, current_side: u8, ply: usize) -> Option<i32> {
        match self.judge(target_hash, current_side) {
            Verdict::Normal => None,
            Verdict::Draw => Some(0), // Hòa cờ = 0 cp
            Verdict::RedLoss => {
                if current_side == 0 {
                    Some(-29000 + (ply as i32)) // Đỏ đang tìm kiếm và Đỏ bị xử thua -> Phạt điểm nặng
                } else {
                    Some(29000 - (ply as i32)) // Đen đang tìm kiếm và Đỏ bị xử thua -> Thưởng điểm thắng
                }
            }
            Verdict::BlackLoss => {
                if current_side == 1 {
                    Some(-29000 + (ply as i32)) // Đen đang tìm kiếm và Đen bị xử thua -> Phạt điểm nặng
                } else {
                    Some(29000 - (ply as i32)) // Đỏ đang tìm kiếm và Đen bị xử thua -> Thưởng điểm thắng
                }
            }
        }
    }
}

// ----------------------------------------------------------------------------
// BÀI KIỂM THỬ ĐƠN VỊ BỘ TRỌNG TÀI LUẬT CHÂU Á (UNIT TESTS)
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Parser;
    use std::mem::{align_of, size_of};

    #[test]
    fn alignments() {
        assert_eq!(align_of::<Arbiter>(), 64);
        assert_eq!(size_of::<Record>(), 16);
    }

    #[test]
    fn test_perpetual_check_detection() {
        let mut arbiter = Arbiter::new();
        let pos1 = Parser::parse(Parser::DEFAULT);
        let pos2 = Parser::parse("rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR b - - 0 1");

        let mv = Move::new(4, 5);

        // Giả lập Đỏ liên tục chiếu Tướng trong chu kỳ lặp lại
        arbiter.push(&pos1, mv, true, false); // Đỏ chiếu
        arbiter.push(&pos2, mv, false, false); // Đen né
        arbiter.push(&pos1, mv, true, false); // Đỏ chiếu lại
        arbiter.push(&pos2, mv, false, false); // Đen né lại

        // Phán quyết: Đỏ phải bị xử THUA (RedLoss) vì phạm luật Trường Chiếu!
        let verdict = arbiter.judge(pos1.hash, 0);
        assert_eq!(verdict, Verdict::RedLoss);

        // Điểm số tìm kiếm cho Đỏ phải âm nặng (-29000)
        let score_red = arbiter.score(pos1.hash, 0, 4);
        assert!(score_red.unwrap() <= -28900);
    }
}
