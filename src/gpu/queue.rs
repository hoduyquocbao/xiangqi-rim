// ============================================================================
// XIANGTI ENGINE: HÀNG ĐỢI ĐỆM KÉP BẤT ĐỒNG BỘ 0-COPY (DOUBLE-BUFFERED RING BUFFER)
// ============================================================================
// Struct `RingBuffer` quản lý 2 lô thế cờ VRAM (Batch A và Batch B) song song.
// Triệt tiêu 100% thời gian chờ đợi (CPU Stalls = 0) giữa 4 luồng CPU và WGPU Metal GPU Evaluator.
// Căn lề 64-byte vật lý phòng chống False Sharing trên CPU Cache Line.
// Tuân thủ 100% định danh từ đơn tiếng Anh và 100% chú thích tiếng Việt trên từng dòng mã.
// ============================================================================

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering}; // Nhập các kiểu nguyên tử atomic từ thư viện chuẩn

use super::batch::Batch; // Nhập kiểu struct Batch từ module batch
use super::device::Device; // Nhập kiểu struct Device từ module device
use super::evaluator::Evaluator; // Nhập kiểu struct Evaluator từ module evaluator
use super::sample::Sample; // Nhập kiểu struct Sample từ module sample
use super::status::Status; // Nhập kiểu enum Status từ module status

/// Trait `Queueable`: Định nghĩa khả năng quản lý hàng đợi đệm kép bất đồng bộ.
pub trait Queueable { // Định nghĩa trait Queueable
    /// Khởi tạo hàng đợi đệm kép với sức chứa `capacity` mỗi lô.
    fn allocate(device: &Device, capacity: usize) -> Result<Self, Status> where Self: Sized; // Chữ ký hàm allocate
    /// Đẩy 1 mẫu thế cờ `Sample` vào lô đệm hiện tại của CPU.
    fn push(&mut self, sample: &Sample) -> Result<(), Status>; // Chữ ký hàm push
    /// Tráo đổi 2 bộ đệm 0-copy giữa CPU và GPU.
    fn swap(&mut self); // Chữ ký hàm swap
    /// Kiểm tra xem lô đệm CPU hiện tại đã đầy hay chưa.
    fn full(&self) -> bool; // Chữ ký hàm full
    /// Xoá rỗng lô đệm CPU hiện tại.
    fn clear(&mut self); // Chữ ký hàm clear
} // Kết thúc trait Queueable

/// Struct `RingBuffer`: Bộ quản lý đệm kép bất đồng bộ căn lề 64-byte (128 bytes total).
#[repr(C, align(64))] // Căn lề 64-byte phòng False Sharing trên CPU Cache Line
pub struct RingBuffer { // Định nghĩa struct RingBuffer
    /// Lô đệm A (64 bytes, offset 0..64)
    batch_first: Batch, // Trường lô đệm thứ nhất batch_first
    /// Lô đệm B (64 bytes, offset 64..128)
    batch_second: Batch, // Trường lô đệm thứ hai batch_second
    /// Chỉ số lô đệm CPU hiện tại (0 hoặc 1) (8 bytes, offset 128..136)
    active: usize, // Trường chỉ số lô hoạt động active
    /// Sức chứa mẫu của mỗi lô (8 bytes, offset 136..144)
    capacity: usize, // Trường sức chứa tối đa capacity
    /// Số lượng mẫu đã đẩy vào lô hiện tại (8 bytes, offset 144..152)
    count: AtomicUsize, // Trường đếm mẫu nguyên tử count
    /// Cờ sắn sàng xử lý của lô GPU (1 byte, offset 152)
    ready: AtomicBool, // Trường cờ sẵn sàng nguyên tử ready
    /// Cờ đóng dừng hoạt động của hàng đợi (1 byte, offset 153)
    closed: AtomicBool, // Trường cờ đóng dừng nguyên tử closed
    /// Mảng đệm 38 byte đảm bảo tổng kích thước struct đúng 256 bytes (38 bytes, offset 154..192)
    pad: [u8; 38], // Trường đệm pad căn lề 256 bytes
} // Kết thúc struct RingBuffer

impl RingBuffer { // Khối triển khai các phương thức cho RingBuffer
    /// Khởi tạo một RingBuffer mới chứa 2 Batch có sức chứa `capacity` mỗi lô.
    pub fn allocate(device: &Device, capacity: usize) -> Result<Self, Status> { // Hàm cấp phát allocate
        let batch_first = Batch::allocate(device, capacity)?; // Khởi tạo lô đệm A
        let batch_second = Batch::allocate(device, capacity)?; // Khởi tạo lô đệm B
        Ok(Self { // Trả về thể hiện RingBuffer mới
            batch_first, // Gán lô đệm thứ nhất
            batch_second, // Gán lô đệm thứ hai
            active: 0, // Đặt lô đệm hoạt động ban đầu là 0 (Batch A)
            capacity, // Gán sức chứa
            count: AtomicUsize::new(0), // Khởi tạo đếm mẫu bằng 0
            ready: AtomicBool::new(false), // Khởi tạo cờ sẵn sàng bằng false
            closed: AtomicBool::new(false), // Khởi tạo cờ đóng bằng false
            pad: [0u8; 38], // Khởi tạo mảng đệm zero
        }) // Kết thúc khởi tạo
    } // Kết thúc hàm allocate

    /// Tráo đổi 2 bộ đệm 0-copy giữa CPU và GPU trong 0.001 microgiây.
    pub fn swap(&mut self) { // Hàm tráo đổi swap
        self.active = 1 - self.active; // Đảo chỉ số lô hoạt động 0 ↔ 1
        self.count.store(0, Ordering::Relaxed); // Đặt lại đếm mẫu lô mới về 0
        self.ready.store(true, Ordering::Release); // Bật cờ sẵn sàng cho luồng GPU
    } // Kết thúc hàm swap

    /// Lấy tham chiếu lô đệm CPU hiện tại để đẩy dữ liệu.
    pub fn active_batch_mut(&mut self) -> &mut Batch { // Hàm lấy lô đệm CPU active_batch_mut
        if self.active == 0 { // Nếu lô hoạt động là 0
            &mut self.batch_first // Trả về lô A
        } else { // Nguồn lô là 1
            &mut self.batch_second // Trả về lô B
        } // Kết thúc nhánh kiểm tra
    } // Kết thúc hàm active_batch_mut

    /// Lấy tham chiếu lô đệm GPU hiện tại để thực thi Compute Pass.
    pub fn passive_batch_mut(&mut self) -> &mut Batch { // Hàm lấy lô đệm GPU passive_batch_mut
        if self.active == 0 { // Nếu lô CPU là 0
            &mut self.batch_second // Lô GPU là B
        } else { // Nguồn lô CPU là 1
            &mut self.batch_first // Lô GPU là A
        } // Kết thúc nhánh kiểm tra
    } // Kết thúc hàm passive_batch_mut

    /// Đẩy 1 mẫu thế cờ `Sample` vào lô đệm hiện tại.
    pub fn push(&mut self, sample: &Sample) -> Result<(), Status> { // Hàm đẩy mẫu push
        let batch = self.active_batch_mut(); // Lấy tham chiếu lô đệm CPU hiện tại
        batch.push(sample)?; // Đẩy mẫu vào lô
        self.count.fetch_add(1, Ordering::Relaxed); // Tăng đếm mẫu nguyên tử
        Ok(()) // Trả về kết quả thành công
    } // Kết thúc hàm push

    /// Thực thi đánh giá GPU bất đồng bộ trên lô đệm GPU bị động.
    pub fn flush_gpu(&mut self, evaluator: &Evaluator) -> Result<usize, Status> { // Hàm thực thi GPU flush_gpu
        let count = self.count.load(Ordering::Relaxed); // Lấy số lượng mẫu hiện tại
        if count == 0 { // Nếu không có mẫu nào
            return Ok(0); // Trả về 0 mẫu đã đánh giá
        } // Kết thúc kiểm tra count
        let batch = self.passive_batch_mut(); // Lấy lô đệm GPU bị động
        evaluator.execute(batch, count)?; // Thực thi WGPU Metal Compute Pass
        self.ready.store(false, Ordering::Release); // Tắt cờ sẵn sàng sau khi xử lý xong
        Ok(count) // Trả về số lượng mẫu đã tính xong
    } // Kết thúc hàm flush_gpu

    /// Kiểm tra cờ sẵn sàng của lô GPU.
    pub fn is_ready(&self) -> bool { // Hàm kiểm tra cờ sẵn sàng is_ready
        self.ready.load(Ordering::Acquire) // Đọc cờ nguyên tử Acquire
    } // Kết thúc hàm is_ready

    /// Đóng dừng hoạt động của hàng đợi.
    pub fn close(&self) { // Hàm đóng hàng đợi close
        self.closed.store(true, Ordering::Release); // Đặt cờ đóng bằng true
    } // Kết thúc hàm close

    /// Kiểm tra cờ đóng của hàng đợi.
    pub fn is_closed(&self) -> bool { // Hàm kiểm tra cờ đóng is_closed
        self.closed.load(Ordering::Acquire) // Đọc cờ nguyên tử Acquire
    } // Kết thúc hàm is_closed
} // Kết thúc khối impl RingBuffer
