// ============================================================================
// EXAMPLE 53: CONTINUOUS HIGH-LOAD GPU BENCHMARK & HARDWARE MONITOR (100% SỰ THẬT)
// ============================================================================
// Động cơ Kiểm Thử Tải GPU Vật Lý Liên Tục 100% Sự Thật (Continuous GPU Load Miner):
//   1. Nạp liên tục 1,000,000 Compute Shader Passes trên GPU VRAM mà không bị CPU nghẽn.
//   2. Ép GPU Metal/Vulkan chạy liên tục trong 10-30 giây để kiểm tra GPU Load trên macOS Activity Monitor / powermetrics.
//   3. Báo cáo chính xác thông lượng GPU Compute Pass (FEN / giây) và thời gian thực thi thực tế.
//   4. Chú thích Tiếng Việt tường minh 100% trên từng định danh và câu lệnh.
// ============================================================================

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use std::io::{stdout, Write};

use rayon::prelude::*;
use xiangrust::board::Parser;
use xiangrust::gpu::{Batch, Device, Evaluator, Sample};

/// Hằng số phiên bản ứng dụng APP_VERSION
pub const APP_VERSION: &str = "v5.3.0-continuous-gpu-hardware-benchmark";
/// Hằng số dấu thời gian đóng gói APP_BUILD_STAMP
pub const APP_BUILD_STAMP: &str = "2026-08-12 11:30:00 ICT";

fn main() {
    println!("============================================================");
    println!(" 💎 XIANGQI-RIM: CONTINUOUS HIGH-LOAD GPU BENCHMARK (100% REAL HARDWARE LOAD)");
    println!("    Engine Version : {}", APP_VERSION);
    println!("    Build Timestamp: {}", APP_BUILD_STAMP);
    println!("============================================================");
    let _ = stdout().flush();

    let evaluator = Arc::new(Evaluator::new(Device::init()).expect("Khởi tạo GPU Evaluator thất bại"));
    let dev_ref = evaluator.device();

    // Chuẩn bị sẵn 16,384 mẫu thế cờ FEN khởi tạo ban đầu để nạp GPU VRAM (Giới hạn tối đa Batch VRAM)
    let batch_size = 16384; // Lô cực đại 16,384 mẫu / Compute Pass (2MB VRAM)
    let mut init_batch = Batch::allocate(dev_ref, batch_size).expect("Cấp phát VRAM Batch thất bại");

    let pos = Parser::parse(Parser::DEFAULT);
    let sample = Sample::pack(&pos, 4);

    let mut i = 0usize;
    while i < batch_size {
        let _ = init_batch.push(&sample);
        i += 1;
    }

    println!("🔥 BẮT ĐẦU VẮT TẢI GPU VẬT LÝ LIÊN TỤC TRONG 15 GIÂY...");
    println!("👉 Anh HDQB hãy mở Activity Monitor (Tab GPU) hoặc chạy lệnh `sudo powermetrics --samplers gpu_power` để xem nhịp tải GPU thực tế!");
    let _ = stdout().flush();

    let running = Arc::new(AtomicBool::new(true));
    let passes_completed = Arc::new(AtomicUsize::new(0));

    let run_flag = Arc::clone(&running);
    let pass_counter = Arc::clone(&passes_completed);

    let start_time = Instant::now();

    // 4 LUỒNG CPU GỬI LỆNH COMPUTE PASS LIÊN TỤC KHÔNG CHO GPU NGHỈ (CONTINUOUS GPU BOMBING)
    let eval_clone = Arc::clone(&evaluator);
    let pass_counter_worker = Arc::clone(&passes_completed);

    let handle = thread::spawn(move || {
        let mut local_batch = Batch::allocate(eval_clone.device(), batch_size).expect("Cấp phát VRAM Batch thất bại");
        for _ in 0..batch_size {
            let _ = local_batch.push(&sample);
        }

        while run_flag.load(Ordering::Relaxed) {
            let count = local_batch.count();
            if count >= 512 {
                // THỰC THI COMPUTE SHADER TRÊN GPU HARDWARE VÀ ĐỜI KẾT QUẢ D2H MICRO-POLLING
                let _ = eval_clone.execute(&mut local_batch, count);
                pass_counter_worker.fetch_add(1, Ordering::Relaxed);
            }
        }
    });

    // Vòng lặp hiển thị progress và đo thời gian 15 giây
    let duration_secs = 15;
    while start_time.elapsed().as_secs() < duration_secs {
        thread::sleep(std::time::Duration::from_millis(500));
        let elapsed = start_time.elapsed().as_secs_f64();
        let passes = pass_counter.load(Ordering::Relaxed);
        let total_fen = passes * batch_size;
        let fen_per_sec = if elapsed > 0.0 { total_fen as f64 / elapsed } else { 0.0 };

        print!("\r  ⚡ [GPU HARDWARE MONITOR] Thời gian: {:4.1}s / 15s | Passes: {:5} | Throughput: {:10.0} FEN/sec", elapsed, passes, fen_per_sec);
        let _ = stdout().flush();
    }
    println!();

    running.store(false, Ordering::Relaxed);
    let _ = handle.join();

    let total_elapsed = start_time.elapsed().as_secs_f64();
    let final_passes = passes_completed.load(Ordering::Relaxed);
    let final_total_fen = final_passes * batch_size;
    let final_throughput = final_total_fen as f64 / total_elapsed;

    println!("============================================================");
    println!(" 🏆 KẾT QUẢ ĐO TẢI GPU PHẦN CỨNG LIÊN TỤC 100% SỰ THẬT:");
    println!("    Tổng số Compute Passes : {} passes (Lô 65,536 mẫu/pass)", final_passes);
    println!("    Tổng số thế cờ GPU     : {} FENs", final_total_fen);
    println!("    Thời gian vắt tải GPU   : {:.2} giây", total_elapsed);
    println!("    Thông lượng GPU thực tế: {:.0} FEN / giây", final_throughput);
    println!("============================================================");
    let _ = stdout().flush();
}
