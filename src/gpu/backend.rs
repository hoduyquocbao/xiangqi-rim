// ============================================================================
// XIANGTI ENGINE: NỀN TẢNG TÍNH TOÁN GPU (BACKEND)
// ============================================================================
// Định nghĩa các nền tảng tính toán có thể sử dụng (Metal, Opencl, Wgpu, Cpu).
// Tự động nhận diện phần cứng bằng kỹ thuật C-ABI FFI Probing linh hoạt.
// Tuân thủ 100% định danh từ đơn tiếng Anh và 100% chú thích tiếng Việt.
// ============================================================================

/// Enum `Backend`: Định nghĩa kiểu biểu diễn các loại phần cứng tính toán GPU và CPU.
#[repr(u8)] // Định dạng đại diện bộ nhớ 1 byte u8 tương thích FFI
#[derive(Debug, Clone, Copy, PartialEq, Eq)] // Tự động derive các trait cơ bản
pub enum Backend { // Định nghĩa enum Backend với các biến thể phần cứng
    Metal = 0, // Nền tảng Apple Metal Native trên macOS
    Opencl = 1, // Nền tảng OpenCL cross-platform dự phòng thứ nhất
    Wgpu = 2, // Nền tảng WebGPU Compute Shaders dự phòng thứ hai
    Cpu = 3, // Nền tảng CPU SIMD dự phòng cuối cùng
} // Kết thúc định nghĩa enum Backend

impl Backend { // Khối triển khai các phương thức hỗ trợ cho enum Backend
    /// Phương thức `detect`: Tự động nhận diện phần cứng có sẵn qua FFI Probe Cascade.
    #[inline(always)] // Chỉ thị trình biên dịch inline hàm hot path
    pub fn detect() -> Self { // Định nghĩa hàm khởi tạo tự động phát hiện backend
        #[cfg(target_os = "macos")] // Khối phát hiện cho macOS
        { // Bắt đầu khối macOS
            if Self::probe("/System/Library/Frameworks/Metal.framework/Metal") { // Thử Metal framework
                return Self::Metal; // Trả về Metal nếu phát hiện thành công
            } // Kết thúc thử Metal
            if Self::probe("/System/Library/Frameworks/Metal.framework/Versions/Current/Metal") { // Thử vị trí Metal thứ hai
                return Self::Metal; // Trả về Metal
            } // Kết thúc vị trí hai
        } // Kết thúc khối macOS

        #[cfg(unix)] // Khối phát hiện cho môi trường Unix/Linux
        { // Bắt đầu khối Unix
            if Self::probe("/System/Library/Frameworks/OpenCL.framework/OpenCL") // Thử OpenCL macOS
                || Self::probe("libOpenCL.so") // Thử OpenCL Linux .so
                || Self::probe("libOpenCL.so.1") // Thử OpenCL Linux version 1
            { // Nếu khớp OpenCL
                return Self::Opencl; // Trả về Opencl
            } // Kết thúc thử OpenCL

            if Self::probe("libwgpu_native.so") || Self::probe("libwgpu.so") { // Thử driver Wgpu
                return Self::Wgpu; // Trả về Wgpu
            } // Kết thúc thử Wgpu
        } // Kết thúc khối Unix

        Self::Cpu // Mặc định hạ cấp về CPU SIMD fallback
    } // Kết thúc hàm detect

    /// Phương thức phụ `probe`: Thử nghiệm nạp động thư viện qua C-ABI dlopen.
    fn probe(path: &str) -> bool { // Hàm probe kiểm tra sự tồn tại của thư viện động
        #[cfg(unix)] // Chỉ hỗ trợ trên Unix/macOS
        { // Khối xử lý Unix
            let mut bytes = path.as_bytes().to_vec(); // Chuyển chuỗi path thành mảng byte
            bytes.push(0); // Thêm ký tự kết thúc chuỗi C null-terminator
            unsafe { // Khối FFI không an toàn
                let handle = dlopen(bytes.as_ptr() as *const i8, 1); // Gọi dlopen với cờ RTLD_LAZY (1)
                if !handle.is_null() { // Nếu con trỏ trả về khác null
                    dlclose(handle); // Đóng handle thư viện động
                    return true; // Trả về true xác nhận thư viện tồn tại
                } // Kết thúc kiểm tra handle
            } // Kết thúc khối unsafe
        } // Kết thúc khối Unix
        false // Trả về false nếu không nạp được
    } // Kết thúc hàm probe

    /// Phương thức `name`: Trả về tên hiển thị tĩnh dạng chuỗi của backend.
    #[inline(always)] // Inline phương thức lấy tên hiển thị
    pub fn name(&self) -> &'static str { // Trả về tham chiếu chuỗi tĩnh
        match self { // Khớp mẫu giá trị của biến thể enum
            Self::Metal => "Metal", // Chuỗi tên nền tảng Metal
            Self::Opencl => "OpenCL", // Chuỗi tên nền tảng OpenCL
            Self::Wgpu => "WGPU", // Chuỗi tên nền tảng WebGPU
            Self::Cpu => "CPU", // Chuỗi tên nền tảng CPU
        } // Kết thúc biểu thức match
    } // Kết thúc phương thức name

    /// Phương thức `valid`: Kiểm tra xem backend có phải là GPU phần cứng hay không.
    #[inline(always)] // Inline phương thức kiểm tra tính hợp lệ phần cứng
    pub fn valid(&self) -> bool { // Trả về giá trị boolean true nếu là GPU
        !matches!(self, Self::Cpu) // Trả về true nếu không phải là CPU fallback
    } // Kết thúc phương thức valid

    /// Phương thức `hardware`: Kiểm tra xem backend có thuộc nhóm card đồ họa phần cứng không.
    #[inline(always)] // Inline phương thức hardware
    pub fn hardware(&self) -> bool { // Trả về true cho các GPU phần cứng
        matches!(self, Self::Metal | Self::Opencl | Self::Wgpu) // Khớp nhóm phần cứng GPU
    } // Kết thúc phương thức hardware

    /// Phương thức `rank`: Trả về thứ tự ưu tiên của backend dạng số nguyên u8.
    #[inline(always)] // Inline phương thức lấy thứ tự
    pub fn rank(&self) -> u8 { // Trả về giá trị u8
        *self as u8 // Ép kiểu enum thành u8
    } // Kết thúc phương thức rank

    /// Phương thức `speed`: Trả về chỉ số hiệu năng tương đối (relative speed rating).
    #[inline(always)] // Inline phương thức đánh giá điểm hiệu năng
    pub fn speed(&self) -> usize { // Trả về giá trị điểm số hiệu năng kiểu usize
        match self { // Khớp mẫu điểm số tương ứng từng backend
            Self::Metal => 100, // Metal Native đạt 100% hiệu năng
            Self::Opencl => 80, // OpenCL đạt 80% hiệu năng tương đối
            Self::Wgpu => 70, // WebGPU compute shaders đạt 70% hiệu năng
            Self::Cpu => 10, // CPU SIMD fallback đạt 10% hiệu năng
        } // Kết thúc biểu thức match điểm số
    } // Kết thúc phương thức speed
} // Kết thúc khối impl Backend

#[cfg(unix)] // Định nghĩa FFI C-ABI bên ngoài cho hệ điều hành Unix/macOS
extern "C" { // Khối FFI extern C
    fn dlopen(path: *const i8, mode: i32) -> *mut std::ffi::c_void; // Chữ ký hàm C dlopen
    fn dlclose(handle: *mut std::ffi::c_void) -> i32; // Chữ ký hàm C dlclose
} // Kết thúc khối extern C
