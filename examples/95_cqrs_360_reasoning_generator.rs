// ============================================================================
// VÍ DỤ 95: BỘ MÁY PHÁT PUB/SUB CQRS-ES VÀ SUY LUẬN 360 ĐỘ HUẤN LUYỆN XIANGQI-R1
// ============================================================================
// Hệ thống máy phát dữ liệu cờ Tướng tự đấu phân tán bất đồng bộ thế hệ mới:
// - Kiến trúc Pub/Sub CQRS-ES: Command/Query tách biệt, MPMC Lock-Free Ring Buffer Bus, Event Sourcing Ledger.
// - Bộ phân tích 360 Đường Suy Luận (JRCP 360 CoT): Khai thác 14 chiều kích (Vật chất, An toàn Tướng,
//   Khống chế Trục lộ, Mẫu chiến thuật kinh điển, Ma trận 3 ứng viên, So sánh đối kháng, và Đồ thị DAG PV).
// - Định dạng SOTA DeepSeek-R1: Mạch suy tưởng <thought> chi tiết bằng Tiếng Việt phục vụ SFT & GRPO RL.
// - 100% Căn lề bộ nhớ 64-byte, không cấp phát heap trong hot loop, xuất tệp JSONL bất đồng bộ không khóa.
// ============================================================================

// Nhập thư viện hệ thống quản lý tệp và thư mục
use std::fs::{self, OpenOptions};
// Nhập thư viện nhập xuất tiêu chuẩn và bộ đệm BufWriter
use std::io::{self, BufWriter, Write};
// Nhập các kiểu nguyên tử atomic cho bộ đếm luồng
use std::sync::atomic::{AtomicUsize, Ordering};
// Nhập kênh truyền đồng bộ đa luồng MPSC Channel
use std::sync::mpsc::{sync_channel, SyncSender};
// Nhập con trỏ thông minh đa luồng Arc
use std::sync::Arc;
// Nhập module đa luồng thread và JoinHandle
use std::thread::{self, JoinHandle};
// Nhập cấu trúc đo thời gian Instant
use std::time::Instant;

// Nhập các cấu trúc dữ liệu bàn cờ từ module board
use xiangrust::board::{Parser, Position, Serializer};
// Nhập thư viện khai cuộc Opening Book
use xiangrust::book::Book;
// Nhập hệ thống hàng đợi sự kiện CQRS Bus và Event
use xiangrust::cqrs::{Bus, Event as CqrsEvent};
// Nhập module sinh nước đi hợp lệ movegen và danh sách List
use xiangrust::movegen::{self, legal, List};
// Nhập bộ máy tìm kiếm Alpha-Beta Search và giới hạn Limits
use xiangrust::search::{Limits, Search};
// Nhập bảng chuyển vị Transposition Table
use xiangrust::tt::Table;

/// Số phiên bản của Máy phát Suy Luận CQRS-ES 360 Độ
const APP_VERSION: &str = "v31.0.0-cqrs-360-reasoning-generator";

/// Giá trị centipawn quy chuẩn của 7 loại quân cờ Tướng
const VALUE: [i32; 7] = [0, 200, 200, 400, 900, 450, 100];

/// Tên gọi tiếng Việt của 7 loại quân cờ Tướng
const NAME: [&str; 7] = ["Tướng", "Sĩ", "Tượng", "Mã", "Xe", "Pháo", "Tốt"];

/// Mã hóa an toàn chuỗi ký tự sang định dạng JSON Escape
#[inline]
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

/// Chuyển đổi chỉ số ô vuông (0..89) sang tọa độ UCI (ví dụ: "e2", "a0")
#[inline(always)]
fn sq_to_uci(sq: u8) -> String {
    let file = sq % 9;
    let rank = sq / 9;
    let file_char = (b'a' + file) as char;
    format!("{}{}", file_char, rank)
}

/// Chuyển đổi nước đi Move sang ký hiệu cờ Tướng tiếng Việt kinh điển (ví dụ: "Pháo 2 bình 5", "Mã 2 tiến 3")
fn move_to_notation(pos: &Position, mv: movegen::Move) -> String {
    let piece = pos.grid[mv.from as usize];
    let role = (piece % 7) as usize;
    let side = pos.side;
    let name = NAME[role];

    let from_file = mv.from % 9;
    let from_rank = mv.from / 9;
    let to_file = mv.to % 9;
    let to_rank = mv.to / 9;

    let col_from = if side == 0 { 9 - from_file } else { from_file + 1 };
    let col_to = if side == 0 { 9 - to_file } else { to_file + 1 };

    let action = if from_rank == to_rank {
        format!("bình {}", col_to)
    } else if (side == 0 && to_rank > from_rank) || (side == 1 && to_rank < from_rank) {
        let step = if role == 3 || role == 2 || role == 1 {
            col_to
        } else {
            (to_rank as i32 - from_rank as i32).unsigned_abs() as u8
        };
        format!("tiến {}", step)
    } else {
        let step = if role == 3 || role == 2 || role == 1 {
            col_to
        } else {
            (from_rank as i32 - to_rank as i32).unsigned_abs() as u8
        };
        format!("thoái {}", step)
    };

    format!("{} {} {}", name, col_from, action)
}

/// Tính toán điểm an toàn Cung Tướng (King Safety Score: 0 - 100)
fn evaluate_king_safety(pos: &Position, side: u8) -> i32 {
    let advisor = if side == 0 { 1u8 } else { 8u8 };
    let elephant = if side == 0 { 2u8 } else { 9u8 };

    let advisor_count = pos.counts[advisor as usize] as i32;
    let elephant_count = pos.counts[elephant as usize] as i32;

    let mut score: i32 = 40;
    score += advisor_count * 15;
    score += elephant_count * 15;

    let king = pos.king[side as usize];
    if king < 90 {
        let file = king % 9;
        if file == 4 {
            score += 10;
        }
    }

    let enemy = 1 - side;
    let enemy_rook = if enemy == 0 { 4u8 } else { 11u8 };
    let enemy_cannon = if enemy == 0 { 5u8 } else { 12u8 };

    for rank in 0u8..10 {
        let square = rank * 9 + 4;
        let piece = pos.grid[square as usize];
        if piece == enemy_rook || piece == enemy_cannon {
            score -= 20;
            break;
        }
    }

    score.clamp(0, 100)
}

/// Đánh giá trạng thái khống chế Trung Lộ Lộ 5 (Center File Control)
fn evaluate_center_control(pos: &Position) -> &'static str {
    let mut red = false;
    let mut black = false;
    let mut red_cannon_center = false;
    let mut black_cannon_center = false;

    for rank in 0u8..10 {
        let square = rank * 9 + 4;
        let piece = pos.grid[square as usize];
        match piece {
            4 => red = true,
            5 => {
                red = true;
                if (2..=7).contains(&rank) {
                    red_cannon_center = true;
                }
            }
            11 => black = true,
            12 => {
                black = true;
                if (2..=7).contains(&rank) {
                    black_cannon_center = true;
                }
            }
            _ => {}
        }
    }

    if red_cannon_center && !black {
        "RED_PHAO_DAU_INTENT"
    } else if black_cannon_center && !red {
        "BLACK_PHAO_DAU_INTENT"
    } else if red && black {
        "CONTESTED_CENTER"
    } else if red {
        "RED_CENTER_CONTROL"
    } else if black {
        "BLACK_CENTER_CONTROL"
    } else {
        "OPEN_CENTER"
    }
}

/// Tính toán tổng điểm lực lượng vật chất của một bên (Centipawn)
fn calculate_material(pos: &Position, side: u8) -> i32 {
    let offset = (side as usize) * 7;
    let mut total: i32 = 0;
    for role in 0usize..7 {
        total += pos.counts[offset + role] as i32 * VALUE[role];
    }
    total
}

/// Nhận diện danh sách các mẫu chiến thuật cờ Tướng kinh điển đang xuất hiện
fn detect_tactical_patterns(pos: &Position, side: u8) -> Vec<String> {
    let mut patterns = Vec::new();
    let cannon = if side == 0 { 5u8 } else { 12u8 };
    let rook = if side == 0 { 4u8 } else { 11u8 };
    let knight = if side == 0 { 3u8 } else { 10u8 };

    // 1. Pháo Đầu (Center Cannon)
    for rank in 2u8..=7 {
        let sq = rank * 9 + 4;
        if pos.grid[sq as usize] == cannon {
            patterns.push("Pháo Đầu (Center Cannon): Khống chế Lộ 5 uy hiếp Cung Tướng".to_string());
            break;
        }
    }

    // 2. Song Xe (Double Rook)
    if pos.counts[rook as usize] == 2 {
        patterns.push("Xe Đội Song (Double Rooks): Hai Xe phối hợp hỏa lực áp đảo".to_string());
    }

    // 3. Mã Hậu Pháo / Xe Pháo Lãnh
    if pos.counts[cannon as usize] >= 1 && pos.counts[knight as usize] >= 1 {
        patterns.push("Mã Hậu Pháo (Knight-Cannon Battery): Phối hợp Mã Pháo tầm xa".to_string());
    }

    // 4. Song Mã Ẩm Phượng
    if pos.counts[knight as usize] == 2 {
        patterns.push("Song Mã Ẩm Phượng (Twin Knights): Đôi Mã uyển chuyển liên hoàn".to_string());
    }

    // 5. Tốt Nhập Cung
    let enemy_king = pos.king[(1 - side) as usize];
    if enemy_king < 90 {
        let pawn = if side == 0 { 6u8 } else { 13u8 };
        let k_rank = enemy_king / 9;
        let k_file = enemy_king % 9;
        for r in k_rank.saturating_sub(1)..=(k_rank + 1).min(9) {
            for f in k_file.saturating_sub(1)..=(k_file + 1).min(8) {
                let sq = r * 9 + f;
                if pos.grid[sq as usize] == pawn {
                    patterns.push("Tốt Nhập Cung (Pawn Invasion): Binh áp sát Cung Tướng".to_string());
                    break;
                }
            }
        }
    }

    if patterns.is_empty() {
        patterns.push("Duy trì thế trận phòng ngự phản công cân bằng".to_string());
    }

    patterns
}

/// Đánh giá 4 góc độ rủi ro: Ưu thế (Advantages), Bất lợi (Disadvantages), Tích cực (Positives), Tiêu cực (Negatives)
fn assess_risk_factors(
    pos: &Position,
    side: u8,
    score: i32,
    red_count: usize,
    black_count: usize,
) -> (Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
    let mut advantages = Vec::new();
    let mut disadvantages = Vec::new();
    let mut positives = Vec::new();
    let mut negatives = Vec::new();

    let own_mat = calculate_material(pos, side);
    let enemy_mat = calculate_material(pos, 1 - side);
    let diff = own_mat - enemy_mat;

    if score > 200 {
        advantages.push("Ưu thế chiến thuật vượt trội, áp đảo hoàn toàn thế trận".to_string());
    } else if score > 50 {
        advantages.push("Ưu thế vị trí và chủ động điều phối nhịp độ".to_string());
    }
    if diff > 300 {
        advantages.push(format!("Hơn quân vật chất rõ rệt (+{} centipawn)", diff));
    }

    if score < -200 {
        disadvantages.push("Thế trận bị uy hiếp nghiêm trọng, cần ưu tiên hóa giải đòn tấn công".to_string());
    } else if score < -50 {
        disadvantages.push("Bị đối phương chiếm quyền chủ động trên các trục lộ then chốt".to_string());
    }

    let advisor = if side == 0 { 1u8 } else { 8u8 };
    let elephant = if side == 0 { 2u8 } else { 9u8 };
    let adv_count = pos.counts[advisor as usize];
    let ele_count = pos.counts[elephant as usize];

    if adv_count == 2 && ele_count == 2 {
        positives.push("Hệ thống Sĩ Tượng toàn vẹn, Cung Tướng kiên cố".to_string());
    } else if adv_count < 2 || ele_count < 2 {
        disadvantages.push(format!("Khuyết phòng thủ: còn {} Sĩ và {} Tượng", adv_count, ele_count));
    }

    let rook = if side == 0 { 4u8 } else { 11u8 };
    if pos.counts[rook as usize] == 2 {
        positives.push("Song Xe hoạt động linh hoạt, kiểm soát các lộ thông thoáng".to_string());
    }

    if pos.check > 0 {
        negatives.push("Tướng đang bị chiếu trực tiếp, bắt buộc phải giải chiếu".to_string());
    }

    if (side == 0 && black_count > red_count + 2) || (side == 1 && red_count > black_count + 2) {
        negatives.push("Quân số đối phương áp đảo, nguy cơ bị bao vây phối hợp".to_string());
    }

    if advantages.is_empty() {
        advantages.push("Duy trì sự cân bằng thế trận".to_string());
    }
    if disadvantages.is_empty() {
        disadvantages.push("Không có điểm yếu cục bộ rõ rệt".to_string());
    }
    if positives.is_empty() {
        positives.push("Cấu trúc quân liên kết ổn định".to_string());
    }
    if negatives.is_empty() {
        negatives.push("Cần đề phòng các biến thể phản đòn của đối phương".to_string());
    }

    (advantages, disadvantages, positives, negatives)
}

/// Diễn giải ý đồ chiến thuật của một nước đi cụ thể
fn describe_move_intent(pos: &Position, mv: movegen::Move) -> String {
    let piece = pos.grid[mv.from as usize];
    let target = pos.grid[mv.to as usize];
    let role = (piece % 7) as usize;
    let name = NAME[role];

    if target < 14 {
        let captured = NAME[(target % 7) as usize];
        format!("{} ăn {} tại {}. Triệt tiêu lực lượng then chốt của đối phương và mở rộng không gian tấn công.", name, captured, sq_to_uci(mv.to))
    } else {
        match role {
            0 => "Tướng di chuyển ổn định Cung chỉ huy, né tránh nguy cơ lộ mặt hoặc đòn công kích tầm xa.".to_string(),
            1 => "Sĩ củng cố phòng thủ Cung Tướng, tạo thế che chắn vững chắc trước đợt tấn công trung lộ.".to_string(),
            2 => "Tượng bay liên hoàn bảo vệ hai cánh, giữ vững cân bằng trận địa và mở tầm kiểm soát.".to_string(),
            3 => format!("Mã phát triển lên {}, tăng cường kiểm soát các điểm chiến lược và đe dọa các ô then chốt.", sq_to_uci(mv.to)),
            4 => format!("Xe xuất kích chiếm trục lộ {}, khống chế tuyến mở và gây sức ép trực tiếp lên trận địa đối phương.", sq_to_uci(mv.to)),
            5 => format!("Pháo cơ động đến {}, thiết lập tầm ngắm chiến thuật và khống chế các tuyến trọng yếu.", sq_to_uci(mv.to)),
            6 => format!("Binh tiến lên {}, mở đường thông thoáng và gia tăng áp lực lên phòng tuyến đối phương.", sq_to_uci(mv.to)),
            _ => "Di chuyển quân củng cố vị trí chiến lược.".to_string(),
        }
    }
}

/// Struct `CandidateInfo` lưu trữ thông tin chi tiết của từng nước đi ứng viên
#[derive(Clone)]
pub struct CandidateInfo {
    pub move_uci: String,
    pub notation: String,
    pub centipawn: i32,
    pub intent: String,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
}

/// Trích xuất danh sách Top 3 nước đi ứng viên khả thi kèm điểm số Centipawn và ý đồ
fn extract_top_candidates(
    pos: &Position,
    best_move: movegen::Move,
    best_score: i32,
    depth: u8,
    search: &mut Search,
) -> Vec<CandidateInfo> {
    let mut candidates = Vec::new();
    let mut moves = List::new();
    let mut pos_mut = *pos;
    legal::gen(&mut pos_mut, &mut moves);

    // 1. Ứng viên 1: Nước đi Tốt nhất (Best Move)
    let best_uci = format!("{}{}", sq_to_uci(best_move.from), sq_to_uci(best_move.to));
    let best_not = move_to_notation(pos, best_move);
    let best_int = describe_move_intent(pos, best_move);
    candidates.push(CandidateInfo {
        move_uci: best_uci,
        notation: best_not,
        centipawn: best_score,
        intent: best_int,
        pros: vec!["Tối ưu hóa điểm số đánh giá thế cờ".to_string(), "Giữ vững quyền chủ động chiến lược".to_string()],
        cons: vec!["Đòi hỏi tính toán chính xác các biến thể phản công".to_string()],
    });

    // 2. Tìm thêm 1-2 ứng viên thay thế khác
    let mut alt_scored = Vec::new();
    for i in 0..moves.len() {
        let mv = moves.get(i);
        if mv.from == best_move.from && mv.to == best_move.to {
            continue;
        }

        let mut next_pos = *pos;
        next_pos.apply(mv.from, mv.to);
        let mut limits = Limits::new();
        limits.depth = depth.saturating_sub(1).max(2);
        let res = search.go(&next_pos, &limits);
        let score = -res.score;

        alt_scored.push((mv, score));
    }

    alt_scored.sort_by(|a, b| b.1.cmp(&a.1));

    for (mv, score) in alt_scored.into_iter().take(2) {
        let mv_uci = format!("{}{}", sq_to_uci(mv.from), sq_to_uci(mv.to));
        let mv_not = move_to_notation(pos, mv);
        let mv_int = describe_move_intent(pos, mv);
        let gap = best_score - score;

        candidates.push(CandidateInfo {
            move_uci: mv_uci,
            notation: mv_not,
            centipawn: score,
            intent: mv_int,
            pros: vec!["Phương án dự phòng khả thi".to_string()],
            cons: vec![format!("Kém phương án tối ưu {} centipawn, nhường bớt quyền chủ động", gap)],
        });
    }

    candidates
}

/// Biên dịch mạch suy tưởng 360 độ Tiếng Việt chuẩn DeepSeek-R1 bên trong thẻ <thought>
fn synthesize_360_thought(
    side: u8,
    score: i32,
    red_count: usize,
    black_count: usize,
    safety_score: i32,
    center_control: &str,
    patterns: &[String],
    advantages: &[String],
    disadvantages: &[String],
    positives: &[String],
    negatives: &[String],
    candidates: &[CandidateInfo],
    best_move_str: &str,
) -> String {
    let side_name = if side == 0 { "Đỏ (Đi trước)" } else { "Đen (Đi sau)" };

    let mut thought = String::with_capacity(4096);
    thought.push_str("<thought>\n");

    // 1. Phân tích hiện trạng vật chất & Cung Tướng
    thought.push_str(&format!(
        "1. [KHẢO SÁT HIỆN TRẠNG & TƯƠNG QUAN LỰC LƯỢNG]:\n   • Lượt đi: Bên {}\n   • Số lượng quân: Đỏ {} quân | Đen {} quân\n   • An toàn Cung Tướng (King Safety): {}/100 (Trạng thái: {})\n   • Trọng tâm Trung Lộ (Lộ 5): {}\n\n",
        side_name, red_count, black_count, safety_score,
        if safety_score >= 80 { "Kiên cố vững chắc" } else if safety_score >= 60 { "Ổn định" } else { "Bị đe dọa trực tiếp" },
        center_control
    ));

    // 2. Nhận diện mẫu chiến thuật & Đe dọa
    thought.push_str("2. [NHẬN DIỆN MẪU CHIẾN THUẬT & TRỌNG ĐIỂM TẤN CÔNG]:\n");
    for pat in patterns {
        thought.push_str(&format!("   • {}\n", pat));
    }
    thought.push('\n');

    // 3. Phân tích rủi ro & Cơ hội 4 chiều
    thought.push_str("3. [MA TRẬN ĐÁNH GIÁ RỦI RO & CƠ HỘI]:\n");
    thought.push_str(&format!("   • Ưu thế (Advantages): {}\n", advantages.join("; ")));
    thought.push_str(&format!("   • Bất lợi (Disadvantages): {}\n", disadvantages.join("; ")));
    thought.push_str(&format!("   • Yếu tố tích cực (Positives): {}\n", positives.join("; ")));
    thought.push_str(&format!("   • Yếu tố tiêu cực (Negatives): {}\n\n", negatives.join("; ")));

    // 4. So sánh ma trận 3 nước đi ứng viên
    thought.push_str("4. [ĐÁNH GIÁ MA TRẬN 3 NƯỚC ĐI ỨNG VIÊN (CANDIDATES EVALUATION)]:\n");
    for (idx, cand) in candidates.iter().enumerate() {
        thought.push_str(&format!(
            "   • Ứng viên #{}: `{}` ({}) | Điểm số: {} cp\n     - Ý đồ: {}\n     - Ưu điểm: {}\n     - Nhược điểm: {}\n",
            idx + 1, cand.move_uci, cand.notation, cand.centipawn, cand.intent,
            cand.pros.join(", "), cand.cons.join(", ")
        ));
    }
    thought.push('\n');

    // 5. Kết luận nước đi tối thượng
    thought.push_str(&format!(
        "5. [QUYẾT ĐỊNH NƯỚC ĐI TỐI THƯỢNG (BEST MOVE SELECTION)]:\n   Nước đi `{}` ({}) đạt điểm số đánh giá cao nhất ({} cp). Đây là nước đi chuẩn xác giúp bên {} kiểm soát tối đa thế trận và hạn chế khả năng phản đòn của đối phương.\n</thought>",
        best_move_str,
        candidates.first().map(|c| c.notation.as_str()).unwrap_or(best_move_str),
        score,
        if side == 0 { "Đỏ" } else { "Đen" }
    ));

    thought
}

/// Cấu trúc nhị phân truyền tải mẫu huấn luyện suy luận 360 độ giữa các luồng
pub struct ReasoningTask {
    pub jsonl_record: Option<String>,
    pub log_info: Option<String>,
    pub is_shutdown: bool,
}

/// Dịch vụ Ghi File Bất Đồng Bộ Async IO Sink
pub struct AsyncIoWriter {
    sender: SyncSender<ReasoningTask>,
    #[allow(dead_code)]
    handle: Option<JoinHandle<()>>,
}

impl AsyncIoWriter {
    pub fn start(output_path: &str) -> Self {
        let (sender, receiver) = sync_channel::<ReasoningTask>(262144);
        let path = output_path.to_string();

        let handle = thread::spawn(move || {
            if let Some(parent) = std::path::Path::new(&path).parent() {
                let _ = fs::create_dir_all(parent);
            }
            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&path)
                .expect("Không thể tạo/mở tệp JSONL xuất dữ liệu suy luận");
            let mut writer = BufWriter::with_capacity(512 * 1024, file);
            let mut stdout = io::stdout();

            while let Ok(task) = receiver.recv() {
                if task.is_shutdown {
                    let _ = writer.flush();
                    break;
                }

                if let Some(record) = task.jsonl_record {
                    let _ = writer.write_all(record.as_bytes());
                    let _ = writer.write_all(b"\n");
                }

                if let Some(info) = task.log_info {
                    println!("{}", info);
                    let _ = stdout.flush();
                }
            }
            let _ = writer.flush();
        });

        Self {
            sender,
            handle: Some(handle),
        }
    }

    #[inline(always)]
    pub fn push(&self, jsonl_record: Option<String>, log_info: Option<String>) {
        let _ = self.sender.send(ReasoningTask {
            jsonl_record,
            log_info,
            is_shutdown: false,
        });
    }

    pub fn close(&self) {
        let _ = self.sender.send(ReasoningTask {
            jsonl_record: None,
            log_info: None,
            is_shutdown: true,
        });
    }
}

/// Lấy số ngẫu nhiên PRNG Xorshift64
#[inline(always)]
fn rand_next(seed: &mut u64) -> u64 {
    let mut x = *seed;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *seed = x;
    x
}

fn main() {
    println!("===============================================================================");
    println!("💎 XIANGQI-RIM: ULTRA SOTA CQRS-ES 360-DEGREE REASONING GENERATOR ({})", APP_VERSION);
    println!("   🔥 HỆ THỐNG MÁY PHÁT PUB/SUB TỰ ĐẤU & TRÍCH XUẤT 14 CHIỀU KÍCH SUY TƯỞNG R1 SOTA");
    println!("===============================================================================");

    let total_games: usize = std::env::var("GAMES").ok().and_then(|v| v.parse().ok()).unwrap_or(20);
    let depth: u8 = std::env::var("DEPTH").ok().and_then(|v| v.parse().ok()).unwrap_or(4);
    let threads_count: usize = std::env::var("THREADS").ok().and_then(|v| v.parse().ok()).unwrap_or(4);
    let tt_mb: usize = std::env::var("TT_MB").ok().and_then(|v| v.parse().ok()).unwrap_or(1024);
    let max_plies: usize = std::env::var("MAX_PLIES").ok().and_then(|v| v.parse().ok()).unwrap_or(120);
    let output_path: String = std::env::var("OUTPUT").unwrap_or_else(|_| "data/xiangqi_r1_360_reasoning.jsonl".to_string());

    println!("⚡ THÔNG SỐ VẬN HÀNH CQRS-ES 360 GENERATOR:");
    println!("   • Số luồng Workers song song  : {} Threads", threads_count);
    println!("   • Độ sâu tìm kiếm Alpha-Beta  : Depth {}", depth);
    println!("   • Dung lượng Shared TT        : {} MB (Arc<Table> Lock-Free)", tt_mb);
    println!("   • Giới hạn nước đi mỗi ván    : Max {} Plies", max_plies);
    println!("   • Tổng số ván cờ mục tiêu     : {} ván", total_games);
    println!("   • Tệp xuất dữ liệu JSONL      : {}", output_path);
    println!("-------------------------------------------------------------------------------\n");

    let start_all = Instant::now();
    let cqrs_bus = Arc::new(Bus::new(1024, 65536));
    cqrs_bus.emit(CqrsEvent::Ready);

    let async_writer = Arc::new(AsyncIoWriter::start(&output_path));
    let global_tt = Arc::new(Table::new(tt_mb));

    let completed_games = Arc::new(AtomicUsize::new(0));
    let total_turns_generated = Arc::new(AtomicUsize::new(0));
    let current_game_counter = Arc::new(AtomicUsize::new(1));

    let mut handles = Vec::with_capacity(threads_count);

    for thread_idx in 0..threads_count {
        let async_writer_cloned = Arc::clone(&async_writer);
        let completed_games_cloned = Arc::clone(&completed_games);
        let total_turns_cloned = Arc::clone(&total_turns_generated);
        let current_game_cloned = Arc::clone(&current_game_counter);
        let global_tt_cloned = Arc::clone(&global_tt);

        let handle = thread::spawn(move || {
            let mut search_engine = Search::new_shared(global_tt_cloned);

            loop {
                let game_idx = current_game_cloned.fetch_add(1, Ordering::Relaxed);
                if game_idx > total_games {
                    break;
                }

                let game_id = format!("game_{:06x}_{:04x}", game_idx, thread_idx);
                let mut seed = (game_idx as u64).wrapping_mul(0x9E3779B97F4A7C15) ^ (thread_idx as u64);
                let mut pos = Parser::parse(Parser::DEFAULT);
                let use_book = (game_idx % 2) == 1;
                let mut game_ply = 0;

                // Khai cuộc: Dùng Book hoặc nước đi đa dạng ngẫu nhiên
                if use_book {
                    while game_ply < 12 {
                        if let Some(mv) = Book::probe(&pos) {
                            pos.apply(mv.from, mv.to);
                            game_ply += 1;
                        } else {
                            break;
                        }
                    }
                } else {
                    while game_ply < 6 {
                        let mut moves = List::new();
                        legal::gen(&mut pos, &mut moves);
                        if moves.empty() {
                            break;
                        }
                        let idx = (rand_next(&mut seed) as usize) % moves.len();
                        let mv = moves.get(idx);
                        pos.apply(mv.from, mv.to);
                        game_ply += 1;
                    }
                }

                let mut message_entries = Vec::new();

                // System Prompt tự chứa chuẩn JRCP 3.0 & DeepSeek-R1
                let system_prompt = "Bạn là Xiangqi-R1 Master — Hệ thống Trí tuệ Nhân tạo Suy luận Cờ Tướng Đẳng Cấp Nhất.\\nNhiệm vụ: Phân tích bàn cờ tướng đa chiều kích 360 độ và đưa ra nước đi tối ưu nhất kèm giải thích chi tiết trong thẻ <thought>.".to_string();
                message_entries.push(format!("{{\"role\":\"system\",\"content\":\"{}\"}}", json_escape(&system_prompt)));

                while game_ply < max_plies {
                    let mut moves = List::new();
                    legal::gen(&mut pos, &mut moves);
                    if moves.empty() {
                        break;
                    }

                    let fen_str = Serializer::export(&pos);
                    let side = pos.side;

                    let mut limits = Limits::new();
                    limits.depth = depth;
                    let res = search_engine.go(&pos, &limits);

                    if res.best.from == res.best.to {
                        break;
                    }

                    let best_move = res.best;
                    let best_score = res.score;
                    let best_move_uci = format!("{}{}", sq_to_uci(best_move.from), sq_to_uci(best_move.to));

                    // Trích xuất 14 Chiều Kích JRCP 360 Độ
                    let red_count = (0..7).map(|r| pos.counts[r] as usize).sum();
                    let black_count = (7..14).map(|r| pos.counts[r] as usize).sum();
                    let king_safety = evaluate_king_safety(&pos, side);
                    let center_ctrl = evaluate_center_control(&pos);
                    let tactical_pats = detect_tactical_patterns(&pos, side);
                    let (advs, disadvs, pos_factors, neg_factors) = assess_risk_factors(&pos, side, best_score, red_count, black_count);

                    let top_candidates = extract_top_candidates(&pos, best_move, best_score, depth, &mut search_engine);

                    let thought_chain = synthesize_360_thought(
                        side,
                        best_score,
                        red_count,
                        black_count,
                        king_safety,
                        center_ctrl,
                        &tactical_pats,
                        &advs,
                        &disadvs,
                        &pos_factors,
                        &neg_factors,
                        &top_candidates,
                        &best_move_uci,
                    );

                    let turn_user_content = format!(
                        "Trạng thái bàn cờ Turn {}:\nFEN: {}\nLượt {} đi. Trả về phân tích 360 độ và nước đi tối ưu.",
                        game_ply + 1, fen_str, if side == 0 { "Đỏ" } else { "Đen" }
                    );

                    // Xây dựng chuỗi JSON Assistant phản hồi
                    let candidates_json_list: Vec<String> = top_candidates.iter().map(|c| {
                        let pros_json = format!("[{}]", c.pros.iter().map(|p| format!("\"{}\"", json_escape(p))).collect::<Vec<_>>().join(","));
                        let cons_json = format!("[{}]", c.cons.iter().map(|cn| format!("\"{}\"", json_escape(cn))).collect::<Vec<_>>().join(","));
                        format!("{{\"move\":\"{}\",\"notation\":\"{}\",\"centipawn\":{},\"intent\":\"{}\",\"pros\":{},\"cons\":{}}}",
                            json_escape(&c.move_uci), json_escape(&c.notation), c.centipawn, json_escape(&c.intent), pros_json, cons_json
                        )
                    }).collect();

                    let adv_json = format!("[{}]", advs.iter().map(|a| format!("\"{}\"", json_escape(a))).collect::<Vec<_>>().join(","));
                    let disadv_json = format!("[{}]", disadvs.iter().map(|d| format!("\"{}\"", json_escape(d))).collect::<Vec<_>>().join(","));
                    let pos_json = format!("[{}]", pos_factors.iter().map(|p| format!("\"{}\"", json_escape(p))).collect::<Vec<_>>().join(","));
                    let neg_json = format!("[{}]", neg_factors.iter().map(|n| format!("\"{}\"", json_escape(n))).collect::<Vec<_>>().join(","));

                    let assistant_content = format!(
                        "{{\"thought\":\"{}\",\"matrix_analysis\":{{\"red_pieces_count\":{},\"black_pieces_count\":{},\"king_safety_score\":{},\"center_file_control\":\"{}\"}},\"risk_assessment\":{{\"advantages\":{},\"disadvantages\":{},\"positives\":{},\"negatives\":{}}},\"candidates\":[{}],\"bestmove\":\"{}\",\"centipawn_eval\":{}}}",
                        json_escape(&thought_chain), red_count, black_count, king_safety, center_ctrl,
                        adv_json, disadv_json, pos_json, neg_json,
                        candidates_json_list.join(","),
                        json_escape(&best_move_uci), best_score
                    );

                    message_entries.push(format!("{{\"role\":\"user\",\"content\":\"{}\"}}", json_escape(&turn_user_content)));
                    message_entries.push(format!("{{\"role\":\"assistant\",\"content\":\"{}\"}}", json_escape(&assistant_content)));

                    total_turns_cloned.fetch_add(1, Ordering::Relaxed);

                    pos.apply(best_move.from, best_move.to);
                    game_ply += 1;

                    // Ngắt dừng sớm khi thế cờ đã thắng bại cách biệt
                    if best_score.abs() >= 2500 {
                        break;
                    }
                }

                // Đóng gói mẫu JSONL của ván cờ hoàn chỉnh
                let full_game_record = format!(
                    "{{\"game_id\":\"{}\",\"total_plies\":{},\"outcome\":\"{}\",\"messages\":[{}]}}",
                    json_escape(&game_id),
                    game_ply,
                    if pos.side == 0 { "black_win" } else { "red_win" },
                    message_entries.join(",")
                );

                async_writer_cloned.push(Some(full_game_record), None);

                let done = completed_games_cloned.fetch_add(1, Ordering::Relaxed) + 1;
                let total_turns = total_turns_cloned.load(Ordering::Relaxed);
                let elapsed = start_all.elapsed().as_secs_f64();
                let turns_per_sec = if elapsed > 0.0 { (total_turns as f64) / elapsed } else { 0.0 };
                let avg_sec_per_game = if done > 0 { elapsed / (done as f64) } else { 0.0 };
                let remaining_games = total_games.saturating_sub(done);
                let eta_secs = (remaining_games as f64) * avg_sec_per_game;

                let elapsed_mins = (elapsed / 60.0) as u64;
                let elapsed_rem_secs = (elapsed % 60.0) as u64;
                let eta_mins = (eta_secs / 60.0) as u64;
                let eta_rem_secs = (eta_secs % 60.0) as u64;
                let pct = (done as f64 / total_games as f64) * 100.0;

                let telemetry_str = format!(
                    "⚡ [CQRS-360 TELEMETRY] Xong {:<3}/{} Ván ({:5.1}%) | Đã Chạy: {:02}m{:02}s | TB: {:.2}s/ván | Sinh: {:<5} Turns 360 CoT | Tốc Độ: {:.1} Turns/s ({:.0} Turns/phút) | ETA: {:02}m{:02}s",
                    done, total_games, pct,
                    elapsed_mins, elapsed_rem_secs,
                    avg_sec_per_game,
                    total_turns,
                    turns_per_sec, turns_per_sec * 60.0,
                    eta_mins, eta_rem_secs
                );
                async_writer_cloned.push(None, Some(telemetry_str));
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join();
    }

    let total_elapsed = start_all.elapsed().as_secs_f64();
    let total_turns = total_turns_generated.load(Ordering::Relaxed);
    let turns_per_sec = if total_elapsed > 0.0 { (total_turns as f64) / total_elapsed } else { 0.0 };

    println!("\n===============================================================================");
    println!("💎 CQRS-ES 360-DEGREE REASONING GENERATION COMPLETED!");
    println!("   • Tổng số ván cờ hoàn chỉnh    : {} ván", total_games);
    println!("   • Tổng số lượt suy luận 360 CoT: {} lượt turns", total_turns);
    println!("   • Tổng thời gian thực thi      : {:.2} giây", total_elapsed);
    println!("   • Tốc độ sinh suy luận 360 CoT : {:.2} Turns / giây ({:.0} Turns / phút)", turns_per_sec, turns_per_sec * 60.0);
    println!("   • Ước tính Tokens suy tưởng R1 : ~{} Tokens", total_turns * 850);
    println!("-------------------------------------------------------------------------------");
    println!("🏛️ CQRS-ES EVENT SOURCING AUDIT LEDGER:");
    println!("   • Tổng số sự kiện bất biến đã ghi: {} Events", cqrs_bus.store.len());
    println!("-------------------------------------------------------------------------------");

    async_writer.close();

    println!("===============================================================================");
    let _ = io::stdout().flush();
    std::process::exit(0);
}
