# BÀI HỌC XƯƠNG MÁU & NGUYÊN TẮC QUY ĐỊNH MỚI (2026-08-12)
# Tác giả: HDQB & Antigravity Agent
# Phạm vi: Toàn bộ mã nguồn examples/, tests/, scripts/, Rust Engine, Python Notebooks

---

## 1. QUY TẮC BẮT BUỘC YIELD KẾT QUẢ TỨC THÌ & MONITOR TELEMETRY REALTIME (MANDATORY IMMEDIATE YIELD & TELEMETRY PROTOCOL)

### 1.1 Nguyên nhân ra đời quy tắc
Trong các phiên làm việc trước, Agent có thói quen viết mã xử lý hàng loạt nhưng không in kết quả trung gian (`println!`, `stdout().flush()`), khiến người dùng và hệ thống rơi vào trạng thái "chạy ngầm mù thông tin". Đồng thời, thiếu việc đo đạc thông số hạ tầng phần cứng thực tế (RAM, CPU, GPU).

### 1.2 Ràng buộc sắt bắt buộc từ 2026-08-12
Mọi Agent thế hệ tương lai khi bổ sung hoặc chỉnh sửa bất kỳ tệp nào trong `examples/`, `tests/`, `scripts/`:

1. **Yield Live Output Tức Thì (Immediate Live Yield)**:
   - Trong các vòng lặp xử lý (mining, search, benchmarks), BẮT BUỘC phải in trực tiếp kết quả trung gian theo thời gian thực ra màn hình ngay lập tức (dùng `println!`, `stdout().flush()`) định kỳ từng 1,000 mẫu hoặc mỗi 300ms.
   - **NGHIÊM CẤM** im lặng chờ đến khi chạy xong toàn bộ batch mới xuất thông tin!

2. **Bắt buộc In Thông Số Telemetry Hạ Tầng (Realtime Telemetry Monitoring)**:
   - BẮT BUỘC phải in và báo cáo đủ 3 chỉ số phần cứng thực tế:
     - **RAM RSS Memory Usage**: Dung lượng RAM chiếm dụng (MB).
     - **CPU Utilization / Threads**: Số luồng CPU / Tải CPU %.
     - **GPU Compute Load / VRAM**: Tải GPU % và dung lượng VRAM (MB).

3. **Kỷ luật trung thực 100%**:
   - **NGHIÊM CẤM** báo cáo khống, tự bịa số liệu NPS/Throughput chưa qua đo đạc thực tế từ log execution!

---

## 2. KẾ THỪA BẢNG MỤC LỤC KÝ ỨC VĨNH CỬU INDEX.MD
- Tệp này đã được đăng ký vào tệp Bảng Mục Lục [`INDEX.md`](file://.agents/memory/INDEX.md).
