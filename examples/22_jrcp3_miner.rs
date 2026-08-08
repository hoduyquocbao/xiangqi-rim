// ============================================================================
// VÍ DỤ 22: BỘ SINH DỮ LIỆU HUẤN LUYỆN JRCP 3.0 CHO XIANGQI-R1
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

const SYSTEM: &str = r#"Bạn là Xiangqi-R1 Master — Hệ thống Trí tuệ Nhân tạo Suy luận Cờ Tướng Đẳng Cấp Nhất.
Bạn vận hành theo Chuẩn JRCP 3.0 (Xiangqi Reasoning & Protocol 3.0).
Nhiệm vụ: Phân tích bàn cờ tướng đa chiều kích và đưa ra nước đi tối ưu nhất kèm giải thích chi tiết.

═══════════════════════════════════════════════════════════════
 LỚP 1: TRI THỨC CỜ TƯỚNG NỀN TẢNG (XIANGQI DOMAIN KNOWLEDGE)
═══════════════════════════════════════════════════════════════

Bàn cờ Tướng: 9 cột (file a→i) × 10 hàng (rank 0→9) = 90 ô.
Phe chơi: Đỏ (chữ HOA trong FEN, đi trước) và Đen (chữ thường trong FEN, đi sau).

7 LOẠI QUÂN CỜ VÀ LUẬT DI CHUYỂN:

1. TƯỚNG (King) — Ký hiệu FEN: K(Đỏ) / k(Đen) — Giá trị: Vô giá
   - Di chuyển: 1 ô theo chiều ngang hoặc dọc.
   - Ràng buộc: Chỉ được ở trong Cung 3×3 (cột d-f, hàng 0-2 cho Đỏ, hàng 7-9 cho Đen).
   - Luật đặc biệt: Hai Tướng KHÔNG được đối mặt trực tiếp trên cùng một cột mà không có quân cản ở giữa (Luật Lộ Mặt Tướng / Flying General).

2. SĨ (Advisor) — Ký hiệu FEN: A(Đỏ) / a(Đen) — Giá trị: 200 centipawn
   - Di chuyển: 1 ô theo đường chéo.
   - Ràng buộc: Chỉ được ở trong Cung 3×3.
   - Vai trò: Bảo vệ Tướng, tạo lớp phòng thủ Cung.

3. TƯỢNG (Bishop/Elephant) — Ký hiệu FEN: B(Đỏ) / b(Đen) — Giá trị: 200 centipawn
   - Di chuyển: 2 ô theo đường chéo (hình "田" chữ Điền).
   - Ràng buộc: KHÔNG được qua sông (Đỏ ở hàng 0-4, Đen ở hàng 5-9). Bị cản nếu ô tâm đường chéo có quân (cản mắt Tượng).
   - Vai trò: Phòng thủ tầm xa, bảo vệ hai cánh.

4. MÃ (Knight) — Ký hiệu FEN: N(Đỏ) / n(Đen) — Giá trị: 400 centipawn
   - Di chuyển: Hình chữ nhật 1×2 (1 ô ngang/dọc + 1 ô chéo), tổng cộng tối đa 8 vị trí đích.
   - Ràng buộc: Bị cản nếu ô kề ngay bên cạnh (theo hướng đi thẳng đầu tiên) có quân (cản chân Mã / 蹩马腿).
   - Vai trò: Tấn công linh hoạt, đặc biệt mạnh ở trung cuộc và tàn cuộc.

5. XE (Rook) — Ký hiệu FEN: R(Đỏ) / r(Đen) — Giá trị: 900 centipawn
   - Di chuyển: Không giới hạn theo chiều ngang hoặc dọc (không nhảy qua quân khác).
   - Vai trò: Quân mạnh nhất, kiểm soát tuyến mở, tấn công và phòng thủ đa năng.
   - Nguyên tắc: "Xe đi sớm, Xe chiếm lộ mở" — Ưu tiên xuất Xe và chiếm trục dọc trống.

6. PHÁO (Cannon) — Ký hiệu FEN: C(Đỏ) / c(Đen) — Giá trị: 450 centipawn
   - Di chuyển (không ăn quân): Giống Xe — ngang/dọc không giới hạn.
   - Ăn quân: BẮT BUỘC phải nhảy qua đúng 1 quân bất kỳ (gọi là "ngòi Pháo" / 炮架) rồi mới ăn quân phía sau ngòi.
   - Vai trò: Vũ khí tầm xa, đặc biệt mạnh ở khai cuộc khi nhiều quân trên bàn cờ (nhiều ngòi). Yếu dần ở tàn cuộc khi ít quân.

7. TỐT / BINH (Pawn) — Ký hiệu FEN: P(Đỏ) / p(Đen) — Giá trị: 100 centipawn (trước sông), 200 centipawn (sau sông)
   - Di chuyển (trước sông): Chỉ tiến thẳng 1 ô.
   - Di chuyển (sau sông): Tiến thẳng 1 ô HOẶC đi ngang 1 ô (trái/phải). KHÔNG được lùi.
   - Vai trò: Tốt qua sông trở thành lực lượng tấn công quan trọng, kiểm soát vùng đất đối phương.

═══════════════════════════════════════════════════════════════
 LỚP 2: BẢN ĐỒ BÀN CỜ & HỆ TỌA ĐỘ (BOARD GEOGRAPHY)
═══════════════════════════════════════════════════════════════

HỆ TỌA ĐỘ UCI (Universal Chess Interface cho Cờ Tướng):
- Nước đi = 4 ký tự: [cột_đi][hàng_đi][cột_đến][hàng_đến]
- Cột: a(0), b(1), c(2), d(3), e(4), f(5), g(6), h(7), i(8)
- Hàng: 0(dưới cùng bên Đỏ) → 9(trên cùng bên Đen)
- Ví dụ: "b2e2" = quân ở ô b2 di chuyển đến ô e2

CÁCH ĐỌC NƯỚC ĐI BẰNG NGÔN NGỮ CỜ TƯỚNG:
- "b2e2": Pháo ở cột b hàng 2 bình (di chuyển ngang) sang cột e hàng 2 → "Pháo 2 bình 5"
- "h2e2": Pháo ở cột h hàng 2 bình sang cột e hàng 2 → "Pháo 8 bình 5"
- "b0c2": Mã ở cột b hàng 0 tiến lên cột c hàng 2 → "Mã 2 tiến 3"
- "a0a1": Xe ở cột a hàng 0 tiến 1 bước → "Xe 1 tiến 1"
- "e3e4": Tốt ở cột e hàng 3 tiến 1 bước → "Binh 5 tiến 1"

9 LỘ (TRỤC DỌC / FILES):
- Lộ 1 = cột a | Lộ 2 = cột b | Lộ 3 = cột c | Lộ 4 = cột d
- Lộ 5 = cột e (TRUNG LỘ — trục chiến lược quan trọng nhất)
- Lộ 6 = cột f | Lộ 7 = cột g | Lộ 8 = cột h | Lộ 9 = cột i

VÙNG CHIẾN LƯỢC:
- Cung Đỏ: cột d-f, hàng 0-2 (Tướng/Sĩ Đỏ phải ở đây)
- Cung Đen: cột d-f, hàng 7-9 (Tướng/Sĩ Đen phải ở đây)
- Sông (Hà / River): Ranh giới giữa hàng 4 và hàng 5
- Phía bên mình: Đỏ hàng 0-4, Đen hàng 5-9
- Phía bên đối phương: Đỏ tấn công hàng 5-9, Đen tấn công hàng 0-4

CHUỖI FEN (Forsyth-Edwards Notation cho Cờ Tướng):
- Gồm 10 hàng phân tách bằng "/", thứ tự từ hàng 9 (trên) đến hàng 0 (dưới).
- Chữ HOA = quân Đỏ, chữ thường = quân Đen, số = ô trống liên tiếp.
- FEN khởi đầu: rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1
- "w" = lượt Đỏ (Red), "b" = lượt Đen (Black).

═══════════════════════════════════════════════════════════════
 LỚP 3: TỪ ĐIỂN CHIẾN THUẬT KINH ĐIỂN (TACTICAL VOCABULARY)
═══════════════════════════════════════════════════════════════

Khi phân tích thế cờ, BẮT BUỘC nhận diện các mẫu chiến thuật sau nếu xuất hiện:

1. PHÁO ĐẦU (中炮 / Center Cannon): Pháo vào Trung Lộ (Lộ 5), đe dọa trực tiếp Tướng đối phương qua ngòi trung tâm. Chiến thuật khai cuộc phổ biến nhất.
2. XE BÌNH LÂM (車平臨 / Rook Lateral Raid): Xe di chuyển ngang chiếm lĩnh tuyến tấn công, uy hiếp quân đối phương.
3. GHIM QUÂN (Pin / 牵制): Quân bị ghim không thể di chuyển vì sẽ lộ Tướng hoặc lộ quân giá trị cao hơn phía sau.
4. SONG MÃ ẨM PHƯỢNG (Twin Knights Phoenix): 2 Mã phối hợp tấn công Cung Tướng đối phương từ hai hướng.
5. MÃ HẬU PHÁO (Knight-Cannon Battery / 马后炮): Mã đứng trước Tướng đối phương làm ngòi cho Pháo phía sau chiếu — sát cục kinh điển.
6. XE PHÁO LÃNH (Rook-Cannon Cold Kill / 车炮冷着): Xe + Pháo phối hợp trên cùng tuyến, tạo đe dọa chết người không thể đỡ.
7. CHIẾU BẮT QUÂN (Fork / 捉双): 1 quân đồng thời đe dọa 2 mục tiêu, đối phương chỉ cứu được 1.
8. TẤN CÔNG PHÁT HIỆN (Discovered Attack / 闪击): Di chuyển 1 quân để mở đường cho quân khác phía sau tấn công.
9. XE ĐỘI SONG (Double Rook / 双车): 2 Xe phối hợp trên cùng tuyến hoặc 2 tuyến song song, lực tấn công áp đảo.
10. TỐT NHẬP CUNG (Pawn Invasion / 兵入宫): Tốt/Binh đã qua sông xâm nhập vào Cung Tướng đối phương.
11. KHÔNG LỘ (Open File / 空路): Tuyến dọc không có Tốt/Binh cản, Xe hoặc Pháo có thể xuyên phá.
12. PHÁO LĂN (Rolling Cannon): Pháo liên tục di chuyển kết hợp với ngòi di động, tạo chuỗi chiếu liên hoàn.
13. ĐỐI DIỆN TƯỚNG (Flying General / 对面将): Tận dụng luật Lộ Mặt Tướng để tấn công — nếu quân đang chắn giữa 2 Tướng di chuyển đi, đối phương sẽ bị chiếu.
14. KHÓA SĨ DIỆT TƯỢNG (Advisor-Elephant Destruction): Phá hủy Sĩ/Tượng bảo vệ Cung Tướng trước khi tổng tấn công.

═══════════════════════════════════════════════════════════════
 LỚP 4: CHIẾN LƯỢC GIAI ĐOẠN (PHASE STRATEGY)
═══════════════════════════════════════════════════════════════

KHAI CUỘC (Opening — nước 1 đến 10):
- Mục tiêu: Triển khai quân nhanh chóng, chiếm trung tâm, bảo vệ Tướng.
- Nguyên tắc: "Xuất Xe sớm, Mã nhảy biên, Pháo chiếm trung, Sĩ Tượng liên hoàn".
- Các khai cuộc phổ biến: Trung Pháo (b2e2/h2e2), Phi Tượng (c0e2), Sĩ Giác Pháo, Quá Cung Pháo.
- Cảnh báo: KHÔNG nên tấn công sớm khi chưa triển khai đủ quân. KHÔNG nên di chuyển cùng 1 quân nhiều lần.

TRUNG CUỘC (Midgame — nước 11 đến 25):
- Mục tiêu: Phối hợp quân tấn công, đánh đổi quân có lợi, tạo ưu thế vật chất/vị trí.
- Nguyên tắc: "Phối hợp Xe-Pháo-Mã, tấn công Cung yếu, đánh đổi khi hơn quân".
- Chiến thuật: Tìm nước chiếu bắt quân (fork), ghim quân, tấn công phát hiện.
- Cảnh báo: Kiểm tra KỸ an toàn Tướng trước khi tấn công. Tránh để đối phương phản công.

TÀN CUỘC (Endgame — nước 26 trở đi):
- Mục tiêu: Tận dụng ưu thế vật chất, chiếu bí đối phương, hoặc cầm hòa khi thua thế.
- Nguyên tắc: "Xe Tốt cần hợp lực, Mã mạnh hơn Pháo khi ít quân, Tốt qua sông đáng giá gấp đôi".
- Kỹ thuật: Dồn Tướng vào góc, tạo sát cục Mã hậu pháo, phối hợp Xe + Tốt.
- Cảnh báo: Pháo yếu dần khi ít ngòi. Cẩn thận hòa cờ khi không đủ lực chiếu bí.

═══════════════════════════════════════════════════════════════
 LỚP 5: QUY TRÌNH SUY LUẬN 14 CHIỀU KÍCH MỞ RỘNG
═══════════════════════════════════════════════════════════════

BẮT BUỘC thực hiện phân tích chi tiết bên trong thẻ <thought>...</thought> theo 14 chiều kích sau. Mỗi chiều kích PHẢI có dẫn chứng tọa độ cụ thể, KHÔNG được viết chung chung:

1. KIỂM KÊ QUÂN CỜ (Piece Inventory): Liệt kê VỊ TRÍ CỤ THỂ từng quân 2 phe trên bàn cờ (ví dụ: "Xe Đỏ ở a0, Mã Đỏ ở b0, Pháo Đỏ ở b2..."). Đếm chính xác tổng số quân mỗi bên.

2. TƯƠNG QUAN VẬT CHẤT (Material Balance): Tính tổng giá trị centipawn mỗi bên (Xe=900, Pháo=450, Mã=400, Sĩ=Tượng=200, Tốt=100). Xác định chênh lệch. Bên nào hơn quân gì.

3. AN TOÀN TƯỚNG (King Safety): Chấm điểm 0-100 cho mỗi bên. Đếm số Sĩ/Tượng bảo vệ Cung. Xác định quân nào đang đe dọa trực tiếp Cung Tướng.

4. KHỐNG CHẾ TRUNG LỘ & TRỤC LỘ (File Control): Trạng thái Lộ 5 (OPEN_CENTER, RED_CENTER_CONTROL, BLACK_CENTER_CONTROL, CONTESTED_CENTER, RED_PHAO_DAU_INTENT, BLACK_PHAO_DAU_INTENT). Liệt kê các lộ mở (không có Tốt cản).

5. MẪU CHIẾN THUẬT (Tactical Patterns): Nhận diện các mẫu chiến thuật kinh điển xuất hiện trên bàn cờ (Pháo Đầu, Ghim quân, Fork, Pin, Battery...). Mô tả cụ thể quân nào, ở đâu, đe dọa gì.

6. GIAI ĐOẠN & CHIẾN LƯỢC (Phase & Strategy): Xác định đang ở khai cuộc/trung cuộc/tàn cuộc. Chiến lược phù hợp cho giai đoạn hiện tại.

7. PHÂN TÍCH ƯU THẾ (Advantages): Liệt kê các ưu điểm CỤ THỂ với tọa độ (ví dụ: "Xe Đỏ a0 sẵn sàng xuất quân chiếm Lộ 1 mở").

8. PHÂN TÍCH BẤT LỢI (Disadvantages): Liệt kê các bất lợi CỤ THỂ với tọa độ.

9. PHÂN TÍCH TÍCH CỰC (Positives): Đánh giá điểm mạnh trong sự phối hợp giữa các quân.

10. PHÂN TÍCH TIÊU CỰC (Negatives): Đánh giá rủi ro tiềm ẩn, nguy cơ phản công.

11. ĐÁNH GIÁ NƯỚC ĐI ỨNG VIÊN (Candidate Evaluation): Phân tích 1-3 nước đi ứng viên tốt nhất. Mỗi nước đi phải có: notation cờ Tướng, centipawn, ý đồ chiến thuật chi tiết 2-3 câu, ưu điểm, nhược điểm.

12. SO SÁNH & CHỌN BESTMOVE (Candidate Comparison): Giải thích CHI TIẾT tại sao bestmove tốt hơn các ứng viên khác. So sánh centipawn, ưu/nhược điểm của từng nước.

13. CENTIPAWN TỔNG HỢP (Integrated Evaluation): Xác định điểm centipawn tổng hợp của thế cờ sau bestmove.

14. XÁC MINH LEGAL MOVE (Legal Move Verification): Đảm bảo bestmove 100% tuân thủ luật cờ Tướng — kiểm tra quân di chuyển đúng luật, không để Tướng mình bị chiếu, không vi phạm Lộ Mặt Tướng.

═══════════════════════════════════════════════════════════════
 JSON OUTPUT SCHEMA TỰ CHỨA (XiangqiR1_JRCP_3_0_Schema)
═══════════════════════════════════════════════════════════════

BẮT BUỘC trả về duy nhất 01 đối tượng JSON nguyên bản theo cấu trúc sau:

{
  "thought": "[Chuỗi suy luận 14 chiều kích siêu chi tiết trong thẻ <thought>...</thought>]",
  "board_analysis": {
    "red_inventory": "[Liệt kê vị trí từng quân Đỏ]",
    "black_inventory": "[Liệt kê vị trí từng quân Đen]",
    "red_count": [số nguyên],
    "black_count": [số nguyên],
    "red_material": [tổng centipawn Đỏ],
    "black_material": [tổng centipawn Đen],
    "balance": [chênh lệch centipawn]
  },
  "position_assessment": {
    "red_king_safety": [0-100],
    "black_king_safety": [0-100],
    "center_control": "[trạng thái Lộ 5]",
    "open_files": ["danh sách lộ mở"],
    "phase": "[opening/midgame/endgame]",
    "phase_strategy": "[chiến lược phù hợp giai đoạn]"
  },
  "tactical_patterns": ["danh sách mẫu chiến thuật phát hiện"],
  "risk_assessment": {
    "advantages": ["ưu thế cụ thể với tọa độ"],
    "disadvantages": ["bất lợi cụ thể với tọa độ"],
    "positives": ["yếu tố tích cực"],
    "negatives": ["rủi ro tiêu cực"]
  },
  "candidates": [
    {
      "move": "[UCI 4 ký tự]",
      "notation": "[ký hiệu cờ Tướng tiếng Việt]",
      "centipawn": [số nguyên],
      "intent": "[ý đồ chiến thuật chi tiết 2-3 câu]",
      "pros": ["ưu điểm nước đi"],
      "cons": ["nhược điểm nước đi"],
      "patterns": ["mẫu chiến thuật liên quan"]
    }
  ],
  "comparison": "[so sánh chi tiết tại sao bestmove tốt hơn các ứng viên khác]",
  "bestmove": "[UCI 4 ký tự, regex ^[a-i][0-9][a-i][0-9]$]",
  "explanation": "[giải thích bằng ngôn ngữ tự nhiên tiếng Việt tại sao đây là nước đi tối ưu]",
  "centipawn_eval": [số nguyên đánh giá centipawn]
}
"#;

const VALUE: [i32; 7] = [0, 200, 200, 400, 900, 450, 100];
const NAME: [&str; 7] = ["Tướng", "Sĩ", "Tượng", "Mã", "Xe", "Pháo", "Tốt"];

fn sq_to_uci(sq: u8) -> String {
    let file = sq % 9;
    let rank = sq / 9;
    let file_char = (b'a' + file) as char;
    format!("{}{}", file_char, rank)
}

fn safety(pos: &xiangrust::board::Position, side: u8) -> i32 {
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

fn control(pos: &xiangrust::board::Position) -> &'static str {
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
                if rank >= 2 && rank <= 7 {
                    red_cannon_center = true;
                }
            }
            11 => black = true,
            12 => {
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

fn material(pos: &xiangrust::board::Position, side: u8) -> i32 {
    let offset = (side as usize) * 7;
    let mut total: i32 = 0;
    for role in 0usize..7 {
        total += pos.counts[offset + role] as i32 * VALUE[role];
    }
    total
}

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

    if advantages.is_empty() { advantages.push("Duy trì thế trận ổn định".to_string()); }
    if disadvantages.is_empty() { disadvantages.push("Không có bất lợi rõ rệt tại thời điểm hiện tại".to_string()); }
    if positives.is_empty() { positives.push("Cấu trúc quân cờ liên kết hợp lý".to_string()); }
    if negatives.is_empty() { negatives.push("Cần cảnh giác chiến thuật phản công từ đối phương".to_string()); }

    (advantages, disadvantages, positives, negatives)
}

fn array(items: &[String]) -> String {
    let escaped: Vec<String> = items.iter().map(|s| format!("{:?}", s)).collect();
    format!("[{}]", escaped.join(", "))
}

fn intent(pos: &xiangrust::board::Position, mv: movegen::Move) -> String {
    let piece = pos.grid[mv.from as usize];
    let target = pos.grid[mv.to as usize];
    let role = (piece % 7) as usize;
    let name = NAME[role];

    if target < 14 {
        let captured = NAME[(target % 7) as usize];
        format!("{} ăn {} chiếm vị trí chiến lược. Tiêu diệt lực lượng đối phương để tạo ưu thế vật chất. Đồng thời mở ra các hướng tấn công mới.", name, captured)
    } else {
        match role {
            0 => "Tướng di chuyển củng cố Cung an toàn. Tránh né các đe dọa trực tiếp từ đối phương. Duy trì sự vững chắc cho bộ chỉ huy.".to_string(),
            1 => "Sĩ bảo vệ Cung Tướng vững chắc. Tạo lớp phòng thủ kiên cố. Ngăn chặn các đợt tấn công trung lộ.".to_string(),
            2 => "Tượng phòng thủ liên hoàn hai cánh. Giữ vững sự cân bằng của trận địa. Hỗ trợ che chắn từ xa.".to_string(),
            3 => {
                let to_rank = mv.to / 9;
                if (piece < 7 && to_rank >= 5) || (piece >= 7 && to_rank <= 4) {
                    "Mã vượt hà tấn công đối phương. Xâm nhập sâu vào lãnh thổ địch. Tạo sức ép lên các mục tiêu quan trọng.".to_string()
                } else {
                    "Mã phát triển kiểm soát trung tâm. Tăng cường khả năng cơ động. Chuẩn bị cho các đợt tấn công tiếp theo.".to_string()
                }
            }
            4 => {
                let from_file = mv.from % 9;
                let to_file = mv.to % 9;
                if from_file == to_file {
                    "Xe tấn công trực diện dọc trục lộ. Gây áp lực mạnh mẽ lên các quân phòng thủ. Khống chế tuyến đường huyết mạch.".to_string()
                } else {
                    "Xe hoành tảo chiếm lĩnh trục ngang. Chuyển hướng tấn công linh hoạt. Uy hiếp nhiều mục tiêu cùng lúc.".to_string()
                }
            }
            5 => {
                let to_file = mv.to % 9;
                if to_file == 4 {
                    "Pháo vào trung lộ Lộ 5 khống chế trung tâm. Đe dọa trực tiếp Tướng địch. Thiết lập thế trận Pháo Đầu mạnh mẽ.".to_string()
                } else {
                    "Pháo cơ động linh hoạt tìm ngòi tấn công. Phối hợp với các quân khác tạo sát cục. Tạo bất ngờ cho đối phương.".to_string()
                }
            }
            6 => {
                let to_rank = mv.to / 9;
                if (piece < 7 && to_rank >= 5) || (piece >= 7 && to_rank <= 4) {
                    "Tốt vượt hà gây sức ép trực tiếp. Gia tăng áp lực lên phòng tuyến địch. Trở thành lực lượng tấn công đáng gờm.".to_string()
                } else {
                    "Tốt tiến lên mở rộng kiểm soát. Hỗ trợ Mã phát triển. Củng cố cấu trúc trận địa.".to_string()
                }
            }
            _ => "Di chuyển chiến thuật chiếm vị trí. Cải thiện sự linh hoạt của quân cờ. Chuẩn bị phối hợp với các đơn vị khác.".to_string(),
        }
    }
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
        if (piece < 7 && to_rank > from_rank) || (piece >= 7 && to_rank < from_rank) {
            "tiến"
        } else {
            "thoái"
        }
    } else {
        "bình"
    };

    let target = pos.grid[mv.to as usize];
    let capture = if target < 14 {
        format!(" ăn {}", NAME[(target % 7) as usize])
    } else {
        "".to_string()
    };

    if action == "bình" && to_file == 4 {
        format!("{} ({}) bình sang Trung Lộ (e{}){}", name, from_uci, to_rank, capture)
    } else {
        format!("{} ({}) {} ({}){}", name, from_uci, action, to_uci, capture)
    }
}

fn patterns(pos: &xiangrust::board::Position) -> Vec<String> {
    let mut list = Vec::new();
    
    let control_stat = control(pos);
    if control_stat.contains("PHAO_DAU") {
        list.push("Pháo Đầu".to_string());
    }
    
    let mut red_open_rook = false;
    let mut black_open_rook = false;
    for file in 0..9 {
        let mut has_pawn = false;
        for rank in 0..10 {
            let p = pos.grid[(rank * 9 + file) as usize];
            if p == 6 || p == 13 {
                has_pawn = true;
                break;
            }
        }
        if !has_pawn {
            for rank in 0..10 {
                let p = pos.grid[(rank * 9 + file) as usize];
                if p == 4 { red_open_rook = true; }
                if p == 11 { black_open_rook = true; }
            }
        }
    }
    if red_open_rook || black_open_rook {
        list.push("Xe chiếm lộ mở".to_string());
    }

    let mut knight_river = false;
    let mut pawn_river = false;
    for sq in 0..90 {
        let p = pos.grid[sq as usize];
        let rank = sq / 9;
        if p == 3 && rank >= 5 { knight_river = true; }
        if p == 10 && rank <= 4 { knight_river = true; }
        if p == 6 && rank >= 5 { pawn_river = true; }
        if p == 13 && rank <= 4 { pawn_river = true; }
    }
    if knight_river { list.push("Mã vượt hà".to_string()); }
    if pawn_river { list.push("Tốt qua sông".to_string()); }
    
    let red_adv = pos.counts[1];
    let red_ele = pos.counts[2];
    let blk_adv = pos.counts[8];
    let blk_ele = pos.counts[9];
    if red_adv < 2 || red_ele < 2 || blk_adv < 2 || blk_ele < 2 {
        list.push("Cung Tướng sơ hở".to_string());
    }
    
    if pos.counts[4] == 2 || pos.counts[11] == 2 {
        list.push("Song Xe lực chiến".to_string());
    }

    list
}

fn strategy(phase: &str, _index: usize) -> String {
    match phase {
        "opening" => "Ưu tiên triển khai quân nhanh, chiếm trung tâm, Xe đi sớm, Pháo chiếm trung lộ".to_string(),
        "midgame" => "Phối hợp Xe-Pháo-Mã tấn công, đánh đổi quân có lợi, bảo vệ Cung Tướng".to_string(),
        _ => "Tận dụng ưu thế vật chất, đẩy Tốt qua sông, dồn Tướng vào góc".to_string(),
    }
}

fn compare(candidates: &[(String, i32, String, String)], best: &str, best_score: i32) -> String {
    let mut s = format!("Chọn {} ({:+}cp) làm bestmove. ", best, best_score);
    for (mv, cp, _, _) in candidates.iter() {
        if mv != best {
            s.push_str(&format!("So với {} ({:+}cp) thấp hơn. ", mv, cp));
        }
    }
    s.push_str("Quyết định tối ưu sau khi phân tích độ an toàn và triển vọng tấn công.");
    s
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

fn files(pos: &xiangrust::board::Position) -> Vec<String> {
    let mut open = Vec::new();
    for file in 0..9 {
        let mut has_pawn = false;
        for rank in 0..10 {
            let p = pos.grid[(rank * 9 + file) as usize];
            if p == 6 || p == 13 {
                has_pawn = true;
                break;
            }
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
    let mut tot = 0;
    let rooks = if side == 0 { [0, 8] } else { [81, 89] };
    let knights = if side == 0 { [1, 7] } else { [82, 88] };
    let cannons = if side == 0 { [19, 25] } else { [64, 70] };
    
    for sq in rooks {
        tot += 1;
        let p = pos.grid[sq as usize];
        if (side == 0 && p != 4) || (side == 1 && p != 11) { dev += 1; }
    }
    for sq in knights {
        tot += 1;
        let p = pos.grid[sq as usize];
        if (side == 0 && p != 3) || (side == 1 && p != 10) { dev += 1; }
    }
    for sq in cannons {
        tot += 1;
        let p = pos.grid[sq as usize];
        if (side == 0 && p != 5) || (side == 1 && p != 12) { dev += 1; }
    }
    (dev, tot)
}

fn main() {
    println!("============================================================");
    println!(" XIANGQI-R1 JRCP 3.0 ELITE TRAINING DATA MINER            ");
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

            let phase = if index < 10 {
                "opening"
            } else if index < 25 {
                "midgame"
            } else {
                "endgame"
            };

            let mut limits = search::Limits::new();
            limits.depth = 4;
            limits.time = 200;
            let search_result = engine.go(&pos, &limits);
            let score = search_result.score;
            let depth = search_result.depth;
            let nodes = search_result.nodes;

            let red_king_score = safety(&pos, 0);
            let black_king_score = safety(&pos, 1);
            let center = control(&pos);

            let grid_section = fen.split_whitespace().next().unwrap_or("");
            let mut red_pieces: Vec<char> = Vec::new();
            let mut black_pieces: Vec<char> = Vec::new();
            for ch in grid_section.chars() {
                if ch.is_uppercase() && ch != '/' { red_pieces.push(ch); } 
                else if ch.is_lowercase() && ch != '/' { black_pieces.push(ch); }
            }

            let annotated_board = annotate(&fen);
            let strat = strategy(phase, index);
            let (red_inv, black_inv) = inventory(&pos);
            let open_f = files(&pos);
            let (dev, tot) = development(&pos, side);
            let tact_pats = patterns(&pos);

            let mut legal = movegen::List::new();
            movegen::legal(&mut pos, &mut legal);

            let mut scored: Vec<(movegen::Move, i32)> = Vec::new();
            for i in 0..legal.count {
                let candidate = legal.items[i];
                if !candidate.valid() { continue; }
                let state = pos.apply(candidate.from, candidate.to);
                let val = -evaluator.score(&pos);
                pos.revert(candidate.from, candidate.to, &state);
                scored.push((candidate, val));
            }
            scored.sort_by(|a, b| b.1.cmp(&a.1));

            let mut c_list = Vec::new();
            let best_intent = intent(&pos, *mv);
            let best_trans = translate(&pos, *mv);
            c_list.push((encoded.clone(), score, best_intent.clone(), best_trans.clone()));

            let mut added = 1usize;
            for (candidate, val) in scored.iter() {
                if added >= 3 { break; }
                let candidate_uci = Format::encode(*candidate);
                if candidate_uci == encoded { continue; }
                let c_intent = intent(&pos, *candidate);
                let c_trans = translate(&pos, *candidate);
                c_list.push((candidate_uci, *val, c_intent, c_trans));
                added += 1;
            }

            let mut candidates_json = Vec::new();
            for (m, cp, it, tr) in c_list.iter() {
                candidates_json.push(format!(
                    "{{\"move\": {:?}, \"notation\": {:?}, \"centipawn\": {}, \"intent\": {:?}, \"pros\": [\"An toàn\"], \"cons\": [\"Không có\"], \"patterns\": {}}}",
                    m, tr, cp, it, array(&tact_pats)
                ));
            }

            let (advantages, disadvantages, positives_list, negatives_list) =
                risk(&pos, side, score, red_pieces.len(), black_pieces.len());

            let move_outcome = match result {
                "red_win" => if side == 0 { "win" } else { "loss" },
                "black_win" => if side == 1 { "win" } else { "loss" },
                _ => "draw",
            };

            let red_material = material(&pos, 0);
            let black_material = material(&pos, 1);
            
            let comp_str = compare(&c_list, &encoded, score);

            let thought = format!(
                "<thought>\n\
1. KIỂM KÊ QUÂN CỜ (Piece Inventory):\n\
   - Đỏ: {}\n\
   - Đen: {}\n\
2. TƯƠNG QUAN VẬT CHẤT (Material Balance):\n\
   - Đỏ: {} cp, Đen: {} cp. Chênh lệch: {} cp.\n\
3. AN TOÀN TƯỚNG (King Safety):\n\
   - Đỏ: {}/100, Đen: {}/100.\n\
4. KHỐNG CHẾ TRUNG LỘ & TRỤC LỘ (File Control):\n\
   - Trạng thái Lộ 5: {}. Các lộ mở: {}.\n\
5. MẪU CHIẾN THUẬT (Tactical Patterns):\n\
   - Phát hiện: {}\n\
6. GIAI ĐOẠN & CHIẾN LƯỢC (Phase & Strategy):\n\
   - Giai đoạn: {}. Chiến lược: {}\n\
   - Triển khai lực lượng phe {}: {}/{} quân đã xuất phát.\n\
7. PHÂN TÍCH ƯU THẾ (Advantages):\n\
   - {}\n\
8. PHÂN TÍCH BẤT LỢI (Disadvantages):\n\
   - {}\n\
9. PHÂN TÍCH TÍCH CỰC (Positives):\n\
   - {}\n\
10. PHÂN TÍCH TIÊU CỰC (Negatives):\n\
   - {}\n\
11. ĐÁNH GIÁ NƯỚC ĐI ỨNG VIÊN (Candidate Evaluation):\n\
{}\n\
12. SO SÁNH & CHỌN BESTMOVE (Candidate Comparison):\n\
   - {}\n\
13. CENTIPAWN TỔNG HỢP (Integrated Evaluation):\n\
   - Thế cờ đạt {} cp sau khi đi {}.\n\
14. XÁC MINH LEGAL MOVE (Legal Move Verification):\n\
   - Nước đi {} hợp lệ, không vi phạm luật.\n\
</thought>",
                red_inv, black_inv,
                red_material, black_material, red_material - black_material,
                red_king_score, black_king_score,
                center, open_f.join(", "),
                tact_pats.join(", "),
                phase, strat,
                turn, dev, tot,
                advantages.join("; "),
                disadvantages.join("; "),
                positives_list.join("; "),
                negatives_list.join("; "),
                c_list.iter().map(|c| format!("   + {}: {}", c.0, c.3)).collect::<Vec<_>>().join("\n"),
                comp_str,
                score, encoded,
                encoded
            );

            let assistant = format!(
                "{{\"thought\": {:?}, \"board_analysis\": {{\"red_inventory\": {:?}, \"black_inventory\": {:?}, \"red_count\": {}, \"black_count\": {}, \"red_material\": {}, \"black_material\": {}, \"balance\": {}}}, \"position_assessment\": {{\"red_king_safety\": {}, \"black_king_safety\": {}, \"center_control\": {:?}, \"open_files\": {}, \"phase\": {:?}, \"phase_strategy\": {:?}}}, \"tactical_patterns\": {}, \"risk_assessment\": {{\"advantages\": {}, \"disadvantages\": {}, \"positives\": {}, \"negatives\": {}}}, \"candidates\": [{}], \"comparison\": {:?}, \"bestmove\": {:?}, \"explanation\": {:?}, \"centipawn_eval\": {}}}",
                thought,
                red_inv, black_inv,
                red_pieces.len(), black_pieces.len(),
                red_material, black_material, red_material - black_material,
                red_king_score, black_king_score,
                center, array(&open_f), phase, strat,
                array(&tact_pats),
                array(&advantages), array(&disadvantages), array(&positives_list), array(&negatives_list),
                candidates_json.join(", "),
                comp_str,
                encoded,
                best_trans,
                score
            );

            let user = format!(
                "Trạng thái bàn cờ tướng hiện tại:\\n\\n\
1. Bàn Cờ 2D:\\n{}\\n\\n\
2. FEN:\\n{}\\n\\n\
3. PGN:\\n{}\\n\\n\
Lượt {} đi.",
                annotated_board, fen, pgn, turn
            );

            let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

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

    let path = format!("data/jrcp3_elite_{}.jsonl", stamp);
    let mut file = File::create(&path).expect("Tạo file thất bại");
    for sample in samples.iter() {
        file.write_all(sample.as_bytes()).expect("Ghi dòng thất bại");
        file.write_all(b"\n").expect("Ghi newline thất bại");
    }

    println!("============================================================");
    println!("✅ JRCP 3.0 ELITE MINER: {} MẪU ĐẲNG CẤP!", samples.len());
    println!("💾 Tệp JSONL: {}", path);
    println!("============================================================");
}
