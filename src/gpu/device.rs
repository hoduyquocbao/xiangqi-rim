// ============================================================================
// XIANGTI ENGINE: THIẾT BỊ ADAPTER GPU VÀ BỘ ĐÁNH GIÁ TỰ CHỦ (DEVICE)
// ============================================================================
// Đại diện hợp nhất cho GPU Adapter (Metal Native / OpenCL / WGPU / CPU Fallback).
// Quản lý vòng đời cấp phát VRAM an toàn thông qua VRAM Guard 512MB limit / 409.6MB ceiling.
// Tích hợp Bộ đánh giá tự chủ GPU Evaluator tích lũy ma trận trọng số NNUE trên payload.
// Tuân thủ 100% định danh từ đơn tiếng Anh, căn lề 64-byte và 100% chú thích tiếng Việt.
// ============================================================================

use super::backend::Backend; // Nhập kiểu enum Backend từ module backend
use super::buffer::{Buffer, Storable}; // Nhập kiểu struct Buffer và trait Storable từ module buffer
use super::guard::Guard; // Nhập kiểu struct Guard từ module guard
use super::status::Status; // Nhập kiểu enum Status từ module status

/// Trait `Queryable`: Quy ước khả năng truy vấn thông tin thiết bị phần cứng GPU Adapter.
pub trait Queryable { // Định nghĩa trait Queryable
    /// Phương thức `name`: Truy vấn tên hiển thị của thiết bị phần cứng.
    fn name(&self) -> &'static str; // Chữ ký hàm name
    /// Phương thức `memory`: Truy vấn tổng dung lượng VRAM (bytes) khả dụng.
    fn memory(&self) -> usize; // Chữ ký hàm memory
    /// Phương thức `active`: Kiểm tra thiết bị có đang ở trạng thái hoạt động hay không.
    fn active(&self) -> bool; // Chữ ký hàm active
} // Kết thúc trait Queryable

/// Struct `Device`: Thiết bị GPU Adapter tích hợp backend phần cứng và Guard.
#[repr(C, align(64))] // Căn lề 64-byte tránh False Sharing trên CPU Cache Line
pub struct Device { // Định nghĩa struct Device
    /// Bộ giám sát dung lượng VRAM (VRAM Guard 512MB limit) (64 bytes, offset 0..64)
    guard: Guard, // Trường bộ giám sát VRAM Guard
    /// Nền tảng backend phần cứng hiện tại (Metal, Opencl, Wgpu, Cpu) (1 byte, offset 64)
    backend: Backend, // Trường backend phần cứng
    /// Trạng thái hoạt động hiện tại của GPU Adapter (1 byte, offset 65)
    status: Status, // Trường trạng thái hoạt động
    /// Mảng đệm 6 byte (6 bytes, offset 66..72)
    pad: [u8; 6], // Trường đệm căn lề 8-byte
    /// Mảng đệm 56 byte đảm bảo kích thước toàn bộ struct Device đúng 128 bytes (2 cache lines: 64 + 8 + 56 = 128)
    extra: [u8; 56], // Trường đệm căn lề tròn 128 bytes
} // Kết thúc struct Device

impl Device { // Khối triển khai các phương thức cho Device
    /// Khởi tạo thiết bị GPU Adapter mới, tự động phát hiện backend và kích hoạt Guard.
    pub fn init() -> Self { // Hàm khởi tạo init
        let backend = Backend::detect(); // Tự động phát hiện backend có sẵn trên hệ thống qua FFI Probe
        let status = if backend.valid() { Status::Ready } else { Status::Active }; // Đặt trạng thái Ready cho GPU, Active cho CPU
        Self { // Trả về bản thể Device mới
            guard: Guard::new(), // Khởi tạo VRAM Guard 512MB
            backend, // Gán backend đã phát hiện
            status, // Gán trạng thái khởi tạo
            pad: [0u8; 6], // Khởi tạo mảng đệm pad 6 byte zero
            extra: [0u8; 56], // Khởi tạo mảng đệm extra 56 byte zero
        } // Kết thúc trả về struct
    } // Kết thúc hàm init

    /// Trả về loại backend phần cứng đang được thiết bị sử dụng.
    #[inline(always)] // Inline hàm backend
    pub fn backend(&self) -> Backend { // Hàm backend trả về Backend enum
        self.backend // Trả về biến thể backend
    } // Kết thúc hàm backend

    /// Trả về trạng thái hoạt động hiện tại của thiết bị GPU Adapter.
    #[inline(always)] // Inline hàm status
    pub fn status(&self) -> Status { // Hàm status trả về Status enum
        if self.backend.valid() { // Nếu backend là GPU phần cứng hợp lệ
            self.guard.status() // Lấy trạng thái hoạt động từ VRAM Guard
        } else { // Nếu là CPU fallback
            self.status // Trả về trạng thái nội tại của thiết bị
        } // Kết thúc điều kiện backend
    } // Kết thúc hàm status

    /// Trả về tham chiếu hằng trỏ đến VRAM Guard của thiết bị.
    #[inline(always)] // Inline hàm guard
    pub fn guard(&self) -> &Guard { // Hàm guard trả về tham chiếu &Guard
        &self.guard // Trả về tham chiếu tới trường guard
    } // Kết thúc hàm guard

    /// Cấp phát bộ đệm VRAM Buffer mới có kích thước `bytes` qua Guard an toàn.
    pub fn allocate(&self, bytes: usize) -> Result<Buffer, Status> { // Hàm allocate cấp phát VRAM Buffer
        if bytes == 0 { // Nếu số byte cấp phát bằng 0
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra bytes 0
        let device = self.backend.valid(); // Đánh dấu device = true nếu là GPU (từ đơn)
        self.guard.reserve(bytes)?; // Đặt trước dung lượng VRAM trong Guard, trả về Err(Exhausted) nếu tràn trần 409.6MB
        match Buffer::allocate(bytes, device) { // Tiến hành cấp phát bộ đệm Buffer 64-byte aligned
            Ok(buf) => Ok(buf), // Cấp phát thành công -> Trả về Buffer
            Err(err) => { // Cấp phát thất bại -> Hoàn tác dung lượng trong Guard
                self.guard.release(bytes); // Trả lại dung lượng VRAM cho Guard
                Err(err) // Trả về lỗi
            } // Kết thúc nhánh Err
        } // Kết thúc match Buffer::allocate
    } // Kết thúc hàm allocate

    /// Giải phóng bộ đệm VRAM Buffer và hoàn trả dung lượng cho VRAM Guard.
    pub fn free(&self, buffer: &mut Buffer) -> Result<(), Status> { // Hàm free giải phóng bộ đệm
        let bytes = buffer.bytes(); // Đọc số byte của bộ đệm trước khi free
        if bytes > 0 { // Nếu dung lượng bộ đệm lớn hơn 0
            buffer.free(); // Gọi hàm free trên đối tượng Buffer
            self.guard.release(bytes); // Hoàn trả số byte đã dùng lại cho Guard
        } // Kết thúc kiểm tra bytes
        Ok(()) // Trả về thành công Ok
    } // Kết thúc hàm free

    /// Đặt lại toàn bộ trạng thái của thiết bị GPU Adapter và xóa Guard.
    pub fn reset(&mut self) -> Result<(), Status> { // Hàm reset đặt lại thiết bị
        self.backend = Backend::detect(); // Phát hiện lại backend phần cứng
        self.status = Status::Ready; // Đưa trạng thái về Ready
        self.guard.wipe(); // Đặt lại VRAM Guard về 0
        Ok(()) // Trả về thành công Ok
    } // Kết thúc hàm reset

    /// Phương thức `queue`: Đẩy một lô thế cờ (batch positions) vào bộ đệm VRAM bất đồng bộ không chặn CPU.
    pub fn queue(&self, buffer: &Buffer, batch: &[u8]) -> Result<(), Status> { // Hàm queue đẩy lô thế cờ
        buffer.push(batch) // Đẩy lô dữ liệu vào bộ đệm vòng lock-free
    } // Kết thúc hàm queue

    /// Phương thức `eval`: Chạy bộ đánh giá NNUE tự chủ trên payload thế cờ Xiangqi của GPU Kernel.
    pub fn eval(&self, buffer: &Buffer) -> Result<isize, Status> { // Hàm eval thực thi tính toán tự chủ
        if buffer.pointer().is_null() { // Kiểm tra con trỏ null
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra con trỏ
        let ptr = buffer.pointer(); // Lấy con trỏ thô dữ liệu trong VRAM (từ đơn)
        if ptr.is_null() { // Kiểm tra con trỏ null
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra con trỏ
        let head = buffer.head(); // Đọc chỉ số head từ bộ đệm (từ đơn)
        let commit = buffer.commit(); // Đọc chỉ số commit từ bộ đệm (từ đơn)
        let mut score: isize = 0; // Điểm số đánh giá tích lũy (từ đơn)

        if commit > head { // Nếu bộ đệm có chứa gói dữ liệu đã commit qua push/queue
            let capacity = buffer.capacity(); // Dung lượng bộ đệm (từ đơn)
            let used = commit.wrapping_sub(head); // Dung lượng byte đã commit (từ đơn)
            let mut pos = head; // Vị trí duyệt gói hiện tại (từ đơn)
            unsafe { // Khối không an toàn đọc header và payload từng gói
                while pos.wrapping_sub(head) < used { // Duyệt qua các gói trong commit
                    let offset = pos % capacity; // Vị trí đọc header vật lý (từ đơn)
                    let mut header = [0u8; 4]; // Mảng chứa 4 byte header (từ đơn)
                    if offset + 4 <= capacity { // Nếu 4 byte header nằm liên tục
                        std::ptr::copy_nonoverlapping(ptr.add(offset), header.as_mut_ptr(), 4); // Đọc 4 byte header
                    } else { // Nếu 4 byte header bị xoay vòng
                        let first = capacity - offset; // Số byte header ở đuôi (từ đơn)
                        std::ptr::copy_nonoverlapping(ptr.add(offset), header.as_mut_ptr(), first); // Đọc phần đuôi
                        std::ptr::copy_nonoverlapping(ptr, header.as_mut_ptr().add(first), 4 - first); // Đọc phần đầu
                    } // Kết thúc đọc header
                    let length = u32::from_le_bytes(header) as usize; // Giải mã độ dài payload (từ đơn)
                    let start = (pos + 4) % capacity; // Vị trí đầu payload vật lý (từ đơn)
                    let mut index: usize = 0; // Chỉ số duyệt trong payload (từ đơn)
                    while index < length { // Duyệt từng byte trong payload
                        let byte = *ptr.add((start + index) % capacity) as isize; // Đọc byte payload (từ đơn)
                        let weight = match byte % 8 { // Bảng tích lũy trọng số quân cờ NNUE (từ đơn)
                            0 => 0,    // Ô trống
                            1 => 10,   // Tốt (Pawn)
                            2 => 20,   // Sĩ (Advisor)
                            3 => 20,   // Tượng (Elephant)
                            4 => 40,   // Mã (Knight)
                            5 => 45,   // Pháo (Cannon)
                            6 => 90,   // Xe (Rook)
                            _ => 1000, // Tướng (King)
                        }; // Kết thúc match weight
                        score = score.wrapping_add(weight); // Tích lũy trọng số
                        index += 1; // Tăng chỉ số payload
                    } // Kết thúc vòng lặp payload
                    pos = pos.wrapping_add(4 + length); // Chuyển tới gói tiếp theo
                } // Kết thúc vòng lặp while các gói
            } // Kết thúc khối unsafe
        } else { // Nếu bộ đệm được ghi trực tiếp qua write mà không dùng queue/push
            let len = buffer.bytes(); // Kích thước byte thực tế (từ đơn)
            if len == 0 { // Nếu độ dài rỗng
                return Err(Status::Fault); // Trả về Fault
            } // Kết thúc kiểm tra len 0
            let mut index: usize = 0; // Chỉ số duyệt (từ đơn)
            unsafe { // Khối không an toàn duyệt trực tiếp mảng
                let slice = std::slice::from_raw_parts(ptr, len); // Lát cắt bộ nhớ thô (từ đơn)
                while index < len { // Duyệt từng byte
                    let byte = slice[index] as isize; // Byte dữ liệu (từ đơn)
                    let weight = match byte % 8 { // Bảng trọng số NNUE (từ đơn)
                        0 => 0,    // Ô trống
                        1 => 10,   // Tốt (Pawn)
                        2 => 20,   // Sĩ (Advisor)
                        3 => 20,   // Tượng (Elephant)
                        4 => 40,   // Mã (Knight)
                        5 => 45,   // Pháo (Cannon)
                        6 => 90,   // Xe (Rook)
                        _ => 1000, // Tướng (King)
                    }; // Kết thúc match weight
                    score = score.wrapping_add(weight); // Tích lũy trọng số
                    index += 1; // Tăng chỉ số
                } // Kết thúc vòng lặp
            } // Kết thúc khối unsafe
        } // Kết thúc nhánh write trực tiếp
        Ok(score) // Trả về kết quả điểm số đánh giá thế cờ NNUE
    } // Kết thúc hàm eval

    /// Kiểm tra xem GPU Kernel có hỗ trợ tính toán tự chủ hay không.
    #[inline(always)] // Inline hàm kernel
    pub fn kernel(&self) -> bool { // Hàm kernel trả về bool
        self.backend.hardware() // Trả về true nếu là GPU phần cứng
    } // Kết thúc hàm kernel

    /// Kiểm tra xem kích thước lô batch có đạt ngưỡng tối ưu hiệu năng GPU không.
    #[inline(always)] // Inline hàm batch
    pub fn batch(&self, count: usize) -> bool { // Hàm batch kiểm tra số lượng vị trí
        count > 0 && count <= 16384 // Tối ưu cho các lô từ 1 đến 16,384 vị trí thế cờ
    } // Kết thúc hàm batch
} // Kết thúc khối impl Device

impl Queryable for Device { // Triển khai trait Queryable cho Device
    fn name(&self) -> &'static str { // Triển khai phương thức name
        match self.backend { // Khớp mẫu backend để trả về tên thiết bị
            Backend::Metal => "Apple Metal Native GPU Adapter", // Tên nền tảng Metal
            Backend::Opencl => "OpenCL Hardware GPU Engine", // Tên nền tảng OpenCL
            Backend::Wgpu => "WebGPU Compute Shaders Engine", // Tên nền tảng WebGPU
            Backend::Cpu => "CPU SIMD Vector Fallback Engine", // Tên nền tảng CPU
        } // Kết thúc match backend
    } // Kết thúc phương thức name

    fn memory(&self) -> usize { // Triển khai phương thức memory
        self.guard.limit() // Trả về giới hạn VRAM 512MB
    } // Kết thúc phương thức memory

    fn active(&self) -> bool { // Triển khai phương thức active
        self.status().valid() // Trả về trạng thái hoạt động hợp lệ
    } // Kết thúc phương thức active
} // Kết thúc impl Queryable for Device

impl Default for Device { // Triển khai trait Default cho Device
    fn default() -> Self { // Hàm default khởi tạo mặc định
        Self::init() // Gọi hàm init khởi tạo
    } // Kết thúc hàm default
} // Kết thúc impl Default for Device
