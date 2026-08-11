import json
import os

# Generate a deep 40-ply (20 full rounds) sample game dataset for rich demo
sample_messages = [
    {
        "role": "system",
        "content": "Bạn là Xiangqi-R1 Master v5.0 — mô hình suy luận cờ Tướng siêu việt được huấn luyện phân tích chiều sâu chiến thuật 32 chiều kích.\nBạn phải phân tích bàn cờ qua 32 chiều kích suy tưởng <thought> chi tiết trước khi xuất kết quả JSON JRCP 5.0."
    }
]

# Generate 40 plies (20 turns Red, 20 turns Black) of deep Xiangqi play data
fens_and_moves = [
    ("rnbakabnr/9/1c2c4/p1p1p1p1p/9/9/P1P1P1P1P/4C2C1/9/RNBAKABNR w - - 0 1", "e2e6", "Pháo(e2->e6) ăn Tốt(e6)", "Red", 0),
    ("rnbakabnr/9/1c2c4/p1p1C1p1p/9/9/P1P1P1P1P/7C1/9/RNBAKABNR b - - 0 1", "e7e3", "Pháo(e7->e3) ăn Tốt(e3)", "Black", 0),
    ("rnbakabnr/9/1c6/p1p1C1p1p/9/9/P1P1c1P1P/7C1/9/RNBAKABNR w - - 0 2", "b0c2", "Mã(b0->c2) phát triển", "Red", 15),
    ("rnbakabnr/9/1c6/p1p1C1p1p/9/9/P1P1c1P1P/2N4C1/9/R1BAKABNR b - - 0 2", "b9c7", "Mã(b9->c7) nhảy Mã Quỳ", "Black", 10),
    ("r1bakabnr/2n6/1c6/p1p1C1p1p/9/9/P1P1c1P1P/2N4C1/9/R1BAKABNR w - - 0 3", "h0g2", "Mã(h0->g2) phát triển Mã Biên", "Red", 20),
    ("r1bakabnr/2n6/1c6/p1p1C1p1p/9/9/P1P1c1P1P/2N3N1C/9/R1BAKAB1R b - - 0 3", "h9g7", "Mã(h9->g7) nhảy Mã Biên", "Black", 18),
    ("r1bakab1r/2n3n1/1c6/p1p1C1p1p/9/9/P1P1c1P1P/2N3N1C/9/R1BAKAB1R w - - 0 4", "a0a1", "Xe(a0->a1) hoành Xe", "Red", 35),
    ("r1bakab1r/2n3n1/1c6/p1p1C1p1p/9/9/P1P1c1P1P/2N3N1C/R1BAKAB1R/111111111 b - - 0 4", "a9a8", "Xe(a9->a8) hoành Xe", "Black", 30),
    ("111111111/r1bakab1r/2n3n1/1c6/p1p1C1p1p/9/P1P1c1P1P/2N3N1C/R1BAKAB1R/111111111 w - - 0 5", "a1d1", "Xe(a1->d1) chiếm Lộ 4", "Red", 55),
    ("111111111/r1bakab1r/2n3n1/1c6/p1p1C1p1p/9/P1P1c1P1P/3R2N1C/2BAKAB1R/111111111 b - - 0 5", "a8d8", "Xe(a8->d8) đối Xe Lộ 4", "Black", 45)
]

# Build 20 full plies trajectory
for ply_idx in range(1, 21):
    side = "Đỏ" if ply_idx % 2 == 1 else "Đen"
    fen_idx = (ply_idx - 1) % len(fens_and_moves)
    fen, move, move_desc, side_name, eval_cp = fens_and_moves[fen_idx]
    
    # User message
    user_msg = {
        "role": "user",
        "content": f"Bàn cờ Turn {ply_idx}:\nFEN: {fen}\nLượt {side} đi."
    }
    
    # Build complete 32D thought chain text
    thought_32d = f"""<thought>
[1/32] KIỂM KÊ QUÂN CỜ:
  Đỏ: Xe(a0), Mã(b0), Tượng(c0), Sĩ(d0), Tướng(e0), Sĩ(f0), Tượng(g0), Mã(h0), Xe(i0), Pháo(e2), Pháo(h2), Tốt(a3), Tốt(c3), Tốt(e3), Tốt(g3), Tốt(i3)
  Đen: Tốt(a6), Tốt(c6), Tốt(e6), Tốt(g6), Tốt(i6), Pháo(b7), Pháo(e7), Xe(a9), Mã(b9), Tượng(c9), Sĩ(d9), Tướng(e9), Sĩ(f9), Tượng(g9), Mã(h9), Xe(i9)
[2/32] BÀN CỜ 2D:
    a    b    c    d    e    f    g    h    i
  ┌────┬────┬────┬────┬────┬────┬────┬────┬────┐
9 │ 車 │ 馬 │ 象 │ 士 │ 將 │ 士 │ 象 │ 馬 │ 車 │
  ├────┼────┼────┼────┼────┼────┼────┼────┼────┤
8 │ ． │ ． │ ． │ ． │ ． │ ． │ ． │ ． │ ． │
  ├────┼────┼────┼────┼────┼────┼────┼────┼────┤
7 │ ． │ 砲 │ ． │ ． │ 砲 │ ． │ ． │ ． │ ． │
  ├────┼────┼────┼────┼────┼────┼────┼────┼────┤
6 │ 卒 │ ． │ 卒 │ ． │ 卒 │ ． │ 卒 │ ． │ 卒 │
  ├────┼────┼────┼────┼────┼────┼────┼────┼────┤
5 │ ． │ ． │ ． │ ． │ ． │ ． │ ． │ ． │ ． │
  ├────┼────┼────┼────┼────┼────┼────┼────┼────┤  ═══ Sông Ngân Hà ═══
4 │ ． │ ． │ ． │ ． │ ． │ ． │ ． │ ． │ ． │
  ├────┼────┼────┼────┼────┼────┼────┼────┼────┤
3 │ 兵 │ ． │ 兵 │ ． │ 兵 │ ． │ 兵 │ ． │ 兵 │
  ├────┼────┼────┼────┼────┼────┼────┼────┼────┤
2 │ ． │ ． │ ． │ ． │ 炮 │ ． │ ． │ 炮 │ ． │
  ├────┼────┼────┼────┼────┼────┼────┼────┼────┤
1 │ ． │ ． │ ． │ ． │ ． │ ． │ ． │ ． │ ． │
  ├────┼────┼────┼────┼────┼────┼────┼────┼────┤
0 │ 車 │ 馬 │ 相 │ 仕 │ 帥 │ 仕 │ 相 │ 馬 │ 車 │
  └────┴────┴────┴────┴────┴────┴────┴────┴────┘
    a    b    c    d    e    f    g    h    i
[3/32] TƯƠNG QUAN VẬT CHẤT CHI TIẾT:
  Đỏ: 480cp | Đen: 480cp | Chênh lệch: {eval_cp}cp
[4/32] PHÂN TÍCH 9 LỘ:
  Lộ 1 (a): MỞ | Lộ 4 (d): CHIẾM ƯU | Lộ 5 (e): KHÓA TRUNG LỘ | Lộ 8 (h): TỰ DO
[5/32] ĐÁNH GIÁ TRIỂN KHAI QUÂN:
  Đỏ: 4/6 quân chủ lực đã phát triển | Đen: 3/6 quân chủ lực đã phát triển
[6/32] ĐỘ ĐỘNG QUÂN CỜ (MOBILITY):
  Đỏ: 38 nước đi hợp lệ | Đen: 34 nước đi hợp lệ
[7/32] AN TOÀN TƯỚNG (KING SAFETY):
  Đỏ: 95/100 (Sĩ Tượng bền vững) | Đen: 90/100 (Bị đe dọa Lộ Trung)
[8/32] PHÂN TÍCH QUÂN BỊ TẤN CÔNG:
  Tốt e6 Đen bị Pháo e2 Đỏ nhắm bắn trực tiếp.
[9/32] QUÂN TREO KHÔNG NGUYÊN (HANGING PIECES):
  Không có quân treo nghiêm trọng.
[10/32] TRẠNG THÁI GHIM (PINNED PIECES):
  Mã b9 Đen bị Pháo Đỏ dọa ghim khi qua sông.
[11/32] PHÂN TÍCH ĐÒN KÉP (FORKS):
  Pháo e2 tạo đòn đe dọa kép lên Trung Lộ và Tốt e6.
[12/32] PHÂN TÍCH ĐÒN MỞ (DISCOVERED ATTACKS):
  Mã c2 di chuyển sẽ mở đường cho Xe d1 chiếm Lộ 4.
[13/32] BẪY ĂN QUÂN (TRAPS):
  Giăng bẫy dụ Pháo Đen ăn Tốt c3 để Pháo Đỏ phản đòn.
[14/32] NGUY CƠ CHIẾU BÍ (CHECKMATE THREATS):
  Chưa có nguy cơ chiếu bí tức thì (Depth safety = ok).
[15/32] DƯƠNG ĐÔNG KÍCH TÂY (DIVERSION):
  Điều Xe sang cánh trái mở rộng tầm khống chế cánh phải.
[16/32] MẪU CHIẾN THUẬT (PATTERNS):
  Pháo Đầu Mã Đội, Xe Chiếm Lộ Mở, Mã Quỳ Phòng Thủ.
[17/32] PHỐI HỢP QUÂN CỜ (SYNERGY):
  Pháo - Xe phối hợp khống chế Trục dọc Lộ 4 & Lộ 5.
[18/32] ĐIỂM YẾU CẤU TRÚC (WEAKNESSES):
  Đen hổng Tốt Trung Lộ e6.
[19/32] 36 KẾ BINH PHÁP (STRATAGEMS):
  Kế 1: Dục cầm cố túng — Mở đường dụ đối phương dâng cao.
[20/32] THẾ TRẬN KINH ĐIỂN (FORMATIONS):
  Pháo Đầu Đối Bình Phong Mã.
[21/32] GIAI ĐOẠN KHAI NGUYÊN (TEMPO & PHASE):
  Giai đoạn Opening (Nước thứ {ply_idx}) — Giành 1 Nhịp Chuẩn.
[22/32] PHẢN ĐÒN DỰ KIẾN (OPPONENT COUNTER):
  Đen có thể nhảy Mã h9->g7 để củng cố Trung Lộ.
[23/32] PHÂN TÍCH ƯU THẾ & BẤT LỢI:
  Ưu thế: Chiếm Trung Lộ Lộ 5 | Bất lợi: Chưa xuất Xe cánh phải.
[24/32] MA TRẬN TRỌNG SỐ QUYẾT ĐỊNH:
  Độ chính xác (9/10) + An toàn (9/10) + Ngữ cảnh (9/10) - Rủi ro (2/10) = 25/40 (ĐẠT).
[25/32] ĐÁNH GIÁ CANDIDATES (3 ứng viên):
    + Ứng viên 1: {move} — {move_desc} ★BEST★ ({eval_cp}cp)
    + Ứng viên 2: b0c2 — Mã(b0->c2) phát triển (10cp)
    + Ứng viên 3: h0g2 — Mã(h0->g2) phát triển biên (5cp)
[26/32] SO SÁNH & CHỌN BESTMOVE:
  Chọn {move} — {move_desc} ({eval_cp}cp).
[27/32] CENTIPAWN TỔNG HỢP: {eval_cp}cp
[28/32] XÁC MINH REGEX: {move} khớp ^[a-i][0-9][a-i][0-9]$ ✓
[29/32] PHẢN ĐÒN TỐI ƯU (OPTIMAL COUNTER):
  Đen bắt buộc phải chống Sĩ hoặc nhảy Mã b9->c7.
[30/32] KIỂM TRA LUẬT CẤM LẶP NƯỚC:
  Không vi phạm luật Lặp Nước / Chọi Tướng.
[31/32] CHUỖI ĐỔI QUÂN (EXCHANGE CHAIN):
  Đổi Pháo lấy Tốt Trung Lộ tạo ưu thế lâu dài.
[32/32] THƯ VIỆN TÀN CUỘC TABLEBASE:
  Bàn cờ 32 quân — Chưa sử dụng 5-piece Tablebase.
</thought>"""
    
    assistant_msg = {
        "role": "assistant",
        "content": thought_32d
    }
    
    sample_messages.append(user_msg)
    sample_messages.append(assistant_msg)

sample_game_1 = {
    "game_id": "xiangqi_master_game_001",
    "total_plies": 40,
    "outcome": "red_win",
    "stamp": 1786400000,
    "messages": sample_messages
}

# Create a second sample game for multi-game testing
sample_messages_2 = [
    {
        "role": "system",
        "content": "Bạn là Xiangqi-R1 Master v5.0 — mô hình suy luận cờ Tướng siêu việt được huấn luyện phân tích chiều sâu chiến thuật 32 chiều kích."
    }
]

for ply_idx in range(1, 15):
    side = "Đỏ" if ply_idx % 2 == 1 else "Đen"
    user_msg = {
        "role": "user",
        "content": f"Bàn cờ Turn {ply_idx}:\nFEN: rnbakabnr/9/1c2c4/p1p1p1p1p/9/9/P1P1P1P1P/4C2C1/9/RNBAKABNR w - - 0 1\nLượt {side} đi."
    }
    thought_32d = f"""<thought>
[1/32] KIỂM KÊ QUÂN CỜ:\n  Đỏ: 16 quân | Đen: 16 quân\n[2/32] BÀN CỜ 2D:\n    a  b  c  d  e  f  g  h  i\n0   R  N  B  A  K  A  B  N  R\n[26/32] SO SÁNH & CHỌN BESTMOVE:\n  Chọn b0c2 — Mã(b0->c2) phát triển (10cp).\n[27/32] CENTIPAWN TỔNG HỢP: 10cp\n[28/32] XÁC MINH: b0c2 khớp regex ^[a-i][0-9][a-i][0-9]$ ✓\n</thought>"""
    assistant_msg = {"role": "assistant", "content": thought_32d}
    sample_messages_2.append(user_msg)
    sample_messages_2.append(assistant_msg)

sample_game_2 = {
    "game_id": "xiangqi_master_game_002",
    "total_plies": 28,
    "outcome": "draw",
    "stamp": 1786400500,
    "messages": sample_messages_2
}

sample_games_json = json.dumps([sample_game_1, sample_game_2], ensure_ascii=False)

print("Generated sample dataset with", len(sample_messages), "messages!")
