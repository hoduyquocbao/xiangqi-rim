# BẢN GHI BÀI HỌC XƯƠNG MÁU & RỦI RO ĐẮT GIÁ HỆ THỐNG XIANGRUST ENGINE
# Ngày ghi nhận: 2026-08-07 | Tác giả: Antigravity Agent (Gemini)

---

## 1. LỖI BÙ MÙ CHIẾN THUẬT DO BẠO BỆNH BETA CUTOFF TẠI NÚT GỐC (Aspiration Window Fail-High at Root)
- **Triệu chứng**: Khi chạy ở `Depth = 12` hoặc các độ sâu cao có sử dụng cửa sổ Aspiration Window `[alpha, beta]`, Engine trả về `BestMove: 0000` hoặc bị kẹt lặp lại tính toán.
- **Nguyên nhân gốc rễ**: Khi nước đi đầu tiên tại `ply = 0` tạo ra điểm số $\ge \beta$, hàm `Core::pvs` lập tức cắt tỉa và trả về `beta` ở dòng 317 mà **không chạy xuống dòng 325** (`stack[0].pv.update`). Do đó `stack[0].pv.len` vẫn bằng 0. Khi hết giờ (`2000ms`), `Core::iterate` không đọc được nước đi mới từ `stack[0].pv` và trả về `Move::none()` ("0000").
- **Bài học khắc phục**:
  1. Tại nút gốc `ply == 0`, bắt buộc phải gọi `stack[ply].pv.update(mv, &child)` ngay khi xảy ra Beta Cutoff trước khi `return beta`.
  2. Kiểm tra tính hợp lệ `stack[0].pv.items[0].valid()` trong `Core::iterate` để giữ nguyên nước đi tốt nhất từ các độ sâu đã hoàn thành trước đó (ví dụ Depth 11, 10...), tuyệt đối không ghi đè thành `"0000"`.

---

## 2. LỖI Ô NHIỄM BẢNG BĂM KHI TÌM KIẾM BỊ NGẮT DỞ DANG (Transposition Table Abort Pollution)
- **Triệu chứng**: Điểm số đánh giá ở các nước đi sau bị trượt hoặc thay đổi bất thường sau khi phiên tìm kiếm trước đó bị ngắt bởi `TimeLimit`.
- **Nguyên nhân gốc rễ**: Khi `timer.check()` phát hiện hết giờ và gán `abort = true`, một số nhánh cờ chưa duyệt xong ở độ sâu dở dang vẫn bị lưu vào Transposition Table qua `table.save_with`.
- **Bài học khắc phục**: Bọc cờ `!timer.abort.load(Ordering::Relaxed)` xung quanh tất cả các lệnh `table.save_with` trong `Core::pvs`. Khi phiên bị ngắt, hủy toàn bộ quyền ghi TT dở dang.

---

## 3. KHỐNG CHẾ THỜI GIAN HAI TẦNG: SOFT LIMIT VS HARD LIMIT
- **Bài học**: Nếu chỉ dùng 1 mốc Hard Limit (2000ms), Engine có thể bắt đầu một độ sâu mới (ví dụ Depth 13) ở mốc 1950ms và bị ngắt chỉ sau 30ms, làm lãng phí tài nguyên CPU mà không đem lại kết quả.
- **Giải pháp**: Triển khai **Soft Limit ($1,400\text{ms}$ = 70% max time)** để ngăn không cho Engine khởi chạy độ sâu mới nếu thời gian còn lại không đủ, và **Hard Limit ($1,980\text{ms}$)** để dừng an toàn phiên đang chạy.

---

## 4. PHÂN TÁCH PHIÊN CÁCH LY WEBSOCKET CLIENT & TÁI SỬ DỤNG TT PER CONNECTION
- **Bài học**: Nếu cấp phát mới `Search::new(mb)` ở mỗi frame WebSocket `search`, Engine vừa tốn công allocate 256MB RAM trên Heap ở mỗi nước đi, vừa làm mất toàn bộ cache Transposition Table từ nước đi trước.
- **Giải pháp**: Tạo phiên `session_search` nằm trong luồng kết nối WebSocket của từng Client. Mỗi Tab trình duyệt sở hữu 1 phiên `Search` độc lập, vừa cách ly hoàn toàn giữa các Tabs, vừa tái sử dụng 100% cache TT qua từng nước đi.

---

## 5. CHỐNG TRÀN BỘ NHỚ LOCALSTORAGE 5MB TRÊN TRÌNH DUYỆT BẰNG INDEXEDDB
- **Bài học**: `localStorage` trình duyệt chỉ chứa tối đa 5MB. Khi lưu trữ mảng mẫu kinh nghiệm AI `dataset`, sau vài ngàn ván cờ trình duyệt sẽ ném lỗi `QuotaExceededError` gây crash ứng dụng.
- **Giải pháp**: Triển khai module `web/src/storage/db.js` dựa trên chuẩn **IndexedDB API** cho phép lưu trữ kinh nghiệm nhị phân hàng trăm MB/GB không giới hạn, kết hợp với bộ đệm an toàn 1,000 phần tử trên `localStorage`.
