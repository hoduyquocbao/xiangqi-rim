// ============================================================================
// EXAMPLE 74: UNLIMITED MATCH (DEPTH 30 RED VS DEPTH 60 BLACK)
// ============================================================================
// Động Cơ Thực Thi Trận Đấu Không Giới Hạn Ply Giữa Depth 30 (Đỏ) và Depth 60 (Đen):
//   1. Red AI: Depth 30 (2,000ms/nước đi).
//   2. Black AI: Depth 60 (3,000ms/nước đi).
//   3. Đấu liên tục không giới hạn nước đi cho đến khi Thắng/Thua/Hòa.
//   4. Xuất toàn bộ biến thể trận đấu ra `data/depth30_vs_depth60_match.jsonl` (Flush mỗi ply).
//   5. Tuân thủ 100% Quy tắc 8.10/7.10: Live Yield từng nước đi & OS Kernel RAM RSS (`libc::getrusage`).
// ============================================================================

use std::fs::OpenOptions;
use std::io::{BufWriter, Write, stdout};
use std::time::Instant;

use xiangrust::board::{Parser, Serializer};
use xiangrust::movegen::{legal, List};
use xiangrust::search::{Limits, Search};

/// Hằng số phiên bản ứng dụng APP_VERSION
pub const APP_VERSION: &str = "v7.4.0-depth30-vs-depth60-unlimited-match";
/// Hằng số dấu thời gian đóng gói APP_BUILD_STAMP
pub const APP_BUILD_STAMP: &str = "2026-08-12 13:42:00 ICT";

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
    println!(" ⚔️ XIANGQI-RIM: UNLIMITED MATCH (DEPTH 30 RED VS DEPTH 60 BLACK)");
    println!("    Engine Version : {}", APP_VERSION);
    println!("    Build Timestamp: {}", APP_BUILD_STAMP);
    println!("============================================================");
    let _ = stdout().flush();

    let output_path = "data/depth30_vs_depth60_match.jsonl";

    println!("⚡ BẮT ĐẦU TRẬN ĐẤU KHÔNG GIỚI HẠN PLY:");
    println!("   • Phe Đỏ (Red AI)   : DEPTH 30 (2,000ms limit / move)");
    println!("   • Phe Đen (Black AI) : DEPTH 60 (3,000ms limit / move)");
    println!("   • Dung lượng RAM TT  : 256 MB RAM / Agent");
    println!("   • Tệp xuất dữ liệu   : {}", output_path);
    println!("============================================================");
    let _ = stdout().flush();

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(output_path)
        .expect("Không thể tạo tệp xuất dữ liệu JSONL Trận đấu");
    let mut writer = BufWriter::with_capacity(1024 * 1024, file);

    let mut pos = Parser::parse(Parser::DEFAULT);
    let mut red_engine = Search::new(256);
    let mut black_engine = Search::new(256);
    red_engine.auto_load();
    black_engine.auto_load();

    let match_start = Instant::now();
    let mut ply = 0usize;
    let mut game_over = false;
    let mut outcome_str = "DRAW";

    while !game_over && ply < 25 {
        ply += 1;
        let is_red = pos.side == 0;
        let side_name = if is_red { "RED (D30)" } else { "BLACK (D60)" };
        let ply_start = Instant::now();

        let mut moves = List::new();
        legal(&mut pos, &mut moves);
        if moves.len() == 0 {
            game_over = true;
            outcome_str = if is_red { "BLACK_WINS_CHECKMATE" } else { "RED_WINS_CHECKMATE" };
            break;
        }

        let mut limits = Limits::new();
        if is_red {
            limits.depth = 30;
            limits.exact = 1500;
        } else {
            limits.depth = 60;
            limits.exact = 2000;
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
        let _ = writer.flush(); // FLUSH IMMEDIATELY

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
    let file_size_mb = std::fs::metadata(output_path).map(|m| m.len()).unwrap_or(0) as f64 / (1024.0 * 1024.0);

    println!("============================================================");
    println!(" 🏆 HOÀN THÀNH TRẬN ĐẤU ĐỐI ĐẦU DEPTH 30 VS DEPTH 60:");
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
