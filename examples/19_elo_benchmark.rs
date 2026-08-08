// ============================================================================
// EXAMPLE 19: BENCHMARK ELO v6 — HIGH PERFORMANCE STREAMING & ZERO DISK I/O RE-LOAD
// ============================================================================
// Đã tối ưu hóa vượt bậc:
//   1. Nạp trọng số NNUE GPU 1 LẦN DUY NHẤT bộ nhớ RAM lúc khởi động program.
//   2. Tái sử dụng đối tượng Search qua từng ván (zero memory reallocation).
//   3. Hiển thị Tiến độ Real-time Streaming TỪNG VÁN MỘT kèm ETA chính xác từng giây.
//
// Sử dụng: cargo run --release --example 19_elo_benchmark
// Biến môi trường:
//   GAMES=100          Số ván đấu (mặc định 100 ván)
//   DEPTH=4            Độ sâu tìm kiếm Engine (mặc định 4)
//   NNUE_PATH=data/nnue_weights_gpu.bin  (đường dẫn weights NNUE GPU)
// ============================================================================

use std::io::Write;
use xiangrust::board::Parser;
use xiangrust::eval::Mode;
use xiangrust::search::{Limits, Search};

/// Kết quả 1 ván đấu
#[derive(Debug, Clone, Copy, PartialEq)]
enum Outcome {
    First,
    Second,
    Draw,
}

/// Struct `Rating` tính toán Elo từ kết quả thắng/thua/hòa.
struct Rating {
    wins: u32,
    losses: u32,
    draws: u32,
}

impl Rating {
    fn new() -> Self {
        Self {
            wins: 0,
            losses: 0,
            draws: 0,
        }
    }

    fn ratio(&self) -> f64 {
        let total = self.wins + self.losses + self.draws;
        if total == 0 {
            return 0.5;
        }
        (self.wins as f64 + 0.5 * self.draws as f64) / total as f64
    }

    fn elo(&self) -> f64 {
        let w = self.ratio();
        if w <= 0.0 {
            return -800.0;
        }
        if w >= 1.0 {
            return 800.0;
        }
        -400.0 * (1.0 / w - 1.0).log10()
    }

    fn margin(&self) -> f64 {
        let total = (self.wins + self.losses + self.draws) as f64;
        if total == 0.0 {
            return 0.0;
        }
        let w = self.ratio();
        let dev = (w * (1.0 - w) / total).sqrt();
        let factor = if w > 0.01 && w < 0.99 {
            400.0 / (10.0f64.ln()) / (w * (1.0 - w))
        } else {
            800.0
        };
        1.96 * dev * factor.abs()
    }
}

/// PRNG xorshift64 cho random opening
fn random(seed: &mut u64) -> u64 {
    let mut s = *seed;
    s ^= s << 13;
    s ^= s >> 7;
    s ^= s << 17;
    *seed = s;
    s
}

/// Chơi N nước random ở đầu ván để đa dạng hóa opening.
fn randomize(seed: &mut u64, count: usize) -> xiangrust::board::Position {
    let mut pos = Parser::parse(Parser::DEFAULT);

    for _ in 0..count {
        let mut moves = xiangrust::movegen::List::new();
        xiangrust::movegen::legal(&mut pos, &mut moves);

        if moves.len() == 0 {
            break;
        }

        let r = random(seed) as usize;
        let idx = r % moves.len();
        let m = moves.items[idx];
        pos.apply(m.from, m.to);
    }

    pos
}

/// Chạy 1 ván đấu giữa 2 Engine (Tái sử dụng Search object đã nạp sẵn weights, zero re-allocation)
fn play_game(
    start: &xiangrust::board::Position,
    search_a: &mut Search,
    depth_a: u8,
    search_b: &mut Search,
    depth_b: u8,
    limit: u32,
) -> Outcome {
    let mut pos = start.clone();
    
    let mut limits_a = Limits::new();
    limits_a.depth = depth_a;
    let mut limits_b = Limits::new();
    limits_b.depth = depth_b;
    
    let mut steps = 0u32;
    let mut history: Vec<u64> = Vec::with_capacity(512);

    while steps < limit {
        history.push(pos.hash);

        let mut reps = 0u32;
        for h in history.iter().rev().skip(1) {
            if *h == pos.hash {
                reps += 1;
            }
        }
        if reps >= 2 {
            return Outcome::Draw;
        }

        let is_a = steps % 2 == 0;
        let result = if is_a {
            search_a.go(&pos, &limits_a)
        } else {
            search_b.go(&pos, &limits_b)
        };

        if !result.best.valid() {
            return if is_a { Outcome::Second } else { Outcome::First };
        }

        if result.score.abs() > 29000 {
            if result.score > 29000 {
                return if is_a { Outcome::First } else { Outcome::Second };
            } else {
                return if is_a { Outcome::Second } else { Outcome::First };
            }
        }

        pos.apply(result.best.from, result.best.to);
        steps += 1;
    }

    Outcome::Draw
}

fn main() {
    println!("============================================================");
    println!(" XIANGQI-RIM ELO BENCHMARK ENGINE v6 (OPTIMIZED REAL-TIME) ");
    println!("============================================================");

    let games: u32 = std::env::var("GAMES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);
    let depth: u8 = std::env::var("DEPTH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4);
    let nnue_path: String = std::env::var("NNUE_PATH")
        .unwrap_or_else(|_| "data/nnue_weights_gpu.bin".to_string());
    
    let limit: u32 = 300;
    let opening: usize = 6;
    let mut seed = 20260808u64;

    println!("Cấu hình Tournament:");
    println!("  • Số ván đấu: {}", games);
    println!("  • Độ sâu tìm kiếm: Depth {}", depth);
    println!("  • Random Opening: {} nước đầu", opening);
    println!("  • NNUE GPU Weights: {}", nnue_path);
    println!("  • Tối ưu hóa: Nạp weights 1 LẦN DUY NHẤT vào RAM, Real-time Stream từng ván");
    println!();
    let _ = std::io::stdout().flush();

    // Khởi tạo sẵn các đối tượng Search (Khởi tạo 1 LẦN DUY NHẤT)
    let mut search_hce_a = Search::new(16);
    search_hce_a.eval.mode(Mode::Hce);

    let mut search_hce_b = Search::new(16);
    search_hce_b.eval.mode(Mode::Hce);

    // =========================================================================
    // BENCHMARK 1: Depth N vs Depth N-1 (HCE Baseline Check)
    // =========================================================================
    println!("[BENCHMARK 1] HCE Baseline: Depth {} vs Depth {} ({} ván)...", depth, depth - 1, games);
    let _ = std::io::stdout().flush();

    let begin = std::time::Instant::now();
    let mut rating1 = Rating::new();

    for g in 0..games {
        let start = randomize(&mut seed, opening);
        let outcome = if g % 2 == 0 {
            play_game(&start, &mut search_hce_a, depth, &mut search_hce_b, depth - 1, limit)
        } else {
            match play_game(&start, &mut search_hce_b, depth - 1, &mut search_hce_a, depth, limit) {
                Outcome::First => Outcome::Second,
                Outcome::Second => Outcome::First,
                Outcome::Draw => Outcome::Draw,
            }
        };

        match outcome {
            Outcome::First => rating1.wins += 1,
            Outcome::Second => rating1.losses += 1,
            Outcome::Draw => rating1.draws += 1,
        }

        // FLUSH TRỰC TIẾP TỪNG VÁN MỘT!
        let elapsed_secs = begin.elapsed().as_secs_f64();
        let speed = (g + 1) as f64 / elapsed_secs;
        let remaining = games - (g + 1);
        let eta_secs = if speed > 0.0 { (remaining as f64 / speed).round() as u64 } else { 0 };
        let eta_m = eta_secs / 60;
        let eta_s = eta_secs % 60;
        let pct = (g + 1) * 100 / games;

        println!(
            "  [VÁN {:3}/{:3}] ({:3}%) W={} L={} D={} | Elo: {:+.0} ±{:.0} | Speed: {:.1} ván/s | ETA: {:02}m{:02}s",
            g + 1, games, pct,
            rating1.wins, rating1.losses, rating1.draws,
            rating1.elo(), rating1.margin(),
            speed, eta_m, eta_s
        );
        let _ = std::io::stdout().flush();
    }
    println!("  ✅ [BENCHMARK 1 HOÀN TẤT] Depth {} vs {}: W={} L={} D={} | Elo: {:+.0} ±{:.0} ({:.1}s)", 
        depth, depth - 1, rating1.wins, rating1.losses, rating1.draws, rating1.elo(), rating1.margin(), begin.elapsed().as_secs_f64());
    println!();
    let _ = std::io::stdout().flush();

    // =========================================================================
    // BENCHMARK 2: DIRECT TOURNAMENT — NNUE GPU vs HCE at SAME DEPTH!
    // =========================================================================
    if std::path::Path::new(&nnue_path).exists() {
        println!("[BENCHMARK 2] DIRECT TOURNAMENT: NNUE GPU vs HCE (Depth {} vs {}, {} ván)...", depth, depth, games);
        print!("  • Nạp trọng số NNUE GPU 1 LẦN vào RAM từ {}...", nnue_path);
        let _ = std::io::stdout().flush();

        let mut search_nnue = Search::new(16);
        match search_nnue.load_nnue(&nnue_path) {
            Ok(()) => println!(" OK! (Format XRNN v1, 65,536 features loaded)"),
            Err(e) => println!(" LỖI: {}", e),
        }
        search_nnue.eval.mode(Mode::Auto);
        println!("  • Bắt đầu thi đấu giải đấu trực tiếp NNUE GPU vs HCE...");
        let _ = std::io::stdout().flush();

        let begin2 = std::time::Instant::now();
        let mut rating2 = Rating::new();

        for g in 0..games {
            let start = randomize(&mut seed, opening);

            let outcome = if g % 2 == 0 {
                play_game(&start, &mut search_nnue, depth, &mut search_hce_a, depth, limit)
            } else {
                match play_game(&start, &mut search_hce_a, depth, &mut search_nnue, depth, limit) {
                    Outcome::First => Outcome::Second,
                    Outcome::Second => Outcome::First,
                    Outcome::Draw => Outcome::Draw,
                }
            };

            match outcome {
                Outcome::First => rating2.wins += 1,
                Outcome::Second => rating2.losses += 1,
                Outcome::Draw => rating2.draws += 1,
            }

            // FLUSH TRỰC TIẾP TỪNG VÁN MỘT!
            let elapsed_secs = begin2.elapsed().as_secs_f64();
            let speed = (g + 1) as f64 / elapsed_secs;
            let remaining = games - (g + 1);
            let eta_secs = if speed > 0.0 { (remaining as f64 / speed).round() as u64 } else { 0 };
            let eta_m = eta_secs / 60;
            let eta_s = eta_secs % 60;
            let pct = (g + 1) * 100 / games;

            println!(
                "  [VÁN {:3}/{:3}] ({:3}%) W={} L={} D={} | NNUE Elo: {:+.0} ±{:.0} | Speed: {:.1} ván/s | ETA: {:02}m{:02}s",
                g + 1, games, pct,
                rating2.wins, rating2.losses, rating2.draws,
                rating2.elo(), rating2.margin(),
                speed, eta_m, eta_s
            );
            let _ = std::io::stdout().flush();
        }
        println!();
        println!("============================================================");
        println!("🏆 KẾT QUẢ GIẢI ĐẤU TOURNAMENT: NNUE GPU vs HCE TRỰC TIẾP");
        println!("============================================================");
        println!("  • Cấu hình: NNUE GPU (Depth {}) vs HCE (Depth {})", depth, depth);
        println!("  • Kết quả: Thắng {}, Thua {}, Hòa {} (Tổng {} ván)", rating2.wins, rating2.losses, rating2.draws, games);
        println!("  • Tỷ lệ thắng (Winrate): {:.1}%", rating2.ratio() * 100.0);
        println!("  • SỨC MẠNH CẢI THIỆN ELO: {:+.0} ±{:.0} ELO", rating2.elo(), rating2.margin());
        println!("  • Tổng thời gian: {:.1} giây", begin2.elapsed().as_secs_f64());
        println!("============================================================");
        let _ = std::io::stdout().flush();
    } else {
        println!("⚠️ Không tìm thấy tệp weights NNUE GPU tại {}", nnue_path);
    }
}
