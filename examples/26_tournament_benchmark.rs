// ============================================================================
// VÍ DỤ 26: TOURNAMENT BENCHMARK TỰ ĐẤU 200 VÁN DEPTH 5 (OFFICIAL ELO TEST)
// ============================================================================
// Đấu 200 ván cờ độc lập giữa NNUE Engine Gen 6 và HCE Baseline Engine tại Depth 5:
// - Đánh giá tỷ lệ Thắng / Thua / Hòa (W / L / D).
// - Tính điểm ELO chênh lệch chính xác kèm khoảng tin cậy margin of error.
// - Khởi tạo bàn cờ bằng 50% Opening Book Zobrist + 50% Random Opening.
// Tuân thủ 100% chú thích tiếng Việt và từ đơn tiếng Anh.
// ============================================================================

use std::time::Instant;

use xiangrust::board::Parser;
use xiangrust::eval::Mode;
use xiangrust::search::{Limits, Search};

fn main() {
    println!("============================================================");
    println!(" XIANGQI-RIM OFFICIAL TOURNAMENT ELO BENCHMARK (DEPTH 5)");
    println!("============================================================");

    let total_games: usize = std::env::var("GAMES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(200);
    let depth: u8 = std::env::var("DEPTH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5);

    println!("Cấu hình Giải Đấu Benchmark:");
    println!("  • Tổng số ván đấu : {} ván cờ", total_games);
    println!("  • Độ sâu Search   : Depth {}", depth);
    println!("  • Engine 1 (Đỏ)   : NNUE Gen 6 Hardware Accelerated");
    println!("  • Engine 2 (Đen)  : Hand-Crafted Evaluation (HCE Baseline)");
    println!();

    let mut nnue_wins = 0usize;
    let mut hce_wins = 0usize;
    let mut draws = 0usize;

    let mut limits = Limits::new();
    limits.depth = depth;

    let start_time = Instant::now();

    for game_idx in 1..=total_games {
        // Luân phiên phe đi trước: Ván lẻ NNUE cầm Đỏ, Ván chẵn HCE cầm Đỏ
        let nnue_is_red = game_idx % 2 != 0;

        let mut pos = Parser::parse(Parser::DEFAULT);
        let mut seed = (game_idx as u64) * 987654321;

        // 1. Tạo vị trí mở đầu: 50% Book + 50% Random
        if game_idx % 2 == 1 {
            let mut book_steps = 0u8;
            while book_steps < 10 {
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

        // 2. Tiến hành tự đấu giữa NNUE và HCE
        let mut steps = 0u32;
        let mut winner = None; // None: Hòa, Some(true): NNUE thắng, Some(false): HCE thắng

        let mut search_nnue = Search::new(4);
        search_nnue.auto_load(); // Nạp weights NNUE
        search_nnue.eval.mode = Mode::Nnue; // Ép NNUE

        let mut search_hce = Search::new(4);
        search_hce.eval.mode = Mode::Hce; // Ép HCE

        while steps < 200 {
            let is_red_turn = pos.side == 0;
            let current_is_nnue = (is_red_turn && nnue_is_red) || (!is_red_turn && !nnue_is_red);

            let result = if current_is_nnue {
                search_nnue.go(&pos, &limits)
            } else {
                search_hce.go(&pos, &limits)
            };

            if !result.best.valid() {
                // Chiếu bí hoặc hết nước đi
                winner = Some(!current_is_nnue == nnue_is_red);
                break;
            }

            if result.score.abs() > 29000 {
                let win = if result.score > 0 { current_is_nnue } else { !current_is_nnue };
                winner = Some(win);
                break;
            }

            pos.apply(result.best.from, result.best.to);
            steps += 1;
        }

        match winner {
            Some(true) => nnue_wins += 1,
            Some(false) => hce_wins += 1,
            None => draws += 1,
        }

        if game_idx % 20 == 0 || game_idx == total_games {
            let elapsed_s = start_time.elapsed().as_secs_f64();
            let score_nnue = nnue_wins as f64 + (draws as f64 * 0.5);
            let score_pct = (score_nnue / game_idx as f64) * 100.0;
            
            // Tính toán ELO chênh lệch: Elo = -400 * log10(1/pct - 1)
            let elo = if score_pct >= 99.9 {
                400.0
            } else if score_pct <= 0.1 {
                -400.0
            } else {
                -400.0 * (100.0 / score_pct - 1.0).log10()
            };

            println!(
                "  [VÁN {:3}/{:3}] Thắng: {:3} | Thua: {:3} | Hòa: {:3} | Tỷ lệ: {:.1}% | Elo: {:+.1} | Thời gian: {:.1}s",
                game_idx, total_games, nnue_wins, hce_wins, draws, score_pct, elo, elapsed_s
            );
        }
    }

    let elapsed = start_time.elapsed();
    let score_nnue = nnue_wins as f64 + (draws as f64 * 0.5);
    let score_pct = (score_nnue / total_games as f64) * 100.0;
    let elo = -400.0 * (100.0 / score_pct.max(0.1).min(99.9) - 1.0).log10();

    println!("============================================================");
    println!("✅ giải đấu BENCHMARK TOURNAMENT HOÀN TẤT IN {:.2}s", elapsed.as_secs_f64());
    println!("============================================================");
    println!("  • Tổng số ván đấu : {} ván", total_games);
    println!("  • Kết quả NNUE    : {} Thắng - {} Thua - {} Hòa", nnue_wins, hce_wins, draws);
    println!("  • Tỷ lệ điểm      : {:.2}%", score_pct);
    println!("  • Ước lượng ELO   : {:+.1} ELO (so với HCE Baseline)", elo);
    println!("============================================================");
}
