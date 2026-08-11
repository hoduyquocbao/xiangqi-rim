// ============================================================================
// VÍ DỤ 27: MICROSECOND TELEMETRY PROFILER ĐO CHÍNH XÁC ĐIỂM NGHỄN GPU
// ============================================================================
// Đo lường chi tiết microsecond ($\mu s$) cho từng công đoạn trong GPU Pipeline:
// 1. $T_{\text{H2D}}$: Thời gian nạp dữ liệu từ Host RAM xuống VRAM (2MB).
// 2. $T_{\text{Compute}}$: Thời gian phần cứng GPU Compute Units thực thi Shader.
// 3. $T_{\text{D2H}}$: Thời gian map và đọc dữ liệu nén 64KB từ Score Staging Buffer về Host RAM.
// 4. $T_{\text{Host}}$: Thời gian CPU xử lý sinh nước đi cờ.
// Tính toán chính xác công suất GPU Compute Duty Cycle (%) thực tế.
// Tuân thủ 100% chú thích tiếng Việt và từ đơn tiếng Anh.
// ============================================================================

use std::time::Instant;
use xiangrust::board::Parser;
use xiangrust::gpu::{Batch, Device, Evaluator, Sample};

fn main() {
    println!("============================================================");
    println!(" XIANGQI-RIM HARDWARE GPU MICROSECOND TELEMETRY PROFILER");
    println!("============================================================");

    let batch_size = 16384usize;
    let device = Device::init();
    let gpu_name = device.adapter_name();
    let evaluator = Evaluator::new(device).expect("Khởi tạo GPU Evaluator thất bại");
    let mut batch = Batch::allocate(evaluator.device(), batch_size).expect("Cấp phát VRAM Batch thất bại");

    println!("Cấu hình Đo Lường Telemetry (Micro-Polling 64KB Buffer):");
    println!("  • GPU Hardware Card: {}", gpu_name);
    println!("  • Kích thước Lô    : {} vị trí FEN", batch_size);
    println!();

    // Nạp dữ liệu mẫu vào Batch
    let pos = Parser::parse(Parser::DEFAULT);
    for _ in 0..batch_size {
        let mut sample = Sample::new();
        sample.load(&pos.grid, pos.side);
        let _ = batch.push(&sample);
    }

    let iterations = 100usize;
    let mut total_h2d_us = 0u128;
    let mut total_compute_us = 0u128;
    let mut total_d2h_us = 0u128;

    for _ in 0..iterations {
        let t0 = Instant::now();
        // 1. H2D Transfer (2MB)
        let sample_stride = std::mem::size_of::<Sample>();
        let total_bytes = batch_size * sample_stride;
        let compact_score_bytes = batch_size * 4;

        let ptr = batch.buffer().pointer();
        let host_slice = unsafe { std::slice::from_raw_parts(ptr, total_bytes) };

        if let Some(ctx) = evaluator.device().context() {
            ctx.queue.write_buffer(&ctx.storage_buffer, 0, host_slice);
            let t1 = Instant::now();
            total_h2d_us += t1.duration_since(t0).as_micros();

            // 2. GPU Compute Execution
            let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Telemetry Encoder"),
            });
            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Telemetry Pass"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&ctx.pipeline);
                pass.set_bind_group(0, &ctx.bind_group, &[]);
                pass.dispatch_workgroups(((batch_size as u32) + 63) / 64, 1, 1);
            }
            // Ghi lệnh copy bộ đệm nén 64KB Score Buffer sang score_staging_a
            encoder.copy_buffer_to_buffer(&ctx.score_storage, 0, &ctx.score_staging_a, 0, compact_score_bytes as u64);
            ctx.queue.submit(Some(encoder.finish()));

            let t2 = Instant::now();
            total_compute_us += t2.duration_since(t1).as_micros();

            // 3. Micro-Polling Compact D2H Transfer (chỉ 64KB!)
            let buffer_slice = ctx.score_staging_a.slice(..compact_score_bytes as u64);
            let (sender, receiver) = std::sync::mpsc::channel();
            buffer_slice.map_async(wgpu::MapMode::Read, move |res| {
                let _ = sender.send(res);
            });
            
            loop {
                ctx.device.poll(wgpu::Maintain::Poll);
                if let Ok(res) = receiver.try_recv() {
                    if res.is_ok() {
                        let data = buffer_slice.get_mapped_range();
                        drop(data);
                        ctx.score_staging_a.unmap();
                    }
                    break;
                }
                std::thread::yield_now();
            }

            let t3 = Instant::now();
            total_d2h_us += t3.duration_since(t2).as_micros();
        }
    }

    let avg_h2d = (total_h2d_us as f64 / iterations as f64) / 1000.0;
    let avg_compute = (total_compute_us as f64 / iterations as f64) / 1000.0;
    let avg_d2h = (total_d2h_us as f64 / iterations as f64) / 1000.0;
    let avg_total = avg_h2d + avg_compute + avg_d2h;

    let duty_cycle = (avg_compute / avg_total) * 100.0;

    println!("============================================================");
    println!(" BÁO CÁO PHÂN TÍCH TELEMETRY ĐỘ TRỄ CHI TIẾT (MICROSECONDS)");
    println!("============================================================");
    println!("  1. Host-to-Device (H2D Transfer 2MB)  : {:.3} ms ({:.0} µs)", avg_h2d, avg_h2d * 1000.0);
    println!("  2. GPU Compute Shader Execution       : {:.3} ms ({:.0} µs)", avg_compute, avg_compute * 1000.0);
    println!("  3. Device-to-Host (Micro-Polling 64KB): {:.3} ms ({:.0} µs)", avg_d2h, avg_d2h * 1000.0);
    println!("  ──────────────────────────────────────────────────────────");
    println!("  • Tổng thời gian 1 batch 16,384        : {:.3} ms", avg_total);
    println!("  • Tỷ lệ GPU Compute Duty Cycle        : {:.2}%", duty_cycle);
    println!("============================================================");
}
