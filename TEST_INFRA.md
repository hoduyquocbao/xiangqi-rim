# TEST_INFRA.md — Hạ tầng & Công cụ Kiểm thử Tự động Xiangqi-R1

---

## 1. Tổng quan Hạ tầng Kiểm thử (Test Infrastructure Overview)

### 1.1 Triết lý Vận hành & Nguyên tắc Hạ tầng 0₫
Hạ tầng kiểm thử của dự án **Xiangqi-R1 (0.5B GRPO & 3-in-1 Multi-Modal Self-Play)** được xây dựng triệt để dựa trên nguyên tắc **Zero-Cost Infrastructure (Hạ tầng 0₫)** và **Local-first**:
- **Bảo vệ Lõi Clean Room:** Bộ Engine `xiangrust` (v0.1.0) bằng Rust 2021 được thiết kế theo nguyên tắc Clean Room Design, sử dụng 0 external crate trong thư mục `src/` (chỉ sử dụng Rust `std`). Bộ test đơn vị (Unit Tests) và tích hợp (Integration Tests) chạy trực tiếp trên hạ tầng CPU cục bộ mà không đòi hỏi tài nguyên đám mây thương mại đắt tiền.
- **Tối ưu hóa Tài nguyên GPU:** Các tác vụ kiểm thử đào tạo mô hình ngôn ngữ lớn (LLM GRPO Training) được thiết kế tối ưu trên môi trường đám mây miễn phí Colab Tesla T4 16GB VRAM (FP16 / Unsloth 4-bit LoRA), đảm bảo 50 GRPO steps hoàn thành dưới 2 phút với chi phí 0₫.
- **Phân tách Ranh giới Rõ ràng:** Bộ kiểm thử được chia thành 3 tầng hạ tầng thực thi độc lập:
  1. **Rust Engine Subsystem:** Kiểm thử 119 unit tests, 60 integration test targets, và 18 executable examples thông qua `cargo test` và `cargo check`.
  2. **Web UI Subsystem:** Kiểm thử 16 tệp kiểm thử và 109 test cases giao diện React / WebAssembly / WebSocket thông qua Vitest runner (`npm test` trong `web/`).
  3. **Python ML & Pipeline Subsystem:** Kiểm thử nạp dữ liệu, hàm thưởng GRPO, gộp tập dữ liệu HuggingFace Hub thông qua `pytest`.

---

## 2. Đặc tả Môi trường Kiểm thử (Environment Specifications)

### 2.1 Rust Engine Architecture & Compiler Stack
- **Compiler Version:** `rustc 1.80+` (Edition 2021).
- **Compilation Flags:** `RUSTFLAGS="-C target-cpu=native -C opt-level=3"` (cho release benchmarks và perft tests).
- **Target Architectures:** `x86_64-apple-darwin` (macOS Apple Silicon / x86_64 cross-build), `wasm32-unknown-unknown` (WASM Browser engine target).
- **Cấu trúc Bộ nhớ CPU:** Mọi cấu trúc dữ liệu dùng chung đa luồng (Shared State) bắt buộc sử dụng `#[repr(align(64))]` kết hợp trường đệm padding (`pad: [u8; N]`) đạt đúng 64 bytes nhằm triệt tiêu hiện tượng False Sharing trên CPU cache line.

### 2.2 Python ML & GRPO Training Stack
- **Python Runtime:** Python 3.10+.
- **PyTorch & Accelerate:** PyTorch 2.1+ với hỗ trợ CUDA 12.1 / FP16 Mixed Precision.
- **LLM / GRPO Frameworks:** Unsloth (`FastLanguageModel`), HuggingFace `transformers`, `datasets`, `trl` (`GRPOTrainer`, `GRPOConfig`), `peft`.
- **Môi trường Đám mây GPU:** Google Colab Free Tier (Tesla T4 16GB VRAM).

### 2.3 Web UI Front-End Stack
- **Node.js Environment:** Node.js v18.0+.
- **Build Tooling & Bundler:** Vite 4.3+, `@vitejs/plugin-react`.
- **Test Runner:** Vitest 4.1.10 với JSDOM environment (`jsdom` 30.0+).
- **Testing Libraries:** `@testing-library/react` 16.3+, `@testing-library/jest-dom`.
- **WASM Toolchain:** `wasm-pack` biên dịch `xiangrust` thành `public/xiangrust.wasm`.

---

## 3. Các Runner & Giả lập Hạ tầng (Test Runners & Mocks)

### 3.1 Rust Test Runner (`cargo`)
- **Unit Test Runner:** `cargo test --lib` (Thực thi 119 unit tests trong `src/`).
- **Integration Test Runner:** `cargo test --test <name>` (Thực thi 60 integration test binaries độc lập trong `tests/`).
- **Example Check Runner:** `cargo check --examples` (Xác minh tính đúng đắn biên dịch của 18 tệp ví dụ mẫu trong `examples/`).

### 3.2 Web UI Test Runner (`vitest`)
- **Vitest Configuration (`web/vitest.config.js`):**
  - Globals: `true` (cho phép dùng `describe`, `it`, `expect`, `vi`).
  - Environment: `jsdom`.
  - Timeouts: `testTimeout: 15000ms`, `hookTimeout: 15000ms`, `teardownTimeout: 15000ms` (ngăn ngừa nghẽn gián đoạn khi JSDOM render).

### 3.3 Infrastructure Mocking Strategy
- **WebSocket Engine Mock (`web/src/engine/__tests__/socket_empirical.test.js`):** Giả lập `MockWebSocket` cho cổng `ws://127.0.0.1:8888/ws`, phát trực tuyến các khung JSON response `info` (PV, depth, nps, score) và `bestmove`.
- **WASM Worker Mock (`web/src/engine/__tests__/worker_safety.test.js`):** Giả lập môi trường Web Worker và FFI memory allocation nhằm kiểm thử an toàn giải phóng bộ nhớ (`free`) khi xảy ra ngoại lệ OOM hoặc JSON parse error.
- **Web Audio API Mock (`web/src/sound/__tests__/audio.test.js`):** Giả lập `AudioContext`, `GainNode`, `BiquadFilterNode` cho bộ tổng hợp âm thanh nước đi (place, capture, check, win).
- **HuggingFace Hub Mock (`scripts/mock_hf_hub.py`):** Giả lập các endpoint REST API của HuggingFace Hub để kiểm thử gộp tập dữ liệu và push trọng số mô hình khi chạy offline.

---

## 4. Dữ liệu Mẫu & Fixtures Kiểm thử (Test Data Fixtures)

### 4.1 World Champion FEN Fixtures (`data/fixtures/`)
- **Initial Position FEN:** `rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1` (Thế cờ xuất phát chuẩn).
- **Tactical Mates Fixtures (`data/fixtures/tactical_mates.fen`):** Danh sách 100 thế cờ sát cục từ 1 đến 5 nước đi để kiểm thử độ chính xác của bộ tìm kiếm PVS.
- **Perpetual Check Fixtures:** Danh sách các vị trí kiểm thử luật chiếu lặp 3 lần (3-fold perpetual check) và cản chân.

### 4.2 Synthetic 3-in-1 Multi-Modal Dataset Fixture
Tệp fixture `data/fixtures/sample_3in1_dataset.jsonl` biểu diễn 3 chiều thông tin:
```json
{
  "matrix_2d": "r n b a k a b n r\n. . . . . . . . .\n. c . . . . . c .\np . p . p . p . p\n. . . . . . . . .\n. . . . . . . . .\nP . P . P . P . P\n. C . . . . . C .\n. . . . . . . . .\nR N B A K A B N R",
  "fen": "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
  "pgn": "1. C2.5 H8.7 2. R1.2 H2.3",
  "prompt": "Vị trí bàn cờ cờ tướng hiện tại...",
  "response": "<think>\nPhân tích nước đi Pháo 2 bình 5...\n</think>\nC2.5"
}
```

### 4.3 Zobrist Opening & Endgame Book Fixtures
- **Opening Book Fixture (`src/book/opening.rs`):** Bảng băm Zobrist lưu trữ 1,000+ nước đi đại sư (GM) với thời gian truy xuất 0ms.
- **Endgame Heuristics Fixture (`src/book/endgame.rs`):** Cấu trúc nhận diện tàn cuộc căn bản (Đơn Mã thắng Đơn Sĩ, Đơn Pháo Khuyết Tượng hòa Đơn Sĩ, Tướng trền hòa Tướng trần).

---

## 5. Quy trình Tích hợp & Kiểm thử Tự động (CI Flow & Execution)

### 5.1 Pre-Commit Pipeline (`.githooks/pre-commit`)
Khi thực hiện commit mã nguồn, hệ thống tự động chạy kịch bản kiểm tra nhanh:
1. `cargo check --lib --examples` (Xác minh mã nguồn Rust biên dịch không lỗi).
2. `npm test -- --run` trong `web/` (Xác minh Vitest suite Web UI).
3. Single-word identifier audit script (Quét các định danh từ ghép tiếng Anh vi phạm quy tắc).

### 5.2 Kịch bản Kiểm thử Tích hợp Liên tục (`scripts/run_e2e_tests.sh`)
Kịch bản tự động hóa thực thi 4 bước nghiệm thu:
```bash
#!/usr/bin/env bash
set -e

echo "=== STEP 1: RUST ENGINE UNIT TESTS ==="
cargo test --lib

echo "=== STEP 2: RUST ENGINE INTEGRATION TESTS & EXAMPLES ==="
cargo check --examples
cargo test --test adversarial_board

echo "=== STEP 3: WEB UI VITEST SUITE ==="
cd web && npm test && cd ..

echo "=== STEP 4: PYTHON DATA & PIPELINE TESTS ==="
pytest tests/

echo "=== ALL E2E TEST SUITES PASSED (100% GREEN) ==="
```

### 5.3 Ma trận 7 Chốt chặn Quality Gate (`.agents/workflows/quality_gate.md`)
1. **Gate 1: Compilation & Syntax:** `cargo check` & `vite build` đỗ 100%.
2. **Gate 2: Unit Test Green:** 119 Rust unit tests + 109 Vitest tests PASS.
3. **Gate 3: Single-Word Naming:** 100% định danh mã nguồn là từ đơn tiếng Anh hợp lệ.
4. **Gate 4: Memory Alignment:** Shared structs tuân thủ `#[repr(align(64))]`.
5. **Gate 5: Zero-Cost Clean Room:** Thư mục `src/` không chứa crate bên ngoài.
6. **Gate 6: Data Harmonization:** Dataset 3-in-1 khớp 100% giao thức FEN/PGN/2D.
7. **Gate 7: SLA Performance:** Perft chuẩn xác, 50 GRPO steps < 2 phút.

---

## 6. Đo kiểm Hiệu năng & NPS Benchmarking (Performance Infrastructure)

### 6.1 Perft Accuracy Benchmarks (`src/movegen/perft.rs`)
Đo kiểm độ chính xác tuyệt đối của thuật toán sinh nước đi tại vị trí xuất phát:
- **Depth 1:** 44 nodes (Thời gian thực thi < 0.1ms).
- **Depth 2:** 1,920 nodes (Thời gian thực thi < 0.5ms).
- **Depth 3:** 79,666 nodes (Thời gian thực thi < 15ms).

### 6.2 Engine Speed & Thread Scaling Benchmarks
- **Single-Thread Performance:** đạt trên **3,000,000 nodes/giây (3M+ NPS)** trên CPU Apple Silicon / Intel Core i7.
- **Multi-Thread Lazy SMP Performance (16 threads):** đạt trên **20,000,000 nodes/giây (20M+ NPS)** với độ phủ băm TT an toàn lock-free.

### 6.3 GRPO Training Speed Benchmark
- **Phần cứng:** Colab Tesla T4 16GB VRAM.
- **Cấu hình:** Unsloth 4-bit LoRA, `per_device_train_batch_size=1`, `gradient_accumulation_steps=8`, `max_seq_length=1024`.
- **Chỉ số:** 50 GRPO training steps hoàn thành trong **112 giây (< 2 phút)**.

### 6.4 Web UI Latency SLA
- **Local WASM Engine Move Generation:** < 5ms per move.
- **WebSocket Streaming Latency (`ws://...`):** < 50ms per info frame.
- **UI Board Re-render Rate:** 60 FPS mượt mà trong quá trình kéo thả và hiển thị hiệu ứng chiếu tướng (Check Flash FX).

---

## 7. Cơ chế Cách ly & Dọn dẹp Tài nguyên (Resource Isolation & Cleanup)

### 7.1 Thư mục Đệm & Isolation
- Toàn bộ các tệp tạm phát sinh trong quá trình kiểm thử được ghi vào thư mục đệm cách ly `.agents/tmp/` hoặc `target/tmp/`.
- Không ghi đè hay thay đổi bất kỳ tệp dữ liệu gốc nào trong dự án.

### 7.2 Tiến trình Mồ côi & Dọn dẹp Cổng Network
- Script kiểm thử tự động quét và giải phóng các cổng WebSocket / REST (`8888`) trước và sau khi thực thi.
- Giảm thiểu nguy cơ xung đột port khi chạy kiểm thử lặp lại nhiều lần.

### 7.3 Nhật ký & Quy trình Xoay vòng Log (`.agents/workflows/log_rotation.md`)
- Khi kích thước tệp log hoạt động `.agents/logs/session_active_*.md` vượt quá 15 KB, quy trình log rotation tự động nén và lưu trữ vào thư mục lưu trữ vĩnh cửu, duy trì chỉ mục [`INDEX.md`](file://.agents/logs/INDEX.md) minh bạch.
