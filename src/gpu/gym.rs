// ============================================================================
// XIANGTI ENGINE: ĐỘNG CƠ GIA TỐC GPU CHO GYM DEPTH 12 (GYM ACCELERATOR)
// ============================================================================
// Struct `Gym` đóng vai trò là Động cơ Gia tốc GPU hợp nhất cho luồng GYM Depth 12.
// Tích hợp Device, Evaluator, Kernel, Buffer và Batch trong cùng một cấu trúc căn lề 64-byte.
// Hỗ trợ chế độ Zero-Copy Shared Memory (`MTLResourceStorageModeShared`) trên macOS Intel iGPU.
// Tuân thủ 100% định danh từ đơn tiếng Anh, căn lề 64-byte và 100% chú thích tiếng Việt.
// ============================================================================

use super::batch::{Batch, Batchable}; // Nhập kiểu struct Batch và trait Batchable từ module batch
use super::buffer::Buffer; // Nhập kiểu struct Buffer từ module buffer
use super::device::Device; // Nhập kiểu struct Device từ module device
use super::evaluator::{Evaluable, Evaluator}; // Nhập kiểu struct Evaluator và trait Evaluable từ module evaluator
use super::kernel::{Dispatchable, Kernel}; // Nhập kiểu struct Kernel và trait Dispatchable từ module kernel
use super::sample::Sample; // Nhập kiểu struct Sample từ module sample
use super::status::Status; // Nhập kiểu enum Status từ module status

/// Trait `Accelerable`: Quy ước khả năng gia tốc tính toán lô thế cờ GYM trên GPU.
pub trait Accelerable { // Định nghĩa trait Accelerable
    /// Phương thức `submit`: Gửi 1 mẫu thế cờ `Sample` vào tiến trình gia tốc GPU.
    fn submit(&mut self, sample: &Sample) -> Result<(), Status>; // Chữ ký hàm submit
    /// Phương thức `process`: Ép xử lý lô mẫu thế cờ và tính điểm song song trên GPU.
    fn process(&mut self) -> Result<usize, Status>; // Chữ ký hàm process
    /// Phương thức `evaluate`: Đánh giá nhanh 1 thế cờ dạng mảng byte.
    fn evaluate(&mut self, position: &[u8]) -> Result<i32, Status>; // Chữ ký hàm evaluate
    /// Phương thức `flush`: Thu hồi toàn bộ kết quả và dọn trống lô tích lũy.
    fn flush(&mut self) -> Result<usize, Status>; // Chữ ký hàm flush
} // Kết thúc trait Accelerable

/// Struct `Gym`: Động cơ gia tốc GPU GYM Depth 12 căn lề 64-byte (704 bytes total).
#[repr(C, align(64))] // Căn lề 64-byte vật lý phòng chống False Sharing trên CPU Cache Line
pub struct Gym { // Định nghĩa struct Gym
    /// Thiết bị GPU Adapter phần cứng hợp nhất (128 bytes, offset 0..128)
    device: Device, // Trường thiết bị device
    /// Bộ đánh giá ma trận lô thế cờ NNUE (256 bytes, offset 128..384)
    evaluator: Evaluator, // Trường bộ đánh giá evaluator
    /// Compute Kernel gia tốc nút lá PVS search (64 bytes, offset 384..448)
    kernel: Kernel, // Trường compute kernel
    /// Bộ đệm VRAM Buffer liên tục 0-copy (64 bytes, offset 448..512)
    buffer: Buffer, // Trường bộ đệm buffer
    /// Container chứa lô mẫu thế cờ (128 bytes, offset 512..640)
    batch: Batch, // Trường container batch
    /// Trạng thái hoạt động nội tại của động cơ GPU (1 byte, offset 640)
    status: Status, // Trường trạng thái status
    /// Cờ hoạt động của động cơ gia tốc (1 byte, offset 641)
    active: u8, // Trường cờ hoạt động active
    /// Cờ bộ nhớ dùng chung Zero-Copy Shared Memory (1 byte, offset 642)
    shared: u8, // Trường cờ bộ nhớ dùng chung shared
    /// Mảng đệm 5 byte căn lề 8-byte boundary (5 bytes, offset 643..648)
    pad: [u8; 5], // Trường đệm pad
    /// Độ sâu huấn luyện mục tiêu (8 bytes, offset 648..656)
    depth: u64, // Trường độ sâu depth
    /// Tổng số lượng mẫu thế cờ đã được gia tốc tính toán (8 bytes, offset 656..664)
    count: u64, // Trường bộ đếm count
    /// Giới hạn dung lượng lô xử lý tối đa (8 bytes, offset 664..672)
    limit: u64, // Trường giới hạn limit
    /// Mảng đệm 32 byte đảm bảo tổng kích thước tròn 704 bytes (32 bytes, offset 672..704)
    extra: [u8; 32], // Trường đệm extra căn lề 704 bytes
} // Kết thúc struct Gym

impl Gym { // Khối triển khai các phương thức cho Gym
    /// Khởi tạo động cơ GPU Gym mới với thiết bị `device` cho trước.
    pub fn new(device: Device) -> Result<Self, Status> { // Hàm khởi tạo new
        let evaluator = Evaluator::new(Device::init())?; // Khởi tạo bộ đánh giá Evaluator
        let kernel = Kernel::init(4096, 128, 256)?; // Khởi tạo Compute Kernel 4096 positions
        let limit = 4096u64; // Kích thước lô mặc định 4096
        let bytes = (limit as usize) * 128; // Tính tổng dung lượng bộ đệm VRAM (512KB)
        let buffer = device.allocate(bytes)?; // Cấp phát bộ đệm Zero-Copy VRAM Buffer
        let batch = Batch::allocate(&device, limit as usize)?; // Cấp phát lô chứa vị trí
        let shared = cfg!(target_os = "macos"); // Tự động kích hoạt Zero-Copy trên macOS

        Ok(Self { // Trả về bản thể Gym mới
            device, // Gán thiết bị device
            evaluator, // Gán bộ đánh giá evaluator
            kernel, // Gán kernel
            buffer, // Gán bộ đệm buffer
            batch, // Gán lô batch
            status: Status::Ready, // Khởi tạo trạng thái Ready
            active: 1, // Gán cờ active = 1
            shared: if shared { 1 } else { 0 }, // Gán cờ shared
            pad: [0u8; 5], // Khởi tạo mảng đệm pad 5 byte zero
            depth: 12, // Độ sâu huấn luyện mặc định 12
            count: 0, // Bộ đếm mẫu bắt đầu từ 0
            limit, // Gán giới hạn limit = 4096
            extra: [0u8; 32], // Khởi tạo mảng đệm extra 32 byte zero
        }) // Kết thúc kết quả Ok
    } // Kết thúc hàm new

    /// Khởi tạo động cơ GPU Gym mặc định, tự động phát hiện phần cứng GPU.
    pub fn init() -> Result<Self, Status> { // Hàm khởi tạo init
        Self::new(Device::init()) // Gọi hàm new với Device::init()
    } // Kết thúc hàm init

    /// Gửi một mẫu thế cờ `Sample` vào động cơ GPU Gym.
    pub fn submit(&mut self, sample: &Sample) -> Result<(), Status> { // Hàm submit
        if self.active == 0 || !sample.valid() { // Kiểm tra động cơ không hoạt động hoặc mẫu lỗi
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra
        if self.batch.full() { // Kiểm tra lô đã đầy trần dung lượng chưa
            self.process()?; // Tự động xử lý lô cũ và làm trống bộ đệm
        } // Kết thúc tự động xử lý khi lô đầy
        self.batch.push(sample)?; // Đẩy mẫu vào batch
        if self.batch.count() >= (self.limit as usize) { // Nếu lô đạt trần dung lượng
            let _ = self.process()?; // Tự động kích hoạt xử lý lô
        } // Kết thúc tự động xử lý
        Ok(()) // Trả về thành công Ok
    } // Kết thúc hàm submit

    /// Xử lý và tính điểm toàn bộ các thế cờ đang tích lũy trong lô bằng GPU.
    pub fn process(&mut self) -> Result<usize, Status> { // Hàm process
        if self.batch.empty() { // Nếu lô rỗng
            return Ok(0); // Trả về 0 không làm gì
        } // Kết thúc kiểm tra rỗng
        let processed = self.evaluator.flush(&mut self.batch)?; // Ép bộ đánh giá Evaluator tính điểm lô
        self.count += processed as u64; // Cộng dồn số mẫu đã xử lý
        self.batch.clear(); // Đặt lại bộ đếm số lượng mẫu trong lô về 0
        Ok(processed) // Trả về số mẫu đã hoàn tất
    } // Kết thúc hàm process

    /// Đánh giá nhanh một thế cờ dạng mảng byte `position`.
    pub fn evaluate(&mut self, position: &[u8]) -> Result<i32, Status> { // Hàm evaluate
        if position.is_empty() { // Kiểm tra mảng rỗng
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra
        let mut sample = Sample::new(); // Khởi tạo mẫu thế cờ tạm thời (ô cờ 14: EMPTY)
        let ptr = &mut sample as *mut Sample as *mut u8; // Lấy con trỏ thô tới sample
        let len = position.len().min(90); // Lấy độ dài tối đa 90 bytes ô cờ
        unsafe { // Khối unsafe sao chép bộ nhớ
            std::ptr::copy_nonoverlapping(position.as_ptr(), ptr, len); // Sao chép thế cờ vào sample
        } // Kết thúc khối unsafe
        self.evaluator.eval(&sample) // Đánh giá tính điểm trực tiếp qua Evaluator
    } // Kết thúc hàm evaluate

    /// Kích hoạt Compute Kernel GPU xử lý song song các nút lá PVS search.
    pub fn execute(&mut self) -> Result<usize, Status> { // Hàm execute
        if self.active == 0 { // Kiểm tra động cơ dừng hoạt động
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra
        self.kernel.execute(&self.buffer) // Kích hoạt Compute Kernel trên bộ đệm VRAM Buffer
    } // Kết thúc hàm execute

    /// Ép xuất bản toàn bộ lô mẫu và thu hồi kết quả đánh giá.
    pub fn flush(&mut self) -> Result<usize, Status> { // Hàm flush
        self.process() // Gọi hàm process thực thi
    } // Kết thúc hàm flush

    /// Đặt lại toàn bộ trạng thái của động cơ gia tốc GPU Gym.
    pub fn reset(&mut self) -> Result<(), Status> { // Hàm reset
        self.status = Status::Ready; // Đưa trạng thái về Ready
        self.count = 0; // Đặt lại bộ đếm mẫu về 0
        self.batch.clear(); // Xóa sạch dữ liệu trong lô batch
        self.kernel.reset(); // Đặt lại Compute Kernel
        Ok(()) // Trả về thành công Ok
    } // Kết thúc hàm reset

    /// Trả về tham chiếu tới thiết bị GPU Device.
    #[inline(always)] // Inline hàm device
    pub fn device(&self) -> &Device { // Hàm device trả về &Device
        &self.device // Trả về tham chiếu device
    } // Kết thúc hàm device

    /// Trả về tham chiếu tới bộ đánh giá Evaluator.
    #[inline(always)] // Inline hàm evaluator
    pub fn evaluator(&self) -> &Evaluator { // Hàm evaluator trả về &Evaluator
        &self.evaluator // Trả về tham chiếu evaluator
    } // Kết thúc hàm evaluator

    /// Trả về tham chiếu tới Compute Kernel.
    #[inline(always)] // Inline hàm kernel
    pub fn kernel(&self) -> &Kernel { // Hàm kernel trả về &Kernel
        &self.kernel // Trả về tham chiếu kernel
    } // Kết thúc hàm kernel

    /// Trả về tham chiếu tới bộ đệm Buffer.
    #[inline(always)] // Inline hàm buffer
    pub fn buffer(&self) -> &Buffer { // Hàm buffer trả về &Buffer
        &self.buffer // Trả về tham chiếu buffer
    } // Kết thúc hàm buffer

    /// Trả về tham chiếu tới lô Batch.
    #[inline(always)] // Inline hàm batch
    pub fn batch(&self) -> &Batch { // Hàm batch trả về &Batch
        &self.batch // Trả về tham chiếu batch
    } // Kết thúc hàm batch

    /// Trả về trạng thái hoạt động của động cơ GPU.
    #[inline(always)] // Inline hàm status
    pub fn status(&self) -> Status { // Hàm status trả về Status
        self.status // Trả về status
    } // Kết thúc hàm status

    /// Kiểm tra cờ hoạt động active.
    #[inline(always)] // Inline hàm active
    pub fn active(&self) -> bool { // Hàm active trả về bool
        self.active != 0 // Trả về cờ active dạng boolean
    } // Kết thúc hàm active

    /// Kiểm tra cờ bộ nhớ dùng chung Zero-Copy shared.
    #[inline(always)] // Inline hàm shared
    pub fn shared(&self) -> bool { // Hàm shared trả về bool
        self.shared != 0 // Trả về cờ shared dạng boolean
    } // Kết thúc hàm shared

    /// Trả về tổng số mẫu thế cờ đã xử lý count.
    #[inline(always)] // Inline hàm count
    pub fn count(&self) -> usize { // Hàm count trả về usize
        self.count as usize // Trả về count ép kiểu usize
    } // Kết thúc hàm count

    /// Trả về độ sâu huấn luyện depth.
    #[inline(always)] // Inline hàm depth
    pub fn depth(&self) -> usize { // Hàm depth trả về usize
        self.depth as usize // Trả về depth ép kiểu usize
    } // Kết thúc hàm depth

    /// Trả về giới hạn lô limit.
    #[inline(always)] // Inline hàm limit
    pub fn limit(&self) -> usize { // Hàm limit trả về usize
        self.limit as usize // Trả về limit ép kiểu usize
    } // Kết thúc hàm limit
} // Kết thúc khối impl Gym

impl Accelerable for Gym { // Triển khai trait Accelerable cho Gym
    fn submit(&mut self, sample: &Sample) -> Result<(), Status> { // Triển khai submit
        self.submit(sample) // Gọi phương thức submit nội tại
    } // Kết thúc phương thức submit

    fn process(&mut self) -> Result<usize, Status> { // Triển khai process
        self.process() // Gọi phương thức process nội tại
    } // Kết thúc phương thức process

    fn evaluate(&mut self, position: &[u8]) -> Result<i32, Status> { // Triển khai evaluate
        self.evaluate(position) // Gọi phương thức evaluate nội tại
    } // Kết thúc phương thức evaluate

    fn flush(&mut self) -> Result<usize, Status> { // Triển khai flush
        self.flush() // Gọi phương thức flush nội tại
    } // Kết thúc phương thức flush
} // Kết thúc impl Accelerable for Gym

impl Evaluable for Gym { // Triển khai trait Evaluable cho Gym
    fn submit(&mut self, sample: &Sample) -> Result<(), Status> { // Triển khai submit
        self.submit(sample) // Gọi phương thức submit nội tại
    } // Kết thúc phương thức submit

    fn flush(&mut self, batch: &mut Batch) -> Result<usize, Status> { // Triển khai flush
        self.evaluator.flush(batch) // Ép xuất bản lô qua Evaluator
    } // Kết thúc phương thức flush

    fn eval(&self, sample: &Sample) -> Result<i32, Status> { // Triển khai eval
        self.evaluator.eval(sample) // Tính điểm mẫu thế cờ qua Evaluator
    } // Kết thúc phương thức eval
} // Kết thúc impl Evaluable for Gym

impl Batchable for Gym { // Triển khai trait Batchable cho Gym
    fn allocate(device: &Device, capacity: usize) -> Result<Batch, Status> where Self: Sized { // Triển khai allocate
        Batch::allocate(device, capacity) // Gọi hàm allocate từ Batch
    } // Kết thúc phương thức allocate

    fn push(&mut self, sample: &Sample) -> Result<(), Status> { // Triển khai push
        self.batch.push(sample) // Đẩy mẫu vào batch nội tại
    } // Kết thúc phương thức push

    fn pull(&self, index: usize) -> Result<Sample, Status> { // Triển khai pull
        self.batch.pull(index) // Trích xuất mẫu tại chỉ số index
    } // Kết thúc phương thức pull

    fn write(&mut self, index: usize, sample: &Sample) -> Result<(), Status> { // Triển khai write
        self.batch.write(index, sample) // Ghi đè mẫu tại chỉ số index
    } // Kết thúc phương thức write

    fn full(&self) -> bool { // Triển khai full
        self.batch.full() // Kiểm tra lô đầy hay chưa
    } // Kết thúc phương thức full

    fn empty(&self) -> bool { // Triển khai empty
        self.batch.empty() // Kiểm tra lô rỗng hay không
    } // Kết thúc phương thức empty

    fn clear(&mut self) { // Triển khai clear
        self.batch.clear(); // Xóa sạch dữ liệu trong lô batch
    } // Kết thúc phương thức clear
} // Kết thúc impl Batchable for Gym

impl Dispatchable for Gym { // Triển khai trait Dispatchable cho Gym
    fn dispatch(&mut self, buffer: &Buffer, count: usize) -> Result<usize, Status> { // Triển khai dispatch
        self.kernel.dispatch(buffer, count) // Điều phối Compute Kernel xử lý lô
    } // Kết thúc phương thức dispatch

    fn finish(&self, buffer: &Buffer, target: &mut [isize]) -> Result<usize, Status> { // Triển khai finish
        self.kernel.finish(buffer, target) // Thu hồi kết quả từ Kernel
    } // Kết thúc phương thức finish

    fn poll(&self) -> Status { // Triển khai poll
        self.kernel.poll() // Truy vấn trạng thái Kernel
    } // Kết thúc phương thức poll

    fn flush(&mut self, buffer: &Buffer) -> Result<usize, Status> { // Triển khai flush
        self.kernel.flush(buffer) // Ép xuất bản lô vào Kernel
    } // Kết thúc phương thức flush
} // Kết thúc impl Dispatchable for Gym

#[cfg(test)] // Module kiểm thử unit tests cho Gym GPU Accelerator
mod tests { // Cấu hình module tests
    use super::*; // Nhập tất cả đối tượng từ module cha

    #[test] // Đánh dấu hàm kiểm thử cấu trúc và căn lề bộ nhớ 64-byte
    fn layout() { // Hàm test layout Gym
        assert_eq!(std::mem::size_of::<Gym>() % 64, 0); // Kiểm tra tổng kích thước chia hết cho 64 bytes
        assert_eq!(std::mem::align_of::<Gym>(), 64); // Kiểm tra căn lề align_of đúng 64 bytes
    } // Kết thúc hàm layout

    #[test] // Đánh dấu hàm kiểm thử luồng khởi tạo và gia tốc thế cờ GPU Gym
    fn acceleration() { // Hàm test luồng gia tốc Gym
        let mut gym = Gym::init().unwrap(); // Khởi tạo động cơ GPU Gym tự động
        assert!(gym.active()); // Động cơ đang ở trạng thái active
        assert_eq!(gym.depth(), 12); // Độ sâu mặc định là 12

        let sample = Sample::new(); // Khởi tạo mẫu thế cờ
        assert!(gym.submit(&sample).is_ok()); // Gửi mẫu vào động cơ GPU
        assert_eq!(gym.batch().count(), 1); // Lô đã tiếp nhận 1 mẫu

        let processed = gym.process().unwrap(); // Xử lý đánh giá lô mẫu
        assert_eq!(processed, 1); // Đã xử lý xong 1 mẫu
        assert_eq!(gym.count(), 1); // Bộ đếm tổng số mẫu bằng 1

        let position = [14u8; 90]; // Mảng 90 bytes ô cờ rỗng (14: EMPTY)
        let score = gym.evaluate(&position).unwrap(); // Đánh giá nhanh vị trí
        assert_eq!(score, 0); // Điểm thế cờ rỗng bằng 0
    } // Kết thúc hàm acceleration
} // Kết thúc module tests
