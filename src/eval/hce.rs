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
        let mut total = 0i32;
        let mut role = 0usize;
        while role < 7 {
            let count = (pos.counts[role] + pos.counts[role + 7]) as i32;
            total += count * Self::WEIGHT[role];
            role += 1;
        }
        if total > 32 {
            32
        } else {
            total
        }
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
            0, -10, 0, 0, 0, 0, 0, -10, 0,
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
            0, 5, 10, 15, 10, 15, 10, 5, 0,
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

            // 1. Xe (Rook)
            let mut rooks = pos.piece[color * 7 + 4];
            while let Some(sq) = rooks.pop() {
                let moves = lookup::rook(sq.0, pos.occupied, pos.color[color]);
                let count = moves.count() as i32;
                mg += sign * count * 6;
                eg += sign * count * 8;
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
            let advisors = pos.counts[color * 7 + 1];
            let bishops = pos.counts[color * 7 + 2];

            let guards = advisors + bishops;
            if guards == 4 {
                mg += sign * 150;
                eg += sign * 100;
            } else if guards < 2 {
                mg -= sign * 100;
                eg -= sign * 150;
            }

            let square = pos.king[color];
            let base = if color == 0 { 0 } else { 9 };
            let rank = square / 9;

            if rank != base {
                mg -= sign * 60;
                eg -= sign * 80;
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
                    mg += sign * 50;
                    eg += sign * 100;

                    let depth = if color == 0 { rank - 4 } else { 5 - rank };
                    mg += sign * (depth as i32 * 20);
                    eg += sign * (depth as i32 * 40);

                    if file >= 3 && file <= 5 {
                        mg += sign * 40;
                        eg += sign * 80;
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
        let mut mg = 0i32;
        let mut eg = 0i32;

        let mut role = 0usize;
        while role < 14 {
            let color = role / 7;
            let piece = role % 7;
            let sign = if color == 0 { 1 } else { -1 };

            let mut bb = pos.piece[role];
            while let Some(sq) = bb.pop() {
                let (mid, end) = Table::get(piece, color, sq.0);

                mg += sign * (Value::MG[piece] + mid);
                eg += sign * (Value::EG[piece] + end);
            }

            role += 1;
        }

        let (mid, end) = Mobility::evaluate(pos);
        mg += mid;
        eg += end;

        let (mid, end) = King::evaluate(pos);
        mg += mid;
        eg += end;

        let (mid, end) = Pawn::evaluate(pos);
        mg += mid;
        eg += end;

        let (mid, end) = crate::eval::trap::Trap::evaluate(pos);
        mg += mid;
        eg += end;

        let phase = Value::phase(pos);
        Value::taper(mg, eg, phase)
    }
}

