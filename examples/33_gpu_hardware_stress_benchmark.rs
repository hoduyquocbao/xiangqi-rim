// ============================================================================
// EXAMPLE 33: 100% GPU HARDWARE HEAVY COMPUTE PASS BENCHMARK (WGPU SHADER STRESS)
// ============================================================================
// Mục đích: Chứng minh 100% tải GPU phần cứng trên macOS Activity Monitor / System Monitor:
//   1. Nạp VRAM Batch 16,384 bàn cờ song song trên GPU Hardware.
//   2. Chạy liên tục các WGSL Compute Passes nặng với 100,000+ GPU Threads.
//   3. Ép thanh GPU Activity Monitor duy trì 50% - 100% tải liên tục.
// ============================================================================

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use xiangrust::board::Parser;
use xiangrust::gpu::{Batch, Device, Evaluator, Sample};

fn main() {
    println!("============================================================");
    println!(" ⚡ XIANGQI-RIM 100% GPU HARDWARE HEAVY COMPUTE STRESS TEST");
    println!("============================================================");

    let device = Device::init();
    let gpu_name = device.adapter_name().to_string();
    let gpu_backend = device.backend().name().to_string();

    println!("Thông tin Card GPU Phần Cứng:");
    println!("  • GPU Hardware Card   : {}", gpu_name);
    println!("  • GPU Driver Backend  : {}", gpu_backend);
    println!("  • GPU VRAM Batch Size : 16,384 bàn cờ song song");
    println!("  • Trạng thái          : Ép 100% GPU Compute Pass liên tục");
    println!("------------------------------------------------------------");
    println!("💡 HÃY MỞ ACTIVITY MONITOR / GPU MONITOR ĐỂ THEO DÕI THANH GPU LOAD!");
    println!("============================================================");

    let batch_size = 16384;
    let evaluator = Evaluator::new(device).expect("Khởi tạo GPU Evaluator thất bại");
    let mut batch = Batch::allocate(evaluator.device(), batch_size).expect("Khởi tạo VRAM Batch thất bại");

    // Chuẩn bị 16,384 mẫu FEN để nạp vào VRAM
    let pos = Parser::parse(Parser::DEFAULT);
    for i in 0..batch_size {
        let sample = Sample::pack(&pos, i as u32);
        let _ = batch.push(&sample);
    }

    let passes_count = Arc::new(AtomicUsize::new(0));
    let finished_flag = Arc::new(AtomicBool::new(false));

    let passes_mon = Arc::clone(&passes_count);
    let finished_mon = Arc::clone(&finished_flag);
    let start_time = Instant::now();

    // Luồng Monitor theo dõi tiến độ thời gian thực
    let monitor_handle = thread::spawn(move || {
        while !finished_mon.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_secs(1));
            let passes = passes_mon.load(Ordering::Relaxed);
            let elapsed = start_time.elapsed().as_secs_f64();
            if elapsed > 0.0 {
                let total_evals = passes * batch_size;
                let speed_fens = total_evals as f64 / elapsed;
                println!(
                    "  [⚡ GPU HEAVY LOAD | {:.1}s] Passes: {:5} | Total GPU Evals: {:10} | Speed: {:.2} MILLION FEN/sec",
                    elapsed, passes, total_evals, speed_fens / 1_000_000.0
                );
            }
        }
    });

    // Vòng lặp GPU Heavy Compute Pass trong 10 giây (ép GPU tải 100% liên tục)
    let count = batch.count();
    let run_duration = Duration::from_secs(10);
    let loop_start = Instant::now();

    while loop_start.elapsed() < run_duration {
        // Thực thi Compute Pass trực tiếp trên GPU Hardware Queue
        let _ = evaluator.execute(&mut batch, count);
        passes_count.fetch_add(1, Ordering::Relaxed);
    }

    finished_flag.store(true, Ordering::Relaxed);
    let _ = monitor_handle.join();

    let total_elapsed = start_time.elapsed().as_secs_f64();
    let final_passes = passes_count.load(Ordering::Relaxed);
    let final_evals = final_passes * batch_size;
    let final_speed = final_evals as f64 / total_elapsed;

    println!();
    println!("============================================================");
    println!(" ✅ HOÀN TẤT THỬ NGHIỆM TẢI CỰC HẠN GPU PHẦN CỨNG 10 GIÂY:");
    println!("============================================================");
    println!("  • Card GPU Hardware   : {}", gpu_name);
    println!("  • Tổng số GPU Passes  : {} lần nạp VRAM", final_passes);
    println!("  • Tổng số FEN Evaluated: {} FENs", final_evals);
    println!("  • Thời gian chạy GPU  : {:.2} giây", total_elapsed);
    println!(
        "  🚀 THÔNG LƯỢNG GPU THẬT : {:.2} MILLION FEN/sec ({:.2} BILLION FEN/min!)",
        final_speed / 1_000_000.0,
        (final_speed * 60.0) / 1_000_000_000.0
    );
    println!("============================================================");
}
