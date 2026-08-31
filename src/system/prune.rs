// ============================================================================
// MODULE PRUNE: HỆ THỐNG CẮT TỈA TÌM KIẾM SOTA (SEARCH PRUNING SYSTEM)
// ============================================================================
// `PruneSystem` chịu trách nhiệm:
// - Reverse Futility Pruning (RFP) tại các tầng nông.
// - Dynamic Null Move Pruning (NMP) cho các thế cờ an toàn.
// - ProbCut và Singular Extensions.
// ============================================================================

use crate::system::weights::Weights;

/// Struct `PruneSystem` quản lý toàn bộ thuật toán cắt tỉa tìm kiếm PVS
pub struct PruneSystem;

impl PruneSystem {
    /// Tính toán biên độ Reverse Futility Pruning (RFP)
    #[inline(always)]
    pub fn rfp_margin(depth: i32, improving: bool, weights: &Weights) -> i32 {
        let base = weights.prune.rfp_base * depth;
        if improving {
            base
        } else {
            base + 40
        }
    }

    /// Tính toán mức độ giảm tầng của Null Move Pruning (NMP)
    #[inline(always)]
    pub fn nmp_reduction(depth: i32, weights: &Weights) -> i32 {
        weights.prune.nmp_base + depth / 4
    }

    /// Tính toán biên độ Singular Extension
    #[inline(always)]
    pub fn singular_margin(depth: i32, weights: &Weights) -> i32 {
        weights.prune.singular_margin * depth
    }
}
