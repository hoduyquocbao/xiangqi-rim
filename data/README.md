---
license: mit
language:
- vi
tags:
- xiangqi
- chinese-chess
- reasoning
- jrcp-2.0
size_categories:
- 100K<n<1M
---

# Xiangqi-R1 JRCP 2.0 Training Dataset

Bộ dữ liệu huấn luyện đẳng cấp cho mô hình LLM Xiangqi-R1 — AI Cờ Tướng suy luận 14 chiều kích.

## Thống Kê

| Chỉ số | Giá trị |
|---|---|
| Tổng mẫu | **29,774** |
| Opening | 251 |
| Midgame | 2,902 |
| Endgame | 18,687 |
| Win | 317 |
| Loss | 304 |
| Draw | 21,219 |

## Định Dạng

Mỗi mẫu là 1 dòng JSON (JSONL) theo chuẩn JRCP 2.0 Conversation:

```json
{
  "messages": [
    {"role": "system", "content": "...JRCP 2.0 System Prompt..."},
    {"role": "user", "content": "...Ma trận 2D + FEN + PGN..."},
    {"role": "assistant", "content": "...JRCP 2.0 Structured Output JSON..."}
  ],
  "move": "b2e2",
  "eval": 0,
  "outcome": "draw",
  "phase": "opening",
  "depth": 4,
  "nodes": 12345,
  "stamp": 1786172603
}
```

## Nguồn Gốc

Dữ liệu được khai thác từ Native Rust Engine (xiangrust) depth=4 tự đấu,
với phân tích 14 chiều kích JRCP 2.0 cho mỗi vị trí bàn cờ.

## Giấy Phép

MIT License — Tự do sử dụng cho nghiên cứu và thương mại.
