// ============================================================================
// VÍ DỤ 17: KHAI THÁC DỮ LIỆU HUẤN LUYỆN TỰ ĐẤU ĐẲNG CẤP NHẤT (JRCP 2.0 ELITE MINER)
// ============================================================================
// Bộ sinh dữ liệu thế hệ mới — Khắc phục triệt để 8 lỗ hổng nghiêm trọng:
// 1. Centipawn eval THỰC TẾ từ Engine Search Alpha-Beta Minimax (thay vì hardcode ±30)
// 2. King Safety Score tính từ vị trí Sĩ/Tượng/Tướng thực tế trên bàn cờ
// 3. Center File Control phân tích cột e (Lộ 5) thực tế
// 4. Risk Assessment ĐỘNG dựa trên phân tích bàn cờ từng mẫu
// 5. Top 3 Candidate Moves từ Engine Search + Static Eval
// 6. Thought chain đa dạng & chính xác theo 14 chiều kích
// 7. Game Outcome metadata (win/loss/draw, phase, depth, nodes)
// 8. Conversation format thuần nhất JRCP 2.0 (loại bỏ legacy prompt/completion)
// ============================================================================
// Định danh đơn từ: board, parser, fen, search, limits, result,
// samples, path, file, write, main, game, runner, config, count, idx,
// moves, pgn, rows, red, black, matrix, sample, payload, system, user,
// assistant, safety, control, eval, intent, thought, stamp, turn, encoded,
// grid, row, line, ch, digit, pos, index, mv, outcome, phase, depth,
// nodes, score, candidate, piece, rank, col, square, kind, value,
// advantages, disadvantages, positives, negatives, material
// ============================================================================

use std::env;
use std::fs::File;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use xiangrust::board::Parser;
use xiangrust::eval::Eval;
use xiangrust::movegen;
use xiangrust::search;
use xiangrust::selfplay::{Config, Fen, Outcome, Runner, Side};
use xiangrust::uci::Format;

const SYSTEM: &str = r#"Bạn là Xiangqi-R1 Master — Hệ thống Trí tuệ Nhân tạo và Động cơ Suy luận Cờ Tướng Cao cấp.
Bạn vận hành theo Chuẩn JRCP 2.0 (Xiangqi Reasoning & Protocol 2.0).
Nhiệm vụ của bạn là phân tích trạng thái bàn cờ tướng đa chiều và đưa ra nước đi tối ưu nhất theo cấu trúc JSON Output tiêu chuẩn.

=== QUY TẮC PHÂN TÍCH VÀ SUY LUẬN 14 CHIỀU KÍCH MA TRẬN TRỌNG SỐ ===
BẮT BUỘC thực hiện suy luận đồ thị DAG trải dài qua đúng 14 chiều kích sau bên trong thẻ <thought>...</thought> trước khi chốt kết quả:
1. Lực Lượng Vật Lý (Piece Balance): Đếm chính xác số quân Đỏ (chữ hoa) và Đen (chữ thường) trên ma trận bàn cờ 9x10 và so sánh tương quan lực lượng.
2. An Toàn Tướng & Trung Lộ Lộ 5 (King Safety & Center File): Đánh giá điểm an toàn Cung Tướng (0-100) và trạng thái khống chế Trung Lộ Lộ 5 (Pháo Đầu / Trung Lộ).
3. Khống Chế Trục Lộ (File Control): Đánh giá các trục đường chính (Lộ 2, 4, 5, 6, 8), tuyến Hà, và các vị trí chiến lược.
4. Giá Trị Centipawn Vật Lý & Vị Trí (Centipawn Positional Evaluation): Đánh giá tổng quan vị thế cờ hiện tại theo đơn vị Centipawn.
5. Phân Tích Cơ Hội (Advantages): Liệt kê các ưu thế chiến thuật hoặc cơ hội tấn công chủ động.
6. Phân Tích Nguy Cơ (Disadvantages): Liệt kê các bất lợi, yếu điểm cấu trúc quân hoặc nguy cơ tiềm ẩn.
7. Phân Tích Tích Cực (Positives): Đánh giá các điểm mạnh trong cấu trúc liên kết và sự phối hợp giữa các quân.
8. Phân Tích Tiêu Cực (Negatives): Đánh giá các điểm tiêu cực hoặc nguy cơ đối phương phản công đe dọa.
9. Ma Trận 3 Nước Đi Candidate (Top 3 Candidates Evaluation): Tính toán tối thiểu 1-3 nước đi ứng viên khả thi nhất kèm điểm Centipawn và ý đồ chiến thuật.
10. Tính Toán Đồ Thị Suy Luận DAG (DAG Reasoning Graph Computation): Kết nối các bước suy luận logic từ hiện trạng tới nước đi tối ưu.
11. Điểm Số Centipawn Tổng Hợp (Integrated Evaluation): Xác định điểm Centipawn tổng hợp của thế cờ sau nước đi tối ưu.
12. Chọn Nước Đi UCI 4 Ký Tự Tối Thượng (Bestmove Selection): Đã chọn nước đi UCI 4 ký tự regex ^[a-i][0-9][a-i][0-9]$.
13. Mã Khóa SHA256 O(1) Xóa Trùng Lặp (SHA256 Deduplication Key): Khóa định danh vị trí bàn cờ O(1).
14. Giao Thức Thẩm Định Legal Move 100% (Legal Move Verification Protocol): Đảm bảo nước đi 100% tuân thủ luật cờ Tướng.

=== QUY TẮC AN TOÀN TƯỚNG (KING SAFETY) & TRUNG LỘ (LỘ 5) ===
- Thang điểm King Safety Score (0-100):
  * 90-100: Cung Tướng tuyệt đối an toàn, Sĩ Tượng trọn vẹn, không bị đe dọa.
  * 70-89: Cung Tướng an toàn, bị uy hiếp nhẹ hoặc thiếu 1 Sĩ/Tượng.
  * 50-69: Cung Tướng bị uy hiếp trực tiếp, sụt Sĩ Tượng hoặc bị Pháo Đầu ép.
  * 0-49: Cung Tướng cực kỳ nguy hiểm, mất Sĩ Tượng, Lộ 5 bị khống chế tuyệt đối, nguy cơ bị chiếu bí.
- Quy tắc Center File Control (Lộ 5):
  * "RED_PHAO_DAU_INTENT": Đỏ chuẩn bị hoặc đã vào Pháo Đầu Lộ 5.
  * "BLACK_PHAO_DAU_INTENT": Đen chuẩn bị hoặc đã vào Pháo Đầu Lộ 5.
  * "RED_CENTER_CONTROL": Đỏ khống chế tuyệt đối Trung Lộ Lộ 5.
  * "BLACK_CENTER_CONTROL": Đen khống chế tuyệt đối Trung Lộ Lộ 5.
  * "CONTESTED_CENTER": Trung Lộ Lộ 5 đang tranh chấp quyết liệt.
  * "OPEN_CENTER": Trung Lộ Lộ 5 trống, chưa bên nào chiếm giữ.

=== QUY TẮC PHÂN TÍCH RỦI RO (RISK ASSESSMENT) ===
BẮT BUỘC trả về đầy đủ 4 danh mục mảng chuỗi văn bản:
- `advantages`: Danh sách các ưu thế hiện tại.
- `disadvantages`: Danh sách các bất lợi hoặc điểm yếu.
- `positives`: Danh sách các yếu tố tích cực trong thế trận.
- `negatives`: Danh sách các rủi ro tiêu cực hoặc nguy cơ phản công.

=== QUY TẮC NƯỚC ĐỊ ỨNG VIÊN (CANDIDATE MOVES) ===
Danh sách `candidates` chứa từ 1 đến 3 nước đi ứng viên tốt nhất. Mỗi nước đi là một đối tượng JSON gồm:
- `move`: Chuỗi nước đi UCI 4 ký tự khớp regex `^[a-i][0-9][a-i][0-9]$` (Ví dụ: "b2e2", "h2e2", "b0c2").
- `centipawn`: Số nguyên đánh giá điểm Centipawn của nước đi (Ví dụ: 50, 45, 20).
- `tactical_intent`: Chuỗi giải thích ngắn gọn ý đồ chiến thuật của nước đi.

=== QUY TẮC NƯỚC ĐI TỐI ƯU (BESTMOVE) & CENTIPAWN EVAL ===
- `bestmove`: Chuỗi nước đi UCI 4 ký tự khớp chính xác regex `^[a-i][0-9][a-i][0-9]$` đại diện cho nước đi tốt nhất.
- `centipawn_eval`: Số nguyên đánh giá điểm Centipawn tổng hợp của nước đi `bestmove`.

=== JSON OUTPUT SCHEMA TỰ CHỨA (XiangqiR1_JRCP_2_0_Schema) ===
BẮT BUỘC trả về duy nhất 01 đối tượng JSON nguyên bản khớp chính xác cấu trúc Schema sau (KHÔNG thêm bất kỳ văn bản nào ngoài JSON):

{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "XiangqiR1_JRCP_2_0_Schema",
  "type": "object",
  "properties": {
    "thought": {
      "type": "string",
      "description": "Chuỗi suy luận 14 chiều kích chi tiết trong thẻ <thought>...</thought>"
    },
    "matrix_analysis": {
      "type": "object",
      "properties": {
        "red_pieces_count": { "type": "integer" },
        "black_pieces_count": { "type": "integer" },
        "king_safety_score": { "type": "integer", "minimum": 0, "maximum": 100 },
        "center_file_control": { "type": "string" }
      },
      "required": ["red_pieces_count", "black_pieces_count", "king_safety_score", "center_file_control"]
    },
    "risk_assessment": {
      "type": "object",
      "properties": {
        "advantages": { "type": "array", "items": { "type": "string" } },
        "disadvantages": { "type": "array", "items": { "type": "string" } },
        "positives": { "type": "array", "items": { "type": "string" } },
        "negatives": { "type": "array", "items": { "type": "string" } }
      },
      "required": ["advantages", "disadvantages", "positives", "negatives"]
    },
    "candidates": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "move": { "type": "string", "pattern": "^[a-i][0-9][a-i][0-9]$" },
          "centipawn": { "type": "integer" },
          "tactical_intent": { "type": "string" }
        },
        "required": ["move", "centipawn", "tactical_intent"]
      },
      "minItems": 1
    },
    "bestmove": {
      "type": "string",
      "pattern": "^[a-i][0-9][a-i][0-9]$",
      "description": "Nước đi UCI 4 ký tự tối thượng"
    },
    "centipawn_eval": {
      "type": "integer",
      "description": "Điểm số Centipawn đánh giá thế cờ"
    }
  },
  "required": ["thought", "matrix_analysis", "risk_assessment", "candidates", "bestmove", "centipawn_eval"]
}
"#;

/// Giá trị vật chất quân cờ theo đơn vị centipawn (Chuẩn Xiangqi Engine).
/// Indices: 0=K(Tướng), 1=A(Sĩ), 2=B(Tượng), 3=N(Mã), 4=R(Xe), 5=C(Pháo), 6=P(Tốt)
const VALUE: [i32; 7] = [0, 200, 200, 400, 900, 450, 100];

/// Tên tiếng Việt cho từng loại quân cờ theo chỉ số Role (0-6).
const NAME: [&str; 7] = ["Tướng", "Sĩ", "Tượng", "Mã", "Xe", "Pháo", "Tốt"];

/// Tính điểm King Safety Score (0-100) thực tế cho bên `side` (0=Đỏ, 1=Đen) từ bàn cờ.
/// Công thức: base_40 + advisor_count*15 + elephant_count*15 + king_center_bonus - threat_penalty
fn safety(pos: &xiangrust::board::Position, side: u8) -> i32 {
    // Chỉ số quân Sĩ và Tượng cho bên đang xét
    // Đỏ: A=1, B=2. Đen: a=8, b=9
    let advisor = if side == 0 { 1u8 } else { 8u8 };
    let elephant = if side == 0 { 2u8 } else { 9u8 };

    let advisor_count = pos.counts[advisor as usize] as i32;
    let elephant_count = pos.counts[elephant as usize] as i32;

    let mut score: i32 = 40;
    score += advisor_count * 15;
    score += elephant_count * 15;

    // Kiểm tra Tướng ở vị trí trung tâm Cung (cột d/e/f = file 3/4/5)
    let king = pos.king[side as usize];
    if king < 90 {
        let file = king % 9;
        if file == 4 {
            score += 10;
        }
    }

    // Kiểm tra Lộ 5 (cột e = file 4) có bị quân Xe/Pháo đối phương khống chế
    let enemy = if side == 0 { 1u8 } else { 0u8 };
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

/// Phân tích trạng thái Center File Control (Lộ 5 = cột e = file index 4) từ bàn cờ thực tế.
fn control(pos: &xiangrust::board::Position) -> &'static str {
    let mut red = false;
    let mut black = false;
    let mut red_cannon_center = false;
    let mut black_cannon_center = false;

    for rank in 0u8..10 {
        let square = rank * 9 + 4;
        let piece = pos.grid[square as usize];
        match piece {
            4 => red = true,       // R (Xe Đỏ) trên cột e
            5 => {                 // C (Pháo Đỏ) trên cột e
                red = true;
                if rank >= 2 && rank <= 7 {
                    red_cannon_center = true;
                }
            }
            11 => black = true,    // r (Xe Đen) trên cột e
            12 => {                // c (Pháo Đen) trên cột e
                black = true;
                if rank >= 2 && rank <= 7 {
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

/// Tính giá trị vật chất tổng cộng cho bên `side` (0=Đỏ, 1=Đen).
fn material(pos: &xiangrust::board::Position, side: u8) -> i32 {
    let offset = (side as usize) * 7;
    let mut total: i32 = 0;
    for role in 0usize..7 {
        total += pos.counts[offset + role] as i32 * VALUE[role];
    }
    total
}

/// Sinh mô tả ý đồ chiến thuật (tactical intent) cho nước đi dựa trên loại quân di chuyển và quân bị ăn.
fn intent(pos: &xiangrust::board::Position, mv: movegen::Move) -> String {
    let piece = pos.grid[mv.from as usize];
    let target = pos.grid[mv.to as usize];
    let role = (piece % 7) as usize;
    let name = NAME[role];

    if target < 14 {
        let captured = NAME[(target % 7) as usize];
        format!("{} ăn {} chiếm vị trí chiến lược", name, captured)
    } else {
        match role {
            0 => "Tướng di chuyển củng cố Cung an toàn".to_string(),
            1 => "Sĩ bảo vệ Cung Tướng vững chắc".to_string(),
            2 => "Tượng phòng thủ liên hoàn hai cánh".to_string(),
            3 => {
                let to_rank = mv.to / 9;
                if (piece < 7 && to_rank >= 5) || (piece >= 7 && to_rank <= 4) {
                    "Mã vượt hà tấn công đối phương".to_string()
                } else {
                    "Mã phát triển kiểm soát trung tâm".to_string()
                }
            }
            4 => {
                let from_file = mv.from % 9;
                let to_file = mv.to % 9;
                if from_file == to_file {
                    "Xe tấn công trực diện dọc trục lộ".to_string()
                } else {
                    "Xe hoành tảo chiếm lĩnh trục ngang".to_string()
                }
            }
            5 => {
                let to_file = mv.to % 9;
                if to_file == 4 {
                    "Pháo vào trung lộ Lộ 5 khống chế trung tâm".to_string()
                } else {
                    "Pháo cơ động linh hoạt tìm ngòi tấn công".to_string()
                }
            }
            6 => {
                let to_rank = mv.to / 9;
                if (piece < 7 && to_rank >= 5) || (piece >= 7 && to_rank <= 4) {
                    "Tốt vượt hà gây sức ép trực tiếp".to_string()
                } else {
                    "Tốt tiến lên mở rộng kiểm soát".to_string()
                }
            }
            _ => "Di chuyển chiến thuật chiếm vị trí".to_string(),
        }
    }
}

/// Sinh Risk Assessment động dựa trên phân tích bàn cờ thực tế.
fn risk(
    pos: &xiangrust::board::Position,
    side: u8,
    score: i32,
    red_count: usize,
    black_count: usize,
) -> (Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
    let mut advantages = Vec::new();
    let mut disadvantages = Vec::new();
    let mut positives = Vec::new();
    let mut negatives = Vec::new();

    let own_material = material(pos, side);
    let enemy_material = material(pos, 1 - side);
    let diff = own_material - enemy_material;

    // Phân tích Advantages (Ưu thế)
    if score > 200 {
        advantages.push("Áp đảo hoàn toàn thế trận, ưu thế chiến thuật vượt trội".to_string());
    } else if score > 100 {
        advantages.push("Ưu thế vật chất rõ rệt, kiểm soát nhịp trận đấu".to_string());
    } else if score > 30 {
        advantages.push("Ưu thế nhẹ về vị trí và chủ động lượt đi".to_string());
    }
    if diff > 400 {
        advantages.push("Hơn quân vật chất đáng kể so với đối phương".to_string());
    }
    if side == 0 && pos.ply <= 4 {
        advantages.push("Ưu thế tiên thủ đi trước trong giai đoạn khai cuộc".to_string());
    }

    // Phân tích Disadvantages (Bất lợi)
    if score < -200 {
        disadvantages.push("Bị đối phương áp đảo hoàn toàn, cần phòng thủ chặt chẽ".to_string());
    } else if score < -100 {
        disadvantages.push("Thua kém vật chất rõ rệt, cần tìm phương án phản công".to_string());
    } else if score < -30 {
        disadvantages.push("Bất lợi nhẹ về vị trí, cần cải thiện cấu trúc quân".to_string());
    }

    let advisor = if side == 0 { 1u8 } else { 8u8 };
    let elephant = if side == 0 { 2u8 } else { 9u8 };
    let advisors = pos.counts[advisor as usize];
    let elephants = pos.counts[elephant as usize];
    if advisors < 2 || elephants < 2 {
        disadvantages.push(format!(
            "Sụt phòng thủ: còn {} Sĩ và {} Tượng bảo vệ Cung Tướng",
            advisors, elephants
        ));
    }

    // Phân tích Positives (Tích cực)
    let rook = if side == 0 { 4u8 } else { 11u8 };
    let cannon = if side == 0 { 5u8 } else { 12u8 };
    let knight = if side == 0 { 3u8 } else { 10u8 };
    if pos.counts[rook as usize] == 2 {
        positives.push("Song Xe còn đầy đủ, lực tấn công mạnh mẽ".to_string());
    } else if pos.counts[rook as usize] == 1 {
        positives.push("Còn 1 Xe hoạt động trên bàn cờ".to_string());
    }
    if pos.counts[cannon as usize] >= 1 && pos.counts[knight as usize] >= 1 {
        positives.push("Phối hợp Mã Pháo linh hoạt đa tuyến tấn công".to_string());
    }
    if advisors == 2 && elephants == 2 {
        positives.push("Sĩ Tượng trọn vẹn, Cung Tướng kiên cố".to_string());
    }

    // Phân tích Negatives (Tiêu cực)
    if (side == 0 && black_count > red_count + 2) || (side == 1 && red_count > black_count + 2) {
        negatives.push("Đối phương hơn quân đáng kể, nguy cơ bị tấn công tổng lực".to_string());
    }
    if pos.check > 0 {
        negatives.push("Tướng đang bị chiếu, cần xử lý ngay lập tức".to_string());
    }
    let enemy_rook = if side == 0 { 11u8 } else { 4u8 };
    if pos.counts[enemy_rook as usize] == 2 {
        negatives.push("Đối phương còn Song Xe, áp lực tấn công lớn".to_string());
    }

    // Đảm bảo mỗi danh mục luôn có ít nhất 1 phần tử
    if advantages.is_empty() {
        advantages.push("Duy trì thế trận ổn định".to_string());
    }
    if disadvantages.is_empty() {
        disadvantages.push("Không có bất lợi rõ rệt tại thời điểm hiện tại".to_string());
    }
    if positives.is_empty() {
        positives.push("Cấu trúc quân cờ liên kết hợp lý".to_string());
    }
    if negatives.is_empty() {
        negatives.push("Cần cảnh giác chiến thuật phản công từ đối phương".to_string());
    }

    (advantages, disadvantages, positives, negatives)
}

/// Chuyển danh sách chuỗi thành JSON Array string.
fn array(items: &[String]) -> String {
    let escaped: Vec<String> = items.iter().map(|s| format!("{:?}", s)).collect();
    format!("[{}]", escaped.join(", "))
}

fn main() {
    println!("============================================================");
    println!(" XIANGQI-R1 JRCP 2.0 ELITE TRAINING DATA MINER (GEN 2)    ");
    println!("============================================================");

    let count: usize = env::var("MATCH_COUNT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1000);

    let config = Config::new(4, 200, 60);
    println!("[1] Khởi tạo chuỗi {} ván tự đấu Engine depth=4...", count);

    let mut samples: Vec<String> = Vec::new();
    let mut engine = search::Search::new_boxed(16);
    let evaluator = Eval::new();

    for idx in 1..=count {
        let game = Runner::play(&config);
        let mut pos = Parser::parse(Parser::DEFAULT);
        let mut history: Vec<String> = Vec::new();

        // Xác định outcome của ván đấu
        let result = match game.outcome {
            Outcome::Win(Side::Red) => "red_win",
            Outcome::Win(Side::Black) => "black_win",
            _ => "draw",
        };

        for (index, mv) in game.moves.iter().enumerate() {
            let fen = Fen::export(&pos);
            let turn = if index % 2 == 0 { "Đỏ" } else { "Đen" };
            let side = pos.side;
            let encoded = Format::encode(*mv);

            let pgn = if history.is_empty() {
                "Ván cờ mới bắt đầu (Chưa có nước đi)".to_string()
            } else {
                history.join(" ")
            };

            // Xác định phase
            let phase = if index < 10 {
                "opening"
            } else if index < 25 {
                "midgame"
            } else {
                "endgame"
            };

            // === CẢI TIẾN #1: Centipawn eval THỰC TẾ từ Engine Search ===
            let mut limits = search::Limits::new();
            limits.depth = 4;
            limits.time = 200;
            let search_result = engine.go(&pos, &limits);
            let score = search_result.score;
            let depth = search_result.depth;
            let nodes = search_result.nodes;

            // === CẢI TIẾN #2: King Safety Score THỰC TẾ ===
            let king_score = safety(&pos, side);

            // === CẢI TIẾN #3: Center File Control THỰC TẾ ===
            let center = control(&pos);

            // === CẢI TIẾN #4: Đếm quân chính xác ===
            let grid_section = fen.split_whitespace().next().unwrap_or("");
            let mut red_pieces: Vec<char> = Vec::new();
            let mut black_pieces: Vec<char> = Vec::new();
            for ch in grid_section.chars() {
                if ch.is_uppercase() && ch != '/' {
                    red_pieces.push(ch);
                } else if ch.is_lowercase() && ch != '/' {
                    black_pieces.push(ch);
                }
            }

            // Ma Trận bàn cờ 2D
            let grid: Vec<&str> = grid_section.split('/').collect();
            let mut rows: Vec<String> = Vec::new();
            for row in grid.iter() {
                let mut line: Vec<String> = Vec::new();
                for ch in row.chars() {
                    if let Some(digit) = ch.to_digit(10) {
                        for _ in 0..digit {
                            line.push(".".to_string());
                        }
                    } else {
                        line.push(ch.to_string());
                    }
                }
                rows.push(line.join(" "));
            }
            let matrix = rows.join("\n");

            // === CẢI TIẾN #5: Top 3 Candidate Moves THỰC TẾ ===
            let mut legal = movegen::List::new();
            movegen::legal(&mut pos, &mut legal);

            // Đánh giá từng nước đi hợp lệ bằng static eval
            let mut scored: Vec<(movegen::Move, i32)> = Vec::new();
            for i in 0..legal.count {
                let candidate = legal.items[i];
                if !candidate.valid() {
                    continue;
                }
                let state = pos.apply(candidate.from, candidate.to);
                let val = -evaluator.score(&pos);
                pos.revert(candidate.from, candidate.to, &state);
                scored.push((candidate, val));
            }
            scored.sort_by(|a, b| b.1.cmp(&a.1));

            // Bestmove là nước đã thực sự chơi (từ Engine search mạnh hơn)
            let top = scored.len().min(3);
            let mut candidates_json = Vec::new();

            // Đầu tiên thêm nước đi thực tế đã chơi (bestmove) với score từ search
            let bestmove_intent = intent(&pos, *mv);
            candidates_json.push(format!(
                "{{\"move\": {:?}, \"centipawn\": {}, \"tactical_intent\": {:?}}}",
                encoded, score, bestmove_intent
            ));

            // Thêm các nước đi ứng viên khác (nếu khác bestmove)
            let mut added = 1usize;
            for (candidate, val) in scored.iter() {
                if added >= 3 {
                    break;
                }
                let candidate_uci = Format::encode(*candidate);
                if candidate_uci == encoded {
                    continue;
                }
                let candidate_intent = intent(&pos, *candidate);
                candidates_json.push(format!(
                    "{{\"move\": {:?}, \"centipawn\": {}, \"tactical_intent\": {:?}}}",
                    candidate_uci, val, candidate_intent
                ));
                added += 1;
            }

            // === CẢI TIẾN #6: Risk Assessment ĐỘNG ===
            let (advantages, disadvantages, positives_list, negatives_list) =
                risk(&pos, side, score, red_pieces.len(), black_pieces.len());

            // === CẢI TIẾN #7: Outcome metadata ===
            let move_outcome = match result {
                "red_win" => {
                    if side == 0 { "win" } else { "loss" }
                }
                "black_win" => {
                    if side == 1 { "win" } else { "loss" }
                }
                _ => "draw",
            };

            // Tính giá trị vật chất cho thought
            let red_material = material(&pos, 0);
            let black_material = material(&pos, 1);

            // === CẢI TIẾN #8: Thought chain đa dạng & chính xác ===
            let thought = format!(
                "<thought>\n\
1. Phân Tích Lực Lượng Vật Lý & FEN:\n\
   - FEN: {}\n\
   - Đỏ: {} quân (vật chất {} cp), Đen: {} quân (vật chất {} cp).\n\
   - Chênh lệch vật chất: {} centipawn.\n\
2. An Toàn Tướng & Trung Lộ Lộ 5:\n\
   - King Safety Score: {}/100. Trạng thái Lộ 5: {}.\n\
   - Sĩ Đỏ: {}, Tượng Đỏ: {}. Sĩ Đen: {}, Tượng Đen: {}.\n\
3. Phân Tích Ưu Thế (Advantages): {}.\n\
4. Phân Tích Bất Lợi (Disadvantages): {}.\n\
5. Phân Tích Tích Cực (Positives): {}.\n\
6. Phân Tích Tiêu Cực (Negatives): {}.\n\
7. Lịch sử PGN: {}.\n\
8. Giai đoạn: {} (Nước thứ {}).\n\
9. Engine Search depth={} nodes={} score={} cp.\n\
10. Đánh Giá {} Candidate Moves:\n{}\n\
11. Quyết Định Bestmove: '{}' đạt {} centipawn.\n\
</thought>",
                fen,
                red_pieces.len(), red_material,
                black_pieces.len(), black_material,
                red_material - black_material,
                king_score, center,
                pos.counts[1], pos.counts[2], pos.counts[8], pos.counts[9],
                advantages.join("; "),
                disadvantages.join("; "),
                positives_list.join("; "),
                negatives_list.join("; "),
                pgn,
                phase, index + 1,
                depth, nodes, score,
                candidates_json.len().min(top.max(1)),
                candidates_json.iter().enumerate()
                    .map(|(i, c)| format!("   - Candidate {}: {}", i + 1, c))
                    .collect::<Vec<String>>()
                    .join("\n"),
                encoded, score,
            );

            // Tạo assistant output JSON JRCP 2.0 đầy đủ
            let assistant = format!(
                "{{\"thought\": {:?}, \"matrix_analysis\": {{\"red_pieces_count\": {}, \"black_pieces_count\": {}, \"king_safety_score\": {}, \"center_file_control\": {:?}}}, \"risk_assessment\": {{\"advantages\": {}, \"disadvantages\": {}, \"positives\": {}, \"negatives\": {}}}, \"candidates\": [{}], \"bestmove\": {:?}, \"centipawn_eval\": {}}}",
                thought,
                red_pieces.len(),
                black_pieces.len(),
                king_score,
                center,
                array(&advantages),
                array(&disadvantages),
                array(&positives_list),
                array(&negatives_list),
                candidates_json.join(", "),
                encoded,
                score,
            );

            // User prompt đa chiều
            let user = format!(
                "Trạng thái bàn cờ tướng hiện tại (Biểu diễn đa chiều: Ma trận 2D, Chuỗi FEN chuẩn, và Lịch sử nước đi PGN):\n\n\
1. Ma Trận Bàn Cờ 2D (9x10):\n{}\n\n\
2. Chuỗi Chuẩn FEN (Forsyth-Edwards Notation):\n{}\n\n\
3. Lịch Sử Nước Đi PGN (Move History):\n{}\n\n\
Đến lượt {} đi. Hãy suy nghĩ sâu sắc trong thẻ <thought> và trả về JRCP 2.0 Structured Output JSON:",
                matrix, fen, pgn, turn
            );

            let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

            // === CẢI TIẾN #9: Conversation format thuần nhất JRCP 2.0 ===
            let sample = format!(
                "{{\"messages\": [{{\"role\": \"system\", \"content\": {:?}}}, {{\"role\": \"user\", \"content\": {:?}}}, {{\"role\": \"assistant\", \"content\": {:?}}}], \"move\": {:?}, \"eval\": {}, \"outcome\": {:?}, \"phase\": {:?}, \"depth\": {}, \"nodes\": {}, \"stamp\": {}}}",
                SYSTEM, user, assistant, encoded, score, move_outcome, phase, depth, nodes, stamp
            );

            samples.push(sample);
            history.push(encoded.clone());
            pos.apply(mv.from, mv.to);
        }

        if idx % 50 == 0 || idx == count {
            println!(" -> Tiến độ: {}/{} ván ({} mẫu)", idx, count, samples.len());
        }
    }

    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    std::fs::create_dir_all("data").ok();

    // Ghi dạng JSONL (1 JSON/dòng — tối ưu cho huấn luyện)
    let path = format!("data/jrcp2_elite_{}.jsonl", stamp);
    let mut file = File::create(&path).expect("Tạo file thất bại");
    for sample in samples.iter() {
        file.write_all(sample.as_bytes()).expect("Ghi dòng thất bại");
        file.write_all(b"\n").expect("Ghi newline thất bại");
    }

    println!("============================================================");
    println!("✅ JRCP 2.0 ELITE MINER: {} MẪU ĐẲNG CẤP!", samples.len());
    println!("💾 Tệp JSONL: {}", path);
    println!("============================================================");
}
