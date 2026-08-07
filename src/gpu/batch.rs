// ============================================================================
// XIANGTI ENGINE: CONTAINER CHỨA LÔ THẾ CỜ TRUYỀN VRAM (BATCH)
// ============================================================================
// Struct `Batch` quản lý lô từ 1k đến 8k đối tượng `Sample` liên tục trên VRAM.
// Tận dụng cơ chế Unified Memory 0-Copy (`shared` mode) trên macOS Intel iGPU.
// Căn lề 64-byte vật lý phòng chống False Sharing.
// Tuân thủ 100% định danh từ đơn tiếng Anh và 100% chú thích tiếng Việt.
// ============================================================================

use super::buffer::Buffer; // Nhập kiểu struct Buffer từ module buffer
use super::device::Device; // Nhập kiểu struct Device từ module device
use super::sample::Sample; // Nhập kiểu struct Sample từ module sample
use super::status::Status; // Nhập kiểu enum Status từ module status

/// Trait `Batchable`: Định nghĩa khả năng quản lý lô mẫu thế cờ truyền lên GPU.
pub trait Batchable { // Định nghĩa trait Batchable
    /// Phương thức `allocate`: Khởi tạo và cấp phát VRAM cho lô thế cờ với sức chứa `capacity`.
    fn allocate(device: &Device, capacity: usize) -> Result<Batch, Status> where Self: Sized; // Chữ ký hàm allocate
    /// Phương thức `push`: Đẩy 1 mẫu thế cờ `Sample` vào cuối lô thế cờ.
    fn push(&mut self, sample: &Sample) -> Result<(), Status>; // Chữ ký hàm push
    /// Phương thức `pull`: Trích xuất 1 mẫu thế cờ `Sample` tại chỉ số `index` trong lô.
    fn pull(&self, index: usize) -> Result<Sample, Status>; // Chữ ký hàm pull
    /// Phương thức `write`: Ghi đè 1 mẫu thế cờ `Sample` tại chỉ số `index` trong lô.
    fn write(&mut self, index: usize, sample: &Sample) -> Result<(), Status>; // Chữ ký hàm write
    /// Phương thức `full`: Kiểm tra xem lô thế cờ đã đầy sức chứa hay chưa.
    fn full(&self) -> bool; // Chữ ký hàm full
    /// Phương thức `empty`: Kiểm tra xem lô thế cờ có rỗng hay không.
    fn empty(&self) -> bool; // Chữ ký hàm empty
    /// Phương thức `clear`: Đặt lại số lượng mẫu trong lô về 0.
    fn clear(&mut self); // Chữ ký hàm clear
} // Kết thúc trait Batchable

/// Struct `Batch`: Container quản lý lô mẫu thế cờ căn lề 64-byte (128 bytes total).
#[repr(C, align(64))] // Căn lề 64-byte phòng False Sharing trên CPU Cache Line
pub struct Batch { // Định nghĩa struct Batch
    /// Bộ đệm VRAM Buffer liên tục (64 bytes, offset 0..64)
    buffer: Buffer, // Trường bộ đệm VRAM buffer
    /// Số lượng mẫu thế cờ hiện có trong lô (8 bytes, offset 64..72)
    count: usize, // Trường số lượng mẫu count
    /// Sức chứa tối đa số lượng mẫu của lô (8 bytes, offset 72..80)
    capacity: usize, // Trường sức chứa tối đa capacity
    /// Kích thước byte của mỗi phần tử mẫu (128 bytes) (8 bytes, offset 80..88)
    stride: usize, // Trường độ dài phần tử stride
    /// Trạng thái kết quả của lô (1 byte, offset 88)
    status: Status, // Trường trạng thái status
    /// Cờ hoạt động của lô (1 byte, offset 89)
    active: bool, // Trường cờ hoạt động active
    /// Cờ bộ nhớ dùng chung 0-copy (1 byte, offset 90)
    shared: bool, // Trường cờ bộ nhớ dùng chung shared
    /// Mảng đệm 37 byte đảm bảo tổng kích thước struct đúng 128 bytes (37 bytes, offset 91..128)
    pad: [u8; 37], // Trường đệm pad căn lề 128 bytes
} // Kết thúc struct Batch

impl Batch { // Khối triển khai các phương thức cho Batch
    /// Khởi tạo một Batch mới chứa tối đa `capacity` mẫu.
    pub fn allocate(device: &Device, capacity: usize) -> Result<Self, Status> { // Hàm cấp phát allocate
        if capacity == 0 || capacity > 16384 { // Kiểm tra sức chứa không hợp lệ
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra capacity
        let stride = std::mem::size_of::<Sample>(); // Lấy kích thước byte của Sample (128 bytes)
        let bytes = capacity.saturating_mul(stride); // Tính tổng số byte cần cấp phát VRAM
        let buffer = device.allocate(bytes)?; // Cấp phát VRAM Buffer từ Device
        let shared = buffer.shared(); // Đọc cờ zero-copy shared mode
        Ok(Self { // Trả về bản thể Batch mới
            buffer, // Gán bộ đệm buffer
            count: 0, // Khởi tạo số lượng mẫu count = 0
            capacity, // Gán sức chứa capacity
            stride, // Gán độ dài stride
            status: Status::Ready, // Khởi tạo trạng thái Ready
            active: true, // Gán cờ active = true
            shared, // Gán cờ shared
            pad: [0u8; 37], // Khởi tạo mảng đệm pad zero 37 bytes
        }) // Kết thúc kết quả Ok
    } // Kết thúc hàm allocate

    /// Đẩy thêm 1 mẫu `Sample` vào cuối lô Batch.
    pub fn push(&mut self, sample: &Sample) -> Result<(), Status> { // Hàm push thêm mẫu
        if self.count >= self.capacity { // Nếu số lượng mẫu đã đạt sức chứa tối đa
            return Err(Status::Full); // Trả về lỗi Full
        } // Kết thúc kiểm tra đầy lô
        let offset = self.count * self.stride; // Tính vị trí offset byte trong buffer
        let bytes = std::mem::size_of::<Sample>(); // Độ dài byte của Sample (128 bytes)
        let src = unsafe { // Khối unsafe lấy lát cắt byte của sample
            std::slice::from_raw_parts(sample as *const Sample as *const u8, bytes) // Lấy con trỏ thô của sample
        }; // Kết thúc khối unsafe
        let ptr = self.buffer.mutable(); // Lấy con trỏ khả biến tới vùng nhớ buffer
        if ptr.is_null() { // Nếu con trỏ null
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra con trỏ null
        unsafe { // Khối unsafe sao chép bộ nhớ
            std::ptr::copy_nonoverlapping(src.as_ptr(), ptr.add(offset), bytes); // Ghi sample vào offset
        } // Kết thúc khối unsafe
        self.count += 1; // Tăng số lượng mẫu count lên 1
        self.status = Status::Active; // Cập nhật trạng thái Active
        Ok(()) // Trả về thành công Ok
    } // Kết thúc hàm push

    /// Trích xuất mẫu `Sample` tại vị trí `index` trong lô.
    pub fn pull(&self, index: usize) -> Result<Sample, Status> { // Hàm pull đọc mẫu
        if index >= self.count { // Nếu chỉ số vượt quá số lượng mẫu hiện có
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra chỉ số
        let offset = index * self.stride; // Tính vị trí offset byte trong buffer
        let bytes = std::mem::size_of::<Sample>(); // Độ dài byte của Sample (128 bytes)
        let ptr = self.buffer.pointer(); // Lấy con trỏ hằng tới vùng nhớ buffer
        if ptr.is_null() { // Nếu con trỏ null
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra con trỏ null
        let mut sample = Sample::new(); // Khởi tạo mẫu thế cờ tạm
        unsafe { // Khối unsafe sao chép bộ nhớ
            let dst = &mut sample as *mut Sample as *mut u8; // Lấy con trỏ đích của sample
            std::ptr::copy_nonoverlapping(ptr.add(offset), dst, bytes); // Đọc từ buffer vào sample
        } // Kết thúc khối unsafe
        Ok(sample) // Trả về mẫu thế cờ đã đọc
    } // Kết thúc hàm pull

    /// Ghi đè/cập nhật mẫu `Sample` tại chỉ số `index` trong lô.
    pub fn write(&mut self, index: usize, sample: &Sample) -> Result<(), Status> { // Hàm write ghi mẫu
        if index >= self.count { // Nếu chỉ số vượt quá số lượng mẫu hiện có
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra chỉ số
        let offset = index * self.stride; // Tính vị trí offset byte trong buffer
        let bytes = std::mem::size_of::<Sample>(); // Độ dài byte của Sample (128 bytes)
        let src = unsafe { // Khối unsafe lấy lát cắt byte của sample
            std::slice::from_raw_parts(sample as *const Sample as *const u8, bytes) // Lấy con trỏ thô của sample
        }; // Kết thúc khối unsafe
        let ptr = self.buffer.mutable(); // Lấy con trỏ khả biến tới vùng nhớ buffer
        if ptr.is_null() { // Nếu con trỏ null
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra con trỏ null
        unsafe { // Khối unsafe sao chép bộ nhớ
            std::ptr::copy_nonoverlapping(src.as_ptr(), ptr.add(offset), bytes); // Ghi sample vào offset
        } // Kết thúc khối unsafe
        Ok(()) // Trả về thành công Ok
    } // Kết thúc hàm write

    /// Trả về số lượng mẫu hiện có trong lô.
    #[inline(always)] // Inline hàm đọc count
    pub fn count(&self) -> usize { // Hàm count trả về usize
        self.count // Trả về count
    } // Kết thúc hàm count

    /// Trả về sức chứa tối đa của lô.
    #[inline(always)] // Inline hàm đọc capacity
    pub fn capacity(&self) -> usize { // Hàm capacity trả về usize
        self.capacity // Trả về capacity
    } // Kết thúc hàm capacity

    /// Trả về kích thước stride của 1 mẫu.
    #[inline(always)] // Inline hàm đọc stride
    pub fn stride(&self) -> usize { // Hàm stride trả về usize
        self.stride // Trả về stride
    } // Kết thúc hàm stride

    /// Trả về tổng số bytes dữ liệu của các mẫu đã nạp trong lô.
    #[inline(always)] // Inline hàm đọc bytes
    pub fn bytes(&self) -> usize { // Hàm bytes trả về usize
        self.count * self.stride // Trả về tổng số bytes
    } // Kết thúc hàm bytes

    /// Trả về trạng thái kết quả của lô.
    #[inline(always)] // Inline hàm đọc status
    pub fn status(&self) -> Status { // Hàm status trả về Status
        self.status // Trả về status
    } // Kết thúc hàm status

    /// Trả về cờ hoạt động active.
    #[inline(always)] // Inline hàm đọc active
    pub fn active(&self) -> bool { // Hàm active trả về bool
        self.active // Trả về active
    } // Kết thúc hàm active

    /// Trả về cờ bộ nhớ dùng chung zero-copy shared.
    #[inline(always)] // Inline hàm đọc shared
    pub fn shared(&self) -> bool { // Hàm shared trả về bool
        self.shared // Trả về shared
    } // Kết thúc hàm shared

    /// Trả về con trỏ hằng tới vùng nhớ VRAM của lô.
    #[inline(always)] // Inline hàm đọc pointer
    pub fn pointer(&self) -> *const u8 { // Hàm pointer trả về *const u8
        self.buffer.pointer() // Gọi pointer từ buffer
    } // Kết thúc hàm pointer

    /// Trả về con trỏ khả biến tới vùng nhớ VRAM của lô.
    #[inline(always)] // Inline hàm đọc mutable
    pub fn mutable(&mut self) -> *mut u8 { // Hàm mutable trả về *mut u8
        self.buffer.mutable() // Gọi mutable từ buffer
    } // Kết thúc hàm mutable

    /// Trả về tham chiếu tới bộ đệm VRAM Buffer.
    #[inline(always)] // Inline hàm đọc buffer
    pub fn buffer(&self) -> &Buffer { // Hàm buffer trả về &Buffer
        &self.buffer // Trả về tham chiếu buffer
    } // Kết thúc hàm buffer

    /// Kiểm tra xem lô đã đầy sức chứa chưa.
    #[inline(always)] // Inline hàm full
    pub fn full(&self) -> bool { // Hàm full trả về bool
        self.count >= self.capacity // So sánh count và capacity
    } // Kết thúc hàm full

    /// Kiểm tra xem lô có rỗng không.
    #[inline(always)] // Inline hàm empty
    pub fn empty(&self) -> bool { // Hàm empty trả về bool
        self.count == 0 // So sánh count với 0
    } // Kết thúc hàm empty

    /// Đặt lại số lượng mẫu trong lô về 0.
    #[inline(always)] // Inline hàm clear
    pub fn clear(&mut self) { // Hàm clear xóa lô
        self.count = 0; // Đặt count = 0
        self.status = Status::Ready; // Trả trạng thái về Ready
    } // Kết thúc hàm clear
} // Kết thúc khối impl Batch

impl Batchable for Batch { // Triển khai trait Batchable cho Batch
    fn allocate(device: &Device, capacity: usize) -> Result<Batch, Status> { // Triển khai allocate
        Batch::allocate(device, capacity) // Gọi phương thức allocate của Batch
    } // Kết thúc phương thức allocate

    fn push(&mut self, sample: &Sample) -> Result<(), Status> { // Triển khai push
        self.push(sample) // Gọi phương thức push nội tại
    } // Kết thúc phương thức push

    fn pull(&self, index: usize) -> Result<Sample, Status> { // Triển khai pull
        self.pull(index) // Gọi phương thức pull nội tại
    } // Kết thúc phương thức pull

    fn write(&mut self, index: usize, sample: &Sample) -> Result<(), Status> { // Triển khai write
        self.write(index, sample) // Gọi phương thức write nội tại
    } // Kết thúc phương thức write

    fn full(&self) -> bool { // Triển khai full
        self.full() // Gọi phương thức full nội tại
    } // Kết thúc phương thức full

    fn empty(&self) -> bool { // Triển khai empty
        self.empty() // Gọi phương thức empty nội tại
    } // Kết thúc phương thức empty

    fn clear(&mut self) { // Triển khai clear
        self.clear(); // Gọi phương thức clear nội tại
    } // Kết thúc phương thức clear
} // Kết thúc impl Batchable for Batch

#[cfg(test)] // Module kiểm thử unit tests cho Batch
mod tests { // Cấu hình module tests
    use super::*; // Nhập tất cả đối tượng từ module cha

    #[test] // Đánh dấu hàm kiểm thử cấu trúc và căn lề bộ nhớ 128-byte
    fn test_batch_struct_layout_and_alignment() { // Hàm test layout Batch
        assert_eq!(std::mem::size_of::<Batch>(), 128); // Kiểm tra size_of đúng 128 bytes
        assert_eq!(std::mem::align_of::<Batch>(), 64); // Kiểm tra align_of đúng 64 bytes
    } // Kết thúc hàm test_batch_struct_layout_and_alignment

    #[test] // Đánh dấu hàm kiểm thử cấp phát, đẩy và rút dữ liệu mẫu
    fn test_batch_push_pull_and_operations() { // Hàm test thao tác Batch
        let device = Device::init(); // Khởi tạo thiết bị GPU Device
        let mut batch = Batch::allocate(&device, 10).unwrap(); // Cấp phát Batch chứa 10 mẫu
        assert!(batch.empty()); // Ban đầu lô rỗng -> empty() == true
        assert!(!batch.full()); // Ban đầu lô chưa đầy -> full() == false
        assert_eq!(batch.capacity(), 10); // Sức chứa capacity bằng 10
        assert_eq!(batch.count(), 0); // Số lượng mẫu count bằng 0

        let mut sample = Sample::new(); // Khởi tạo mẫu thế cờ
        sample.store(150); // Đặt điểm số 150 centipawns
        assert!(batch.push(&sample).is_ok()); // Đẩy mẫu vào batch thành công
        assert_eq!(batch.count(), 1); // Số lượng mẫu cập nhật bằng 1
        assert!(!batch.empty()); // Lô không còn rỗng

        let read = batch.pull(0).unwrap(); // Trích xuất mẫu tại chỉ số 0
        assert_eq!(read.score(), 150); // Điểm số trích xuất đúng 150

        batch.clear(); // Đặt lại lô về rỗng
        assert!(batch.empty()); // Sau clear lô rỗng hoàn toàn
        assert_eq!(batch.count(), 0); // Count bằng 0
    } // Kết thúc hàm test_batch_push_pull_and_operations

    #[test] // Đánh dấu hàm kiểm thử ghi đè mẫu thế cờ vào lô
    fn test_batch_write_back_operations() { // Hàm test write Batch
        let device = Device::init(); // Khởi tạo thiết bị GPU Device
        let mut batch = Batch::allocate(&device, 10).unwrap(); // Cấp phát Batch chứa 10 mẫu
        let mut sample = Sample::new(); // Khởi tạo mẫu thế cờ
        sample.store(100); // Đặt điểm 100
        batch.push(&sample).unwrap(); // Đẩy mẫu vào batch
        assert_eq!(batch.pull(0).unwrap().score(), 100); // Kiểm tra điểm ban đầu 100

        let mut updated = batch.pull(0).unwrap(); // Trích xuất bản sao
        updated.store(500); // Cập nhật điểm 500
        assert!(batch.write(0, &updated).is_ok()); // Ghi ngược trở lại batch tại index 0

        let read = batch.pull(0).unwrap(); // Đọc lại mẫu
        assert_eq!(read.score(), 500); // Điểm số cập nhật đúng 500
    } // Kết thúc hàm test_batch_write_back_operations
} // Kết thúc module tests
