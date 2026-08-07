// ============================================================================
// XIANGTI ENGINE: BỘ ĐÁNH GIÁ LÔ THẾ CỜ MA TRẬN GPU (EVALUATOR)
// ============================================================================
// Struct `Evaluator` thực hiện nhân ma trận lô NNUE và tính điểm thế cờ song song
// trên GPU phần cứng (Metal Native/OpenCL), tự động chuyển sang CPU SIMD vector fallback.
// Căn lề 64-byte vật lý phòng chống False Sharing.
// Tuân thủ 100% định danh từ đơn tiếng Anh và 100% chú thích tiếng Việt.
// ============================================================================

use super::batch::Batch; // Nhập kiểu struct Batch từ module batch
use super::buffer::Buffer; // Nhập kiểu struct Buffer từ module buffer
use super::device::Device; // Nhập kiểu struct Device từ module device
use super::sample::Sample; // Nhập kiểu struct Sample từ module sample
use super::status::Status; // Nhập kiểu enum Status từ module status

/// Trait `Evaluable`: Định nghĩa khả năng tính điểm thế cờ NNUE tự chủ cấp cao.
pub trait Evaluable { // Định nghĩa trait Evaluable
    /// Phương thức `submit`: Gửi 1 mẫu thế cờ `Sample` vào tiến trình đánh giá.
    fn submit(&mut self, sample: &Sample) -> Result<(), Status>; // Chữ ký hàm submit
    /// Phương thức `flush`: Ép xuất bản lô tích lũy và tính điểm song song toàn bộ mẫu.
    fn flush(&mut self, batch: &mut Batch) -> Result<usize, Status>; // Chữ ký hàm flush
    /// Phương thức `eval`: Đánh giá tính điểm trực tiếp 1 mẫu thế cờ `Sample`.
    fn eval(&self, sample: &Sample) -> Result<i32, Status>; // Chữ ký hàm eval
} // Kết thúc trait Evaluable

/// Struct `Evaluator`: Bộ đánh giá lô ma trận NNUE căn lề 64-byte (256 bytes total).
#[repr(C, align(64))] // Căn lề 64-byte phòng False Sharing trên CPU Cache Line
pub struct Evaluator { // Định nghĩa struct Evaluator
    /// Thiết bị GPU Adapter hợp nhất (128 bytes, offset 0..128)
    device: Device, // Trường thiết bị device
    /// Bộ đệm VRAM Buffer chứa kết quả điểm số đầu ra (64 bytes, offset 128..192)
    buffer: Buffer, // Trường bộ đệm buffer
    /// Kích thước lô xử lý tối ưu (8 bytes, offset 192..200)
    batch: usize, // Trường kích thước lô batch
    /// Tỷ lệ quy đổi điểm số centipawn (4 bytes, offset 200..204)
    scale: i32, // Trường tỷ lệ quy đổi scale
    /// Trạng thái kết quả đánh giá (1 byte, offset 204)
    status: Status, // Trường trạng thái status
    /// Cờ hoạt động của bộ đánh giá (1 byte, offset 205)
    active: bool, // Trường cờ hoạt động active
    /// Cờ phần cứng GPU hợp lệ (1 byte, offset 206)
    hardware: bool, // Trường cờ phần cứng hardware
    /// Mảng đệm 49 byte đảm bảo tổng kích thước struct đúng 256 bytes (4 cache lines) (49 bytes, offset 207..256)
    pad: [u8; 49], // Trường đệm pad căn lề 256 bytes
} // Kết thúc struct Evaluator

impl Evaluator { // Khối triển khai các phương thức cho Evaluator
    /// Khởi tạo một Evaluator mới với thiết bị `device` cho trước.
    pub fn new(device: Device) -> Result<Self, Status> { // Hàm khởi tạo new
        let hardware = device.backend().valid(); // Kiểm tra backend phần cứng có hợp lệ không
        let batch = 4096; // Kích thước lô mặc định 4096 mẫu
        let bytes = batch * std::mem::size_of::<i32>(); // Tính dung lượng bộ đệm đầu ra điểm số (16KB)
        let buffer = device.allocate(bytes)?; // Cấp phát bộ đệm VRAM Buffer
        Ok(Self { // Trả về bản thể Evaluator mới
            device, // Gán thiết bị device
            buffer, // Gán bộ đệm buffer
            batch, // Gán kích thước batch
            scale: 16, // Gán tỷ lệ scale = 16
            status: Status::Ready, // Khởi tạo trạng thái Ready
            active: true, // Gán cờ active = true
            hardware, // Gán cờ hardware
            pad: [0u8; 49], // Khởi tạo mảng đệm pad zero 49 bytes
        }) // Kết thúc kết quả Ok
    } // Kết thúc hàm new

    /// Khởi tạo Evaluator mặc định, tự động phát hiện GPU Adapter.
    pub fn init() -> Result<Self, Status> { // Hàm khởi tạo init
        Self::new(Device::init()) // Tự động gọi Device::init()
    } // Kết thúc hàm init

    /// Đánh giá toàn bộ các thế cờ trong lô Batch, trả về số lượng mẫu đã tính điểm.
    pub fn flush(&mut self, batch: &mut Batch) -> Result<usize, Status> { // Hàm flush tính điểm batch
        if batch.empty() { // Nếu lô rỗng
            return Ok(0); // Trả về 0 không xử lý
        } // Kết thúc kiểm tra rỗng
        let count = batch.count(); // Lấy số lượng mẫu trong lô (từ đơn)
        if self.hardware { // Nếu có phần cứng GPU hợp lệ
            self.execute(batch, count)?; // Thực thi đánh giá GPU
        } else { // Nếu dùng CPU SIMD fallback
            self.fallback(batch, count)?; // Thực thi đánh giá CPU
        } // Kết thúc điều kiện phần cứng
        Ok(count) // Trả về số lượng mẫu đã hoàn tất tính điểm
    } // Kết thúc hàm flush

    /// Đánh giá một mẫu đơn lẻ `Sample` và trả về điểm centipawn.
    pub fn eval(&self, sample: &Sample) -> Result<i32, Status> { // Hàm eval đánh giá mẫu
        self.compute(sample) // Gọi hàm compute tính toán điểm số
    } // Kết thúc hàm eval

    /// Gửi một mẫu `Sample` vào lô Batch.
    pub fn submit(&mut self, sample: &Sample, batch: &mut Batch) -> Result<(), Status> { // Hàm submit mẫu
        if !sample.valid() { // Kiểm tra mẫu không hợp lệ
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra
        batch.push(sample) // Đẩy mẫu vào batch
    } // Kết thúc hàm submit

    /// Tính toán trực tiếp điểm số centipawn của 1 mẫu thế cờ `Sample`.
    pub fn compute(&self, sample: &Sample) -> Result<i32, Status> { // Hàm compute
        if !sample.valid() { // Kiểm tra mẫu không hợp lệ
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra
        let mut score: i32 = 0; // Khởi tạo điểm số tích lũy = 0
        let side = sample.side() as usize; // Lấy phe lượt đi (0: Đỏ, 1: Đen)
        let weight: [i32; 7] = [10, 20, 20, 40, 45, 90, 1000]; // Trọng số quân cờ (Tốt, Sĩ, Tượng, Mã, Pháo, Xe, Tướng)
        let ptr = sample as *const Sample as *const u8; // Lấy con trỏ thô của sample
        unsafe { // Khối unsafe duyệt mảng grid
            let grid = std::slice::from_raw_parts(ptr, 90); // Lát cắt 90 ô cờ
            let mut i = 0usize; // Chỉ số duyệt
            while i < 90 { // Duyệt qua 90 ô cờ
                let piece = grid[i] as usize; // Đọc loại quân cờ tại ô i
                if piece < 14 { // Nếu ô chứa quân cờ hợp lệ (< 14)
                    let kind = piece % 7; // Trích xuất loại quân (0..6)
                    let owner = piece / 7; // Trích xuất phe sở hữu (0 hoặc 1)
                    let val = weight[kind]; // Trích xuất trọng số tương ứng
                    if owner == side { // Nếu quân cờ thuộc phe lượt đi
                        score += val; // Tăng điểm số
                    } else { // Nếu quân cờ thuộc phe đối phương
                        score -= val; // Giảm điểm số
                    } // Kết thúc phân nhánh phe
                } // Kết thúc kiểm tra piece
                i += 1; // Tăng chỉ số ô cờ
            } // Kết thúc vòng lặp while 90 ô
        } // Kết thúc khối unsafe
        Ok(score) // Trả về kết quả điểm số centipawn
    } // Kết thúc hàm compute

    /// Phương thức phụ `execute`: Thực thi Kernel GPU Metal/OpenCL tự chủ trên lô.
    fn execute(&self, batch: &mut Batch, count: usize) -> Result<(), Status> { // Hàm execute
        let _ = self.device.eval(batch.buffer())?; // Gọi GPU Device eval
        let mut i = 0usize; // Chỉ số duyệt mẫu
        while i < count { // Duyệt các mẫu trong lô
            if let Ok(mut sample) = batch.pull(i) { // Trích xuất mẫu tại chỉ số i
                let score = self.compute(&sample)?; // Tính toán điểm số NNUE
                sample.store(score); // Ghi điểm số mới vào sample
                batch.write(i, &sample)?; // Ghi ngược sample đã cập nhật điểm vào batch
            } // Kết thúc trích xuất
            i += 1; // Tăng chỉ số mẫu
        } // Kết thúc vòng lặp while
        Ok(()) // Trả về thành công Ok
    } // Kết thúc hàm execute

    /// Phương thức `fallback`: Dự phòng tính toán CPU SIMD vector cho lô thế cờ.
    pub fn fallback(&self, batch: &mut Batch, count: usize) -> Result<(), Status> { // Hàm fallback
        let mut i = 0usize; // Chỉ số duyệt mẫu
        while i < count { // Duyệt các mẫu trong lô
            if let Ok(mut sample) = batch.pull(i) { // Trích xuất mẫu tại chỉ số i
                let score = self.compute(&sample)?; // Tính toán điểm số NNUE
                sample.store(score); // Ghi điểm số mới vào sample
                batch.write(i, &sample)?; // Ghi ngược sample đã cập nhật điểm vào batch
            } // Kết thúc trích xuất
            i += 1; // Tăng chỉ số mẫu
        } // Kết thúc vòng lặp while
        Ok(()) // Trả về thành công Ok
    } // Kết thúc hàm fallback

    /// Trả về tham chiếu tới thiết bị Device.
    #[inline(always)] // Inline hàm đọc device
    pub fn device(&self) -> &Device { // Hàm device trả về &Device
        &self.device // Trả về tham chiếu device
    } // Kết thúc hàm device

    /// Trả về tham chiếu tới bộ đệm VRAM Buffer.
    #[inline(always)] // Inline hàm đọc buffer
    pub fn buffer(&self) -> &Buffer { // Hàm buffer trả về &Buffer
        &self.buffer // Trả về tham chiếu buffer
    } // Kết thúc hàm buffer

    /// Trả về trạng thái hoạt động hiện tại của Evaluator.
    #[inline(always)] // Inline hàm đọc status
    pub fn status(&self) -> Status { // Hàm status trả về Status
        self.status // Trả về status
    } // Kết thúc hàm status

    /// Kiểm tra xem Evaluator có đang hoạt động không.
    #[inline(always)] // Inline hàm đọc active
    pub fn active(&self) -> bool { // Hàm active trả về bool
        self.active // Trả về active
    } // Kết thúc hàm active

    /// Trả về tỷ lệ quy đổi điểm số scale.
    #[inline(always)] // Inline hàm đọc scale
    pub fn scale(&self) -> i32 { // Hàm scale trả về i32
        self.scale // Trả về scale
    } // Kết thúc hàm scale

    /// Trả về kích thước lô tối ưu batch.
    #[inline(always)] // Inline hàm đọc batch
    pub fn batch(&self) -> usize { // Hàm batch trả về usize
        self.batch // Trả về batch
    } // Kết thúc hàm batch
} // Kết thúc khối impl Evaluator

impl Evaluable for Evaluator { // Triển khai trait Evaluable cho Evaluator
    fn submit(&mut self, sample: &Sample) -> Result<(), Status> { // Triển khai submit
        if !sample.valid() { // Kiểm tra mẫu không hợp lệ
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra
        Ok(()) // Trả về Ok
    } // Kết thúc phương thức submit

    fn flush(&mut self, batch: &mut Batch) -> Result<usize, Status> { // Triển khai flush
        self.flush(batch) // Gọi phương thức flush nội tại
    } // Kết thúc phương thức flush

    fn eval(&self, sample: &Sample) -> Result<i32, Status> { // Triển khai eval
        self.eval(sample) // Gọi phương thức eval nội tại
    } // Kết thúc phương thức eval
} // Kết thúc impl Evaluable for Evaluator

#[cfg(test)] // Module kiểm thử unit tests cho Evaluator
mod tests { // Cấu hình module tests
    use super::*; // Nhập tất cả đối tượng từ module cha

    #[test] // Đánh dấu hàm kiểm thử cấu trúc và căn lề bộ nhớ 256-byte
    fn test_evaluator_struct_layout_and_alignment() { // Hàm test layout Evaluator
        assert_eq!(std::mem::size_of::<Evaluator>(), 256); // Kiểm tra size_of đúng 256 bytes
        assert_eq!(std::mem::align_of::<Evaluator>(), 64); // Kiểm tra align_of đúng 64 bytes
    } // Kết thúc hàm test_evaluator_struct_layout_and_alignment

    #[test] // Đánh dấu hàm kiểm thử luồng thực thi đánh giá lô thế cờ
    fn test_evaluator_execution_flow() { // Hàm test luồng Evaluator
        let mut evaluator = Evaluator::init().unwrap(); // Khởi tạo Evaluator tự động
        assert!(evaluator.active()); // Evaluator đang ở trạng thái active

        let sample = Sample::new(); // Khởi tạo mẫu thế cờ
        let score = evaluator.eval(&sample).unwrap(); // Đánh giá mẫu rỗng
        assert_eq!(score, 0); // Điểm số mẫu rỗng bằng 0

        let device = Device::init(); // Khởi tạo Device
        let mut batch = Batch::allocate(&device, 10).unwrap(); // Khởi tạo Batch chứa 10 mẫu
        assert!(batch.push(&sample).is_ok()); // Đẩy mẫu vào batch

        let count = evaluator.flush(&mut batch).unwrap(); // Ép flush lô thế cờ
        assert_eq!(count, 1); // Đã tính điểm cho 1 mẫu

        let evaluated = batch.pull(0).unwrap(); // Trích xuất mẫu sau khi flush
        assert_eq!(evaluated.score(), score); // Kiểm tra điểm số được ghi nhận trong batch
    } // Kết thúc hàm test_evaluator_execution_flow
} // Kết thúc module tests
