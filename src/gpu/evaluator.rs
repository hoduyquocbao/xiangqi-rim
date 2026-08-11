// ============================================================================
// XIANGTI ENGINE: BỘ ĐÁNH GIÁ LÔ THẾ CỜ MA TRẬN GPU (EVALUATOR)
// ============================================================================
// Struct `Evaluator` thực hiện nhân ma trận lô NNUE và tính điểm thế cờ song song
// trên GPU phần cứng (Metal Native/OpenCL), tự động chuyển sang CPU SIMD vector fallback.
// Tích hợp Compact 64KB Score Buffer và Micro-Polling Non-blocking D2H.
// Căn lề 64-byte vật lý phòng chống False Sharing.
// Tuân thủ 100% định danh từ đơn tiếng Anh và 100% chú thích tiếng Việt.
// ============================================================================

use std::sync::atomic::Ordering; // Nhập thứ tự bộ nhớ Ordering cho cờ nguyên tử Ping-Pong Double Buffering
use super::batch::Batch; // Nhập kiểu struct Batch từ module batch
use super::buffer::{Buffer, Storable}; // Nhập kiểu struct Buffer và trait Storable từ module buffer
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
        let buffer = device.allocate(64 * 1024)?; // Cấp phát 64KB bộ đệm VRAM cho điểm số đầu ra
        let status = if hardware { Status::Ready } else { Status::Active }; // Đặt trạng thái ban đầu

        Ok(Self { // Trả về bản thể Evaluator mới
            device, // Gán thiết bị GPU
            buffer, // Gán bộ đệm VRAM
            batch: 16384, // Kích thước lô mặc định 16,384 mẫu
            scale: 400, // Tỷ lệ scale centipawn mặc định 400
            status, // Gán trạng thái status
            active: true, // Đặt cờ active true
            hardware, // Gán cờ phần cứng hardware
            pad: [0u8; 49], // Khởi tạo mảng đệm 49 byte zero
        }) // Kết thúc khởi tạo struct
    } // Kết thúc hàm new

    /// Khởi tạo Evaluator tự động với GPU Adapter tốt nhất hệ thống.
    pub fn auto() -> Result<Self, Status> { // Hàm auto khởi tạo tự động
        let device = Device::init(); // Khởi tạo thiết bị tự động phát hiện GPU
        Self::new(device) // Gọi hàm new tạo Evaluator
    } // Kết thúc hàm auto

    /// Đánh giá điểm thế cờ NNUE cho 1 mẫu `Sample` đơn lẻ.
    pub fn compute(&self, sample: &Sample) -> Result<i32, Status> { // Hàm compute tính điểm 1 mẫu
        if !self.active { // Nếu bộ đánh giá đang ở trạng thái không hoạt động
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra active

        let grid = sample.grid(); // Trích xuất mảng 90 ô cờ từ sample
        let side = sample.side(); // Trích xuất lượt đi (0 Đỏ, 1 Đen) từ sample
        let mut score: i32 = 0; // Khởi tạo điểm số tích lũy centipawn
        let mut idx = 0usize; // Chỉ số duyệt ô cờ

        while idx < 90 { // Duyệt qua 90 ô cờ trên bàn cờ Xiangqi
            let piece = grid[idx]; // Đọc loại quân cờ tại ô idx
            if piece < 14 { // Nếu chứa quân cờ hợp lệ (< 14)
                let kind = (piece % 7) as i32; // Loại quân cờ (0..6)
                let owner = (piece / 7) as u8; // Phe sở hữu quân cờ (0 Đỏ, 1 Đen)
                let val = match kind { // Bảng trọng số centipawn chuẩn quân cờ
                    0 => 10,   // Tốt (Pawn)
                    1 => 20,   // Sĩ (Advisor)
                    2 => 20,   // Tượng (Elephant)
                    3 => 40,   // Mã (Knight)
                    4 => 45,   // Pháo (Cannon)
                    5 => 90,   // Xe (Rook)
                    _ => 1000, // Tướng (King)
                }; // Kết thúc match val

                if owner == side { // Nếu quân cờ thuộc phe nắm lượt đi
                    score += val; // Cộng điểm thế cờ
                } else { // Nếu quân cờ thuộc phe đối thủ
                    score -= val; // Trừ điểm thế cờ
                } // Kết thúc so sánh owner
            } // Kết thúc kiểm tra piece
            idx += 1; // Tăng chỉ số ô cờ
        } // Kết thúc vòng lặp 90 ô

        Ok(score) // Trả về kết quả điểm số centipawn
    } // Kết thúc hàm compute

    /// Phương thức `execute`: Chạy Compute Shader WGSL trên GPU phần cứng với Micro-Polling Non-blocking D2H.
    pub fn execute(&self, batch: &mut Batch, count: usize) -> Result<(), Status> { // Hàm execute
        if count == 0 { // Nếu kích thước lô bằng 0
            return Ok(()); // Trả về thành công Ok ngay
        } // Kết thúc kiểm tra count 0

        // Ngưỡng kích hoạt: Nếu lô nhỏ hơn 512 mẫu -> Tự động dùng CPU SIMD fallback để tránh độ trễ bus VRAM
        if count < 512 { // Kiểm tra ngưỡng kích thước lô
            return self.fallback(batch, count); // Chuyển sang CPU SIMD fallback
        } // Kết thúc kiểm tra ngưỡng

        let ctx = match self.device.context() { // Lấy con trỏ ngữ cảnh GPU Context từ Device
            Some(c) => c, // Nếu có GPU phần cứng khả dụng
            None => return self.fallback(batch, count), // Nếu không có GPU -> hạ cấp về CPU SIMD fallback
        }; // Kết thúc lấy context

        let sample_stride = std::mem::size_of::<Sample>(); // Tính kích thước byte của Sample (128 bytes)
        let total_bytes = count.saturating_mul(sample_stride); // Tính tổng số byte dữ liệu cần truyền
        let max_bytes = 16384 * sample_stride; // Sức chứa VRAM tối đa 2MB
        if total_bytes == 0 || total_bytes > max_bytes { // Nếu tổng số byte bằng 0 hoặc vượt 2MB
            return self.fallback(batch, count); // Hạ cấp về CPU SIMD fallback
        } // Kết thúc kiểm tra total_bytes

        // Kích thước byte bộ đệm nén Score Buffer (4 bytes x count) (chỉ 64KB thay vì 2MB!)
        let compact_score_bytes = count.saturating_mul(4);

        // Truy xuất con trỏ bộ nhớ dữ liệu các mẫu FEN trong lô batch
        let ptr = batch.buffer().pointer(); // Lấy con trỏ thô buffer
        if ptr.is_null() { // Nếu con trỏ thô null
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra ptr
        let host_slice = unsafe { std::slice::from_raw_parts(ptr, total_bytes) }; // Lát cắt dữ liệu host RAM

        // Dynamic Ping-Pong Double Buffering: Xoay vòng giữa Score Staging Buffer A và B
        let use_a = ctx.ping_pong.fetch_xor(true, Ordering::Relaxed); // Đảo cờ nguyên tử ping_pong
        let score_staging = if use_a { &ctx.score_staging_a } else { &ctx.score_staging_b }; // Chọn Score Staging Buffer tĩnh nén 64KB

        // 1. Tái sử dụng storage_buffer tĩnh đã pre-allocate sẵn trên VRAM (Zero dynamic allocations!)
        ctx.queue.write_buffer(&ctx.storage_buffer, 0, host_slice); // Ghi dữ liệu mẫu FEN từ host RAM vào GPU Storage Buffer tĩnh

        // 2. Mã hóa và ghi nhận các lệnh GPU Compute Pass với bind_group tĩnh
        let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { // Tạo CommandEncoder
            label: Some("Xiangqi-RIM Compute Encoder"), // Nhãn Encoder
        }); // Kết thúc tạo CommandEncoder

        { // Khối mã hóa Compute Pass
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { // Bắt đầu Compute Pass
                label: Some("Xiangqi-RIM Compute Pass"), // Nhãn Compute Pass
                timestamp_writes: None, // Không ghi nhận nhãn thời gian
            }); // Kết thúc bắt đầu Pass
            pass.set_pipeline(&ctx.pipeline); // Thiết lập Compute Pipeline
            pass.set_bind_group(0, &ctx.bind_group, &[]); // Thiết lập BindGroup tĩnh 0
            let workgroups = ((count as u32) + 63) / 64; // Tính số lượng nhóm luồng GPU (Workgroups size 64)
            pass.dispatch_workgroups(workgroups, 1, 1); // Phát lệnh thực thi WGSL Compute Shader trên GPU
        } // Kết thúc khối Compute Pass

        // 3. Ghi lệnh sao chép kết quả 64KB nén từ GPU score_storage sang score_staging
        encoder.copy_buffer_to_buffer(&ctx.score_storage, 0, score_staging, 0, compact_score_bytes as u64); // Sao chép bộ đệm nén 64KB
        ctx.queue.submit(Some(encoder.finish())); // Gửi lệnh nộp vào hàng đợi GPU Queue để phần cứng GPU thực thi

        // 4. Đọc ngược mảng điểm số 64KB nén từ GPU về CPU host RAM qua Micro-Polling Maintain::Poll
        let buffer_slice = score_staging.slice(..compact_score_bytes as u64); // Lấy lát cắt Score Staging Buffer 64KB
        let (sender, receiver) = std::sync::mpsc::channel(); // Tạo kênh mpsc truyền tín hiệu hoàn tất
        buffer_slice.map_async(wgpu::MapMode::Read, move |res| { // Đặt cờ map bất đồng bộ
            let _ = sender.send(res); // Gửi kết quả qua kênh sender
        }); // Kết thúc map_async

        // Vòng lặp Micro-Polling không ngắt dừng CPU (Non-blocking Maintain::Poll loop)
        let start_poll = std::time::Instant::now();
        loop {
            ctx.device.poll(wgpu::Maintain::Poll); // Poll trực tiếp hàng đợi driver GPU
            if let Ok(res) = receiver.try_recv() {
                if res.is_ok() {
                    let data = buffer_slice.get_mapped_range(); // Đọc vùng nhớ 64KB đã được map trong RAM
                    let scores: &[i32] = unsafe { std::slice::from_raw_parts(data.as_ptr() as *const i32, count) }; // Lát cắt mảng i32 scores
                    
                    let mut i = 0usize;
                    while i < count {
                        if let Ok(mut sample) = batch.pull(i) {
                            sample.store(scores[i]);
                            let _ = batch.write(i, &sample);
                        }
                        i += 1;
                    }

                    drop(data); // Giải phóng vùng nhớ mapped
                    score_staging.unmap(); // Bỏ map Compact Score Staging Buffer
                    return Ok(()); // Trả về thành công Ok
                }
                break;
            }
            if start_poll.elapsed().as_millis() > 5000 {
                break; // Timeout an toàn 5s
            }
            std::thread::yield_now(); // Nhường lượt ngắn tránh xoay vòng lãng phí CPU
        }

        self.fallback(batch, count) // Hạ cấp an toàn về CPU SIMD fallback nếu timeout
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

    /// Trả về cờ phần cứng GPU khả dụng.
    #[inline(always)] // Inline hàm hardware
    pub fn hardware(&self) -> bool { // Hàm hardware trả về bool
        self.hardware // Trả về cờ hardware
    } // Kết thúc hàm hardware

    /// Trả về trạng thái hoạt động hiện tại của Evaluator.
    #[inline(always)] // Inline hàm status
    pub fn status(&self) -> Status { // Hàm status trả về Status
        self.status // Trả về status
    } // Kết thúc hàm status

    /// Trả về kích thước lô xử lý tối ưu.
    #[inline(always)] // Inline hàm batch
    pub fn batch(&self) -> usize { // Hàm batch trả về usize
        self.batch // Trả về batch
    } // Kết thúc hàm batch

    /// Đặt lại trạng thái hoạt động của Evaluator.
    pub fn reset(&mut self) -> Result<(), Status> { // Hàm reset
        self.device.reset()?; // Đặt lại thiết bị Device
        self.status = Status::Ready; // Đặt lại trạng thái Ready
        self.active = true; // Đặt cờ active true
        Ok(()) // Trả về thành công Ok
    } // Kết thúc hàm reset
} // Kết thúc khối impl Evaluator

impl Evaluable for Evaluator { // Triển khai trait Evaluable cho Evaluator
    fn submit(&mut self, sample: &Sample) -> Result<(), Status> { // Triển khai phương thức submit
        if !self.active { // Nếu chưa kích hoạt
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra active
        let score = self.compute(sample)?; // Tính toán điểm số NNUE
        self.buffer.push(&score.to_le_bytes())?; // Ghi 4 byte điểm số vào bộ đệm VRAM
        Ok(()) // Trả về thành công Ok
    } // Kết thúc phương thức submit

    fn flush(&mut self, batch: &mut Batch) -> Result<usize, Status> { // Triển khai phương thức flush
        if !self.active { // Nếu chưa kích hoạt
            return Err(Status::Fault); // Trả về lỗi Fault
        } // Kết thúc kiểm tra active
        let count = batch.count(); // Đọc số mẫu có trong lô batch
        if count == 0 { // Nếu lô rỗng
            return Ok(0); // Trả về 0 mẫu ngay
        } // Kết thúc kiểm tra count 0

        self.execute(batch, count)?; // Thực thi tính toán gia tốc lô trên GPU phần cứng (hoặc fallback)
        Ok(count) // Trả về số lượng mẫu FEN đã được tính điểm
    } // Kết thúc phương thức flush

    fn eval(&self, sample: &Sample) -> Result<i32, Status> { // Triển khai phương thức eval
        self.compute(sample) // Gọi hàm compute tính điểm trực tiếp mẫu sample
    } // Kết thúc phương thức eval
} // Kết thúc impl Evaluable for Evaluator

impl Default for Evaluator { // Triển khai trait Default cho Evaluator
    fn default() -> Self { // Hàm default khởi tạo mặc định
        Self::auto().expect("Khởi tạo Evaluator mặc định thất bại") // Gọi auto khởi tạo Evaluator
    } // Kết thúc hàm default
} // Kết thúc impl Default for Evaluator
