// ============================================================================
// XIANGTI ENGINE: THIẾT BỊ ADAPTER GPU VÀ BỘ ĐÁNH GIÁ TỰ CHỦ (DEVICE)
// ============================================================================
// Đại diện hợp nhất cho GPU Adapter (Metal Native / OpenCL / WGPU / CPU Fallback).
// Quản lý vòng đời cấp phát VRAM an toàn thông qua VRAM Guard 512MB limit / 409.6MB ceiling.
// Tích hợp Mạng Nơ-ron NNUE 32MB VRAM Storage Buffer nạp trực tiếp vào Compute Shader.
// Tuân thủ 100% định danh từ đơn tiếng Anh, căn lề 64-byte và 100% chú thích tiếng Việt.
// ============================================================================

use std::sync::atomic::AtomicBool; // Nhập cờ nguyên tử AtomicBool cho Double-Buffering
use std::sync::Arc; // Nhập kiểu Arc quản lý con trỏ đếm tham chiếu dùng chung
use wgpu::util::DeviceExt; // Nhập DeviceExt mở rộng hàm create_buffer_init
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

/// Struct `GpuContext`: Lưu trữ các đối tượng hạ tầng GPU phần cứng thực tế của wgpu.
pub struct GpuContext { // Định nghĩa struct GpuContext
    /// Thiết bị điều khiển GPU Device phần cứng
    pub device: wgpu::Device, // Trường thiết bị device
    /// Hàng đợi gửi lệnh GPU Queue
    pub queue: wgpu::Queue, // Trường hàng đợi queue
    /// Pipeline thực thi Compute Shader WGSL
    pub pipeline: wgpu::ComputePipeline, // Trường đường ống pipeline
    /// Bind Group Layout của Storage Buffer
    pub layout: wgpu::BindGroupLayout, // Trường bố cục layout
    /// Storage Buffer VRAM cấp phát tĩnh sẵn (2MB = 16,384 samples x 128 bytes)
    pub storage_buffer: wgpu::Buffer, // Trường bộ đệm Storage Buffer tĩnh
    /// Staging Buffer A Host RAM cấp phát tĩnh sẵn (2MB = 16,384 samples x 128 bytes) cho Ping-Pong Double Buffering
    pub staging_buffer_a: wgpu::Buffer, // Trường bộ đệm Staging Buffer A
    /// Staging Buffer B Host RAM cấp phát tĩnh sẵn (2MB = 16,384 samples x 128 bytes) cho Ping-Pong Double Buffering
    pub staging_buffer_b: wgpu::Buffer, // Trường bộ đệm Staging Buffer B
    /// Storage Buffer nén 64KB VRAM (16,384 x 4 bytes i32)
    pub score_storage: wgpu::Buffer, // Trường bộ đệm Score Storage Buffer nén 64KB
    /// Staging Buffer nén A 64KB Host RAM (16,384 x 4 bytes i32)
    pub score_staging_a: wgpu::Buffer, // Trường bộ đệm Score Staging A nén 64KB
    /// Staging Buffer nén B 64KB Host RAM (16,384 x 4 bytes i32)
    pub score_staging_b: wgpu::Buffer, // Trường bộ đệm Score Staging B nén 64KB
    /// Storage Buffer chứa trọng số Mạng Nơ-ron NNUE 33.57MB VRAM (Binding 2)
    pub weight_buffer: wgpu::Buffer, // Trường bộ đệm Weight Buffer NNUE 33.57MB
    /// BindGroup cấp phát tĩnh sẵn liên kết Storage Buffer với Shader
    pub bind_group: wgpu::BindGroup, // Trường nhóm liên kết BindGroup tĩnh
    /// Cờ nguyên tử điều khiển xoay vòng Ping-Pong Double Buffering (false -> A, true -> B)
    pub ping_pong: AtomicBool, // Trường cờ nguyên tử ping_pong
    /// Tên hiển thị của thiết bị GPU Adapter phần cứng thực tế
    pub name: String, // Trường tên hiển thị name
} // Kết thúc struct GpuContext

/// Struct `Device`: Thiết bị GPU Adapter tích hợp backend phần cứng và Guard (128 bytes total).
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
    /// Con trỏ dùng chung chứa ngữ cảnh phần cứng GPU thực tế qua wgpu (8 bytes, offset 72..80)
    context: Option<Arc<GpuContext>>, // Trường ngữ cảnh context
    /// Mảng đệm 48 byte đảm bảo kích thước toàn bộ struct Device đúng 128 bytes (48 bytes, offset 80..128)
    extra: [u8; 48], // Trường đệm căn lề tròn 128 bytes
} // Kết thúc struct Device

impl Device { // Khối triển khai các phương thức cho Device
    /// Khởi tạo thiết bị GPU Adapter mới, tự động phát hiện backend và kích hoạt Guard.
    pub fn init() -> Self { // Hàm khởi tạo init
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor { // Khởi tạo wgpu Instance mới
            backends: wgpu::Backends::all(), // Hỗ trợ tất cả các GPU Backend (Metal/Vulkan/DX12)
            flags: wgpu::InstanceFlags::ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER, // Cho phép headless Vulkan GPU trên Linux Colab
            ..Default::default() // Sử dụng mặc định cho các trường còn lại
        }); // Kết thúc khởi tạo Instance

        // Liệt kê tất cả các Adapter và ưu tiên chọn card phần cứng thực sự, loại bỏ 100% Cpu/llvmpipe
        let mut adapter = None;
        for a in instance.enumerate_adapters(wgpu::Backends::all()) {
            let info = a.get_info();
            let name_lc = info.name.to_lowercase();
            if info.device_type != wgpu::DeviceType::Cpu 
               && !name_lc.contains("llvmpipe") 
               && !name_lc.contains("softpipe") 
               && !name_lc.contains("swrast") {
                adapter = Some(a);
                break;
            }
        }

        if adapter.is_none() { // Nếu chưa tìm được qua enumeration -> Dùng request_adapter fallback
            adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            }));
        }

        let mut backend = Backend::Cpu; // Khởi tạo mặc định CPU backend
        let mut context = None; // Khởi tạo mặc định context None

        // 1. Thử nhận diện qua opencl3 Native GPU trên Linux / Colab CUDA Driver Container
        if let Ok(platforms) = opencl3::platform::get_platforms() {
            for platform in platforms {
                if let Ok(devices) = platform.get_devices(opencl3::device::CL_DEVICE_TYPE_GPU) {
                    if !devices.is_empty() {
                        if let Ok(dev_name) = opencl3::device::Device::new(devices[0]).name() {
                            if !dev_name.to_lowercase().contains("llvmpipe") {
                                backend = Backend::Opencl;
                                break;
                            }
                        }
                    }
                }
            }
        }

        if let Some(adapter) = adapter { // Nếu lấy được GPU Adapter phần cứng thành công
            let info = adapter.get_info(); // Lấy thông tin GPU Adapter
            if info.device_type != wgpu::DeviceType::Cpu && !info.name.to_lowercase().contains("llvmpipe") {
                backend = match info.backend { // Khớp mẫu loại backend thực tế
                    wgpu::Backend::Metal => Backend::Metal, // macOS Apple Metal Native
                    wgpu::Backend::Vulkan => Backend::Opencl, // Linux / Windows Vulkan
                    wgpu::Backend::Dx12 => Backend::Wgpu, // Windows DirectX12
                    _ => Backend::Wgpu, // Mặc định WebGPU/OpenGL
                }; // Kết thúc match backend
            }

            if let Ok((device, queue)) = pollster::block_on(adapter.request_device( // Khởi tạo Device và Queue từ GPU Adapter
                &wgpu::DeviceDescriptor { // Cấu hình tham số mô tả Device
                    label: Some("Xiangqi-RIM GPU Device"), // Nhãn tên thiết bị GPU
                    required_features: wgpu::Features::empty(), // Không yêu cầu feature đặc biệt
                    required_limits: wgpu::Limits::default(), // Sử dụng giới hạn VRAM/Limits mặc định
                    memory_hints: Default::default(), // Sử dụng mô tả bộ nhớ mặc định
                }, // Kết thúc cấu hình Descriptor
                None, // Không dùng đường dẫn vết trace
            )) { // Bắt đầu khối nếu khởi tạo Device thành công
                let shader_str = include_str!("shader.wgsl"); // Nạp chuỗi Compute Shader WGSL từ tệp nội bộ
                let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { // Tạo Module Shader WGSL
                    label: Some("Xiangqi-RIM Compute Shader"), // Nhãn Module Shader
                    source: wgpu::ShaderSource::Wgsl(shader_str.into()), // Nạp mã nguồn WGSL
                }); // Kết thúc tạo Module Shader

                let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { // Tạo BindGroupLayout cho Storage Buffer
                    label: Some("Xiangqi-RIM BindGroupLayout"), // Nhãn Layout
                    entries: &[
                        wgpu::BindGroupLayoutEntry { // Entry 0: BatchBuffer Storage (2MB)
                            binding: 0, // Ô liên kết binding 0
                            visibility: wgpu::ShaderStages::COMPUTE, // Cho phép truy cập từ Compute Shader
                            ty: wgpu::BindingType::Buffer { // Kiểu tài nguyên là Buffer
                                ty: wgpu::BufferBindingType::Storage { read_only: false }, // Kiểu Storage Buffer đọc/ghi
                                has_dynamic_offset: false, // Không dùng offset động
                                min_binding_size: None, // Không giới hạn kích thước tối thiểu
                            }, // Kết thúc kiểu BindingType
                            count: None, // Không dùng mảng tài nguyên
                        },
                        wgpu::BindGroupLayoutEntry { // Entry 1: ScoreBuffer Storage nén 64KB
                            binding: 1, // Ô liên kết binding 1
                            visibility: wgpu::ShaderStages::COMPUTE, // Cho phép truy cập từ Compute Shader
                            ty: wgpu::BindingType::Buffer { // Kiểu tài nguyên là Buffer
                                ty: wgpu::BufferBindingType::Storage { read_only: false }, // Kiểu Storage Buffer đọc/ghi
                                has_dynamic_offset: false, // Không dùng offset động
                                min_binding_size: None, // Không giới hạn kích thước tối thiểu
                            }, // Kết thúc kiểu BindingType
                            count: None, // Không dùng mảng tài nguyên
                        },
                        wgpu::BindGroupLayoutEntry { // Entry 2: WeightBuffer Storage NNUE 33.57MB
                            binding: 2, // Ô liên kết binding 2
                            visibility: wgpu::ShaderStages::COMPUTE, // Cho phép truy cập từ Compute Shader
                            ty: wgpu::BindingType::Buffer { // Kiểu tài nguyên là Buffer
                                ty: wgpu::BufferBindingType::Storage { read_only: true }, // Kiểu Storage Buffer chỉ đọc
                                has_dynamic_offset: false, // Không dùng offset động
                                min_binding_size: None, // Không giới hạn kích thước tối thiểu
                            }, // Kết thúc kiểu BindingType
                            count: None, // Không dùng mảng tài nguyên
                        },
                    ], // Kết thúc mảng entries
                }); // Kết thúc tạo BindGroupLayout

                let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { // Tạo PipelineLayout
                    label: Some("Xiangqi-RIM PipelineLayout"), // Nhãn Pipeline Layout
                    bind_group_layouts: &[&layout], // Liên kết mảng Layout
                    push_constant_ranges: &[], // Không dùng hằng số đẩy Push Constant
                }); // Kết thúc tạo PipelineLayout

                let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { // Tạo ComputePipeline thực thi
                    label: Some("Xiangqi-RIM ComputePipeline"), // Nhãn Compute Pipeline
                    layout: Some(&pipeline_layout), // Gán Pipeline Layout
                    module: &module, // Gán Shader Module
                    entry_point: Some("main"), // Tên hàm điểm vào main trong shader WGSL
                    compilation_options: Default::default(), // Tùy chọn biên dịch mặc định
                    cache: None, // Không dùng bộ đệm cache ngoài
                }); // Kết thúc tạo ComputePipeline

                // Pre-allocate 2MB Storage Buffer tĩnh (16,384 samples x 128 bytes)
                let max_bytes = 16384 * 128; // 2,097,152 bytes = 2MB
                let storage_buffer = device.create_buffer(&wgpu::BufferDescriptor { // Khởi tạo Storage Buffer tĩnh
                    label: Some("Xiangqi-RIM Static Storage Buffer"), // Nhãn bộ đệm Storage tĩnh
                    size: max_bytes as u64, // Kích thước 2MB
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST, // Cờ cho phép đọc/ghi Storage và sao chép
                    mapped_at_creation: false, // Không map ngay khi khởi tạo
                }); // Kết thúc tạo Storage Buffer tĩnh

                // Pre-allocate Staging Buffer A tĩnh (2MB) cho Ping-Pong Double Buffering
                let staging_buffer_a = device.create_buffer(&wgpu::BufferDescriptor { // Khởi tạo Staging Buffer A
                    label: Some("Xiangqi-RIM Static Staging Buffer A"), // Nhãn bộ đệm Staging A
                    size: max_bytes as u64, // Kích thước 2MB
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, // Cờ cho phép đọc map và làm đích sao chép
                    mapped_at_creation: false, // Không map ngay khi khởi tạo
                }); // Kết thúc tạo Staging Buffer A

                // Pre-allocate Staging Buffer B tĩnh (2MB) cho Ping-Pong Double Buffering
                let staging_buffer_b = device.create_buffer(&wgpu::BufferDescriptor { // Khởi tạo Staging Buffer B
                    label: Some("Xiangqi-RIM Static Staging Buffer B"), // Nhãn bộ đệm Staging B
                    size: max_bytes as u64, // Kích thước 2MB
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, // Cờ cho phép đọc map và làm đích sao chép
                    mapped_at_creation: false, // Không map ngay khi khởi tạo
                }); // Kết thúc tạo Staging Buffer B

                // Pre-allocate 64KB Score Storage Buffer nén (16,384 x 4 bytes i32)
                let max_score_bytes = 16384 * 4; // 65,536 bytes = 64KB
                let score_storage = device.create_buffer(&wgpu::BufferDescriptor { // Khởi tạo Score Storage Buffer
                    label: Some("Xiangqi-RIM Compact Score Storage Buffer"), // Nhãn Score Storage
                    size: max_score_bytes as u64, // Kích thước 64KB
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST, // Cờ Storage và Copy
                    mapped_at_creation: false, // Không map ngay khi tạo
                }); // Kết thúc tạo Score Storage

                // Pre-allocate 64KB Score Staging Buffer A nén (64KB)
                let score_staging_a = device.create_buffer(&wgpu::BufferDescriptor { // Khởi tạo Score Staging Buffer A
                    label: Some("Xiangqi-RIM Compact Score Staging A"), // Nhãn Score Staging A
                    size: max_score_bytes as u64, // Kích thước 64KB
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, // Cờ Map Read và Copy DST
                    mapped_at_creation: false, // Không map ngay khi tạo
                }); // Kết thúc tạo Score Staging A

                // Pre-allocate 64KB Score Staging Buffer B nén (64KB)
                let score_staging_b = device.create_buffer(&wgpu::BufferDescriptor { // Khởi tạo Score Staging Buffer B
                    label: Some("Xiangqi-RIM Compact Score Staging B"), // Nhãn Score Staging B
                    size: max_score_bytes as u64, // Kích thước 64KB
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, // Cờ Map Read và Copy DST
                    mapped_at_creation: false, // Không map ngay khi tạo
                }); // Kết thúc tạo Score Staging B

                // Nạp tệp trọng số XRNN v1 (33.57MB) từ đĩa
                let weight_bytes = std::fs::read("data/nnue_weights_gen6.bin").unwrap_or_else(|_| vec![0u8; 33571504]);
                let weight_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Xiangqi-RIM NNUE Weight Buffer 33.57MB"),
                    contents: &weight_bytes,
                    usage: wgpu::BufferUsages::STORAGE,
                });

                // Khởi tạo BindGroup tĩnh duy nhất liên kết cả BatchBuffer (0), ScoreBuffer (1), và WeightBuffer (2)
                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { // Tạo BindGroup tĩnh
                    label: Some("Xiangqi-RIM Static BindGroup"), // Nhãn BindGroup tĩnh
                    layout: &layout, // Gán BindGroupLayout
                    entries: &[
                        wgpu::BindGroupEntry { // Binding 0: Storage Buffer 2MB
                            binding: 0, // Binding 0
                            resource: storage_buffer.as_entire_binding(), // Liên kết với Storage Buffer tĩnh
                        },
                        wgpu::BindGroupEntry { // Binding 1: Score Storage Buffer nén 64KB
                            binding: 1, // Binding 1
                            resource: score_storage.as_entire_binding(), // Liên kết với Score Storage Buffer nén
                        },
                        wgpu::BindGroupEntry { // Binding 2: Weight Storage Buffer NNUE 33.57MB
                            binding: 2, // Binding 2
                            resource: weight_buffer.as_entire_binding(), // Liên kết với Weight Storage Buffer NNUE
                        },
                    ], // Kết thúc mảng entries
                }); // Kết thúc tạo BindGroup tĩnh

                let adapter_name = info.name.clone(); // Đọc tên phần cứng GPU Adapter
                context = Some(Arc::new(GpuContext { // Tạo bản thể GpuContext dùng chung
                    device, // Gán thiết bị device
                    queue, // Gán hàng đợi queue
                    pipeline, // Gán đường ống pipeline
                    layout, // Gán bố cục layout
                    storage_buffer, // Gán bộ đệm Storage tĩnh
                    staging_buffer_a, // Gán bộ đệm Staging A
                    staging_buffer_b, // Gán bộ đệm Staging B
                    score_storage, // Gán bộ đệm Score Storage 64KB
                    score_staging_a, // Gán bộ đệm Score Staging A 64KB
                    score_staging_b, // Gán bộ đệm Score Staging B 64KB
                    weight_buffer, // Gán bộ đệm Weight Buffer 33.57MB
                    bind_group, // Gán nhóm liên kết BindGroup tĩnh
                    ping_pong: AtomicBool::new(false), // Khởi tạo cờ ping_pong false
                    name: adapter_name, // Gán tên hiển thị name
                })); // Kết thúc khởi tạo GpuContext
            } // Kết thúc khối if Ok device
        } // Kết thúc khối if Some adapter

        let status = if context.is_some() { Status::Ready } else { Status::Active }; // Đặt trạng thái Ready nếu có GPU
        Self { // Trả về bản thể Device mới
            guard: Guard::new(), // Khởi tạo VRAM Guard 512MB
            backend, // Gán backend đã phát hiện
            status, // Gán trạng thái khởi tạo
            pad: [0u8; 6], // Khởi tạo mảng đệm pad 6 byte zero
            context, // Gán con trỏ ngữ cảnh context GPU
            extra: [0u8; 48], // Khởi tạo mảng đệm extra 48 byte zero
        } // Kết thúc trả về struct
    } // Kết thúc hàm init

    /// Trả về tham chiếu ngữ cảnh GPU Context nếu phần cứng khả dụng.
    pub fn context(&self) -> Option<Arc<GpuContext>> { // Hàm context lấy con trỏ GpuContext
        self.context.clone() // Sao chép con trỏ đếm tham chiếu Arc
    } // Kết thúc hàm context

    /// Trả về tên hiển thị thực tế của card đồ họa GPU phần cứng.
    pub fn adapter_name(&self) -> String { // Hàm adapter_name lấy tên GPU
        if let Some(ref ctx) = self.context { // Nếu có ngữ cảnh GPU phần cứng
            ctx.name.clone() // Trả về tên card đồ họa thực tế (ví dụ: Apple M1/Intel/Nvidia)
        } else { // Nếu là CPU fallback
            "CPU SIMD Vector Engine".to_string() // Trả về chuỗi tên CPU
        } // Kết thúc kiểm tra context
    } // Kết thúc hàm adapter_name

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
        *self = Self::init(); // Khởi tạo lại thiết bị mới
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
            Backend::Cuda => "NVIDIA CUDA (cuda:0) Native Hardware Engine", // Tên nền tảng NVIDIA CUDA Native
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
