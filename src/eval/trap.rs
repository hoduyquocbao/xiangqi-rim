// ============================================================================
// MODULE EVAL TRAP: BỘ ĐÁNH GIÁ BẪY CỜ VÀ THẮT CHẶT KHÔNG GIAN (PIECE TRAP ENGINE)
// ============================================================================
// Module `trap.rs` triển khai tri thức chuyên sâu về bẫy cờ và khống chế không gian:
// 1. Mã nghẽn chân (Trapped Knight): Phát hiện Mã bị vây kẹt cản chân không có ô hạ cánh an toàn.
// 2. Xe bị kẹt góc (Trapped Rook): Phát hiện Xe bị nhốt ở góc không có đường xuất trận.
// 3. Pháo mất ngòi (Trapped Cannon): Phát hiện Pháo bị giam hãm không có ngòi cơ động.
// 4. Độ thắt chặt không gian (Piece Constriction): Đánh giá tỷ lệ ô bị phong tỏa của quân chủ lực.
// 100% Clean Room std-only, căn lề 64-byte, 100% chú thích tiếng Việt & từ đơn tiếng Anh.
// ============================================================================

use crate::board::{Bitboard, Position, Square};
use crate::movegen::lookup;

const COL_BASE: u64 = 1u64 | (1u64 << 9) | (1u64 << 18) | (1u64 << 27) | (1u64 << 36);

/// Mặt nạ 9 cột dọc O(1)
pub static FILE_MASKS: [Bitboard; 9] = [
    Bitboard::from_raw(COL_BASE << 0, COL_BASE << 0),
    Bitboard::from_raw(COL_BASE << 1, COL_BASE << 1),
    Bitboard::from_raw(COL_BASE << 2, COL_BASE << 2),
    Bitboard::from_raw(COL_BASE << 3, COL_BASE << 3),
    Bitboard::from_raw(COL_BASE << 4, COL_BASE << 4),
    Bitboard::from_raw(COL_BASE << 5, COL_BASE << 5),
    Bitboard::from_raw(COL_BASE << 6, COL_BASE << 6),
    Bitboard::from_raw(COL_BASE << 7, COL_BASE << 7),
    Bitboard::from_raw(COL_BASE << 8, COL_BASE << 8),
];

/// Mặt nạ 10 hàng ngang O(1)
pub static RANK_MASKS: [Bitboard; 10] = [
    Bitboard::from_raw(0x1FFu64 << 0, 0),
    Bitboard::from_raw(0x1FFu64 << 9, 0),
    Bitboard::from_raw(0x1FFu64 << 18, 0),
    Bitboard::from_raw(0x1FFu64 << 27, 0),
    Bitboard::from_raw(0x1FFu64 << 36, 0),
    Bitboard::from_raw(0, 0x1FFu64 << 0),
    Bitboard::from_raw(0, 0x1FFu64 << 9),
    Bitboard::from_raw(0, 0x1FFu64 << 18),
    Bitboard::from_raw(0, 0x1FFu64 << 27),
    Bitboard::from_raw(0, 0x1FFu64 << 36),
];

/// Struct `Trap` đánh giá bẫy cờ và mức độ phong tỏa không gian của quân chủ lực.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug, Default)]
pub struct Trap;

impl Trap {
    /// Khởi tạo đối tượng `Trap` mới.
    #[inline(always)]
    pub const fn new() -> Self {
        Self
    }

    /// Đánh giá điểm bẫy cờ và phong tỏa cho cả 2 phe, trả về cặp `(mg, eg)` điểm centipawn.
    #[inline(always)]
    pub fn evaluate(pos: &Position) -> (i32, i32) {
        let mut mg = 0i32;
        let mut eg = 0i32;

        let mut color = 0usize;
        while color < 2 {
            let sign = if color == 0 { 1 } else { -1 };

            // 1. Đánh giá Mã nghẽn chân (Trapped Knight)
            let mut knights = pos.piece[color * 7 + 3];
            while let Some(sq) = knights.pop() {
                let mut safe_destinations = 0i32;
                let mut blocked_legs = 0i32;

                let dest_bb = lookup::KNIGHT[sq.0 as usize];
                let mut dest_iter = dest_bb;
                while let Some(dest_sq) = dest_iter.pop() {
                    let leg_sq = lookup::leg(sq.0 as usize, dest_sq.0 as usize);
                    if leg_sq != 255 {
                        let leg_code = pos.grid[leg_sq as usize];
                        if leg_code < 14 {
                            blocked_legs += 1;
                        } else {
                            let dest_code = pos.grid[dest_sq.0 as usize];
                            if dest_code >= 14 || (dest_code as usize / 7) != color {
                                safe_destinations += 1;
                            }
                        }
                    }
                }

                // Nếu Mã bị cản từ 3 chân trở lên hoặc có <= 1 ô hạ cánh an toàn -> Bẫy Mã nghẽn chân
                if safe_destinations == 0 || blocked_legs >= 3 {
                    mg -= sign * 100;
                    eg -= sign * 160;
                } else if safe_destinations == 1 {
                    mg -= sign * 60;
                    eg -= sign * 100;
                }

                // Mã lao xuống đáy bàn cờ đối phương (rank 0 của Red hoặc rank 9 của Black) khi chưa có phối hợp Xe/Pháo
                let r = sq.rank();
                let is_opp_back_rank = if color == 0 { r >= 8 } else { r <= 1 };
                if is_opp_back_rank && safe_destinations <= 2 {
                    mg -= sign * 80;
                    eg -= sign * 140;
                }
            }

            // 2. Đánh giá Xe bị kẹt góc/chưa xuất trận (Trapped/Undeveloped Corner Rook)
            let mut rooks = pos.piece[color * 7 + 4];
            while let Some(sq) = rooks.pop() {
                let rank = sq.rank();
                let file = sq.file();

                // Xe ở góc ban đầu (file 0, 8 tại rank 0 hoặc rank 9)
                let is_home_corner = (file == 0 || file == 8) && (if color == 0 { rank == 0 } else { rank == 9 });
                if is_home_corner {
                    let rook_moves = lookup::rook(sq.0, pos.occupied, pos.color[1 - color]);
                    let count = rook_moves.count();
                    if count <= 1 {
                        mg -= sign * 90;
                        eg -= sign * 140;
                    }
                }
            }

            // 3. Đánh giá Pháo mất ngòi/kẹt ngòi (Trapped Cannon)
            let mut cannons = pos.piece[color * 7 + 5];
            while let Some(sq) = cannons.pop() {
                let king_bb = lookup::KING[color][sq.0 as usize];
                let mut screens = 0i32;
                let mut king_iter = king_bb;
                while let Some(target_sq) = king_iter.pop() {
                    let target_code = pos.grid[target_sq.0 as usize];
                    if target_code < 14 {
                        screens += 1;
                    }
                }

                // Pháo bị bao vây 3 phía trở lên không có ngòi phát huy sức mạnh
                if screens >= 3 {
                    mg -= sign * 30;
                    eg -= sign * 50;
                }
            }

            // 4. Bẫy "Điệu Hổ Ly Sơn" & Tướng lộ cung (Lured / Exposed King)
            let king_sq = pos.king[color];
            let kf = king_sq % 9;
            let kr = king_sq / 9;
            let exposed = if color == 0 {
                kr < 9 || kf != 4 // Đỏ: Tướng bị dụ lên hàng 7, 8 hoặc lệch cột
            } else {
                kr > 0 || kf != 4 // Đen: Tướng bị dụ xuống hàng 1, 2 hoặc lệch cột
            };
            if exposed {
                mg -= sign * 60;
                eg -= sign * 100;
            }

            // 5. Cấu trúc gãy Sĩ Tượng ("Khuyết Sĩ kỵ Xe, Khuyết Tượng kỵ Pháo")
            let other = color ^ 1;
            let advisors = pos.piece[color * 7 + 1].count();
            let bishops = pos.piece[color * 7 + 2].count();
            let opp_rooks = pos.piece[other * 7 + 4].count();
            let opp_cannons = pos.piece[other * 7 + 5].count();

            // Khuyết Sĩ khi đối phương còn Xe
            if advisors < 2 && opp_rooks > 0 {
                let penalty = (2 - advisors as i32) * 40;
                mg -= sign * penalty;
                eg -= sign * (penalty + 20);
            }

            // Khuyết Tượng khi đối phương còn Pháo
            if bishops < 2 && opp_cannons > 0 {
                let penalty = (2 - bishops as i32) * 30;
                mg -= sign * penalty;
                eg -= sign * (penalty + 20);
            }

            // 6. Bẫy "Dương Đông Kích Tây" & Pháo Đầu ép trung lộ (Center Cannon Pinning)
            let mut center_cannons = pos.piece[color * 7 + 5];
            while let Some(sq) = center_cannons.pop() {
                if sq.file() == 4 {
                    // Pháo chiếm trung lộ cột 4
                    mg += sign * 35;
                    eg += sign * 45;
                }
            }

            // 7. Bẫy "Trùng Pháo" (Double Cannon Direct Alignment)
            let cannon_bb = pos.piece[color * 7 + 5];
            let mut iter = cannon_bb;
            if let (Some(c1), Some(c2)) = (iter.pop(), iter.pop()) {
                if c1.file() == c2.file() || c1.rank() == c2.rank() {
                    // Hai pháo cùng nằm trên 1 đường thẳng
                    mg += sign * 40;
                    eg += sign * 55;
                }
            }

            // 8. Bẫy "Xe Thọc Đáy & Đè Cung" (Bottom-rank Rook Infiltration & Throat Control)
            let enemy_rooks = pos.piece[other * 7 + 4];
            let mut er_iter = enemy_rooks;
            let own_bottom_rank = if color == 0 { 0 } else { 9 };
            let own_throat_rank = if color == 0 { 1 } else { 8 };
            while let Some(er_sq) = er_iter.pop() {
                let r_rank = er_sq.rank();
                let r_file = er_sq.file();
                // Xe địch lọt vào hàng đáy hoặc hàng cổ họng của cung ta
                if (r_rank == own_bottom_rank || r_rank == own_throat_rank) && r_file >= 2 && r_file <= 6 {
                    mg -= sign * 80;
                    eg -= sign * 120;
                }
            }

            // 9. Bẫy "Mã Ngọa Tào & Mã Điếu Ngư" (Palace Flank Threat Knight)
            let enemy_knights = pos.piece[other * 7 + 3];
            let mut ek_iter = enemy_knights;
            while let Some(ek_sq) = ek_iter.pop() {
                let k_rank = ek_sq.rank();
                let k_file = ek_sq.file();
                // Mã Ngọa Tào (hàng 2/7, cột 2/6) hoặc Mã Điếu Ngư (hàng 3/6, cột 1/7)
                let is_ngoa_tao = if color == 0 {
                    (k_rank == 1 || k_rank == 2) && (k_file == 2 || k_file == 6)
                } else {
                    (k_rank == 8 || k_rank == 7) && (k_file == 2 || k_file == 6)
                };
                let is_dieu_ngu = if color == 0 {
                    (k_rank == 2 || k_rank == 3) && (k_file == 1 || k_file == 7)
                } else {
                    (k_rank == 7 || k_rank == 6) && (k_file == 1 || k_file == 7)
                };
                if is_ngoa_tao {
                    mg -= sign * 160;
                    eg -= sign * 240;
                } else if is_dieu_ngu {
                    mg -= sign * 140;
                    eg -= sign * 210;
                }
            }

            // 10. Bẫy "Tiểu Tốt Qua Sông & Tốt Áp Cung / Ép Quân Lớn" (Passed Pawn & Palace Infiltration Threat)
            let enemy_pawns = pos.piece[other * 7 + 6];
            let mut ep_iter = enemy_pawns;
            while let Some(ep_sq) = ep_iter.pop() {
                let p_rank = ep_sq.rank();
                let p_file = ep_sq.file();
                let dist_to_palace = if color == 0 { p_rank } else { 9 - p_rank };
                let is_across = if color == 0 { p_rank <= 4 } else { p_rank >= 5 };
                let is_palace_flank = p_file >= 1 && p_file <= 7;
                let is_in_palace_box = p_file >= 2 && p_file <= 6;

                if is_across && is_palace_flank {
                    if dist_to_palace == 0 {
                        // Tốt địch thọc đáy cung ta (sát đáy cung Tướng)
                        let pen_mg = if is_in_palace_box { 180 } else { 90 };
                        let pen_eg = if is_in_palace_box { 260 } else { 130 };
                        mg -= sign * pen_mg;
                        eg -= sign * pen_eg;
                    } else if dist_to_palace == 1 {
                        // Tốt địch ở hàng cổ họng cung (hàng 1/8)
                        let pen_mg = if is_in_palace_box { 140 } else { 70 };
                        let pen_eg = if is_in_palace_box { 200 } else { 100 };
                        mg -= sign * pen_mg;
                        eg -= sign * pen_eg;
                    } else if dist_to_palace == 2 {
                        // Tốt địch ở hàng 2/7 (hàng Tốt áp sát Sĩ Tượng)
                        let pen_mg = if is_in_palace_box { 100 } else { 50 };
                        let pen_eg = if is_in_palace_box { 150 } else { 75 };
                        mg -= sign * pen_mg;
                        eg -= sign * pen_eg;
                    } else if dist_to_palace == 3 {
                        // Tốt địch ở hàng 3/6 (hàng điểm huyệt)
                        let pen_mg = if is_in_palace_box { 60 } else { 30 };
                        let pen_eg = if is_in_palace_box { 90 } else { 45 };
                        mg -= sign * pen_mg;
                        eg -= sign * pen_eg;
                    } else {
                        // Tốt địch vừa qua sông (hàng 4/5)
                        mg -= sign * 40;
                        eg -= sign * 70;
                    }

                    // Kiểm tra Tốt địch đang đâm thẳng vào Quân lớn của ta (Mã, Pháo, Xe, Tượng)
                    let own_majors = pos.piece[color * 7 + 2] | pos.piece[color * 7 + 3] | pos.piece[color * 7 + 4] | pos.piece[color * 7 + 5];
                    let mut om_iter = own_majors;
                    while let Some(om_sq) = om_iter.pop() {
                        if om_sq.file() == p_file {
                            let om_r = om_sq.rank();
                            let forward_dist = if color == 0 { (p_rank as i8) - (om_r as i8) } else { (om_r as i8) - (p_rank as i8) };
                            if forward_dist == 1 || forward_dist == 2 {
                                // Tốt địch chỉ cách 1-2 bước đâm trực diện quân lớn
                                mg -= sign * 150;
                                eg -= sign * 220;
                            }
                        }
                    }
                }
            }

            // 11. Bảo Toàn Xe Chủ Lực Đè Cung (Dominant Infiltrating Rook Pin)
            let own_rooks = pos.piece[color * 7 + 4];
            let mut or_iter = own_rooks;
            while let Some(or_sq) = or_iter.pop() {
                let r_rank = or_sq.rank();
                let r_file = or_sq.file();
                let is_infiltrating = if color == 0 {
                    r_rank >= 7 && r_file >= 2 && r_file <= 6
                } else {
                    r_rank <= 2 && r_file >= 2 && r_file <= 6
                };
                if is_infiltrating {
                    mg += sign * 60;
                    eg += sign * 90;
                }
            }

            // 12. Phạt Xe Chưa Xuất Trận (Undeveloped Corner Rook Penalty)
            let mut or_init_iter = own_rooks;
            while let Some(or_sq) = or_init_iter.pop() {
                let is_corner = if color == 0 {
                    or_sq.0 == 0 || or_sq.0 == 8 || or_sq.0 == 7
                } else {
                    or_sq.0 == 81 || or_sq.0 == 89 || or_sq.0 == 88
                };
                if is_corner {
                    mg -= sign * 40;
                }
            }

            // 13. Khống Chế Tốt Đối Phương Qua Sông & Phong Tỏa Trung Lộ (Central Advanced Pawn Blockade)
            let enemy_pawns = pos.piece[other * 7 + 6];
            let mut ep_blk_iter = enemy_pawns;
            while let Some(ep_sq) = ep_blk_iter.pop() {
                let p_rank = ep_sq.rank();
                let p_file = ep_sq.file();
                let is_across = if color == 0 { p_rank <= 4 } else { p_rank >= 5 };
                if is_across {
                    // Ô tiến về phía trước của Tốt địch
                    let forward_sq = if color == 0 {
                        if p_rank > 0 { Some(ep_sq.0 - 9) } else { None }
                    } else {
                        if p_rank < 9 { Some(ep_sq.0 + 9) } else { None }
                    };
                    if let Some(fsq) = forward_sq {
                        let is_center_file = p_file >= 3 && p_file <= 5;
                        if pos.occupied.test(Square(fsq)) {
                            // Tốt địch bị quân chặn đầu (Blockaded) -> Thưởng phe phòng thủ
                            if is_center_file {
                                mg += sign * 40;
                                eg += sign * 70;
                            } else {
                                mg += sign * 20;
                                eg += sign * 35;
                            }
                        } else {
                            // Tốt địch thông thoáng tự do tiến sát cung -> Phạt nhẹ phe phòng thủ
                            if is_center_file {
                                mg -= sign * 50;
                                eg -= sign * 90;
                            } else {
                                mg -= sign * 30;
                                eg -= sign * 50;
                            }
                        }
                    }
                }
            }

            // 14. Chênh Lệch Xe Độc Quyền & Nghiêm Cấm Đổi Xe Lấy Quân Nhỏ (Rook Monopoly & Anti-Sacrifice)
            let own_rook_count = pos.counts[color * 7 + 4];
            let enemy_rook_count = pos.counts[other * 7 + 4];
            if own_rook_count == 0 && enemy_rook_count >= 1 {
                mg -= sign * 200;
                eg -= sign * 350;
            } else if own_rook_count >= 1 && enemy_rook_count == 0 {
                mg += sign * 200;
                eg += sign * 350;
            }

            // 15. Xe Kiểm Soát Cột Mở Đe Dọa Quân Trục Lộ (Open File Rook Attack Corridor)
            let mut or_open_iter = own_rooks;
            while let Some(r_sq) = or_open_iter.pop() {
                let r_file = r_sq.file();
                let is_flank_corridor = r_file == 1 || r_file == 3 || r_file == 5 || r_file == 7;
                if is_flank_corridor {
                    let obstacles = FILE_MASKS[r_file as usize] & pos.occupied;
                    if obstacles.count() <= 2 {
                        // Cột mở hoặc bán mở cho Xe thông suốt -> Thưởng kiểm soát thế trận
                        mg += sign * 25;
                        eg += sign * 40;
                    }
                }
            }

            // 16. Bảo Toàn Tượng Tâm Che Chắn Cung Tướng (Central Guardian Elephant Fortress)
            let center_elephant_sq = if color == 0 { Square(22) } else { Square(67) };
            let has_center_elephant = pos.piece[color * 7 + 2].test(center_elephant_sq);
            let enemy_attackers = pos.counts[other * 7 + 4] + pos.counts[other * 7 + 3] + pos.counts[other * 7 + 5];
            if enemy_attackers >= 1 {
                if has_center_elephant {
                    mg += sign * 35;
                    eg += sign * 55;
                } else {
                    mg -= sign * 50;
                    eg -= sign * 80;
                }
            }

            // 17. Khống Chế Đe Dọa Ngọa Long Mã (Infiltrating Knight Threat)
            let enemy_knights = pos.piece[other * 7 + 3];
            let mut ek_threat_iter = enemy_knights;
            while let Some(ek_sq) = ek_threat_iter.pop() {
                let k_rank = ek_sq.rank();
                let is_near_palace = if color == 0 { k_rank <= 5 } else { k_rank >= 4 };
                if is_near_palace {
                    // Kiểm tra các ô nhảy của Mã địch hướng vào cung
                    let jumps = lookup::KNIGHT[ek_sq.0 as usize];
                    let mut jump_iter = jumps;
                    while let Some(j_sq) = jump_iter.pop() {
                        let leg = lookup::leg(ek_sq.0 as usize, j_sq.0 as usize);
                        if leg != 255 && !pos.occupied.test(Square(leg)) {
                            // Chân Mã thông thoáng
                            let j_rank = j_sq.rank();
                            let j_file = j_sq.file();
                            let is_dangerous = if color == 0 {
                                (j_rank <= 3 && (j_file >= 1 && j_file <= 7)) || (j_rank <= 2 && (j_file >= 3 && j_file <= 5))
                            } else {
                                (j_rank >= 6 && (j_file >= 1 && j_file <= 7)) || (j_rank >= 7 && (j_file >= 3 && j_file <= 5))
                            };
                            if is_dangerous {
                                // Phạt nặng đòn Mã thọc cổ / điểm huyệt cung Tướng
                                mg -= sign * 180;
                                eg -= sign * 280;
                            }
                        } else if leg != 255 {
                            // Chân Mã địch bị chặn (Chèn chân Mã) -> Thưởng phe phòng thủ
                            mg += sign * 50;
                            eg += sign * 80;
                        }
                    }
                }
            }

            // 18. Điều Xe Về Thủ Cung & Phong Tỏa Vùng Cấm (Rook Palace Defense & Anti-Flank Isolation)
            let enemy_invaders = pos.piece[other * 7 + 3] | pos.piece[other * 7 + 4] | pos.piece[other * 7 + 5] | pos.piece[other * 7 + 6];
            let mut inv_iter = enemy_invaders;
            let mut invader_count = 0usize;
            while let Some(inv_sq) = inv_iter.pop() {
                let inv_r = inv_sq.rank();
                let inv_f = inv_sq.file();
                let is_in_territory = if color == 0 { inv_r <= 3 && inv_f >= 1 && inv_f <= 7 } else { inv_r >= 6 && inv_f >= 1 && inv_f <= 7 };
                if is_in_territory {
                    invader_count += 1;
                }
            }

            if invader_count >= 1 {
                let mut or_def_iter = own_rooks;
                let mut rook_defending = false;
                while let Some(or_sq) = or_def_iter.pop() {
                    let or_r = or_sq.rank();
                    let or_f = or_sq.file();
                    let is_defending = if color == 0 { or_r <= 3 && or_f >= 2 && or_f <= 6 } else { or_r >= 6 && or_f >= 2 && or_f <= 6 };
                    if is_defending {
                        rook_defending = true;
                        break;
                    }
                }
                if rook_defending {
                    mg += sign * 50;
                    eg += sign * 80;
                } else {
                    let penalty = if invader_count >= 2 { 90 } else { 50 };
                    mg -= sign * penalty;
                    eg -= sign * (penalty + 60);
                }
            }

            // 19. Bảo Toàn Binh Lực Quân Lớn (Major Piece Balance & Anti-Sacrifice)
            let own_majors = pos.counts[color * 7 + 4] * 2 + pos.counts[color * 7 + 5] + pos.counts[color * 7 + 3];
            let enemy_majors = pos.counts[other * 7 + 4] * 2 + pos.counts[other * 7 + 5] + pos.counts[other * 7 + 3];
            if own_majors < enemy_majors {
                let diff = (enemy_majors - own_majors) as i32;
                mg -= sign * diff * 220;
                eg -= sign * diff * 320;
            } else if own_majors > enemy_majors {
                let diff = (own_majors - enemy_majors) as i32;
                mg += sign * diff * 220;
                eg += sign * diff * 320;
            }

            // Cảnh báo mất sạch Xe khi đối phương còn nhiều quân tấn công cơ động
            if pos.counts[color * 7 + 4] == 0 && (pos.counts[other * 7 + 5] >= 2 || pos.counts[other * 7 + 3] >= 2) {
                mg -= sign * 80;
                eg -= sign * 140;
            }

            // 20. Giải Tỏa Bẫy Xe Ghim Trục Dọc/Ngang (Rook Skewer & Alignment Pin)
            let mut own_pieces_bb = Bitboard::empty();
            for p in 0..7 {
                own_pieces_bb |= pos.piece[color * 7 + p];
            }
            let enemy_rooks_bb = pos.piece[other * 7 + 4];
            let mut er_skewer_iter = enemy_rooks_bb;
            while let Some(er_sq) = er_skewer_iter.pop() {
                let er_f = er_sq.file();
                let er_r = er_sq.rank();

                // Kiểm tra ghim cột dọc O(1)
                let col_own_count = (FILE_MASKS[er_f as usize] & own_pieces_bb).count();
                if col_own_count >= 2 {
                    // Xe địch đang ghim từ 2 quân ta trở lên trên cùng 1 cột dọc -> Phạt thế bị ghim
                    mg -= sign * 45;
                    eg -= sign * 75;
                }

                // Kiểm tra ghim hàng ngang O(1)
                let row_own_count = (RANK_MASKS[er_r as usize] & own_pieces_bb).count();
                if row_own_count >= 2 {
                    // Xe địch đang ghim từ 2 quân ta trở lên trên cùng 1 hàng ngang -> Phạt thế bị ghim
                    mg -= sign * 45;
                    eg -= sign * 75;
                }
            }

            // 21. Cấm Tượng Rời Cung Khi Có Xe/Pháo Địch Đe Dọa (Central Elephant Inviolability)
            let enemy_cannons_bb = pos.piece[other * 7 + 5];
            let has_attacking_threat = (enemy_rooks_bb | enemy_cannons_bb).active();
            if has_attacking_threat {
                let center_e_sq = if color == 0 { Square(22) } else { Square(67) };
                if !pos.piece[color * 7 + 2].test(center_e_sq) {
                    // Mất Tượng tâm khi đối phương còn Xe hoặc Pháo -> Phạt vừa phải giữ trận
                    mg -= sign * 90;
                    eg -= sign * 140;
                }
            }

            // 22. Phạt Nặng Mã Bị Ép Vào Góc Đáy / Biên Cùng (Corner & Rim Trapped Knight)
            let mut own_knights_bb = pos.piece[color * 7 + 3];
            while let Some(k_sq) = own_knights_bb.pop() {
                let k_f = k_sq.file();
                let k_r = k_sq.rank();
                let is_corner_rim = (k_f == 0 || k_f == 8) && (k_r == 0 || k_r == 1 || k_r == 8 || k_r == 9);
                if is_corner_rim {
                    // Kiểm tra số bước nhảy hợp lệ của Mã này
                    let mut valid_jumps = 0usize;
                    let jumps = lookup::KNIGHT[k_sq.0 as usize] & !pos.occupied;
                    let mut j_iter = jumps;
                    while let Some(j_sq) = j_iter.pop() {
                        let leg = lookup::leg(k_sq.0 as usize, j_sq.0 as usize);
                        if leg != 255 && !pos.occupied.test(Square(leg)) {
                            valid_jumps += 1;
                        }
                    }
                    if valid_jumps <= 1 {
                        // Mã bị kẹt góc không lối thoát
                        mg -= sign * 45;
                        eg -= sign * 75;
                    }
                }
            }

            // 23. Bẫy "Mã Nhập Cung Tướng Bất Toàn" (Knight Trapping Palace Center)
            let center_palace_sq = if color == 0 { Square(13) } else { Square(76) }; // e1 hoặc e8
            if pos.piece[color * 7 + 3].test(center_palace_sq) {
                // Mã tự chui vào tâm Cung Tướng làm kẹt Sĩ và bóp nghẹt đường chạy của Tướng
                mg -= sign * 80;
                eg -= sign * 120;
            }

            // 24. Khống Chế Sát Cục Tướng Lệch Sườn (Flank Exposed King Squeeze & Mating Net)
            let king_sq = pos.piece[color * 7 + 0].lsb_idx();
            let k_f = king_sq % 9;
            if k_f == 3 || k_f == 5 {
                // Tướng bị lùa ra sườn d (lộ 4) hoặc f (lộ 6)
                let advisors_count = pos.piece[color * 7 + 1].count();
                if advisors_count < 2 {
                    // Tướng lộ sườn thiếu Sĩ che chắn -> Phạt vừa phải
                    mg -= sign * 80;
                    eg -= sign * 130;
                }
            }

            // 25. Bẫy Trầm Pháo Đáy Cung & Xe Pháo Sát Đáy (Bottom Rank Infiltrating Cannon & Rook)
            let enemy_cannons = pos.piece[other * 7 + 5];
            let mut ec_iter = enemy_cannons;
            while let Some(ec_sq) = ec_iter.pop() {
                let ec_r = ec_sq.rank();
                let is_bottom_rank = if color == 0 { ec_r == 0 } else { ec_r == 9 };
                if is_bottom_rank {
                    // Pháo địch đã thọc xuống đáy Cung Tướng (Trầm Pháo Đáy Cung)
                    mg -= sign * 80;
                    eg -= sign * 130;
                }
            }

            let mut er_bottom_iter = enemy_rooks_bb;
            while let Some(er_sq) = er_bottom_iter.pop() {
                let er_r = er_sq.rank();
                let is_bottom_rank = if color == 0 { er_r == 0 } else { er_r == 9 };
                if is_bottom_rank {
                    // Xe địch đã thọc xuống đáy Cung Tướng
                    mg -= sign * 90;
                    eg -= sign * 140;
                }
            }

            // 26. Bẫy Song Mã Pháo Sát Cục (Double Knights Palace Infiltration)
            let mut own_knights_palace = 0u32;
            let mut ok_iter = pos.piece[color * 7 + 3];
            while let Some(k_sq) = ok_iter.pop() {
                let k_r = k_sq.rank();
                let is_palace_zone = if color == 0 { k_r >= 6 } else { k_r <= 3 };
                if is_palace_zone {
                    own_knights_palace += 1;
                }
            }
            if own_knights_palace >= 2 {
                // Sở hữu Song Mã thọc sâu vào trận địa đối phương -> Thế công cực mạnh sát cục
                mg += sign * 90;
                eg += sign * 140;
            }

            // 27. Xe Địch Đè Tuyến Sườn & Tuyến Tượng (Enemy Rook Infiltration into Defending Fortress)
            let mut er_fort_iter = pos.piece[other * 7 + 4];
            while let Some(er_sq) = er_fort_iter.pop() {
                let er_r = er_sq.rank();
                let is_in_fortress = if color == 0 { er_r <= 2 } else { er_r >= 7 };
                if is_in_fortress {
                    // Xe địch đã lọt vào 3 hàng phòng thủ sân nhà
                    mg -= sign * 80;
                    eg -= sign * 130;
                }
            }

            // 28. Song Xa Hợp Lực Phối Hợp Áp Đảo (Two Rooks Advantage & Rook Parity)
            let own_rooks_total = pos.counts[color * 7 + 4];
            let enemy_rooks_total = pos.counts[other * 7 + 4];
            if enemy_rooks_total >= 2 && own_rooks_total <= 1 {
                // Địch còn Song Xa trong khi ta chỉ còn 1 Xe hoặc 0 Xe -> Phạt vừa phải giữ quân
                mg -= sign * 100;
                eg -= sign * 160;
            } else if own_rooks_total >= 2 && enemy_rooks_total <= 1 {
                mg += sign * 100;
                eg += sign * 160;
            }

            // 29. Bẫy Trung Pháo Chiếu Cung Kẹp Sĩ (Central Cannon Palace Screen Pin & Skewer)
            let king_sq_c = pos.piece[color * 7 + 0].lsb_idx();
            let k_f_c = king_sq_c % 9;
            let k_r_c = (king_sq_c / 9) as u8;
            if k_f_c == 4 { // Tướng ta đang ở trung lộ (cột e)
                let enemy_cannons_c = pos.piece[other * 7 + 5];
                let mut ec_c_iter = enemy_cannons_c;
                while let Some(ec_sq) = ec_c_iter.pop() {
                    if ec_sq.file() == 4 { // Pháo địch cũng ở trung lộ
                        let ec_r = ec_sq.rank();
                        let (min_r, max_r) = if k_r_c < ec_r { (k_r_c, ec_r) } else { (ec_r, k_r_c) };
                        let mut pieces_between = 0usize;
                        let mut r_scan = min_r + 1;
                        while r_scan < max_r {
                            if pos.occupied.test(Square(r_scan * 9 + 4)) {
                                pieces_between += 1;
                            }
                            r_scan += 1;
                        }
                        if pieces_between == 1 {
                            // Pháo địch đang ghim Tướng ta qua đúng 1 ngòi (Pháo Trùm Trung Lộ) -> CỰC KỲ NGUY HIỂM!
                            let has_enemy_rook = pos.counts[other * 7 + 4] > 0;
                            let pen_mg = if has_enemy_rook { 160 } else { 100 };
                            let pen_eg = if has_enemy_rook { 240 } else { 150 };
                            mg -= sign * pen_mg;
                            eg -= sign * pen_eg;
                        } else if pieces_between == 2 {
                            // Pháo địch ghim qua 2 ngòi Cung Tướng (Sĩ/Tượng kẹt giữa)
                            let has_enemy_rook = pos.counts[other * 7 + 4] > 0;
                            let pen_mg = if has_enemy_rook { 120 } else { 70 };
                            let pen_eg = if has_enemy_rook { 170 } else { 110 };
                            mg -= sign * pen_mg;
                            eg -= sign * pen_eg;
                        }
                    }
                }
            }

            // 30. Song Mã Đoạt Cung Tàn Cuộc (Endgame Two Knights King Hunt)
            let enemy_rooks_cnt = pos.counts[other * 7 + 4];
            let own_knights_cnt = pos.counts[color * 7 + 3];
            if enemy_rooks_cnt == 0 && own_knights_cnt >= 2 {
                let enemy_king_sq = pos.piece[other * 7 + 0].lsb_idx();
                let ek_r = (enemy_king_sq / 9) as i32;
                let ek_f = (enemy_king_sq % 9) as i32;
                let mut ok_hunt_iter = pos.piece[color * 7 + 3];
                while let Some(k_sq) = ok_hunt_iter.pop() {
                    let k_r = k_sq.rank() as i32;
                    let k_f = k_sq.file() as i32;
                    let dist = (ek_r - k_r).abs() + (ek_f - k_f).abs();
                    if dist <= 4 {
                        // Mã tiếp cận Cung Tướng đối phương khi địch hết Xe -> Thưởng lớn dứt điểm
                        mg += sign * 50;
                        eg += sign * 90;
                    }
                }
            }

            // 31. Phòng Thủ Khẩn Cấp Tốt Địch Nhập Cung Tàn Cuộc (Emergency Blockade Against Infiltrating Endgame Pawns)
            let mut ep_endgame_iter = pos.piece[other * 7 + 6];
            while let Some(ep_sq) = ep_endgame_iter.pop() {
                let p_r = ep_sq.rank();
                let p_f = ep_sq.file();
                let is_near_palace = p_f >= 2 && p_f <= 6;
                if is_near_palace {
                    let depth_in_fortress = if color == 0 { (3 as i32) - (p_r as i32) } else { (p_r as i32) - (6 as i32) };
                    if depth_in_fortress >= 0 {
                        // Tốt địch áp sát hàng 3 hoặc đã lọt sâu vào Cung Tướng (hàng 0, 1, 2)
                        let pen_mg = 80 + depth_in_fortress * 40;
                        let pen_eg = 140 + depth_in_fortress * 60;
                        mg -= sign * pen_mg;
                        eg -= sign * pen_eg;
                    }
                }
            }

            // 32. Phối Hợp Tướng Xuất Lộ Hỗ Trợ Tấn Công (Attacking King & High King Support)
            if enemy_rooks_cnt == 0 {
                let our_king_sq = pos.piece[color * 7 + 0].lsb_idx();
                let our_k_f = our_king_sq % 9;
                if our_k_f == 3 || our_k_f == 5 {
                    // Tướng ta xuất lộ 4 hoặc lộ 6 khi đối phương hết Xe -> Làm ngòi/mặt tướng kiểm soát lộ
                    mg += sign * 35;
                    eg += sign * 65;
                }
            }

            // 33. Nguy Cơ Tốt Biên Tiến Thẳng Áp Sát Cung Sát Cục (Flank Passed Pawn Infiltration & King Flank Threat)
            let mut ep_flank_iter = pos.piece[other * 7 + 6];
            while let Some(ep_sq) = ep_flank_iter.pop() {
                let p_r = ep_sq.rank();
                let p_f = ep_sq.file();
                if p_f <= 1 || p_f >= 7 {
                    let depth_in_camp = if color == 0 { 4 - p_r.min(4) } else { p_r.max(5) - 5 };
                    if depth_in_camp >= 2 {
                        // Tốt biên địch đã qua sông và tiến sâu >= 2 hàng -> Cực kỳ nguy hiểm cho Tướng cùng phía
                        let has_enemy_knights = pos.counts[other * 7 + 3] > 0;
                        let flank_pen_mg = if has_enemy_knights { 140 } else { 70 };
                        let flank_pen_eg = if has_enemy_knights { 250 } else { 120 };
                        mg -= sign * flank_pen_mg;
                        eg -= sign * flank_pen_eg;
                    }
                }
            }

            // 34. Hiệp Lực Phòng Thủ Hậu Phương / Mã Hồi Cung Cứu Chúa (Endgame Home Camp Defense & King Shield)
            let mut ep_danger_count = 0usize;
            let mut ep_palace_scan = pos.piece[other * 7 + 6];
            while let Some(ep_sq) = ep_palace_scan.pop() {
                let p_r = ep_sq.rank();
                let is_deep = if color == 0 { p_r <= 2 } else { p_r >= 7 };
                if is_deep {
                    ep_danger_count += 1;
                }
            }
            if ep_danger_count >= 1 {
                let our_k_sq = pos.piece[color * 7 + 0].lsb_idx();
                let our_k_r = (our_k_sq / 9) as i32;
                let our_k_f = (our_k_sq % 9) as i32;
                let mut own_majors = pos.piece[color * 7 + 3] | pos.piece[color * 7 + 5]; // Mã và Pháo ta
                let mut defenders_near_k = 0usize;
                while let Some(m_sq) = own_majors.pop() {
                    let m_r = m_sq.rank() as i32;
                    let m_f = m_sq.file() as i32;
                    let dist = (our_k_r - m_r).abs() + (our_k_f - m_f).abs();
                    if dist <= 3 {
                        defenders_near_k += 1;
                    }
                }
                if defenders_near_k == 0 && ep_danger_count >= 2 {
                    // Tốt địch lọt sâu vào cung nhưng quân chủ lực ta ở quá xa không về cứu -> Phạt nặng
                    mg -= sign * 120;
                    eg -= sign * 200;
                } else if defenders_near_k >= 1 {
                    // Có quân chủ lực gần Tướng bảo vệ và đánh chặn Tốt -> Thưởng an toàn
                    mg += sign * 40;
                    eg += sign * 80;
                }
            }

            color += 1;
        }

        (mg, eg)
    }
}

// ----------------------------------------------------------------------------
// UNIT TESTS CHO BỘ ĐÁNH GIÁ BẪY CỜ TRAP
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Parser;

    #[test]
    fn alignments() {
        assert_eq!(std::mem::align_of::<Trap>(), 64);
    }

    #[test]
    fn default_position_trap() {
        let pos = Parser::parse(Parser::DEFAULT);
        let (mg, eg) = Trap::evaluate(&pos);
        // Ở vị trí ban đầu, cả 2 bên đối xứng hoàn hảo
        assert_eq!(mg, 0);
        assert_eq!(eg, 0);
    }

    #[test]
    fn exposed_king_trap() {
        // Đỏ Tướng bị kéo sang cột 3 (d0 thay vì e0)
        let pos = Parser::parse("rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNB1KABNR w - - 0 1");
        let (mg, eg) = Trap::evaluate(&pos);
        assert!(mg < 0, "Đỏ bị Tướng lộ cung phải có điểm âm mg: {}", mg);
        assert!(eg < 0, "Đỏ bị Tướng lộ cung phải có điểm âm eg: {}", eg);
    }
}
