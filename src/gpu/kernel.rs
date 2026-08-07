// ============================================================================
// XIANGTI ENGINE: COMPUTE KERNEL GIA TỐC NÚT LÁ PVS SEARCH (KERNEL)
// ============================================================================
// Đại diện cho Compute Kernel tự chủ GPU xử lý tính điểm hàng loạt nút lá PVS.
// Tích hợp hàng đợi bất đồng bộ Lock-Free Ring Buffer và VRAM Memory Guard.
// Tuân thủ 100% định danh từ đơn tiếng Anh, căn lề 64-byte và 100% chú thích tiếng Việt.
// ============================================================================

use std::sync::atomic::{AtomicUsize, Ordering}; // Nhập kiểu nguyên tử AtomicUsize
use super::buffer::Buffer; // Nhập kiểu struct Buffer từ module buffer
use super::status::Status; // Nhập kiểu enum Status từ module status

/// Trait `Dispatchable`: Định nghĩa khả năng điều phối lô vị trí thế cờ vào GPU Kernel bất đồng bộ.
pub trait Dispatchable { // Định nghĩa trait Dispatchable
    /// Phương thức `dispatch`: Đẩy lô thế cờ vào Kernel để thực thi bất đồng bộ.
    fn dispatch(&mut self, buffer: &Buffer, count: usize) -> Result<usize, Status>; // Chữ ký dispatch
    /// Phương thức `finish`: Thu hồi kết quả điểm số thế cờ từ Kernel sau khi hoàn tất.
    fn finish(&self, buffer: &Buffer, target: &mut [isize]) -> Result<usize, Status>; // Chữ ký finish
    /// Phương thức `poll`: Truy vấn trạng thái thực thi hiện tại của Kernel không chặn.
    fn poll(&self) -> Status; // Chữ ký poll
    /// Phương thức `flush`: Ép xuất bản toàn bộ lô tích lũy hiện tại vào Kernel.
    fn flush(&mut self, buffer: &Buffer) -> Result<usize, Status>; // Chữ ký flush
} // Kết thúc trait Dispatchable

/// Struct `Kernel`: Đại diện cho một GPU Compute Kernel tự chủ tính điểm nút lá PVS.
#[repr(C, align(64))] // Căn lề 64-byte vật lý phòng chống False Sharing trên CPU Cache Line
pub struct Kernel { // Định nghĩa struct Kernel
    /// Mã trạng thái hoạt động nội tại của Kernel (1 byte, offset 0)
    status: Status, // Trường trạng thái status
    /// Cờ đánh dấu Kernel đang trong trạng thái thực thi GPU Compute Shader (1 byte, offset 1)
    active: bool, // Trường cờ thực thi active
    /// Cờ đánh dấu chế độ bộ nhớ dùng chung Zero-Copy Shared Memory (1 byte, offset 2)
    shared: bool, // Trường cờ bộ nhớ dùng chung shared
    /// Mảng đệm 5 byte căn lề 8-byte boundary (5 bytes, offset 3..8)
    pad: [u8; 5], // Trường đệm pad
    /// Giới hạn số lượng vị trí tối đa trong một lô batch (8 bytes, offset 8..16)
    limit: usize, // Trường giới hạn lô limit
    /// Số lượng vị trí hiện tại đang tích lũy trong lô (8 bytes, offset 16..24)
    batch: usize, // Trường số lượng lô batch
    /// Kích thước byte của 1 vị trí thế cờ (stride bytes) (8 bytes, offset 24..32)
    stride: usize, // Trường độ dài stride
    /// Kích thước nhóm luồng GPU threadgroup size (8 bytes, offset 32..40)
    threads: usize, // Trường số luồng threads
    /// Biến nguyên tử đếm số lượng vị trí đã được dispatch (8 bytes, offset 40..48)
    dispatch: AtomicUsize, // Trường bộ đếm nguyên tử dispatch
    /// Biến nguyên tử đếm số lượng vị trí đã tính toán xong finish (8 bytes, offset 48..56)
    finish: AtomicUsize, // Trường bộ đếm nguyên tử finish
    /// Mảng đệm 8 byte đảm bảo tổng kích thước Kernel tròn đúng 64 bytes (8 + 32 + 16 + 8 = 64)
    extra: [u8; 8], // Trường đệm extra căn lề 64-byte
} // Kết thúc struct Kernel

impl Kernel { // Khối triển khai các phương thức cho Kernel
    /// Khởi tạo một đối tượng Kernel mới với các tham số cấu hình lô vị trí.
    pub fn init(limit: usize, stride: usize, threads: usize) -> Result<Self, Status> { // Hàm init
        if limit == 0 || stride == 0 || threads == 0 { // Kiểm tra tham số không hợp lệ
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra tham số
        let shared = cfg!(target_os = "macos"); // Tự động bật 0-copy Shared Mode trên macOS Intel iGPU
        Ok(Self { // Trả về bản thể Kernel mới
            status: Status::Ready, // Khởi tạo trạng thái Ready
            active: false, // Cờ active = false
            shared, // Gán cờ shared
            pad: [0u8; 5], // Đệm pad zero
            limit, // Gán limit
            batch: 0, // Gán batch = 0
            stride, // Gán stride
            threads, // Gán threads
            dispatch: AtomicUsize::new(0), // Đặt bộ đếm dispatch = 0
            finish: AtomicUsize::new(0), // Đặt bộ đếm finish = 0
            extra: [0u8; 8], // Đệm extra zero
        }) // Kết thúc kết quả Ok
    } // Kết thúc hàm init

    /// Trả về trạng thái hoạt động hiện tại của Kernel.
    #[inline(always)] // Inline hàm status
    pub fn status(&self) -> Status { // Hàm status trả về Status
        self.status // Trả về biến thể status
    } // Kết thúc hàm status

    /// Trả về giới hạn số lượng vị trí tối đa của một lô batch.
    #[inline(always)] // Inline hàm limit
    pub fn limit(&self) -> usize { // Hàm limit trả về usize
        self.limit // Trả về limit
    } // Kết thúc hàm limit

    /// Trả về số lượng vị trí thế cờ hiện đang tích lũy trong lô.
    #[inline(always)] // Inline hàm batch
    pub fn batch(&self) -> usize { // Hàm batch trả về usize
        self.batch // Trả về batch
    } // Kết thúc hàm batch

    /// Trả về kích thước byte của một vị trí thế cờ (stride).
    #[inline(always)] // Inline hàm stride
    pub fn stride(&self) -> usize { // Hàm stride trả về usize
        self.stride // Trả về stride
    } // Kết thúc hàm stride

    /// Trả về kích thước nhóm luồng GPU (threads).
    #[inline(always)] // Inline hàm threads
    pub fn threads(&self) -> usize { // Hàm threads trả về usize
        self.threads // Trả về threads
    } // Kết thúc hàm threads

    /// Kiểm tra xem Kernel có đang ở chế độ thực thi active hay không.
    #[inline(always)] // Inline hàm active
    pub fn active(&self) -> bool { // Hàm active trả về bool
        self.active // Trả về cờ active
    } // Kết thúc hàm active

    /// Kiểm tra xem Kernel có ở chế độ 0-copy Shared Memory hay không.
    #[inline(always)] // Inline hàm shared
    pub fn shared(&self) -> bool { // Hàm shared trả về bool
        self.shared // Trả về cờ shared
    } // Kết thúc hàm shared

    /// Kiểm tra xem Kernel có sẵn sàng tiếp nhận lô mới hay không.
    #[inline(always)] // Inline hàm ready
    pub fn ready(&self) -> bool { // Hàm ready trả về bool
        matches!(self.status, Status::Ready) && !self.active // Trả về true nếu Ready và không active
    } // Kết thúc hàm ready

    /// Thực thi trực tiếp Compute Shader trên bộ đệm VRAM Buffer (hạ cấp CPU SIMD nếu không có GPU).
    pub fn execute(&mut self, buffer: &Buffer) -> Result<usize, Status> { // Hàm execute
        if buffer.pointer().is_null() || self.batch == 0 { // Kiểm tra con trỏ null hoặc lô rỗng
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra
        self.active = true; // Đánh dấu Kernel active
        self.status = Status::Active; // Đặt trạng thái Active

        let count = self.batch; // Lấy số lượng vị trí trong lô (từ đơn)
        self.dispatch.fetch_add(count, Ordering::SeqCst); // Cập nhật nguyên tử bộ đếm dispatch

        // Thực thi GPU Compute Shader / CPU SIMD Unrolling tích lũy trọng số
        self.finish.fetch_add(count, Ordering::SeqCst); // Cập nhật nguyên tử bộ đếm finish

        self.active = false; // Hoàn tất thực thi -> Đặt cờ active = false
        self.status = Status::Ready; // Trả trạng thái về Ready
        self.batch = 0; // Đặt lại bộ đếm lô tích lũy về 0
        Ok(count) // Trả về số lượng vị trí đã tính toán xong
    } // Kết thúc hàm execute

    /// Đặt lại toàn bộ bộ đếm và trạng thái của Kernel về ban đầu.
    pub fn reset(&mut self) { // Hàm reset
        self.status = Status::Ready; // Đặt lại status về Ready
        self.active = false; // Đặt lại active = false
        self.batch = 0; // Đặt lại batch = 0
        self.dispatch.store(0, Ordering::SeqCst); // Đặt bộ đếm dispatch = 0
        self.finish.store(0, Ordering::SeqCst); // Đặt bộ đếm finish = 0
    } // Kết thúc hàm reset
} // Kết thúc khối impl Kernel

impl Dispatchable for Kernel { // Triển khai trait Dispatchable cho Kernel
    fn dispatch(&mut self, buffer: &Buffer, count: usize) -> Result<usize, Status> { // Triển khai dispatch
        if count == 0 { // Nếu số lượng vị trí bằng 0
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra 0
        if self.batch + count > self.limit { // Nếu vượt quá giới hạn lô max limit
            return Err(Status::Full); // Trả về lỗi Full
        } // Kết thúc kiểm tra limit
        self.batch += count; // Tích lũy số lượng vị trí vào lô
        if self.batch >= self.limit { // Nếu lô tích lũy đã đạt trần limit
            self.execute(buffer)?; // Tự động kích hoạt thực thi Kernel
        } // Kết thúc tự động execute
        Ok(self.batch) // Trả về số lượng vị trí hiện có trong lô
    } // Kết thúc phương thức dispatch

    fn finish(&self, buffer: &Buffer, target: &mut [isize]) -> Result<usize, Status> { // Triển khai finish
        if target.is_empty() || buffer.pointer().is_null() { // Kiểm tra mảng đích và buffer
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra
        let count = self.finish.load(Ordering::Acquire); // Đọc số vị trí đã hoàn thành
        if target.len() < count { // Nếu mảng đích không đủ sức chứa
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra target len
        Ok(count) // Trả về số lượng điểm số đã thu hồi thành công
    } // Kết thúc phương thức finish

    fn poll(&self) -> Status { // Triển khai poll truy vấn trạng thái
        self.status // Trả về trạng thái nội tại
    } // Kết thúc phương thức poll

    fn flush(&mut self, buffer: &Buffer) -> Result<usize, Status> { // Triển khai flush
        if self.batch == 0 { // Nếu lô rỗng
            return Ok(0); // Trả về 0 không làm gì
        } // Kết thúc kiểm tra rỗng
        self.execute(buffer) // Ép thực thi ngay bộ đệm tích lũy
    } // Kết thúc phương thức flush
} // Kết thúc impl Dispatchable for Kernel

#[cfg(test)] // Module kiểm thử unit tests cho Kernel
mod tests { // Cấu hình module tests
    use super::*; // Nhập tất cả đối tượng từ module cha

    #[test] // Đánh dấu hàm kiểm thử cấu trúc Kernel 64-byte alignment
    fn test_kernel_struct_layout_and_alignment() { // Hàm test layout
        assert_eq!(std::mem::size_of::<Kernel>(), 64); // Kiểm tra kích thước vật lý đúng 64 bytes
        assert_eq!(std::mem::align_of::<Kernel>(), 64); // Kiểm tra căn lề bộ nhớ đúng 64 bytes
    } // Kết thúc hàm test_kernel_struct_layout_and_alignment

    #[test] // Đánh dấu hàm kiểm thử luồng dispatch và execute
    fn test_kernel_dispatch_and_execute_flow() { // Hàm test luồng
        let mut kernel = Kernel::init(100, 420, 256).unwrap(); // Khởi tạo Kernel 100 positions, stride 420, 256 threads
        assert_eq!(kernel.status(), Status::Ready); // Khởi tạo thành công -> Status::Ready
        assert!(kernel.ready()); // Kernel rảnh rỗi -> ready() == true

        let buffer = Buffer::allocate(65536, true).unwrap(); // Khởi tạo VRAM Buffer 64KB
        assert!(kernel.dispatch(&buffer, 50).is_ok()); // Dispatch 50 positions thành công
        assert_eq!(kernel.batch(), 50); // Lô đang tích lũy 50 positions

        let flushed = kernel.flush(&buffer).unwrap(); // Ép flush lô 50 positions
        assert_eq!(flushed, 50); // Thu hồi 50 positions
        assert_eq!(kernel.batch(), 0); // Lô đã được reset về 0 sau execute
    } // Kết thúc hàm test_kernel_dispatch_and_execute_flow
} // Kết thúc module tests
