// ============================================================================
// PHÂN HỆ LEARN: ADAPTIVE ONLINE REINFORCEMENT LEARNING & PERSISTENT MEMORY
// ============================================================================
// Module quản lý học máy tăng cường online (Reinforcement Learning) cho Cờ Tướng:
// 1. replay: Bộ đệm mảng xoay vòng Experience Replay Buffer (10,000 transition samples).
// 2. trace: Cập nhật vết điều kiện Eligibility Trace & TD(lambda) Error delta_t.
// 3. blunder: Nhận diện nước đi sai lầm blunder (>=200cp drop) và phạt điểm Move Ordering.
// 4. store: Lưu trữ trí nhớ kinh nghiệm nhị phân (b"XRLN", 64B header, 32B record) xuống đĩa.
// 5. adapt: Tối ưu phương trình bàn cờ C_board, PV stability, Dynamic Aspiration Window, Adaptive LMR.
// 6. trainer: Trình quản lý tự đấu huấn luyện online trainer over multiple games.
// 100% chú thích tiếng Việt & 100% định danh từ đơn tiếng Anh (Single-Word Principle).
// ============================================================================

/// Module `replay`: Bộ đệm kinh nghiệm xoay vòng (Experience Replay Buffer)
pub mod replay;

/// Module `trace`: Vết điều kiện Temporal Difference (TD(lambda) Eligibility Trace)
pub mod trace;

/// Module `blunder`: Phân tích nước lỗi (Blunder Analysis & Penalty Bias)
pub mod blunder;

/// Module `store`: Lưu trữ bộ nhớ kinh nghiệm nhị phân (Persistent Memory Storage)
pub mod store;

/// Module `adapt`: Tối ưu phương trình giới hạn tìm kiếm (Adaptive Search Limits)
pub mod adapt;

/// Module `trainer`: Trình huấn luyện tự đấu học máy online (Online RL Trainer)
pub mod trainer;

/// Module `gym`: Môi trường tự huấn luyện ngầm lũy tiến độ sâu tốc độ cao (Progressive GYM Engine)
pub mod gym;

/// Module `audit`: Phân hệ chẩn đoán rủi ro, mặt tối và ngây thơ tiềm ẩn
pub mod audit;

/// Module `nnue`: Huấn luyện mạng nơ-ron NNUE từ dữ liệu tự đấu (NNUE Training Engine)
pub mod nnue;

// Re-export các cấu trúc dữ liệu cốt lõi để các module bên ngoài và ví dụ dễ dàng truy cập
pub use adapt::Adapt;
pub use audit::{Audit, Report as AuditReport};
pub use blunder::{Blunder, Fault};
pub use gym::{Gym, Match, Status};
pub use nnue::{Datum, Forward, Network};
pub use replay::{Replay, Sample};
pub use store::{Header, Record, Store};
pub use trace::{Entry, Trace};
pub use trainer::{Stats, Trainer};
