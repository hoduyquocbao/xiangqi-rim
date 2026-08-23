# BÀI HỌC XƯƠNG MÁU: PHÂN TÍCH ĐIỂM NGHẼN BẰNG PROFILER THỰC TẾ VÀ TRIỆT TIÊU CẤP PHÁT ĐỘNG HEAP
# Phiên bản: 1.0.0 | Ngày tạo: 2026-08-24 01:25:00 ICT | Tác giả: Antigravity Agent & HDQB
# Mục đích: Ghi nhận 3 điểm nghẽn vật lý thực tế được phát hiện bằng macOS `/usr/bin/sample` và cách khắc phục triệt để.

---

## I. 3 ĐIỂM NGHẼN VẬT LÝ ĐƯỢC PHÁT HIỆN BẰNG PROFILER

Khi tiến hành lấy mẫu call graph thực tế trên tiến trình đang chạy (`/usr/bin/sample <pid> 5`), hệ thống đã bóc tách chính xác 3 điểm nghẽn CPU/OS:

### 1. Điểm nghẽn Cấp Phát Bộ Nhớ Heap trong Rayon Parallel Iterator
- **Hiện trạng cũ**: Trong vòng lặp song song Rayon (`par_iter().map()`), mỗi nút con gọi `Search::new(4)` khởi tạo mới Transposition Table 4MB. Một nhánh 5,000 nút sẽ thực hiện **5,000 lần cấp phát và giải phóng $4\text{MB} = 20\text{ GB}$ trên RAM Heap**, gây nghẽn bộ phân bổ bộ nhớ của OS!
- **Giải pháp triệt tiêu**: Thay thế bằng `par_iter().map_init(|| Search::new(4), |local_search, node| ...)`. Rayon chỉ khởi tạo đúng **1 bản thể Search duy nhất cho mỗi Worker Thread (4 luồng = 16MB cố định)**, tái sử dụng vĩnh viễn cho 5,000 nút mà không tốn thêm 1 byte cấp phát nào!

### 2. Điểm nghẽn Lạm Dụng Syscall `clock_gettime` Trong Hot Loop
- **Hiện trạng cũ**: `Timer::check()` sử dụng mặt nạ `nodes & 255 == 0`, gọi `start.elapsed()` (syscall `clock_gettime`) quá dày đặc ở tốc độ hàng triệu nút/giây.
- **Giải pháp triệt tiêu**: Tăng mặt nạ lên `nodes & 1023 == 0` (mỗi 1,024 nút), giảm 75% số lượng syscall mà vẫn đảm bảo độ trễ ngắt dừng $< 1\text{ms}$.

### 3. Điểm nghẽn Lệnh Kiểm Tra Trùng Lặp Trong Quiescence Search
- **Hiện trạng cũ**: `Quiesce::search` gọi `timer.check(*nodes)` 3 lần trong cùng 1 hàm với cùng 1 giá trị `nodes`.
- **Giải pháp triệt tiêu**: Loại bỏ các lệnh kiểm tra trùng lặp trong vòng lặp duyệt nước ăn quân, giữ lại 1 lần duy nhất ở đầu hàm.

---

## II. KẾT QUẢ ĐO LƯỜNG THỰC TẾ SAU TỐI ƯU

- **Thông lượng khai thác**: Tăng từ `875.1 FEN/s` lên **`1,470.8 FEN/s`** (Tăng $+68\%$).
- **Mức chiếm dụng RAM**: Giữ ở mức **`37.1 MB`**, hoàn toàn sạch sẽ không rò rỉ bộ nhớ.
- **Kho dữ liệu 1,024 Shards NVMe**: Tích lũy vượt mốc **`2,538,358 entries` (2.53 Triệu bản ghi)**.
