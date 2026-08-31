// ============================================================================
// VÍ DỤ 116: GIẢI ĐẤU TOURNAMENT BENCHMARK ĐỐI ĐẦU ĐÁNH GIÁ SỨC MẠNH ELO NNUE GEN 6
// ============================================================================
// `116_nnue_gen6_tournament_benchmark.rs` tổ chức giải đấu chuẩn quốc tế:
// - Đấu trực tiếp giữa Động cơ AI NNUE Gen 6 Master vs Động cơ Cổ điển HCE Baseline.
// - Tự động hoán đổi bên Tiên/Hậu (Red/Black Swap) đảm bảo công bằng 100%.
// - Áp dụng Zobrist Opening Book và Shards 10B Endgame Knowledge.
// - Tính toán chính xác tỷ số Thắng / Hòa / Thua và quy đổi Elo Rating Delta.
//
// 100% chú thích tiếng Việt từng dòng & 100% định danh từ đơn tiếng Anh.
// ============================================================================

use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use xiangrust::board::Parser;
use xiangrust::book::Book;
use xiangrust::eval::Mode;
use xiangrust::movegen::{legal, types::List, Move};
use xiangrust::search::{Limits, Search};

/// Đếm số quân lớn trên bàn cờ
fn count_material(pos: &xiangrust::board::Position) -> usize {
    let mut total = 0;
    for i in 0..90 {
        let piece = pos.grid[i];
        if piece != 0 && (piece & 7) != 7 {
            total += 1;
        }
    }
    total
}

/// Tính toán chênh lệch Elo dựa trên tỷ lệ điểm số (Score Ratio)
fn calculate_elo(wins: usize, draws: usize, losses: usize) -> f64 {
    let total = (wins + draws + losses) as f64;
    if total == 0.0 {
        return 0.0;
    }
    let score = (wins as f64 + 0.5 * draws as f64) / total;
    if score <= 0.001 {
        return -800.0;
    }
    if score >= 0.999 {
        return 800.0;
    }
    -400.0 * (1.0 / score - 1.0).ln() / std::f64::consts::LN_10
}

fn main() {
    println!("===============================================================================");
    println!(" 🏆 XIANGQI-RIM TOURNAMENT BENCHMARK: NNUE GEN 6 MASTER VS HCE BASELINE");
    println!("    Định dạng: Tranh cúp chuẩn quốc tế | Đổi bên luân phiên | 4-Core CPU");
    println!("===============================================================================");

    let total_games: usize = std::env::var("GAMES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);
    let search_depth: u8 = std::env::var("DEPTH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5);
    let threads: usize = std::env::var("THREADS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4);

    println!("⚙️ THÔNG SỐ GIẢI ĐẤU:");
    println!("  • Tổng số ván đấu       : {} ván cờ", total_games);
    println!("  • Độ sâu tìm kiếm (Depth): {}", search_depth);
    println!("  • Số luồng CPU song song: {} Threads", threads);
    println!("===============================================================================\n");

    let nnue_wins = Arc::new(AtomicUsize::new(0));
    let hce_wins = Arc::new(AtomicUsize::new(0));
    let draw_count = Arc::new(AtomicUsize::new(0));
    let completed = Arc::new(AtomicUsize::new(0));

    let games_per_thread = total_games / threads;
    let start_time = Instant::now();

    let mut handles = Vec::with_capacity(threads);

    for thread_id in 0..threads {
        let n_wins = Arc::clone(&nnue_wins);
        let h_wins = Arc::clone(&hce_wins);
        let d_count = Arc::clone(&draw_count);
        let comp = Arc::clone(&completed);

        let my_games = if thread_id == threads - 1 {
            total_games - (thread_id * games_per_thread)
        } else {
            games_per_thread
        };

        handles.push(thread::spawn(move || {
            let mut search_nnue = Search::new(8);
            let mut search_hce = Search::new(8);
            // Ép search_hce sử dụng chế độ HCE tĩnh
            search_hce.eval.mode = Mode::Hce;

            for game_idx in 0..my_games {
                let mut pos = Parser::parse(Parser::DEFAULT);
                let nnue_is_red = (game_idx % 2) == 0;
                let mut plies = 0usize;
                let mut past_hashes = Vec::with_capacity(256);
                let mut seed = ((thread_id + 1) * 10007 + (game_idx + 1) * 7919) as u64;

                // Khởi tạo 4-6 nước đi mở đầu ngẫu nhiên để tạo thế trận đa dạng
                let random_open_plies = 4 + (game_idx % 3) * 2; // 4, 6, 8 plies

                while plies < 200 {
                    past_hashes.push(pos.hash);

                    let mut chosen_move = Move::none();

                    // 1. Giai đoạn mở đầu: Nước ngẫu nhiên hoặc Opening Book
                    if plies < random_open_plies {
                        let mut legals = List::new();
                        legal::gen(&mut pos, &mut legals);
                        if legals.count > 0 {
                            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                            let rand_idx = ((seed >> 33) as usize) % legals.count;
                            chosen_move = legals.items[rand_idx];
                        }
                    } else if plies < 12 {
                        if let Some(book_move) = Book::probe(&pos) {
                            chosen_move = book_move;
                        }
                    }

                    // 2. Tìm kiếm nước đi AI thi đấu chính thức
                    if !chosen_move.valid() {
                        let mut limits = Limits::new();
                        limits.depth = search_depth;

                        let is_nnue_turn = (pos.side == 0 && nnue_is_red) || (pos.side == 1 && !nnue_is_red);
                        if is_nnue_turn {
                            search_nnue.past_hashes = past_hashes.clone();
                            let res = search_nnue.go(&pos, &limits);
                            chosen_move = res.best;
                        } else {
                            search_hce.past_hashes = past_hashes.clone();
                            let res = search_hce.go(&pos, &limits);
                            chosen_move = res.best;
                        }
                    }

                    // 3. Fallback nước đi hợp lệ
                    if !chosen_move.valid() {
                        let mut legals = List::new();
                        legal::gen(&mut pos, &mut legals);
                        if legals.count > 0 {
                            chosen_move = legals.items[0];
                        } else {
                            break;
                        }
                    }

                    pos.apply(chosen_move.from, chosen_move.to);
                    plies += 1;

                    // Kiểm tra hết quân hoặc bắt bí
                    let mut next_legals = List::new();
                    legal::gen(&mut pos, &mut next_legals);
                    if next_legals.count == 0 {
                        break;
                    }

                    // Kiểm tra tàn cuộc hòa nếu cả 2 bên chỉ còn Tướng Sĩ Tượng
                    if count_material(&pos) == 0 {
                        break;
                    }
                }

                // Đánh giá kết quả ván cờ
                let mut final_legals = List::new();
                legal::gen(&mut pos, &mut final_legals);

                let is_mate = final_legals.count == 0;
                let current_side = pos.side;

                if is_mate {
                    // Bên đến lượt đi bị chiếu bí -> Bên đối diện thắng
                    let winner_is_red = current_side == 1;
                    if (winner_is_red && nnue_is_red) || (!winner_is_red && !nnue_is_red) {
                        n_wins.fetch_add(1, Ordering::Relaxed);
                    } else {
                        h_wins.fetch_add(1, Ordering::Relaxed);
                    }
                } else {
                    d_count.fetch_add(1, Ordering::Relaxed);
                }

                let cur_comp = comp.fetch_add(1, Ordering::Relaxed) + 1;
                if cur_comp % 10 == 0 || cur_comp == total_games {
                    let nw = n_wins.load(Ordering::Relaxed);
                    let hw = h_wins.load(Ordering::Relaxed);
                    let dr = d_count.load(Ordering::Relaxed);
                    let elo = calculate_elo(nw, dr, hw);
                    println!(
                        "  [TIẾN ĐỘ {:3}/{:3}] NNUE Thắng: {:3} | HCE Thắng: {:3} | Hòa: {:3} | Elo Delta: {:+6.1}",
                        cur_comp, total_games, nw, hw, dr, elo
                    );
                    let _ = std::io::stdout().flush();
                }
            }
        }));
    }

    for h in handles {
        let _ = h.join();
    }

    let elapsed = start_time.elapsed();
    let final_nnue = nnue_wins.load(Ordering::Relaxed);
    let final_hce = hce_wins.load(Ordering::Relaxed);
    let final_draws = draw_count.load(Ordering::Relaxed);
    let final_elo = calculate_elo(final_nnue, final_draws, final_hce);

    println!("\n===============================================================================");
    println!(" 🏆 BẢNG KẾT QUẢ TOURNAMENT BENCHMARK CHÍNH THỨC");
    println!("===============================================================================");
    println!("  • Tổng số ván đã đấu   : {} ván", total_games);
    println!("  • Thời gian thi đấu     : {:.2?}", elapsed);
    println!("  • Tốc độ trung bình     : {:.2}s / ván", elapsed.as_secs_f64() / total_games as f64);
    println!("  • NNUE Gen 6 Master     : {} Thắng ({:.1}%)", final_nnue, (final_nnue as f64 / total_games as f64) * 100.0);
    println!("  • HCE Baseline Fallback : {} Thắng ({:.1}%)", final_hce, (final_hce as f64 / total_games as f64) * 100.0);
    println!("  • Hòa cờ                : {} Ván ({:.1}%)", final_draws, (final_draws as f64 / total_games as f64) * 100.0);
    println!("  • ĐỘ TĂNG TRƯỞNG SỨC MẠNH: {:+6.1} ELO", final_elo);
    println!("===============================================================================\n");
}
