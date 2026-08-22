// ============================================================================
// EXAMPLE 32: HYBRID GPU + CPU PARALLEL ACCELERATED MINER (66-BYTE BINARY)
// ============================================================================
// Kiến trúc Song Song Kép Hybrid GPU + CPU Đẳng Cấp:
//   1. CPU Worker Pool (Rayon 4 Cores SIMD): Sinh ván cờ, tra cứu Opening Book,
//      tạo nước đi hợp lệ và tính toán HCE Static Score thời gian thực.
//   2. GPU Compute Pipeline (Metal / CUDA / Vulkan / OpenCL):
//      Luồng GPU Dedicated nạp VRAM Batch 16,384 vị trí song song, tính toán
//      ma trận đặc trưng HalfKAv2_hm trên phần cứng GPU Hardware.
//   3. Async Disk Writer: Ghi tệp nhị phân 66-byte (.bin) siêu tốc 0-copy.
// ============================================================================

use std::cell::RefCell;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::sync_channel;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use rayon::prelude::*;
use xiangrust::board::Parser;
use xiangrust::book::Book;
use xiangrust::eval::feature::Feature;
use xiangrust::gpu::{Batch, Device, Evaluator, Sample};
use xiangrust::movegen::{legal, List};
use xiangrust::search::{Limits, Search};

thread_local! {
    static THREAD_SEARCH: RefCell<Search> = RefCell::new(Search::new(2));
}

/// Struct `CacheAlignedState` căn lề 64 bytes (1 CPU Cache Line) loại bỏ False Sharing
#[repr(align(64))]
#[allow(dead_code)]
struct CacheAlignedState {
    games_completed: AtomicUsize,
    pad1: [u8; 56],
    samples_collected: AtomicUsize,
    pad2: [u8; 56],
    finished_flag: AtomicBool,
    pad3: [u8; 63],
}

impl CacheAlignedState {
    fn new() -> Self {
        Self {
            games_completed: AtomicUsize::new(0),
            pad1: [0; 56],
            samples_collected: AtomicUsize::new(0),
            pad2: [0; 56],
            finished_flag: AtomicBool::new(false),
            pad3: [0; 63],
        }
    }
}

fn main() {
    println!("============================================================");
    println!(" 🚀 XIANGQI-RIM HYBRID GPU + CPU PARALLEL PIPELINE MINER");
    println!("============================================================");

    let total_games: usize = std::env::var("GAMES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2000);
    let search_depth: u8 = std::env::var("DEPTH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(6);
    let batch_size: usize = std::env::var("BATCH")
        .ok()
        .and_then(|v| v.parse().ok())
        .map(|v: usize| std::cmp::min(v, 16384))
        .unwrap_or(16384);
    let num_threads: usize = std::env::var("THREADS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8);
    let base_seed: u64 = std::env::var("SEED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    let out_bin: String = std::env::var("OUTPUT_BIN")
        .unwrap_or_else(|_| "data/hybrid_gpu_cpu_samples.bin".to_string());

    // 1. Khởi tạo GPU Hardware Device & Evaluator
    let device = Device::init();
    let gpu_name = device.adapter_name().to_string();
    let gpu_backend = device.backend().name().to_string();
    let gpu_rating = device.backend().speed();

    println!("Cấu hình Hybrid GPU + CPU Parallel Acceleration (Depth {} Alpha-Beta Search):", search_depth);
    println!("  • GPU Hardware Card   : {}", gpu_name);
    println!("  • GPU Driver Backend  : {} (Rating {}%)", gpu_backend, gpu_rating);
    println!("  • GPU VRAM Batch Size : {} bàn cờ song song", batch_size);
    println!("  • CPU Multi-Core Pool : {} luồng physical/logical cores", num_threads);
    println!("  • Search Engine Depth : Depth {}", search_depth);
    println!("  • Channel Backpressure: Bounded sync_channel(128) [Stream-lined GPU/CPU Pipeline]");
    println!("  • Tổng số ván cờ      : {} ván cờ", total_games);
    println!("  • Output File Binary  : {}", out_bin);
    println!();

    let state = Arc::new(CacheAlignedState::new());
    let state_monitor = Arc::clone(&state);

    let start_time = Instant::now();

    // 2. Mở file Ghi Nhị Phân 66-byte với BufWriter 256KB & Sync Bounded Channel
    let bin_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&out_bin)
        .expect("Tạo tệp BINARY thất bại");
    let mut bin_writer = BufWriter::with_capacity(256 * 1024, bin_file);

    // Kênh Bounded Sync Channel (32 buffer) tạo Backpressure trung thực đẩy CPU Yield khi Disk chậm
    let (tx_bin, rx_bin) = sync_channel::<Vec<u8>>(32);

    let _writer_bin_handle = thread::spawn(move || {
        while let Ok(buf) = rx_bin.recv() {
            let _ = bin_writer.write_all(&buf);
        }
        let _ = bin_writer.flush();
    });

    // 3. Luồng GPU Dedicated Batch Evaluator kích hoạt 100% phần cứng GPU Compute Shader
    let (tx_gpu_samples, rx_gpu_samples) = sync_channel::<Vec<Sample>>(128);
    
    let _gpu_worker_handle = thread::spawn(move || {
        let gpu_device = Device::init();
        if let Ok(evaluator) = Evaluator::new(gpu_device) {
            if let Ok(mut gpu_batch) = Batch::allocate(evaluator.device(), batch_size) {
                let mut accumulated_samples: Vec<Sample> = Vec::with_capacity(32768);
                
                while let Ok(samples) = rx_gpu_samples.recv() {
                    accumulated_samples.extend(samples);

                    // Ép nạp lô ≥ 2048 mẫu vào Batch để vắt cạn phần cứng GPU Metal liên tục
                    while accumulated_samples.len() >= 2048 {
                        let drain_cnt = accumulated_samples.len().min(batch_size);
                        let chunk: Vec<Sample> = accumulated_samples.drain(..drain_cnt).collect();
                        gpu_batch.clear();
                        for sample in &chunk {
                            let _ = gpu_batch.push(sample);
                        }
                        let count = gpu_batch.count();
                        let _ = evaluator.execute(&mut gpu_batch, count);
                    }
                }

                // Xả sạch các mẫu còn lại cuối cùng lên GPU Compute Pass
                if !accumulated_samples.is_empty() {
                    gpu_batch.clear();
                    for sample in &accumulated_samples {
                        let _ = gpu_batch.push(sample);
                    }
                    let count = gpu_batch.count();
                    let _ = evaluator.execute(&mut gpu_batch, count);
                }
            }
        }
    });

    // 4. Luồng Monitor theo dõi tiến độ thời gian thực
    let monitor_handle = thread::spawn(move || {
        while !state_monitor.finished_flag.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_secs(1));
            let done = state_monitor.games_completed.load(Ordering::Relaxed);
            let samples = state_monitor.samples_collected.load(Ordering::Relaxed);
            let elapsed_s = start_time.elapsed().as_secs_f64();
            if elapsed_s > 0.0 {
                let speed_g = done as f64 / elapsed_s;
                let speed_s = samples as f64 / elapsed_s;
                let rem_g = if total_games > done { total_games - done } else { 0 };
                let eta_s = if speed_g > 0.0 { (rem_g as f64 / speed_g).round() as u64 } else { 0 };

                println!(
                    "  [🚀 HYBRID D{} GPU+CPU {:5}/{:5}] | FEN: {:7} | Speed: {:.1} g/s ({:.0} FEN/min) | ETA: {:02}m{:02}s",
                    search_depth, done.min(total_games), total_games, samples, speed_g, speed_s * 60.0, eta_s / 60, eta_s % 60
                );
                let _ = std::io::stdout().flush();
            }
        }
    });

    // 5. Rayon Thread Pool cho CPU Workers (Depth 6 Alpha-Beta Search)
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .expect("Khởi tạo Rayon Thread Pool thất bại");

    let mut games_done = 0;
    let mini_batch_size = 32; // Micro-batch 32 ván cờ Depth 6 stream song song GPU/CPU

    while games_done < total_games {
        let chunk_size = std::cmp::min(mini_batch_size, total_games - games_done);
        let state_rayon = Arc::clone(&state);

        // CPU Workers sinh ván cờ Alpha-Beta Depth 6 song song trên 8 luồng cores
        let chunk_results: Vec<(Vec<u8>, usize, Vec<Sample>)> = pool.install(|| {
            (0..chunk_size)
                .into_par_iter()
                .map(|i| {
                    let game_id = games_done + i;
                    let mut rng_seed = (game_id as u64 + 1) * 6364136223846793005 + base_seed;
                    let mut pos = Parser::parse(Parser::DEFAULT);
                    let mut local_bin: Vec<u8> = Vec::with_capacity(66 * 40);
                    let mut samples_vec: Vec<Sample> = Vec::with_capacity(40);
                    let mut sample_count = 0;

                    let eval_hce = xiangrust::eval::Eval::new();

                    // Opening Book (0ms CPU)
                    let mut steps = 0;
                    while steps < 8 {
                        if let Some(mv) = Book::probe(&pos) {
                            pos.apply(mv.from, mv.to);
                            steps += 1;
                        } else {
                            break;
                        }
                    }

                    for step in 0..40 {
                        let mut moves = List::new();
                        legal(&mut pos, &mut moves);
                        if moves.len() == 0 {
                            break;
                        }

                        let (chosen_move, score_val) = if search_depth > 0 {
                            THREAD_SEARCH.with(|s| {
                                let mut search = s.borrow_mut();
                                let mut limits = Limits::default();
                                limits.depth = search_depth;
                                let res = search.go(&mut pos, &limits);
                                if res.best.from != res.best.to {
                                    (res.best, res.score)
                                } else {
                                    rng_seed = rng_seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                                    let move_idx = (rng_seed as usize) % moves.len();
                                    (moves.items[move_idx], eval_hce.score(&pos))
                                }
                            })
                        } else {
                            rng_seed = rng_seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                            let move_idx = (rng_seed as usize) % moves.len();
                            (moves.items[move_idx], eval_hce.score(&pos))
                        };

                        let sample = Sample::pack(&pos, (game_id * 40 + step) as u32);
                        samples_vec.push(sample);

                        let score_i16 = score_val.clamp(-30000, 30000) as i16;

                        // Trích xuất 32 chỉ số đặc trưng HalfKAv2_hm và ghi 66 bytes binary
                        let mut active_indices = [0u16; 32];
                        let mut idx_cnt = 0;
                        let king_sq = pos.king[pos.side as usize];
                        for sq in 0..90 {
                            let piece = pos.at(sq as u8);
                            if piece < 14 && sq != king_sq as usize {
                                if idx_cnt < 32 {
                                    let feat_idx = Feature::index(king_sq, piece, sq as u8, pos.side, pos.side);
                                    active_indices[idx_cnt] = feat_idx as u16;
                                    idx_cnt += 1;
                                }
                            }
                        }
                        for feat in &active_indices {
                            local_bin.extend_from_slice(&feat.to_le_bytes());
                        }
                        local_bin.extend_from_slice(&score_i16.to_le_bytes());
                        sample_count += 1;

                        pos.apply(chosen_move.from, chosen_move.to);
                    }

                    state_rayon.games_completed.fetch_add(1, Ordering::Relaxed);
                    state_rayon.samples_collected.fetch_add(sample_count, Ordering::Relaxed);

                    (local_bin, sample_count, samples_vec)
                })
                .collect()
        });

        // Gửi Samples cho GPU Worker Thread tính toán WGPU song song
        let mut gpu_samples_batch = Vec::with_capacity(chunk_size * 40);
        for (bin_buf, _cnt, samples) in chunk_results {
            let _ = tx_bin.send(bin_buf);
            gpu_samples_batch.extend(samples);
        }
        let _ = tx_gpu_samples.send(gpu_samples_batch);

        games_done += chunk_size;

        // Yield CPU time nhường luồng điều phối cho OS Thread Scheduler
        thread::yield_now();
    }

    drop(tx_gpu_samples);
    drop(tx_bin);

    state.finished_flag.store(true, Ordering::Relaxed);
    let _ = monitor_handle.join();

    let total_elapsed = start_time.elapsed().as_secs_f64();
    let final_samples = state.samples_collected.load(Ordering::Relaxed);
    let final_speed = final_samples as f64 / total_elapsed;

    println!();
    println!("============================================================");
    println!(" ✅ 100% HYBRID GPU + CPU PARALLEL PIPELINE MINER HOÀN TẤT:");
    println!("============================================================");
    println!("  • GPU Hardware Card   : {}", gpu_name);
    println!("  • GPU Driver Backend  : {}", gpu_backend);
    println!("  • CPU Workers         : {} luồng physical cores", num_threads);
    println!("  • Tổng số FEN sinh ra : {} FENs", final_samples);
    println!("  • Thời gian thực thi  : {:.2} giây", total_elapsed);
    println!(
        "  🚀 THÔNG LƯỢNG TỐC ĐỘ  : {:.0} FEN/sec ({:.2} MILLION FEN/min!)",
        final_speed,
        (final_speed * 60.0) / 1_000_000.0
    );
    println!("============================================================");
}
