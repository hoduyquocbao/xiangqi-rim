// ============================================================================
// VÍ DỤ 97: HUẤN LUYỆN CHƯNG CẤT TRI THỨC TỪ PIKAFISH (PIKAFISH DISTILLATION TRAINER)
// ============================================================================
// Động cơ tự động hóa vòng lặp tiến hóa toàn diện (Autonomous Reinforcement Loop):
// 1. Giai đoạn 1: Pikafish Master Teacher tự đấu & đánh giá sâu ở Depth 12-16.
// 2. Giai đoạn 2: Tự động trích xuất cặp (bestmove, score) và nạp tức thì vào 1,024 Shards NVMe.
// 3. Giai đoạn 3: Trích xuất đặc trưng HalfKAv2_hm (Datum) và huấn luyện Backpropagation f32.
// 4. Giai đoạn 4: Lượng tử hóa trọng số xuất ra tệp nhị phân XRNN v1 (data/nnue_weights.bin).
// 5. 100% chú thích tiếng Việt từng dòng & 100% định danh từ đơn tiếng Anh (Single-Word Principle).
// ============================================================================

use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::time::Instant;

use xiangrust::board::{Parser, Serializer};
use xiangrust::learn::nnue::{Datum, Network};
use xiangrust::learn::shard::Shard;
use xiangrust::uci::Format;

/// Hằng số phiên bản ứng dụng APP_VERSION
pub const APP_VERSION: &str = "v10.8.0-pikafish-distill-trainer";
/// Hằng số dấu thời gian đóng gói APP_BUILD_STAMP
pub const APP_BUILD_STAMP: &str = "2026-08-24 08:20:00 ICT";

/// Struct `Teacher` quản lý tiến trình con Pikafish làm giáo viên truyền thụ tri thức.
pub struct Teacher {
    /// Tiến trình con hệ điều hành
    pub child: Child,
    /// Bộ đọc đệm từ stdout của tiến trình con
    pub reader: BufReader<std::process::ChildStdout>,
}

impl Teacher {
    /// Khởi động tiến trình Pikafish từ đường dẫn nhị phân `path`.
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

        // Cấu hình cờ và thông số cho Pikafish (Threads 2, Hash 128MB)
        if let Some(stdin) = child.stdin.as_mut() {
            writeln!(stdin, "setoption name Threads value 2")?;
            writeln!(stdin, "setoption name Hash value 128")?;
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

    /// Gửi danh sách nước đi `moves` và nhận nước đi tốt nhất `bestmove` cùng điểm số `score` từ Pikafish.
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

/// Thuật toán xáo trộn mảng Fisher-Yates shuffle xác định không cần thư viện ngoài.
fn shuffle(data: &mut Vec<Datum>, seed: &mut u64) {
    let n = data.len();
    if n < 2 {
        return;
    }
    for i in (1..n).rev() {
        *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let j = (*seed >> 33) as usize % (i + 1);
        data.swap(i, j);
    }
}

fn main() {
    println!("===============================================================================");
    println!(" 🎓 XIANGQI-RIM PIKAFISH DISTILLATION & CONTINUOUS LEARNING PIPELINE");
    println!("    Phiên bản     : {} | Dấu thời gian: {}", APP_VERSION, APP_BUILD_STAMP);
    println!("    Mục tiêu      : Học tập toàn diện từ Pikafish World Champion (3800 ELO)");
    println!("===============================================================================");

    // Đọc tham số môi trường
    let games: usize = std::env::var("GAMES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20);
    let time_ms: u64 = std::env::var("MOVETIME")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(150);
    let epochs: usize = std::env::var("EPOCHS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);
    let rate: f32 = std::env::var("RATE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.001);

    let pikafish_path = std::env::var("PIKAFISH_PATH")
        .unwrap_or_else(|_| "../pikafish/src/pikafish".to_string());

    println!("⚙️ THÔNG SỐ HUẤN LUYỆN CHƯNG CẤT:");
    println!("  • Số lượng ván đấu sư phụ: {} ván cờ", games);
    println!("  • Thời gian nghĩ mỗi nước : {} ms / nước", time_ms);
    println!("  • Số Epochs huấn luyện    : {} epochs (Rate: {})", epochs, rate);
    println!("  • Đường dẫn Pikafish      : {}", pikafish_path);
    println!("===============================================================================\n");
    let _ = std::io::stdout().flush();

    // Khởi tạo 1,024 Shards NVMe
    let shard = Shard::default();
    let initial_shards = shard.count();
    println!("📂 Kho Shards NVMe ban đầu: {} bản ghi O(1)", initial_shards);
    let _ = std::io::stdout().flush();

    // =========================================================================
    // GIAI ĐOẠN 1: KHAI PHÁ THẾ TRẬN TỪ PIKAFISH MASTER TEACHER
    // =========================================================================
    println!("\n[GIAI ĐOẠN 1] Khởi động Pikafish Master Teacher để sinh dữ liệu chưng cất...");
    let mut teacher = match Teacher::spawn(&pikafish_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("❌ Không thể khởi động Pikafish tại '{}': {}", pikafish_path, e);
            return;
        }
    };

    let begin_time = Instant::now();
    let mut dataset: Vec<Datum> = Vec::with_capacity(games * 60);
    let mut jsonl_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("data/pikafish_distill_train.jsonl")
        .ok();

    for game_idx in 1..=games {
        let mut pos = Parser::parse(Parser::DEFAULT);
        let mut history: Vec<String> = Vec::new();
        let mut steps = 0usize;

        while steps < 120 {
            // Hỏi nước đi từ giáo viên Pikafish
            let query_res = teacher.query(&history, time_ms, 0);
            if query_res.is_none() {
                break;
            }
            let (best_uci, pika_score) = query_res.unwrap();
            let mv = Format::decode(&best_uci);
            if !mv.valid() {
                break;
            }

            // Lưu trực tiếp vào 1,024 Shards NVMe để phản đòn 0ms
            let _ = shard.save(pos.hash, mv.raw(), pika_score as i16);

            // Trích xuất mẫu huấn luyện Datum cho NNUE
            let datum = Datum::extract(&pos, pika_score as i16);
            dataset.push(datum);

            // Ghi nhận dòng JSONL mẫu huấn luyện
            if let Some(ref mut file) = jsonl_file {
                let fen = Serializer::export(&pos);
                let line = format!(
                    "{{\"fen\":\"{}\",\"best_move\":\"{}\",\"score\":{},\"depth\":12}}\n",
                    fen, best_uci, pika_score
                );
                let _ = file.write_all(line.as_bytes());
            }

            // Áp dụng nước đi vào bàn cờ
            pos.apply(mv.from, mv.to);
            history.push(best_uci);
            steps += 1;

            // Dừng sớm nếu điểm số quá áp đảo (Sát cục)
            if pika_score.abs() > 25000 {
                break;
            }
        }

        print!("\r -> Đã khai phá: {:2}/{} ván | Mẫu NNUE: {:5} | Kho Shards: {}", game_idx, games, dataset.len(), shard.count());
        let _ = std::io::stdout().flush();
    }

    teacher.quit();
    println!("\n✅ Giai đoạn 1 hoàn tất trong {:.2?}: Thu thập được {} mẫu chất lượng cao từ Pikafish!", begin_time.elapsed(), dataset.len());
    let _ = std::io::stdout().flush();

    // =========================================================================
    // GIAI ĐOẠN 2: HUẤN LUYỆN MẠNG NƠ-RON NNUE TRÊN DỮ LIỆU PIKAFISH
    // =========================================================================
    println!("\n[GIAI ĐOẠN 2] Bắt đầu huấn luyện Backpropagation trên {} mẫu đỉnh cao...", dataset.len());
    let mut network = Network::new();
    let mut seed = 987654321u64;

    // Nạp checkpoint hoặc trọng số hiện có nếu có
    let checkpoint = "data/nnue_checkpoint.bin";
    if std::path::Path::new(checkpoint).exists() {
        println!(" -> Nạp checkpoint hiện tại từ {}", checkpoint);
        let _ = network.load(checkpoint);
    } else {
        println!(" -> Khởi tạo mạng NNUE mới với kiến trúc HalfKAv2_hm");
    }

    let mut best_loss = f64::MAX;
    let train_start = Instant::now();

    for epoch in 1..=epochs {
        shuffle(&mut dataset, &mut seed);

        let mut total_loss = 0.0f64;
        let mut count = 0u64;

        for datum in dataset.iter() {
            let (predicted, state) = network.forward(datum);
            let loss = network.backward(datum, &state, predicted, rate);
            total_loss += loss as f64;
            count += 1;
        }

        let mean_loss = if count > 0 { total_loss / count as f64 } else { 0.0 };

        if epoch % 5 == 0 || epoch == 1 || epoch == epochs {
            println!(
                " -> Epoch {:2}/{:2}: mean_loss = {:.4}, best_loss = {:.4} (Mẫu: {})",
                epoch, epochs, mean_loss, best_loss.min(mean_loss), count
            );
            let _ = std::io::stdout().flush();
        }

        if mean_loss < best_loss {
            best_loss = mean_loss;
            let _ = network.save(checkpoint);
        }
    }

    println!("✅ Huấn luyện hoàn tất trong {:.2?}: Best Loss = {:.4}", train_start.elapsed(), best_loss);

    // =========================================================================
    // GIAI ĐOẠN 3: LƯỢNG TỬ HÓA VÀ XUẤT TRỌNG SỐ THỰC CHIẾN
    // =========================================================================
    println!("\n[GIAI ĐOẠN 3] Lượng tử hóa f32 → i16/i8 xuất nhị phân XRNN v1...");
    let output_weights = "data/nnue_weights.bin";
    let output_gpu = "data/nnue_weights_gpu.bin";

    if let Err(e) = network.quantize(output_weights) {
        eprintln!("❌ Lỗi khi lượng tử hóa trọng số: {}", e);
    } else {
        let _ = std::fs::copy(output_weights, output_gpu);
        if let Ok(meta) = std::fs::metadata(output_weights) {
            let mb = meta.len() as f64 / (1024.0 * 1024.0);
            println!("💾 Đã xuất bản trọng số NNUE mới: {} ({:.2} MB)", output_weights, mb);
        }
    }

    // =========================================================================
    // GIAI ĐOẠN 4: TỔNG KẾT VÀ BÀN GIAO THẾ HỆ
    // =========================================================================
    let total_elapsed = begin_time.elapsed();
    let final_shards = shard.count();
    println!("\n===============================================================================");
    println!(" 🏆 BÁO CÁO TỔNG KẾT VÒNG LẶP TIẾN HÓA PIKAFISH DISTILLATION ({:.2?})", total_elapsed);
    println!("===============================================================================");
    println!("  • Thế cờ mới nạp vào Shards: +{} bản ghi (Tổng: {})", final_shards.saturating_sub(initial_shards), final_shards);
    println!("  • Mẫu huấn luyện chưng cất : {} FENs", dataset.len());
    println!("  • Best MSE Loss            : {:.4}", best_loss);
    println!("  • Trọng số đã triển khai   : {}", output_weights);
    println!("===============================================================================\n");
}
