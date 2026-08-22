// ============================================================================
// XIANGTI ENGINE: MODULE ADAPTER GIA TỐC GPU VÀ QUẢN LÝ BỘ NHỚ VRAM (MILESTONE M1 + M2)
// ============================================================================
// Re-export các submodule thành phần: backend, guard, buffer, device, status, sample, batch, kernel, evaluator.
// Tuân thủ 100% kiến trúc Clean Room std-only, căn lề 64-byte phần cứng và từ đơn.
// ============================================================================

/// Submodule `backend`: Enum phân cấp các nền tảng tính toán GPU (Metal, Opencl, Wgpu, Cpu)
pub mod backend; // Khai báo submodule backend quản lý các loại phần cứng tính toán

/// Submodule `batch`: Container chứa lô thế cờ truyền VRAM
pub mod batch; // Khai báo submodule batch quản lý lô mẫu thế cờ

/// Submodule `buffer`: Khối bộ đệm VRAM/RAM liên tục căn lề 64-byte phần cứng
pub mod buffer; // Khai báo submodule buffer quản lý con trỏ bộ nhớ host/device

/// Submodule `device`: Thiết bị đại diện GPU Adapter hợp nhất tích hợp Guard và Backend
pub mod device; // Khai báo submodule device đại diện cho GPU Adapter

/// Submodule `evaluator`: Bộ đánh giá lô thế cờ ma trận GPU tự chủ
pub mod evaluator; // Khai báo submodule evaluator tính điểm song song

/// Submodule `guard`: Bộ giám sát chống tràn dung lượng VRAM 512MB với trần an toàn 409.6MB
pub mod guard; // Khai báo submodule guard theo dõi cấp phát bộ nhớ đệm VRAM

/// Submodule `kernel`: Compute Kernel gia tốc song song các nút lá PVS search
pub mod kernel; // Khai báo submodule kernel thực thi GPU Compute Shader

/// Submodule `gym`: Động cơ gia tốc GPU cho môi trường tự huấn luyện GYM Depth 12
pub mod gym; // Khai báo submodule gym quản lý động cơ gia tốc GPU GYM

/// Submodule `sample`: Đại diện 1 mẫu vị trí thế cờ căn lề 128 bytes
pub mod sample; // Khai báo submodule sample cấu trúc dữ liệu thế cờ

/// Submodule `status`: Mã trạng thái kết quả của các thao tác GPU và bộ nhớ VRAM
pub mod status; // Khai báo submodule status định nghĩa mã kết quả hoạt động

/// Submodule `aggregator`: Bộ gom mẫu vị trí cờ lá không khóa lock-free căn lề 64-byte
pub mod aggregator; // Khai báo submodule aggregator quản lý gom mẫu cờ lá

/// Submodule `cuda`: Tích hợp GPU CUDA Native C++ FFI cho NVIDIA Colab
pub mod cuda;

/// Submodule `queue`: Hàng đợi đệm kép bất đồng bộ RingBuffer 0-copy căn lề 64-byte
pub mod queue; // Khai báo submodule queue quản lý hàng đợi đệm kép

pub use aggregator::{Aggregatable, Aggregator}; // Re-export struct Aggregator và trait Aggregatable
pub use backend::Backend; // Re-export kiểu enum Backend
pub use batch::{Batch, Batchable}; // Re-export struct Batch và trait Batchable
pub use buffer::{Buffer, Storable}; // Re-export kiểu struct Buffer và trait Storable
pub use device::{Device, Queryable}; // Re-export kiểu struct Device và trait Queryable
pub use evaluator::{Evaluable, Evaluator}; // Re-export struct Evaluator và trait Evaluable
pub use guard::{Guard, Validatable}; // Re-export struct Guard và trait Validatable
pub use gym::{Accelerable, Gym}; // Re-export struct Gym và trait Accelerable
pub use kernel::{Dispatchable, Kernel}; // Re-export struct Kernel và trait Dispatchable
pub use queue::RingBuffer; // Re-export kiểu struct RingBuffer từ submodule queue
pub use sample::{Sample, Sampleable}; // Re-export struct Sample và trait Sampleable
pub use status::Status; // Re-export kiểu enum Status chính thức
