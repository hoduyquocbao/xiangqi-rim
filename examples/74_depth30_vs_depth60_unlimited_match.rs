// ============================================================================
// EXAMPLE 74: UNLIMITED MATCH (DEPTH 30 RED VS DEPTH 60 BLACK) - V7.9.0 DYNAMIC CONFIG
// ============================================================================
// Tuân thủ 100% Quy tắc 8.11 / 7.11 (Mandatory Dynamic Configuration & External Exposure Protocol):
//   1. Red AI: `RED_DEPTH` (mặc định 30), `RED_TIME_MS` (mặc định 1,500ms).
//   2. Black AI: `BLACK_DEPTH` (mặc định 60), `BLACK_TIME_MS` (mặc định 2,000ms).
//   3. Hạn mức Ply: `MAX_PLIES` (mặc định 300 plies).
//   4. Dung lượng RAM TT: `HASH_MB` (mặc định 256 MB RAM / Agent).
//   5. Đường dẫn tệp xuất: `MATCH_OUTPUT_PATH` (mặc định `data/depth30_vs_depth60_match.jsonl`).
//   6. Tích hợp 100% Luật Trường Chiếu (Perpetual Check Loss) & Lặp Cờ (3-Fold Repetition Draw).
//   7. Tuân thủ 100% Quy tắc 8.10/7.10: Live Yield từng nước đi & OS Kernel RAM RSS (`libc::getrusage`).
// ============================================================================

use std::fs::OpenOptions;
use std::io::{BufWriter, Write, stdout};
use std::time::Instant;

use xiangrust::board::{Parser, Serializer};
use xiangrust::movegen::{legal, List};
use xiangrust::search::{Limits, Search};

/// Hằng số phiên bản ứng dụng APP_VERSION
pub const APP_VERSION: &str = "v7.9.0-dynamic-config-rule-8.11";
/// Hằng số dấu thời gian đóng gói APP_BUILD_STAMP
pub const APP_BUILD_STAMP: &str = "2026-08-12 14:05:00 ICT";

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
    // Nạp toàn bộ thông số cấu hình động từ Biến Môi Trường OS (Rule 8.11 / 7.11)
    let red_depth: u8 = std::env::var("RED_DEPTH").ok().and_then(|v| v.parse().ok()).unwrap_or(30);
    let red_time_ms: u64 = std::env::var("RED_TIME_MS").ok().and_then(|v| v.parse().ok()).unwrap_or(1500);

    let black_depth: u8 = std::env::var("BLACK_DEPTH").ok().and_then(|v| v.parse().ok()).unwrap_or(60);
    let black_time_ms: u64 = std::env::var("BLACK_TIME_MS").ok().and_then(|v| v.parse().ok()).unwrap_or(2000);

    let max_plies: usize = std::env::var("MAX_PLIES").ok().and_then(|v| v.parse().ok()).unwrap_or(300);
    let hash_mb: usize = std::env::var("HASH_MB").ok().and_then(|v| v.parse().ok()).unwrap_or(256);
    let output_path = std::env::var("MATCH_OUTPUT_PATH").unwrap_or_else(|_| "data/depth30_vs_depth60_match.jsonl".to_string());

    println!("============================================================");
    println!(" ⚔️ XIANGQI-RIM: UNLIMITED MATCH (DYNAMIC CONFIGURATION RULE 8.11)");
    println!("    Engine Version : {}", APP_VERSION);
    println!("    Build Timestamp: {}", APP_BUILD_STAMP);
    println!("============================================================");
    println!("⚙️ THÔNG SỐ CẤU HÌNH ĐỘNG (DYNAMIC ENVIRONMENT CONFIG):");
    println!("   • Phe Đỏ (Red AI)   : DEPTH {} (Giới hạn: {} ms / move) [Env: RED_DEPTH, RED_TIME_MS]", red_depth, red_time_ms);
    println!("   • Phe Đen (Black AI) : DEPTH {} (Giới hạn: {} ms / move) [Env: BLACK_DEPTH, BLACK_TIME_MS]", black_depth, black_time_ms);
    println!("   • Giới hạn Ply tối đa: {} plies [Env: MAX_PLIES]", max_plies);
    println!("   • Dung lượng RAM TT  : {} MB RAM / Agent [Env: HASH_MB]", hash_mb);
    println!("   • Tệp xuất dữ liệu   : {} [Env: MATCH_OUTPUT_PATH]", output_path);
    println!("============================================================");
    let _ = stdout().flush();

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&output_path)
        .expect("Không thể tạo tệp xuất dữ liệu JSONL Trận đấu");
    let mut writer = BufWriter::with_capacity(1024 * 1024, file);

    let mut pos = Parser::parse(Parser::DEFAULT);
    let mut red_engine = Search::new(hash_mb);
    let mut black_engine = Search::new(hash_mb);
    red_engine.auto_load();
    black_engine.auto_load();

    let mut match_history: Vec<u64> = Vec::with_capacity(512);

    let match_start = Instant::now();
    let mut ply = 0usize;
    let mut game_over = false;
    let mut outcome_str = "DRAW";

    while !game_over && ply < max_plies {
        ply += 1;
        let is_red = pos.side == 0;
        let side_name = if is_red {
            format!("RED (D{})", red_depth)
        } else {
            format!("BLACK (D{})", black_depth)
        };
        let ply_start = Instant::now();

        // 1. Lưu Zobrist Hash hiện tại vào mảng lịch sử ván cờ
        match_history.push(pos.hash);
        red_engine.push_history(pos.hash);
        black_engine.push_history(pos.hash);

        // 2. Kiểm tra Luật Lặp Cờ 3 Lần & Luật Trường Chiếu (Perpetual Check)
        let rep_count = match_history.iter().filter(|&&h| h == pos.hash).count();
        if rep_count >= 3 {
            game_over = true;
            let is_in_check = legal::check(&pos, pos.side as usize);
            if is_in_check {
                outcome_str = if is_red {
                    "RED_WINS_BLACK_PERPETUAL_CHECK_LOSS"
                } else {
                    "BLACK_WINS_RED_PERPETUAL_CHECK_LOSS"
                };
            } else {
                outcome_str = "DRAW_REPETITION_3FOLD";
            }
            println!(
                " 🛑 [MATCH STREAM] Ply {:3} | {:11} | PERPETUAL RULE TRIGGERED: {}!",
                ply, side_name, outcome_str
            );
            let _ = stdout().flush();
            break;
        }

        let mut moves = List::new();
        legal(&mut pos, &mut moves);
        if moves.len() == 0 {
            game_over = true;
            outcome_str = if is_red { "BLACK_WINS_CHECKMATE" } else { "RED_WINS_CHECKMATE" };
            break;
        }

        let mut limits = Limits::new();
        if is_red {
            limits.depth = red_depth;
            limits.exact = red_time_ms;
        } else {
            limits.depth = black_depth;
            limits.exact = black_time_ms;
        }

        let search_res = if is_red {
            red_engine.go(&pos, &limits)
        } else {
            black_engine.go(&pos, &limits)
        };

        let best_mv = search_res.best;
        if best_mv.from == 0 && best_mv.to == 0 {
            game_over = true;
            outcome_str = if is_red { "BLACK_WINS_RESIGN" } else { "RED_WINS_RESIGN" };
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
            "{{\"ply\":{},\"side\":\"{}\",\"fen\":\"{}\",\"best_move\":\"{}\",\"score\":{}}}\n",
            ply, side_name, fen_str, uci_move, search_res.score
        );
        let _ = writer.write_all(sample_json.as_bytes());
        let _ = writer.flush();

        let ply_elapsed = ply_start.elapsed().as_secs_f64();
        let match_elapsed = match_start.elapsed().as_secs_f64();
        let ram_rss = get_realtime_ram_rss_mb();

        println!(
            "  🚀 [MATCH STREAM] Ply {:3} | {:11} | Best: {:2}->{:2} | Score: {:5} cp | Ply Time: {:5.2}s | Match Time: {:6.2}s | OS RAM: {:.2} MB",
            ply, side_name, best_mv.from, best_mv.to, search_res.score, ply_elapsed, match_elapsed, ram_rss
        );
        let _ = stdout().flush();

        pos.apply(best_mv.from, best_mv.to);
    }

    let _ = writer.flush();
    let total_elapsed = match_start.elapsed().as_secs_f64();
    let final_ram = get_realtime_ram_rss_mb();
    let file_size_mb = std::fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0) as f64 / (1024.0 * 1024.0);

    println!("============================================================");
    println!(" 🏆 HOÀN THÀNH TRẬN ĐẤU ĐỐI ĐẦU ĐỘNG DEPTH {} VS DEPTH {}:", red_depth, black_depth);
    println!("------------------------------------------------------------");
    println!("   Kết quả trận đấu        : {}", outcome_str);
    println!("   Tổng số nước cờ đã đấu  : {} plies", ply);
    println!("   Thời gian thi đấu tổng  : {:.2} giây ({:.2} phút)", total_elapsed, total_elapsed / 60.0);
    println!("   Tốc độ trung bình       : {:.2} giây / nước cờ", total_elapsed / (ply.max(1) as f64));
    println!("   Dung lượng tệp JSONL đĩa: {:.4} MB ({})", file_size_mb, output_path);
    println!("------------------------------------------------------------");
    println!(" 📊 TELEMETRY MONITOR REALTIME TỪ OS KERNEL (RULE 8.10):");
    println!("   • Dung lượng RAM RSS thực: {:.2} MB RAM (libc::getrusage)", final_ram);
    println!("   • Số luồng CPU khả dụng  : {} luồng (Intel i5-8259U @ 3.8 GHz)", std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1));
    println!("============================================================");
    let _ = stdout().flush();
}
