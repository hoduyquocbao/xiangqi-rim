# Original User Request

## 2026-08-04T14:21:29Z

# THIẾT KẾ & XÂY DỰNG ENGINE CỜ TƯỚNG "XIANGRUST" (RUST 2021)

Working directory: /Users/hdqb/workspaces/backend-xiangqi-ai-debugger-v1
Integrity mode: development

## TỔNG QUAN VÀ ĐẶC TẢ KỸ THUẬT

### 1. Yêu cầu hệ thống
- Tên dự án: XiangRust (`xiangrust`)
- Ngôn ngữ: Rust 2021 Edition, MIT License, UCI v2 protocol.
- Clean Room Design: Không phụ thuộc crate bên ngoài trong `src/` (chỉ sử dụng Rust `std`).
- Mục tiêu hiệu năng: 3M nodes/s (đơn luồng), 20M+ nodes/s (đa luồng).

### 2. Kiến trúc & Các Module Cốt lõi
- **Board (`src/board/`)**: `Square` (0..89), `Piece` (mã hóa loại+màu), `Move` (16-bit), `Bitboard` (128-bit align 16), `Position` (420 bytes, align 64), `Zobrist`, `StateInfo`, FEN parser/serializer.
- **MoveGen (`src/movegen/`)**: Sinh nước đi pseudo-legal & legal dựa trên Lookup Tables (`KING_ATTACKS`, `ADVISOR_ATTACKS`, `KNIGHT_ATTACKS`, `ELEPHANT_ATTACKS`, `PAWN_ATTACKS`, `ROOK_ATTACKS`, `CANNON_ATTACKS`).
- **Eval (`src/eval/`)**: NNUE `HalfKAv2_hm` feature extraction (65k features), `Accumulator` gia tăng, `FeatureTransformer`, `AffineTransform` với SIMD (AVX2/NEON/AVX-512), `OutputLayer`, kết hợp `HCE` (Hand-Crafted Evaluation) làm dự phòng.
- **Search (`src/search/`)**: PVS (Principal Variation Search), Quiescence Search, Aspiration Window, LMR, Null Move Pruning, Futility Pruning, Check Extensions, History/Killer/Counter Tables, Time Manager.
- **Transposition Table (`src/tt/`)**: TTEntry 16 bytes (AtomicU64, lock-free, align 16), TranspositionTable (align 64).
- **Threading (`src/thread/`)**: Lazy SMP Zero-Lock ThreadPool.
- **UCI (`src/uci/`)**: UCI Parser (`position`, `go`, `stop`, `setoption`, `ucinewgame`, `isready`, `uci`, `quit`), Engine.
- **CQRS-ES (`src/cqrs/`)**: Tách biệt Command, Query và Event Bus.
- **Circuit Breaker (`src/circuit/`)**: CircuitBreaker quản lý trạng thái Closed/Open/HalfOpen cho NNUE sang HCE fallback.

## ACCEPTANCE CRITERIA
- Mã nguồn biên dịch thành công với `cargo build --release` (Clean compile, no warnings/errors).
- Perft test thành công (Depth 1 = 44 nodes cho FEN bàn cờ ban đầu) và pass toàn bộ `cargo test`.
- Hoạt động chính xác theo giao thức UCI v2.
- Đạt mục tiêu hiệu năng O(1) cache-friendly và 3M+ nodes/s.

## 2026-08-05T07:36:04Z

# Teamwork Project Prompt — Draft

Nâng cấp hiệu năng Engine Cờ Tướng XiangRust (tăng NPS từ ~6M lên 9M-12M+), tối ưu hóa các điểm nghẽn bộ nhớ/CPU, tạo thư mục `examples/` trực quan hóa cách sử dụng lõi SDK/Framework, và báo cáo phân tích nguyên nhân suy giảm NPS.

Working directory: `/Users/hdqb/workspaces/backend-xiangqi-ai-debugger-v1`
Integrity mode: development

## Requirements

### R1. Phân tích nguyên nhân & Tối ưu hoá Hiệu suất (NPS Diagnostics & Performance Optimization)
- Tiến hành phân tích sâu nguyên nhân tại sao hiệu năng tính toán (NPS) bị sụt giảm từ 9M xuống ~6M nodes/s (kiểm tra lock contention, atomic operations, cache miss, perft vs search overhead, memory alignment).
- Refactor tối ưu hóa các điểm nghẽn hiệu năng trong `src/` để khôi phục và nâng tầm hiệu suất đạt 9M+ nodes/s (đơn luồng/đa luồng).

### R2. Thư mục Ví dụ Mẫu (`examples/`)
Xây dựng đầy đủ các tệp mã nguồn mẫu minh họa trực quan trong `examples/`:
- `examples/01_board_and_fen.rs`: Hướng dẫn khởi tạo bàn cờ, đọc/ghi FEN, Zobrist Hashing và thao tác Bitboard.
- `examples/02_move_generation.rs`: Hướng dẫn sinh nước đi pseudo-legal, legal và đo kiểm Perft.
- `examples/03_evaluation_nnue.rs`: Hướng dẫn tính điểm NNUE HalfKAv2_hm tích lũy gia tăng Accumulator và HCE fallback.
- `examples/04_search_engine.rs`: Hướng dẫn chạy bộ tìm kiếm PVS, Aspiration Window, và thiết lập Limits.
- `examples/05_uci_protocol.rs`: Hướng dẫn tích hợp và vận hành giao thức UCI v2 STDIN/STDOUT.
- `examples/06_cqrs_event_bus.rs`: Hướng dẫn kiến trúc CQRS Command/Query/Event Bus & máy ngắt mạch Circuit Breaker.

### R3. Chất lượng Mã nguồn & Chú thích Siêu Chi tiết (Hyper-Detailed Vietnamese Documentation)
- Mọi tệp mã nguồn ví dụ mới trong `examples/` và các đoạn mã refactor đều phải có chú thích tiếng Việt siêu chi tiết tới từng dòng mã.
- Tuân thủ nghiêm ngặt quy tắc định danh từ đơn tiếng Anh (Single-Word Identifiers) và căn lề bộ nhớ phần cứng (`repr(C, align(64))`/`align(16)`).

## Acceptance Criteria

### Verification & Quality Criteria
- [ ] Mã nguồn biên dịch thành công 100% với `cargo build --release` và `cargo check --examples`.
- [ ] Chạy thành công toàn bộ `cargo test --release` (100/100 tests PASSED).
- [ ] Phân tích và đưa ra báo cáo rõ ràng về lý do NPS bị giảm và giải pháp tối ưu thành công.
- [ ] Đạt/Vượt mốc hiệu năng mục tiêu (>= 9M+ nodes/s).


## 2026-08-05T15:31:46Z

# Teamwork Project Prompt — Draft

Nghiên cứu, phân tích và thực nghiệm chẩn đoán giới hạn vật lý CPU (Physical CPU Limits) cho chỉ số NPS (Nodes Per Second) của Engine Cờ Tướng XiangRust. Xác định rõ giới hạn hiện tại đang thiên về phía Kiến trúc mã nguồn (Software Architecture) hay Giới hạn vật lý phần cứng CPUs (Hardware Limits), đồng thời lập báo cáo phân tích vật lý chuyên sâu `reports/cpu_physical_limit_analysis.md`.

Working directory: `/Users/hdqb/workspaces/backend-xiangqi-ai-debugger-v1`
Integrity mode: development

## Requirements

### R1. Phân tích Toán học & Vật lý Chu kỳ CPU (Clock Cycles & Hardware IPC Analysis)
- Phân tích chi tiết mức tiêu tốn chu kỳ xung nhịp CPU (CPU clock cycles per node) trên từng đơn vị thao tác: Bitboard bitwise, MoveGen, NNUE Accumulator SIMD vectorization, Transposition Table atomic accesses.
- Lập mô hình toán học tính toán giới hạn vật lý lý thuyết tối đa (Theoretical Maximum NPS) của 1 nhân CPU ở xung nhịp cụ thể (ví dụ 4.0 GHz) khi đạt IPC tối đa (Instructions Per Cycle = 4..8 ops/cycle).

### R2. Đánh giá Điểm nghẽn Kiến trúc Phần mềm vs Vật lý Phần cứng (Software Architecture vs Hardware Limit Bottleneck Assessment)
- So sánh khoảng cách giữa NPS thực tế của XiangRust (10.07M single-thread, 72.8M 4-thread) với giới hạn lý thuyết vật lý.
- Đưa ra kết luận minh bạch và chính xác: Giới hạn NPS hiện tại đang bị ràng buộc bởi yếu tố nào (Memory Bandwidth, Cache Latency, Atomic Contention, SIMD Latency, hay Branch Misprediction).

### R3. Báo cáo Nghiên cứu & Chú thích Siêu Chi tiết (Comprehensive Report & Documentation)
- Xuất bản báo cáo chuyên sâu `reports/cpu_physical_limit_analysis.md` bằng Tiếng Việt 100%, diễn giải toán học & vật lý vi xử lý chuyên nghiệp.
- Mọi kịch bản đo kiểm benchmark bổ sung (nếu có) phải có chú thích siêu chi tiết tới từng dòng mã.

## Acceptance Criteria

### Verification & Quality Criteria
- [ ] Báo cáo `reports/cpu_physical_limit_analysis.md` được tạo đầy đủ với phân tích toán học và số liệu thực nghiệm.
- [ ] Phân tích rõ ràng giới hạn lý thuyết NPS của 1 nhân CPU (ví dụ: ở 4GHz, 1 node tiêu tốn bao nhiêu cycles -> max NPS lý thuyết).
- [ ] Đánh giá chính xác tỷ trọng nghẽn giữa Kiến trúc phần mềm (Software Architecture) vs Giới hạn vật lý CPUs (Hardware Limits).
- [ ] Đảm bảo toàn bộ mã nguồn kiểm thử biên dịch và chạy thành công 100% (`cargo test --release` PASSED).

## 2026-08-05T16:19:20Z

# Teamwork Project Prompt — Draft

Phân tích và chẩn đoán hiện tượng suy giảm NPS khi tăng số luồng từ 4 threads (72.8M NPS) lên 8 threads và 16 threads (38.5M NPS) trong kiến trúc Lazy SMP của XiangRust. Lập báo cáo phân tích chi tiết vi kiến trúc CPU và xung đột bộ nhớ đệm `reports/smp_thread_scaling_analysis.md`.

Working directory: `/Users/hdqb/workspaces/backend-xiangqi-ai-debugger-v1`
Integrity mode: development

## Requirements

### R1. Phân tích Vi kiến trúc Nhân CPU (P-cores vs E-cores & Hardware Topology)
- Phân tích ảnh hưởng của cấu trúc nhân vi xử lý (Performance Cores vs Efficiency Cores) trên vi xử lý Mac Apple Silicon và hiện tượng tranh chấp tài nguyên SMT/Hyper-Threading.
- Đo kiểm vi phân tích sự khác biệt về tốc độ xử lý (cycles per node) khi luồng chạy trên nhân P-core so với nhân E-core.

### R2. Phân tích Tranh chấp Bộ nhớ Đệm Nguyên tử & Bão hòa L2/L3 Cache (Atomic Cache Line Bouncing & Coherency Protocol)
- Phân tích cơ chế Cache Coherency (giao thức MESI/MOESI) khi 16 luồng đồng thời thực hiện thao tác băm nguyên tử `AtomicU64` vào Transposition Table dùng chung.
- Định lượng chi phí dọn đệm L1/L2 (Cache Line Bouncing) tăng theo cấp số nhân $O(N^2)$ từ 4 luồng ($4^2=16$) lên 16 luồng ($16^2=256$).

### R3. Báo cáo Chuyên sâu & Giải pháp Tối ưu (Comprehensive Report & Optimization Recommendations)
- Xuất bản báo cáo chuyên sâu `reports/smp_thread_scaling_analysis.md` bằng Tiếng Việt 100%, diễn giải toán học & vật lý vi xử lý chuyên nghiệp.
- Đề xuất các giải pháp nâng cao hiệu năng Lazy SMP (như Thread Affinity / Core Pinning, TT Cluster Partitioning, Depth Helper Offsets).

## Acceptance Criteria

### Verification & Quality Criteria
- [ ] Báo cáo `reports/smp_thread_scaling_analysis.md` được tạo đầy đủ với phân tích toán học và số liệu thực nghiệm.
- [ ] Giải thích minh bạch 3 nguyên nhân cốt lõi tại sao 8/16 threads chậm hơn 4 threads (Topology P/E-cores, Cache Line Bouncing, TT Collision).
- [ ] Đề xuất phương án tối ưu nâng cao hiệu năng Lazy SMP đa luồng.
- [ ] Đảm bảo toàn bộ mã nguồn kiểm thử biên dịch và chạy thành công 100% (`cargo test --release` PASSED).

## 2026-08-05T09:55:08Z

# Teamwork Project Prompt — Draft

Thực hiện mọi biện pháp tối ưu hóa hiệu năng tối đa (Maximum Performance Optimization) cho Engine Cờ Tướng XiangRust: Triển khai Thread Affinity (P-Core Pinning), TT Cluster Partitioning, Search Diversification cho Lazy SMP, tối ưu hóa SIMD NNUE và hot path inlining để đạt NPS tối đa cả đơn luồng và đa luồng.

Working directory: `/Users/hdqb/workspaces/backend-xiangqi-ai-debugger-v1`
Integrity mode: development

## Requirements

### R1. Tối ưu hóa Đa luồng Lazy SMP Tối đa (Lazy SMP Maximization & Affinity)
- Triển khai `Affinity` struct định tuyến luồng ưu tiên P-cores (`QOS_CLASS_USER_INTERACTIVE` trên macOS).
- Triển khai `Partition` / TT Sharding triệt tiêu bùng nổ MESI Cache Line Bouncing giữa các luồng.
- Triển khai `Diversity` search offset số nguyên tố và History Bias scaling giảm tỷ lệ trùng lặp cây từ 68% xuống ~12%, nâng NPS đa luồng thực tế vọt mốc 100M+ NPS.

### R2. Tối ưu hóa Hot Loop & SIMD Acceleration (SIMD & Core Inlining)
- Rà soát và thêm chỉ thị `#[inline(always)]` cho toàn bộ các hàm hot path trong `board`, `movegen`, `eval`, `tt`, `search`.
- Tối ưu hóa unrolling 64-way/32-way SIMD cho NNUE Accumulator `add`, `update`, `modify` và Affine transform.

### R3. Đảm bảo Tuân thủ Quy tắc & Kiểm thử Tuyệt đối (Strict Rules & Testing)
- Giữ vững 100% Clean Room Design (0 external crates trong `src/`).
- Duy trì 100% định danh từ đơn tiếng Anh (Single-Word Identifiers) và căn lề bộ nhớ `align(64)` / `align(16)`.
- Duy trì chú thích tiếng Việt chi tiết tới từng dòng mã cho các phần mã nguồn nâng cấp.
- Pass 100% bộ kiểm thử tự động `cargo test --release` và `cargo check --examples`.

## Acceptance Criteria

### Verification & Quality Criteria
- [ ] Mã nguồn biên dịch thành công 100% với `cargo build --release` và `cargo check --examples`.
- [ ] Chạy PASSED 100% bộ unit & integration tests (`cargo test --release`).
- [ ] Tối ưu hóa Lazy SMP đa luồng đẩy NPS đa luồng đạt mốc vượt trội (>= 100M+ NPS).
- [ ] Mọi tệp refactor giữ nguyên chú thích tiếng Việt siêu chi tiết tới từng dòng mã.

## 2026-08-06T01:53:33Z

# THIẾT KẾ & XÂY DỰNG MODULE TỰ ĐẤU (SELF-PLAY SIMULATION) & SÁCH KHAI CUỘC / TÀN CUỘC ĐẲNG CẤP CHO XIANGRUST

Working directory: `/Users/hdqb/workspaces/backend-xiangqi-ai-debugger-v1`
Integrity mode: development

## Requirements

### R1. Triển khai Module Tự đấu Self-Play (`examples/12_self_play_simulation.rs` hoặc `src/selfplay/`)
- Xây dựng công cụ mô phỏng trận đấu tự đấu (Self-Play Match Engine) giữa các cấu hình AI hoặc giữa các độ sâu khác nhau.
- Đo đạc chi tiết chỉ số NPS trung bình, số nút/nước, thời gian suy nghĩ từng nước, tỷ lệ thắng/hòa/thua, xuất báo cáo PGN/FEN và phát hiện bẫy lặp lại (Repetition rule check).

### R2. Triển khai Thư viện Khai cuộc Zobrist Hash O(1) (`src/book/opening.rs` & `examples/13_opening_and_endgame_book.rs`)
- Tích hợp bộ Khai cuộc băm Zobrist $O(1)$ với 1,000+ nước đi biến thể kinh điển thế giới: Pháo Đầu (Trung Pháo), Bình Phong Mã, Khởi Mã Cuộc, Quá Cung Pháo, Thuận Pháo, Nghịch Pháo, Sĩ Tiến Pháo, Tiến Binh Cuộc.
- Tra cứu nước đi khai cuộc chuẩn Grandmaster trong **0ms** dựa trên Zobrist Key của bàn cờ.

### R3. Triển khai Tri thức Tàn cuộc Chuyên sâu (`src/book/endgame.rs`)
- Tích hợp tri thức tàn cuộc thực dụng 0-dependency (Endgame Tablebase / Heuristics): Đơn Mã thắng Đơn Sĩ, Đơn Pháo Khuyết Tượng hòa Đơn Sĩ, Xe Mã thắng Xe Sĩ Tượng, Hai Pháo thắng Khuyết Sĩ Tượng, v.v.
- Tự động nhận diện thế cờ tàn cuộc lý thuyết để gán điểm bonus/penalty chính xác tuyệt đối, tránh hòa vô lý hoặc bỏ lỡ cơ hội sát thủ.

### R4. Bộ Tài liệu Đặc tả & Chú thích Chi tiết (`docs/`)
- Soạn thảo tài liệu đặc tả chuyên sâu `docs/self_play_and_books.md` 100% tiếng Việt giải thích chi tiết cơ chế Self-Play, cấu trúc Zobrist Opening Book và luật tàn cuộc.
- Chú thích tiếng Việt 100% từng dòng mã nguồn, mã nguồn 100% tiếng Anh tuân thủ quy tắc từ đơn (Single-Word Identifiers) và căn lề bộ nhớ `repr(C, align(64))`.

## Acceptance Criteria

### Verification & Quality Criteria
- [ ] `cargo check --examples` và `cargo build --release` biên dịch 100% thành công không warning/error.
- [ ] 100% Unit tests và Integration tests PASSED (`cargo test --release`).
- [ ] Chạy ví dụ `12_self_play_simulation` hoàn thành các trận tự đấu liên tục không văng lỗi, xuất báo cáo thống kê NPS & nước đi đầy đủ.


## 2026-08-06T07:28:35Z

# THIẾT KẾ & XÂY DỰNG THUẬT TOÁN AI TỰ THUẤN LUYỆN THÍCH ỨNG (ONLINE REINFORCEMENT LEARNING & PERSISTENT MEMORY) CHO XIANGRUST

Working directory: `/Users/hdqb/workspaces/backend-xiangqi-ai-debugger-v1`
Integrity mode: development

## Requirements

### R1. Triển khai Module Học thích ứng Online (`src/learn/` & `examples/14_online_learning_and_trainer.rs`)
- Triển khai thuật toán Học kinh nghiệm Online (Experience Replay & Temporal Difference Learning $TD(\lambda)$) tự động ghi nhận ván đấu và điều chỉnh trọng số lịch sử/đánh giá.
- Tự động phát hiện các ván đấu thua do sai lầm (Blunder Analysis) để điều chỉnh điểm phạt (Penalty Bias) tránh lặp lại cùng sai lầm trong tương lai.

### R2. Lưu trữ Trí nhớ Kinh nghiệm Lâu dài trên Ổ đĩa (Persistent Memory Storage)
- Xây dựng cơ chế lưu vết & nạp tự động Bảng Kinh nghiệm (Experience Memory Storage) xuống tệp nhị phân / JSON để duy trì trí nhớ học tập bền vững giữa các lần khởi động lại máy chủ.
- Tự động đồng bộ các nước đi tốt mới vào Bảng băm Zobrist Opening Book & Endgame Memory Table.

### R3. Tối ưu hóa Phương trình Suy luận & Adaptive Search Limits
- Xây dựng thuật toán tự điều chỉnh độ sâu (Adaptive Depth & Time Manager) dựa trên độ phức tạp của thế cờ (Board Complexity Equation).
- Tự động nới rộng/thu hẹp cửa sổ Aspiration Window và LMR reduction dựa trên độ ổn định của tuyến PV.

### R4. Bộ Tài liệu Đặc tả Toán học & Hướng dẫn Huấn luyện (`docs/online_learning_architecture.md`)
- Soạn thảo tài liệu đặc tả 100% tiếng Việt diễn giải toán học các phương trình $TD(\lambda)$, Experience Replay, và thuật toán tự học.
- Chú thích tiếng Việt 100% từng dòng mã nguồn, mã nguồn 100% tiếng Anh tuân thủ quy tắc từ đơn (Single-Word Identifiers) và căn lề bộ nhớ `repr(C, align(64))`.

## Acceptance Criteria

### Verification & Quality Criteria
- [ ] `cargo check --examples` và `cargo build --release` biên dịch 100% thành công không warning/error.
- [ ] 100% Unit tests và Integration tests PASSED (`cargo test --release`).
- [ ] Chạy thử nghiệm `14_online_learning_and_trainer` tích lũy thành công kinh nghiệm qua 10+ ván tự đấu, lưu và nạp lại thành công trí nhớ từ ổ đĩa.
- [ ] Tỷ lệ thắng/hòa của AI sau khi học thích ứng tăng rõ rệt so với phiên bản ban đầu.
- [ ] Mọi tệp ví dụ và tài liệu hướng dẫn đạt tiêu chuẩn sản phẩm đẳng cấp, chuyên nghiệp.

## 2026-08-06T17:22:28Z

# THIẾT KẾ & XÂY DỰNG NỀN TẢNG GIA TỐC GPU TÍCH HỢP (INTEL iGPU 512MB) CHO GYM DEPTH 12 XIANGRUST

Working directory: `/Users/hdqb/workspaces/backend-xiangqi-ai-debugger-v1`
Integrity mode: development

## Requirements

### R1. Kiến Trúc Adapter / Wrapper GPU Đa Nền Tảng (`src/gpu/`)
- Thiết kế lớp trừu tượng `GpuAdapter` (Ports & Adapters Pattern) tự động ưu tiên Metal API Native trên macOS cho Intel iGPU 512MB, đồng thời hỗ trợ bộ chuyển đổi dự phòng OpenCL / WGPU Compute Shaders.
- Quản lý bộ nhớ VRAM an toàn 512MB, ngăn ngừa tràn bộ nhớ đệm (VRAM Out-of-Memory Protection).

### R2. Gia Tốc Kép GPU (NNUE Batch Evaluation & Parallel Search Evaluator)
- **NNUE Batch Evaluation**: Gửi đồng thời lô hàng nghìn thế cờ (Batch Positions) lên iGPU để nhân ma trận song song (Matrix Multiplication & SIMD Unrolling), giảm 0ms CPU overhead.
- **Parallel Search Evaluator**: Hỗ trợ duyệt hàng loạt nút lá (Leaf Nodes Evaluation) song song trên GPU cho luồng GYM Depth 12, tăng vọt chỉ số NPS tổng thể.

### R3. Tích Hợp Luồng Tự Huấn Luyện Ngầm GYM (`src/learn/gym.rs`)
- Tích hợp pipeline tính toán GPU vào luồng GYM Depth 12, cho phép CPU tập trung duyệt nhánh PVS và GPU chịu trách nhiệm đánh giá thế cờ song song.

## Acceptance Criteria

### Verification & Quality Criteria
- [ ] Mã nguồn Rust biên dịch 100% thành công không warning/error với `cargo check --release`.
- [ ] Nhận diện và phát hiện thành công Intel iGPU 512MB trên macOS qua Metal API / OpenCL.
- [ ] 100% unit tests và integration tests PASSED (`cargo test --release`).
- [ ] Tốc độ tính toán GYM Depth 12 tăng vọt rõ rệt so với chạy đơn thuần trên CPU.
- [ ] Tuân thủ 100% quy tắc định danh từ đơn tiếng Anh (Single-Word Identifiers) và chú thích tiếng Việt chi tiết tới từng dòng mã.

## 2026-08-06T17:25:48Z

# CHỈ THỊ BỔ SUNG QUAN TRỌNG TỪ ANH HDQB: CHỐNG NGHẼN BĂNG THÔNG GIAO TIẾP CPU-GPU (ZERO-COPY & AUTONOMOUS LOGIC)

LƯU Ý ĐẶC BIỆT: Chi phí truyền nhận dữ liệu qua lại giữa CPU và GPU (Bus Transfer Overhead) là cực kỳ cao. Nếu mỗi nút (node) lại gửi dữ liệu qua lại giữa CPU và GPU thì chi phí giao tiếp sẽ triệt tiêu toàn bộ lợi ích hiệu năng!

YÊU CẦU THIẾT KẾ CẮT GIẢM NGHẼN GIAO TIẾP:
1. **Tận Dụng Bộ Nhớ Dùng Chung Unified Memory 0-Copy (Shared Storage Mode)**: Với Intel iGPU tích hợp trên macOS, CPU và GPU dùng chung RAM vật lý. BẮT BUỘC sử dụng `MTLResourceStorageModeShared` (hoặc `CL_MEM_USE_HOST_PTR`) để đọc/ghi bộ nhớ 0-copy, triệt tiêu 100% chi phí copy dữ liệu qua Bus!
2. **Asynchronous Ring Buffer Queue (Hàng Đợi Vòng Bất Đồng Bộ)**: Độc lập logic tính toán trên GPU và CPU. CPU đẩy các lô thế cờ vào Ring Buffer bất đồng bộ lock-free, GPU tự động xử lý theo lô (Chunk Batch) mà không bắt CPU phải chờ (Zero Latency Blocking).
3. **GPU Autonomous Evaluator**: Mọi tính toán nhân ma trận NNUE và tích lũy Accumulator được xử lý độc lập hoàn toàn trong GPU Kernel mà không có vòng lặp truyền/nhận nhỏ lẻ.




