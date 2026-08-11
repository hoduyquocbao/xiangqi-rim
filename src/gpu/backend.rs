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
    /// Phương thức `detect`: Tự động nhận diện phần cứng GPU thực tế thông qua wgpu Instance.
    pub fn detect() -> Self { // Định nghĩa hàm khởi tạo tự động phát hiện backend phần cứng GPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor { // Khởi tạo wgpu Instance mới
            backends: wgpu::Backends::all(), // Cho phép tất cả các backend phần cứng GPU (Metal, Vulkan, DX12, Gl)
            ..Default::default() // Sử dụng các giá trị mặc định cho các cấu hình khác
        }); // Kết thúc khởi tạo Instance
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions { // Yêu cầu Adapter GPU bất đồng bộ bằng pollster
            power_preference: wgpu::PowerPreference::HighPerformance, // Ưu tiên card đồ họa hiệu năng cao dGPU/iGPU
            compatible_surface: None, // Không yêu cầu bề mặt hiển thị GUI surface (dành cho Compute)
            force_fallback_adapter: false, // Không ép buộc sử dụng CPU fallback adapter
        })); // Kết thúc yêu cầu Adapter
        if let Some(adapter) = adapter { // Nếu phát hiện được phần cứng GPU Adapter hợp lệ
            let info = adapter.get_info(); // Lấy thông tin chi tiết về phần cứng GPU Adapter
            match info.backend { // Kiểm tra loại backend thực tế của GPU Adapter
                wgpu::Backend::Metal => Self::Metal, // Metal Native trên Apple Silicon / macOS
                wgpu::Backend::Vulkan => Self::Opencl, // Vulkan / OpenCL trên Linux / Windows / Android
                wgpu::Backend::Dx12 => Self::Wgpu, // DirectX12 trên Windows
                wgpu::Backend::Gl => Self::Wgpu, // OpenGL / WebGPU fallback
                _ => Self::Wgpu, // Mặc định trả về Wgpu
            } // Kết thúc match backend
        } else { // Nếu không tìm thấy GPU Adapter hợp lệ
            Self::Cpu // Hạ cấp an toàn về CPU SIMD fallback
        } // Kết thúc kiểm tra adapter
    } // Kết thúc hàm detect

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
