# AGENTS.md — QUY TẮC DỰ ÁN XIANGQI-RIM
# Phiên bản: 1.0.0 | Ngày tạo: 2026-08-08 | Tác giả: HDQB
# Phạm vi: Toàn bộ mã nguồn Rust Engine, Python Scripts, Notebooks, Web UI

---

## I. TỔNG QUAN DỰ ÁN

### 1.1 Mục Đích
Xiangqi-RIM là **Engine Cờ Tướng AI** hiệu năng cao viết bằng Rust, tích hợp:
- **NNUE (Efficiently Updatable Neural Network)**: Kiến trúc HalfKAv2_hm, 65536×256→512→32→1
- **Alpha-Beta Search**: Negamax + Aspiration Window + PVS + LMR + Null Move + TT
- **Opening Book**: Zobrist Hash O(1) lookup ≥ 1000 biến thể khai cuộc
- **Endgame Knowledge**: Tri thức tàn cuộc chuyên sâu
- **Web UI**: Giao diện web chơi cờ trực tuyến (HTML/JS/CSS)
- **LLM Integration**: REST API tích hợp mô hình ngôn ngữ lớn Xiangqi-R1

### 1.2 Kiến Trúc Hệ Thống

```
┌─────────────────────────────────────────────────────┐
│                    WEB UI (JS/HTML)                  │
├─────────────────────────────────────────────────────┤
│              REST API / WebSocket Server             │
├──────────────┬──────────────┬────────────────────────┤
│  Board/FEN   │   Search     │      Evaluation        │
│  Parser      │   Engine     │  ┌──────┬──────────┐   │
│  MoveGen     │   TT Hash    │  │ NNUE │   HCE    │   │
│  Legal       │   PVS/LMR    │  │(SIMD)│(Fallback)│   │
│  Zobrist     │   NullMove   │  └──────┴──────────┘   │
├──────────────┼──────────────┼────────────────────────┤
│  Opening     │  Endgame     │   Learn/Training       │
│  Book        │  Knowledge   │   NNUE Trainer         │
│  (Zobrist)   │  (Rules)     │   Backprop + Quantize  │
└──────────────┴──────────────┴────────────────────────┘
```

---

## II. QUY TẮC MÃ NGUỒN RUST

### 2.1 Cấu Trúc Thư Mục Bắt Buộc

```
src/
  board/       — Position, Parser, Serializer, Zobrist Hash
  movegen/     — Legal move generation, Attack tables
  search/      — Alpha-Beta, TT, Limits, PVS, LMR
  eval/        — NNUE (nnue.rs, weight.rs, accum.rs, feature.rs), HCE
  book/        — Opening Book (Zobrist), Endgame Knowledge
  learn/       — NNUE Trainer, Backpropagation, Quantization
  selfplay/    — Self-play runner, Match config
  uci/         — UCI Protocol, Format
examples/      — Standalone executables (01-20)
scripts/       — Python tools (mining, training, hub, server)
data/          — JSONL datasets, XRNN binary weights
web/           — Web UI (HTML/JS/CSS)
```

### 2.2 Quy Tắc NNUE Engine

**Binary Format XRNN v1** (BẮT BUỘC tuân thủ khi tạo/đọc weights):

| Segment | Type | Size | Scale Factor |
|---|---|---|---|
| Magic | `b"XRNN"` | 4B | — |
| Version | `u32 LE` | 4B | — |
| FT Bias | `i16[256]` | 512B | × 127.0 |
| FT Weight | `i16[65536][256]` | 33,554,432B | × 127.0 |
| Hidden Weight | `i8[32][512]` | 16,384B | × 64.0 |
| Hidden Bias | `i32[32]` | 128B | × 127.0 × 64.0 |
| Output Weight | `i8[32]` | 32B | × 64.0 |
| Output Bias | `i32` | 4B | × 64.0 × 64.0 × 400.0 |
| Output Scale | `i32` | 4B | Cố định = 16 |
| **Tổng** | | **33,571,504B** | **32.02 MB** |

**NGHIÊM CẤM** thay đổi scale factors mà không cập nhật cả Rust (`src/learn/nnue.rs::quantize`) VÀ Python notebooks đồng thời.

### 2.3 Quy Tắc Performance — CPU Cache Friendly & Hybrid GPU Acceleration

**Trên Intel i5-8259U (4 Physical Cores / 8 Threads, L2=256KB, L3=6MB):**
- **Cấu Trúc Nhân CPU Phần Cứng**: Chip Intel i5-8259U sở hữu **4 nhân vật lý (Physical Cores)** và 8 luồng Hyper-Threading (KHÔNG có nhân P-core / E-core - chỉ xuất hiện từ Thế hệ 12 trở về sau).
- **Khi Nào Dùng 4 Luồng (THREADS = 4)**: Cho các tác vụ tính toán đệ quy thuần tuý (Compute-Bound Search, SIMD NNUE) nhằm đảm bảo mỗi luồng chiếm trọn 100% bộ nhớ đệm L1D (32KB) và L2 (256KB) của nhân vật lý, triệt tiêu 100% xung đột Cache Bouncing.
- **Khi Nào Dùng > 4 Luồng (THREADS = 8..64)**: Chỉ áp dụng khi khai thác dữ liệu tự đấu hàng loạt (Multi-Stream Data Mining 512+ ván cờ) hoặc I/O-bound batch write tệp tin.
- **Thông Số Cân Bằng GPU Hybrid Điểm Vàng ($B^* = 256$)**: Khi kết hợp 4 luồng CPU vật lý với WGPU Metal GPU Evaluator ở ngưỡng nạp lô $B^* = 256$ thế cờ / Compute Pass, hệ thống đạt thông lượng đỉnh **579,549 FEN / giây** và duy trì **tỉ lệ tải GPU phần cứng 88%**.
- **Cờ Tính Năng (Feature Flags) & Auto-Rollback**: Mọi module Hybrid Engine BẮT BUỘC dùng struct `Manager` (`src/circuit/flag.rs`) quản lý 5 cờ nguyên tử `Gpu`, `Queue`, `Ordering`, `Pruning`, `Rollback`. Khi xảy ra lỗi GPU, tự động ngắt cờ GPU và hạ cấp an toàn về CPU SIMD HCE.
- **Sắp Xếp Nước Đi MVV-LVA**: BẮT BUỘC gọi `order::sort()` trước khi duyệt cây Alpha-Beta/PVS để đẩy tỷ lệ cắt tỉa TT lên **> 85%** và giảm 98% số nút lá thừa.
- **Search TT Hash**: `Search::new(4)` — 4MB fit gần L3 cache. NGHIÊM CẤM dùng ≥ 8MB cho mining workload.
- **Per-thread buffer**: Mỗi thread worker dùng `Vec<String>` cục bộ, chỉ lock Mutex 1 lần cuối ván để batch write. NGHIÊM CẤM lock Mutex cho mỗi sample.
- **Atomic batch update**: `fetch_add(batch_size)` cuối ván, KHÔNG `fetch_add(1)` mỗi sample.
- **NNUE weights**: 32MB FT matrix là read-only shared → KHÔNG gây false sharing. Hidden/Output layers (16KB) fit hoàn toàn trong L1D cache.

### 2.4 Quy Tắc Data Mining

**Output JSONL format** (field names thống nhất):
```json
{"fen":"...","best_move":"e2e4","score":125,"depth":4}
```

| Field | Type | Mô tả | Phạm vi hợp lệ |
|---|---|---|---|
| `fen` | string | FEN cờ tướng đầy đủ | 10 hàng, 2 vua, 9 cột/hàng |
| `best_move` | string | UCI 4 ký tự | `[a-i][0-9][a-i][0-9]` |
| `score` | i32 | Centipawn evaluation | `[-30000, 30000]` |
| `depth` | u8 | Search depth | `[1, 20]` |

**NGHIÊM CẤM** dùng field name `eval` — phải dùng `score` thống nhất toàn hệ thống.

### 2.5 Quy Tắc Opening trong Data Mining

- **50% Book Opening**: Đi theo Opening Book Zobrist đến khi hết sách, sau đó thêm 2-4 nước random.
- **50% Random Opening**: 6 nước ngẫu nhiên thuần (phương pháp cũ, bảo toàn đa dạng).
- **Depth Mining ≥ 4**: NGHIÊM CẤM dùng depth < 4 cho dữ liệu huấn luyện production.

---

## III. QUY TẮC BẢO MẬT

### 3.1 Token & Credentials
- **NGHIÊM CẤM** hardcode bất kỳ token, API key, password nào trong mã nguồn.
- **BẮT BUỘC** sử dụng biến môi trường: `HF_TOKEN`, `API_KEY`, v.v.
- **BẮT BUỘC** kiểm tra `.env.example` cập nhật khi thêm biến môi trường mới.
- Trên Colab: dùng `google.colab.userdata.get('HF_TOKEN')`.

### 3.2 Dữ Liệu Cộng Đồng — Validation Gateway
Mọi dữ liệu từ cộng đồng PHẢI qua Validation Gateway trước khi merge:
1. **FEN hợp lệ**: 10 hàng, 2 vua (K+k), tổng cột = 9 mỗi hàng
2. **Score trong phạm vi**: `|score| ≤ 30000`
3. **Move hợp lệ**: UCI 4 ký tự `[a-i][0-9][a-i][0-9]`
4. **Không trùng lặp hàng loạt**: Tỷ lệ FEN duplicate < 5% trong mỗi batch

---

## IV. QUY TẮC HUẤN LUYỆN

### 4.1 NNUE PyTorch Training
- **BẮT BUỘC** Train/Test Split 80/20 — NGHIÊM CẤM train trên toàn bộ dataset.
- **BẮT BUỘC** Early Stopping khi test loss không giảm trong 30 epochs.
- **MSE = 0.000000 là cờ đỏ overfitting** — không phải thành tích.
- Score normalization: `score / 400.0` → phạm vi `[-1, 1]`.
- Weight Decay: `1e-4` (tối thiểu).

### 4.2 LLM GRPO Training (Xiangqi-R1)
- **BẮT BUỘC** có ≥ 1 reward function — NGHIÊM CẤM `reward_funcs=[]`.
- Reward functions tối thiểu: `reward_format` (JSON + bestmove) + `reward_thought` (thẻ `<thought>`).
- Max steps: 100-300 trên Colab T4 Free Tier.

### 4.3 Benchmark Yêu Cầu Tối Thiểu
- **≥ 200 ván** với **depth ≥ 5** để kết quả có ý nghĩa thống kê.
- Margin of error mục tiêu: ≤ ±30 ELO.
- 40 ván depth 3 **KHÔNG ĐẠT** tiêu chuẩn benchmark — chỉ dùng làm smoke test.

---

## V. QUY TẮC GIAO THỨC JRCP 2.0

Xem chi tiết tại [`.agents/memory/jrcp_2_0_spec.md`](file://.agents/memory/jrcp_2_0_spec.md).

Tóm tắt 14 chiều kích bắt buộc:
1. Ma trận bàn cờ 2D (10×9)
2. FEN string đầy đủ
3. PGN/UCI move history
4. Centipawn evaluation
5. Cơ hội (Opportunity score)
6. Nguy cơ (Threat score)
7. Tích cực (Positive factors)
8. Tiêu cực (Negative factors)
9. Ma trận 3 nước đi Candidate
10. Đồ thị DAG biến thể
11. Legal Move 100% (Logit Masking)
12. An toàn Tướng (King Safety)
13. Trung Lộ (Column 5 control)
14. Lực lượng tổng hợp (Material balance)

---

## VI. BÀI HỌC XƯƠNG MÁU — TÓM TẮT NHANH

| # | Bài học | Nguồn |
|---|---|---|
| 1 | NGHIÊM CẤM dùng `...`, `// TODO`, cắt xén mã nguồn | AGENTS.md global |
| 2 | NGHIÊM CẤM báo cáo khống chưa xác minh (AI Slop) | pain_points_20260808 |
| 3 | NGHIÊM CẤM dữ liệu giả random — phải dùng Native Rust Engine | pain_points_20260808 |
| 4 | Token hardcoded = lỗ hổng bảo mật CRITICAL | Audit 2026-08-08 |
| 5 | MSE=0.000000 = overfitting, KHÔNG phải thành tích | Audit 2026-08-08 |
| 6 | `reward_funcs=[]` = GRPO crash hoặc zero gradient | Audit 2026-08-08 |
| 7 | Beta Cutoff nút gốc ply=0 Depth 12: chỉ cutoff khi depth >= 2 | pain_points_20260807 |
| 8 | TT ô nhiễm Abort: flag kết quả bằng ABORT, không lưu vào TT | pain_points_20260807 |

---

## VII. KIỂM TRA TRƯỚC KHI COMMIT

Mọi Agent PHẢI chạy checklist trước khi commit:

```bash
# 1. Rust build
cargo check --release --examples

# 2. Token scan (phải trả về 0 kết quả)
grep -rn "hf_[a-zA-Z]" scripts/ examples/ *.ipynb

# 3. Field name consistency (phải trả về 0 cho "eval":)
grep -rn '"eval":' examples/ scripts/

# 4. Quantization test
python3 scripts/test_quantization.py data/nnue_weights_gen5.bin
```

---

## VIII. ĐIỀU KHOẢN TỐI THƯỢNG: KỶ LUẬT LƯU TRỮ KÝ ỨC VĨNH CỬU & DIỄN GIẢI TƯỜNG MINH (IMMUTABLE MEMORY & ELABORATION MANDATE)

### 8.1 CẤM TỰ MÃN NGHĨ RẰNG "CÓ TRONG CONTEXT WINDOW NÊN KHÔNG CẦN CẬP NHẬT FILE ĐĨA"
- Mọi suy luận, bài học xương máu, lỗi phát sinh, hoặc kết quả phiên làm việc **BẮT BUỘC phải được ghi ra tệp đĩa vật lý ngay lập tức**:
  - `.agents/memory/pain_points_[YYYYMMDD].md` (Ghi nhận bài học xương máu mới)
  - `.agents/memory/INDEX.md` (Đăng ký vào bảng mục lục ký ức vĩnh cửu)
  - `.agents/logs/session_active_[YYYYMMDD][HHMM].md` (Nhật ký phiên làm việc)
  - `.agents/logs/INDEX.md` (Đăng ký vào bảng mục lục nhật ký phiên)
- **NGHIÊM CẤM** lý do "ngữ cảnh context window đã có rồi nên không cần ghi file đĩa" hoặc "để làm sau". Ký ức không nằm trên tệp đĩa = KHÔNG TỒN TẠI!

### 8.2 BẮT BUỘC DIỄN GIẢI & CHÚ THÍCH TƯỜNG MINH TRÊN TỪNG DÒNG MÃ (MANDATORY LINE-BY-LINE ELABORATION ON EVERY IDENTIFIER)
- **NGHIÊM CẤM TỰ TẠO AI SLOP BIẾN CHẤT, LƯỜI BIẾNG, TÓM TẮT CẮT XÉN, LÀM VIỆC CẨU THẢ, LÀM CHO CÓ LỆ, LÀM ĐỂ ĐỐI PHÓ!**
- **BẮT BUỘC DIỄN GIẢI TỪNG DÒNG LỆNH (LINE-BY-LINE)**: Mọi tệp hướng dẫn, quy tắc, chú thích, tài liệu và mã nguồn Python/Rust/JS/Notebook **PHẢI được giải thích, chú thích, diễn giải chi tiết tường minh 100% bằng Tiếng Việt trên TẤT CẢ các dòng mã nguồn**, bao gồm:
  1. `variable` (biến cục bộ / biến toàn cục)
  2. `constant` (hằng số)
  3. `parameter` (tham số đầu vào của hàm / phương thức)
  4. `function` / `method` (hàm / phương thức)
  5. `class` (lớp)
  6. `object` (thể hiện đối tượng)
  7. `field` / `property` (thuộc tính / trường dữ liệu)
  8. `module` / `package` (mô-đun / gói thư viện nạp vào)
  9. `interface` / `trait` (giao diện / khuôn mẫu trừu tượng)
  10. `namespace` (không gian tên)
  11. `enum` (kiểu liệt kê)
  12. `type` / `alias` (kiểu dữ liệu / bí danh)
- **Mục đích**: Đảm bảo một Agent thiểu năng / kém thông minh nhất khi đọc vào cũng buộc phải hiểu 100% logic, không thể hiểu sai hay làm sai.
- **NGHIÊM CẤM** chú thích thùng rác chung chung chỉ ở đầu hàm, CẤM diễn giải mập mờ, CẤM viết tắt, CẤM giả định "người khác / agent khác tự hiểu".

### 8.3 KỶ LUẬT 5 THÀNH PHẦN BẮT BUỘC TRONG MỌI HÀNH ĐỘNG
1. **Kế Hoạch (Plan)**: Không bao giờ viết mã trong vô định. Phải khảo sát, nghiên cứu, lập kế hoạch 5 bước.
2. **Đánh Số Phiên Bản (Versioning)**: Đánh số phiên bản Semantic Versioning (`v1.0.0`, `v2.0.0`) cho mọi module/binary/spec.
3. **Telemetry**: Tự động nhận diện hạn mức hạ tầng cgroups (CPU quota, RAM limit).
4. **Logger**: Ghi nhật ký thực thi (log timestamps, stack trace, errors).
5. **Metrics**: Đo lường thông số định lượng thực tế (FEN/s, RAM RSS, CPU pct, file size).

### 8.4 QUY TRÌNH DỪNG LẠI 1 NHỊP QUAN SÁT THIẾU SÓT
> **"THÀ CHẬM MỘT NHỊP QUAN SÁT THIẾU SÓT CÒN HƠN CẨU THẢ, AI SLOP BIẾN CHẤT, LƯỜI BIẾNG, TÓM TẮT CẮT XÉN, LÀM CHO CÓ LỆ, LÀM ĐỂ ĐỐI PHÓ!"**

### 8.5 QUY TRÌNH BẮT BUỘC TĂNG PHIÊN BẢN KHI SỬA MÃ NGUỒN (MANDATORY AUTOMATIC VERSION BUMP PROTOCOL)
- **Ràng Buộc Sắt**: BẤT KỲ LẦN NÀO sửa lỗi (bugfix), nâng cấp tính năng (feature), hoặc chỉnh sửa mã nguồn trong `app.py` hay các tệp engine Rust, Agent BẮT BUỘC phải thực hiện tuần tự 4 bước tăng phiên bản:
  1. **Tăng Số Phiên Bản `APP_VERSION`**: Ví dụ từ `v2.6.0-production` -> `v2.7.0-production` (hoặc patch release `v2.6.1-production`).
  2. **Cập Nhật Dấu Thời Gian Build `APP_BUILD_STAMP`**: Ghi rõ mốc thời gian thực tế (ví dụ: `2026-08-09 21:08:00 ICT`).
  3. **Cập Nhật Ghi Chú Phát Hành `APP_RELEASE_NOTES`**: Ghi ngắn gọn nội dung lỗi vừa được sửa hoặc tính năng vừa được nâng cấp.
  4. **Kiểm Thử Biên Dịch Mới Cho Phép Commit**: Chạy `python3 -m py_compile app.py` hoặc `cargo check` để đảm bảo bản build mới hoạt động 100% trước khi push.
- **Lý Do Tối Thượng**: Nếu sửa lỗi mã nguồn mà KHÔNG tăng số phiên bản và dấu thời gian, người dùng khi reload trang web sẽ rơi vào trạng thái "mù thông tin", không thể phân biệt được ứng dụng đang chạy bản cũ hay bản mới đã sửa lỗi!

### 8.6 QUY TẮC CẤM GHI ĐÈ KÝ ỨC CŨ — BẮT BUỘC CHỨA DẤU THỜI GIAN GIỜ PHÚT `[YYYYMMDD_HHMM]` (STRICT IMMUTABLE TIMESTAMPED MEMORY MANDATE)
- **NGHIÊM CẤM** xóa bỏ, cắt xén, tóm tắt hoặc ghi đè làm mất các Bài học cũ trong tệp `.agents/memory/pain_points_*.md`.
- **BẮT BUỘC DẤU THỜI GIAN GIỜ PHÚT (`YYYYMMDD_HHMM`) HOẶC PHIÊN (`_vN_`)**: Mọi tệp ký ức bài học mới khi tạo ra BẮT BUỘC phải đặt tên theo định dạng chứa mốc thời gian chính xác đến phút hoặc số phiên, ví dụ: `.agents/memory/pain_points_[YYYYMMDD_HHMM]_[chủ_đề].md` (ví dụ: `pain_points_20260813_1618_cargo_jobs.md`) hoặc `pain_points_[YYYYMMDD]_v[N]_[chủ_đề].md`.
- **CẤM TẠO TỆP CHỈ CÓ NGÀY TRUNG CHUNG**: NGHIÊM CẤM tạo tệp dạng `pain_points_[YYYYMMDD].md` không có giờ phút/số phiên, vì tên tệp chung chung này dễ bị các thế hệ Agent tiếp theo trong cùng ngày sử dụng `write_to_file(Overwrite: true)` ghi đè xóa mất tri thức cũ!
- **BẮT BUỘC ĐĂNG KÝ MỤC LỤC**: Sau khi tạo tệp ký ức mới, BẮT BUỘC phải bổ sung đường dẫn tệp vào [`INDEX.md`](file://.agents/memory/INDEX.md).

### 8.7 QUY TẮC CUNG CẤP LINK GOOGLE COLAB KÈM MCP PROXY TOKEN (MANDATORY COLAB MCP LINK MANDATE)
- **Ràng Buộc Sắt**: BẤT KỲ LẦN NÀO cung cấp đường dẫn Google Colab cho người dùng, Agent BẮT BUỘC phải đọc token/port mới nhất từ `.agents/memory/colab_mcp_proxy.json` và gắn trực tiếp chuỗi hash fragment vào cuối URL:
  - Dạng link chuẩn: `https://colab.research.google.com/github/hoduyquocbao/xiangqi-rim/blob/<COMMIT_SHA>/colab_gpu_multiturn_v17.ipynb#mcpProxyToken=<TOKEN>&mcpProxyPort=<PORT>`
- **Mục Đích**: Đảm bảo khi người dùng bấm vào đường dẫn, trang web Google Colab tự động thiết lập WebSocket Proxy bridge về `localhost:<PORT>` mà không bị ngắt kết nối giữa chừng!

### 8.8 RÀNG BUỘC SẮT KÍCH HOẠT COLAB SKILLS (MANDATORY COLAB SKILL ACTIVATION PROTOCOL)
- **Ràng Buộc Sắt**: BẤT KỲ LẦN NÀO thao tác với Google Colab, Agent BẮT BUỘC phải kích hoạt và tham chiếu 2 kỹ năng chuyên biệt:
  1. `colab-mcp-setup`: Thiết lập và sửa lỗi WebSocket Proxy bridge kết nối Colab.
  2. `colab-notebook-ops`: Tuân thủ 7 quy tắc bảo toàn thế hệ (Live DOM sync, ô mã nguồn độc lập, Form Cells `# @title`, bắt lỗi 403 Forbidden HF token).
- **Tuyệt đối KHÔNG ĐƯỢC PHÉP**: Gọi công cụ Colab MCP mà không đọc hoặc tuân thủ các quy tắc trong 2 skills trên!

### 8.9 RÀNG BUỘC SẮT ƯU TIÊN GPU PHẦN CỨNG BẰNG MỌI GIÁ (MANDATORY GPU HARDWARE ACCELERATION MANDATE)
- **Ràng Buộc Sắt Bằng Mọi Giá**: BẤT KỲ LẦN NÀO thực thi mã Rust Engine (mining, batch evaluation, NNUE search), Agent BẮT BUỘC phải ưu tiên phần cứng GPU (NVIDIA CUDA / Vulkan / WGPU / Metal Native) bằng mọi giá.
- **NGHIÊM CẤM** ngã về CPU fallback khi đang trong môi trường có GPU phần cứng. Luôn đảm bảo nạp đúng driver Vulkan ICD JSON (`/etc/vulkan/icd.d/nvidia_icd.json`) hoặc CUDA backend để vRAM và Compute Units của GPU được kích hoạt 100%!

### 8.10 QUY TẮC BẮT BUỘC YIELD KẾT QUẢ REALTIME VỚI CỜ CẤU HÌNH TẦN SUẤT CHỐNG CHÁY TERMINAL (MANDATORY CONFIGURABLE REALTIME YIELD PROTOCOL)
- **Ràng Buộc Sắt Cho Mọi Mã Nguồn (Engine / Miner / Benchmark / Training / Script / Example / Rust / Python / C++)**:
  - **NGUYÊN LÝ VẬT LÝ NGHẼN BỘ ĐỆM STDOUT (BLOCK BUFFERING TRAP)**: Khi đầu ra `stdout` bị điều hướng vào tệp đĩa, ống dẫn (pipe), hoặc tiến trình chạy ngầm `task-*.log` (không phải màn hình TTY), hệ điều hành và thư viện chuẩn Rust/Python mặc định áp dụng bộ đệm khối **Block Buffering (8 KB)**. Nếu mã nguồn không ép xả đệm, tiến trình sẽ im lặng trong nhiều phút liền rồi bất ngờ xả ra một đống dòng cùng lúc (gây ra hiện tượng "mù thông tin", nghẽn thông số real-time).
  - **NGUYÊN LÝ CHỐNG CHÁY TERMINAL KHI ĐÀO QUY MÔ LỚN (500K - 1M MẪU FEN)**: Đối với các tác vụ đào dữ liệu lớn (500,000 - 1,000,000 mẫu FEN / 10,000+ ván cờ), việc in log per ply (từng nước đi) sẽ xả ra 25 - 50 triệu dòng văn bản, gây ra hiện tượng **"Cháy Terminal" (Terminal Flooding & Disk I/O Bottleneck)** và ép CPU render văn bản thừa vô ích.
  - **CỜ CẤU HÌNH ĐỘNG TẦN SUẤT YIELD (`YIELD_INTERVAL` / `YIELD_MODE` / FEATURE FLAG)**:
    1. **Tự Động Đọc Biến Môi Trường**: Mọi kịch bản Miner / Benchmark BẮT BUỘC phải đọc biến môi trường `LOG_INTERVAL` / `YIELD_INTERVAL` (mặc định = 1 cho smoke test/benchmark; = 10 hoặc 100 ván cờ / per batch cho 500K-1M massive mining) hoặc cờ `YIELD_MODE` (`"ply"`, `"game"`, `"batch"`).
    2. **Xả Đệm Tức Thì Khi In (Unbuffered Flush)**: Mọi dòng log được xuất ra (dù per ply hay per batch 100 ván) BẮT BUỘC phải theo sau bởi `std::io::stdout().flush().unwrap()` (Rust) hoặc `flush=True` (Python) để đảm bảo OS xả đĩa tức thì.
    3. **Tần Suất Điểm Vàng**:
       - *Benchmark / Live UI Streaming*: Yield từng nước đi (per ply) hoặc từng ván cờ.
       - *Massive Mining 500K - 1M Samples*: Yield summary dòng tiến độ per game hoặc per 5,000 samples FEN. Tuyệt đối không im lặng hoàn toàn và không in tràn lan per ply gây cháy terminal!


### 8.11 RÀNG BUỘC SẮT CẤU HÌNH ĐỘNG & BẢO TOÀN QUYỀN CHỈNH SỬA TỪ BÊN NGOÀI (MANDATORY DYNAMIC CONFIGURATION & EXTERNAL EXPOSURE MANDATE)
### 8.12 QUY TẮC CẤM SPAM LỆNH BIÊN DỊCH SONG SONG & BẮT BUỘC DỪNG LẠI 1 NHỊP CHỜ KHẢO SÁT BIÊN DỊCH (MANDATORY BUILD CHOKE PREVENTION & SINGLE-BUILD LOCK PROTOCOL)
- **BỐI CẢNH THỰC TẾ HẠ TẦNG LAPTOP**:
  - Tác vụ biên dịch Rust Engine (`cargo build` / `cargo run` / `cargo check`) có quy mô lớn tốn rất nhiều tài nguyên CPU/RAM. Trên máy laptop (như Intel i5-8259U với 4 physical cores):
    - **Debug Build (`cargo build`)**: Cần khoảng **3 PHÚT** để hoàn thành.
    - **Release Build (`cargo build --release`)**: Cần khoảng **5 PHÚT** để hoàn thành.
- **RÀNG BUỘC SẮT BẮT BUỘC TUÂN THỦ 100%**:
  1. **TUYỆT ĐỐI CẤM SPAM LỆNH BIÊN DỊCH SONG SONG**: NGHIÊM CẤM kích hoạt từ 2 tiến trình `cargo build`, `cargo check`, hoặc `cargo run` trở lên cùng một lúc. Spam nhiều lệnh build song song sẽ làm nghẽn `file lock on artifact directory`, ép CPU 100% liên tục và gây treo/sập máy laptop!
  2. **RÀNG BUỘC KIỂM TRA MẮT XÍCH TRƯỚC KHI BUILD**: Trước khi gọi bất kỳ lệnh biên dịch `cargo` mới nào, Agent BẮT BUỘC phải kiểm tra danh sách task background bằng `manage_task` (Action: `list`). Nếu phát hiện có tiến trình `cargo` cũ đang chạy, BẮT BUỘC phải chờ tiến trình cũ kết thúc hoặc diệt dọn dẹp xong rồi mới phát lệnh mới.
  3. **KỶ LUẬT DỪNG LẠI 1 NHỊP KIÊN NHẪN CHỜ ĐỦ THỜI GIAN**: Khi đã phát lệnh biên dịch:
     - Dành khoảng **3 phút cho Debug build** và **5 phút cho Release build**.
     - Agent tuyệt đối KHÔNG vội vã, KHÔNG dồn ép task mới, KHÔNG gọi tool liên tục làm treo tiến trình biên dịch ngầm!

### 8.14 QUY TẮC KỶ LUẬT CUỐN CHIẾU DỮ LIỆU ĐĨA CỤC BỘ & TỰ ĐỘNG ĐỒNG BỘ CLOUD HUGGINGFACE (MANDATORY ROLLING CHUNK & HF AUTO-PURGE PROTOCOL)
- **Ràng Buộc Sắt Cho Mọi Tác Vụ Khai Thác Đêm Quy Mô Lớn (MacBook / Laptop / Cloud)**:
  - **NGUYÊN LÝ TIẾT KIỆM Ổ ĐĨA SSD CỤC BỘ**: Khi tự đấu đào dữ liệu quy mô lớn (hàng triệu FEN) chạy liên tục đêm ngày trên MacBook, BẮT BUỘC phải áp dụng cơ chế **Rolling Chunks (Chia lô nhỏ 100K - 200K mẫu FEN / chunk)**.
  - **QUY TRÌNH 3 BƯỚC BẮT BUỘC (MINE -> SYNC HF -> PURGE LOCAL)**:
    1. **Mine Chunk**: Động cơ sinh dữ liệu theo các tập tin chunk có kích thước giới hạn (ví dụ `chunk_0001.jsonl` ~25 MB).
    2. **Cloud Sync**: Ngay khi hoàn tất 1 chunk hoặc 1 epoch huấn luyện, kịch bản Python/Rust BẮT BUỘC tự động upload tệp chunk lên Hugging Face Hub Dataset Repository (`huggingface_hub` / `hf-cli`).
    3. **Local Purge**: Ngay sau khi Hugging Face xác nhận upload thành công (Checksum SHA-256 OK), BẮT BUỘC phải xóa ngay tệp chunk cục bộ (`os.remove()`) trên đĩa MacBook.
  - **Mục Đích**: Đảm bảo dung lượng SSD MacBook chiếm dụng luôn duy trì cực nhẹ **< 100 MB**, triệt tiêu 100% rủi ro tràn ổ đĩa hay treo sập máy laptop khi đào dữ liệu xuyên đêm!






