// ============================================================================
// MODULE EVAL: BỘ ĐÁNH GIÁ THẾ CỜ (NNUE NEURAL NETWORK & HCE FALLBACK)
// ============================================================================
// Module `eval` chịu trách nhiệm chấm điểm số thế cờ (Evaluation Score) theo centipawns:
// - Sử dụng Mạng nơ-ron nhân tạo NNUE (`HalfKAv2_hm` 65,536 đặc trưng vị trí).
// - Triển khai bộ tích lũy `Accumulator` cập nhật $O(1)$ gia tăng cực nhanh.
// - Tích hợp bộ dự phòng `HCE` (Hand-Crafted Evaluation) kết hợp máy trạng thái
//   `CircuitBreaker` tự động hạ cấp nếu NNUE gặp sự cố hoặc trả về giá trị vô lý.
// ============================================================================

/// Module con `accum` quản lý bộ tích lũy Accumulator O(1)
pub mod accum;
/// Module con `feature` quản lý trích xuất chỉ số đặc trưng vị trí cờ
pub mod feature;
/// Module con `hce` bộ đánh giá luật tĩnh HCE dự phòng
pub mod hce;
/// Module con `nnue` kiến trúc mạng nơ-ron NNUE 3 lớp (FeatureTransformer, AffineLayer, OutputLayer)
pub mod nnue;
/// Module con `weight` chứa bộ trọng số tĩnh mạng nơ-ron
pub mod weight;
/// Module con `trap` đánh giá bẫy cờ Mã nghẽn chân, Xe kẹt góc, Pháo mất ngòi
pub mod trap;
/// Module con `sieve` bộ lọc Bloom Filter O(1) chống trùng FEN thế cờ
pub mod sieve;
/// Module con `nnue_avx2` cập nhật vi phân Accumulator bằng SIMD AVX2 intrinsics
pub mod nnue_avx2;
/// Module con `hanging` đánh giá quân bị tấn công và quân không có người bảo vệ
pub mod hanging;

pub use accum::Accum;
pub use nnue_avx2::Accumulator as AccumulatorAvx2;
pub use feature::Feature;
pub use hanging::Hanging;
pub use hce::Hce;
pub use nnue::Nnue;
pub use sieve::Sieve;
pub use trap::Trap;
pub use weight::Weight;

use crate::board::Position;
use crate::circuit::{Breaker, Check};

/// Enum `Mode` cấu hình chế độ đánh giá:
/// - `Auto`: Tự động dùng NNUE, nếu lỗi tự nhảy về HCE (Default)
/// - `Nnue`: Ép buộc sử dụng NNUE
/// - `Hce`: Ép buộc sử dụng HCE tĩnh
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Auto,
    Nnue,
    Hce,
}

/// Struct `Eval` bọc bộ đánh giá thế cờ, căn lề bộ nhớ 64-byte (`#[repr(C, align(64))]`).
#[repr(C, align(64))]
#[derive(Clone, Debug)]
pub struct Eval {
    /// Bộ đánh giá thủ công HCE
    pub hce: Hce,
    /// Bộ đánh giá mạng nơ-ron NNUE
    pub nnue: Nnue,
    /// Bộ tích lũy đặc trưng Accumulator hiện tại
    pub accum: Accum,
    /// Chế độ đánh giá được chọn
    pub mode: Mode,
    /// Máy trạng thái ngắt mạch CircuitBreaker bảo vệ hệ thống
    pub circuit: Breaker,
}

impl Default for Eval {
    /// Khởi tạo mặc định đối tượng `Eval`.
    fn default() -> Self {
        Self::new()
    }
}

impl Eval {
    /// Khởi tạo đối tượng `Eval` mới.
    #[inline(always)]
    pub fn new() -> Self {
        let mut eval = Self {
            hce: Hce::new(),
            nnue: Nnue::new(),
            accum: Accum::new(),
            mode: Mode::Auto,
            circuit: Breaker::new(),
        };
        if std::path::Path::new("data/nnue_weights_gen9.bin").exists() {
            let _ = eval.load("data/nnue_weights_gen9.bin");
        } else if std::path::Path::new("data/nnue_weights.bin").exists() {
            let _ = eval.load("data/nnue_weights.bin");
        }
        eval
    }

    /// Nạp trọng số NNUE từ tệp nhị phân format XRNN.
    /// Tự động chuyển sang chế độ Auto (NNUE ưu tiên, HCE dự phòng).
    pub fn load(&mut self, path: &str) -> Result<(), String> {
        self.nnue.load(path)?;
        self.mode = Mode::Auto;
        Ok(())
    }

    /// Đặt chế độ đánh giá cho Engine.
    #[inline(always)]
    pub fn mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    /// Kiểm tra xem bộ tích lũy NNUE accumulator có cần thiết phải cập nhật hay không.
    #[inline(always)]
    pub fn enabled(&self) -> bool {
        match self.mode {
            Mode::Hce => false,
            Mode::Nnue | Mode::Auto => self.nnue.loaded,
        }
    }

    /// Đặt lại bộ tích lũy `Accumulator` từ đầu theo trạng thái bàn cờ `pos`.
    /// Ép buộc inlining `#[inline(always)]` loại bỏ hoàn toàn overhead gọi hàm trên hot path.
    #[inline(always)]
    pub fn reset(&mut self, pos: &Position) {
        self.accum.reset(pos, &self.nnue.weight);
    }

    /// Cập nhật bộ tích lũy `Accumulator` gia tăng $O(1)$ khi MakeMove.
    /// Ép buộc inlining `#[inline(always)]` triệt tiêu hoàn toàn chi phí tạo khung hàm (stack frame overhead).
    #[inline(always)]
    pub fn apply(
        &mut self,
        pos: &Position,
        from: u8,
        to: u8,
        moving: u8,
        captured: u8,
    ) {
        self.accum.apply(pos, from, to, moving, captured, &self.nnue.weight);
    }

    /// Khôi phục bộ tích lũy `Accumulator` gia tăng $O(1)$ khi UndoMove.
    /// Ép buộc inlining `#[inline(always)]` tối ưu hóa tốc độ hoàn tác nước đi.
    #[inline(always)]
    pub fn revert(
        &mut self,
        pos: &Position,
        from: u8,
        to: u8,
        moving: u8,
        captured: u8,
    ) {
        self.accum.revert(pos, from, to, moving, captured, &self.nnue.weight);
    }

    /// Tính điểm thế cờ hiện tại theo góc nhìn của bên nắm lượt đi (`pos.side`).
    /// Kết quả dương đại diện cho ưu thế của bên đi, âm đại diện cho yếu thế.
    /// Tối ưu hóa: Hợp nhất 2 nhánh Mode::Nnue và Mode::Auto thành 1 nhánh chung
    /// vì logic hoàn toàn giống nhau, loại bỏ duplicate code và giảm instruction cache pressure.
    #[inline(always)]
    pub fn score(&self, pos: &Position) -> i32 {
        // 0. Tích hợp Endgame Knowledge Base nhận diện các thế cờ tàn cuộc lý thuyết
        if let Some(score) = crate::book::Endgame::eval(pos) {
            return score;
        }

        let raw = match self.mode {

            Mode::Hce => self.hce.evaluate(pos),
            // Hợp nhất Mode::Nnue và Mode::Auto thành 1 nhánh duy nhất
            // Cả 2 đều thử NNUE trước, nếu thất bại thì fallback sang HCE
            Mode::Nnue | Mode::Auto => {
                if self.nnue.loaded && self.circuit.allow(0) {
                    let val = self.nnue.evaluate(&self.accum, pos.side);
                    let valid = Check::valid(val, -29999, 29999);
                    self.circuit.record(valid, 0);
                    if valid {
                        // Hiệu chỉnh chiến thuật tối hậu từ Trap, Hanging, Passed Pawns, King Safety, Skills & Combos
                        let (trap_m, trap_e) = crate::eval::trap::Trap::evaluate(pos);
                        let (hang_m, hang_e) = crate::eval::hanging::Hanging::evaluate(pos);
                        let (pawn_m, pawn_e) = crate::eval::hce::Pawn::evaluate(pos);
                        let (king_m, king_e) = crate::eval::hce::King::evaluate(pos);
                        let mut skill_m = 0;
                        let mut skill_e = 0;
                        let default_weights = crate::system::weights::Weights::default();
                        crate::system::Skill::evaluate(pos, &default_weights, &mut skill_m, &mut skill_e);
                        crate::system::Combo::evaluate(pos, &default_weights, &mut skill_m, &mut skill_e);

                        let phase = crate::eval::hce::Value::phase(pos);
                        let raw_bias = crate::eval::hce::Value::taper(
                            trap_m + hang_m + pawn_m + king_m + skill_m,
                            trap_e + hang_e + pawn_e + king_e + skill_e,
                            phase,
                        );
                        // Chuẩn hóa và kẹp biên an toàn: Giữ NNUE làm trí tuệ chủ đạo 80-85%,
                        // các luật bảo vệ và tổ hợp chiến thuật đóng vai trò rào chắn an toàn tối đa 400cp
                        let bounded_bias = (raw_bias / 2).clamp(-400, 400);
                        let side_sign = if pos.side == 0 { 1 } else { -1 };
                        let corrected = val + (bounded_bias * side_sign);
                        return corrected.clamp(-29999, 29999);
                    } else {
                        self.hce.evaluate(pos)
                    }
                } else {
                    self.hce.evaluate(pos)
                }
            }
        };

        // Chuyển đổi điểm số HCE thuần túy theo lượt đi (Red side 0: giữ nguyên, Black side 1: đổi dấu)
        if pos.side == 0 {
            raw
        } else {
            -raw
        }
    }
}

// ----------------------------------------------------------------------------
// KHU VỰC BÀI KIỂM THỬ ĐƠN VỊ (UNIT TESTS) CHO MODULE EVAL
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Parser;

    /// Kiểm thử điểm số vị trí ban đầu: Phải cân bằng gần 0 centipawns (|score| < 100).
    #[test]
    fn default() {
        let pos = Parser::parse(Parser::DEFAULT);
        let mut eval = Eval::new();
        eval.mode(Mode::Hce);
        eval.reset(&pos);
        let score = eval.score(&pos);

        assert!(
            score.abs() < 100,
            "Điểm vị trí ban đầu BẮT BUỘC phải cân bằng xấp xỉ 0 centipawns: {}",
            score
        );
    }

    /// Kiểm thử tính đổi dấu của góc nhìn đánh giá (Relative Evaluation Perspective):
    /// Điểm của Đỏ phải bằng đúng âm điểm của Đen khi đổi lượt đi ở chế độ HCE.
    #[test]
    fn perspective() {
        let mut pos = Parser::parse(Parser::DEFAULT);
        let mut eval = Eval::new();
        eval.mode(Mode::Hce);

        let red = eval.score(&pos);
        pos.side = 1;
        let black = eval.score(&pos);

        assert!(
            red == -black,
            "Điểm số BẮT BUỘC phải đảo dấu khi đổi phe nắm lượt đi!"
        );
    }

    /// Kiểm thử tính nhất quán giữa cập nhật gia tăng $O(1)$ và tính toán lại từ đầu của Accumulator.
    #[test]
    fn incremental() {
        let mut pos = Parser::parse(Parser::DEFAULT);
        let mut eval = Eval::new();

        eval.reset(&pos);
        let initial = eval.accum;

        let from = 64u8; // Pháo Đỏ ô b2
        let to = 67u8;   // Pháo Đỏ vào trung lộ e2
        let moving = pos.grid[from as usize];
        let captured = pos.grid[to as usize];

        eval.apply(&pos, from, to, moving, captured);
        let state = pos.apply(from, to);

        let mut fresh = Accum::new();
        fresh.reset(&pos, &eval.nnue.weight);

        assert!(
            eval.accum == fresh,
            "Cập nhật gia tăng Accumulator BẮT BUỘC trùng khớp 100% với reset!"
        );

        pos.revert(from, to, &state);
        eval.revert(&pos, from, to, moving, captured);
        assert!(
            eval.accum == initial,
            "Hoàn tác Accumulator BẮT BUỘC trùng khớp 100% với Accumulator ban đầu!"
        );
    }

    /// Kiểm thử tính chính xác của Accumulator khi Tướng di chuyển (gây reset lại Feature Transformer).
    #[test]
    fn check() {
        let mut pos = Parser::parse(Parser::DEFAULT);
        let mut eval = Eval::new();
        eval.reset(&pos);

        let moves = [
            (4u8, 13u8),  // Tướng Đỏ 4 -> 13
            (85u8, 76u8), // Tướng Đen 85 -> 76
            (13u8, 14u8), // Tướng Đỏ 13 -> 14
            (76u8, 75u8), // Tướng Đen 76 -> 75
            (14u8, 5u8),  // Tướng Đỏ 14 -> 5
            (75u8, 84u8), // Tướng Đen 75 -> 84
        ];

        let mut stack = Vec::new();

        for &(from, to) in &moves {
            let moving = pos.grid[from as usize];
            let captured = pos.grid[to as usize];
            let initial = eval.accum;

            eval.apply(&pos, from, to, moving, captured);

            let state = pos.apply(from, to);
            stack.push((from, to, moving, captured, state, initial));

            let mut fresh = Accum::new();
            fresh.reset(&pos, &eval.nnue.weight);

            assert!(
                eval.accum == fresh,
                "Accumulator sau nước đi Tướng ({from} -> {to}) BẮT BUỘC khớp với reset!",
            );
        }

        while let Some((from, to, moving, captured, state, expected)) = stack.pop() {
            pos.revert(from, to, &state);

            eval.revert(&pos, from, to, moving, captured);

            assert!(
                eval.accum == expected,
                "Accumulator sau hoàn tác BẮT BUỘC khớp với Accumulator dự kiến!",
            );
        }
    }

    /// Kiểm thử căn lề bộ nhớ SIMD 64-byte cho tất cả các cấu trúc đánh giá.
    #[test]
    fn alignments() {
        assert!(crate::simd::align::<Accum>() == 64);
        assert!(crate::simd::align::<Eval>() == 64);
        assert!(crate::simd::align::<nnue::Transform>() == 64);
        assert!(crate::simd::align::<nnue::Affine<512, 32>>() == 64);
        assert!(crate::simd::align::<nnue::Output<32>>() == 64);
        assert!(crate::simd::align::<Nnue>() == 64);
    }
}




