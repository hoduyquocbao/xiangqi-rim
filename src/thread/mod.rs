// ============================================================================
// MODULE THREAD: BỘ QUẢN LÝ ĐA LUỒNG LAZY SMP ZERO-LOCK THREADPOOL
// ============================================================================
// Lazy SMP (Symmetric Multi-Processing) là kỹ thuật song song hóa tìm kiếm cây cờ hiệu quả nhất:
// - Mỗi Worker Thread chạy độc lập một cây PVS riêng biệt mà KHÔNG cần dùng Mutex/Lock.
// - Các luồng liên thông dữ liệu qua bảng băm nguyên tử Transposition Table (`AtomicU64`).
// - Tín hiệu ngắt (`Signal`) sử dụng `AtomicBool` và `AtomicU8` phản hồi ngắt tức thì < 1ms.
// ============================================================================

/// Module con `affinity` quản lý gán định tuyến luồng ưu tiên P-Core (macOS QoS)
pub mod affinity;
/// Module con `pool` quản lý tập hợp các luồng Worker ThreadPool
pub mod pool;
/// Module con `signal` quản lý tín hiệu dừng và trạng thái đồng bộ không khóa
pub mod signal;
/// Module con `worker` đại diện cho một luồng làm việc Worker Thread độc lập
pub mod worker;

pub use affinity::Affinity;
pub use pool::Pool;
pub use signal::{Signal, State};
pub use worker::Worker;


