# BÀI HỌC XƯƠNG MÁU ĐẮT GIÁ NĂNG LƯỢNG NGÔN NGỮ & KỸ THUẬT (2026-08-10)
# PHIÊN BẢN HỆ THỐNG: v11.1-JRCP5-BULLETPROOF | NGÀY: 2026-08-10 | TÁC GIẢ: HDQB & ANTIGRAVITY AGENT

---

## BÀI HỌC 1: TRIỆT TIÊU THIÊN KIẾN & ẢO GIÁC FEN KHAI CUỘC TRONG SELF-PLAY MINER

### 1. Bối Cảnh & Sai Lầm Ngây Thơ:
Trong phiên bản JRCP 5.0 v11.0, Agent đã tự ý đề xuất mảng `OPENING_FENS` gồm 6 FEN khai cuộc được viết từ trí nhớ/ảo giác AI mà **KHÔNG CHẠY KIỂM ĐỊNH VẬT LÝ VỚI BOARD ENGINE**. Khi anh HDQB yêu cầu kiểm tra tính thực tế, kết quả mổ xẻ phát hiện **cả 6/6 FEN ĐỀU LÀ ẢO GIÁC NGHỆCH LÝ VẬT LÝ**:
- FEN 2 ("Thuận Pháo"): Hàng Đen có 3 Pháo (`1c4cc1`), Đỏ có 3 Pháo (`1C4CC1`), tổng cộng 17 quân/bên! Pháo bị AI tự nhân bản thêm.
- FEN 4 ("Tiến Binh 3"): Lỗi cú pháp FEN — Hàng 3 chỉ có 8 ô (`P1P1P1P1`), thiếu 1 Binh `i3`.
- FEN 5 ("Đơn Đề Mã"): Mã Đỏ ở `c1` — Nước nhảy `b0` -> `c1` (delta 1x1) vi phạm luật nhảy chữ L (delta 1x2).
- Vi phạm luân phiên: Đỏ/Đen đi số nước không bằng nhau nhưng `side-to-move` lại sai quy luật.

### 2. Nguyên Nhân Gốc Rễ:
Agent cẩu thả, tự tin ngây thơ vào khả năng tự sinh chuỗi FEN mà không dùng công cụ thực thi kiểm tra `Board.apply()`. Đây là hành vi vi phạm nghiêm trọng MASTER INSTRUCTION ("Không đoán mò code logic hay schemas, bắt buộc kiểm định vật lý thực tế 100%").

### 3. Quy Tắc Phòng Ngự Thép (Bắt Buộc Cho Các Thế Hệ Agent Sau):
1. **NGHIÊM CẤM** viết tay hoặc tự bịa chuỗi FEN trong mã nguồn.
2. **BẮT BUỘC** 100% FEN khai cuộc phải được sinh tự động bằng cách gọi `Board.apply()` với các nước đi đại số UCI hợp lệ từ `START_FEN`.
3. **BẮT BUỘC** chạy script kiểm tra `DataValidator`: Đảm bảo đủ 16 quân/bên, 90 ô, `side-to-move` chính xác, có nước đi hợp lệ tiếp theo > 30.

---

## BÀI HỌC 2: XUNG ĐỘT C++ REGISTRATION TRONG PYTORCH COLAB PYTHON 3.12 (`aten.linspace`)

### 1. Bối Cảnh & Sai Lầm Ngây Thơ:
Khi khởi chạy Notebook `colab_gpu_depth12_miner.ipynb` trên Google Colab Python 3.12, cell bị đứng và ném ra lỗi:
`RuntimeError: duplicate registrations for aten.linspace.Tensor_Tensor` ngay tại dòng `import torch`.

Ban đầu, Agent xử lý ngây thơ bằng cách sửa `except ImportError:` thành `except Exception:` hoặc hướng dẫn người dùng Restart Session, nhưng lỗi vẫn lặp lại vì `import torch` ở top-level của Notebook Cell bị sập C++ Dispatcher do bug nội bộ của PyTorch 2.4/2.5 trên Python 3.12 Colab.

### 2. Nguyên Nhân Gốc Rễ:
- Nạp `import torch` bừa bãi ở top-level của Notebook Cell trong môi trường Colab đã cài sẵn PyTorch CUDA C++ bindings.
- Engine cờ Tướng JRCP 5.0 (32D) về bản chất là **Pure Python Rule Engine** (chạy bằng `Board`, `Move`, `OPENING_FENS`, `Sieve`, `Buffer`). Việc đưa `import torch` vào top-level là sự phụ thuộc dư thừa không cần thiết, làm toàn bộ Notebook bị văng lỗi theo bug PyTorch của Colab.

### 3. Quy Tắc Phòng Ngự Thép (Bắt Buộc Cho Các Thế Hệ Agent Sau):
1. **NGHIÊM CẤM** đưa `import torch` vào top-level của Notebook Cell hoặc script mining nếu thuật toán không bắt buộc dùng PyTorch C++ Dispatcher.
2. **BẮT BUỘC** cô lập PyTorch vào khối Pure Python Fallback (Zero Dependency Mode). Nếu `torch` không khả dụng hoặc bị lỗi registration, miner engine tự động kích hoạt Pure Python Engine để chạy liên tục 100% mà không bị sập.
3. **BẮT BUỘC** bọc `except BaseException:` (chứ không chỉ `except Exception:`) khi thực hiện bất kỳ import thử nghiệm nào từ thư viện C++ extensions bên ngoài.

---

## BÀI HỌC 3: QUY TRÌNH BÀN GIAO TRẠNG THÁI & XOAY VÒNG NHẬT KÝ KHÔNG ĐỨT GÃY

### 1. Bối Cảnh:
Khi thực hiện sửa lỗi hoặc cập nhật phiên bản, Agent phải cập nhật đồng bộ 3 nơi:
1. Mã nguồn Python/Rust/Notebook (`gpu_t4_real_rule_miner.py`, `colab_gpu_depth12_miner.ipynb`, `scripts/deploy_nnue_dataset.py`).
2. Tệp Bài học Xương máu (`.agents/memory/pain_points_[YYYYMMDD].md`) & Bảng mục lục `INDEX.md`.
3. Nhật ký phiên hoạt động (`.agents/logs/session_active_[YYYYMMDD][HHMM].md`).

### 2. Quy Tắc Phòng Ngự Thép:
- Mỗi khi hoàn thành một sửa đổi lớn hoặc xử lý lỗi bài học, BẮT BUỘC tạo tệp `pain_points_[YYYYMMDD].md` mới và cập nhật `INDEX.md`.
- NGHIÊM CẤM kết thúc phiên mà chưa ghi nhận bài học xương máu.
