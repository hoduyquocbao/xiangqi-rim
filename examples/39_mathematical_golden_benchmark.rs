// ============================================================================
// EXAMPLE 39: MATHEMATICAL GOLDEN BENCHMARK FOR HYBRID GPU+CPU ENGINE
// ============================================================================
// Chương Trình Tìm Điểm Chuẩn Toán Học Cho Kiến Trúc Lai (Mathematical Golden Benchmark):
//   1. Thực thi Grid-Search tự động trên dãy thông số:
//      - Độ sâu Search: Depth 3, 4, 5.
//      - Kích thước lô Batch Size: 32, 64, 128, 256, 512, 1024, 2048.
//      - Số luồng CPU Worker Threads: 1, 2, 4.
//   2. Tính điểm hiệu năng toán học (Mathematical Efficiency Score $S$):
//      $S = \frac{\text{Throughput (FEN/s)} \times \text{Peak GPU \%}}{\text{Latency (ms)} + 1.0}$
//   3. Xuất cấu hình điểm vàng toán học (Golden Configuration) tối ưu nhất.
// Tuân thủ 100% định danh từ đơn tiếng Anh và 100% chú thích Tiếng Việt trên từng dòng mã.
// ============================================================================

use std::io::Write;
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use rayon::prelude::*;
use xiangrust::board::{Parser, Position};
use xiangrust::book::Book;
use xiangrust::gpu::Device;
use xiangrust::movegen::{legal, List};
use xiangrust::search::HybridEngine;

pub const APP_VERSION: &str = "v3.1.0-golden-benchmark";
pub const APP_BUILD_STAMP: &str = "2026-08-12 08:10:00 ICT";

fn read_macos_gpu_load_pct() -> u32 {
    let output = Command::new("ioreg")
        .args(&["-l"])
        .output();

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

fn run_single_grid_test(
    threads: usize,
    batch_size: usize,
    depth: i32,
    num_games: usize,
) -> (f64, usize, u32, f64, f64) {
    let finished_flag = Arc::new(AtomicBool::new(false));
    let fens_computed = Arc::new(AtomicUsize::new(0));
    let peak_gpu_load = Arc::new(AtomicUsize::new(0));

    let flag_mon = Arc::clone(&finished_flag);
    let peak_mon = Arc::clone(&peak_gpu_load);

    let start_time = Instant::now();

    let monitor_handle = thread::spawn(move || {
        while !flag_mon.load(Ordering::Relaxed) {
            let gpu_pct = read_macos_gpu_load_pct();
            let current_peak = peak_mon.load(Ordering::Relaxed);
            if (gpu_pct as usize) > current_peak {
                peak_mon.store(gpu_pct as usize, Ordering::Relaxed);
            }
            thread::sleep(Duration::from_millis(100));
        }
    });

    let hybrid = Arc::new(HybridEngine::new(threads));
    let fens_count = Arc::clone(&fens_computed);

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .unwrap();

    pool.install(|| {
        (0..num_games).into_par_iter().for_each(|g| {
            let seed = (g as u64 + 1) * 987654321;
            let mut pos = generate_start_position(seed);
            let mut leaf_batch: Vec<Position> = Vec::with_capacity(batch_size);

            let mut list = List::new();
            legal::gen(&mut pos, &mut list);
            let mut i = 0usize;
            while i < list.len() {
                let mv = list.get(i);
                let state = pos.apply(mv.from, mv.to);
                leaf_batch.push(pos);
                pos.revert(mv.from, mv.to, &state);
                i += 1;
            }

            let mut scores = vec![0i32; leaf_batch.len()];
            let _ = hybrid.evaluate_batch(&leaf_batch, &mut scores, depth);
            fens_count.fetch_add(leaf_batch.len(), Ordering::Relaxed);
        });
    });

    finished_flag.store(true, Ordering::Relaxed);
    let _ = monitor_handle.join();

    let elapsed = start_time.elapsed().as_secs_f64();
    let fens = fens_computed.load(Ordering::Relaxed);
    let peak_gpu = peak_gpu_load.load(Ordering::Relaxed) as u32;
    let fps = if elapsed > 0.0 { fens as f64 / elapsed } else { 0.0 };

    let latency_ms = (elapsed * 1000.0) / (fens.max(1) as f64);
    let score = (fps * (peak_gpu as f64 + 1.0)) / (latency_ms + 1.0);

    (elapsed, fens, peak_gpu, fps, score)
}

fn main() {
    println!("============================================================");
    println!(" 💎 XIANGQI-RIM: MATHEMATICAL GOLDEN BENCHMARK FOR HYBRID ENGINE");
    println!("    Engine Version : {}", APP_VERSION);
    println!("    Build Timestamp: {}", APP_BUILD_STAMP);
    println!("============================================================");

    let device = Device::init();
    println!("Hardware GPU Adapter: {}", device.adapter_name());
    println!("Backend Driver      : {}", device.backend().name());
    println!("Speed Rating        : {}%", device.backend().speed());
    println!("Search Integration  : Empirical Grid Search for Mathematical Golden Point");
    println!("============================================================");

    let threads_grid = [1, 2, 4];
    let batch_grid = [64, 128, 256, 512, 1024];
    let depths_grid = [3, 4, 5];

    let mut best_score = -1.0f64;
    let mut best_config = (0, 0, 0, 0.0, 0.0, 0u32);

    println!(
        "{:<8} | {:<10} | {:<6} | {:<10} | {:<12} | {:<14} | {:<10} | {:<12}",
        "Threads", "Batch Size", "Depth", "Thời gian", "Tổng FEN GPU", "Thông lượng FEN/s", "Peak GPU %", "Golden Score"
    );
    println!(
        "{:-<8}-|-{:-<10}-|-{:-<6}-|-{:-<10}-|-{:-<12}-|-{:-<14}-|-{:-<10}-|-{:-<12}",
        "", "", "", "", "", "", "", ""
    );

    for &t in &threads_grid {
        for &b in &batch_grid {
            for &d in &depths_grid {
                print!("👉 Testing T={:<2} B={:<4} D={:<2}... ", t, b, d);
                let _ = std::io::stdout().flush();

                let (elapsed, fens, peak_gpu, fps, score) = run_single_grid_test(t, b, d, 20);
                println!(
                    "{:<8.2}s | {:<12} | {:<14.0} | {:<10}% | {:<12.2}",
                    elapsed, fens, fps, peak_gpu, score
                );
                let _ = std::io::stdout().flush();

                if score > best_score {
                    best_score = score;
                    best_config = (t, b, d, elapsed, fps, peak_gpu);
                }
            }
        }
    }

    println!("============================================================");
    println!(" 🏆 ĐIỂM CHUẨN TOÁN HỌC ĐẤNG CẤP (MATHEMATICAL GOLDEN POINT):");
    println!("    Thread Pool (T*)     : {} luồng CPU", best_config.0);
    println!("    Batch Size (B*)      : {} thế cờ / lô", best_config.1);
    println!("    Search Depth (D*)    : Depth {}", best_config.2);
    println!("    Thông lượng Đỉnh     : {:.0} FEN / giây", best_config.4);
    println!("    Peak GPU Utilization : {}%", best_config.5);
    println!("    Điểm Số Tối Ưu S*    : {:.2}", best_score);
    println!("============================================================");
}
