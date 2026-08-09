# 🏛️ ĐẶC TẢ CHUẨN JRCP 3.0 (XIANGQI REASONING & PROTOCOL 3.0)
# Thư mục: .agents/memory/jrcp_3_0_spec.md
# Phiên bản: 3.0.0 | Ngày tạo: 2026-08-09 | Tác giả: HDQB & Antigravity Agent
# Mục đích: Định nghĩa chuẩn cấu trúc dữ liệu huấn luyện thế hệ mới cho Xiangqi-R1.
# Tiến hóa: JRCP 2.0 (14 chiều kích cơ bản) → JRCP 3.0 (14 chiều kích siêu chi tiết + 5 lớp tri thức tự chứa)

---

## I. TẦM NHÌN & TRIẾT LÝ JRCP 3.0

JRCP 3.0 ra đời từ bài học xương máu của JRCP 2.0: **dữ liệu huấn luyện thiếu chi tiết = mô hình ảo giác**.

### Nguyên tắc Vàng:
> "Càng nhiều chi tiết, tài liệu, giải thích, chú thích, diễn giải, đặc tả kỹ thuật
> trong dữ liệu huấn luyện → mô hình càng thông minh, bớt ảo giác."

### So sánh JRCP 2.0 vs JRCP 3.0:

| Khía Cạnh | JRCP 2.0 | JRCP 3.0 |
|---|---|---|
| System Prompt | Chỉ có quy tắc phân tích | **5 lớp tri thức tự chứa** (luật cờ, bản đồ, chiến thuật, giai đoạn, quy trình) |
| Thought Chain | ~10 dòng, sơ sài | **25-40 dòng**, dẫn chứng tọa độ cụ thể |
| Board Representation | Raw FEN + matrix 2D | **Annotated board** + tên quân Việt + tọa độ |
| UCI Notation | Chỉ có "b2e2" | **"Pháo (b2) bình sang Trung Lộ (e2) — Pháo Đầu"** |
| Tactical Intent | 1 câu generic | **2-3 câu chi tiết** + ưu/nhược điểm |
| Risk Assessment | Lặp lại, generic | **Cụ thể với tọa độ** ("Xe Đỏ a0 chưa xuất quân") |
| Candidate Analysis | Liệt kê 3 nước | **So sánh chi tiết** tại sao best > others |
| Tactical Patterns | Không có | **14+ mẫu chiến thuật** tự động nhận diện |
| Phase Strategy | Chỉ label | **Chiến lược cụ thể** cho từng giai đoạn |
| Piece Inventory | Đếm số lượng | **Liệt kê vị trí từng quân** |
| JSON Fields | 6 fields | **12+ fields** |
| Tokens/mẫu | ~500 | **~1500-2000** |

---

## II. JSON SCHEMA JRCP 3.0 ĐẦY ĐỦ

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "XiangqiR1_JRCP_3_0_Schema",
  "type": "object",
  "properties": {
    "thought": {
      "type": "string",
      "description": "Chuỗi suy luận 14 chiều kích siêu chi tiết (25-40 dòng) trong thẻ <thought>...</thought>, BẮT BUỘC dẫn chứng tọa độ cụ thể"
    },
    "board_analysis": {
      "type": "object",
      "properties": {
        "red_inventory": { "type": "string", "description": "Liệt kê vị trí từng quân Đỏ (VD: Tướng (e0), Sĩ (d0), ...)" },
        "black_inventory": { "type": "string", "description": "Liệt kê vị trí từng quân Đen" },
        "red_count": { "type": "integer", "description": "Tổng số quân Đỏ" },
        "black_count": { "type": "integer", "description": "Tổng số quân Đen" },
        "red_material": { "type": "integer", "description": "Tổng giá trị vật chất Đỏ (centipawn)" },
        "black_material": { "type": "integer", "description": "Tổng giá trị vật chất Đen (centipawn)" },
        "balance": { "type": "integer", "description": "Chênh lệch vật chất (Đỏ - Đen)" }
      },
      "required": ["red_inventory", "black_inventory", "red_count", "black_count", "red_material", "black_material", "balance"]
    },
    "position_assessment": {
      "type": "object",
      "properties": {
        "red_king_safety": { "type": "integer", "minimum": 0, "maximum": 100 },
        "black_king_safety": { "type": "integer", "minimum": 0, "maximum": 100 },
        "center_control": { "type": "string", "enum": ["OPEN_CENTER", "RED_CENTER_CONTROL", "BLACK_CENTER_CONTROL", "CONTESTED_CENTER", "RED_PHAO_DAU_INTENT", "BLACK_PHAO_DAU_INTENT"] },
        "open_files": { "type": "array", "items": { "type": "string" } },
        "phase": { "type": "string", "enum": ["opening", "midgame", "endgame"] },
        "phase_strategy": { "type": "string" }
      },
      "required": ["red_king_safety", "black_king_safety", "center_control", "open_files", "phase", "phase_strategy"]
    },
    "tactical_patterns": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Danh sách mẫu chiến thuật phát hiện trên bàn cờ"
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
          "notation": { "type": "string", "description": "Ký hiệu cờ Tướng tiếng Việt (VD: Pháo 2 bình 5)" },
          "centipawn": { "type": "integer" },
          "intent": { "type": "string", "description": "Ý đồ chiến thuật chi tiết 2-3 câu" },
          "pros": { "type": "array", "items": { "type": "string" } },
          "cons": { "type": "array", "items": { "type": "string" } },
          "patterns": { "type": "array", "items": { "type": "string" } }
        },
        "required": ["move", "notation", "centipawn", "intent", "pros", "cons", "patterns"]
      },
      "minItems": 1
    },
    "comparison": {
      "type": "string",
      "description": "So sánh chi tiết tại sao bestmove tốt hơn các ứng viên khác"
    },
    "bestmove": {
      "type": "string",
      "pattern": "^[a-i][0-9][a-i][0-9]$"
    },
    "explanation": {
      "type": "string",
      "description": "Giải thích bằng ngôn ngữ tự nhiên tiếng Việt tại sao đây là nước đi tối ưu"
    },
    "centipawn_eval": {
      "type": "integer"
    }
  },
  "required": ["thought", "board_analysis", "position_assessment", "tactical_patterns", "risk_assessment", "candidates", "comparison", "bestmove", "explanation", "centipawn_eval"]
}
```

---

## III. MẪU DỮ LIỆU CONVERSATION JRCP 3.0 CHUẨN HOÀN HẢO

```json
{
  "messages": [
    {
      "role": "system",
      "content": "[NỘI DUNG ĐẦY ĐỦ TỪ .agents/memory/jrcp_3_0_prompt.md — 5 LỚP TRI THỨC TỰ CHỨA]"
    },
    {
      "role": "user",
      "content": "Trạng thái bàn cờ tướng hiện tại:\n\n1. Bàn Cờ Có Chú Thích (Annotated Board):\n9 | r  n  b  a  k  a  b  n  r\n8 | .  .  .  .  .  .  .  .  .\n7 | .  c  .  .  .  .  .  c  .\n6 | p  .  p  .  p  .  p  .  p\n5 | .  .  .  .  .  .  .  .  .\n4 | .  .  .  .  .  .  .  .  .\n3 | P  .  P  .  P  .  P  .  P\n2 | .  C  .  .  .  .  .  C  .\n1 | .  .  .  .  .  .  .  .  .\n0 | R  N  B  A  K  A  B  N  R\n    a  b  c  d  e  f  g  h  i\n\n2. FEN: rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1\n\n3. Kiểm Kê Quân Cờ:\n   Đỏ (16 quân): Tướng (e0), Sĩ (d0), Sĩ (f0), Tượng (c0), Tượng (g0), Mã (b0), Mã (h0), Xe (a0), Xe (i0), Pháo (b2), Pháo (h2), Tốt (a3), Tốt (c3), Tốt (e3), Tốt (g3), Tốt (i3)\n   Đen (16 quân): Tướng (e9), Sĩ (d9), Sĩ (f9), Tượng (c9), Tượng (g9), Mã (b9), Mã (h9), Xe (a9), Xe (i9), Pháo (b7), Pháo (h7), Tốt (a6), Tốt (c6), Tốt (e6), Tốt (g6), Tốt (i6)\n\n4. Lịch Sử PGN: Ván cờ mới bắt đầu\n\n5. Giai Đoạn: Khai cuộc (Nước thứ 1)\n\nĐến lượt Đỏ đi. Hãy suy nghĩ sâu sắc trong thẻ <thought> và trả về JRCP 3.0 Structured Output JSON."
    },
    {
      "role": "assistant",
      "content": "{...JRCP 3.0 JSON OUTPUT SIÊU CHI TIẾT...}"
    }
  ],
  "move": "b2e2",
  "eval": 50,
  "outcome": "win",
  "phase": "opening",
  "depth": 4,
  "nodes": 12345,
  "stamp": 1723161600
}
```

---

## IV. PIPELINE TẠO DỮ LIỆU JRCP 3.0

```
┌─────────────────────────────────────────────────────────────────┐
│  Rust Engine (22_jrcp3_miner.rs)                                │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐    │
│  │ Self-Play │──▶│ Analyze  │──▶│ Generate │──▶│  Output  │    │
│  │  (Config) │   │ Position │   │  JRCP 3.0│   │  JSONL   │    │
│  └──────────┘   │          │   │  Sample  │   └──────────┘    │
│                  │ inventory│   │          │                    │
│                  │ patterns │   │ thought  │                    │
│                  │ safety   │   │ board    │                    │
│                  │ control  │   │ position │                    │
│                  │ material │   │ tactical │                    │
│                  │ risk     │   │ risk     │                    │
│                  │ files    │   │ cands    │                    │
│                  │ develop  │   │ compare  │                    │
│                  │ translate│   │ explain  │                    │
│                  │ annotate │   │ json     │                    │
│                  └──────────┘   └──────────┘                    │
└─────────────────────────────────────────────────────────────────┘
                         │
                         ▼
              data/jrcp3_elite_*.jsonl
                         │
                         ▼
              ┌─────────────────────┐
              │  HuggingFace Hub    │
              │  xiangqi-nnue-dataset│
              │  (community/ folder)│
              └─────────────────────┘
                         │
                         ▼
              ┌─────────────────────┐
              │  GRPO / SFT Train   │
              │  (Colab T4 GPU)     │
              │  xiangqi-r1-0.5b v2 │
              └─────────────────────┘
```

---

## V. NGUYÊN TẮC BẢO TOÀN THỂ HỆ AGENT (GENERATIONAL PRESERVATION)

Mọi Agent tương lai khi đọc tệp này PHẢI nhớ:
- **KHÔNG BAO GIỜ** rút gọn JRCP 3.0 thành chuỗi text tóm tắt đơn giản.
- **KHÔNG BAO GIỜ** xén bớt 5 lớp tri thức trong System Prompt.
- **KHÔNG BAO GIỜ** giảm chi tiết thought chain xuống dưới 20 dòng.
- **LUÔN LUÔN** nhúng System Prompt đầy đủ từ `jrcp_3_0_prompt.md`.
- **LUÔN LUÔN** kiểm tra JSON Schema trước khi xuất dữ liệu.
- **LUÔN LUÔN** dẫn chứng tọa độ cụ thể trong thought chain và risk assessment.
