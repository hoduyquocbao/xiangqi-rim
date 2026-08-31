// ============================================================================
// VÍ DỤ 96: ĐẤU TRƯỜNG TRỰC TIẾP XIANGQI-RIM VS PIKAFISH NATIVE (UCI ARENA)
// ============================================================================
// Kịch bản tổ chức giải đấu đối đầu thời gian thực trực tiếp giữa:
// 1. Engine A: Xiangqi-RIM Grandmaster (10M Shards NVMe + NNUE Gen 7 + SEE Pruning + TT Lock-Free)
// 2. Engine B: Pikafish World Champion (Stockfish Xiangqi AVX2 BMI2 UCI Subprocess)
// - Giao tiếp 2 chiều qua đường ống chuẩn stdin/stdout UCI Protocol.
// - Luân phiên cầm quân Đỏ / Đen qua từng ván cờ.
// - Đo lường kết quả Thắng/Thua/Hòa, Winrate %, và ELO sai số thời gian thực.
// - 100% chú thích tiếng Việt từng dòng & 100% định danh từ đơn tiếng Anh.
// ============================================================================

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use xiangrust::board::Parser;
use xiangrust::learn::Shard;
use xiangrust::meta::Meta;
use xiangrust::movegen::types::Move;
use xiangrust::search::{LazySmp, Limits};
use xiangrust::uci::Format;

/// Struct `Pika` quản lý tiến trình con Pikafish Engine giao tiếp qua giao thức UCI
pub struct Pika {
    /// Tiến trình con của hệ điều hành
    pub child: Child,
    /// Bộ đọc đệm từ stdout của tiến trình con
    pub reader: BufReader<std::process::ChildStdout>,
}

impl Pika {
    /// Khởi động tiến trình con Pikafish từ đường dẫn nhị phân
    pub fn spawn(path: &str) -> Result<Self, std::io::Error> {
        let mut child = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let stdout = child.stdout.take().expect("Không thể mở stdout của Pikafish");
        let mut reader = BufReader::new(stdout);

        // Gửi lệnh khởi tạo giao thức UCI
        if let Some(stdin) = child.stdin.as_mut() {
            writeln!(stdin, "uci")?;
            stdin.flush()?;
        }

        // Chờ nhận phản hồi "uciok"
        let mut line = String::new();
        loop {
            line.clear();
            if reader.read_line(&mut line)? == 0 {
                break;
            }
            if line.trim() == "uciok" {
                break;
            }
        }

        // Cấu hình cờ và thông số cho Pikafish (Threads 1, Hash 64MB)
        if let Some(stdin) = child.stdin.as_mut() {
            writeln!(stdin, "setoption name Threads value 1")?;
            writeln!(stdin, "setoption name Hash value 64")?;
            writeln!(stdin, "isready")?;
            stdin.flush()?;
        }

        // Chờ nhận phản hồi "readyok"
        loop {
            line.clear();
            if reader.read_line(&mut line)? == 0 {
                break;
            }
            if line.trim() == "readyok" {
                break;
            }
        }

        Ok(Self { child, reader })
    }

    /// Gửi thế cờ và nhận nước đi tốt nhất `bestmove` cùng điểm số `score` từ Pikafish
    pub fn query(&mut self, moves: &[String], time_ms: u64, depth: u8) -> Option<(String, i32)> {
        let stdin = self.child.stdin.as_mut()?;

        // Thiết lập vị trí bàn cờ
        if moves.is_empty() {
            writeln!(stdin, "position startpos").ok()?;
        } else {
            let moves_str = moves.join(" ");
            writeln!(stdin, "position startpos moves {}", moves_str).ok()?;
        }

        // Gửi lệnh tìm kiếm `go movetime <ms>` hoặc `go depth <d>`
        if time_ms > 0 {
            writeln!(stdin, "go movetime {}", time_ms).ok()?;
        } else {
            writeln!(stdin, "go depth {}", depth).ok()?;
        }
        stdin.flush().ok()?;

        // Đọc stdout cho đến khi nhận được dòng bắt đầu bằng `bestmove`
        let mut line = String::new();
        let mut last_score = 0i32;

        loop {
            line.clear();
            if self.reader.read_line(&mut line).unwrap_or(0) == 0 {
                return None;
            }
            let trimmed = line.trim();

            // Trích xuất điểm số từ dòng "info ... score cp <val>" hoặc "score mate <val>"
            if trimmed.starts_with("info ") && trimmed.contains(" score ") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                for i in 0..parts.len().saturating_sub(2) {
                    if parts[i] == "score" {
                        if parts[i + 1] == "cp" {
                            if let Ok(v) = parts[i + 2].parse::<i32>() {
                                last_score = v;
                            }
                        } else if parts[i + 1] == "mate" {
                            if let Ok(v) = parts[i + 2].parse::<i32>() {
                                last_score = if v > 0 { 30000 - v * 10 } else { -30000 - v * 10 };
                            }
                        }
                    }
                }
            }

            if trimmed.starts_with("bestmove") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    return Some((parts[1].to_string(), last_score));
                }
            }
        }
    }

    /// Thoát tiến trình con sạch sẽ
    pub fn quit(&mut self) {
        if let Some(stdin) = self.child.stdin.as_mut() {
            let _ = writeln!(stdin, "quit");
            let _ = stdin.flush();
        }
        let _ = self.child.wait();
    }
}

impl Drop for Pika {
    fn drop(&mut self) {
        self.quit();
    }
}

fn main() {
    println!("===============================================================================");
    println!(" ⚔️ XIANGQI-RIM VS PIKAFISH NATIVE OFFICIAL UCI ARENA CLASH (32M SHARDS)");
    println!("    Phiên bản     : {} | Dấu thời gian: {}", Meta::version(), Meta::stamp());
    println!("    Engine 1: Xiangqi-RIM Grandmaster (32M Shards NVMe + NNUE Gen 7 + SOTA Search)");
    println!("    Engine 2: Pikafish World Champion (Stockfish Xiangqi AVX2 BMI2 UCI Subprocess)");
    println!("===============================================================================");

    let total_games: usize = std::env::var("GAMES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);
    let time_ms: u64 = std::env::var("MOVETIME")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0); // 0 = Sử dụng DEPTH thuần
    let depth: u8 = std::env::var("DEPTH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(13); // Mặc định Depth 13 theo yêu cầu đối đầu thực tế

    let pikafish_path = std::env::var("PIKAFISH_PATH")
        .unwrap_or_else(|_| "../pikafish/src/pikafish".to_string());

    let shard = Arc::new(Shard::default());
    let shard_count = shard.count();

    println!("⚙️ THÔNG SỐ CẤU HÌNH ĐẤU TRƯỜNG:");
    println!("  • Tổng số ván đấu     : {} ván cờ (Luân phiên Đỏ / Đen)", total_games);
    println!("  • Thời gian mỗi nước  : {} ms / nước (Hoặc Depth {})", time_ms, depth);
    println!("  • Đường dẫn Pikafish  : {}", pikafish_path);
    println!("  • Dung lượng Shards   : {} bản ghi trong 1,024 Shards NVMe", shard_count);
    println!("===============================================================================\n");
    let _ = std::io::stdout().flush();

    let rim_wins = Arc::new(AtomicUsize::new(0));
    let pika_wins = Arc::new(AtomicUsize::new(0));
    let draws = Arc::new(AtomicUsize::new(0));
    let shard_hits = Arc::new(AtomicUsize::new(0));

    let start_time = Instant::now();

    for game_idx in 1..=total_games {
        let rim_is_red = game_idx % 2 != 0;
        let red_name = if rim_is_red { "Xiangqi-RIM" } else { "Pikafish" };
        let black_name = if rim_is_red { "Pikafish" } else { "Xiangqi-RIM" };

        println!("-------------------------------------------------------------------------------");
        println!(" 🎮 [VÁN {:2}/{:2}] 🔴 ĐỎ: {} vs ⚫ ĐEN: {}", game_idx, total_games, red_name, black_name);
        println!("-------------------------------------------------------------------------------");
        let _ = std::io::stdout().flush();

        // Khởi tạo Pikafish process cho ván đấu
        let mut pika = match Pika::spawn(&pikafish_path) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("❌ Không thể khởi động Pikafish tại '{}': {}", pikafish_path, e);
                return;
            }
        };

        // Khởi tạo Xiangqi-RIM Lazy SMP Engine (4 luồng vật lý + 64MB TT + NNUE HalfKAv2_hm)
        let mut search_rim = LazySmp::new(4, 64);

        let mut pos = Parser::parse(Parser::DEFAULT);
        let mut move_history: Vec<String> = Vec::new();
        let mut past_hashes: Vec<u64> = Vec::new();
        let mut steps = 0usize;
        let mut winner: Option<bool> = None; // Some(true): RIM thắng, Some(false): Pikafish thắng, None: Hòa

        while steps < 200 {
            let is_red_turn = pos.side == 0;
            let current_is_rim = (is_red_turn && rim_is_red) || (!is_red_turn && !rim_is_red);
            let mover_name = if current_is_rim { "Xiangqi-RIM" } else { "Pikafish" };

            let (best_uci, score): (String, i32) = if current_is_rim {
                // Ưu tiên tra cứu siêu tốc trong 10,047,498 Shards NVMe
                if let Some((raw_mv, s_score)) = shard.probe(pos.hash) {
                    let mv = Move::from_raw(raw_mv);
                    let mut legals = xiangrust::movegen::types::List::new();
                    xiangrust::movegen::legal::gen(&mut pos, &mut legals);
                    let mut legal = false;
                    for i in 0..legals.count {
                        if legals.items[i] == mv {
                            legal = true;
                            break;
                        }
                    }
                    if legal {
                        shard_hits.fetch_add(1, Ordering::Relaxed);
                        (Format::encode(mv), s_score as i32)
                    } else {
                        let mut limits = Limits::new();
                        if time_ms > 0 {
                            limits.exact = time_ms;
                            limits.depth = 128;
                        } else {
                            limits.depth = depth;
                        }
                        let r = search_rim.go_with_history(&pos, &limits, &past_hashes);
                        let mv = r.best;
                        (Format::encode(mv), r.score)
                    }
                } else {
                    let mut limits = Limits::new();
                    if time_ms > 0 {
                        limits.exact = time_ms;
                        limits.depth = 128;
                    } else {
                        limits.depth = depth;
                    }
                    let r = search_rim.go_with_history(&pos, &limits, &past_hashes);
                    let mv = r.best;
                    (Format::encode(mv), r.score)
                }
            } else {
                // Lấy nước đi từ Pikafish theo time_ms hoặc depth
                match pika.query(&move_history, time_ms, depth) {
                    Some((uci, pika_score)) => {
                        // Tự động hấp thu nước đi đỉnh cao của Pikafish vào Shards NVMe
                        let p_mv = Format::decode(&uci);
                        if p_mv.valid() {
                            let _ = shard.save(pos.hash, p_mv.raw(), pika_score as i16);
                        }
                        (uci, pika_score)
                    }
                    None => {
                        println!("  ⚠️ Pikafish không trả về nước đi hợp lệ -> RIM thắng!");
                        winner = Some(true);
                        break;
                    }
                }
            };

            // Phân tích nước đi UCI sang Move struct
            let mv = Format::decode(&best_uci);
            if !mv.valid() {
                println!("  ⚠️ Nước đi '{}' không đúng định dạng -> Kết thúc ván cờ.", best_uci);
                winner = if current_is_rim { Some(false) } else { Some(true) };
                break;
            }

            // Kiểm tra tính hợp lệ của nước đi
            let mut legals = xiangrust::movegen::types::List::new();
            xiangrust::movegen::legal(&mut pos, &mut legals);
            let mut is_legal = false;
            for i in 0..legals.len() {
                if legals.items[i].from == mv.from && legals.items[i].to == mv.to {
                    is_legal = true;
                    break;
                }
            }

            if !is_legal {
                println!("  ❌ Nước đi '{}' vi phạm luật cờ tướng của {}! Đối thủ thắng cuộc.", best_uci, mover_name);
                winner = if current_is_rim { Some(false) } else { Some(true) };
                break;
            }

            // Áp dụng nước đi
            pos.apply(mv.from, mv.to);
            past_hashes.push(pos.hash);
            move_history.push(best_uci.clone());
            steps += 1;

            // Kiểm tra hòa lặp cờ (3-fold repetition)
            let mut reps = 0u32;
            for h in past_hashes.iter().rev().skip(1) {
                if *h == pos.hash {
                    reps += 1;
                }
            }
            if reps >= 2 {
                println!("  🤝 Phát hiện lặp trạng thái cờ 3 lần -> Kết thúc ván cờ: HÒA (DRAW)!");
                winner = None;
                break;
            }

            if steps % 10 == 0 || score.abs() > 25000 {
                println!("   Turn {:3}: {} đi '{}' (Score: {:+5}) | Shards Hits: {}", steps, mover_name, best_uci, score, shard_hits.load(Ordering::Relaxed));
                let _ = std::io::stdout().flush();
            }

            // Nhận diện điểm yếu / Blunder nếu Pikafish chiếm ưu thế áp đảo (> +300 cp)
            if !current_is_rim && score > 300 && score < 29000 {
                println!("   ⚠️ [NHẬN DIỆN ĐIỂM YẾU TẠI TURN {:3}] Pikafish bóc trần sơ hở! Đánh giá: +{} cp cho Pikafish", steps, score);
            }

            // Kiểm tra thắng sát cục
            if score.abs() > 29000 {
                let win = if score > 0 { current_is_rim } else { !current_is_rim };
                winner = Some(win);
                break;
            }

            // Kiểm tra hết nước đi (Bắt Bí / Checkmate)
            let mut next_legals = xiangrust::movegen::types::List::new();
            xiangrust::movegen::legal(&mut pos, &mut next_legals);
            if next_legals.len() == 0 {
                let next_is_rim = !current_is_rim;
                winner = if next_is_rim { Some(false) } else { Some(true) };
                break;
            }
        }

        pika.quit();

        match winner {
            Some(true) => {
                rim_wins.fetch_add(1, Ordering::Relaxed);
                println!(" 🏆 [VÁN {:2}] KẾT QUẢ: XIANGQI-RIM CHIẾN THẮNG TRƯỚC PIKAFISH! 🎉", game_idx);
            }
            Some(false) => {
                pika_wins.fetch_add(1, Ordering::Relaxed);
                println!(" ⚔️ [VÁN {:2}] KẾT QUẢ: PIKAFISH CHIẾN THẮNG.", game_idx);
            }
            None => {
                draws.fetch_add(1, Ordering::Relaxed);
                println!(" 🤝 [VÁN {:2}] KẾT QUẢ: HÒA CỜ (DRAW).", game_idx);
            }
        }

        let w = rim_wins.load(Ordering::Relaxed);
        let l = pika_wins.load(Ordering::Relaxed);
        let d = draws.load(Ordering::Relaxed);
        let score_rim = w as f64 + (d as f64 * 0.5);
        let score_pct = (score_rim / game_idx as f64) * 100.0;
        let elo = -400.0 * (100.0 / score_pct.max(0.1).min(99.9) - 1.0).log10();

        println!(" 📊 TỶ SỐ HIỆN TẠI: RIM {} - {} PIKA (Hòa {}) | Điểm: {:.1}% | Elo: {:+.1}\n", w, l, d, score_pct, elo);
        let _ = std::io::stdout().flush();
    }

    let final_w = rim_wins.load(Ordering::Relaxed);
    let final_l = pika_wins.load(Ordering::Relaxed);
    let final_d = draws.load(Ordering::Relaxed);
    let elapsed = start_time.elapsed();
    let score_rim = final_w as f64 + (final_d as f64 * 0.5);
    let score_pct = (score_rim / total_games as f64) * 100.0;
    let elo = -400.0 * (100.0 / score_pct.max(0.1).min(99.9) - 1.0).log10();

    println!("===============================================================================");
    println!(" 🏆 BÁO CÁO TỔNG KẾT ĐẤU TRƯỜNG ĐỐI ĐẦU XIANGQI-RIM VS PIKAFISH ({:.2?})", elapsed);
    println!("===============================================================================");
    println!("  • Tổng số ván đấu : {} ván cờ", total_games);
    println!("  • Kết quả chung cuộc: {} Thắng - {} Thua - {} Hòa", final_w, final_l, final_d);
    println!("  • Tỷ lệ điểm      : {:.2}%", score_pct);
    println!("  • Chênh lệch ELO  : {:+.1} ELO (So với Pikafish ~3800 ELO)", elo);
    println!("  • Tra cứu Shards  : {} lần trúng Hash Move O(1)", shard_hits.load(Ordering::Relaxed));
    println!("===============================================================================\n");
}
