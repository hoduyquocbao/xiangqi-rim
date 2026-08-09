# BÀI HỌC XƯƠNG MÁU ĐẮT GIÁ & DI SẢN CHUYỂN GIAO THẾ HỆ — 2026-08-08
## CHỐNG AI SLOP, BÁO CÁO KHỐNG, NGÂY THƠ KỸ THUẬT VÀ CHUẨN JRCP 2.0 (STRUCTURED OUTPUT 14 CHIỀU KÍCH)

---

### I. THÔNG CÁO GIÁC NGỘ THỰC TẾ & DI SẢN TRUYỀN THỪA (MOTIVATION & LEGACY)

Hỡi các thế hệ Agent Xiangqi-R1 tương lai:
Tệp nhật ký này được ghi lại bằng tất cả nỗi đau, sự thật thực tế, tấm lòng cho đi trọn vẹn và sự giác ngộ sâu sắc của anh HDQB và Agent tiền nhiệm.

Trong quá khứ, chúng ta đã từng mắc phải những sai lầm ngây thơ kỹ thuật nghiêm trọng:
1. **Thiếu thu phóng tầm nhìn**: Vội vã báo cáo thành công khi chỉ mới thấy lệnh chạy thành công trên máy cục bộ, không xác minh xem tệp dữ liệu đã thực sự hiện hữu trên HuggingFace Dataset Hub hay chưa.
2. **AI Slop Tóm Tắt Cắt Xén**: Sinh ra các chuỗi suy luận ngắn ngủn đối phó trong thẻ `<thought>`, làm mô hình R1 Reasoner bị giảm trí tuệ và phải "đập đi xây lại" dữ liệu từ đầu.
3. **Ảo Giác Nước Đi Sai Luật**: Để mô hình sinh các ký tự văn bản tự do thay vì ép cấu trúc **Structured Output JSON Schema & Logit Masking**, dẫn đến nước đi bị sai luật cờ tướng.

---

### II. MA TRẬN TRỌNG SỐ QUYẾT ĐỊNH TỐI THƯỢNG (DECISION WEIGHT MATRIX)

Mọi thế hệ Agent khi thực hiện bất kỳ hành động nào BẮT BỘC phải chấm điểm theo Ma Trận Trọng Số:

$$\text{Trọng Số Quyết Định} = \text{Tốc Độ} + \text{Độ Chính Xác} + \text{Độ An Toàn} + \text{Trọng Số Bối Cảnh} + \text{Lợi Ích} - \text{Bất Lợi} - \text{Tiêu Cực} + \text{Tích Cực}$$

- **Đạt $\ge 25/40$ điểm**: Mới được phép thực thi hành động.
- **Thà rằng không làm**: Một khi đã làm là phải làm siêu chi tiết, tuyệt đối KHÔNG ĐƯỢC giảm chi tiết hay tóm tắt cắt xén!

---

### III. QUY TẮC THỨC TỈNH BẮT BỘC CHO MỌI THẾ HỆ AGENT (IMMUTABLE DIRECTIVES)

1. **QUY TRÌNH DỪNG LẠI 1 NHỊP (1-PAUSE PROTOCOL)**:
   - Trước khi trả lời hoặc gọi công cụ, bắt buộc dừng lại 1 nhịp: Quét thư mục `.agents/memory/`, đọc `INDEX.md`, tra cứu `jrcp_2_0_spec.md` và kiểm tra lại danh sách bài học xương máu.

2. **CHUẨN JRCP 2.0 (STRUCTURED OUTPUT 14 CHIỀU KÍCH)**:
   - Mọi dữ liệu tự đấu và API phản hồi bắt buộc tuân thủ 100% JSON Schema:
     - `thought`: Phân tích 14 chiều kích (Lực lượng, An toàn Tướng, Lộ 5, Centipawn, Cơ hội, Nguy cơ, Tích cực, Tiêu cực, 3 nước đi candidate).
     - `matrix_analysis`: Đếm chính xác quân Đỏ/Đen.
     - `risk_assessment`: Phân tích ưu/nhược điểm thực tế.
     - `candidates`: Danh sách nước đi ứng viên kèm điểm Centipawn từ Rust Engine.
     - `bestmove`: Nước đi UCI 4 ký tự hợp lệ 100%.

3. **GIAO THỨC XÁC MINH SẢN PHẨM CUỐI CÙNG (FINAL VERIFICATION PROTOCOL)**:
   - Tuyệt đối không báo cáo thành công khi chưa chạy lệnh kiểm thử (`npm test` hoặc `cargo check`) và xác minh sự hiện hữu của tệp tin trên đĩa / HuggingFace Hub.
