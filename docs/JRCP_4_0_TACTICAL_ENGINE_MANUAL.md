# 📜 TÀI LIỆU KIẾN TRÚC & CẨM NANG PHÁT TRIỂN ENGINE XIANGQI-R1 JRCP 4.0
## (v10.0-JRCP4-TACTICAL-28D — High-Performance 28-Dimensional Tactical Engine)

---

## I. TỔNG QUAN HỆ THỐNG & TRIẾT LÝ THIẾT KẾ

Engine `gpu_t4_real_rule_miner.py` là bộ sinh dữ liệu huấn luyện (Training Data Mining Engine) thế hệ mới nhất dành cho mô hình AI Cờ Tướng **Xiangqi-R1**. 

### 1. Triết lý "Chiều Sâu Chiến Thuật Chiều Sâu" (Deep Tactical Consciousness)
Các mô hình AI cờ truyền thống thường gặp lỗi "ảo giác" (hallucination) hoặc chọn nước đi ngây thơ do thought chain chỉ dừng lại ở việc mô tả bề mặt (có quân gì trên bàn cờ). **JRCP 4.0** nâng cấp thought chain lên **28 chiều kích chiều sâu chiến thuật**, giúp ngay cả các mô hình nhỏ (Small Language Models / SLMs) hoặc Agent kém thông minh nhất cũng nhìn thấy:
- Vị trí trực quan của toàn bộ bàn cờ (ASCII 2D với tọa độ & ký tự Hán tự).
- Danh sách quân treo (Hanging pieces - quân bị tấn công không được bảo vệ).
- Các đòn chiến thuật nguy hiểm: Đòn kép (Fork), Đòn mở (Discovered attack), Ghim quân (Pin).
- Các bẫy ăn quân có chủ đích và nguy cơ chiếu bí trong N nước.
- 18/36 Kế Binh Pháp (Tôn Tử / Gia Cát Lượng) và 7 thế trận kinh điển.

### 2. Hạ Tầng 0₫ & Tối Ưu Hiệu Năng GPU Mega-Batching
- **64 Ván Cờ Song Song (64-Parallel Slots)**: Thay vì chạy tuần tự từng ván (lãng phí GPU), engine chạy đồng thời 64 ván cờ trên CPU.
- **GPU Mega-Batch Evaluation**: Gom toàn bộ nước đi hợp lệ của 64 ván (từ 2,000 đến 4,000 vị trí FEN) thành 1 Tensor Mega-Batch duy nhất và gửi lên GPU T4 đánh giá trong 1 chu kỳ xung nhịp (~15-20ms).
- **Tối ưu VRAM & FP16**: Sử dụng PyTorch `amp.autocast('cuda')` và mạng Nơ-ron Deep Residual 4-Block (5M parameters) chỉ tiêu tốn 2-4GB VRAM.

---

## II. SƠ ĐỒ LUỒNG DỮ LIỆU (DATAFLOW ARCHITECTURE)

```
┌────────────────────────────────────────────────────────────────────────┐
│                        64-PARALLEL SLOT WORKER                         │
│  [Slot 01] [Slot 02] [Slot 03] ... [Slot 64] (Chạy song song trên RAM)  │
└───────────────────┬────────────────────────────────────────────────────┘
                    │ Gom tất cả Legal Moves của 64 ván cờ
                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                     GPU MEGA-BATCH EVALUATOR                           │
│  Gom 2,000 - 4,000 FEN Tensors  ──> PyTorch FP16 Conv1D ResNet Engine   │
│  Trả về mảng điểm số Centipawn trong 15ms                              │
└───────────────────┬────────────────────────────────────────────────────┘
                    │ Phân phối điểm số về từng Slot
                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                   JRCP 4.0 THOUGHT CHAIN GENERATOR                     │
│  Trích xuất 28 Chiều Kích Chiến Thuật (ASCII, Pin, Fork, 36 Kế...)     │
└───────────────────┬────────────────────────────────────────────────────┘
                    │ Kiểm tra hợp lệ dữ liệu
                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                   STRICT DATA VALIDATOR FIREWALL                       │
│  Xác minh 100% Luật cờ vật lý + Định dạng UCI + Đủ 28/28 Thought Tags  │
└───────────────────┬────────────────────────────────────────────────────┘
                    │ Lọc trùng lặp FEN bằng Bloom Sieve
                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                      LOCAL DISK & HF AUTO-PUSH                         │
│  - Ghi file JSONL theo từng chunk 50MB (Zero-Crash Buffer)             │
│  - Auto-Push lên HuggingFace Hub mỗi 5 phút hoặc khi đẩy đủ Chunk      │
└────────────────────────────────────────────────────────────────────────┘
```

---

## III. MA TRẬN 28 CHIỀU KÍCH JRCP 4.0 CHI TIẾT

Hệ thống chuỗi suy tưởng `<thought>` được cấu trúc nghiêm ngặt thành 5 nhóm chiều kích:

### Nhóm I: Nhận Thức Bàn Cờ (Chiều 1 -> 6)
- **[1/28] KIỂM KÊ QUÂN CỜ (`inventory`)**: Trích xuất tọa độ chính xác của từng quân Đỏ & Đen (`Xe (a0), Pháo (b2)...`).
- **[2/28] BÀN CỜ 2D VĂN BẢN (`ascii`)**: Vẽ bàn cờ 10x9 hiển thị sông Ngân Hà, Cung Tướng, các quân bằng chữ Hán (`帥, 仕, 相, 馬, 車, 炮, 兵 / 將, 士, 象, 馬, 車, 砲, 卒`).
- **[3/28] TƯƠNG QUAN VẬT CHẤT CHI TIẾT (`material`)**: Điểm vật chất chi tiết từng loại quân (Xe=90, Pháo=45, Mã=40, Sĩ=20, Tượng=20, Tốt=10).
- **[4/28] PHÂN TÍCH 9 LỘ (`columns`)**: Đánh giá trạng thái từng cột từ a -> i (MỞ, BÁN MỞ, KHÓA, bên nào đang chiếm lộ).
- **[5/28] MỨC ĐỘ TRIỂN KHAI QUÂN (`deployed`)**: Đếm số quân đã rời vị trí ban đầu và danh sách các quân chưa xuất kích.
- **[6/28] ĐỘ LINH HOẠT / MOBILITY (`mobility`)**: Đếm số nước đi hợp lệ khả thi của cả 2 bên.

### Nhóm II: Phân Tích Đe Dọa (Chiều 7 -> 12)
- **[7/28] AN TOÀN TƯỚNG (`safety`)**: Phân tích tình trạng Cung Tướng, số lượng Sĩ/Tượng còn lại, các quân địch đang đe dọa trực tiếp.
- **[8/28] QUÂN BỊ TẤN CÔNG (`attacked`)**: Phát hiện danh sách các quân cờ đang nằm trong tầm ngắm của đối phương.
- **[9/28] QUÂN TREO / HANGING PIECES (`hanging`)**: Xác định các quân bị tấn công mà **KHÔNG CÓ QUÂN BẢO VỆ** (mục tiêu ăn miễn phí).
- **[10/28] QUÂN BỊ GHIM / PIN (`pinned`)**: Phát hiện các quân bị Xe/Pháo ghim chặt, không thể di chuyển vì sẽ làm lộ Tướng.
- **[11/28] ĐÒN KÉP / FORK (`forks`)**: Phát hiện tình huống 1 quân đe dọa 2 hoặc nhiều quân giá trị cao cùng lúc.
- **[12/28] ĐÒN MỞ / DISCOVERED ATTACK (`discovered`)**: Phát hiện nước di chuyển quân phía trước để mở đường cho Xe/Pháo phía sau chiếu Tướng.

### Nhóm III: Chiến Thuật & Bẫy (Chiều 13 -> 18)
- **[13/28] BẪY ĂN QUÂN (`traps`)**: Phát hiện các mồi nhử đổi quân có lời centipawn hoặc ăn không bị phản đòn.
- **[14/28] CHIẾU BÍ TIỀM ẨN (`checkmate`)**: Phát hiện chuỗi nước đi chiếu sát thủ dẫn tới chiến thắng ngay lập tức.
- **[15/28] DƯƠNG ĐÔNG KÍCH TÂY (`diversion`)**: Phân tích xem nước đi có phải đòn nghi binh (chuyển cánh >3 cột) hay không.
- **[16/28] MẪU CHIẾN THUẬT NÂNG CẤP (`patterns`)**: Nhận diện 15+ mẫu cờ như Pháo Đầu, Mã vượt hà, Tốt qua sông, Song Xe lực chiến, mất Sĩ/Tượng.
- **[17/28] PHỐI HỢP QUÂN (`synergy`)**: Nhận diện sự phối hợp bộ đôi: Song Xe trùng lộ/hàng, Xe-Pháo trùng lộ, Mã-Pháo giao chiến.
- **[18/28] ĐIỂM YẾU CẤU TRÚC (`weakness`)**: Phát hiện Tốt cô lập, Tốt đôi, Cung Tướng trống rỗng.

### Nhóm IV: 36 Kế Binh Pháp & Thế Trận (Chiều 19 -> 22)
- **[19/28] 36 Kế BINH PHÁP ÁP DỤNG (`stratagems`)**: Ánh xạ bàn cờ với 18 kế kinh điển (Vây Ngụy Cứu Triệu, Tá Đao Sát Nhân, Phủ Để Trừu Tân, Điệu Hổ Ly Sơn...).
- **[20/28] THẾ TRẬN KINH ĐIỂN (`formation`)**: Phát hiện các trận hình kinh điển (Pháo Đầu, Bình Phong Mã, Đơn Đề Mã, Quá Cung Pháo, Tiên Phong Xe, Song Phi Tượng, Tam Tử Kinh).
- **[21/28] GIAI ĐOẠN & CHIẾN LƯỢC**: Xác định chính xác 5 giai đoạn: Khai cuộc, Đầu trung cuộc, Trung cuộc, Cuối trung cuộc, Tàn cuộc.
- **[22/28] TEMPO & SÁNG KIẾN (`tempo`)**: Đánh giá bên nào đang làm chủ nhịp độ, buộc đối phương phải bị động chống đỡ.

### Nhóm V: Đánh Giá & Quyết Định (Chiều 23 -> 28)
- **[23/28] ƯU THẾ TỔNG HỢP**: Tổng hợp các ưu thế vật chất và không gian.
- **[24/28] BẤT LỢI TỔNG HỢP**: Tổng hợp các rủi ro và bất lợi.
- **[25/28] ĐÁNH GIÁ CANDIDATES**: Liệt kê 3-5 nước đi ứng viên tốt nhất kèm mô tả hành động.
- **[26/28] SO SÁNH & CHỌN BESTMOVE**: Giải thích lý do chọn nước đi tối ưu nhất.
- **[27/28] CENTIPAWN TỔNG HỢP**: Điểm số đánh giá cuối cùng (ví dụ: `150cp`).
- **[28/28] XÁC MINH**: Kiểm tra định dạng UCI regex và tính hợp lệ vật lý.

---

## IV. HƯỚNG DẪN DÀNH CHO CÁC THẾ HỆ AGENT (DEVELOPER MAINTENANCE GUIDE)

### 1. Nguyên tắc Đơn Từ (Single-Word Identifier Rules)
Mọi tên hàm, tên biến, tên tham số mới thêm vào `gpu_t4_real_rule_miner.py` **BẮT BUỘC** là một từ đơn tiếng Anh (ví dụ: `columns`, `deployed`, `mobility`, `attacked`, `hanging`, `pinned`, `forks`, `discovered`, `traps`, `synergy`, `weakness`, `stratagems`, `formation`, `tempo`).

### 2. Cách Chỉnh Sửa Hoặc Thêm Chiều Kích Mới
Nếu cần nâng cấp lên JRCP 5.0 (ví dụ 32 chiều kích):
1. Thêm hàm phân tích mới vào `class Board`.
2. Cập nhật `make_sample()` để bổ sung tag `[29/32]`, `[30/32]`... vào chuỗi `<thought>`.
3. Đảm bảo cập nhật vòng lặp kiểm tra trong `DataValidator.validate_sample()`:
   ```python
   for i in range(1, 33):
       if f"[{i}/32]" not in thought:
           return False, f"MISSING_THOUGHT_TAG_{i}"
   ```
4. Cập nhật `SYSTEM_PROMPT` và tài liệu này.

### 3. Quy Trình Kiểm Thử Bắt Buộc Khi Sửa Code
Trước khi bàn giao hoặc đẩy code lên sản xuất, bắt buộc phải chạy lệnh kiểm thử AST và Dry-Run:
```bash
# 1. Kiểm tra cú pháp AST
python3 -c "import ast; ast.parse(open('gpu_t4_real_rule_miner.py').read()); print('AST OK')"

# 2. Kiểm tra bộ 6 Checkpoint physical unit tests và 28 thought tags
python3 -c "
from gpu_t4_real_rule_miner import Board, make_sample, DataValidator
b = Board()
b.parse('r1bakab1r/9/1cn3nc1/p1p1p1p1p/9/9/P1P1P1P1P/1CN1C4/9/R1BAKABNR w - - 0 1')
legal = b.legal()
sample, thought = make_sample(b, legal[0].encode(), 50, legal, 0, 12)
valid, reason = DataValidator.validate_sample(b, legal[0].encode(), 50, thought)
print('VALIDATION:', valid, reason)
"
```

---
*Tài liệu được khởi tạo và bảo trì bởi HDQB & Antigravity Agent — Version 10.0.*
