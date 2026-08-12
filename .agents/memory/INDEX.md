# BẢNG MỤC LỤC KÝ ỨC VĨNH CỬU (IMMUTABLE MEMORY INDEX)
# Thư mục: .agents/memory/
# Cập nhật: 2026-08-08 (Kế Thừa Tri Thức Thế Hệ Agent Xiangqi-R1)

---

## 1. Danh Sách Bài Học Xương Máu & Lịch Sử Thất Bại

- [`pain_points_20260807.md`](file://.agents/memory/pain_points_20260807.md) — Bài học xương máu ván cờ: Sửa lỗi Beta Cutoff nút gốc ply=0 ở Depth 12, bảo vệ TT khỏi ô nhiễm Abort, khống chế thời gian Soft/Hard Limit, cách ly phiên WebSocket session_search, và nâng cấp IndexedDB không giới hạn 5MB.
- [`pain_points_20260808.md`](file://.agents/memory/pain_points_20260808.md) — Bài học xương máu đắt giá: Chống AI Slop biến chất, cấm dùng dữ liệu giả random, cấm tóm tắt suy luận cẩu thả, cấm báo cáo khống chưa xác minh HfApi list_repo_files trên HuggingFace Dataset Hub.
- [`pain_points_20260809.md`](file://.agents/memory/pain_points_20260809.md) — **Bài học xương máu tối thượng**: Diệt trừ cẩu thả, AI Slop biến chất & Kỷ luật Kế hoạch - Versioning - Telemetry - Logger - Metrics. Quy tắc Dừng 1 nhịp quan sát thiếu sót trước khi hành động.
- [`pain_points_20260810_colab_mcp.md`](file:///Users/hdqb/workspaces/xiangqi-rim/.agents/memory/pain_points_20260810_colab_mcp.md): Bài học xương máu & quy tắc bắt buộc cho Colab MCP (Live DOM sync, Self-contained Form Cells, 403 Forbidden preflight check).
- [`pain_points_20260810.md`](file:///Users/hdqb/workspaces/xiangqi-rim/.agents/memory/pain_points_20260810.md): Lịch sử bài học xương máu Colab GPU T4, rule cờ tướng, Flying General, Rich Visual HTML, 43 Unit Tests.
- [`pain_points_20260811.md`](file:///Users/hdqb/workspaces/xiangqi-rim/.agents/memory/pain_points_20260811.md): Tích hợp Real-time ELO Benchmark (`26_tournament_benchmark`), nâng 85-90% VRAM Saturation (`BATCH=65536`), và chuyển đổi Apache Parquet (`.parquet` ~190MB).
- [`pain_points_20260812.md`](file:///Users/hdqb/workspaces/xiangqi-rim/.agents/memory/pain_points_20260812.md): Phân tích toàn diện & tố cáo 6 mặt tối, rủi ro tiềm ẩn (Ảo tưởng thông lượng static batch vs dynamic tree search, xung đột Alpha-Beta pruning, nghẽn PCIe transfer latency, và kỷ luật đính chính báo cáo CPU miner).
- [`jrcp_2_0_spec.md`](file://.agents/memory/jrcp_2_0_spec.md) — **Đặc Tả Chuẩn JRCP 2.0 (Xiangqi Reasoning & Protocol 2.0)**: Định nghĩa cấu trúc dữ liệu huấn luyện và Structured Output 14 chiều kích (Ma Trận Bàn Cờ 2D, FEN, PGN, Centipawn, Cơ Hội, Nguy Cơ, Tích Cực, Tiêu Cực, Ma Trận 3 Nước Đi Candidate, Đồ Thị DAG, Legal Move 100%).
- [`jrcp_3_0_prompt.md`](file://.agents/memory/jrcp_3_0_prompt.md) — **System Prompt JRCP 3.0 Tự Chứa 5 Lớp Tri Thức**: Nhúng trực tiếp toàn bộ tri thức cờ Tướng vào system prompt — Từ Điển Quân Cờ & Luật Di Chuyển, Bản Đồ Bàn Cờ & Hệ Tọa Độ, Từ Điển Chiến Thuật Kinh Điển (14+ mẫu), Chiến Lược Giai Đoạn, Quy Trình Suy Luận 14 Chiều Kích Mở Rộng.
- [`jrcp_3_0_spec.md`](file://.agents/memory/jrcp_3_0_spec.md) — **Đặc Tả Chuẩn JRCP 3.0 (Xiangqi Reasoning & Protocol 3.0)**: JSON Schema mở rộng 12+ fields (thought, board_analysis, position_assessment, tactical_patterns, risk_assessment, candidates có notation/pros/cons/patterns, comparison, bestmove, explanation, centipawn_eval). So sánh chi tiết JRCP 2.0 vs 3.0, pipeline tạo dữ liệu, mẫu conversation chuẩn hoàn hảo.

---

## 2. Di Sản Tri Thức & Giao Thức Khởi Động Phiên Bắt Buộc (Generational Legacy Protocol)

Mọi Agent Xiangqi-R1 thế hệ sau khi bắt đầu phiên làm việc BẮT BỘC phải dừng lại 1 nhịp, đọc thuộc lòng các quy tắc và kiểm tra kỹ năng tại `.agents/skills/` trước khi đưa ra bất kỳ hành động nào.
