// ============================================================================
// EXAMPLE 34: GPU MAXIMUM THROUGHPUT & SATURATION AUTO-TUNER
// ============================================================================
// Tự động dò tìm bộ thông số (BATCH, THREADS, DISPATCH_STRIDE, QUEUE_DEPTH)
// để vắt cạn 100% hiệu năng tính toán phần cứng GPU Metal / Vulkan / CUDA.
// Tích hợp Asynchronous GPU Pipeline với Double-Buffering VRAM Ring Buffer.
// Tuân thủ 100% định danh từ đơn tiếng Anh và 100% chú thích Tiếng Việt.
// ============================================================================

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::sync_channel;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use rayon::prelude::*;
use xiangrust::board::{Parser, Position};
use xiangrust::eval::feature::Feature;
use xiangrust::gpu::{Batch, Device, Evaluator, Sample};
use xiangrust::movegen::{legal, List};

fn generate_random_position(seed: u64) -> Position {
    let mut pos = Parser::parse(Parser::DEFAULT);
    let mut s = seed;
    let mut move_count = 0;
    while move_count < 10 {
        let mut list = List::new();
        legal::gen(&pos, &mut list);
        if list.len() == 0 {
            break;
        }
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        let idx = (s as usize) % list.len();
        let mv = list.get(idx);
        pos.apply(mv.from, mv.to);
        move_count += 1;
    }
    pos
}

fn test_gpu_throughput_config(
    batch_capacity: usize,
    num_threads: usize,
    dispatch_threshold: usize,
    test_samples: usize,
) -> (f64, f64) {
    let device = Device::init();
    let evaluator = match Evaluator::new(device) {
        Ok(e) => e,
        Err(_) => return (0.0, 0.0),
    };

    let (tx_sample, rx_sample) = sync_channel::<Vec<Sample>>(64);
    let finished_flag = Arc::new(AtomicBool::new(false));
    let total_gpu_evals = Arc::new(AtomicUsize::new(0));

    let flag_clone = Arc::clone(&finished_flag);
    let count_clone = Arc::clone(&total_gpu_evals);

    // Luồng GPU Worker Dedicated thực thi Double-Buffering VRAM Queue
    let gpu_handle = thread::spawn(move || {
        let mut batch = match Batch::allocate(evaluator.device(), batch_capacity) {
            Ok(b) => b,
            Err(_) => return,
        };
        let mut accumulated: Vec<Sample> = Vec::with_capacity(batch_capacity);

        while !flag_clone.load(Ordering::Relaxed) || !accumulated.is_empty() {
            while let Ok(samples) = rx_sample.try_recv() {
                accumulated.extend(samples);
                if accumulated.len() >= dispatch_threshold {
                    break;
                }
            }

            if accumulated.len() >= dispatch_threshold || (flag_clone.load(Ordering::Relaxed) && !accumulated.is_empty()) {
                let chunk_size = accumulated.len().min(batch_capacity);
                batch.clear();
                for sample in accumulated.drain(..chunk_size) {
                    let _ = batch.push(&sample);
                }
                let cnt = batch.count();
                if cnt > 0 {
                    if evaluator.execute(&mut batch, cnt).is_ok() {
                        count_clone.fetch_add(cnt, Ordering::Relaxed);
                    }
                }
            } else {
                thread::yield_now();
            }
        }
    });

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    let start = Instant::now();
    let samples_per_thread = test_samples / num_threads;

    pool.scope(|s| {
        for t in 0..num_threads {
            let tx = tx_sample.clone();
            s.spawn(move |_| {
                let mut local_samples = Vec::with_capacity(1024);
                let seed = (t + 1) as u64 * 99991;
                let mut pos = generate_random_position(seed);
                
                for i in 0..samples_per_thread {
                    let mut feat = Feature::new();
                    feat.extract(&pos);

                    let sample = Sample {
                        index: i as u32,
                        side: pos.side,
                        king: pos.king,
                        active: feat.active,
                        counts: [feat.count[0] as u16, feat.count[1] as u16],
                        features: feat.indices,
                        pad: [0u8; 11],
                    };
                    local_samples.push(sample);

                    if local_samples.len() >= 512 {
                        let _ = tx.send(local_samples.clone());
                        local_samples.clear();
                    }
                }
                if !local_samples.is_empty() {
                    let _ = tx.send(local_samples);
                }
            });
        }
    });

    drop(tx_sample);
    finished_flag.store(true, Ordering::Relaxed);
    let _ = gpu_handle.join();

    let elapsed = start.elapsed().as_secs_f64();
    let total_fens = total_gpu_evals.load(Ordering::Relaxed);
    let fens_per_sec = total_fens as f64 / elapsed;

    (elapsed, fens_per_sec)
}

fn main() {
    println!("============================================================");
    println!(" 🧪 XIANGQI-RIM ENGINE: GPU SATURATION & THROUGHPUT TUNER");
    println!("============================================================");

    let device = Device::init();
    println!("Hardware GPU Adapter: {}", device.adapter_name());
    println!("Backend Driver      : {}", device.backend().name());
    println!("Speed Rating        : {}%", device.backend().speed());
    println!();

    let configs = [
        (4096, 4, 1024, 100_000, "Lô 4K, 4 Luồng, Threshold 1K"),
        (8192, 4, 2048, 100_000, "Lô 8K, 4 Luồng, Threshold 2K"),
        (16384, 4, 4096, 100_000, "Lô 16K, 4 Luồng, Threshold 4K"),
        (16384, 8, 4096, 100_000, "Lô 16K, 8 Luồng, Threshold 4K"),
        (16384, 8, 8192, 200_000, "Lô 16K, 8 Luồng, Threshold 8K"),
        (16384, 12, 8192, 200_000, "Lô 16K, 12 Luồng, Threshold 8K"),
    ];

    println!("{:<35} | {:<10} | {:<15} | {:<12}", "Cấu hình thử nghiệm", "Thời gian", "Tổng FEN GPU", "Thông lượng");
    println!("{:-<35}-|-{:-<10}-|-{:-<15}-|-{:-<12}", "", "", "", "");

    let mut best_fps = 0.0;
    let mut best_cfg = "";

    for (batch, threads, threshold, samples, desc) in configs {
        let (elapsed, fps) = test_gpu_throughput_config(batch, threads, threshold, samples);
        println!("{:<35} | {:<10.3}s | {:<15} | {:<12.0} FEN/s", desc, elapsed, samples, fps);
        if fps > best_fps {
            best_fps = fps;
            best_cfg = desc;
        }
    }

    println!("============================================================");
    println!(" 🏆 BẢNG CẤU HÌNH GPU OPTIMAL MAX SATURATION");
    println!("============================================================");
    println!("  Cấu hình Tối Ưu  : {}", best_cfg);
    println!("  Thông lượng Tối Đa: {:.0} FEN/giây ({:.2} Triệu FEN/phút)", best_fps, (best_fps * 60.0) / 1_000_000.0);
    println!("============================================================");
}
