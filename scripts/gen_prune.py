import math

table = [[0]*64 for _ in range(64)]
for d in range(64):
    for s in range(64):
        if d >= 3 and s >= 3:
            table[d][s] = int(0.75 + math.log(d) * math.log(s) / 2.25)

rows = []
for d in range(64):
    row_str = ', '.join(f'{table[d][s]}' for s in range(64))
    rows.append(f'    [{row_str}], // Depth {d}')

table_code = '\n'.join(rows)

header = '''// ============================================================================
// MODULE PRUNE: CÔNG THỨC TOÁN HỌC CẮT TẢI CÂY TÌM KIẾM (PRUNING HEURISTICS)
// ============================================================================
// prune.rs tập hợp các công thức cắt tỉa cây cờ tối tân giúp giảm hàng triệu nút duyệt vô ích:
// - LMR_TABLE: Bảng tra cứu tĩnh Logarithmic Late Move Reduction O(1) không tốn chi phí CPU.
// - nmp(): Null Move Pruning reduction.
// - lmr(): Late Move Reduction tra cứu từ bảng tĩnh LMR_TABLE[depth][searched].
// - rfp(): Reverse Futility Pruning margin (depth * 80 centipawns).
// - futility(): Futility Pruning margin theo độ sâu (depth * 90 centipawns).
// ============================================================================

/// Bảng tra cứu tĩnh Logarithmic Late Move Reduction (LMR) cho độ sâu 0..63 và số nước đã duyệt 0..63
pub static LMR_TABLE: [[i32; 64]; 64] = [
'''

body = '''
];

/// Struct Prune chứa các hàm tĩnh tính toán biên độ cắt tỉa cây cờ (Pruning Margins).
pub struct Prune;

impl Prune {
    /// Tính mức giảm độ sâu Null Move Pruning (NMP Reduction): R = 3 + depth / 4.
    #[inline(always)]
    pub const fn nmp(depth: i32) -> i32 {
        3 + depth / 4
    }

    /// Tra cứu mức giảm độ sâu Late Move Reduction (LMR) từ bảng tĩnh O(1).
    #[inline(always)]
    pub fn lmr(depth: i32, index: usize) -> i32 {
        let d = (depth as usize).min(63);
        let s = index.min(63);
        unsafe { *LMR_TABLE.get_unchecked(d).get_unchecked(s) }
    }

    /// Tính biên độ cắt tỉa Reverse Futility Pruning (RFP Margin): depth * 80 Centipawns.
    #[inline(always)]
    pub const fn rfp(depth: i32) -> i32 {
        depth * 80
    }

    /// Tính biên độ cắt tỉa Futility Pruning Margin theo độ sâu: depth * 90 Centipawns.
    #[inline(always)]
    pub const fn futility(depth: i32) -> i32 {
        depth * 90
    }

    /// Tính độ sâu rút gọn cho ProbCut (Probability Cutoff): (depth - 4).max(1).
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

    /// Biên độ Singular Extension Margin: depth * 2 Centipawns.
    #[inline(always)]
    pub const fn singular_margin(depth: i32) -> i32 {
        depth * 2
    }
}
'''

with open('src/search/prune.rs', 'w', encoding='utf-8') as f:
    f.write(header + table_code + body)

print('Updated src/search/prune.rs successfully!')
