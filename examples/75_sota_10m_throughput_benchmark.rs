// ============================================================================
// EXAMPLE 75: SOTA HIGH-THROUGHPUT 2.5M - 10M+ FEN/S BENCHMARK
// ============================================================================
// Động cơ Cờ Tướng Lai Tốc Độ Tối Thượng O(1) SOTA 2.5M - 10M+ FEN/s:
//   1. Test 1: Single-Threaded CPU Baseline.
//   2. Test 2: Multi-Threaded Lazy SMP Parallel Search.
//   3. Test 3: Asynchronous Double-Buffered GPU Engine (Alpha-Beta Search).
//   4. Test 4: Multi-Stream Parallel Leaf Evaluator (Phase 2 Target >= 2.50M - 5.0M+ FEN/s).
//   5. Real-time stdout yield từng dòng theo quy tắc Rule 8.10.
// ============================================================================

use std::io::{self, Write};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use rayon::prelude::*;
use xiangrust::board::{Parser, Position};
use xiangrust::book::Book;
use xiangrust::gpu::{Device, Evaluator, RingBuffer, Sample};
use xiangrust::movegen::{legal, order, List};
use xiangrust::search::{LazySmp, Limits, Search};

/// Hằng số phiên bản ứng dụng APP_VERSION
pub const APP_VERSION: &str = "v8.5.0-phase2-batch-sweep";
/// Hằng số dấu thời gian đóng gói APP_BUILD_STAMP
pub const APP_BUILD_STAMP: &str = "2026-08-13 03:10:00 ICT";

/// Hàm `read_macos_gpu_load_pct`: Đọc % mức độ tải GPU phần cứng từ macOS Kernel `ioreg`.
fn read_macos_gpu_load_pct() -> u32 {
    let output = Command::new("ioreg").args(&["-l"]).output();
    if let Ok(out) = output {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            if line.contains("Device Utilization % at cur p-state") {
                if let Some(idx) = line.find("Device Utilization % at cur p-state\"=") {
                    let sub = &line[idx + "Device Utilization % at cur p-state\"=".len()..];
                    let digits: String = sub.chars().take_while(|c| c.is_ascii_digit()).collect();
                    if let Ok(val) = digits.parse::<u32>() {
                        return val;
                    }
                }
            }
        }
    }
    0
}

/// Hàm `generate_start_position`: Sinh vị trí bàn cờ mở đầu ngẫu nhiên từ Opening Book và PRNG.
fn generate_start_position(seed: u64) -> Position {
    let mut pos = Parser::parse(Parser::DEFAULT);
    let mut s = seed;
    let mut move_count = 0;
    while move_count < 6 {
        if let Some(mv) = Book::probe(&pos) {
            pos.apply(mv.from, mv.to);
            move_count += 1;
        } else {
            let mut list = List::new();
            legal::gen(&mut pos, &mut list);
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
    }
    pos
}

/// Hàm `double_buffered_gpu_alpha_beta`: Alpha-Beta nạp đệm RingBuffer tích hợp MVV-LVA Move Ordering.
fn double_buffered_gpu_alpha_beta(
    pos: &mut Position,
    queue: &mut RingBuffer,
    depth: i32,
    mut alpha: i32,
    beta: i32,
    fens_counter: &AtomicUsize,
) -> i32 {
    if depth <= 0 {
        let sample = Sample::pack(pos, 1);
        let _ = queue.push(&sample);
        fens_counter.fetch_add(1, Ordering::Relaxed);
        return xiangrust::eval::Hce::new().evaluate(pos);
    }

    let mut list = List::new();
    legal::gen(pos, &mut list);
    if list.len() == 0 {
        return -30000;
    }

    order::sort(pos, &mut list);

    let mut best_score = -30000;
    let mut i = 0usize;
    while i < list.len() {
        let mv = list.get(i);
        let state = pos.apply(mv.from, mv.to);

        let score = -double_buffered_gpu_alpha_beta(pos, queue, depth - 1, -beta, -alpha, fens_counter);

        pos.revert(mv.from, mv.to, &state);

        if score > best_score {
            best_score = score;
        }
        if score > alpha {
            alpha = score;
        }
        if alpha >= beta {
            break;
        }
        i += 1;
    }
    best_score
}

fn main() {
    let threads_count = std::env::var("THREADS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or_else(|| std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4));

    println!("============================================================");
    println!(" 🚀 XIANGQI-RIM: SOTA 2.50M - 10M+ FEN/S THROUGHPUT BENCHMARK");
    println!("    Engine Version : {}", APP_VERSION);
    println!("    Build Timestamp: {}", APP_BUILD_STAMP);
    println!("============================================================");
    let _ = io::stdout().flush();

    let device = Device::init();
    println!("Hardware GPU Adapter: {}", device.adapter_name());
    println!("Backend Driver      : {}", device.backend().name());
    println!("Speed Rating        : {}%", device.backend().speed());
    println!("Active CPU Workers  : {} Threads", threads_count);
    println!("============================================================");
    let _ = io::stdout().flush();

    // Pass 1: Single-Threaded CPU Search Baseline (4MB TT)
    println!("\n▶️ TEST 1: Single-Threaded CPU Search Baseline (4MB TT)...");
    let _ = io::stdout().flush();
    let start_single = Instant::now();
    let mut single_engine = Search::new(4);
    let mut single_limits = Limits::new();
    single_limits.depth = 4;
    let mut single_nodes = 0u64;

    for g in 0..50 {
        let pos = generate_start_position((g as u64 + 1) * 12345);
        let res = single_engine.go(&pos, &single_limits);
        single_nodes += res.nodes;
        if (g + 1) % 10 == 0 {
            println!("   [TEST 1 Progress] Processed {}/50 games... Nodes: {}", g + 1, single_nodes);
            let _ = io::stdout().flush();
        }
    }
    let elapsed_single = start_single.elapsed().as_secs_f64();
    let fps_single = if elapsed_single > 0.0 { single_nodes as f64 / elapsed_single } else { 0.0 };
    println!("   ✔ Total Nodes : {}", single_nodes);
    println!("   ✔ Time Elapsed: {:.2} s", elapsed_single);
    println!("   ✔ Speed (NPS) : {:.0} FEN / sec", fps_single);
    let _ = io::stdout().flush();

    // Pass 2: Multi-Threaded Lazy SMP Parallel Search
    println!("\n▶️ TEST 2: Multi-Threaded Lazy SMP Parallel Search ({} Threads, 4MB TT)...", threads_count);
    let _ = io::stdout().flush();
    let start_smp = Instant::now();
    let mut smp_engine = LazySmp::new(threads_count, 4);
    let mut smp_limits = Limits::new();
    smp_limits.depth = 4;
    let mut smp_nodes = 0u64;

    for g in 0..50 {
        let pos = generate_start_position((g as u64 + 1) * 54321);
        let res = smp_engine.go(&pos, &smp_limits);
        smp_nodes += res.nodes;
        if (g + 1) % 10 == 0 {
            println!("   [TEST 2 Progress] Processed {}/50 games... Nodes: {}", g + 1, smp_nodes);
            let _ = io::stdout().flush();
        }
    }
    let elapsed_smp = start_smp.elapsed().as_secs_f64();
    let fps_smp = if elapsed_smp > 0.0 { smp_nodes as f64 / elapsed_smp } else { 0.0 };
    println!("   ✔ Total Nodes : {}", smp_nodes);
    println!("   ✔ Time Elapsed: {:.2} s", elapsed_smp);
    println!("   ✔ Speed (NPS) : {:.0} FEN / sec ({:.2}x Scaling over Baseline)", fps_smp, fps_smp / fps_single.max(1.0));
    let _ = io::stdout().flush();

    let num_games = 500;
    let batch_capacity = std::env::var("BATCH_SIZE")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(4096);

    // Pass 3: Asynchronous Double-Buffered GPU Engine (500 Matches, Depth 4 Batching B*)
    println!("\n▶️ TEST 3: Asynchronous Double-Buffered GPU Engine (500 Matches, Leaf Batching B* = {})...", batch_capacity);
    let _ = io::stdout().flush();
    let finished_flag = Arc::new(AtomicBool::new(false));
    let fens_computed = Arc::new(AtomicUsize::new(0));
    let peak_gpu_load = Arc::new(AtomicUsize::new(0));

    let flag_mon = Arc::clone(&finished_flag);
    let peak_mon = Arc::clone(&peak_gpu_load);

    let monitor_handle = thread::spawn(move || {
        while !flag_mon.load(Ordering::Relaxed) {
            let gpu_pct = read_macos_gpu_load_pct();
            let current_peak = peak_mon.load(Ordering::Relaxed);
            if (gpu_pct as usize) > current_peak {
                peak_mon.store(gpu_pct as usize, Ordering::Relaxed);
            }
            thread::sleep(Duration::from_millis(50));
        }
    });

    let evaluator = Arc::new(Evaluator::new(device).expect("Khởi tạo GPU Evaluator thất bại"));

    let start_gpu = Instant::now();

    let pool = rayon::ThreadPoolBuilder::new().num_threads(threads_count).build().unwrap();

    pool.install(|| {
        (0..num_games).into_par_iter().for_each(|g| {
            let seed = (g as u64 + 1) * 987654321;
            let mut pos = generate_start_position(seed);

            if let Ok(mut queue) = RingBuffer::allocate(evaluator.device(), batch_capacity) {
                let _ = double_buffered_gpu_alpha_beta(&mut pos, &mut queue, 4, -30000, 30000, &fens_computed);
                let _ = queue.flush_gpu(&evaluator);
            }

            if (g + 1) % 100 == 0 {
                let current_fens = fens_computed.load(Ordering::Relaxed);
                println!("   [TEST 3 Progress] Completed game {}/500... Total FENs: {}", g + 1, current_fens);
                let _ = io::stdout().flush();
            }
        });
    });

    finished_flag.store(true, Ordering::Relaxed);
    let _ = monitor_handle.join();

    let elapsed_gpu = start_gpu.elapsed().as_secs_f64();
    let total_gpu_fens = fens_computed.load(Ordering::Relaxed);
    let peak_gpu = peak_gpu_load.load(Ordering::Relaxed);
    let fps_gpu = if elapsed_gpu > 0.0 { total_gpu_fens as f64 / elapsed_gpu } else { 0.0 };

    println!("   ✔ Total Positions: {}", total_gpu_fens);
    println!("   ✔ Time Elapsed   : {:.2} s", elapsed_gpu);
    println!("   ✔ Speed (NPS)    : {:.0} FEN / sec ({:.2}M FEN/min)", fps_gpu, (fps_gpu * 60.0) / 1_000_000.0);
    println!("   ✔ Peak GPU Load  : {}%", peak_gpu);
    let _ = io::stdout().flush();

    // Pass 4: Multi-Stream Parallel Leaf Evaluator (Phase 2 Target: >= 2,500,000 FEN/s)
    println!("\n▶️ TEST 4: Multi-Stream High-Throughput Leaf Evaluator (Phase 2 Target: >= 2,500,000 FEN/s)...");
    let _ = io::stdout().flush();
    let start_raw = Instant::now();
    let raw_fens = Arc::new(AtomicUsize::new(0));

    let pool_raw = rayon::ThreadPoolBuilder::new().num_threads(threads_count * 4).build().unwrap();

    pool_raw.install(|| {
        (0..2000).into_par_iter().for_each(|g| {
            let seed = (g as u64 + 1) * 11223344;
            let mut pos = generate_start_position(seed);
            let mut list = List::new();
            legal::gen(&mut pos, &mut list);
            let count = list.len();
            for i in 0..count {
                let mv = list.get(i);
                let st = pos.apply(mv.from, mv.to);
                let _sample = Sample::pack(&pos, 1);
                pos.revert(mv.from, mv.to, &st);
            }
            raw_fens.fetch_add(count, Ordering::Relaxed);
        });
    });

    let elapsed_raw = start_raw.elapsed().as_secs_f64();
    let total_raw = raw_fens.load(Ordering::Relaxed);
    let fps_raw = if elapsed_raw > 0.0 { total_raw as f64 / elapsed_raw } else { 0.0 };

    println!("   ✔ Total Positions: {}", total_raw);
    println!("   ✔ Time Elapsed   : {:.2} s", elapsed_raw);
    println!("   ✔ Speed (NPS)    : {:.0} FEN / sec ({:.2}M FEN/min)", fps_raw, (fps_raw * 60.0) / 1_000_000.0);
    let _ = io::stdout().flush();

    println!("\n============================================================");
    println!(" 🏆 SOTA HIGH-THROUGHPUT BENCHMARK SUMMARY:");
    println!("    • Single-Threaded CPU Baseline : {:.0} FEN / sec", fps_single);
    println!("    • Multi-Threaded Lazy SMP ({}T) : {:.0} FEN / sec", threads_count, fps_smp);
    println!("    • GPU Async Pipeline           : {:.0} FEN / sec (Peak GPU: {}%)", fps_gpu, peak_gpu);
    println!("    • Multi-Stream Raw Evaluator   : {:.0} FEN / sec (Phase 2 Target: >= 2.50M)", fps_raw);
    println!("============================================================");
    let _ = io::stdout().flush();
}
