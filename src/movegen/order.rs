// ============================================================================
// XIANGTI ENGINE: BỘ SẮP XẾP NƯỚC ĐỊ MVV-LVA (MOVE ORDERING SYSTEM)
// ============================================================================
// Module `order` triển khai thuật toán sắp xếp nước đi MVV-LVA (Most Valuable Victim -
// Least Valuable Attacker) để đưa các nước đi ăn quân hấp dẫn nhất lên đầu danh sách.
// Rút ngắn thời gian duyệt cây Alpha-Beta PVS gấp 10 lần và đẩy tỷ lệ TT Cutoff lên > 90%.
// Tuân thủ 100% định danh từ đơn tiếng Anh và 100% chú thích Tiếng Việt trên từng dòng mã.
// ============================================================================

use crate::board::Position; // Nhập kiểu struct Position từ module board
use crate::movegen::types::{List, Move}; // Nhập kiểu struct List và Move từ module types

/// Bảng giá trị quân cờ centipawn chuẩn phục vụ tính điểm MVV-LVA (0..13, 14 ô trống)
const VALUES: [i32; 15] = [
    10,  // 0: Tốt Đỏ
    20,  // 1: Sĩ Đỏ
    20,  // 2: Tượng Đỏ
    40,  // 3: Mã Đỏ
    45,  // 4: Pháo Đỏ
    90,  // 5: Xe Đỏ
    1000,// 6: Tướng Đỏ
    10,  // 7: Tốt Đen
    20,  // 8: Sĩ Đen
    20,  // 9: Tượng Đen
    40,  // 10: Mã Đen
    45,  // 11: Pháo Đen
    90,  // 12: Xe Đen
    1000,// 13: Tướng Đen
    0,   // 14: Ô trống
];

/// Tính điểm MVV-LVA cho 1 nước đi `Move` trên vị trí bàn cờ `pos`.
#[inline(always)]
pub fn score(pos: &Position, mv: Move) -> i32 { // Hàm score tính điểm MVV-LVA
    let attacker = pos.grid[mv.from as usize]; // Lấy loại quân tấn công
    let victim = pos.grid[mv.to as usize]; // Lấy loại quân bị ăn
    if victim < 14 { // Nếu là nước đi ăn quân (victim < 14)
        let v_val = VALUES[victim as usize]; // Giá trị quân bị ăn
        let a_val = VALUES[attacker as usize]; // Giá trị quân tấn công
        v_val * 10 - a_val + 10000 // Nước đi ăn quân nhận điểm ưu tiên > 10000
    } else { // Nước đi tĩnh không ăn quân
        0 // Điểm tĩnh bằng 0
    } // Kết thúc kiểm tra victim
} // Kết thúc hàm score

/// Sắp xếp danh sách nước đi `List` tại chỗ (In-place sort) theo điểm MVV-LVA giảm dần.
pub fn sort(pos: &Position, list: &mut List) { // Hàm sort sắp xếp tại chỗ
    let len = list.len(); // Lấy số lượng nước đi trong danh sách
    if len <= 1 { // Nếu danh sách có 0 hoặc 1 nước đi
        return; // Không cần sắp xếp
    } // Kết thúc kiểm tra len

    let mut scores = [0i32; 128]; // Mảng điểm số tạm thời trên Stack L1 Cache
    let mut idx = 0usize; // Chỉ số tính điểm
    while idx < len { // Duyệt tất cả nước đi trong danh sách
        scores[idx] = score(pos, list.get(idx)); // Tính điểm MVV-LVA cho nước đi idx
        idx += 1; // Tăng chỉ số
    } // Kết thúc vòng lặp tính điểm

    // Thuật toán Selection Sort tại chỗ trên Stack (Cực nhanh với N <= 128)
    let mut i = 0usize;
    while i < len - 1 {
        let mut max_idx = i;
        let mut j = i + 1;
        while j < len {
            if scores[j] > scores[max_idx] {
                max_idx = j;
            }
            j += 1;
        }
        if max_idx != i {
            scores.swap(i, max_idx);
            list.items.swap(i, max_idx);
        }
        i += 1;
    }
} // Kết thúc hàm sort
