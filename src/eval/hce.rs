// ============================================================================
// MODULE HCE: BỘ ĐÁNH GIÁ THỦ CÔNG DỰ PHÒNG (HAND-CRAFTED EVALUATION & TAPERED EVAL)
// ============================================================================
// `hce.rs` triển khai bộ đánh giá thế cờ dựa trên tri thức chuyên gia Cờ Tướng:
// - `Value`: Giá trị cơ bản của 7 loại quân trong Trung cuộc (MG) và Tàn cuộc (EG).
// - `Table`: Bảng giá trị ô vị trí (Piece-Square Table - PST) cho 7 loại quân.
// - `Mobility`: Đánh giá độ cơ động của Xe, Pháo, Mã.
// - `King`: Đánh giá an toàn Tướng và lực lượng Sĩ Tượng bảo vệ.
// - `Pawn`: Đánh giá Tốt qua sông và Tốt nhập Cung.
// - `Tapered Evaluation`: Nối suy tuyến tính giữa Trung cuộc và Tàn cuộc dựa trên giai đoạn cờ `phase`.
// ============================================================================

use crate::board::{Position, Square};
use crate::movegen::lookup;

/// Struct `Value` quản lý trọng số điểm của 7 loại quân cờ.
pub struct Value;

impl Value {
    /// Giá trị cơ bản Trung cuộc (Middle Game) [Tướng, Sĩ, Tượng, Mã, Xe, Pháo, Tốt]
    pub const MG: [i32; 7] = [20000, 200, 200, 450, 900, 450, 100];
    /// Giá trị cơ bản Tàn cuộc (End Game) [Tướng, Sĩ, Tượng, Mã, Xe, Pháo, Tốt]
    pub const EG: [i32; 7] = [20000, 250, 250, 400, 1000, 350, 120];
    /// Trọng số giai đoạn ván đấu cho từng loại quân [0..32]
    pub const WEIGHT: [i32; 7] = [0, 2, 2, 3, 6, 3, 0];

    /// Tính toán chỉ số giai đoạn ván đấu `phase` ($0 \le phase \le 32$) từ số lượng quân hiện có trên bàn.
    #[inline(always)]
    pub fn phase(pos: &Position) -> i32 {
        let advisor = (pos.counts[1] + pos.counts[8]) as i32 * 2;
        let bishop = (pos.counts[2] + pos.counts[9]) as i32 * 2;
        let knight = (pos.counts[3] + pos.counts[10]) as i32 * 3;
        let rook = (pos.counts[4] + pos.counts[11]) as i32 * 6;
        let cannon = (pos.counts[5] + pos.counts[12]) as i32 * 3;
        (advisor + bishop + knight + rook + cannon).min(32)
    }

    /// Nội suy điểm Tapered Evaluation giữa Trung cuộc (mg) và Tàn cuộc (eg) theo `phase`.
    #[inline(always)]
    pub fn taper(mg: i32, eg: i32, phase: i32) -> i32 {
        (mg * phase + eg * (32 - phase)) / 32
    }
}

/// Struct `Table` lưu trữ Bảng vị trí quân (Piece-Square Tables - PST) 90 ô bàn cờ Cờ Tướng.
pub struct Table;

impl Table {
    /// Mảng vị trí 90 ô Trung cuộc cho 7 loại quân
    pub const MG: [[i32; 90]; 7] = [
        // King (Tướng)
        [
            0, 0, 0, -10, -10, -10, 0, 0, 0,
            0, 0, 0, -10, -10, -10, 0, 0, 0,
            0, 0, 0, -10, -10, -10, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        // Advisor (Sĩ)
        [
            0, 0, 0, 0, 10, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 10, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        // Bishop (Tượng)
        [
            0, 0, 10, 0, 0, 0, 10, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            20, 0, 0, 0, 30, 0, 0, 0, 20,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 10, 0, 0, 0, 10, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        // Knight (Mã)
        [
            0, -25, 0, 0, 0, 0, 0, -25, 0,
            10, 15, 20, 20, 15, 20, 20, 15, 10,
            10, 20, 30, 35, 30, 35, 30, 20, 10,
            15, 25, 35, 40, 35, 40, 35, 25, 15,
            15, 25, 35, 40, 35, 40, 35, 25, 15,
            20, 30, 40, 50, 40, 50, 40, 30, 20,
            20, 30, 40, 50, 40, 50, 40, 30, 20,
            15, 25, 30, 35, 30, 35, 30, 25, 15,
            10, 15, 20, 20, 15, 20, 20, 15, 10,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        // Rook (Xe)
        [
            -30, 15, 20, 25, 20, 25, 20, 15, -30,
            10, 15, 20, 25, 25, 25, 20, 15, 10,
            10, 15, 20, 25, 25, 25, 20, 15, 10,
            15, 20, 25, 30, 30, 30, 25, 20, 15,
            15, 20, 25, 30, 30, 30, 25, 20, 15,
            20, 25, 30, 35, 35, 35, 30, 25, 20,
            20, 25, 30, 35, 35, 35, 30, 25, 20,
            15, 20, 25, 30, 30, 30, 25, 20, 15,
            10, 20, 20, 25, 25, 25, 20, 20, 10,
            0, 10, 15, 20, 15, 20, 15, 10, 0,
        ],
        // Cannon (Pháo)
        [
            0, 0, 10, 15, 20, 15, 10, 0, 0,
            5, 10, 10, 15, 15, 15, 10, 10, 5,
            0, 10, 15, 20, 25, 20, 15, 10, 0,
            0, 10, 15, 20, 25, 20, 15, 10, 0,
            0, 5, 10, 15, 20, 15, 10, 5, 0,
            0, 5, 10, 15, 20, 15, 10, 5, 0,
            0, 10, 15, 20, 25, 20, 15, 10, 0,
            0, 10, 15, 20, 25, 20, 15, 10, 0,
            5, 10, 10, 15, 15, 15, 10, 10, 5,
            0, 0, 10, 15, 20, 15, 10, 0, 0,
        ],
        // Pawn (Tốt)
        [
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            10, 0, 20, 0, 30, 0, 20, 0, 10,
            20, 20, 30, 40, 50, 40, 30, 20, 20,
            30, 35, 45, 60, 70, 60, 45, 35, 30,
            40, 50, 60, 80, 90, 80, 60, 50, 40,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
    ];

    /// Mảng vị trí 90 ô Tàn cuộc (dùng chung với Trung cuộc)
    pub const EG: [[i32; 90]; 7] = Self::MG;

    /// Truy xuất cặp điểm PST (MG, EG) cho loại quân `role`, phe `color`, tại ô `square`.
    #[inline(always)]
    pub fn get(role: usize, color: usize, square: u8) -> (i32, i32) {
        let sq = Square(square);
        let index = if color == 0 {
            sq.index()
        } else {
            sq.flip().index()
        };
        (Self::MG[role][index], Self::EG[role][index])
    }
}

/// Struct `Mobility` tính toán điểm cơ động di chuyển của Xe, Pháo, Mã.
pub struct Mobility;

impl Mobility {
    /// Tính toán cặp điểm cơ động (MG, EG) cho cả 2 phe.
    #[inline(always)]
    pub fn evaluate(pos: &Position) -> (i32, i32) {
        let mut mg = 0i32;
        let mut eg = 0i32;

        let mut color = 0usize;
        while color < 2 {
            let sign = if color == 0 { 1 } else { -1 };
            let other = 1 - color;

            // 1. Xe (Rook)
            let mut rooks = pos.piece[color * 7 + 4];
            while let Some(sq) = rooks.pop() {
                let moves = lookup::rook(sq.0, pos.occupied, pos.color[1 - color]);
                let count = moves.count() as i32;
                mg += sign * count * 6;
                eg += sign * count * 8;

                // Xe lọt sâu vào trận địa áp sát cung Tướng đối phương
                let r_rank = sq.rank();
                let r_file = sq.file();
                let is_deep = if color == 0 { r_rank >= 7 } else { r_rank <= 2 };
                if is_deep {
                    mg += sign * 150;
                    eg += sign * 300;
                }

                // Thưởng Xe chiếm giữ Trung Lộ (Cột 4) và Sườn Cung (Cột 3, 5)
                if r_file == 4 {
                    mg += sign * 90;
                    eg += sign * 120;
                } else if r_file == 3 || r_file == 5 {
                    mg += sign * 50;
                    eg += sign * 70;
                }

                // Phạt Xe bỏ trận địa sang góc biên (Cột 0, 8) khi đối phương còn nhiều quân tấn công
                let enemy_heavies = pos.counts[other * 7 + 4] + pos.counts[other * 7 + 5];
                if (r_file == 0 || r_file == 8) && enemy_heavies > 0 {
                    mg -= sign * 60;
                    eg -= sign * 90;
                }
            }

            // 2. Pháo (Cannon)
            let mut cannons = pos.piece[color * 7 + 5];
            while let Some(sq) = cannons.pop() {
                let moves = lookup::cannon(sq.0, pos.occupied, pos.color[1 - color]);
                let count = moves.count() as i32;
                mg += sign * count * 4;
                eg += sign * count * 5;
            }

            // 3. Mã (Knight)
            let mut knights = pos.piece[color * 7 + 3];
            while let Some(sq) = knights.pop() {
                let attacks = lookup::knight(sq.0 as usize);
                let mut count = 0i32;
                let mut targets = attacks;
                while let Some(to) = targets.pop() {
                    let leg = lookup::leg(sq.0 as usize, to.0 as usize);
                    if leg < 90 && pos.grid[leg as usize] == 14 {
                        count += 1;
                    }
                }
                mg += sign * count * 5;
                eg += sign * count * 7;

                // Phạt Mã bị cản chân/tê liệt (Knight Pin / Immobilization Penalty)
                if count == 0 {
                    mg -= sign * 80;
                    eg -= sign * 130;
                } else if count <= 2 {
                    mg -= sign * 35;
                    eg -= sign * 55;
                }

                // Phạt Mã nhập biên lộ (Edge Knight Trap Penalty) tại cột 0 (a), cột 8 (i) hoặc các góc h9, b9, h0, b0
                let n_rank = sq.rank();
                let n_file = sq.file();
                let is_edge = n_file == 0 || n_file == 8 || (n_rank == 0 && (n_file == 1 || n_file == 7)) || (n_rank == 9 && (n_file == 1 || n_file == 7));
                let crossed_river = if color == 0 { n_rank >= 5 } else { n_rank <= 4 };
                if is_edge && !crossed_river {
                    mg -= sign * 150;
                    eg -= sign * 200;
                }

                // Mã qua sông chiếm lĩnh tiền đồn (Knight Outpost across River)
                if crossed_river {
                    mg += sign * 70;
                    eg += sign * 110;
                }

                // Mã áp sát Cung Tướng đối phương (Ngọa Tào / Điếu Ngư / Mã Sườn)
                let is_deep_knight = if color == 0 { n_rank >= 7 } else { n_rank <= 2 };
                if is_deep_knight {
                    mg += sign * 160;
                    eg += sign * 260;
                }
            }

            color += 1;
        }

        (mg, eg)
    }
}

/// Struct `King` đánh giá độ an toàn của Tướng và hệ thống phòng thủ Sĩ Tượng.
pub struct King;

impl King {
    /// Tính toán điểm an toàn Tướng (MG, EG) cho cả 2 phe.
    #[inline(always)]
    pub fn evaluate(pos: &Position) -> (i32, i32) {
        let mut mg = 0i32;
        let mut eg = 0i32;

        let mut color = 0usize;
        while color < 2 {
            let sign = if color == 0 { 1 } else { -1 };
            let other = 1 - color;
            let advisors = pos.counts[color * 7 + 1];
            let bishops = pos.counts[color * 7 + 2];

            let guards = advisors + bishops;
            if guards == 4 {
                mg += sign * 120;
                eg += sign * 80;
            } else {
                // Khuyết Tượng / Khuyết Sĩ khi đối phương còn Xe/Pháo/Mã
                let enemy_attackers = pos.counts[other * 7 + 4] + pos.counts[other * 7 + 5] + pos.counts[other * 7 + 3];
                if enemy_attackers > 0 {
                    if bishops == 1 {
                        mg -= sign * 80;
                        eg -= sign * 120;
                    } else if bishops == 0 {
                        mg -= sign * 160;
                        eg -= sign * 240;
                    }

                    if advisors == 1 {
                        mg -= sign * 90;
                        eg -= sign * 140;
                    } else if advisors == 0 {
                        mg -= sign * 180;
                        eg -= sign * 260;
                    }
                }
            }

            let square = pos.king[color];
            let base = if color == 0 { 0 } else { 9 };
            let rank = square / 9;
            let file = square % 9;

            // Phạt Tướng rời hàng đáy lên lầu khi đối phương còn quân tấn công (Exposed High-Floor King Penalty)
            let enemy_attackers = pos.counts[other * 7 + 4] + pos.counts[other * 7 + 5] + pos.counts[other * 7 + 3];
            if rank != base {
                if enemy_attackers > 0 {
                    let floor_dist = if color == 0 { rank - base } else { base - rank };
                    if floor_dist == 1 {
                        // Tướng lên tầng 2 (e1 / e8)
                        mg -= sign * 180;
                        eg -= sign * 260;
                    } else {
                        // Tướng lên tầng 3 (e2 / e7 - Tướng Lên Lầu Ba)
                        mg -= sign * 320;
                        eg -= sign * 480;
                    }
                } else {
                    mg -= sign * 40;
                    eg -= sign * 60;
                }
            }

            // Tướng bị lệch tâm cung cờ khi đối phương còn nhiều Xe Pháo
            let enemy_heavies = pos.counts[other * 7 + 4] + pos.counts[other * 7 + 5];
            if file != 4 && enemy_heavies > 0 {
                mg -= sign * 50;
                eg -= sign * 40;
            }

            // Phòng thủ xâm nhập Cung Tướng (Palace Sector Infiltration Defense)
            // Xe, Pháo, hoặc Mã đối phương lọt vào khu vực Cung Tướng (hàng 0..2 của Red hoặc hàng 7..9 của Black)
            let mut enemy_rooks = pos.piece[other * 7 + 4];
            while let Some(r_sq) = enemy_rooks.pop() {
                let r_rank = r_sq.rank();
                let in_my_palace = if color == 0 { r_rank <= 2 } else { r_rank >= 7 };
                if in_my_palace {
                    mg -= sign * 140;
                    eg -= sign * 220;
                }
            }

            let mut enemy_cannons = pos.piece[other * 7 + 5];
            while let Some(c_sq) = enemy_cannons.pop() {
                let c_rank = c_sq.rank();
                let in_my_palace = if color == 0 { c_rank <= 2 } else { c_rank >= 7 };
                if in_my_palace {
                    mg -= sign * 80;
                    eg -= sign * 130;
                }
            }

            let mut enemy_knights = pos.piece[other * 7 + 3];
            while let Some(n_sq) = enemy_knights.pop() {
                let n_rank = n_sq.rank();
                let in_my_palace = if color == 0 { n_rank <= 2 } else { n_rank >= 7 };
                if in_my_palace {
                    mg -= sign * 130;
                    eg -= sign * 200;
                }
            }

            // Đe dọa Xe lộ mở nhắm thẳng hàng đáy (Open File Back-Rank Rook Threat)
            let mut enemy_rooks_ray = pos.piece[other * 7 + 4];
            while let Some(r_sq) = enemy_rooks_ray.pop() {
                let r_file = r_sq.file();
                let r_rank = r_sq.rank();
                let is_aimed_at_home = if color == 0 { r_rank >= 3 } else { r_rank <= 6 };
                if is_aimed_at_home {
                    let base_sq = if color == 0 { r_file } else { 9 * 9 + r_file };
                    let between = lookup::between(r_sq.0 as usize, base_sq as usize);
                    let obstacles = (between & pos.occupied).count();
                    if obstacles == 0 {
                        mg -= sign * 120;
                        eg -= sign * 180;
                    }
                }
            }

            color += 1;
        }

        (mg, eg)
    }
}

/// Struct `Pawn` đánh giá vị trí Tốt qua sông và Tốt dọa Tướng.
pub struct Pawn;

impl Pawn {
    /// Tính toán điểm Tốt qua sông (MG, EG) cho cả 2 phe.
    #[inline(always)]
    pub fn evaluate(pos: &Position) -> (i32, i32) {
        let mut mg = 0i32;
        let mut eg = 0i32;

        let mut color = 0usize;
        while color < 2 {
            let sign = if color == 0 { 1 } else { -1 };
            let mut pawns = pos.piece[color * 7 + 6];

            while let Some(sq) = pawns.pop() {
                let rank = sq.rank();
                let file = sq.file();

                let crossed = if color == 0 { rank >= 5 } else { rank <= 4 };
                if crossed {
                    mg += sign * 140;
                    eg += sign * 260;

                    let depth = if color == 0 { rank - 4 } else { 5 - rank };
                    mg += sign * (depth as i32 * 60);
                    eg += sign * (depth as i32 * 120);

                    // Tốt áp sát cột trung tâm hoặc sườn cung (cột 2, 3, 4, 5, 6)
                    if file >= 2 && file <= 6 {
                        mg += sign * 120;
                        eg += sign * 220;
                    }

                    // Tốt đã lọt vào chính Cung (cột 3, 4, 5) hoặc kẹp Sườn Cung (cột 2, 6) ở hàng 7, 8, 9
                    let in_deep_palace = if color == 0 { rank >= 7 && file >= 3 && file <= 5 } else { rank <= 2 && file >= 3 && file <= 5 };
                    let on_palace_flank = if color == 0 { rank >= 7 && (file == 2 || file == 6) } else { rank <= 2 && (file == 2 || file == 6) };
                    if in_deep_palace {
                        mg += sign * 350;
                        eg += sign * 650;
                    } else if on_palace_flank {
                        mg += sign * 250;
                        eg += sign * 480;
                    }
                }
            }

            color += 1;
        }

        (mg, eg)
    }
}

/// Struct `Hce` bọc toàn bộ logic đánh giá thủ công Hand-Crafted Evaluation.
#[derive(Clone, Copy, Debug, Default)]
pub struct Hce;

impl Hce {
    /// Khởi tạo mặc định Hce.
    #[inline(always)]
    pub const fn new() -> Self {
        Self
    }

    /// Đánh giá điểm số tổng thể của vị trí thế cờ `pos` (trả về điểm centipawn từ góc nhìn lượt đi).
    /// Ép buộc inlining `#[inline(always)]` triệt tiêu overhead gọi hàm trên hot path tìm kiếm.
    #[inline(always)]
    pub fn evaluate(&self, pos: &Position) -> i32 {
        let weights = crate::system::Weights::grandmaster();

        // 1. Fast-Path Thẩm định tàn cuộc lý thuyết O(1)
        if let Some(endgame_score) = crate::system::EndgameSystem::probe(pos, &weights) {
            return if pos.side == 0 { endgame_score } else { -endgame_score };
        }

        // 2. Tận dụng trực tiếp điểm Material + PST được tích lũy vi phân O(1) trong Position
        let mut mg = pos.score_mg;
        let mut eg = pos.score_eg;

        let (mid, end) = Mobility::evaluate(pos);
        mg += mid;
        eg += end;

        let (mid, end) = King::evaluate(pos);
        mg += mid;
        eg += end;

        let (mid, end) = Pawn::evaluate(pos);
        mg += mid;
        eg += end;

        let (mid, end) = crate::system::RookSystem::evaluate(pos, &weights);
        mg += mid;
        eg += end;

        let (mid, end) = crate::system::KingSystem::evaluate(pos, &weights);
        mg += mid;
        eg += end;

        let (mid, end) = crate::system::PawnSystem::evaluate(pos, &weights);
        mg += mid;
        eg += end;

        let (mid, end) = crate::system::KnightSystem::evaluate(pos, &weights);
        mg += mid;
        eg += end;

        let (mid, end) = crate::system::TrapSystem::evaluate(pos, &weights);
        mg += mid;
        eg += end;

        // 3. Tích hợp Hệ thống Kỹ Năng Quân Cờ Tinh Hoa & Tổ Hợp Siêu Cấp Thế Giới
        crate::system::Skill::evaluate(pos, &weights, &mut mg, &mut eg);
        crate::system::Combo::evaluate(pos, &weights, &mut mg, &mut eg);

        let phase = Value::phase(pos);
        Value::taper(mg, eg, phase)
    }
}

