// ============================================================================
// THƯ VIỆN XIANGTI ENGINE: HỆ THỐNG MODULE LÕI CỜ TƯỚNG HIỆU NĂNG CAO
// ============================================================================
// Định nghĩa cấu trúc phân cấp không gian tên (hierarchical modules) của dự án.
// 100% tuân thủ thiết kế Clean Room std-only và Quy tắc Từ Đơn (Single-Word Principle).
// ============================================================================

/// Module `board`: Biểu diễn bàn cờ 90 ô (Square, Piece, Bitboard 128-bit, Position align 64, Zobrist, StateInfo, FEN)
pub mod board;

/// Module `book`: Thư viện nước đi khai cuộc (Opening Book) và tri thức tàn cuộc thực dụng (Endgame Knowledge Base)
pub mod book;


/// Module `circuit`: Máy trạng thái Circuit Breaker (Closed/Open/HalfOpen) tự động hạ cấp NNUE sang HCE dự phòng
pub mod circuit;

/// Module `cqrs`: Kiến trúc CQRS-ES (Command, Query, Event Bus MPMC Bounded Ring Buffer Queue lock-free)
pub mod cqrs;

/// Module `eval`: Bộ đánh giá thế cờ (Mạng nơ-ron NNUE HalfKAv2_hm 65k features, Accumulator O(1), SIMD & HCE)
pub mod eval;

/// Module `gpu`: Tầng giao tiếp card đồ họa GPU Adapter và VRAM Guard 512MB (Milestone M1)
pub mod gpu;

/// Module `movegen`: Bộ sinh nước đi hợp lệ (Pseudo & Legal Move Generator, Lookup Tables tĩnh cho 7 loại quân, Perft)
pub mod movegen;

/// Module `search`: Bộ tìm kiếm PVS (Principal Variation Search, Quiescence, LMR, Pruning, History/Killer/Counter Tables)
pub mod search;

/// Module `simd`: Tăng tốc tính toán đại số tuyến tính SIMD đa nền tảng (AVX2 cho x86_64, NEON cho ARM64, Scalar fallback)
pub mod simd;

/// Module `thread`: Bộ quản lý đa luồng Lazy SMP Zero-Lock ThreadPool chạy tìm kiếm song song không khóa
pub mod thread;

/// Module `tt`: Bảng băm Transposition Table 16-byte AtomicU64 lock-free căn lề 64-byte
pub mod tt;

/// Module `uci`: Bộ xử lý giao thức chuẩn UCI v2 (UCI Parser & Event Loop bất đồng bộ không chặn I/O)
pub mod uci;

/// Module `wasm`: Cổng kết nối FFI C-ABI WebAssembly (std-only, zero-crate WASM bindings)
pub mod wasm;

/// Module `mcp`: Giao thức Model Context Protocol (JSON-RPC 2.0 over STDIN/STDOUT std-only MCP Server)
pub mod mcp;

/// Module `server`: Máy chủ HTTP REST API và WebSocket real-time backend server
pub mod server;

/// Module `selfplay`: Hệ thống tự đấu (Self-Play Engine, Stats, PGN/FEN Exporter)
pub mod selfplay;

/// Module `learn`: Phân hệ học thích ứng online, bộ đệm kinh nghiệm TD(lambda), blunder bias, lưu trữ nhị phân và adaptive search
pub mod learn;

/// Module `p2p`: Mạng phân tán P2P topic broadcast SHA-256 giữ kênh live 24/7 và đồng bộ dataset
pub mod p2p;










