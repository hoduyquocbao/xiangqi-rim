// ============================================================================
// MODULE PRUNE: CÔNG THỨC TOÁN HỌC CẮT TẢI CÂY TÌM KIẾM (PRUNING HEURISTICS)
// ============================================================================
// `prune.rs` tập hợp các công thức cắt tỉa cây cờ tối tân giúp giảm hàng triệu nút duyệt vô ích:
// - `nmp()`: Null Move Pruning reduction $(R = 3 + \lfloor depth / 4 \rfloor)$.
// - `lmr()`: Late Move Reduction formula $(R = 1 + \lfloor depth / 4 \rfloor + \lfloor index / 8 \rfloor)$.
// - `rfp()`: Reverse Futility Pruning margin $(depth \times 120$ centipawns).
// - `futility()`: Futility Pruning margin theo độ sâu (depth 1: 150, depth 2: 300).
// ============================================================================

/// Struct `Prune` chứa các hàm tĩnh tính toán biên độ cắt tỉa cây cờ (Pruning Margins).
pub struct Prune;

impl Prune {
    /// Tính mức giảm độ sâu Null Move Pruning (NMP Reduction): $R = 3 + \lfloor depth / 4 \rfloor$.
    #[inline(always)]
    pub const fn nmp(depth: i32) -> i32 {
        3 + depth / 4
    }

    /// Tính mức giảm độ sâu Late Move Reduction (LMR Reduction): $R = 1 + \lfloor depth / 4 \rfloor + \lfloor index / 8 \rfloor$.
    /// Không áp dụng nếu độ sâu < 3 hoặc chỉ số nước đi < 3.
    #[inline(always)]
    pub const fn lmr(depth: i32, index: usize) -> i32 {
        if depth < 3 || index < 3 {
            0
        } else {
            1 + (depth / 4) + (index as i32 / 8)
        }
    }

    /// Tính biên độ cắt tỉa Reverse Futility Pruning (RFP Margin): $depth \times 120$ Centipawns.
    #[inline(always)]
    pub const fn rfp(depth: i32) -> i32 {
        depth * 120
    }

    /// Tính biên độ cắt tỉa Futility Pruning Margin theo độ sâu (Depth 1: 150 cp, Depth 2: 300 cp).
    #[inline(always)]
    pub const fn futility(depth: i32) -> i32 {
        match depth {
            1 => 150,
            2 => 300,
            _ => 0,
        }
    }

    /// Tính độ sâu rút gọn cho ProbCut (Probability Cutoff): $(depth - 4).max(1)$.
    #[inline(always)]
    pub const fn probcut_depth(depth: i32) -> i32 {
        if depth - 4 > 1 {
            depth - 4
        } else {
            1
        }
    }

    /// Biên độ nới rộng ProbCut Margin: 200 Centipawns.
    #[inline(always)]
    pub const fn probcut_margin() -> i32 {
        200
    }

    /// Biên độ Singular Extension Margin: $depth \times 2$ Centipawns.
    #[inline(always)]
    pub const fn singular_margin(depth: i32) -> i32 {
        depth * 2
    }
}


