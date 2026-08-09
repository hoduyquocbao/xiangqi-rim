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

### 2.3 Quy Tắc Performance — CPU Cache Friendly

**Trên Intel i5-8259U (4P/8L, L2=256KB, L3=6MB):**
- **Search TT Hash**: `Search::new(4)` — 4MB fit gần L3 cache. NGHIÊM CẤM dùng ≥ 8MB cho mining workload.
- **THREADS mặc định**: Số physical cores (4), KHÔNG phải logical cores (8), cho compute-bound workload.
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

### 8.2 BẮT BUỘC DIỄN GIẢI & CHÚ THÍCH TƯỜNG MINH (FORCED ELABORATION MANDATE FOR ALL AGENTS)
- Mọi tệp hướng dẫn, quy tắc, chú thích, tài liệu và mã nguồn **PHẢI được diễn giải, chú thích, giải thích chi tiết tỉ mỉ tới từng dòng, từng thông số, từng trường dữ liệu bằng tiếng Việt 100%**.
- **Mục đích**: Đảm bảo một Agent thiểu năng / kém thông minh nhất khi đọc vào cũng buộc phải hiểu và thực hiện đúng 100%, không thể hiểu sai hay làm sai.
- **NGHIÊM CẤM** diễn giải mập mờ, CẤM viết tắt, CẤM giả định "người khác / agent khác tự hiểu".

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
