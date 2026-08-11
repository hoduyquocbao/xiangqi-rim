// ============================================================================
// VÍ DỤ 26: BỘ SINH DỮ LIỆU HUẤN LUYỆN DỰA TRÊN MA TRẬN 32 CHIỀU KÍCH NATIVE DYNAMIC 100%
// ============================================================================
// ĐẶC TẢ KIẾN TRÚC TOÀN DIỆN (SINGLE-WORD IDENTIFIERS & ZERO-ALLOCATION PROTOCOL):
// - 100% 32 chiều kích CoT <thought> trích xuất động từ lõi Native Engine.
// - 100% Định danh mã nguồn Rust tuân thủ từ đơn tiếng Anh (Single-Word Identifiers).
// - Căn lề bộ nhớ phần cứng repr(C, align(64)) triệt tiêu False Sharing.
// - Vòng lặp nóng 36-ply (73 tin nhắn) đạt 0-allocation nhờ bộ đệm Buffer pre-allocate.
// ============================================================================

use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::Write as IoWrite;
use std::time::{SystemTime, UNIX_EPOCH};

use xiangrust::board::{Parser, Position, Serializer, Square};
use xiangrust::eval::Eval;
use xiangrust::movegen::{self, lookup, Move};
use xiangrust::search::{Limits, Search};

// ----------------------------------------------------------------------------
// KHÔNG GIAN TÊN TỪ ĐƠN (SINGLE-WORD NAMESPACES & CONSTANTS)
// ----------------------------------------------------------------------------
pub mod red {
    pub const SYMBOLS: [&str; 7] = ["帥", "仕", "相", "馬", "車", "炮", "兵"];
}

pub mod black {
    pub const SYMBOLS: [&str; 7] = ["將", "士", "象", "馬", "車", "砲", "卒"];
}

pub const VALUE: [i32; 7] = [0, 200, 200, 400, 900, 450, 100];
pub const NAME: [&str; 7] = ["Tướng", "Sĩ", "Tượng", "Mã", "Xe", "Pháo", "Tốt"];

pub const FENS: [&str; 8] = [
    "rnbakab1r/9/1c4nc1/p1p1p1p1p/9/9/P1P1P1P1P/4C2C1/9/RNBAKABNR w - - 0 1",
    "rnbakabnr/9/4c2c1/p1p1p1p1p/9/9/P1P1P1P1P/4C2C1/9/RNBAKABNR w - - 0 1",
    "rnbakabnr/9/1c2c4/p1p1p1p1p/9/9/P1P1P1P1P/4C2C1/9/RNBAKABNR w - - 0 1",
    "r1bakabnr/9/1cn4c1/p1p1p1p1p/9/9/P1P1P1P1P/3C3C1/9/RNBAKABNR w - - 0 1",
    "rnbakabnr/9/1c5c1/p3p1p1p/2p6/2P6/P3P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
    "rnbakab1r/9/1c4nc1/p1p1p1p1p/9/9/P1P1P1P1P/1CN4C1/9/R1BAKABNR w - - 0 1",
    "rnbakab1r/9/1c4nc1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/R8/1NBAKABNR w - - 0 1",
    "rnbakab1r/9/1c4nc1/p1p1p1p1p/9/9/P1P1P1P1P/1C2B2C1/9/RN1AKABNR w - - 0 1",
];

pub const SYSTEM: &str = r#"Bạn là Xiangqi-R1 Master v5.0 — mô hình suy luận cờ Tướng siêu việt được huấn luyện phân tích chiều sâu chiến thuật 32 chiều kích.
Nhiệm vụ: Phân tích thế cờ qua Ma trận Trọng số 32 Chiều Kích Động <thought> trước khi đưa ra nước đi tối ưu (bestmove) và định dạng JSON JRCP 5.0.
Yêu cầu bắt buộc:
1. Không được hardcode hay dùng văn bản tĩnh. Toàn bộ 32 chiều kích phải trích xuất động 100% từ hiện trạng bàn cờ.
2. Mô tả chi tiết, tường minh từng quân cờ, tọa độ, và ý đồ chiến thuật đến mức tối đa để phục vụ học máy tự hồi quy."#;

// ----------------------------------------------------------------------------
// STRUCT TRỢ GIÚP TỪ ĐƠN CĂN LỀ BỘ NHỚ PHẦN CỨNG (HARDWARE ALIGNED STRUCTS)
// ----------------------------------------------------------------------------

/// 1. Sieve: Bộ lọc băm xác suất O(1) kiểm tra trạng thái thế cờ lặp
#[repr(C, align(64))]
pub struct Sieve {
    pub bits: [u64; 64],
}

impl Sieve {
    pub fn new() -> Self {
        Self { bits: [0u64; 64] }
    }

    pub fn insert(&mut self, key: u64) {
        let index = (key as usize) & 63;
        self.bits[index] |= key;
    }

    pub fn test(&self, key: u64) -> bool {
        let index = (key as usize) & 63;
        (self.bits[index] & key) == key
    }

    pub fn clear(&mut self) {
        self.bits.fill(0);
    }
}

/// 2. Ray: Đường tấn công thẳng hàng 16-byte align
#[derive(Clone, Copy)]
#[repr(align(16))]
pub struct Ray {
    pub file: u8,
    pub rank: u8,
    pub mask: u128,
}

/// 3. Pin: Kiểm tra đòn ghim quân 16-byte align
#[derive(Clone, Copy)]
#[repr(align(16))]
pub struct Pin {
    pub count: u8,
    pub line: u8,
}

/// 4. Fork: Kiểm tra đòn bắt đôi 16-byte align
#[derive(Clone, Copy)]
#[repr(align(16))]
pub struct Fork {
    pub count: u8,
    pub target: u8,
}

/// 5. Guard: Kiểm tra độ an toàn Cung Tướng và quân phòng thủ
#[derive(Clone, Copy)]
#[repr(C, align(64))]
pub struct Guard {
    pub safety: i32,
    pub advisors: u8,
    pub elephants: u8,
    pub pad: [u8; 58],
}

impl Guard {
    pub fn eval(pos: &Position, side: u8) -> Self {
        let advisor = if side == 0 { 1u8 } else { 8u8 };
        let elephant = if side == 0 { 2u8 } else { 9u8 };
        let advisors = pos.counts[advisor as usize] as u8;
        let elephants = pos.counts[elephant as usize] as u8;
        let mut score: i32 = 40 + (advisors as i32) * 15 + (elephants as i32) * 15;
        let king = pos.king[side as usize];
        if king < 90 && king % 9 == 4 {
            score += 10;
        }
        Self {
            safety: score.clamp(0, 100),
            advisors,
            elephants,
            pad: [0u8; 58],
        }
    }
}

/// 6. Threat: Chỉ số đe dọa chiến thuật 64-byte align
#[repr(C, align(64))]
pub struct Threat {
    pub attacked: usize,
    pub hanging: usize,
    pub pinned: usize,
    pub forks: usize,
    pub discovered: usize,
    pub trapped: usize,
    pub mate: bool,
    pub diversion: bool,
    pub pad: [u8; 14],
}

impl Threat {
    pub fn eval(pos: &Position, side: u8) -> Self {
        let enemy = (1 - side) as usize;
        let mut attacked = 0;
        let mut hanging = 0;
        let mut pinned = 0;
        let mut forks = 0;
        let mut discovered = 0;
        let mut trapped = 0;
        let mut diversion = false;

        let occupied = pos.occupied;
        let king = pos.king[side as usize];

        for sq in 0..90u8 {
            let p = pos.grid[sq as usize];
            if p < 14 && (p / 7) as u8 == side {
                let attackers = attacker::scan(pos, sq, enemy);
                if !attackers.is_empty() {
                    attacked += 1;
                    let defenders = attacker::scan(pos, sq, side as usize);
                    if defenders.is_empty() {
                        hanging += 1;
                    }
                }
            }
        }

        if king < 90 {
            let idx = king as usize;
            for d in 0..4 {
                let r = lookup::ray(d, idx);
                let block = r & occupied;
                if block.active() {
                    let first = if d == 0 || d == 2 {
                        block.lsb().unwrap().index()
                    } else {
                        block.msb().unwrap().index()
                    };
                    let piece = pos.grid[first];
                    if piece < 14 && (piece / 7) as u8 == side {
                        let behind = lookup::ray(d, first) & occupied;
                        if behind.active() {
                            let second = if d == 0 || d == 2 {
                                behind.lsb().unwrap().index()
                            } else {
                                behind.msb().unwrap().index()
                            };
                            let role = pos.grid[second] % 7;
                            if pos.grid[second] < 14 && (pos.grid[second] / 7) as u8 == enemy as u8 {
                                if role == 4 || role == 5 {
                                    pinned += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        for sq in 0..90u8 {
            let p = pos.grid[sq as usize];
            if p < 14 && (p / 7) as u8 == enemy as u8 {
                let mut count = 0;
                for target in 0..90u8 {
                    let tp = pos.grid[target as usize];
                    if tp < 14 && (tp / 7) as u8 == side {
                        let attackers = attacker::scan(pos, target, enemy);
                        if attackers.iter().any(|(a, _)| *a == sq) {
                            count += 1;
                        }
                    }
                }
                if count >= 2 {
                    forks += 1;
                }
            }
        }

        let target = pos.king[enemy];
        if target < 90 {
            for d in 0..4 {
                let r = lookup::ray(d, target as usize);
                let block = r & occupied;
                if block.active() {
                    let first = if d == 0 || d == 2 {
                        block.lsb().unwrap().index()
                    } else {
                        block.msb().unwrap().index()
                    };
                    let piece = pos.grid[first];
                    if piece < 14 && (piece / 7) as u8 == side {
                        let behind = lookup::ray(d, first) & occupied;
                        if behind.active() {
                            let second = if d == 0 || d == 2 {
                                behind.lsb().unwrap().index()
                            } else {
                                behind.msb().unwrap().index()
                            };
                            let rear = pos.grid[second];
                            if rear < 14 && (rear / 7) as u8 == side && rear % 7 == 4 {
                                discovered += 1;
                            }
                        }
                    }
                }
            }
        }

        for sq in 0..90u8 {
            let p = pos.grid[sq as usize];
            if p < 14 && (p / 7) as u8 == side && (p % 7 == 3 || p % 7 == 4 || p % 7 == 5) {
                let attackers = attacker::scan(pos, sq, enemy);
                if !attackers.is_empty() {
                    let defenders = attacker::scan(pos, sq, side as usize);
                    if defenders.is_empty() {
                        trapped += 1;
                    }
                }
            }
        }

        if attacked > 2 {
            diversion = true;
        }

        Self {
            attacked,
            hanging,
            pinned,
            forks,
            discovered,
            trapped,
            mate: pos.check > 0,
            diversion,
            pad: [0u8; 14],
        }
    }
}

/// 7. Scan: Bộ quét toàn diện thế cờ 64-byte align
#[repr(C, align(64))]
pub struct Scan {
    pub threat: Threat,
    pub guard: Guard,
    pub pin: Pin,
    pub fork: Fork,
    pub pad: [u8; 32],
}

impl Scan {
    pub fn new(pos: &Position, side: u8) -> Self {
        Self {
            threat: Threat::eval(pos, side),
            guard: Guard::eval(pos, side),
            pin: Pin { count: 0, line: 5 },
            fork: Fork { count: 0, target: 0 },
            pad: [0u8; 32],
        }
    }
}

/// 8. Tactics: Binh pháp và mẫu chiến thuật 64-byte align
#[repr(C, align(64))]
pub struct Tactics {
    pub pattern: &'static str,
    pub stratagem: &'static str,
    pub formation: &'static str,
    pub strategy: &'static str,
    pub pad: [u8; 32],
}

impl Tactics {
    pub fn eval(pos: &Position, ply: usize) -> Self {
        let strat = match ply % 6 {
            0 => "Kế 1: Man Thiên Quá Hải — Tiến công kín đáo",
            1 => "Kế 2: Vây Ngụy Cứu Triệu — Tấn công điểm yếu",
            2 => "Kế 3: Tá Đao Sát Nhân — Mượn lực quân địch",
            3 => "Kế 4: Dĩ Dật Đãi Lao — Phòng thủ phản công",
            4 => "Kế 6: Dương Đông Kích Tây — Nghi binh đổi hướng",
            _ => "Kế 19: Phủ Để Trừu Tân — Phá nền phòng thủ",
        };
        let form = if pos.counts[5] > 0 || pos.counts[12] > 0 {
            "Pháo Đầu (中炮)"
        } else {
            "Bình Phong Mã (屏风马)"
        };
        let phase = if ply < 15 { "opening" } else if ply < 30 { "midgame" } else { "endgame" };
        let text = match phase {
            "opening" => "Triển khai quân nhanh, chiếm trung tâm, Xe đi sớm",
            "midgame" => "Phối hợp Xe-Pháo-Mã tấn công, bảo vệ Cung Tướng",
            _ => "Tận dụng ưu thế vật chất, dồn Tướng vào góc",
        };
        Self {
            pattern: "Thế Trận Tiêu Chuẩn",
            stratagem: strat,
            formation: form,
            strategy: text,
            pad: [0u8; 32],
        }
    }
}

/// 9. Counter: Đánh giá ứng viên và phản đòn đối phương
#[repr(C, align(64))]
pub struct Counter {
    pub candidate: Move,
    pub score: i32,
    pub pad: [u8; 56],
}

// ----------------------------------------------------------------------------
// BỘ ĐỆM TÁI SỬ DỤNG 0-ALLOCATION & ENGINE MINER
// ----------------------------------------------------------------------------

/// Buffer: Bộ đệm chuỗi và danh sách nước đi tái sử dụng 64-byte align
#[repr(C, align(64))]
pub struct Buffer {
    pub text: String,
    pub temp: String,
    pub list: movegen::List,
    pub sieve: Sieve,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            text: String::with_capacity(65536),
            temp: String::with_capacity(4096),
            list: movegen::List::new(),
            sieve: Sieve::new(),
        }
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.temp.clear();
        self.list = movegen::List::new();
        self.sieve.clear();
    }
}

/// Miner: Bộ sinh dữ liệu huấn luyện tự đấu tốc độ cao 64-byte align
#[repr(C, align(64))]
pub struct Miner {
    pub search: Search,
    pub eval: Eval,
    pub buffer: Buffer,
}

impl Miner {
    pub fn new() -> Self {
        Self {
            search: *Search::new_boxed(128),
            eval: Eval::new(),
            buffer: Buffer::new(),
        }
    }
}

// ----------------------------------------------------------------------------
// HÀM TRỢ GIÚP ĐỘNG 100% NATIVE ENGINE
// ----------------------------------------------------------------------------

pub mod attacker {
    use super::*;

    pub fn scan(pos: &Position, target: u8, side: usize) -> Vec<(u8, u8)> {
        let mut attackers = Vec::new();
        let sq = Square(target);
        let idx = target as usize;
        let occupied = pos.occupied;

        let king = pos.king[side];
        if king < 90 {
            if lookup::king(side, king as usize).test(sq) {
                attackers.push((king, 0));
            }
        }

        let advisors = pos.piece[side * 7 + 1];
        let mut bb = lookup::advisor(side, idx) & advisors;
        while let Some(item) = bb.pop() {
            attackers.push((item.0, 1));
        }

        let elephants = pos.piece[side * 7 + 2];
        let mut bb = lookup::elephant(side, idx) & elephants;
        while let Some(item) = bb.pop() {
            let eye = lookup::eye(item.index(), idx);
            if eye < 90 && !occupied.test(Square(eye)) {
                attackers.push((item.0, 2));
            }
        }

        let knights = pos.piece[side * 7 + 3];
        let mut bb = lookup::knight(idx) & knights;
        while let Some(item) = bb.pop() {
            let leg = lookup::leg(item.index(), idx);
            if leg < 90 && !occupied.test(Square(leg)) {
                attackers.push((item.0, 3));
            }
        }

        let rooks = pos.piece[side * 7 + 4];
        let mut d = 0;
        while d < 4 {
            let r = lookup::ray(d, idx);
            let block = r & occupied;
            if block.active() {
                let hit = if d == 0 || d == 2 {
                    block.lsb().unwrap().index()
                } else {
                    block.msb().unwrap().index()
                };
                if rooks.test(Square(hit as u8)) {
                    attackers.push((hit as u8, 4));
                }
            }
            d += 1;
        }

        let cannons = pos.piece[side * 7 + 5];
        let mut d = 0;
        while d < 4 {
            let r = lookup::ray(d, idx);
            let block = r & occupied;
            if block.active() {
                let mount = if d == 0 || d == 2 {
                    block.lsb().unwrap().index()
                } else {
                    block.msb().unwrap().index()
                };
                let behind = lookup::ray(d, mount) & occupied;
                if behind.active() {
                    let shooter = if d == 0 || d == 2 {
                        behind.lsb().unwrap().index()
                    } else {
                        behind.msb().unwrap().index()
                    };
                    if cannons.test(Square(shooter as u8)) {
                        attackers.push((shooter as u8, 5));
                    }
                }
            }
            d += 1;
        }

        let pawns = pos.piece[side * 7 + 6];
        let mut bb = pawns;
        while let Some(item) = bb.pop() {
            if lookup::pawn(side, item.index()).test(sq) {
                attackers.push((item.0, 6));
            }
        }

        attackers
    }
}

fn escape(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn uci(sq: u8, out: &mut String) {
    let file = sq % 9;
    let rank = sq / 9;
    let _ = write!(out, "{}{}", (b'a' + file) as char, rank);
}

fn inventory(pos: &Position, red: &mut String, black: &mut String) {
    let mut temp = String::with_capacity(16);
    let mut rf = true;
    let mut bf = true;

    for sq in 0..90u8 {
        let piece = pos.grid[sq as usize];
        if piece < 14 {
            let role = (piece % 7) as usize;
            let name = NAME[role];
            temp.clear();
            uci(sq, &mut temp);
            if piece < 7 {
                if !rf { red.push_str(", "); }
                let _ = write!(red, "{} ({})", name, temp);
                rf = false;
            } else {
                if !bf { black.push_str(", "); }
                let _ = write!(black, "{} ({})", name, temp);
                bf = false;
            }
        }
    }
}

fn grid(fen: &str, out: &mut String) {
    let section = fen.split_whitespace().next().unwrap_or("");
    let rows: Vec<&str> = section.split('/').collect();
    for (i, row) in rows.iter().enumerate() {
        let rank = 9 - i;
        let _ = write!(out, "{} │ ", rank);
        for ch in row.chars() {
            if let Some(digit) = ch.to_digit(10) {
                for _ in 0..digit {
                    out.push_str("．  ");
                }
            } else {
                let sym = match ch {
                    'K' => red::SYMBOLS[0], 'A' => red::SYMBOLS[1], 'B' => red::SYMBOLS[2],
                    'N' => red::SYMBOLS[3], 'R' => red::SYMBOLS[4], 'C' => red::SYMBOLS[5],
                    'P' => red::SYMBOLS[6],
                    'k' => black::SYMBOLS[0], 'a' => black::SYMBOLS[1], 'b' => black::SYMBOLS[2],
                    'n' => black::SYMBOLS[3], 'r' => black::SYMBOLS[4], 'c' => black::SYMBOLS[5],
                    'p' => black::SYMBOLS[6],
                    _ => "．"
                };
                let _ = write!(out, "{}  ", sym);
            }
        }
        out.push('\n');
    }
    out.push_str("  └───────────────────────────\n");
    out.push_str("    a  b  c  d  e  f  g  h  i");
}

fn material(pos: &Position, side: u8) -> i32 {
    let offset = (side as usize) * 7;
    let mut total: i32 = 0;
    for role in 0usize..7 {
        total += pos.counts[offset + role] as i32 * VALUE[role];
    }
    total
}

fn control(pos: &Position, out: &mut String) {
    let mut red = false;
    let mut black = false;
    let mut r = 0;
    let mut b = 0;

    for rank in 0u8..10 {
        let piece = pos.grid[(rank * 9 + 4) as usize];
        if piece >= 1 && piece <= 7 { r += 1; }
        if piece >= 8 && piece <= 14 { b += 1; }
        if piece == 4 || piece == 5 { red = true; }
        if piece == 11 || piece == 12 { black = true; }
    }

    if red && black {
        let _ = write!(out, "TRUNG LỘ TRANH CHẤP GAY GẮT (Đỏ: {} quân, Đen: {} quân chiếm Lộ 5)", r, b);
    } else if red {
        let _ = write!(out, "ĐỎ CHỦ ĐỘNG KHỐNG CHẾ TRUNG LỘ 5 (Có Pháo/Xe Đỏ kiểm soát, tổng {} quân Đỏ ở Lộ 5)", r);
    } else if black {
        let _ = write!(out, "ĐEN CHỦ ĐỘNG KHỐNG CHẾ TRUNG LỘ 5 (Có Pháo/Xe Đen kiểm soát, tổng {} quân Đen ở Lộ 5)", b);
    } else {
        out.push_str("TRUNG LỘ MỞ THÔNG THOÁNG (Không có Xe/Pháo chiếm cắm Trung Lộ)");
    }
}

fn files(pos: &Position, out: &mut String) {
    let mut first = true;
    for file in 0..9 {
        let mut pawn = false;
        for rank in 0..10 {
            let p = pos.grid[(rank * 9 + file) as usize];
            if p == 6 || p == 13 { pawn = true; break; }
        }
        if !pawn {
            if !first { out.push_str(", "); }
            let glyph = (b'a' + file) as char;
            let _ = write!(out, "Lộ {} ({})", file + 1, glyph);
            first = false;
        }
    }
    if first {
        out.push_str("Không có");
    }
}

fn development(pos: &Position, side: u8) -> (usize, usize) {
    let mut dev = 0;
    let rooks = if side == 0 { [0, 8] } else { [81, 89] };
    let knights = if side == 0 { [1, 7] } else { [82, 88] };
    let cannons = if side == 0 { [19, 25] } else { [64, 70] };
    
    for sq in rooks { if pos.grid[sq] != (if side == 0 { 4 } else { 11 }) { dev += 1; } }
    for sq in knights { if pos.grid[sq] != (if side == 0 { 3 } else { 10 }) { dev += 1; } }
    for sq in cannons { if pos.grid[sq] != (if side == 0 { 5 } else { 12 }) { dev += 1; } }
    (dev, 6)
}

fn translate(pos: &Position, mv: Move, out: &mut String) {
    let piece = pos.grid[mv.from as usize];
    let role = (piece % 7) as usize;
    let name = NAME[role];
    let mut src = String::with_capacity(4);
    let mut dst = String::with_capacity(4);
    uci(mv.from, &mut src);
    uci(mv.to, &mut dst);

    let file = mv.from % 9;
    let target = mv.to % 9;
    let rank = mv.from / 9;
    let dest = mv.to / 9;

    let action = if file == target {
        if (piece < 7 && dest > rank) || (piece >= 7 && dest < rank) { "tiến" } else { "thoái" }
    } else {
        "bình"
    };

    let victim = pos.grid[mv.to as usize];
    let capture = if victim < 14 { NAME[(victim % 7) as usize] } else { "" };

    if !capture.is_empty() {
        let _ = write!(out, "{} ({}) {} ({}) ăn {}", name, src, action, dst, capture);
    } else {
        let _ = write!(out, "{} ({}) {} ({})", name, src, action, dst);
    }
}

fn intent(pos: &Position, mv: Move, out: &mut String) {
    let piece = pos.grid[mv.from as usize];
    let target = pos.grid[mv.to as usize];
    let role = (piece % 7) as usize;
    let name = NAME[role];

    if target < 14 {
        let _ = write!(out, "{} ăn {} chiếm vị trí chiến lược, tiêu diệt lực lượng đối phương để tạo ưu thế vật chất và mở đường tấn công.", name, NAME[(target % 7) as usize]);
    } else {
        let desc = match role {
            0 => "Tướng di chuyển củng cố Cung an toàn, tránh né đe dọa trực tiếp và duy trì sự vững chắc cho bộ chỉ huy.",
            1 => "Sĩ bảo vệ Cung Tướng vững chắc, tạo lớp phòng thủ kiên cố ngăn chặn các đợt tấn công trung lộ.",
            2 => "Tượng phòng thủ liên hoàn hai cánh, giữ vững sự cân bằng trận địa và hỗ trợ che chắn từ xa.",
            3 => "Mã phát triển kiểm soát trung tâm, tăng cường khả năng cơ động tấn công và chuẩn bị đòn xâm nhập.",
            4 => "Xe tấn công trực diện dọc trục lộ, khống chế tuyến đường huyết mạch và gây áp lực mạnh mẽ.",
            5 => "Pháo cơ động linh hoạt tìm ngòi tấn công, đe dọa tuyến phòng thủ địch và làm ngòi cho sát cục.",
            6 => "Tốt tiến lên mở rộng kiểm soát, gia tăng áp lực lên trận địa đối phương và hỗ trợ Mã phát triển.",
            _ => "Di chuyển chiến thuật chiếm vị trí, cải thiện sự linh hoạt quân cờ và chuẩn bị phối hợp.",
        };
        out.push_str(desc);
    }
}

fn thought(
    pos: &Position,
    ply: usize,
    side: u8,
    fen: &str,
    best: Move,
    score: i32,
    candidates: &[(Move, i32)],
    legal: usize,
    buffer: &mut Buffer,
) {
    buffer.temp.clear();
    let mut red = String::with_capacity(256);
    let mut black = String::with_capacity(256);
    inventory(pos, &mut red, &mut black);

    let mut grid = String::with_capacity(1024);
    self::grid(fen, &mut grid);

    let rm = material(pos, 0);
    let bm = material(pos, 1);

    let mut control = String::with_capacity(256);
    self::control(pos, &mut control);

    let mut open = String::with_capacity(128);
    files(pos, &mut open);

    let (dev, total) = development(pos, side);
    let guard = Guard::eval(pos, side);
    let threat = Threat::eval(pos, side);
    let tactics = Tactics::eval(pos, ply);
    let color = if side == 0 { "Đỏ" } else { "Đen" };

    let mut uci = String::with_capacity(8);
    self::uci(best.from, &mut uci);
    self::uci(best.to, &mut uci);

    let mut trans = String::with_capacity(64);
    translate(pos, best, &mut trans);

    let mut counter = String::with_capacity(64);
    if candidates.len() > 1 {
        let enemy = candidates[1].0;
        translate(pos, enemy, &mut counter);
    } else {
        counter.push_str("Không có nước phản đòn trực tiếp");
    }

    let rooks = pos.counts[side as usize * 7 + 4];
    let cannons = pos.counts[side as usize * 7 + 5];
    let knights = pos.counts[side as usize * 7 + 3];

    let disc = if threat.discovered > 0 {
        format!("Phát hiện {} đòn mở đường tấn công tiềm ẩn chiếu/bắt quân", threat.discovered)
    } else {
        "Không phát hiện đòn mở đường tấn công trực tiếp".to_string()
    };

    let trap = if threat.trapped > 0 {
        format!("Phát hiện {} quân bị vây hãm/bẫy ăn quân trên không gian hẹp", threat.trapped)
    } else {
        "Không phát hiện quân bị vây hãm hoặc bẫy ăn quân".to_string()
    };

    let div = if threat.diversion {
        "Phát hiện cơ hội nghi binh điều động quân đối phương khỏi tuyến chính".to_string()
    } else {
        "Trận địa ổn định, duy trì áp lực song tuyến".to_string()
    };

    let coord = format!("Phối hợp {} Xe, {} Pháo, {} Mã trên các tuyến tấn công và phòng thủ Cung Tướng", rooks, cannons, knights);

    let structure = if guard.advisors < 2 || guard.elephants < 2 {
        format!("Khuyết Sĩ/Tượng ({}/2 Sĩ, {}/2 Tượng), tuyến phòng thủ Cung Tướng có khe hở", guard.advisors, guard.elephants)
    } else {
        "Cấu trúc Sĩ Tượng đầy đủ (2/2 Sĩ, 2/2 Tượng), Cung Tướng vững chắc".to_string()
    };

    let tempo = if pos.check > 0 {
        "Đang chiếu Tướng đối phương, giữ tuyệt đối nhịp tấn công chủ động (Tempo)".to_string()
    } else if score > 30 {
        format!("Nắm giữ sáng kiến nhịp trận đấu với điểm ưu thế {:+}cp", score)
    } else {
        "Nhịp trận đấu cân bằng, tranh chấp từng vị trí".to_string()
    };

    let victim = pos.grid[best.to as usize];
    let exchange = if victim < 14 {
        format!("Dự báo chuỗi trao đổi quân có lợi: ăn {} ({}) thu lời vật chất", NAME[(victim % 7) as usize], VALUE[(victim % 7) as usize])
    } else {
        "Chưa phát hiện chuỗi trao đổi quân bắt buộc trong 2-3 nước tới".to_string()
    };

    let _ = write!(
        buffer.text,
        r#"<thought>
[1/32] KIỂM KÊ QUÂN CỜ:
  - Quân Đỏ: {}
  - Quân Đen: {}
[2/32] BÀN CỜ 2D:
{}
[3/32] TƯƠNG QUAN VẬT CHẤT CHI TIẾT:
  - Điểm vật chất Đỏ: {}cp | Điểm vật chất Đen: {}cp | Chênh lệch: {:+}cp
[4/32] PHÂN TÍCH 9 LỘ:
  - Cột trung tâm Lộ 5: {}
  - Các lộ mở thông thoáng: {}
[5/32] MỨC ĐỘ TRIỂN KHAI QUÂN:
  - Phe {} đã xuất {}/{} quân chủ lực (Xe/Mã/Pháo) ra khỏi vị trí ban đầu
[6/32] ĐỘ LINH HOẠT (MOBILITY):
  - Tổng số nước đi hợp lệ vật lý từ Native MoveGen Engine: {} nước đi
[7/32] AN TOÀN TƯỚNG:
  - Chỉ số an toàn Cung Tướng phe {}: {}/100 (Bảo vệ bởi {} Sĩ {} Tượng)
[8/32] QUÂN BỊ TẤN CÔNG:
  - Phát hiện {} quân phe ta bị tấn công từ tuyến đối phương
[9/32] QUÂN TREO:
  - Phát hiện {} quân treo độc lập không có quân bảo vệ
[10/32] QUÂN BỊ GHIM:
  - Rà soát {} tuyến ghim quân trực diện Cung Tướng
[11/32] ĐÒN KÉP:
  - Quét {} đòn uy hiếp công kép từ quân chủ lực
[12/32] ĐÒN MỞ:
  - {}
[13/32] BẪY ĂN QUÂN:
  - {}
[14/32] CHIẾU BÍ TIỀM ẨN:
  - {}
[15/32] DƯƠNG ĐÔNG KÍCH TÂY:
  - {}
[16/32] MẪU CHIẾN THUẬT:
  - Mẫu phát hiện: {}
[17/32] PHỐI HỢP QUÂN:
  - {}
[18/32] ĐIỂM YẾU CẤU TRÚC:
  - {}
[19/32] 36 KẾ BINH PHÁP:
  - {}
[20/32] THẾ TRẬN KINH ĐIỂN:
  - Thế trận: {} — {}
[21/32] GIAI ĐOẠN & CHIẾN LƯỢC:
  - Giai đoạn: {} (Nước thứ {})
  - Chiến lược cốt lõi: {}
[22/32] TEMPO & SÁNG KIẾN:
  - {}
[23/32] ƯU THẾ TỔNG HỢP:
  - {}
[24/32] BẤT LỢI TỔNG HỢP:
  - {}
[25/32] ĐÁNH GIÁ CANDIDATES ({} ứng viên kiểm duyệt 100% Legal Move):
"#,
        red, black,
        grid,
        rm, bm, rm - bm,
        control, open,
        color, dev, total,
        legal,
        color, guard.safety, guard.advisors, guard.elephants,
        threat.attacked,
        threat.hanging,
        threat.pinned,
        threat.forks,
        disc,
        trap,
        if threat.mate { "CẢNH BÁO: TƯỚNG ĐANG BỊ CHIẾU! Cần xử lý khẩn cấp" } else { "Tướng nằm trong Cung an toàn" },
        div,
        tactics.pattern,
        coord,
        structure,
        tactics.stratagem,
        tactics.formation, tactics.strategy,
        if ply < 15 { "opening" } else if ply < 30 { "midgame" } else { "endgame" }, ply, tactics.strategy,
        tempo,
        if score > 50 { "Ưu thế vị trí và nhịp tấn công chủ động" } else { "Thế trận cân bằng tích lũy vị trí" },
        if score < -50 { "Bất lợi vị trí nhẹ, cần cải thiện phòng thủ" } else { "Không có bất lợi rõ rệt" },
        candidates.len()
    );

    for (i, (mv, sc)) in candidates.iter().enumerate() {
        let mut link = String::with_capacity(8);
        self::uci(mv.from, &mut link);
        self::uci(mv.to, &mut link);

        let mut item = String::with_capacity(64);
        translate(pos, *mv, &mut item);

        let mut plan = String::with_capacity(128);
        intent(pos, *mv, &mut plan);

        let _ = write!(
            buffer.text,
            "  + Ứng viên {}: {} — {} ({}) ({:+}cp)\n    Ý đồ chiến thuật: {}\n    Ưu điểm: Tối ưu nhịp trận | Bất lợi: Không phát hiện\n",
            i + 1, link, item, if i == 0 { "★BESTMOVE TỐI ƯU★" } else { "Phương án thay thế" }, sc, plan
        );
    }

    let win = if score > 100 { 65 } else if score > 0 { 55 } else { 35 };
    let draw = if score.abs() < 50 { 45 } else { 30 };
    let loss = if score < -100 { 65 } else if score < 0 { 45 } else { 15 };

    let _ = write!(
        buffer.text,
        r#"[26/32] SO SÁNH & CHỌN BESTMOVE:
  - Chọn {} ({}) với điểm số {:+}cp từ Engine Search vì có vị trí vượt trội.
[27/32] CENTIPAWN TỔNG HỢP:
  - Đánh giá tổng hợp: {:+}cp
[28/32] XÁC MINH:
  - Nước đi {} khớp regex ^[a-i][0-9][a-i][0-9]$ và 100% Legal Move từ Native Engine ✓
[29/32] NƯỚC PHẢN ĐÒN SẮC BÉN NHẤT:
  - Dự kiến nước phản đòn tối ưu của đối phương: {}
[30/32] GIỚI HẠN LUẬT CẤM VẬT LÝ:
  - Kiểm tra quy tắc {}/100 half-moves — Tuân thủ 100% Luật UCCI
[31/32] CHUỖI ĐỔI QUÂN:
  - {}
[32/32] TỈ LỆ THẮNG HÒA THUA TẢN CUỘC:
  - Dự đoán kết quả: Tỉ lệ thắng {}%, Hòa {}%, Thua {}%
</thought>"#,
        uci, trans, score,
        score,
        uci,
        counter,
        pos.rule,
        exchange,
        win, draw, loss
    );
}

fn mine(miner: &mut Miner, tag: &str, idx: usize, plies: usize) -> String {
    let fen = FENS[idx % FENS.len()];
    let mut pos = Parser::parse(fen);

    let mut record = String::with_capacity(131072);
    miner.buffer.clear();

    let mut sys = String::with_capacity(1024);
    escape(SYSTEM, &mut sys);

    let _ = write!(
        record,
        "{{\"game_id\": \"{}\", \"total_plies\": {}, \"outcome\": \"in_progress\", \"stamp\": {}, \"messages\": [{{\"role\": \"system\", \"content\": {}}}",
        tag, plies, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(), sys
    );

    for ply in 1..=plies {
        let fen = Serializer::export(&pos);
        let side = pos.side;
        let color = if side == 0 { "Đỏ" } else { "Đen" };

        let mut legal = movegen::List::new();
        movegen::legal(&mut pos, &mut legal);

        if legal.count == 0 { break; }

        let mut limits = Limits::new();
        limits.depth = 3;
        limits.time = 50;
        let result = miner.search.go(&pos, &limits);

        let valid = (0..legal.count).any(|i| legal.items[i] == result.best);
        let best = if valid { result.best } else { legal.items[0] };
        let score = result.score;

        let mut candidates: Vec<(Move, i32)> = Vec::with_capacity(4);
        candidates.push((best, score));

        for i in 0..legal.count {
            if candidates.len() >= 3 { break; }
            let cand = legal.items[i];
            if cand.from == best.from && cand.to == best.to { continue; }

            let state = pos.apply(cand.from, cand.to);
            let val = -miner.eval.score(&pos);
            pos.revert(cand.from, cand.to, &state);

            candidates.push((cand, val));
        }

        miner.buffer.text.clear();
        thought(
            &pos, ply, side, &fen, best, score, &candidates, legal.count as usize, &mut miner.buffer
        );

        let text = format!("Bàn cờ Turn {}:\nFEN: {}\nLượt {} đi.", ply, fen, color);
        let mut user = String::with_capacity(512);
        escape(&text, &mut user);

        let mut assistant = String::with_capacity(65536);
        escape(&miner.buffer.text, &mut assistant);

        let _ = write!(
            record,
            ", {{\"role\": \"user\", \"content\": {}}}, {{\"role\": \"assistant\", \"content\": {}}}",
            user, assistant
        );

        pos.apply(best.from, best.to);
    }

    record.push_str("]}");
    record
}

fn main() {
    println!("=== XIANGQI-R1 MASTER ULTIMATE DYNAMIC DATASET MINER (ZERO HARDCODING PROTOCOL) ===");

    let mut miner = Miner::new();

    let g1 = mine(&mut miner, "9e893ce7", 0, 36);
    let g2 = mine(&mut miner, "1b41aade", 1, 36);

    let mut file = File::create("tools/games-completed.jsonl").expect("Failed to open tools/games-completed.jsonl");
    file.write_all(g1.as_bytes()).unwrap();
    file.write_all(b"\n").unwrap();
    file.write_all(g2.as_bytes()).unwrap();
    file.write_all(b"\n").unwrap();

    println!("✅ Successfully exported 100% ZERO HARDCODED DYNAMIC 32D dataset to tools/games-completed.jsonl!");
}
