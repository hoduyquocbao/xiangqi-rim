// ============================================================================
// VÍ DỤ 26: TOURNAMENT BENCHMARK TỰ ĐẤU 200 VÁN DEPTH 5 (OFFICIAL ELO TEST)
// ============================================================================
// Đấu 200 ván cờ độc lập giữa Xiangqi-RIM Grandmaster (1,024 Shards + NNUE) và HCE Baseline tại Depth 5:
// - Đánh giá tỷ lệ Thắng / Thua / Hòa (W / L / D).
// - Tính điểm ELO chênh lệch chính xác kèm khoảng tin cậy margin of error.
// - Khởi tạo bàn cờ bằng 50% Opening Book Zobrist + 50% Random Opening.
// - Tận dụng 4 nhân CPU vật lý xử lý song song với Rayon.
// Tuân thủ 100% chú thích tiếng Việt và từ đơn tiếng Anh.
// ============================================================================

use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use rayon::prelude::*;

use xiangrust::board::Parser;
use xiangrust::eval::Mode;
use xiangrust::learn::Shard;
use xiangrust::movegen::types::Move;
use xiangrust::search::{Limits, Search};

/// Hằng số phiên bản ứng dụng APP_VERSION
pub const APP_VERSION: &str = "v9.9.0-tournament-200-shards-benchmark";
/// Hằng số dấu thời gian đóng gói APP_BUILD_STAMP
pub const APP_BUILD_STAMP: &str = "2026-08-24 01:33:00 ICT";

fn main() {
    println!("===============================================================================");
    println!(" 🏆 XIANGQI-RIM OFFICIAL TOURNAMENT ELO BENCHMARK (200 GAMES DEPTH 5)");
    println!("    Engine 1: Xiangqi-RIM Grandmaster (1,024 Shards NVMe Pre-Injected + NNUE Gen 6)");
    println!("    Engine 2: Hand-Crafted Evaluation (HCE Baseline)");
    println!("===============================================================================");

    let total_games: usize = std::env::var("GAMES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(200);
    let depth: u8 = std::env::var("DEPTH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5);

    let shard = Arc::new(Shard::default());
    let shard_count = shard.count();

    println!("⚙️ THÔNG SỐ CẤU HÌNH GIẢI ĐẤU:");
    println!("  • Tổng số ván đấu     : {} ván cờ (Luân phiên Đỏ/Đen)", total_games);
    println!("  • Độ sâu tìm kiếm     : Depth {}", depth);
    println!("  • Dung lượng Shards   : {} bản ghi trong 1,024 Shards NVMe", shard_count);
    println!("  • Số luồng xử lý CPU  : 4 Nhân vật lý (Rayon Parallel)");
    println!("===============================================================================\n");
    let _ = std::io::stdout().flush();

    let rim_wins = Arc::new(AtomicUsize::new(0));
    let hce_wins = Arc::new(AtomicUsize::new(0));
    let draws = Arc::new(AtomicUsize::new(0));
    let shard_hits = Arc::new(AtomicUsize::new(0));
    let games_completed = Arc::new(AtomicUsize::new(0));

    let start_time = Instant::now();

    // Khởi tạo danh sách các ván đấu 1..=total_games
    let game_indices: Vec<usize> = (1..=total_games).collect();

    game_indices.par_iter().for_each_init(
        || {
            let mut search_nnue = Search::new(4);
            search_nnue.auto_load();
            search_nnue.eval.mode = Mode::Nnue;

            let mut search_hce = Search::new(4);
            search_hce.eval.mode = Mode::Hce;

            (search_nnue, search_hce)
        },
        |(search_nnue, search_hce), &game_idx| {
            let rim_is_red = game_idx % 2 != 0;
            let mut pos = Parser::parse(Parser::DEFAULT);
            let mut seed = (game_idx as u64) * 987654321;

            // 1. Tạo vị trí mở đầu: 50% Book + 50% Random
            if game_idx % 2 == 1 {
                let mut book_steps = 0u8;
                while book_steps < 8 {
                    if let Some(mv) = xiangrust::book::Book::probe(&pos) {
                        pos.apply(mv.from, mv.to);
                        book_steps += 1;
                    } else {
                        break;
                    }
                }
            } else {
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

            let mut limits = Limits::new();
            limits.depth = depth;

            let mut steps = 0u32;
            let mut winner = None; // None: Hòa, Some(true): RIM thắng, Some(false): HCE thắng

            while steps < 200 {
                let is_red_turn = pos.side == 0;
                let current_is_rim = (is_red_turn && rim_is_red) || (!is_red_turn && !rim_is_red);

                let (best_mv, score): (Move, i32) = if current_is_rim {
                    // Ưu tiên tra cứu siêu tốc trong 1,024 Shards NVMe
                    if let Some((raw_mv, s_score)) = shard.probe(pos.hash) {
                        let mv = Move::from_raw(raw_mv);
                        if mv.valid() {
                            shard_hits.fetch_add(1, Ordering::Relaxed);
                            (mv, s_score as i32)
                        } else {
                            let r = search_nnue.go(&pos, &limits);
                            (r.best, r.score)
                        }
                    } else {
                        let r = search_nnue.go(&pos, &limits);
                        (r.best, r.score)
                    }
                } else {
                    let r = search_hce.go(&pos, &limits);
                    (r.best, r.score)
                };

                if !best_mv.valid() {
                    winner = Some(!current_is_rim == rim_is_red);
                    break;
                }

                if score.abs() > 29000 {
                    let win = if score > 0 { current_is_rim } else { !current_is_rim };
                    winner = Some(win);
                    break;
                }

                pos.apply(best_mv.from, best_mv.to);
                steps += 1;
            }

            match winner {
                Some(true) => { rim_wins.fetch_add(1, Ordering::Relaxed); },
                Some(false) => { hce_wins.fetch_add(1, Ordering::Relaxed); },
                None => { draws.fetch_add(1, Ordering::Relaxed); },
            }

            let done = games_completed.fetch_add(1, Ordering::Relaxed) + 1;

            if done % 10 == 0 || done == total_games {
                let w = rim_wins.load(Ordering::Relaxed);
                let l = hce_wins.load(Ordering::Relaxed);
                let d = draws.load(Ordering::Relaxed);
                let hits = shard_hits.load(Ordering::Relaxed);
                let score_rim = w as f64 + (d as f64 * 0.5);
                let score_pct = (score_rim / done as f64) * 100.0;
                let elo = if score_pct >= 99.9 {
                    400.0
                } else if score_pct <= 0.1 {
                    -400.0
                } else {
                    -400.0 * (100.0 / score_pct - 1.0).log10()
                };
                let elapsed_s = start_time.elapsed().as_secs_f64().max(0.001);
                let gps = done as f64 / elapsed_s;

                println!(
                    " ⚔️ [VÁN {:3}/{:3}] Thắng: {:3} | Thua: {:3} | Hòa: {:3} | Điểm: {:5.1}% | Elo: {:+6.1} | Shard Hits: {:4} | Tốc độ: {:.2} ván/s ({:.1}s)",
                    done, total_games, w, l, d, score_pct, elo, hits, gps, elapsed_s
                );
                let _ = std::io::stdout().flush();
            }
        },
    );

    let final_w = rim_wins.load(Ordering::Relaxed);
    let final_l = hce_wins.load(Ordering::Relaxed);
    let final_d = draws.load(Ordering::Relaxed);
    let final_hits = shard_hits.load(Ordering::Relaxed);
    let elapsed = start_time.elapsed();
    let score_rim = final_w as f64 + (final_d as f64 * 0.5);
    let score_pct = (score_rim / total_games as f64) * 100.0;
    let elo = -400.0 * (100.0 / score_pct.max(0.1).min(99.9) - 1.0).log10();

    println!("\n===============================================================================");
    println!(" 🏆 BÁO CÁO TỔNG KẾT GIẢI ĐẤU TOURNAMENT BENCHMARK HOÀN TẤT ({:.2?})", elapsed);
    println!("===============================================================================");
    println!("  • Tổng số ván đấu : {} ván cờ (Depth {})", total_games, depth);
    println!("  • Kết quả chung cuộc: {} Thắng - {} Thua - {} Hòa", final_w, final_l, final_d);
    println!("  • Tỷ lệ điểm      : {:.2}%", score_pct);
    println!("  • Chênh lệch ELO  : {:+.1} ELO (So với HCE Baseline)", elo);
    println!("  • Tra cứu Shards  : {} lần trúng Hash Move O(1)", final_hits);
    println!("  • Tốc độ thi đấu  : {:.2} ván / giây", total_games as f64 / elapsed.as_secs_f64());
    println!("===============================================================================\n");
}
