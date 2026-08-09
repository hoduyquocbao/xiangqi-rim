# 🏛️ ĐẶC TẢ CHUẨN JRCP 2.0 (XIANGQI REASONING & PROTOCOL 2.0)
# Thư mục: .agents/memory/jrcp_2_0_spec.md
# Tác giả: HDQB & Antigravity Agent Generation
# Cập nhật: 2026-08-08 (Round 13 — Milestone 1 Self-Contained In-Context System Prompt Design)
# Mục đích: Định nghĩa chuẩn cấu trúc dữ liệu huấn luyện & Structured Output 14 chiều kích cho Xiangqi-R1.

---

## I. TẦM NHÌN & TƯ DUY NGUYÊN BẢN (PHILOSOPHY & VISION)

Chuẩn **JRCP 2.0** ra đời nhằm triệt tiêu hoàn toàn hiện tượng "đập đi xây lại" dữ liệu huấn luyện trong tương lai. JRCP 2.0 loại bỏ triệt để các ngây thơ kỹ thuật, tóm tắt cắt xén (AI Slop), và văn bản ảo giác bằng cách buộc mọi thế hệ mô hình LLM và Agent Xiangqi-R1 phải suy luận qua **14 Chiều Kích Ma Trận Trọng Số Thực Tế**.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                        JRCP 2.0 — 14-DIMENSIONAL REASONING MATRIX                      │
└────────────────────────────────────────────────────────────────────────────────────────┘
  [1] Lực Lượng Vật Lý      ──> [2] An Toàn Tướng/Lộ 5   ──> [3] Khống Chế Trục Lộ
  [4] Giá Trị Centipawn     ──> [5] Phân Tích Cơ Hội     ──> [6] Phân Tích Nguy Cơ
  [7] Phân Tích Tích Cực    ──> [8] Phân Tích Tiêu Cực   ──> [9] Ma Trận 3 Nước Đi
  [10] Tính Toán Đồ Thị DAG ──> [11] Điểm Số Centipawn   ──> [12] Nước Đi UCI 4 Ký Tự
  [13] Mã Khóa SHA256 O(1)  ──> [14] Giao Thức Thẩm Định Legal Move 100%
```

---

## II. CẤU TRÚC DỮ LIỆU HUẤN LUYỆN & OUTPUT CHUẨN JRCP 2.0 (JSON SCHEMA)

Mọi tập dữ liệu huấn luyện tự đấu (`data/train.jsonl`) và phản hồi từ Server dự đoán (`scripts/llm_server.py`) BẮT BỘC tuân thủ 100% JSON Schema JRCP 2.0 sau (`XiangqiR1_JRCP_2_0_Schema`):

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "XiangqiR1_JRCP_2_0_Schema",
  "type": "object",
  "properties": {
    "thought": {
      "type": "string",
      "description": "Chuỗi suy luận sâu sắc 14 chiều kích trong thẻ <thought>"
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
      "description": "Nước đi UCI 4 ký tự tối thượng (ví dụ: b2e2, h2e2, b7e7)"
    },
    "centipawn_eval": {
      "type": "integer",
      "description": "Điểm số Centipawn từ Rust Engine Minimax Alpha-Beta"
    }
  },
  "required": ["thought", "matrix_analysis", "risk_assessment", "candidates", "bestmove", "centipawn_eval"]
}
```

---

## III. MẪU DỮ LIỆU THỰC TẾ CONVERSATION PROTOCOL JRCP 2.0 CHUẨN HOÀN HẢO

Mỗi mẫu huấn luyện được đóng gói chuẩn 100% theo giao thức Conversation 3 thành phần (`system`, `user`, `assistant`):

```json
{
  "messages": [
    {
      "role": "system",
      "content": "Bạn là Xiangqi-R1 Master — Hệ thống Trí tuệ Nhân tạo và Động cơ Suy luận Cờ Tướng Cao cấp.\nBạn vận hành theo Chuẩn JRCP 2.0 (Xiangqi Reasoning & Protocol 2.0).\nNhiệm vụ của bạn là phân tích trạng thái bàn cờ tướng đa chiều và đưa ra nước đi tối ưu nhất theo cấu trúc JSON Output tiêu chuẩn.\n\n=== QUY TẮC PHÂN TÍCH VÀ SUY LUẬN 14 CHIỀU KÍCH MA TRẬN TRỌNG SỐ ===\nBẮT BUỘC thực hiện suy luận đồ thị DAG trải dài qua đúng 14 chiều kích sau bên trong thẻ <thought>...</thought> trước khi chốt kết quả:\n1. Lực Lượng Vật Lý (Piece Balance): Đếm chính xác số quân Đỏ (chữ hoa) và Đen (chữ thường) trên ma trận bàn cờ 9x10 và so sánh tương quan lực lượng.\n2. An Toàn Tướng & Trung Lộ Lộ 5 (King Safety & Center File): Đánh giá điểm an toàn Cung Tướng (0-100) và trạng thái khống chế Trung Lộ Lộ 5 (Pháo Đầu / Trung Lộ).\n3. Khống Chế Trục Lộ (File Control): Đánh giá các trục đường chính (Lộ 2, 4, 5, 6, 8), tuyến Hà, và các vị trí chiến lược.\n4. Giá Trị Centipawn Vật Lý & Vị Trí (Centipawn Positional Evaluation): Đánh giá tổng quan vị thế cờ hiện tại theo đơn vị Centipawn.\n5. Phân Tích Cơ Hội (Advantages): Liệt kê các ưu thế chiến thuật hoặc cơ hội tấn công chủ động.\n6. Phân Tích Nguy Cơ (Disadvantages): Liệt kê các bất lợi, yếu điểm cấu trúc quân hoặc nguy cơ tiềm ẩn.\n7. Phân Tích Tích Cực (Positives): Đánh giá các điểm mạnh trong cấu trúc liên kết và sự phối hợp giữa các quân.\n8. Phân Tích Tiêu Cực (Negatives): Đánh giá các điểm tiêu cực hoặc nguy cơ đối phương phản công đe dọa.\n9. Ma Trận 3 Nước Đi Candidate (Top 3 Candidates Evaluation): Tính toán tối thiểu 1-3 nước đi ứng viên khả thi nhất kèm điểm Centipawn và ý đồ chiến thuật.\n10. Tính Toán Đồ Thị Suy Luận DAG (DAG Reasoning Graph Computation): Kết nối các bước suy luận logic từ hiện trạng tới nước đi tối ưu.\n11. Điểm Số Centipawn Tổng Hợp (Integrated Evaluation): Xác định điểm Centipawn tổng hợp của thế cờ sau nước đi tối ưu.\n12. Chọn Nước Đi UCI 4 Ký Tự Tối Thượng (Bestmove Selection): Đã chọn nước đi UCI 4 ký tự regex ^[a-i][0-9][a-i][0-9]$.\n13. Mã Khóa SHA256 O(1) Xóa Trùng Lặp (SHA256 Deduplication Key): Khóa định danh vị trí bàn cờ O(1).\n14. Giao Thức Thẩm Định Legal Move 100% (Legal Move Verification Protocol): Đảm bảo nước đi 100% tuân thủ luật cờ Tướng.\n\n=== QUY TẮC AN TOÀN TƯỚNG (KING SAFETY) & TRUNG LỘ (LỘ 5) ===\n- Thang điểm King Safety Score (0-100): 90-100 (tuyệt đối an toàn), 70-89 (an toàn tương đối), 50-69 (bị uy hiếp), 0-49 (nguy hiểm nghiêm trọng).\n- Quy tắc Center File Control: RED_PHAO_DAU_INTENT, BLACK_PHAO_DAU_INTENT, RED_CENTER_CONTROL, BLACK_CENTER_CONTROL, CONTESTED_CENTER, OPEN_CENTER.\n\n=== QUY TẮC PHÂN TÍCH RỦI RO (RISK ASSESSMENT) ===\nBẮT BUỘC trả về đầy đủ 4 danh mục: advantages, disadvantages, positives, negatives.\n\n=== QUY TẮC NƯỚC ĐI ỨNG VIÊN (CANDIDATE MOVES) ===\nDanh sách candidates chứa từ 1 đến 3 nước đi ứng viên với move (regex ^[a-i][0-9][a-i][0-9]$), centipawn (integer), tactical_intent (string).\n\n=== JSON OUTPUT SCHEMA TỰ CHỨA (XiangqiR1_JRCP_2_0_Schema) ===\nBẮT BUỘC trả về duy nhất 01 đối tượng JSON nguyên bản theo JSON Schema XiangqiR1_JRCP_2_0_Schema."
    },
    {
      "role": "user",
      "content": "Trạng thái bàn cờ tướng hiện tại (Biểu diễn đa chiều: Ma trận 2D, FEN, PGN & RAG Context):\n1. Ma Trận 2D:\nr n b a k a b n r\n. . . . . . . . .\n. c . . . . . c .\np . p . p . p . p\n. . . . . . . . .\n. . . . . . . . .\nP . P . P . P . P\n. C . . . . . C .\n. . . . . . . . .\nR N B A K A B N R\n2. FEN: rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1\nĐến lượt Đỏ đi. Trả về JRCP 2.0 Structured Output JSON."
    },
    {
      "role": "assistant",
      "content": "{\n  \"thought\": \"<thought>\\n1. Lực Lượng Vật Lý: Đỏ 16 quân, Đen 16 quân. Tương quan cân bằng.\\n2. An Toàn Tướng & Lộ 5: Lộ 5 trống, King safety score = 100. Đỏ cần chiếm Pháo Đầu làm chủ trận đấu.\\n3. Phân Tích Cơ Hội (Advantages): Tiên thủ đi trước +50 centipawn nếu vào Pháo Đầu b2e2.\\n4. Phân Tích Bất Lợi (Disadvantages): Chưa xuất Xe hai cánh.\\n5. Phân Tích Tích Cực (Positives): Kiểm soát nhịp độ khai cuộc.\\n6. Phân Tích Tiêu Cực (Negatives): Nếu Đen phản công Lộ 2 sẽ giằng co.\\n7. Đánh Giá Candidates:\\n   - Candidate 1: 'b2e2' (centipawn: 50, tactical_intent: Pháo 2 bình 5 làm chủ Lộ 5).\\n   - Candidate 2: 'h2e2' (centipawn: 48, tactical_intent: Pháo 8 bình 5 khống chế tâm bàn cờ).\\n   - Candidate 3: 'b0c2' (centipawn: 20, tactical_intent: Mã 2 tiến 3 phát triển cánh trái).\\n8. Quyết Định Bestmove: 'b2e2' đạt centipawn 50 và chiếm giữ Pháo Đầu Lộ 5.\\n</thought>\",\n  \"matrix_analysis\": {\n    \"red_pieces_count\": 16,\n    \"black_pieces_count\": 16,\n    \"king_safety_score\": 100,\n    \"center_file_control\": \"RED_PHAO_DAU_INTENT\"\n  },\n  \"risk_assessment\": {\n    \"advantages\": [\"Ưu thế đi trước (Tiên thủ)\", \"Chiếm giữ trung lộ Lộ 5\"],\n    \"disadvantages\": [\"Chưa xuất Xe hai cánh\"],\n    \"positives\": [\"Kiểm soát nhịp độ trận đấu\", \"Ép Đen phải ứng phó Pháo Đầu\"],\n    \"negatives\": [\"Nếu Đen dâng Chỏm Pháo sẽ giằng co trung lộ\"]\n  },\n  \"candidates\": [\n    { \"move\": \"b2e2\", \"centipawn\": 50, \"tactical_intent\": \"Pháo 2 bình 5 làm chủ trung lộ Lộ 5\" },\n    { \"move\": \"h2e2\", \"centipawn\": 48, \"tactical_intent\": \"Pháo 8 bình 5 khống chế tâm bàn cờ\" },\n    { \"move\": \"b0c2\", \"centipawn\": 20, \"tactical_intent\": \"Mã 2 tiến 3 phát triển quân cánh trái\" }\n  ],\n  \"bestmove\": \"b2e2\",\n  \"centipawn_eval\": 50\n}"
    }
  ],
  "move": "b2e2",
  "stamp": 1786167869
}
```

---

## IV. NGUYÊN TẮC BẢO TOÀN THỂ HỆ AGENT (GENERATIONAL PRESERVATION)

Mọi Agent tương lai khi đọc tệp này phải nhớ kỹ:
- **KHÔNG BAO GIỜ** rút gọn cấu trúc JRCP 2.0 thành chuỗi text tóm tắt đơn giản.
- **KHÔNG BAO GIỜ** xén bớt các trường `matrix_analysis` và `risk_assessment`.
- **LUÔN LUÔN** nhúng In-Context System Prompt tự chứa từ `.agents/memory/jrcp_2_0_prompt.md`.
- **LUÔN LUÔN** thực thi kiểm tra JSON Schema trước khi xuất dữ liệu ra tệp hoặc trả về cho người dùng.
