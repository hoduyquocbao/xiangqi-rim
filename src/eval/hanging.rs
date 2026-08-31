// ============================================================================
// MODULE EVAL HANGING: BỘ ĐÁNH GIÁ QUÂN TREO VÀ QUÂN BỊ TẤN CÔNG KHÔNG NGƯỜI BẢO VỆ
// ============================================================================
// `hanging.rs` giải quyết điểm mù chiến thuật then chốt của Engine:
// 1. Quân Treo (Hanging Pieces): Quân cờ bị quân đối phương tấn công nhưng có 0 quân nhà bảo vệ.
//    - Tốt treo bị dọa: Phạt -120cp (MG) / -180cp (EG).
//    - Mã/Pháo treo bị dọa: Phạt -380cp (MG) / -450cp (EG).
//    - Xe treo bị dọa: Phạt -850cp (MG) / -900cp (EG).
// 2. Quân bị đe dọa bởi quân nhỏ hơn (MVD - More Valuable Defender):
//    - Xe/Pháo/Mã bị Tốt đối phương tấn công trực diện.
// 3. 100% Clean Room std-only, căn lề 64-byte, 100% chú thích tiếng Việt & từ đơn tiếng Anh.
// ============================================================================

use crate::board::{Bitboard, Position, Square};
use crate::movegen::lookup;

/// Struct `Hanging` đánh giá quân bị tấn công và quân không có người giữ.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug, Default)]
pub struct Hanging;

impl Hanging {
    /// Khởi tạo một đối tượng `Hanging` mới.
    #[inline(always)]
    pub const fn new() -> Self {
        Self
    }

    /// Đánh giá điểm số phạt quân treo cho cả 2 phe, trả về cặp `(mg, eg)` centipawn.
    #[inline(always)]
    pub fn evaluate(pos: &Position) -> (i32, i32) {
        let mut mg = 0i32;
        let mut eg = 0i32;

        let occupied = pos.occupied;

        let mut color = 0usize;
        while color < 2 {
            let sign = if color == 0 { 1 } else { -1 };
            let other = 1 - color;
            let own_pieces = pos.color[color];
            let enemy_pieces = pos.color[other];

            // 1. Duyệt qua toàn bộ các loại quân của phe hiện tại (Sĩ, Tượng, Mã, Xe, Pháo, Tốt)
            let mut piece_type = 1usize; // Bắt đầu từ Sĩ (1), Tượng (2), Mã (3), Xe (4), Pháo (5), Tốt (6)
            while piece_type <= 6 {
                let mut bb = pos.piece[color * 7 + piece_type];
                while let Some(sq) = bb.pop() {
                    let mut attackers = 0u32;
                    let mut has_pawn_attacker = false;

                    // Kiểm tra bị tấn công bởi Tốt địch
                    let enemy_pawns = pos.piece[other * 7 + 6];
                    let pawn_attacks = lookup::PAWN[color][sq.0 as usize] & enemy_pawns;
                    if !pawn_attacks.is_empty() {
                        attackers += pawn_attacks.count();
                        has_pawn_attacker = true;
                    }

                    // Kiểm tra bị tấn công bởi Mã địch
                    let enemy_knights = pos.piece[other * 7 + 3];
                    let knight_attacks = lookup::KNIGHT[sq.0 as usize] & enemy_knights;
                    let mut k_iter = knight_attacks;
                    while let Some(k_sq) = k_iter.pop() {
                        let leg = lookup::leg(k_sq.0 as usize, sq.0 as usize);
                        if leg != 255 && !occupied.test(Square(leg)) {
                            attackers += 1;
                        }
                    }

                    // Kiểm tra bị tấn công bởi Xe địch
                    let enemy_rooks = pos.piece[other * 7 + 4];
                    let rook_attacks = lookup::rook(sq.0, occupied, Bitboard::empty()) & enemy_rooks;
                    if !rook_attacks.is_empty() {
                        attackers += rook_attacks.count();
                    }

                    // Kiểm tra bị tấn công bởi Pháo địch
                    let enemy_cannons = pos.piece[other * 7 + 5];
                    let cannon_attacks = lookup::cannon(sq.0, occupied, enemy_pieces) & enemy_cannons;
                    if !cannon_attacks.is_empty() {
                        attackers += cannon_attacks.count();
                    }

                    if attackers > 0 {
                        // Kiểm tra số quân nhà bảo vệ (Defenders)
                        let mut solid_defenders = 0u32;

                        // Bảo vệ bởi Tốt nhà
                        let own_pawns = pos.piece[color * 7 + 6];
                        let p_defs = lookup::PAWN[other][sq.0 as usize] & own_pawns;
                        solid_defenders += p_defs.count();

                        // Bảo vệ bởi Sĩ nhà
                        let own_advisors = pos.piece[color * 7 + 1];
                        let a_defs = lookup::ADVISOR[color][sq.0 as usize] & own_advisors;
                        solid_defenders += a_defs.count();

                        // Bảo vệ bởi Tượng nhà
                        let own_bishops = pos.piece[color * 7 + 2];
                        let mut b_iter = lookup::ELEPHANT[color][sq.0 as usize] & own_bishops;
                        while let Some(b_sq) = b_iter.pop() {
                            let eye = lookup::eye(b_sq.0 as usize, sq.0 as usize);
                            if eye != 255 && !occupied.test(Square(eye)) {
                                solid_defenders += 1;
                            }
                        }

                        // Bảo vệ bởi Mã nhà
                        let own_knights = pos.piece[color * 7 + 3];
                        let mut kn_iter = lookup::KNIGHT[sq.0 as usize] & own_knights;
                        while let Some(kn_sq) = kn_iter.pop() {
                            let leg = lookup::leg(kn_sq.0 as usize, sq.0 as usize);
                            if leg != 255 && !occupied.test(Square(leg)) {
                                solid_defenders += 1;
                            }
                        }

                        // Bảo vệ bởi Xe nhà
                        let own_rooks = pos.piece[color * 7 + 4];
                        let r_defs = lookup::rook(sq.0, occupied, Bitboard::empty()) & own_rooks;
                        solid_defenders += r_defs.count();

                        // Bảo vệ bởi Tướng nhà trong Cung
                        let own_king = pos.king[color];
                        if (own_king as usize) < 90 && lookup::KING[color][sq.0 as usize].test(Square(own_king as u8)) {
                            solid_defenders += 1;
                        }

                        // Bảo vệ bởi Pháo nhà qua ngòi (phòng thủ gián tiếp)
                        let own_cannons = pos.piece[color * 7 + 5];
                        let c_defs = lookup::cannon(sq.0, occupied, own_pieces) & own_cannons;
                        let cannon_defenders = c_defs.count();

                        let defenders = solid_defenders + cannon_defenders;

                        // 1. Trường hợp quân hoàn toàn không có người bảo vệ (Hanging)
                        if defenders == 0 {
                            // 1. Quân hoàn toàn bị treo (0 quân nhà bảo vệ)
                            let is_rook_threat = !rook_attacks.is_empty();
                            match piece_type {
                                1 => { // Sĩ treo
                                    let penalty = if is_rook_threat { 300 } else { 260 };
                                    mg -= sign * penalty;
                                    eg -= sign * (penalty + 60);
                                }
                                2 => { // Tượng treo
                                    let penalty = if is_rook_threat { 300 } else { 260 };
                                    mg -= sign * penalty;
                                    eg -= sign * (penalty + 60);
                                }
                                6 => { // Tốt treo
                                    mg -= sign * 140;
                                    eg -= sign * 200;
                                }
                                3 => { // Mã treo
                                    let penalty = if is_rook_threat { 460 } else { 400 };
                                    mg -= sign * penalty;
                                    eg -= sign * (penalty + 70);
                                }
                                5 => { // Pháo treo
                                    let penalty = if is_rook_threat { 480 } else { 420 };
                                    mg -= sign * penalty;
                                    eg -= sign * (penalty + 70);
                                }
                                4 => { // Xe treo
                                    mg -= sign * 900;
                                    eg -= sign * 950;
                                }
                                _ => {}
                            }
                        } else if solid_defenders == 0 && cannon_defenders > 0 && piece_type < 6 {
                            // Quân lớn chỉ được giữ qua ngòi Pháo (dễ bị lôi kéo quân thủ làm vỡ trận)
                            mg -= sign * 100;
                            eg -= sign * 150;
                        } else if has_pawn_attacker && piece_type < 6 {
                            // 2. Quân lớn (Sĩ, Tượng, Mã, Pháo, Xe) bị Tốt địch tấn công trực diện (kể cả có bảo vệ thì vẫn lỗ)
                            match piece_type {
                                1 | 2 => { mg -= sign * 180; eg -= sign * 220; } // Sĩ/Tượng bị Tốt đe dọa
                                3 => { mg -= sign * 250; eg -= sign * 280; } // Mã bị Tốt đe dọa
                                5 => { mg -= sign * 280; eg -= sign * 320; } // Pháo bị Tốt đe dọa
                                4 => { mg -= sign * 650; eg -= sign * 750; } // Xe bị Tốt đe dọa
                                _ => {}
                            }
                        } else if attackers > defenders {
                            // 3. Số quân tấn công áp đảo số quân phòng thủ
                            mg -= sign * 120;
                            eg -= sign * 180;
                        }
                    }
                }
                piece_type += 1;
            }

            color += 1;
        }

        (mg, eg)
    }
}
