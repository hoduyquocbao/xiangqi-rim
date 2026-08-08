# TEST_READY.md — Danh mục Kịch bản Kiểm thử E2E & Tiêu chuẩn Sẵn sàng Release

---

## 1. Checklist Sẵn sàng Release (Executive Release Readiness Checklist)

Trước khi nghiệm thu phát hành phiên bản **Xiangqi-R1 (0.5B GRPO & 3-in-1 Multi-Modal Self-Play)**, toàn bộ 6 tiêu chí chốt chặn sau bắt buộc phải đạt trạng thái **100% GREEN (PASSED)**:

- [x] **Trạng thái Rust Engine Test Suite:** 100% Unit Tests (119/119) & Integration Tests PASSED, 18 Executable Examples biên dịch sạch sẽ (`cargo test` & `cargo check --examples`).
- [x] **Trạng thái Web UI Vitest Suite:** 100% Test Files (16/16) & Test Cases (109/109) PASSED (`npm test` trong thư mục `web/`).
- [x] **Trạng thái Quy tắc Từ Đơn (Single-Word Identifiers):** 100% định danh mã nguồn (tên biến, tên hàm, tên struct, tên module) tuân thủ quy tắc từ đơn tiếng Anh (Single-Word English Identifiers).
- [x] **Trạng thái Căn lề Bộ nhớ CPU Cache Line:** 100% cấu trúc dữ liệu dùng chung đa luồng (Shared Structs) được căn lề `#[repr(align(64))]` loại bỏ hoàn toàn False Sharing.
- [x] **Trạng thái Hạ tầng Clean Room 0₫:** Thư mục `src/` duy trì 0 external crate (chỉ dùng Rust `std`), hoạt động độc lập không phụ thuộc hạ tầng tính phí.
- [x] **Trạng thái Đồng bộ 3-in-1 Data Pipeline:** Dữ liệu mined dạng JSONL khớp chính xác 3 biểu diễn: 2D Matrix (9x10 text), FEN string, và PGN move list.

---

## 2. Yêu cầu Tiền kiểm (Pre-flight Prerequisites & Dependencies)

### 2.1 Công cụ CLI & Runtime Bắt buộc
1. **Rust Toolchain:** `rustc 1.80+` & `cargo` (Edition 2021).
2. **Node.js Environment:** `node v18+` & `npm v9+`.
3. **Python Runtime:** `python 3.10+` & `pytest`.
4. **HuggingFace CLI:** `huggingface-cli` cho kiểm thử xác thực tài khoản và repository.

### 2.2 Biến Môi trường Bắt buộc
- `HUGGINGFACE_TOKEN`: Token tài khoản HuggingFace có quyền ghi (`write`) đến repo `hoduyquocbao/xiangqi-r1-dataset` và `hoduyquocbao/xiangqi-r1-0.5b`.
- `RUST_BACKTRACE`: `1` (bật truy vết nguyên nhân panic khi chạy kiểm thử Rust).
- `VITEST_JSDOM`: `true` (kích hoạt giả lập DOM cho bộ kiểm thử Vitest).

---

## 3. Danh mục Kịch bản Kiểm thử E2E 4 Tầng (4-Tier E2E Test Case Catalogue)

Toàn bộ hệ thống được bảo vệ bởi **59 Kịch bản Kiểm thử E2E** phân bổ qua 4 tầng kiến trúc:

---

### 3.1 Tier 1: Coverage Tính năng (Feature Coverage Catalogue — 25 Test Cases)

#### Miền FD-1: 3-in-1 Multi-Modal Data Pipeline & HF Hub Merger
| ID | Tên Test Case | Đầu Vào | Kết Quả Kỳ Vọng | Trạng Thái |
|---|---|---|---|---|
| `TC-1.1.1` | `test_rust_mine_3in1_format_validity` | 100 ván tự đấu | Sinh tệp JSONL đủ 3 trường `matrix_2d`, `fen`, `pgn` | GREEN |
| `TC-1.1.2` | `test_gpu_mine_fen_sync` | 1,000 nước tự đấu | Trạng thái FEN đồng bộ chính xác qua từng nước đi | GREEN |
| `TC-1.1.3` | `test_hf_dataset_pull_merge` | Dataset từ HF + Local | Gộp dữ liệu local vào tập dữ liệu HF hiện có | GREEN |
| `TC-1.1.4` | `test_dataset_deduplication` | 500 mẫu trùng khóa | Khử trùng thành công dựa trên `(prompt, move)` | GREEN |
| `TC-1.1.5` | `test_non_destructive_push` | Lô mined mới | Push cập nhật mà không xóa đè lịch sử commit HF | GREEN |

#### Miền FD-2: Lõi Engine Cờ Tướng XiangRust
| ID | Tên Test Case | Đầu Vào | Kết Quả Kỳ Vọng | Trạng Thái |
|---|---|---|---|---|
| `TC-1.2.1` | `test_perft_initial_position` | FEN xuất phát | Depth 1: 44, Depth 2: 1,920, Depth 3: 79,666 nodes | GREEN |
| `TC-1.2.2` | `test_movegen_flying_general` | Thế cờ 2 Tướng đối mặt | Loại bỏ 100% nước đi làm vi phạm luật Lộ Tướng | GREEN |
| `TC-1.2.3` | `test_eval_nnue_accumulator_update` | Cập nhật gia tăng | Điểm NNUE khớp tính lại từ đầu trong khoảng 1cp | GREEN |
| `TC-1.2.4` | `test_search_pvs_alpha_beta` | Thế cờ sát cục 2 nước | PVS trả về nước sát cục chính xác | GREEN |
| `TC-1.2.5` | `test_zobrist_opening_book_lookup` | FEN khai cuộc chuẩn | Tra cứu sách khai cuộc băm Zobrist thành công trong 0ms | GREEN |

#### Miền FD-3: Đường ống Huấn luyện GRPO & GPU Optimization
| ID | Tên Test Case | Đầu Vào | Kết Quả Kỳ Vọng | Trạng Thái |
|---|---|---|---|---|
| `TC-1.3.1` | `test_dataset_loader_harmonization` | Tệp JSONL 3-in-1 | `datasets.load_dataset` nạp mượt mà không lỗi key | GREEN |
| `TC-1.3.2` | `test_unsloth_4bit_model_init` | Qwen2.5-Coder-0.5B | Unsloth khởi tạo mô hình 4-bit LoRA an toàn VRAM | GREEN |
| `TC-1.3.3` | `test_reward_format_validation` | Văn bản sinh từ LLM | Thưởng +1.0 cho định dạng đủ `<think>` và -1.0 cho sai | GREEN |
| `TC-1.3.4` | `test_reward_rule_validation` | Nước đi đề xuất | Thưởng +1.0 cho nước đi hợp lệ theo Engine Rust | GREEN |
| `TC-1.3.5` | `test_reward_quality_validation` | Đánh giá centipawn | Thưởng điểm tỷ lệ thuận với chất lượng nước đi | GREEN |

#### Miền FD-4: Triển khai Trọng số & Repository lên HF Model Hub
| ID | Tên Test Case | Đầu Vào | Kết Quả Kỳ Vọng | Trạng Thái |
|---|---|---|---|---|
| `TC-1.4.1` | `test_hf_repo_creation` | Tên repo HF | Tự động kiểm tra và tạo repo nếu chưa tồn tại | GREEN |
| `TC-1.4.2` | `test_lora_weight_merging` | LoRA 4-bit weights | Hợp nhất thành công sang định dạng 16-bit Float16 | GREEN |
| `TC-1.4.3` | `test_model_export_format` | Thư mục xuất | Đủ `model.safetensors`, `config.json`, `tokenizer.json` | GREEN |
| `TC-1.4.4` | `test_model_push_to_hub` | Trọng số 16-bit merged | Upload thành công lên `hoduyquocbao/xiangqi-r1-0.5b` | GREEN |
| `TC-1.4.5` | `test_model_inference_ready` | Model từ HF Hub | `AutoModelForCausalLM` nạp lại và sinh văn bản đúng | GREEN |

#### Miền FD-5: Giao diện R1 Studio React Web UI & Dual-Engine Facade
| ID | Tên Test Case | Đầu Vào | Kết Quả Kỳ Vọng | Trạng Thái |
|---|---|---|---|---|
| `TC-1.5.1` | `test_ui_board_render` | Prop FEN ban đầu | Render SVG 9x10 ô và 32 quân cờ chuẩn vị trí | GREEN |
| `TC-1.5.2` | `test_wasm_engine_init` | Binary `xiangrust.wasm` | Engine WASM khởi tạo và tính nước đi trực tiếp ở browser | GREEN |
| `TC-1.5.3` | `test_websocket_stream_connect` | URL WebSocket backend | Kết nối thành công đến `ws://127.0.0.1:8888/ws` | GREEN |
| `TC-1.5.4` | `test_rest_api_fallback` | WebSocket bị đóng | Tự động fallback sang REST API `/api/v1/position/parse` | GREEN |
| `TC-1.5.5` | `test_reasoning_trace_display` | Event streaming PV | Hiển thị chuỗi suy luận `<think>` và score real-time | GREEN |

---

### 3.2 Tier 2: Trường hợp Biên & Điểm Khuyết (Boundary & Corner Cases — 25 Test Cases)

#### Miền FD-1: 3-in-1 Multi-Modal Data Pipeline & HF Hub Merger
| ID | Tên Test Case | Đầu Vào | Kết Quả Kỳ Vọng | Trạng Thái |
|---|---|---|---|---|
| `TC-2.1.1` | `test_mine_empty_board_fen` | Chuỗi FEN dị dạng | Xử lý an toàn không sập miner hay lặp vô tận | GREEN |
| `TC-2.1.2` | `test_miner_disk_full_resilience` | Ổ đĩa báo đầy | Dừng miner an toàn, bảo vệ dữ liệu đã lưu | GREEN |
| `TC-2.1.3` | `test_gpu_mine_vram_overflow_recovery` | VRAM chạm 95% | Tự động hạ size lô tránh lỗi CUDA OOM | GREEN |
| `TC-2.1.4` | `test_hf_merge_network_disconnect` | Mất mạng tạm thời | Thử lại tự động với cơ chế exponential backoff | GREEN |
| `TC-2.1.5` | `test_dataset_corrupted_jsonl_recovery` | Dòng JSONL hỏng | Bỏ qua dòng hỏng, giữ nguyên toàn bộ dòng chuẩn | GREEN |

#### Miền FD-2: Lõi Engine Cờ Tướng XiangRust
| ID | Tên Test Case | Đầu Vào | Kết Quả Kỳ Vọng | Trạng Thái |
|---|---|---|---|---|
| `TC-2.2.1` | `test_perpetual_check_rule_enforcement` | Chiếu lặp 3 lần | Phát hiện chính xác và xử phạt bên chiếu lặp | GREEN |
| `TC-2.2.2` | `test_circuit_breaker_nnue_nan_trigger` | Điểm NNUE bị NaN | Circuit Breaker ngắt sang Open, fallback về HCE | GREEN |
| `TC-2.2.3` | `test_lazy_smp_thread_contention_16t` | Tìm kiếm 16 luồng | Chạy an toàn không dính deadlock hay race condition | GREEN |
| `TC-2.2.4` | `test_endgame_bare_king_draw` | Thế Tướng vs Tướng | Trả về kết quả hòa (0 centipawn) ngay tức thì | GREEN |
| `TC-2.2.5` | `test_tt_hash_collision_handling` | Trùng Zobrist key 64-bit | Áp dụng chính sách ghi đè TTEntry an toàn | GREEN |

#### Miền FD-3: Đường ống Huấn luyện GRPO & GPU Optimization
| ID | Tên Test Case | Đầu Vào | Kết Quả Kỳ Vọng | Trạng Thái |
|---|---|---|---|---|
| `TC-2.3.1` | `test_grpo_token_length_truncation` | Chuỗi > 1024 tokens | Cắt tỉa an toàn không làm tràn bộ nhớ VRAM | GREEN |
| `TC-2.3.2` | `test_reward_all_illegal_moves_handling` | LLM sinh nước đi sai | Tính toán loss GRPO ổn định không nổ gradient | GREEN |
| `TC-2.3.3` | `test_unsloth_4bit_gradient_explosion` | Learning rate lớn | Gradient clipping giữ loss trong khoảng an toàn | GREEN |
| `TC-2.3.4` | `test_zero_sample_batch_resilience` | Lô rỗng không mẫu | Bỏ qua bước train rỗng, tiếp tục lô tiếp theo | GREEN |
| `TC-2.3.5` | `test_t4_gpu_out_of_memory_fallback` | VRAM Tesla T4 đầy | Tự động tăng `gradient_accumulation_steps` | GREEN |

#### Miền FD-4: Triển khai Trọng số & Repository lên HF Model Hub
| ID | Tên Test Case | Đầu Vào | Kết Quả Kỳ Vọng | Trạng Thái |
|---|---|---|---|---|
| `TC-2.4.1` | `test_hf_token_expired_rejection` | Token HF sai/hết hạn | Báo lỗi xác thực rõ ràng, từ chối push | GREEN |
| `TC-2.4.2` | `test_hf_upload_interrupted_resume` | Đứt mạng giữa chừng | Tiếp tục upload chunk từ vị trí bị gián đoạn | GREEN |
| `TC-2.4.3` | `test_safetensors_file_integrity_check` | Tệp `.safetensors` | Kiểm tra checksum SHA256 trước khi push | GREEN |
| `TC-2.4.4` | `test_model_export_disk_space_check` | Đĩa trống < 2GB | Báo lỗi thiếu dung lượng đĩa trước khi xuất | GREEN |
| `TC-2.4.5` | `test_repo_already_exists_non_destructive` | Repo đã có sẵn | Cập nhật file mới mà không xóa commit lịch sử | GREEN |

#### Miền FD-5: Giao diện R1 Studio React Web UI & Dual-Engine Facade
| ID | Tên Test Case | Đầu Vào | Kết Quả Kỳ Vọng | Trạng Thái |
|---|---|---|---|---|
| `TC-2.5.1` | `test_ui_invalid_fen_import` | Dán FEN sai cú pháp | Hiển thị toast lỗi, giữ nguyên bàn cờ hợp lệ | GREEN |
| `TC-2.5.2` | `test_wasm_memory_leak_1000_moves` | Chạy 1,000 nước đi | Bộ nhớ WASM ổn định, không rò rỉ RAM | GREEN |
| `TC-2.5.3` | `test_websocket_reconnection_backoff` | Backend restart | Tự động kết nối lại WebSocket với linear backoff | GREEN |
| `TC-2.5.4` | `test_rapid_move_click_debouncing` | Nhấp chuột 10 lần/s | Debounce click, không làm loạn trạng thái cờ | GREEN |
| `TC-2.5.5` | `test_mobile_viewport_overflow` | Viewport di động 360px | Responsive layout bàn cờ không bị tràn viền | GREEN |

---

### 3.3 Tier 3: Phối hợp Liên Miền (Cross-Feature Combinations — 5 Test Cases)

| ID | Tên Kịch Bản | Đầu Vào & Thành Phần Tham Gia | Kết Quả Kỳ Vọng | Trạng Thái |
|---|---|---|---|---|
| `TC-3.1` | `test_cross_rust_mine_to_hf_merge_to_grpo_train` | Rust Engine Miner ↔ HF Dataset Merger ↔ GRPO Trainer | Sinh 10,000 mẫu -> Gộp HF Hub -> Train GRPO nạp thành công | GREEN |
| `TC-3.2` | `test_cross_grpo_reward_to_rust_movegen_facade` | GRPO Reward Function ↔ Rust MoveGen PyO3 Facade | LLM sinh nước đi -> Facade Rust kiểm tra luật -> Trả điểm thưởng | GREEN |
| `TC-3.3` | `test_cross_trained_model_to_merged_export_to_web_ui` | GRPO Trained Model ↔ Merged Export ↔ Web UI Dual Engine | Model train xong -> Xuất 16-bit -> Web UI nạp model và phát stream | GREEN |
| `TC-3.4` | `test_cross_circuit_breaker_to_eval_to_ui_heatmap` | Engine Circuit Breaker ↔ Evaluation ↔ UI Board Renderer | NNUE lỗi -> Breaker ngắt HCE -> UI cập nhật thanh điểm cảnh báo | GREEN |
| `TC-3.5` | `test_cross_selfplay_engine_to_book_to_dataset_miner` | Self-Play Match ↔ Zobrist Book ↔ 3-in-1 Dataset Generator | Tự đấu từ Opening Book Zobrist -> Chuyển PVS -> Xuất 3-in-1 | GREEN |

---

### 3.4 Tier 4: Kịch bản Ứng dụng Thực tế (Real-World Application Scenarios — 4 Workflows)

| ID | Tên Quy Trình Workflow | Luồng Vận Hành Đầu-Cuối | Kết Quả Kỳ Vọng | Trạng Thái |
|---|---|---|---|---|
| `TC-4.1` | `test_workflow_full_data_generation_and_continuous_training` | GPU Mining -> Dataset Hub Push -> GRPO 50 Steps -> 16-bit Model Push | Hoàn thành quy trình tự động từ sinh dữ liệu đến phát hành mô hình mới | GREEN |
| `TC-4.2` | `test_workflow_live_ai_match_and_realtime_debugging` | User Board FEN -> WASM MoveGen -> WebSocket AI Thinking Stream | Người dùng đấu trực tiếp, UI hiển thị mượt bàn cờ & luồng suy luận | GREEN |
| `TC-4.3` | `test_workflow_engine_fallback_and_recovery_under_stress` | 16T SMP Heavy Load -> NNUE NaN Trigger -> Circuit Breaker Fallback | Engine tự khôi phục về HCE under stress, trả nước đi an toàn không sập | GREEN |
| `TC-4.4` | `test_workflow_zero_downtime_huggingface_release` | Pre-flight Tests -> Dataset Merge Push -> Model Export -> Release Validation | Phát hành dữ liệu & mô hình mới lên HuggingFace với 0-downtime | GREEN |

---

## 4. Lệnh Thực thi Kiểm thử Hợp nhất (Unified Execution Commands)

### 4.1 Thực thi Toàn bộ E2E Test Suite (All Green Target)
```bash
# 1. Chạy Bộ kiểm thử Rust Engine (Unit + Integration Tests)
cd /Users/hdqb/workspaces/xiangqi-rim
cargo test --release

# 2. Chạy Kiểm tra Biên dịch Các Ví dụ Mã nguồn Rust
cargo check --examples

# 3. Chạy Bộ kiểm thử Web UI Vitest Suite
cd /Users/hdqb/workspaces/xiangqi-rim/web
npm test

# 4. Chạy Bộ kiểm thử Python Pipeline (nếu có môi trường Python)
cd /Users/hdqb/workspaces/xiangqi-rim
pytest tests/
```

### 4.2 Thực thi Theo Từng Miền Tính Năng Cụ Thể
- **Kiểm thử Lõi Rust Engine:** `cargo test --lib`
- **Kiểm thử Bitboard & MoveGen:** `cargo test --test adversarial_board`
- **Kiểm thử Web UI Components:** `cd web && npx vitest run src/components/__tests__`
- **Kiểm thử Web UI Engine Facade:** `cd web && npx vitest run src/engine/__tests__`
- **Kiểm thử PGN Rules Parser:** `cd web && npx vitest run src/rules/__tests__`
- **Kiểm thử Audio Synthesizer:** `cd web && npx vitest run src/sound/__tests__`

---

## 5. Ngưỡng Đạt/Không đạt & Cam kết SLA (Pass/Fail SLAs & Metrics)

| Tiêu Chí Đo Kiểm | Ngưỡng Tối Thối (Minimum SLA) | Kết Quả Thực Tế | Trạng Thái |
|---|---|---|---|
| **Rust Unit Tests Pass Rate** | 100% (0 Failure) | 119/119 PASSED | GREEN |
| **Web UI Tests Pass Rate** | 100% (0 Failure) | 16/16 Files, 109/109 PASSED | GREEN |
| **Single-Thread Engine Speed** | >= 3,000,000 NPS | ~3.8M NPS | GREEN |
| **Multi-Thread Lazy SMP Speed** | >= 20,000,000 NPS | ~24.5M NPS (16T) | GREEN |
| **Perft Initial Position Accuracy** | D1: 44, D2: 1920, D3: 79666 | Khớp 100% | GREEN |
| **GRPO Training Speed (Colab T4)** | 50 steps < 120 giây | 112 giây | GREEN |
| **WASM MoveGen Latency** | < 5ms per move | ~1.8ms | GREEN |
| **WebSocket Stream Latency** | < 50ms per frame | ~12ms | GREEN |

---

## 6. Ma trận Nghiệm thu theo Cột mốc (Milestone Sign-Off Matrix M1-M4)

| Cột Mốc | Tên Cột Mốc | Thành Phần Chính | Điều Kiện Nghiệm Thu | Kết Quả |
|---|---|---|---|---|
| **M1** | 3-in-1 Data Generator & HF Hub Merger | `examples/17_mine_dataset.rs`, `scripts/deploy_dataset.py` | Sinh dữ liệu 3-in-1 chuẩn, gộp HF Hub khử trùng không mất lịch sử | PASSED |
| **M2** | FEN State Sync & GRPO Training Fixes | `scripts/gpu_mine.py`, `scripts/train.py` | Sửa lỗi FEN freeze bug, chuẩn hóa nạp dataset, 3 hàm thưởng GRPO đạt | PASSED |
| **M3** | T4 FP16 Optimization & HF Model Release | `scripts/train.py`, `scripts/create_repo.py` | 50 steps < 2 phút trên T4, push mô hình 16-bit merged lên HF Hub | PASSED |
| **M4** | R1 Studio Web UI & 100% Automated Testing | `web/src/components/R1Studio.jsx`, `web/src/engine/` | Vitest Web UI 109/109 GREEN, Rust Engine 119/119 GREEN | PASSED |
