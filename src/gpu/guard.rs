// ============================================================================
// XIANGTI ENGINE: BỘ GIÁM SÁT DUNG LƯỢNG VRAM (VRAM GUARD)
// ============================================================================
// Bảo vệ VRAM 512MB chống tràn bộ nhớ (OOM Protection) với trần an toàn 409.6MB.
// Tích hợp vòng lặp CAS chống underflow nguyên tử khi giải phóng bộ nhớ đa luồng.
// Tuân thủ 100% định danh từ đơn tiếng Anh, căn lề 64-byte và 100% chú thích tiếng Việt.
// ============================================================================

use std::sync::atomic::{AtomicUsize, Ordering}; // Nhập kiểu nguyên tử AtomicUsize
use super::status::Status; // Nhập kiểu enum Status từ module status

/// Trait `Validatable`: Quy ước khả năng xác minh giới hạn an toàn của bộ nhớ VRAM.
pub trait Validatable { // Định nghĩa trait Validatable
    /// Phương thức `validate`: Xác minh xem số byte cấp phát có nằm trong hạn mức an toàn không.
    fn validate(&self, bytes: usize) -> Status; // Định nghĩa chữ ký hàm validate
} // Kết thúc trait Validatable

/// Struct `Guard`: Quản lý và theo dõi nguyên tử dung lượng VRAM đã cấp phát.
#[repr(C, align(64))] // Căn lề 64-byte chống False Sharing trên CPU Cache Line
pub struct Guard { // Định nghĩa struct Guard
    /// Giới hạn VRAM tối đa tuyệt đối tính bằng bytes (512MB = 536,870,912 bytes)
    limit: usize, // Trường giới hạn tối đa tuyệt đối
    /// Trần an toàn VRAM tính bằng bytes (80% của 512MB = 409.6MB = 429,496,729 bytes)
    ceiling: usize, // Trường trần an toàn chống OOM
    /// Dung lượng VRAM đang được cấp phát hiện tại (AtomicUsize)
    allocated: AtomicUsize, // Biến nguyên tử theo dõi byte đang sử dụng
    /// Dung lượng VRAM cấp phát đỉnh cao nhất đã ghi nhận (AtomicUsize)
    peak: AtomicUsize, // Biến nguyên tử ghi lại đỉnh sử dụng
    /// Tổng số lượng các khối bộ đệm đang được giữ (AtomicUsize)
    count: AtomicUsize, // Biến nguyên tử theo dõi số lượng khối cấp phát
    /// Kích thước tối đa của một chunk cấp phát đơn lẻ (64MB = 67,108,864 bytes)
    chunk: usize, // Trường kích thước chunk phân đoạn dữ liệu
    /// Mảng đệm 16 byte đảm bảo kích thước struct tròn đúng 64 bytes
    pad: [u8; 16], // Trường đệm căn lề cache line 64-byte
} // Kết thúc struct Guard

impl Guard { // Khối triển khai phương thức cho struct Guard
    /// Hằng số `LIMIT`: 512 MB tính bằng bytes ($512 \times 1024 \times 1024 = 536,870,912$)
    pub const LIMIT: usize = 536_870_912; // 512 MB tuyệt đối
    /// Hằng số `CEILING`: 409.6 MB tính bằng bytes ($409.6 \times 1024 \times 1024 = 429,496,729$)
    pub const CEILING: usize = 429_496_729; // 80% trần an toàn
    /// Hằng số `CHUNK`: 64 MB tính bằng bytes ($64 \times 1024 \times 1024 = 67,108,864$)
    pub const CHUNK: usize = 67_108_864; // Kích thước phân đoạn 64MB

    /// Khởi tạo Guard mới với giới hạn 512MB và trần an toàn 409.6MB.
    #[inline(always)] // Inline hàm khởi tạo Guard
    pub const fn new() -> Self { // Định nghĩa hàm new hằng số const
        Self { // Trả về bản thể struct Guard
            limit: Self::LIMIT, // Gán giới hạn 512MB
            ceiling: Self::CEILING, // Gán trần an toàn 409.6MB
            allocated: AtomicUsize::new(0), // Khởi tạo số byte allocated = 0
            peak: AtomicUsize::new(0), // Khởi tạo đỉnh peak = 0
            count: AtomicUsize::new(0), // Khởi tạo số lượng count = 0
            chunk: Self::CHUNK, // Gán kích thước phân đoạn 64MB
            pad: [0u8; 16], // Khởi tạo mảng đệm 16 byte zero
        } // Kết thúc khởi tạo struct
    } // Kết thúc hàm new

    /// Kiểm tra và đặt trước số byte VRAM cần cấp phát (Reserve VRAM).
    pub fn reserve(&self, bytes: usize) -> Result<usize, Status> { // Hàm reserve kiểm tra trần an toàn
        if bytes == 0 { // Nếu số byte cần đặt bằng 0
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra 0
        let current = self.allocated.load(Ordering::Relaxed); // Đọc số byte đang dùng hiện tại
        if current.saturating_add(bytes) > self.ceiling { // Kiểm tra nếu vượt quá trần an toàn 409.6MB
            return Err(Status::Exhausted); // Vượt trần -> Trả về lỗi Exhausted (chống OOM)
        } // Kết thúc điều kiện kiểm tra trần
        let next = self.allocated.fetch_add(bytes, Ordering::SeqCst) + bytes; // Tăng dung lượng allocated nguyên tử
        if next > self.ceiling { // Rà soát lại điều kiện đua luồng (race condition) sau khi tăng
            self.allocated.fetch_sub(bytes, Ordering::SeqCst); // Hoàn tác lại dung lượng đã tăng
            return Err(Status::Exhausted); // Trả về lỗi Exhausted
        } // Kết thúc rà soát lại đua luồng
        self.count.fetch_add(1, Ordering::Relaxed); // Tăng số lượng khối đệm thêm 1
        let mut max = self.peak.load(Ordering::Relaxed); // Đọc đỉnh dung lượng hiện tại
        while next > max { // Vòng lặp cập nhật đỉnh dung lượng cao nhất (CAS loop)
            match self.peak.compare_exchange_weak(max, next, Ordering::SeqCst, Ordering::Relaxed) { // Cập nhật nguyên tử peak
                Ok(_) => break, // Cập nhật thành công -> Thoát vòng lặp
                Err(actual) => max = actual, // Thất bại -> Cập nhật lại giá trị thực tế và thử lại
            } // Kết thúc match compare_exchange
        } // Kết thúc vòng lặp CAS
        Ok(next) // Trả về kết quả thành công chứa tổng số byte đã dùng sau cấp phát
    } // Kết thúc hàm reserve

    /// Giải phóng số byte VRAM đã dùng an toàn chống underflow qua vòng lặp CAS.
    pub fn release(&self, bytes: usize) { // Hàm release trả lại bộ nhớ
        if bytes == 0 { // Nếu số byte cần giải phóng bằng 0
            return; // Thoát sớm không làm gì
        } // Kết thúc kiểm tra 0

        // Giảm dung lượng allocated nguyên tử an toàn bằng vòng lặp CAS
        let mut current = self.allocated.load(Ordering::Relaxed); // Đọc dung lượng allocated hiện tại
        loop { // Vòng lặp CAS giảm allocated
            if current == 0 { // Nếu dung lượng bằng 0
                break; // Thoát vòng lặp không làm gì
            } // Kết thúc kiểm tra 0
            let free = if bytes > current { current } else { bytes }; // Tính số byte thực tế có thể trừ
            let next = current - free; // Tính giá trị allocated mới sau khi trừ
            match self.allocated.compare_exchange_weak( // Thử cập nhật CAS
                current, // Giá trị kỳ vọng
                next, // Giá trị mới
                Ordering::SeqCst, // Thứ tự bộ nhớ thành công
                Ordering::Relaxed, // Thứ tự bộ nhớ thất bại
            ) { // Xử lý kết quả CAS
                Ok(_) => break, // Cập nhật thành công -> Thoát vòng lặp
                Err(actual) => current = actual, // Cập nhật giá trị thực tế mới và thử lại
            } // Kết thúc match CAS
        } // Kết thúc vòng lặp CAS allocated

        // Giảm số lượng khối count nguyên tử an toàn bằng vòng lặp CAS
        let mut count = self.count.load(Ordering::Relaxed); // Đọc số lượng khối count hiện tại
        loop { // Vòng lặp CAS giảm count
            if count == 0 { // Nếu số lượng khối bằng 0
                break; // Thoát vòng lặp
            } // Kết thúc kiểm tra 0
            let next = count - 1; // Giảm số lượng khối đi 1
            match self.count.compare_exchange_weak( // Thử cập nhật CAS
                count, // Giá trị kỳ vọng
                next, // Giá trị mới
                Ordering::SeqCst, // Thứ tự bộ nhớ thành công
                Ordering::Relaxed, // Thứ tự bộ nhớ thất bại
            ) { // Xử lý kết quả CAS
                Ok(_) => break, // Thành công -> Thoát vòng lặp
                Err(actual) => count = actual, // Cập nhật giá trị thực tế mới và thử lại
            } // Kết thúc match CAS
        } // Kết thúc vòng lặp CAS count
    } // Kết thúc hàm release

    /// Trả về số byte VRAM đang được cấp phát hiện tại.
    #[inline(always)] // Inline hàm đọc allocated
    pub fn allocated(&self) -> usize { // Hàm allocated trả về usize
        self.allocated.load(Ordering::Relaxed) // Đọc nguyên tử dung lượng allocated
    } // Kết thúc hàm allocated

    /// Trả về đỉnh dung lượng VRAM cao nhất đã cấp phát.
    #[inline(always)] // Inline hàm đọc peak
    pub fn peak(&self) -> usize { // Hàm peak trả về usize
        self.peak.load(Ordering::Relaxed) // Đọc nguyên tử đỉnh peak
    } // Kết thúc hàm peak

    /// Trả về tổng số lượng khối bộ đệm VRAM đang được giữ.
    #[inline(always)] // Inline hàm đọc count
    pub fn count(&self) -> usize { // Hàm count trả về usize
        self.count.load(Ordering::Relaxed) // Đọc nguyên tử số khối count
    } // Kết thúc hàm count

    /// Trả về giới hạn VRAM tối đa tuyệt đối (512MB).
    #[inline(always)] // Inline hàm đọc limit
    pub fn limit(&self) -> usize { // Hàm limit trả về usize
        self.limit // Trả về giá trị limit 512MB
    } // Kết thúc hàm limit

    /// Trả về trần an toàn VRAM (409.6MB).
    #[inline(always)] // Inline hàm đọc ceiling
    pub fn ceiling(&self) -> usize { // Hàm ceiling trả về usize
        self.ceiling // Trả về giá trị ceiling 409.6MB
    } // Kết thúc hàm ceiling

    /// Tính toán số lượng phân đoạn chunking 64MB cho một dung lượng byte cho trước.
    #[inline(always)] // Inline hàm chunks
    pub fn chunks(&self, bytes: usize) -> usize { // Hàm chunks tính số lượng khối 64MB
        if bytes == 0 { // Nếu dung lượng bằng 0
            0 // Trả về 0 khối
        } else { // Nếu dung lượng lớn hơn 0
            (bytes + self.chunk - 1) / self.chunk // Phép chia làm tròn lên theo kích thước chunk 64MB
        } // Kết thúc điều kiện kiểm tra 0
    } // Kết thúc hàm chunks

    /// Trả về trạng thái hiện tại của VRAM Guard.
    pub fn status(&self) -> Status { // Hàm status kiểm tra trạng thái Guard
        let current = self.allocated.load(Ordering::Relaxed); // Đọc số byte allocated
        if current >= self.ceiling { // Nếu dung lượng đạt hoặc vượt trần an toàn 409.6MB
            Status::Exhausted // Trả về trạng thái cạn kệt VRAM Exhausted
        } else if current > 0 { // Nếu dung lượng lớn hơn 0 và dưới trần an toàn
            Status::Active // Trả về trạng thái Active đang hoạt động
        } else { // Nếu chưa cấp phát bộ nhớ nào
            Status::Ready // Trả về trạng thái Ready sẵn sàng
        } // Kết thúc điều kiện phân loại trạng thái
    } // Kết thúc hàm status

    /// Xóa toàn bộ thống kê cấp phát và đặt lại bộ đếm về 0.
    pub fn wipe(&self) { // Hàm wipe reset bộ đếm Guard
        self.allocated.store(0, Ordering::SeqCst); // Đặt số byte allocated về 0
        self.peak.store(0, Ordering::SeqCst); // Đặt đỉnh peak về 0
        self.count.store(0, Ordering::SeqCst); // Đặt số khối count về 0
    } // Kết thúc hàm wipe
} // Kết thúc khối impl Guard

impl Validatable for Guard { // Triển khai trait Validatable cho Guard
    fn validate(&self, bytes: usize) -> Status { // Hàm validate kiểm tra dung lượng
        let current = self.allocated.load(Ordering::Relaxed); // Đọc số byte đang dùng
        let target = current.saturating_add(bytes); // Tính tổng dung lượng dự kiến
        if target <= self.ceiling { // Nếu nhỏ hơn hoặc bằng trần an toàn 409.6MB
            Status::Ready // Trả về trạng thái Ready hợp lệ
        } else if target <= self.limit { // Nếu vượt trần an toàn nhưng dưới 512MB
            Status::Full // Trả về trạng thái Full cần chia nhỏ lô
        } else { // Nếu vượt quá 512MB tuyệt đối
            Status::Fail // Trả về trạng thái Fail báo tràn cứng
        } // Kết thúc điều kiện phân loại
    } // Kết thúc hàm validate
} // Kết thúc impl Validatable for Guard

impl Default for Guard { // Triển khai trait Default cho Guard
    fn default() -> Self { // Hàm default tạo bản thể mặc định
        Self::new() // Gọi hàm new khởi tạo
    } // Kết thúc hàm default
} // Kết thúc impl Default for Guard
