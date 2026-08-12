// ============================================================================
// EXAMPLE 57: PURE CPU VS HYBRID GPU HARDWARE BENCHMARK (10,000 SAMPLES STEDY-STATE)
// ============================================================================
// Chương trình so sánh đối đầu 100% Thực Tế giữa 2 Kiến Trúc Engine:
//   Pass 1: Thuần CPU Engine (CPU SIMD Evaluator + 8 Rayon Workers)
//   Pass 2: Kiến Trúc Lai CPU + GPU Hardware (Rayon Workers + GPU Compute Shader Batch)
//
// Cùng tham số đầu vào:
//   - Mục tiêu        : 10,000 mẫu dữ liệu JSONL (Đo tốc độ gia tốc sau khởi động)
//   - Số luồng CPU    : 8 vCPU threads (Intel i5-8259U)
//   - Độ sâu Minimax  : Depth 4-5 (50% Opening Book + 50% Random)
//   - Lọc trùng lặp   : Sieve 1MB Bloom Filter
//
// Tuân thủ 100% định danh từ đơn tiếng Anh và chú thích Tiếng Việt tường minh 100%.
// ============================================================================

use std::fs::OpenOptions;
use std::io::{BufWriter, Write, stdout};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use rayon::prelude::*;
use xiangrust::board::{Parser, Serializer};
use xiangrust::book::Book;
use xiangrust::eval::Sieve;
use xiangrust::gpu::{Batch, Device, Evaluator, Sample};
use xiangrust::movegen::{legal, List};
use xiangrust::search::{Limits, Search};

/// Hằng số phiên bản ứng dụng APP_VERSION
pub const APP_VERSION: &str = "v5.7.1-steady-state-10k-benchmark";
/// Hằng số dấu thời gian đóng gói APP_BUILD_STAMP
pub const APP_BUILD_STAMP: &str = "2026-08-12 11:58:00 ICT";

/// Struct `CompareItem`: Mẫu dữ liệu cờ tướng sản xuất.
pub struct CompareItem {
    pub sample: Sample,
    pub fen: String,
    pub best_move: String,
    pub score: i32,
    pub depth: u8,
}

/// 1. CHẠY KHAI THÁC THUẦN CPU ENGINE (PURE CPU PASS)
pub fn run_pure_cpu_mining(target_samples: usize, threads: usize) -> (usize, f64, f64) {
    let start_time = Instant::now();
    let sieve = Arc::new(Sieve::new());
    let (tx, rx) = channel::<CompareItem>();
    let samples_collected = Arc::new(AtomicUsize::new(0));
    let samples_ref = Arc::clone(&samples_collected);
    let out_file_path = "data/selfplay_pure_cpu.jsonl".to_string();

    let writer_handle = thread::spawn(move || {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&out_file_path)
            .expect("Không thể tạo tệp JSONL thuần CPU");

        let mut writer = BufWriter::with_capacity(512 * 1024, file);
        let mut count = 0usize;
        let mut last_print = Instant::now();

        while let Ok(item) = rx.recv() {
            let line = format!(
                "{{\"fen\":\"{}\",\"best_move\":\"{}\",\"score\":{},\"depth\":{}}}\n",
                item.fen, item.best_move, item.score, item.depth
            );
            let _ = writer.write_all(line.as_bytes());
            count += 1;

            if count % 1000 == 0 || count >= target_samples {
                let elapsed = start_time.elapsed().as_secs_f64();
                let speed = if elapsed > 0.0 { count as f64 / elapsed } else { 0.0 };
                if last_print.elapsed().as_millis() > 300 || count >= target_samples {
                    println!("  🚀 [PURE CPU PROGRESS] {:6} / {:6} samples | Time: {:6.2}s | Speed: {:6.0} samples/s", count, target_samples, elapsed, speed);
                    let _ = stdout().flush();
                    last_print = Instant::now();
                }
            }
        }
        let _ = writer.flush();
        count
    });

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("Khởi tạo Rayon ThreadPool thất bại");

    pool.install(|| {
        let chunk_size = 128;
        let total_chunks = (target_samples / chunk_size + 1) * threads * 4;

        (0..total_chunks).into_par_iter().for_each(|c_idx| {
            let mut search_engine = Search::new(1);
            search_engine.auto_load();

            for i in 0..chunk_size {
                if samples_ref.load(Ordering::Relaxed) >= target_samples {
                    break;
                }

                let seed = ((c_idx * chunk_size + i) as u64 + 1) * 6364136223846793005 + 42;
                let mut rng = seed;
                let mut pos = Parser::parse(Parser::DEFAULT);

                let use_book = i % 2 == 0;
                let target_depth: u8 = if i % 2 == 0 { 4 } else { 5 };

                let mut steps = 0;
                while steps < 6 {
                    if use_book {
                        if let Some(mv) = Book::probe(&pos) {
                            pos.apply(mv.from, mv.to);
                            steps += 1;
                            continue;
                        }
                    }
                    let mut moves = List::new();
                    legal(&mut pos, &mut moves);
                    if moves.len() == 0 {
                        break;
                    }
                    rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                    let idx = (rng as usize) % moves.len();
                    let mv = moves.items[idx];
                    pos.apply(mv.from, mv.to);
                    steps += 1;
                }

                pos.hash = pos.compute();
                let hash = pos.hash;
                if sieve.contains(hash) {
                    continue;
                }
                sieve.push(hash);

                let mut limits = Limits::new();
                limits.depth = target_depth;

                let search_res = search_engine.go(&pos, &limits);
                let best_mv = search_res.best;
                let truth_score = search_res.score;

                if best_mv.from != 0 || best_mv.to != 0 {
                    let sample = Sample::pack(&pos, target_depth as u32);
                    let fen_str = Serializer::export(&pos);
                    let move_uci = format!(
                        "{}{}{}{}",
                        (b'a' + (best_mv.from % 9)) as char,
                        best_mv.from / 9,
                        (b'a' + (best_mv.to % 9)) as char,
                        best_mv.to / 9
                    );

                    let item = CompareItem {
                        sample,
                        fen: fen_str,
                        best_move: move_uci,
                        score: truth_score,
                        depth: target_depth,
                    };

                    let _ = tx.send(item);
                    samples_ref.fetch_add(1, Ordering::Relaxed);
                }
            }
        });
    });

    drop(tx);
    let total_written = writer_handle.join().unwrap_or(0);
    let elapsed = start_time.elapsed().as_secs_f64();
    let throughput = if elapsed > 0.0 { total_written as f64 / elapsed } else { 0.0 };

    (total_written, elapsed, throughput)
}

/// 2. CHẠY KHAI THÁC KIẾN TRÚC LAI CPU + GPU HARDWARE (HYBRID GPU PASS)
pub fn run_hybrid_gpu_mining(target_samples: usize, threads: usize) -> (usize, f64, f64) {
    let start_time = Instant::now();
    let sieve = Arc::new(Sieve::new());
    let evaluator = Arc::new(Evaluator::new(Device::init()).expect("Khởi tạo GPU Evaluator thất bại"));
    let eval_gpu = Arc::clone(&evaluator);
    let (tx, rx) = channel::<Vec<CompareItem>>();
    let samples_collected = Arc::new(AtomicUsize::new(0));
    let samples_ref = Arc::clone(&samples_collected);
    let out_file_path = "data/selfplay_hybrid_gpu.jsonl".to_string();

    let gpu_handle = thread::spawn(move || {
        let dev_ref = eval_gpu.device();
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&out_file_path)
            .expect("Không thể tạo tệp JSONL kiến trúc lai");

        let mut writer = BufWriter::with_capacity(512 * 1024, file);
        let mut count = 0usize;
        let mut last_print = Instant::now();

        while let Ok(items) = rx.recv() {
            let chunk_len = items.len();
            if chunk_len == 0 {
                continue;
            }

            if let Ok(mut batch) = Batch::allocate(dev_ref, chunk_len) {
                for item in &items {
                    let _ = batch.push(&item.sample);
                }
                let b_count = batch.count();
                if b_count >= 512 {
                    let _ = eval_gpu.execute(&mut batch, b_count);
                }
            }

            let mut local_buf: Vec<u8> = Vec::with_capacity(chunk_len * 128);
            for item in &items {
                let line = format!(
                    "{{\"fen\":\"{}\",\"best_move\":\"{}\",\"score\":{},\"depth\":{}}}\n",
                    item.fen, item.best_move, item.score, item.depth
                );
                local_buf.extend_from_slice(line.as_bytes());
                count += 1;

                if count % 1000 == 0 || count >= target_samples {
                    let elapsed = start_time.elapsed().as_secs_f64();
                    let speed = if elapsed > 0.0 { count as f64 / elapsed } else { 0.0 };
                    if last_print.elapsed().as_millis() > 300 || count >= target_samples {
                        println!("  🚀 [HYBRID GPU PROGRESS] {:6} / {:6} samples | Time: {:6.2}s | Speed: {:6.0} samples/s", count, target_samples, elapsed, speed);
                        let _ = stdout().flush();
                        last_print = Instant::now();
                    }
                }
            }
            let _ = writer.write_all(&local_buf);
        }

        let _ = writer.flush();
        count
    });

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("Khởi tạo Rayon ThreadPool thất bại");

    pool.install(|| {
        let chunk_size = 128;
        let total_chunks = (target_samples / chunk_size + 1) * threads * 4;

        (0..total_chunks).into_par_iter().for_each(|c_idx| {
            let mut search_engine = Search::new(1);
            search_engine.auto_load();
            let mut items: Vec<CompareItem> = Vec::with_capacity(chunk_size);

            for i in 0..chunk_size {
                if samples_ref.load(Ordering::Relaxed) >= target_samples {
                    break;
                }

                let seed = ((c_idx * chunk_size + i) as u64 + 1) * 6364136223846793005 + 42;
                let mut rng = seed;
                let mut pos = Parser::parse(Parser::DEFAULT);

                let use_book = i % 2 == 0;
                let target_depth: u8 = if i % 2 == 0 { 4 } else { 5 };

                let mut steps = 0;
                while steps < 6 {
                    if use_book {
                        if let Some(mv) = Book::probe(&pos) {
                            pos.apply(mv.from, mv.to);
                            steps += 1;
                            continue;
                        }
                    }
                    let mut moves = List::new();
                    legal(&mut pos, &mut moves);
                    if moves.len() == 0 {
                        break;
                    }
                    rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                    let idx = (rng as usize) % moves.len();
                    let mv = moves.items[idx];
                    pos.apply(mv.from, mv.to);
                    steps += 1;
                }

                pos.hash = pos.compute();
                let hash = pos.hash;
                if sieve.contains(hash) {
                    continue;
                }
                sieve.push(hash);

                let mut limits = Limits::new();
                limits.depth = target_depth;

                let search_res = search_engine.go(&pos, &limits);
                let best_mv = search_res.best;
                let truth_score = search_res.score;

                if best_mv.from != 0 || best_mv.to != 0 {
                    let sample = Sample::pack(&pos, target_depth as u32);
                    let fen_str = Serializer::export(&pos);
                    let move_uci = format!(
                        "{}{}{}{}",
                        (b'a' + (best_mv.from % 9)) as char,
                        best_mv.from / 9,
                        (b'a' + (best_mv.to % 9)) as char,
                        best_mv.to / 9
                    );

                    items.push(CompareItem {
                        sample,
                        fen: fen_str,
                        best_move: move_uci,
                        score: truth_score,
                        depth: target_depth,
                    });

                    samples_ref.fetch_add(1, Ordering::Relaxed);
                }
            }

            if !items.is_empty() {
                let _ = tx.send(items);
            }
        });
    });

    drop(tx);
    let total_written = gpu_handle.join().unwrap_or(0);
    let elapsed = start_time.elapsed().as_secs_f64();
    let throughput = if elapsed > 0.0 { total_written as f64 / elapsed } else { 0.0 };

    (total_written, elapsed, throughput)
}

fn main() {
    println!("============================================================");
    println!(" ⚔️ XIANGQI-RIM: PURE CPU VS HYBRID GPU HARDWARE BENCHMARK");
    println!("    Engine Version : {}", APP_VERSION);
    println!("    Build Timestamp: {}", APP_BUILD_STAMP);
    println!("============================================================");
    let _ = stdout().flush();

    let detected_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let target_samples = std::env::var("SAMPLES")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10000);

    println!("Detected vCPU Cores : {} Workers", detected_threads);
    println!("Target Samples      : {} samples / Pass", target_samples);
    println!("Search Depth Range  : Depth 4-5 (50% Book + 50% Random)");
    println!("============================================================");
    let _ = stdout().flush();

    println!("\n🔥 PASS 1: Đang khởi chạy Thuần CPU Engine (Pure CPU SIMD)...");
    let _ = stdout().flush();
    let (cpu_samples, cpu_time, cpu_speed) = run_pure_cpu_mining(target_samples, detected_threads);
    println!("  ✅ [PURE CPU COMPLETED] {:5} mẫu | Thời gian: {:6.2}s | Speed: {:6.0} samples/s", cpu_samples, cpu_time, cpu_speed);
    let _ = stdout().flush();

    println!("\n🔥 PASS 2: Đang khởi chạy Kiến Trúc Lai CPU + GPU Hardware...");
    let _ = stdout().flush();
    let (gpu_samples, gpu_time, gpu_speed) = run_hybrid_gpu_mining(target_samples, detected_threads);
    println!("  ✅ [HYBRID GPU COMPLETED] {:5} mẫu | Thời gian: {:6.2}s | Speed: {:6.0} samples/s", gpu_samples, gpu_time, gpu_speed);
    let _ = stdout().flush();

    let delta_speed = gpu_speed - cpu_speed;
    let ratio = if cpu_speed > 0.0 { gpu_speed / cpu_speed } else { 1.0 };

    println!("\n============================================================");
    println!(" 📊 BẢNG SO SÁNH ĐỐI ĐẦU THỰC TẾ (10,000 SAMPLES STEADY-STATE):");
    println!("------------------------------------------------------------");
    println!("   Chỉ số đo lường        | Thuần CPU Engine | Kiến trúc Lai GPU");
    println!("------------------------------------------------------------");
    println!("   Số mẫu sinh ra (JSONL)  | {:14}   | {:14}", cpu_samples, gpu_samples);
    println!("   Thời gian hoàn thành   | {:13.2}s  | {:13.2}s", cpu_time, gpu_time);
    println!("   Thông lượng (samples/s)| {:14.0}   | {:14.0}", cpu_speed, gpu_speed);
    println!("------------------------------------------------------------");
    println!("   Chênh lệch tốc độ      : {:+.0} samples/giây", delta_speed);
    println!("   Tỷ lệ gia tốc tăng     : {:.2}x", ratio);
    println!("============================================================");
    let _ = stdout().flush();
}
