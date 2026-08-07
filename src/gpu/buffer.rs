// ============================================================================
// XIANGTI ENGINE: KHỐI BỘ NHỚ ĐỆM VRAM/RAM HÀNG ĐỢI KHÔNG KHÓA (BUFFER)
// ============================================================================
// Quản lý khối bộ đệm liên tục căn lề 64-byte phần cứng trên host và device.
// Tích hợp hàng đợi vòng không khóa (Lock-Free Ring Buffer Queue) sử dụng phép toán
// modulo xoay vòng (modulo wrapping) và đồng bộ hóa chỉ số nguyên tử commit (Atomic Commit Index).
// Tuân thủ 100% định danh từ đơn tiếng Anh, căn lề 64-byte và 100% chú thích tiếng Việt.
// ============================================================================

use std::alloc::{alloc_zeroed, dealloc, Layout}; // Nhập các hàm cấp phát bộ nhớ std::alloc
use std::ptr::{null_mut, copy_nonoverlapping, write_bytes}; // Nhập các hàm thao tác con trỏ std::ptr
use std::sync::atomic::{AtomicUsize, Ordering}; // Nhập các kiểu dữ liệu nguyên tử lock-free
use super::status::Status; // Nhập kiểu enum Status từ module status

/// Trait `Storable`: Định nghĩa khả năng đẩy và kéo dữ liệu qua lại giữa RAM host và VRAM device.
pub trait Storable { // Định nghĩa trait Storable
    /// Phương thức `push`: Đẩy dữ liệu từ mảng nguồn `source` vào bộ đệm VRAM.
    fn push(&self, source: &[u8]) -> Result<(), Status>; // Chữ ký phương thức push không khóa
    /// Phương thức `pull`: Đọc dữ liệu từ bộ đệm VRAM ra mảng đích `target`.
    fn pull(&self, target: &mut [u8]) -> Result<(), Status>; // Chữ ký phương thức pull không khóa
} // Kết thúc trait Storable

/// Struct `Buffer`: Đại diện cho một khối bộ đệm VRAM/RAM căn lề 64-byte tích hợp Ring Buffer.
#[repr(C, align(64))] // Căn lề 64-byte vật lý phòng chống False Sharing
pub struct Buffer { // Định nghĩa struct Buffer
    /// Con trỏ thô trỏ đến vùng nhớ đệm 64-byte aligned (8 bytes)
    pointer: *mut u8, // Trường con trỏ bộ nhớ
    /// Kích thước dữ liệu thực tế tính bằng bytes (8 bytes)
    bytes: usize, // Trường kích thước byte sử dụng
    /// Dung lượng tổng cộng được cấp phát tính bằng bytes (bội số của 64) (8 bytes)
    capacity: usize, // Trường dung lượng sức chứa tổng
    /// Con trỏ đầu hàng đợi vòng không khóa (Head atomic index) (8 bytes)
    head: AtomicUsize, // Trường chỉ số head nguyên tử
    /// Con trỏ đuôi hàng đợi vòng không khóa (Tail atomic index) (8 bytes)
    tail: AtomicUsize, // Trường chỉ số tail nguyên tử
    /// Con trỏ xuất bản dữ liệu hoàn thành (Commit atomic index) (8 bytes)
    commit: AtomicUsize, // Trường chỉ số commit nguyên tử
    /// Cờ xác nhận bộ đệm đã căn lề 64-byte hay chưa (1 byte)
    aligned: bool, // Trường cờ căn lề 64-byte
    /// Cờ xác định vùng nhớ đệm thuộc VRAM (device) hay RAM (host) (1 byte)
    device: bool, // Trường cờ thiết bị VRAM
    /// Cờ chế độ Unified Memory 0-Copy (Shared Mode) trên Intel iGPU macOS (1 byte)
    shared: bool, // Trường cờ chia sẻ bộ nhớ zero-copy
    /// Mảng đệm 13 byte đảm bảo tổng kích thước struct đúng 64 bytes (48 + 3 + 13 = 64)
    pad: [u8; 13], // Trường đệm căn lề cache line 64-byte
} // Kết thúc struct Buffer

impl Buffer { // Khối triển khai các phương thức cho Buffer
    /// Cấp phát một khối bộ đệm Buffer mới có dung lượng `bytes` được căn lề 64-byte.
    pub fn allocate(bytes: usize, device: bool) -> Result<Self, Status> { // Hàm cấp phát allocate
        if bytes == 0 { // Nếu dung lượng yêu cầu bằng 0
            return Err(Status::Fault); // Trả về lỗi Fault không hợp lệ
        } // Kết thúc kiểm tra dung lượng 0
        let capacity = match bytes.checked_next_power_of_two() { // Tính toán dung lượng theo lũy thừa 2 an toàn
            Some(c) => c.max(64), // Đảm bảo dung lượng tối thiểu 64 bytes
            None => return Err(Status::Fault), // Vượt quá giới hạn usize -> Trả về lỗi Fault
        }; // Kết thúc match checked_next_power_of_two
        let layout = match Layout::from_size_align(capacity, 64) { // Tạo cấu trúc Layout căn lề 64-byte
            Ok(l) => l, // Tạo thành công -> Lấy layout
            Err(_) => return Err(Status::Fault), // Tạo thất bại -> Trả về lỗi Fault
        }; // Kết thúc match Layout
        let pointer = unsafe { alloc_zeroed(layout) }; // Cấp phát bộ nhớ 0 bằng alloc_zeroed không an toàn
        if pointer.is_null() { // Nếu con trỏ trả về bị null (cấp phát thất bại)
            return Err(Status::Exhausted); // Trả về lỗi Exhausted do hết bộ nhớ
        } // Kết thúc kiểm tra con trỏ null
        let aligned = (pointer as usize) % 64 == 0; // Kiểm tra con trỏ thực tế có chia hết cho 64 không
        let shared = cfg!(target_os = "macos") && device; // Trên macOS Intel iGPU, tự động chọn Shared Zero-Copy Mode
        Ok(Self { // Trả về bản thể Buffer mới
            pointer, // Gán con trỏ vùng nhớ
            bytes, // Gán kích thước byte sử dụng
            capacity, // Gán dung lượng capacity đã làm tròn
            head: AtomicUsize::new(0), // Khởi tạo head index = 0
            tail: AtomicUsize::new(0), // Khởi tạo tail index = 0
            commit: AtomicUsize::new(0), // Khởi tạo commit index = 0
            aligned, // Gán cờ căn lề 64-byte (từ đơn)
            device, // Gán cờ thiết bị device (từ đơn)
            shared, // Gán cờ zero-copy shared mode (từ đơn)
            pad: [0u8; 13], // Khởi tạo mảng đệm 13 byte zero
        }) // Kết thúc kết quả Ok
    } // Kết thúc hàm allocate

    /// Trả về con trỏ hằng (*const u8) trỏ tới dữ liệu trong bộ đệm.
    #[inline(always)] // Inline hàm đọc con trỏ hằng
    pub fn pointer(&self) -> *const u8 { // Hàm pointer trả về *const u8
        self.pointer as *const u8 // Ép kiểu con trỏ thành *const u8
    } // Kết thúc hàm pointer

    /// Trả về con trỏ khả biến (*mut u8) trỏ tới dữ liệu trong bộ đệm.
    #[inline(always)] // Inline hàm đọc con trỏ khả biến
    pub fn mutable(&mut self) -> *mut u8 { // Hàm mutable trả về *mut u8
        self.pointer // Trả về con trỏ khả biến
    } // Kết thúc hàm mutable

    /// Trả về kích thước byte dữ liệu thực tế.
    #[inline(always)] // Inline hàm đọc bytes
    pub fn bytes(&self) -> usize { // Hàm bytes trả về usize
        self.bytes // Trả về số byte dữ liệu
    } // Kết thúc hàm bytes

    /// Trả về dung lượng cấp phát tổng cộng của bộ đệm.
    #[inline(always)] // Inline hàm đọc capacity
    pub fn capacity(&self) -> usize { // Hàm capacity trả về usize
        self.capacity // Trả về số byte capacity
    } // Kết thúc hàm capacity

    /// Kiểm tra xem bộ đệm có căn lề 64-byte hợp lệ hay không.
    #[inline(always)] // Inline hàm aligned
    pub fn aligned(&self) -> bool { // Hàm aligned trả về bool
        self.aligned // Trả về cờ aligned
    } // Kết thúc hàm aligned

    /// Kiểm tra xem bộ đệm thuộc vùng nhớ device VRAM hay không.
    #[inline(always)] // Inline hàm device
    pub fn device(&self) -> bool { // Hàm device trả về bool
        self.device // Trả về cờ device
    } // Kết thúc hàm device

    /// Kiểm tra xem bộ đệm có đang ở chế độ Unified Memory 0-Copy (Shared Mode) hay không.
    #[inline(always)] // Inline hàm shared
    pub fn shared(&self) -> bool { // Hàm shared trả về bool
        self.shared // Trả về cờ shared mode
    } // Kết thúc hàm shared

    /// Trả về chỉ số head hiện tại của hàng đợi vòng nguyên tử.
    #[inline(always)] // Inline hàm head
    pub fn head(&self) -> usize { // Hàm head trả về usize
        self.head.load(Ordering::Acquire) // Trả về giá trị head
    } // Kết thúc hàm head

    /// Trả về chỉ số commit hiện tại của hàng đợi vòng nguyên tử.
    #[inline(always)] // Inline hàm commit
    pub fn commit(&self) -> usize { // Hàm commit trả về usize
        self.commit.load(Ordering::Acquire) // Trả về giá trị commit
    } // Kết thúc hàm commit

    /// Ghi dữ liệu từ lát cắt host slice `source` vào trong bộ đệm.
    pub fn write(&mut self, source: &[u8]) -> Result<(), Status> { // Hàm write sao chép dữ liệu vào buffer
        if source.len() > self.capacity || self.pointer.is_null() { // Nếu độ dài nguồn vượt sức chứa hoặc con trỏ null
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra giới hạn ghi
        unsafe { // Khối không an toàn sao chép bộ nhớ
            copy_nonoverlapping(source.as_ptr(), self.pointer, source.len()); // Sao chép bộ nhớ không chồng lấp
        } // Kết thúc khối unsafe
        self.bytes = source.len(); // Cập nhật kích thước dữ liệu thực tế
        Ok(()) // Trả về thành công Ok
    } // Kết thúc hàm write

    /// Đọc dữ liệu từ bộ đệm ra lát cắt host slice khả biến `target`.
    pub fn read(&self, target: &mut [u8]) -> Result<(), Status> { // Hàm read đọc dữ liệu ra target
        if target.len() < self.bytes || self.pointer.is_null() { // Nếu mảng đích nhỏ hơn dữ liệu hoặc con trỏ null
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra giới hạn đọc
        unsafe { // Khối không an toàn sao chép bộ nhớ
            copy_nonoverlapping(self.pointer as *const u8, target.as_mut_ptr(), self.bytes); // Sao chép bộ nhớ ra mảng đích
        } // Kết thúc khối unsafe
        Ok(()) // Trả về thành công Ok
    } // Kết thúc hàm read

    /// Ghi 0 toàn bộ vùng nhớ đệm (Zero out buffer memory).
    pub fn clear(&mut self) { // Hàm clear xóa sạch dữ liệu về 0
        if !self.pointer.is_null() && self.capacity > 0 { // Nếu con trỏ hợp lệ và capacity lớn hơn 0
            unsafe { // Khối unsafe thao tác bộ nhớ
                write_bytes(self.pointer, 0, self.capacity); // Xóa sạch toàn bộ bytes về 0
            } // Kết thúc khối unsafe
            self.bytes = 0; // Đặt kích thước dữ liệu thực tế về 0
            self.head.store(0, Ordering::SeqCst); // Đặt head index về 0
            self.tail.store(0, Ordering::SeqCst); // Đặt tail index về 0
            self.commit.store(0, Ordering::SeqCst); // Đặt commit index về 0
        } // Kết thúc kiểm tra con trỏ
    } // Kết thúc hàm clear

    /// Giải phóng vùng nhớ đệm đã cấp phát và đưa con trỏ về null.
    pub fn free(&mut self) { // Hàm free giải phóng bộ nhớ
        if !self.pointer.is_null() && self.capacity > 0 { // Nếu con trỏ hợp lệ và capacity lớn hơn 0
            let layout = match Layout::from_size_align(self.capacity, 64) { // Tái tạo layout 64-byte alignment
                Ok(l) => l, // Lấy layout
                Err(_) => return, // Lỗi layout -> Bỏ qua
            }; // Kết thúc match Layout
            unsafe { // Khối không an toàn dealloc
                dealloc(self.pointer, layout); // Giải phóng bộ nhớ với dealloc
            } // Kết thúc khối unsafe
            self.pointer = null_mut(); // Đưa con trỏ về null_mut
            self.bytes = 0; // Đặt số byte dữ liệu về 0
            self.capacity = 0; // Đặt capacity về 0
            self.aligned = false; // Đặt cờ aligned về false
            self.shared = false; // Đặt cờ shared về false
        } // Kết thúc kiểm tra giải phóng
    } // Kết thúc hàm free
} // Kết thúc khối impl Buffer

impl Storable for Buffer { // Triển khai trait Storable cho Buffer
    fn push(&self, source: &[u8]) -> Result<(), Status> { // Triển khai phương thức push đóng khung u32 length header
        if source.is_empty() || self.pointer.is_null() { // Nếu mảng nguồn rỗng hoặc con trỏ null
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra
        let length = source.len(); // Độ dài dữ liệu mảng nguồn (từ đơn)
        let total = 4 + length; // Tổng số byte cần cho header u32 + payload (từ đơn)
        if total > self.capacity { // Nếu tổng số byte vượt quá dung lượng buffer
            return Err(Status::Full); // Trả về lỗi Full
        } // Kết thúc kiểm tra dung lượng

        let mut tail = self.tail.load(Ordering::Acquire); // Đọc chỉ số tail hiện tại (từ đơn)
        loop { // Vòng lặp CAS đặt trước vùng nhớ ghi
            let head = self.head.load(Ordering::Acquire); // Đọc chỉ số head hiện tại (từ đơn)
            let used = tail.wrapping_sub(head); // Dung lượng byte hiện đang bị chiếm dụng (từ đơn)
            let free = self.capacity.saturating_sub(used); // Dung lượng rảnh khả dụng (từ đơn)
            if total > free { // Nếu tổng số byte cần đẩy vượt quá dung lượng rảnh
                return Err(Status::Full); // Trả về lỗi Full
            } // Kết thúc kiểm tra rảnh
            let next = tail.wrapping_add(total); // Chỉ số tail dự kiến sau khi đẩy (từ đơn)
            match self.tail.compare_exchange_weak( // Thử đặt trước chỉ số tail qua CAS
                tail, // Giá trị tail cũ
                next, // Giá trị tail mới
                Ordering::SeqCst, // Thứ tự thành công
                Ordering::Acquire, // Thứ tự thất bại
            ) { // Xử lý kết quả CAS
                Ok(_) => break, // Đặt trước thành công -> Thoát vòng lặp
                Err(actual) => tail = actual, // Đặt trước thất bại -> Cập nhật tail và thử lại
            } // Kết thúc match CAS
        } // Kết thúc vòng lặp CAS

        let header = (length as u32).to_le_bytes(); // Chuyển độ dài thành mảng 4-byte header u32 le (từ đơn)
        let offset = tail % self.capacity; // Vị trí ghi header vật lý với toán tử modulo (từ đơn)

        unsafe { // Khối không an toàn sao chép header và payload vào buffer
            if offset + 4 <= self.capacity { // Nếu 4 byte header nằm liên tục
                copy_nonoverlapping(header.as_ptr(), self.pointer.add(offset), 4); // Ghi 4 byte header
            } else { // Nếu 4 byte header bị xoay vòng qua biên capacity
                let first = self.capacity - offset; // Số byte header ở phần đuôi
                copy_nonoverlapping(header.as_ptr(), self.pointer.add(offset), first); // Ghi phần đuôi
                copy_nonoverlapping(header.as_ptr().add(first), self.pointer, 4 - first); // Ghi phần đầu (offset 0)
            } // Kết thúc ghi header

            let start = (tail + 4) % self.capacity; // Vị trí ghi payload vật lý (từ đơn)
            if start + length <= self.capacity { // Nếu payload nằm liên tục
                copy_nonoverlapping(source.as_ptr(), self.pointer.add(start), length); // Ghi 1 khối payload liên tục
            } else { // Nếu payload bị xoay vòng qua biên capacity
                let first = self.capacity - start; // Số byte payload ở phần đuôi (từ đơn)
                copy_nonoverlapping(source.as_ptr(), self.pointer.add(start), first); // Ghi phần 1 ở đuôi
                copy_nonoverlapping(source.as_ptr().add(first), self.pointer, length - first); // Ghi phần 2 ở đầu
            } // Kết thúc ghi payload
        } // Kết thúc khối unsafe

        while self.commit.load(Ordering::Acquire) != tail { // Vòng lặp chờ spin lock-free cho xuất bản commit
            std::hint::spin_loop(); // Tối ưu hóa chu kỳ CPU spin-loop
        } // Kết thúc vòng lặp chờ commit
        self.commit.store(tail.wrapping_add(total), Ordering::SeqCst); // Tăng chỉ số commit nguyên tử xuất bản dữ liệu
        Ok(()) // Trả về thành công Ok
    } // Kết thúc phương thức push

    fn pull(&self, target: &mut [u8]) -> Result<(), Status> { // Triển khai phương thức pull đóng khung u32 length header
        if target.is_empty() || self.pointer.is_null() { // Nếu mảng đích rỗng hoặc con trỏ null
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra mảng đích và con trỏ
        let mut head = self.head.load(Ordering::Acquire); // Đọc chỉ số head hiện tại (từ đơn)
        loop { // Vòng lặp CAS đặt trước vùng nhớ đọc
            let commit = self.commit.load(Ordering::Acquire); // Đọc chỉ số commit nguyên tử đã xuất bản (từ đơn)
            let used = commit.wrapping_sub(head); // Dung lượng byte thực sự đã xuất bản có thể đọc (từ đơn)
            if used < 4 { // Nếu chưa có đủ 4 byte header
                return Err(Status::Ready); // Trả về Ready (sẵn sàng chờ dữ liệu mới)
            } // Kết thúc kiểm tra header

            let mut header = [0u8; 4]; // Mảng chứa 4 byte header (từ đơn)
            let offset = head % self.capacity; // Vị trí đọc header vật lý (từ đơn)
            unsafe { // Khối không an toàn đọc header
                if offset + 4 <= self.capacity { // Nếu 4 byte header nằm liên tục
                    copy_nonoverlapping(self.pointer.add(offset), header.as_mut_ptr(), 4); // Đọc 4 byte header
                } else { // Nếu 4 byte header bị xoay vòng qua biên capacity
                    let first = self.capacity - offset; // Số byte header ở đuôi (từ đơn)
                    copy_nonoverlapping(self.pointer.add(offset), header.as_mut_ptr(), first); // Đọc phần đuôi
                    copy_nonoverlapping(self.pointer, header.as_mut_ptr().add(first), 4 - first); // Đọc phần đầu
                } // Kết thúc đọc header
            } // Kết thúc khối unsafe

            let length = u32::from_le_bytes(header) as usize; // Giải mã độ dài payload từ header (từ đơn)
            let total = 4 + length; // Tổng số byte của cả gói bao gồm header (từ đơn)

            if used < total { // Nếu toàn bộ gói chưa được xuất bản commit hoàn tất
                return Err(Status::Ready); // Trả về Ready chờ commit đủ
            } // Kết thúc kiểm tra gói đủ

            if target.len() < length { // Nếu mảng đích của caller nhỏ hơn độ dài payload
                let actual = self.head.load(Ordering::Acquire); // Đọc lại chỉ số head thực tế kiểm tra tranh chấp (từ đơn)
                if actual != head { // Nếu luồng khác đã tranh chấp làm thay đổi head
                    head = actual; // Cập nhật head thực tế (từ đơn)
                    continue; // Tiếp tục vòng lặp retry không trả về lỗi giả Fault
                } // Kết thúc kiểm tra tranh chấp
                return Err(Status::Fault); // Trả về lỗi Fault thực sự do target nhỏ hơn length
            } // Kết thúc kiểm tra sức chứa target

            let start = (head + 4) % self.capacity; // Vị trí đọc payload vật lý (từ đơn)
            let mut stack = [0u8; 1024]; // Bộ đệm tạm thời trên stack cho gói nhỏ (từ đơn)
            let mut heap; // Khai báo bộ đệm tạm thời trên heap phòng khi gói lớn (từ đơn)
            let temp: &mut [u8] = if length <= 1024 { // Chọn bộ đệm tạm phù hợp kích thước
                &mut stack[..length] // Kích thước <= 1024 -> Dùng stack buffer
            } else { // Kích thước > 1024 -> Khởi tạo heap vector
                heap = vec![0u8; length]; // Khởi tạo vector kích thước length
                &mut heap[..] // Dùng heap buffer
            }; // Kết thúc chọn bộ đệm tạm

            unsafe { // Khối không an toàn sao chép payload vào bộ đệm tạm temp
                if start + length <= self.capacity { // Nếu payload đọc nằm liên tục
                    copy_nonoverlapping(self.pointer.add(start), temp.as_mut_ptr(), length); // Đọc 1 khối liên tục vào temp
                } else { // Nếu payload đọc bị xoay vòng qua biên capacity
                    let first = self.capacity - start; // Số byte đọc ở đuôi (từ đơn)
                    copy_nonoverlapping(self.pointer.add(start), temp.as_mut_ptr(), first); // Đọc phần 1 ở đuôi vào temp
                    copy_nonoverlapping(self.pointer, temp.as_mut_ptr().add(first), length - first); // Đọc phần 2 ở đầu vào temp
                } // Kết thúc điều kiện đọc payload
            } // Kết thúc khối unsafe

            let next = head.wrapping_add(total); // Chỉ số head mới sau khi đọc đúng 1 gói (từ đơn)
            match self.head.compare_exchange_weak( // Thử cập nhật chỉ số head qua CAS
                head, // Giá trị head cũ
                next, // Giá trị head mới
                Ordering::SeqCst, // Thứ tự thành công
                Ordering::Acquire, // Thứ tự thất bại
            ) { // Xử lý kết quả CAS
                Ok(_) => { // CAS thành công -> Sao chép từ temp ra target của caller và thoát vòng lặp
                    target[..length].copy_from_slice(temp); // Sao chép an toàn từ temp ra target
                    break; // Thoát vòng lặp CAS thành công
                } // Kết thúc Ok CAS
                Err(actual) => head = actual, // Thất bại do tranh chấp CAS -> Cập nhật head thực tế và thử lại
            } // Kết thúc match CAS
        } // Kết thúc vòng lặp CAS
        Ok(()) // Trả về thành công Ok
    } // Kết thúc phương thức pull
} // Kết thúc impl Storable for Buffer

impl Drop for Buffer { // Triển khai trait Drop tự động dọn dẹp cho Buffer
    fn drop(&mut self) { // Phương thức drop khi đối tượng hết vòng đời
        self.free(); // Gọi hàm free giải phóng tài nguyên bộ nhớ đệm
    } // Kết thúc phương thức drop
} // Kết thúc impl Drop for Buffer

impl std::fmt::Debug for Buffer { // Triển khai trait Debug cho Buffer với 100% chú thích tiếng Việt
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { // Phương thức định dạng Debug fmt
        f.debug_struct("Buffer") // Khởi tạo builder cấu trúc debug cho Buffer
            .field("pointer", &self.pointer) // Định dạng trường con trỏ pointer
            .field("bytes", &self.bytes) // Định dạng trường kích thước bytes
            .field("capacity", &self.capacity) // Định dạng trường dung lượng capacity
            .field("aligned", &self.aligned) // Định dạng trường cờ căn lề aligned
            .field("device", &self.device) // Định dạng trường cờ thiết bị device
            .field("shared", &self.shared) // Định dạng trường cờ chia sẻ shared
            .finish() // Hoàn tất xây dựng định dạng debug
    } // Kết thúc phương thức fmt
} // Kết thúc impl Debug for Buffer

unsafe impl Send for Buffer {} // Đánh giá Send an toàn cho việc truyền giữa các luồng
unsafe impl Sync for Buffer {} // Đánh giá Sync an toàn cho việc tham chiếu đồng thời giữa các luồng

#[cfg(test)] // Chỉ cấu hình biên dịch khi chạy kiểm thử unit test
mod tests { // Module kiểm thử unit tests cho buffer
    use super::*; // Nhập tất cả thành phần từ module cha

    #[test] // Đánh dấu hàm kiểm thử dung lượng lũy thừa 2
    fn test_allocate_capacity_power_of_two() { // Hàm test cấp phát dung lượng
        let buffer = Buffer::allocate(1, false).unwrap(); // Cấp phát 1 byte -> Dung lượng tối thiểu 64
        assert_eq!(buffer.capacity(), 64); // Kiểm tra capacity bằng 64

        let buffer = Buffer::allocate(63, false).unwrap(); // Cấp phát 63 bytes -> Dung lượng 64
        assert_eq!(buffer.capacity(), 64); // Kiểm tra capacity bằng 64

        let buffer = Buffer::allocate(64, false).unwrap(); // Cấp phát 64 bytes -> Dung lượng 64
        assert_eq!(buffer.capacity(), 64); // Kiểm tra capacity bằng 64

        let buffer = Buffer::allocate(65, false).unwrap(); // Cấp phát 65 bytes -> Dung lượng lũy thừa 2 tiếp theo 128
        assert_eq!(buffer.capacity(), 128); // Kiểm tra capacity bằng 128

        let buffer = Buffer::allocate(100, false).unwrap(); // Cấp phát 100 bytes -> Dung lượng 128
        assert_eq!(buffer.capacity(), 128); // Kiểm tra capacity bằng 128

        assert!(matches!(Buffer::allocate(0, false), Err(Status::Fault))); // Cấp phát 0 byte -> Trả về Fault
        assert!(matches!(Buffer::allocate(usize::MAX, false), Err(Status::Fault))); // Tràn số usize::MAX -> Trả về Fault
    } // Kết thúc hàm test_allocate_capacity_power_of_two

    #[test] // Đánh dấu hàm kiểm thử pull đọc payload an toàn trước CAS
    fn test_pull_payload_reading_before_cas() { // Hàm test đọc pull an toàn
        let buffer = Buffer::allocate(128, false).unwrap(); // Cấp phát buffer dung lượng 128
        let payload = [42u8; 32]; // Tạo dữ liệu mẫu 32 bytes
        assert!(buffer.push(&payload).is_ok()); // Đẩy dữ liệu vào buffer thành công

        let mut target = [0u8; 32]; // Tạo mảng đích 32 bytes
        assert!(buffer.pull(&mut target).is_ok()); // Rút dữ liệu ra target thành công
        assert_eq!(target, payload); // Kiểm tra dữ liệu rút ra khớp 100% với dữ liệu đẩy vào
    } // Kết thúc hàm test_pull_payload_reading_before_cas
} // Kết thúc module tests

