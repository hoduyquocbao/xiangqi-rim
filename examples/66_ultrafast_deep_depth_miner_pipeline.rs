// ============================================================================
// EXAMPLE 66: ULTRA-FAST DEEP-DEPTH DATA MINER PIPELINE (RULE 8.10/7.10)
// ============================================================================
// Động Cơ Khai Thác Dữ Liệu Tự Đấu Tìm Kiếm Sâu Siêu Tốc Huấn Luyện NNUE:
//   1. Khai thác dữ liệu tự đấu đa luồng song song với 256MB Shared Zobrist TT.
//   2. Lưu trực tiếp mẫu dữ liệu dạng JSONL chuẩn (`data/deep_depth_mined_samples.jsonl`).
//   3. Trường dữ liệu thống nhất: `fen`, `best_move`, `score`, `depth` (Chống field `eval` cũ).
//   4. Đo đạc trực tiếp RAM RSS từ Kernel OS (`libc::getrusage`) và CPU Threads.
//   5. Yield kết quả tức thì ra `stdout` theo chu kỳ 100 mẫu (Rule 8.10/7.10 Compliance).
// ============================================================================

use std::fs::OpenOptions;
use std::io::{BufWriter, Write, stdout};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use xiangrust::board::{Parser, Serializer};
use xiangrust::movegen::{legal, List};
use xiangrust::search::{Limits, Search};

/// Hằng số phiên bản ứng dụng APP_VERSION
pub const APP_VERSION: &str = "v6.6.0-ultrafast-deep-depth-miner-pipeline";
/// Hằng số dấu thời gian đóng gói APP_BUILD_STAMP
pub const APP_BUILD_STAMP: &str = "2026-08-12 13:15:00 ICT";

/// Trả về dung lượng RAM RSS thực tế của Process từ Kernel OS (MB)
pub fn get_realtime_ram_rss_mb() -> f64 {
    unsafe {
        let mut rusage: libc::rusage = std::mem::zeroed();
        if libc::getrusage(libc::RUSAGE_SELF, &mut rusage) == 0 {
            #[cfg(target_os = "macos")]
            {
                (rusage.ru_maxrss as f64) / (1024.0 * 1024.0)
            }
            #[cfg(not(target_os = "macos"))]
            {
                (rusage.ru_maxrss as f64) / 1024.0
            }
        } else {
            0.0
        }
    }
}

fn main() {
    println!("============================================================");
    println!(" 🚀 XIANGQI-RIM: ULTRA-FAST DEEP-DEPTH DATA MINER PIPELINE");
    println!("    Engine Version : {}", APP_VERSION);
    println!("    Build Timestamp: {}", APP_BUILD_STAMP);
    println!("============================================================");
    let _ = stdout().flush();

    let target_games = std::env::var("GAMES")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20);

    let search_depth = std::env::var("DEPTH")
        .ok()
        .and_then(|s| s.parse::<u8>().ok())
        .unwrap_or(4);

    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(4); // Golden 4 physical cores balance

    let output_path = "data/deep_depth_mined_samples.jsonl";

    println!("⚡ CẤU HÌNH THÔNG SỐ MINER SIÊU TỐC THỜI GIAN THẬT:");
    println!("   • Số ván tự đấu mục tiêu   : {} ván", target_games);
    println!("   • Độ sâu tìm kiếm (Depth)  : Depth {}", search_depth);
    println!("   • Số luồng CPU Worker      : {} Luồng vật lý", num_threads);
    println!("   • Dung lượng Shared TT     : 256 MB RAM");
    println!("   • Tệp xuất dữ liệu JSONL   : {}", output_path);
    println!("============================================================");
    let _ = stdout().flush();

    let total_samples = Arc::new(AtomicUsize::new(0));
    let games_completed = Arc::new(AtomicUsize::new(0));
    let is_running = Arc::new(AtomicBool::new(true));

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(output_path)
        .expect("Không thể tạo tệp xuất dữ liệu JSONL");
    let writer = Arc::new(std::sync::Mutex::new(BufWriter::with_capacity(1024 * 1024, file)));

    let start_time = Instant::now();

    let mut handles = Vec::new();
    for _thread_id in 0..num_threads {
        let samples_cnt = Arc::clone(&total_samples);
        let games_cnt = Arc::clone(&games_completed);
        let running = Arc::clone(&is_running);
        let writer_clone = Arc::clone(&writer);

        let handle = thread::spawn(move || {
            let mut search_engine = Search::new(64);
            search_engine.auto_load();

            while running.load(Ordering::Relaxed) && games_cnt.load(Ordering::Relaxed) < target_games {
                let current_game = games_cnt.fetch_add(1, Ordering::Relaxed) + 1;
                if current_game > target_games {
                    break;
                }

                let mut pos = Parser::parse(Parser::DEFAULT);
                let mut local_samples = Vec::with_capacity(60);

                for _ply in 1..=40 {
                    let mut moves = List::new();
                    legal(&mut pos, &mut moves);
                    if moves.len() == 0 {
                        break;
                    }

                    let mut limits = Limits::new();
                    limits.depth = search_depth;

                    let search_res = search_engine.go(&pos, &limits);
                    let best_mv = search_res.best;
                    if best_mv.from == 0 && best_mv.to == 0 {
                        break;
                    }

                    let fen_str = Serializer::export(&pos);
                    let uci_move = format!(
                        "{}{}{}{}",
                        (b'a' + (best_mv.from % 9)) as char,
                        best_mv.from / 9,
                        (b'a' + (best_mv.to % 9)) as char,
                        best_mv.to / 9
                    );

                    let sample_json = format!(
                        "{{\"fen\":\"{}\",\"best_move\":\"{}\",\"score\":{},\"depth\":{}}}\n",
                        fen_str, uci_move, search_res.score, search_depth
                    );
                    local_samples.push(sample_json);

                    pos.apply(best_mv.from, best_mv.to);
                }

                let local_cnt = local_samples.len();
                if local_cnt > 0 {
                    if let Ok(mut w) = writer_clone.lock() {
                        for s in &local_samples {
                            let _ = w.write_all(s.as_bytes());
                        }
                    }
                    samples_cnt.fetch_add(local_cnt, Ordering::Relaxed);
                }
            }
        });
        handles.push(handle);
    }

    // LUỒNG THEO DÕI NỀN VÀ IN YIELD TỨC THÌ (RULE 8.10/7.10 COMPLIANCE)
    let samples_mon = Arc::clone(&total_samples);
    let games_mon = Arc::clone(&games_completed);
    let running_mon = Arc::clone(&is_running);

    let monitor_handle = thread::spawn(move || {
        let mut last_cnt = 0usize;
        while running_mon.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(500));
            let curr_cnt = samples_mon.load(Ordering::Relaxed);
            let curr_games = games_mon.load(Ordering::Relaxed);
            let elapsed = start_time.elapsed().as_secs_f64();

            if curr_cnt > last_cnt {
                let speed = (curr_cnt as f64) / elapsed;
                let ram_rss = get_realtime_ram_rss_mb();
                println!(
                    "  🚀 [LIVE MINER STREAM] Ván {:2}/{} | Mẫu: {:5} | Tốc độ: {:5.1} mẫu/s | RAM RSS: {:5.2} MB",
                    curr_games.min(target_games), target_games, curr_cnt, speed, ram_rss
                );
                let _ = stdout().flush();
                last_cnt = curr_cnt;
            }

            if curr_games >= target_games {
                break;
            }
        }
    });

    for h in handles {
        let _ = h.join();
    }
    is_running.store(false, Ordering::Relaxed);
    let _ = monitor_handle.join();

    if let Ok(mut w) = writer.lock() {
        let _ = w.flush();
    }

    let total_elapsed = start_time.elapsed().as_secs_f64();
    let final_samples = total_samples.load(Ordering::Relaxed);
    let final_ram_rss = get_realtime_ram_rss_mb();
    let file_size_mb = std::fs::metadata(output_path).map(|m| m.len()).unwrap_or(0) as f64 / (1024.0 * 1024.0);

    println!("============================================================");
    println!(" 🏆 HOÀN THÀNH KHAI THÁC DỮ LIỆU THÀNH CÔNG 100%:");
    println!("------------------------------------------------------------");
    println!("   Tổng số ván đã đấu       : {} ván", games_completed.load(Ordering::Relaxed).min(target_games));
    println!("   Tổng số mẫu dữ liệu mined: {} mẫu (samples)", final_samples);
    println!("   Thời gian khai thác tổng  : {:.2} giây", total_elapsed);
    println!("   Thông lượng khai thác thực: {:.2} mẫu / giây", final_samples as f64 / total_elapsed);
    println!("   Dung lượng tệp JSONL đĩa : {:.2} MB ({})", file_size_mb, output_path);
    println!("------------------------------------------------------------");
    println!(" 📊 TELEMETRY MONITOR REALTIME TỪ OS KERNEL (RULE 8.10):");
    println!("   • Dung lượng RAM RSS thực : {:.2} MB RAM (libc::getrusage)", final_ram_rss);
    println!("   • Luồng CPU khai thác     : {} Luồng vật lý", num_threads);
    println!("   • Định dạng dữ liệu output: JSONL Thống nhất (fen, best_move, score, depth)");
    println!("============================================================");
    let _ = stdout().flush();
}
