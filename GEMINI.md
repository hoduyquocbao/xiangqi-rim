# GEMINI.md — QUY TẮC BỔ SUNG DÀNH RIÊNG CHO GEMINI (ANTIGRAVITY)
# Phiên bản: 1.0.0 | Ngày tạo: 2026-08-08 | Tác giả: HDQB
# Phạm vi: Bổ sung bên cạnh AGENTS.md cho dự án Xiangqi-RIM

---

## I. ƯU TIÊN ĐỌC QUY TẮC

Trước khi thực hiện bất kỳ hành động nào, Gemini BẮT BUỘC phải đọc theo thứ tự:
1. `AGENTS.md` — Quy tắc dự án Xiangqi-RIM (kiến trúc NNUE, binary format, CPU cache, data mining)
2. `GEMINI.md` — Tệp này — quy tắc bổ sung cho Gemini
3. `.agents/memory/INDEX.md` — Bảng mục lục ký ức vĩnh cửu
4. `.agents/memory/pain_points_*.md` — Bài học xương máu (tất cả các tệp)
5. `.agents/memory/jrcp_2_0_spec.md` — Đặc tả JRCP 2.0

---

## II. THINKING PROTOCOL — QUY TRÌNH TƯ DUY BẮT BUỘC CHO XIANGQI-RIM

### 2.1 Checklist Tư Duy Trước Mỗi Hành Động

**Bước 1 — Xác định Module:**
- Yêu cầu này ảnh hưởng module nào? (board, movegen, search, eval, book, learn, selfplay, uci, web)
- Có module nào khác bị ảnh hưởng gián tiếp không? (ví dụ: sửa eval → ảnh hưởng search → ảnh hưởng benchmark)

**Bước 2 — Kiểm tra Ràng buộc Binary Format:**
- Nếu sửa NNUE: scale factors có thay đổi không? Nếu có → PHẢI cập nhật CẢ `src/learn/nnue.rs::quantize` VÀ Python notebooks đồng thời.
- Binary format XRNN v1 có bị phá vỡ không? → Chạy `python3 scripts/test_quantization.py`

**Bước 3 — Kiểm tra Performance:**
- TT Hash Table size có phù hợp với L3 cache (6MB) không?
- Có allocation động trong hot loop không?
- Có lock contention trên Mutex/Atomic không?

**Bước 4 — Kiểm tra Data Consistency:**
- Field name dùng `score` hay `eval`? (PHẢI là `score`)
- FEN format có đúng chuẩn 10 hàng, 2 vua không?
- UCI move format có đúng 4 ký tự `[a-i][0-9]` không?

### 2.2 Ma Trận Trọng Số Quyết Định — Đặc Thù Xiangqi-RIM

| Tiêu chí | Trọng số | Ví dụ |
|---|---|---|
| **Backward Compatibility** | ×3 | Sửa binary format phá vỡ tất cả weights cũ |
| **Performance Impact** | ×2 | Cache miss, false sharing, allocation trong hot loop |
| **Data Integrity** | ×3 | Sai field name, FEN không hợp lệ → training garbage |
| **Security** | ×4 | Token lộ = CRITICAL, phải fix ngay lập tức |
| **Benchmark Validity** | ×2 | < 200 ván hoặc depth < 5 = không có ý nghĩa thống kê |

---

## III. QUY TẮC ĐẶC THÙ XIANGQI-RIM CHO GEMINI

### 3.1 Rust Code — Engine Core

Khi viết hoặc sửa mã Rust trong `src/`:
- **BẮT BUỘC** `cargo check --release --examples` sau mỗi thay đổi.
- **BẮT BUỘC** `cargo test` nếu sửa logic movegen, eval, search.
- **NGHIÊM CẤM** thay đổi `src/eval/nnue.rs::load()` hoặc `src/learn/nnue.rs::quantize()` mà không cập nhật đồng thời Python notebooks và chạy `test_quantization.py`.
- **Ưu tiên** `#[inline(always)]` cho các hàm trong search hot path.
- **Ưu tiên** `&[T]` slice thay vì `Vec<T>` cho tham số hàm read-only.

### 3.2 Python Scripts — Training & Mining

Khi viết hoặc sửa Python scripts/notebooks:
- **BẮT BUỘC** dùng `os.environ.get('HF_TOKEN')` hoặc `google.colab.userdata.get()` — NGHIÊM CẤM hardcode token.
- **BẮT BUỘC** Train/Test Split 80/20 cho mọi training loop.
- **BẮT BUỘC** Early Stopping (patience ≥ 20 epochs).
- **NGHIÊM CẤM** `reward_funcs=[]` trong GRPO — tối thiểu 1 reward function.
- Field name JSONL output: `score` (KHÔNG phải `eval`).

### 3.3 Benchmark — Quy Tắc Thống Kê

| Mức độ | Số ván | Depth | Mục đích |
|---|---|---|---|
| Smoke Test | 20-40 | 3-4 | Kiểm tra nhanh, không báo cáo ELO |
| Development | 100 | 4 | Đánh giá sơ bộ, margin ±70 ELO |
| **Production** | **≥ 200** | **≥ 5** | **Báo cáo chính thức, margin ≤ ±50 ELO** |
| Tournament | 500-1000 | 5-6 | So sánh thế hệ, margin ≤ ±25 ELO |

**Kết quả benchmark Gen 5 (200 ván, depth 5):**
- NNUE vs HCE: W=12, L=13, D=175 → **Elo: -2 ±48**
- Kết luận: NNUE Gen 5 (train 90K mẫu depth 4) **KHÔNG tổng quát hóa lên depth 5**
- Nguyên nhân: Overfitting trên 90K mẫu depth 4
- Giải pháp: Scale data lên 500K mẫu, bao gồm depth 4+5 hỗn hợp

### 3.4 Data Scale — Quy Tắc Sinh Dữ Liệu

| Thế hệ | Số mẫu | Depth | Opening | Kết quả |
|---|---|---|---|---|
| Gen 1-4 | ~50K | 3 | Random 6 | Baseline |
| Gen 5 | 90K | 4 | Random 6 | +17 ELO (depth 3), -2 ELO (depth 5) |
| **Gen 6** | **500K** | **4-5 hỗn hợp** | **50% Book + 50% Random** | **Mục tiêu: ≥ +20 ELO depth 5** |

Lệnh mining Gen 6:
```bash
GAMES=10000 DEPTH=4 THREADS=4 cargo run --release --example 20_parallel_mine
```
- 10000 ván × ~50 mẫu/ván = ~500K mẫu
- THREADS=4 (physical cores) cho i5-8259U
- Output: `data/selfplay_samples_gen5.jsonl` (rename sau mining)

---

## IV. CPU CACHE OPTIMIZATION — ĐẶC THÙ i5-8259U

### 4.1 Thông Số Phần Cứng

| Parameter | Value | Ảnh hưởng |
|---|---|---|
| Physical Cores | 4 | THREADS mặc định = 4 |
| Logical Cores | 8 (HT) | HT chỉ +15-20% cho compute-bound |
| L1D Cache | 32 KB/core | Hidden/Output NNUE weights (16KB) fit hoàn toàn |
| L2 Cache | 256 KB/core | Position struct + movegen fit |
| L3 Cache | 6 MB (shared) | TT Hash ≤ 4MB để fit |
| Cache Line | 64 bytes | Align shared atomics |

### 4.2 Quy Tắc Cache Friendly cho Mining

1. **Search::new(4)** — TT 4MB fit gần L3 (6MB shared). NGHIÊM CẤM ≥ 8MB.
2. **Per-thread local buffer** — `Vec<String>` cục bộ, batch write cuối ván. NGHIÊM CẤM lock Mutex mỗi sample.
3. **Batch atomic update** — `fetch_add(batch_size)` cuối ván. NGHIÊM CẤM `fetch_add(1)` mỗi sample.
4. **THREADS=4** mặc định cho compute-bound. THREADS=8 chỉ cho I/O-bound workload.

### 4.3 Hiệu Năng Tham Chiếu

| Cấu hình | Throughput | Ghi chú |
|---|---|---|
| THREADS=8, Search(8) | 0.4 ván/s | Hiện tại (baseline) |
| THREADS=4, Search(4), batch write | ~0.6 ván/s | Mục tiêu tối ưu (+50%) |

---

## V. LỖ HỔNG ĐÃ PHÁT HIỆN & CHƯA PHÁT HIỆN

### 5.1 Đã Phát Hiện & Đã Sửa (Commit 7329596, f3f32ce, ead29fe)

| # | Lỗ hổng | Mức | Đã sửa |
|---|---|---|---|
| 1 | HF Token hardcoded trong scripts | CRITICAL | ✅ |
| 2 | `reward_funcs=[]` trong GRPO | CRITICAL | ✅ |
| 3 | Không có Validation Gateway cho data cộng đồng | HIGH | ✅ |
| 4 | Không có Train/Test Split | HIGH | ✅ |
| 5 | Field name `eval`/`score` không thống nhất | MEDIUM | ✅ |
| 6 | Random opening thuần (không Book) | MEDIUM | ✅ |

### 5.2 Chưa Phát Hiện Bởi Agent Trước — Được Bổ Sung Trong AGENTS.md

| # | Lỗ hổng / Thiếu sót | Mức | Trạng thái |
|---|---|---|---|
| 7 | Không có AGENTS.md/GEMINI.md cho dự án | HIGH | ✅ Tạo mới |
| 8 | CPU cache miss: Search(8) > L3 cache | MEDIUM | ⏳ Sửa trong miner |
| 9 | Mutex contention: lock mỗi sample | MEDIUM | ⏳ Sửa trong miner |
| 10 | THREADS=8 mặc định trên 4-core CPU | LOW | ⏳ Sửa trong miner |
| 11 | Benchmark 40 ván depth 3 không đủ tiêu chuẩn | HIGH | ✅ Đã benchmark 200 ván depth 5 |
| 12 | NNUE Gen 5 overfitting (90K mẫu) | HIGH | ⏳ Scale 500K |
| 13 | Không có mixed-depth training data | MEDIUM | ⏳ Thêm depth 5 vào mining |
| 14 | Không có `.agents/workflows/` cho dự án | LOW | ⏳ Tạo sau |
| 15 | Không có `.agents/skills/` cho dự án | LOW | ⏳ Tạo sau |

---

---

## VII. QUY TẮC CỐT LÕI CHO GEMINI (ANTIGRAVITY): BẮT BUỘC TẠO FILE BÀI HỌC VÀ DIỄN GIẢI 100% TƯỜNG MINH

### 7.1 CẤM LÝ DO "CÓ TRONG CONTEXT WINDOW NÊN KHÔNG CẦN CẬP NHẬT FILE ĐĨA"
- Mọi suy luận, bài học xương máu, lỗi phát sinh, hoặc kết quả phiên làm việc **BẮT BUỘC phải được ghi ra tệp đĩa vật lý ngay lập tức**:
  - `.agents/memory/pain_points_[YYYYMMDD].md` (Ghi nhận bài học xương máu mới)
  - `.agents/memory/INDEX.md` (Đăng ký vào bảng mục lục ký ức vĩnh cửu)
  - `.agents/logs/session_active_[YYYYMMDD][HHMM].md` (Nhật ký phiên làm việc)
  - `.agents/logs/INDEX.md` (Đăng ký vào bảng mục lục nhật ký phiên)
- **CẤM** lý do "ngữ cảnh context window đã có rồi thì để sau mới ghi file" hoặc "file cũ đã có rồi nên không cần tạo mới".

### 7.2 BẮT BUỘC DIỄN GIẢI & CHÚ THÍCH TƯỜNG MINH (FORCED ELABORATION MANDATE)
- Mọi tệp hướng dẫn, quy tắc, chú thích, tài liệu và mã nguồn **PHẢI được diễn giải, chú thích, giải thích chi tiết tỉ mỉ tới từng dòng, từng thông số, từng trường dữ liệu bằng tiếng Việt 100%**.
- **Mục đích**: Đảm bảo một Agent thiểu năng / kém thông minh nhất khi đọc vào cũng buộc phải hiểu và thực hiện đúng 100%, không thể hiểu sai hay làm sai.
- **NGHIÊM CẤM** diễn giải mập mờ, CẤM viết tắt, CẤM giả định "người khác / agent khác tự hiểu".

### 7.3 KỶ LUẬT 5 THÀNH PHẦN BẮT BUỘC TRONG MỌI HÀNH ĐỘNG
1. **Kế Hoạch (Plan)**: Không bao giờ viết mã trong vô định. Phải khảo sát, nghiên cứu, lập kế hoạch 5 bước.
2. **Đánh Số Phiên Bản (Versioning)**: Đánh số phiên bản Semantic Versioning (`v1.0.0`, `v2.0.0`) cho mọi module/binary/spec.
3. **Telemetry**: Tự động nhận diện hạn mức hạ tầng cgroups (CPU quota, RAM limit).
4. **Logger**: Ghi nhật ký thực thi (log timestamps, stack trace, errors).
5. **Metrics**: Đo lường thông số định lượng thực tế (FEN/s, RAM RSS, CPU pct, file size).

### 7.4 QUY TRÌNH DỪNG LẠI 1 NHỊP QUAN SÁT THIẾU SÓT
> **"THÀ CHẬM MỘT NHỊP QUAN SÁT THIẾU SÓT CÒN HƠN CẨU THẢ, AI SLOP BIẾN CHẤT, LƯỜI BIẾNG, TÓM TẮT CẮT XÉN, LÀM CHO CÓ LỆ, LÀM ĐỂ ĐỐI PHÓ!"**
