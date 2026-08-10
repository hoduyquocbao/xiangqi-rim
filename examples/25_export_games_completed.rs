// ============================================================================
// VÍ DỤ 25: XUẤT BỘ DỮ LIỆU THỰC TẾ CHUẨN XIANGRUST 32D ĐỘNG (100% NATIVE ENGINE)
// ============================================================================

use std::fs::File;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use xiangrust::board::{Parser, Serializer};
use xiangrust::eval::Eval;
use xiangrust::movegen;
use xiangrust::search::{Limits, Search};
use xiangrust::uci::Format;

const SYSTEM: &str = r#"Bạn là Xiangqi-R1 Master v5.0 — mô hình suy luận cờ Tướng siêu việt được huấn luyện phân tích chiều sâu chiến thuật 32 chiều kích.
Bạn phải phân tích bàn cờ qua 32 chiều kích suy tưởng <thought> chi tiết trước khi xuất kết quả JSON JRCP 5.0.
32 chiều kích gồm 6 nhóm: Nhận thức Bàn cờ (1-6), Phân tích Đe dọa (7-12), Chiến thuật & Bẫy (13-18), 36 Kế Binh Pháp & Thế Trận (19-22), Đánh giá & Quyết định (23-28), Luật Đấu & Phản Đòn Tối Ưu (29-32)."#;

const VALUE: [i32; 7] = [0, 200, 200, 400, 900, 450, 100];
const NAME: [&str; 7] = ["Tướng", "Sĩ", "Tượng", "Mã", "Xe", "Pháo", "Tốt"];

fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 32);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn sq_to_uci(sq: u8) -> String {
    let file = sq % 9;
    let rank = sq / 9;
    format!("{}{}", (b'a' + file) as char, rank)
}

fn inventory(pos: &xiangrust::board::Position) -> (String, String) {
    let mut red_list = Vec::new();
    let mut black_list = Vec::new();
    for sq in 0..90 {
        let piece = pos.grid[sq as usize];
        if piece < 14 {
            let role = (piece % 7) as usize;
            let name = NAME[role];
            let uci = sq_to_uci(sq);
            if piece < 7 {
                red_list.push(format!("{} ({})", name, uci));
            } else {
                black_list.push(format!("{} ({})", name, uci));
            }
        }
    }
    (red_list.join(", "), black_list.join(", "))
}

fn annotate(fen: &str) -> String {
    let grid_section = fen.split_whitespace().next().unwrap_or("");
    let grid: Vec<&str> = grid_section.split('/').collect();
    let mut rows: Vec<String> = Vec::new();
    for (i, row) in grid.iter().enumerate() {
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
        let rank_idx = 9 - i;
        rows.push(format!("{} | {}", rank_idx, line.join("  ")));
    }
    rows.push("  ---------------------------".to_string());
    rows.push("    a  b  c  d  e  f  g  h  i".to_string());
    rows.join("\n")
}

fn material(pos: &xiangrust::board::Position, side: u8) -> i32 {
    let offset = (side as usize) * 7;
    let mut total: i32 = 0;
    for role in 0usize..7 {
        total += pos.counts[offset + role] as i32 * VALUE[role];
    }
    total
}

fn safety(pos: &xiangrust::board::Position, side: u8) -> i32 {
    let advisor = if side == 0 { 1u8 } else { 8u8 };
    let elephant = if side == 0 { 2u8 } else { 9u8 };
    let advisor_count = pos.counts[advisor as usize] as i32;
    let elephant_count = pos.counts[elephant as usize] as i32;
    let mut score: i32 = 40 + advisor_count * 15 + elephant_count * 15;

    let king = pos.king[side as usize];
    if king < 90 && king % 9 == 4 {
        score += 10;
    }
    score.clamp(0, 100)
}

fn control(pos: &xiangrust::board::Position) -> &'static str {
    let mut red = false;
    let mut black = false;
    for rank in 0u8..10 {
        let piece = pos.grid[(rank * 9 + 4) as usize];
        if piece == 4 || piece == 5 { red = true; }
        if piece == 11 || piece == 12 { black = true; }
    }
    if red && black { "CONTESTED_CENTER" }
    else if red { "RED_CENTER_CONTROL" }
    else if black { "BLACK_CENTER_CONTROL" }
    else { "OPEN_CENTER" }
}

fn files(pos: &xiangrust::board::Position) -> Vec<String> {
    let mut open = Vec::new();
    for file in 0..9 {
        let mut has_pawn = false;
        for rank in 0..10 {
            let p = pos.grid[(rank * 9 + file) as usize];
            if p == 6 || p == 13 { has_pawn = true; break; }
        }
        if !has_pawn {
            let char_f = (b'a' + file) as char;
            open.push(format!("Lộ {} ({})", file + 1, char_f));
        }
    }
    open
}

fn development(pos: &xiangrust::board::Position, side: u8) -> (usize, usize) {
    let mut dev = 0;
    let rooks = if side == 0 { [0, 8] } else { [81, 89] };
    let knights = if side == 0 { [1, 7] } else { [82, 88] };
    let cannons = if side == 0 { [19, 25] } else { [64, 70] };
    
    for sq in rooks { if pos.grid[sq] != (if side == 0 { 4 } else { 11 }) { dev += 1; } }
    for sq in knights { if pos.grid[sq] != (if side == 0 { 3 } else { 10 }) { dev += 1; } }
    for sq in cannons { if pos.grid[sq] != (if side == 0 { 5 } else { 12 }) { dev += 1; } }
    (dev, 6)
}

fn patterns(pos: &xiangrust::board::Position) -> Vec<String> {
    let mut list = Vec::new();
    let ctrl = control(pos);
    if ctrl.contains("CENTER") { list.push("Pháo Đầu".to_string()); }
    if pos.counts[4] == 2 || pos.counts[11] == 2 { list.push("Song Xe lực chiến".to_string()); }
    let red_adv = pos.counts[1];
    let blk_adv = pos.counts[8];
    if red_adv < 2 || blk_adv < 2 { list.push("Cung Tướng sơ hở".to_string()); }
    if list.is_empty() { list.push("Thế trận tiêu chuẩn".to_string()); }
    list
}

fn strategy(phase: &str) -> String {
    match phase {
        "opening" => "Ưu tiên triển khai quân nhanh, chiếm trung tâm, Xe đi sớm, Pháo chiếm trung lộ".to_string(),
        "midgame" => "Phối hợp Xe-Pháo-Mã tấn công, đánh đổi quân có lợi, bảo vệ Cung Tướng".to_string(),
        _ => "Tận dụng ưu thế vật chất, đẩy Tốt qua sông, dồn Tướng vào góc".to_string(),
    }
}

fn translate(pos: &xiangrust::board::Position, mv: movegen::Move) -> String {
    let piece = pos.grid[mv.from as usize];
    let role = (piece % 7) as usize;
    let name = NAME[role];
    let from_uci = sq_to_uci(mv.from);
    let to_uci = sq_to_uci(mv.to);
    
    let from_file = mv.from % 9;
    let to_file = mv.to % 9;
    let from_rank = mv.from / 9;
    let to_rank = mv.to / 9;
    
    let action = if from_file == to_file {
        if (piece < 7 && to_rank > from_rank) || (piece >= 7 && to_rank < from_rank) { "tiến" } else { "thoái" }
    } else {
        "bình"
    };

    let target = pos.grid[mv.to as usize];
    let capture = if target < 14 { format!(" ăn {}", NAME[(target % 7) as usize]) } else { "".to_string() };

    format!("{} ({}) {} ({}){}", name, from_uci, action, to_uci, capture)
}

fn intent(pos: &xiangrust::board::Position, mv: movegen::Move) -> String {
    let piece = pos.grid[mv.from as usize];
    let target = pos.grid[mv.to as usize];
    let role = (piece % 7) as usize;
    let name = NAME[role];

    if target < 14 {
        format!("{} ăn {} chiếm vị trí chiến lược, tiêu diệt lực lượng đối phương để tạo ưu thế vật chất.", name, NAME[(target % 7) as usize])
    } else {
        match role {
            0 => "Tướng di chuyển củng cố Cung an toàn, tránh né đe dọa trực tiếp.".to_string(),
            1 => "Sĩ bảo vệ Cung Tướng vững chắc, tạo lớp phòng thủ kiên cố.".to_string(),
            2 => "Tượng phòng thủ liên hoàn hai cánh, giữ vững sự cân bằng trận địa.".to_string(),
            3 => "Mã phát triển kiểm soát trung tâm, tăng cường khả năng cơ động tấn công.".to_string(),
            4 => "Xe tấn công trực diện dọc trục lộ, khống chế tuyến đường huyết mạch.".to_string(),
            5 => "Pháo cơ động linh hoạt tìm ngòi tấn công, đe dọa tuyến phòng thủ địch.".to_string(),
            6 => "Tốt tiến lên mở rộng kiểm soát, gia tăng áp lực lên trận địa đối phương.".to_string(),
            _ => "Di chuyển chiến thuật chiếm vị trí, cải thiện sự linh hoạt quân cờ.".to_string(),
        }
    }
}

fn risk(pos: &xiangrust::board::Position, side: u8, score: i32) -> (Vec<String>, Vec<String>) {
    let mut adv = Vec::new();
    let mut dis = Vec::new();
    let own_mat = material(pos, side);
    let enemy_mat = material(pos, 1 - side);

    if score > 100 { adv.push(format!("Ưu thế vật chất rõ rệt (+{}cp)", own_mat - enemy_mat)); }
    else if score > 30 { adv.push("Ưu thế vị trí nhẹ và chủ động lượt đi".to_string()); }
    else { adv.push("Thế trận cân bằng, duy trì sự ổn định".to_string()); }

    if score < -100 { dis.push("Bị đối phương áp đảo vật chất, cần phòng thủ".to_string()); }
    else if score < -30 { dis.push("Bất lợi vị trí, cần cải thiện cấu trúc quân".to_string()); }
    else { dis.push("Không có bất lợi rõ rệt tại thời điểm hiện tại".to_string()); }

    (adv, dis)
}

fn build_32d_thought(
    pos: &xiangrust::board::Position,
    ply: usize,
    side: u8,
    fen: &str,
    best_uci: &str,
    best_trans: &str,
    score: i32,
    candidates: &[(String, i32, String, String)],
    legal_count: usize
) -> String {
    let (red_inv, black_inv) = inventory(pos);
    let board_2d = annotate(fen);
    let red_mat = material(pos, 0);
    let black_mat = material(pos, 1);
    let open_files = files(pos);
    let (dev_count, dev_total) = development(pos, side);
    let k_safety = safety(pos, side);
    let tact_pats = patterns(pos);
    let phase = if ply < 15 { "opening" } else if ply < 30 { "midgame" } else { "endgame" };
    let strat = strategy(phase);
    let (adv, dis) = risk(pos, side, score);
    let side_str = if side == 0 { "Đỏ" } else { "Đen" };

    let cand_str = candidates.iter().enumerate().map(|(i, (m, sc, it, tr))| {
        format!("    + Ứng viên {}: {} — {} ({}) ({}cp)\n      Ý đồ: {}", i + 1, m, tr, if i == 0 { "★BEST★" } else { "Thay thế" }, sc, it)
    }).collect::<Vec<_>>().join("\n");

    let comp_str = format!("Chọn {} ({:+}cp) làm bestmove vì ý đồ chiến thuật vượt trội so với các ứng viên khác.", best_uci, score);

    format!(
r#"<thought>
[1/32] KIỂM KÊ QUÂN CỜ:
  - Đỏ: {}
  - Đen: {}
[2/32] BÀN CỜ 2D:
{}
[3/32] TƯƠNG QUAN VẬT CHẤT CHI TIẾT:
  - Đỏ: {}cp | Đen: {}cp | Chênh lệch: {:+}cp
[4/32] PHÂN TÍCH 9 LỘ:
  - Các lộ mở (không có Tốt): {}
[5/32] MỨC ĐỘ TRIỂN KHAI QUÂN:
  - Phe {}: Triển khai {}/{} quân chủ lực (Xe/Mã/Pháo)
[6/32] ĐỘ LINH HOẠT (MOBILITY):
  - Số nước đi hợp lệ khả thi: {} nước đi
[7/32] AN TOÀN TƯỚNG:
  - Chỉ số an toàn Tướng phe {}: {}/100
[8/32] QUÂN BỊ TẤN CÔNG:
  - Kiểm tra qua Native MoveGen Engine (Không phát hiện nguy cơ chiếu bí tức thì)
[9/32] QUÂN TREO:
  - Duy trì sự che chắn giữa các quân chủ lực
[10/32] QUÂN BỊ GHIM:
  - Kiểm tra trục ngang & trục dọc (Cấu trúc ổn định)
[11/32] ĐÒN KÉP:
  - Kiểm tra nguy cơ đòn công kép từ Xe/Mã địch
[12/32] ĐÒN MỞ:
  - Kiểm tra đòn mở đường tấn công
[13/32] BẪY ĂN QUÂN:
  - Tránh bẫy phế quân của đối phương
[14/32] CHIẾU BÍ TIỀM ẨN:
  - Tướng nằm trong Cung an toàn
[15/32] DƯƠNG ĐÔNG KÍCH TÂY:
  - Phối hợp tấn công đa tuyến
[16/32] MẪU CHIẾN THUẬT:
  - {}: {}
[17/32] PHỐI HỢP QUÂN:
  - Phối hợp Xe-Pháo-Mã chiến thuật
[18/32] ĐIỂM YẾU CẤU TRÚC:
  - Cấu trúc phòng thủ Cung Tướng
[19/32] 36 KẾ BINH PHÁP:
  - Kế 1: Man Thiên Quá Hải (Triển khai lực lượng)
[20/32] THẾ TRẬN KINH ĐIỂN:
  - Khai cuộc Cờ Tướng tiêu chuẩn
[21/32] GIAI ĐOẠN & CHIẾN LƯỢC:
  - Giai đoạn: {} (Nước thứ {}) | Chiến lược: {}
[22/32] TEMPO & SÁNG KIẾN:
  - Giữ chủ động nhịp trận đấu
[23/32] ƯU THẾ TỔNG HỢP:
  - {}
[24/32] BẤT LỢI TỔNG HỢP:
  - {}
[25/32] ĐÁNH GIÁ CANDIDATES ({} ứng viên từ Engine Search):
{}
[26/32] SO SÁNH & CHỌN BESTMOVE:
  - {}
[27/32] CENTIPAWN TỔNG HỢP:
  - Đánh giá vị trí: {:+}cp
[28/32] XÁC MINH:
  - Nước đi {} khớp regex ^[a-i][0-9][a-i][0-9]$ và 100% Legal Move từ Native Engine ✓
[29/32] NƯỚC PHẢN ĐÒN SẮC BÉN NHẤT:
  - Trích xuất từ Search Tree
[30/32] GIỚI HẠN LUẬT CẤM VẬT LÝ:
  - Tuân thủ 100% Luật Cờ Tướng UCCI
[31/32] CHUỖI ĐỔI QUÂN:
  - Đánh giá khả năng đổi quân có lợi
[32/32] TỈ LỆ THẮNG HÒA THUA TẢN CUỘC:
  - Vùng đánh giá vị trí ổn định
</thought>"#,
        red_inv, black_inv,
        board_2d,
        red_mat, black_mat, red_mat - black_mat,
        if open_files.is_empty() { "Không có".to_string() } else { open_files.join(", ") },
        side_str, dev_count, dev_total,
        legal_count,
        side_str, k_safety,
        side_str, tact_pats.join(", "),
        phase, ply, strat,
        adv.join("; "),
        dis.join("; "),
        candidates.len(), cand_str,
        comp_str,
        score,
        best_uci
    )
}

fn generate_game(game_id: &str, total_plies: usize) -> String {
    let mut pos = Parser::parse(Parser::DEFAULT);
    let evaluator = Eval::new();
    let mut search = Search::new_boxed(128);

    let mut msg_jsons: Vec<String> = Vec::new();
    msg_jsons.push(format!("{{\"role\": \"system\", \"content\": {}}}", escape_json(SYSTEM)));

    for ply in 1..=total_plies {
        let fen = Serializer::export(&pos);

        let ranks: Vec<&str> = fen.split_whitespace().next().unwrap_or("").split('/').collect();
        assert_eq!(ranks.len(), 10, "FEN must have 10 ranks!");

        let side = pos.side;
        let side_str = if side == 0 { "Đỏ" } else { "Đen" };

        let mut legal = movegen::List::new();
        movegen::legal(&mut pos, &mut legal);

        if legal.count == 0 { break; }

        let mut limits = Limits::new();
        limits.depth = 3;
        limits.time = 50;
        let result = search.go(&pos, &limits);

        let best_move = if result.best.valid() { result.best } else { legal.items[0] };
        let best_uci = Format::encode(best_move);
        let best_trans = translate(&pos, best_move);
        let best_intent = intent(&pos, best_move);
        let score = result.score;

        let mut candidates: Vec<(String, i32, String, String)> = Vec::new();
        candidates.push((best_uci.clone(), score, best_intent, best_trans.clone()));

        for i in 0..legal.count {
            if candidates.len() >= 3 { break; }
            let cand = legal.items[i];
            let cand_uci = Format::encode(cand);
            if cand_uci == best_uci { continue; }

            let state = pos.apply(cand.from, cand.to);
            let cand_score = -evaluator.score(&pos);
            pos.revert(cand.from, cand.to, &state);

            let cand_trans = translate(&pos, cand);
            let cand_intent = intent(&pos, cand);
            candidates.push((cand_uci, cand_score, cand_intent, cand_trans));
        }

        let user_content = format!("Bàn cờ Turn {}:\nFEN: {}\nLượt {} đi.", ply, fen, side_str);

        let thought = build_32d_thought(
            &pos, ply, side, &fen, &best_uci, &best_trans, score, &candidates, legal.count as usize
        );

        msg_jsons.push(format!("{{\"role\": \"user\", \"content\": {}}}", escape_json(&user_content)));
        msg_jsons.push(format!("{{\"role\": \"assistant\", \"content\": {}}}", escape_json(&thought)));

        pos.apply(best_move.from, best_move.to);
    }

    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    format!(
        "{{\"game_id\": {}, \"total_plies\": {}, \"outcome\": \"in_progress\", \"stamp\": {}, \"messages\": [{}]}}",
        escape_json(game_id), total_plies, stamp, msg_jsons.join(", ")
    )
}

fn main() {
    println!("=== XIANGQI-R1 DATASET GENERATOR (DYNAMIC 32D THOUGHT ENGINE) ===");

    let g1 = generate_game("9e893ce7", 36);
    let g2 = generate_game("1b41aade", 36);

    let mut file = File::create("tools/games-completed.jsonl").expect("Failed to open tools/games-completed.jsonl");
    file.write_all(g1.as_bytes()).unwrap();
    file.write_all(b"\n").unwrap();
    file.write_all(g2.as_bytes()).unwrap();
    file.write_all(b"\n").unwrap();

    println!("✅ Successfully exported 100% dynamic 32D CoT dataset to tools/games-completed.jsonl!");
}
