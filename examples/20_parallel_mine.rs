// ============================================================================
// EXAMPLE 20: BỘ MINING DỮ LIỆU ĐA LUỒNG REAL-TIME STREAMING & REAL-TIME DISK SAVE
// ============================================================================
// Vận hành N luồng công nhân (worker threads) chạy song song trên tất cả nhân CPU:
//   - Tự đấu Engine depth 4-6 với Random Opening 6 nước.
//   - Ghi dữ liệu trực tiếp vào data/selfplay_samples_gen3.jsonl theo THỜI GIAN THỰC.
//   - Hiển thị tiến độ, tốc độ (mẫu/giây), và thời gian hoàn thành (ETA) trực tiếp.
//
// Sử dụng: cargo run --release --example 20_parallel_mine
// Biến môi trường:
//   GAMES=100          Số ván cờ mục tiêu (mặc định 100)
//   DEPTH=4            Độ sâu tìm kiếm Engine (mặc định 4)
//   THREADS=8          Số luồng CPU song song (mặc định = physical cores)
//   SEED=1             Base seed cho PRNG (mặc định 1, dùng để chạy multi-instance)
//   OUTPUT=data/out.jsonl  Tên file output (mặc định data/selfplay_samples_gen6.jsonl)
// ============================================================================

use std::fs::OpenOptions;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use xiangrust::board::Parser;
use xiangrust::search::{Limits, Search};

/// Mẫu dữ liệu mined được
#[derive(Debug, Clone)]
struct Sample {
    fen: String,
    move_uci: String,
    score: i32,
    depth: u8,
}

fn main() {
    println!("============================================================");
    println!(" XIANGQI-RIM HIGH-THROUGHPUT PARALLEL DATA MINER (REAL-TIME)");
    println!("============================================================");

    let total_games: usize = std::env::var("GAMES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);
    let depth: u8 = std::env::var("DEPTH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4);
    // Mặc định = số physical cores (logical / 2) cho compute-bound workload
    // Trên i5-8259U: 8 logical → 4 physical — tránh HT contention
    let num_threads: usize = std::env::var("THREADS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| {
            let logical = thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
            std::cmp::max(1, logical / 2)
        });

    // Seed cơ sở cho PRNG — mỗi Colab instance dùng seed khác nhau
    let base_seed: u64 = std::env::var("SEED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    // Tên file output — tùy chỉnh cho multi-instance
    let out_file: String = std::env::var("OUTPUT")
        .unwrap_or_else(|_| "data/selfplay_samples_gen6.jsonl".to_string());
    println!("Cấu hình Mining Đa Luồng:");
    println!("  • Tổng số ván cờ: {}", total_games);
    println!("  • Độ sâu Search: Depth {}", depth);
    println!("  • Số luồng CPU: {} Worker Threads", num_threads);
    println!("  • Base Seed: {}", base_seed);
    println!("  • Ghi đĩa Real-Time: {}", out_file);
    println!();

    let games_completed = Arc::new(AtomicUsize::new(0));
    let samples_collected = Arc::new(AtomicUsize::new(0));
    let finished_flag = Arc::new(AtomicBool::new(false));
    
    // File handle dùng chung được bảo vệ bởi Mutex
    let file_mutex = Arc::new(Mutex::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&out_file)
            .expect("Không thể tạo tệp lưu trữ dữ liệu mining")
    ));

    let start_time = Instant::now();
    let mut handles = Vec::with_capacity(num_threads);

    for thread_id in 0..num_threads {
        let games_completed = Arc::clone(&games_completed);
        let samples_collected = Arc::clone(&samples_collected);
        let file_mutex = Arc::clone(&file_mutex);

        let handle = thread::spawn(move || {
            // TT Hash Table 4MB — fit gần L3 cache (6MB shared trên i5-8259U)
            // NGHIÊM CẤM dùng >= 8MB cho mining workload (gây DRAM thrashing)
            let mut search = Search::new(4);
            search.auto_load(); // Tự động nạp GPU NNUE weights nếu có

            let mut limits = Limits::new();
            limits.depth = depth;

            // Seed = base_seed × (thread_id + 1) — mỗi instance + thread có seed duy nhất
            let mut seed = base_seed * (thread_id as u64 + 1) * 123456789;

            while games_completed.load(Ordering::Relaxed) < total_games {
                let current_game = games_completed.fetch_add(1, Ordering::Relaxed);
                if current_game >= total_games {
                    break;
                }

                // 1. Tạo vị trí mở đầu: 50% Book Opening + 50% Random Opening
                let mut pos = Parser::parse(Parser::DEFAULT);
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                let use_book = (seed % 2) == 0;

                if use_book {
                    // Dùng Opening Book: đi theo sách khai cuộc đến khi hết
                    let mut book_steps = 0u8;
                    while book_steps < 12 {
                        if let Some(mv) = xiangrust::book::Book::probe(&pos) {
                            pos.apply(mv.from, mv.to);
                            book_steps += 1;
                        } else {
                            break;
                        }
                    }
                    // Sau khi hết sách, thêm 2-4 nước random để đa dạng hóa
                    let extra = 2 + (seed as usize % 3);
                    for _ in 0..extra {
                        let mut moves = xiangrust::movegen::List::new();
                        xiangrust::movegen::legal(&mut pos, &mut moves);
                        if moves.len() == 0 {
                            break;
                        }
                        seed ^= seed << 13;
                        seed ^= seed >> 7;
                        seed ^= seed << 17;
                        let idx = (seed as usize) % moves.len();
                        let m = moves.items[idx];
                        pos.apply(m.from, m.to);
                    }
                } else {
                    // Random opening thuần: 6 nước ngẫu nhiên (phương pháp cũ)
                    for _ in 0..6 {
                        let mut moves = xiangrust::movegen::List::new();
                        xiangrust::movegen::legal(&mut pos, &mut moves);
                        if moves.len() == 0 {
                            break;
                        }
                        seed ^= seed << 13;
                        seed ^= seed >> 7;
                        seed ^= seed << 17;
                        let idx = (seed as usize) % moves.len();
                        let m = moves.items[idx];
                        pos.apply(m.from, m.to);
                    }
                }

                // 2. Chơi 1 ván cờ và thu thập dữ liệu
                let mut steps = 0u32;
                let mut local_samples = Vec::with_capacity(64);

                while steps < 200 {
                    let fen = xiangrust::board::Serializer::export(&pos);
                    let result = search.go(&pos, &limits);

                    if !result.best.valid() {
                        break;
                    }

                    let move_uci = format!(
                        "{}{}{}{}",
                        (b'a' + (result.best.from % 9)) as char,
                        (b'0' + (9 - (result.best.from / 9))) as char,
                        (b'a' + (result.best.to % 9)) as char,
                        (b'0' + (9 - (result.best.to / 9))) as char
                    );

                    local_samples.push(Sample {
                        fen,
                        move_uci,
                        score: result.score,
                        depth,
                    });

                    if result.score.abs() > 29000 {
                        break;
                    }

                    pos.apply(result.best.from, result.best.to);
                    steps += 1;
                }

                // 3. Ghi trực tiếp xuống đĩa ngay sau mỗi ván!
                samples_collected.fetch_add(local_samples.len(), Ordering::Relaxed);
                if let Ok(mut file) = file_mutex.lock() {
                    for sample in &local_samples {
                        let line = format!(
                            "{{\"fen\":\"{}\",\"best_move\":\"{}\",\"score\":{},\"depth\":{}}}\n",
                            sample.fen, sample.move_uci, sample.score, sample.depth
                        );
                        let _ = file.write_all(line.as_bytes());
                    }
                    let _ = file.flush();
                }
            }
        });

        handles.push(handle);
    }

    // Luồng Monitor theo dõi và in Tiến Độ Real-time Streaming + ETA
    let monitor_games = Arc::clone(&games_completed);
    let monitor_samples = Arc::clone(&samples_collected);
    let monitor_flag = Arc::clone(&finished_flag);

    let monitor_handle = thread::spawn(move || {
        while !monitor_flag.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_secs(2));
            let done = monitor_games.load(Ordering::Relaxed);
            let samples = monitor_samples.load(Ordering::Relaxed);
            let elapsed_s = start_time.elapsed().as_secs_f64();
            let pct = (done * 100) / total_games;
            let speed_g = done as f64 / elapsed_s;
            let speed_s = samples as f64 / elapsed_s;
            let rem_g = if total_games > done { total_games - done } else { 0 };
            let eta_s = if speed_g > 0.0 { (rem_g as f64 / speed_g).round() as u64 } else { 0 };
            let eta_m = eta_s / 60;
            let eta_sec = eta_s % 60;

            println!(
                "  [MINING STREAMING {:3}/{:3}] ({:2}%) | Samples: {:5} | Speed: {:.1} g/s ({:.0} FEN/min) | ETA: {:02}m{:02}s",
                done.min(total_games), total_games, pct.min(100), samples, speed_g, speed_s * 60.0, eta_m, eta_sec
            );
            let _ = std::io::stdout().flush();
        }
    });

    // Đợi tất cả worker threads hoàn tất
    for h in handles {
        let _ = h.join();
    }
    finished_flag.store(true, Ordering::Relaxed);
    let _ = monitor_handle.join();

    let elapsed = start_time.elapsed();
    let total_g = games_completed.load(Ordering::Relaxed).min(total_games);
    let total_s = samples_collected.load(Ordering::Relaxed);
    let speed_g = total_g as f64 / elapsed.as_secs_f64();
    let speed_s = total_s as f64 / elapsed.as_secs_f64();

    println!("============================================================");
    println!("✅ MINING ĐA LUỒNG HOÀN TẤT TRONG {:.2} GIÂY!", elapsed.as_secs_f64());
    println!("============================================================");
    println!("  • Tổng số ván cờ: {} ván", total_g);
    println!("  • Mẫu dữ liệu trích xuất: {} mẫu FEN", total_s);
    println!("  • Tệp lưu trữ đĩa: {} (Dung lượng: {:.2} MB)", &out_file, std::fs::metadata(&out_file).map(|m| m.len() as f64 / (1024.0 * 1024.0)).unwrap_or(0.0));
    println!("  • Tốc độ ván cờ: {:.1} ván/giây", speed_g);
    println!("  • Tốc độ mẫu FEN: {:.1} mẫu/giây ({:.0} mẫu/phút)", speed_s, speed_s * 60.0);
    println!("============================================================");
}
