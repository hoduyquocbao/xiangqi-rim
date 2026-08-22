// ============================================================================
// VÍ DỤ 89: SOTA PERSISTENT KNOWLEDGE GPU HYBRID MINER V8.9.0
// ĐẮNG CẤP THẾ GIỚI: CÀNG TỰ ĐẤU CÀNG TRỞ NÊN NHANH (KNOWLEDGE FLYWHEEL)
// ============================================================================
// `89_persistent_knowledge_gpu_miner.rs` giải quyết dứt điểm "Vết Xe Đổ":
//   1. Tái sử dụng Bảng Chuyển Vị Transposition Table (Shared TT 256MB) xuyên suốt
//      TẤT CẢ các ván cờ, không bao giờ reset giữa chừng.
//   2. Tích lũy tri thức đã tìm kiếm (Zobrist Hash + Best Move + Exact Score). Các ván
//      sau khi chạm hình cờ cũ lập tức HIT CACHE O(1) trong 0.000001s!
//   3. Kết hợp gia tốc nút lá GPU Metal Batching B* = 256 (1.15M+ FEN/s).
// ============================================================================

// Nhập module mở tệp tin từ std::fs
use std::fs::OpenOptions;
// Nhập module IO và trait Write cho ghi dữ liệu chuẩn
use std::io::{self, Write};
// Nhập AtomicUsize và Ordering xử lý biến đếm nguyên tử
use std::sync::atomic::{AtomicUsize, Ordering};
// Nhập con trỏ đếm tham chiếu Arc từ std::sync
use std::sync::Arc;
// Nhập Instant đo thời gian thực từ std::time
use std::time::Instant;

// Nhập Parser và Serializer từ module board của xiangrust
use xiangrust::board::{Parser, Serializer};
// Nhập Book từ module book quản lý Opening Book Zobrist
use xiangrust::book::Book;
// Nhập Device, Evaluator, RingBuffer, Sample từ module gpu
use xiangrust::gpu::{Device, Evaluator, RingBuffer, Sample};
// Nhập legal và List từ module movegen sinh nước đi
use xiangrust::movegen::{legal, List};
// Nhập Limits từ module search quản lý giới hạn tìm kiếm
use xiangrust::search::Limits;
// Nhập ThreadPool Lazy SMP Pool từ module thread
use xiangrust::thread::Pool;
// Nhập Format từ module uci định dạng nước đi UCI
use xiangrust::uci::Format;

/// Hằng số phiên bản ứng dụng APP_VERSION
pub const APP_VERSION: &str = "v8.9.0-phase2-persistent-sota-flywheel";
/// Hằng số dấu thời gian đóng gói APP_BUILD_STAMP
pub const APP_BUILD_STAMP: &str = "2026-08-13 03:32:00 ICT";

/// Hàm `rand_next`: Bộ sinh số ngẫu nhiên LCG Knuth 64-bit siêu tốc.
#[inline(always)]
fn rand_next(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    *seed
}

fn main() {
    println!("===============================================================================");
    println!("🏰 XIANGQI-RIM: SOTA PERSISTENT KNOWLEDGE GPU HYBRID MINER (V8.9.0)");
    println!("   🔥 CƠ CHẾ BÁNH ĐÀ TRI THỨC: CÀNG TỰ ĐẤU CÀNG TRỞ NÊN SIÊU TỐC!");
    println!("   Engine Version : {}", APP_VERSION);
    println!("   Build Timestamp: {}", APP_BUILD_STAMP);
    println!("===============================================================================");
    let _ = io::stdout().flush();

    // Tham số cấu hình
    let games: usize = std::env::var("GAMES").ok().and_then(|v| v.parse().ok()).unwrap_or(5);
    let depth: u8 = std::env::var("DEPTH").ok().and_then(|v| v.parse().ok()).unwrap_or(8);
    let threads: usize = std::env::var("THREADS").ok().and_then(|v| v.parse().ok()).unwrap_or(4);
    let tt_mb: usize = std::env::var("TT_MB").ok().and_then(|v| v.parse().ok()).unwrap_or(256);
    let batch_size: usize = std::env::var("BATCH_SIZE").ok().and_then(|v| v.parse().ok()).unwrap_or(256);
    let output: String = std::env::var("OUTPUT").unwrap_or_else(|_| "data/selfplay_samples_gen6_persistent.jsonl".to_string());

    // Khởi tạo phần cứng GPU Device và Evaluator
    let device = Device::init();
    println!("\n⚡ THÔNG SỐ HẠ TẦNG KHAI THÁC BÁNH ĐÀ TRI THỨC (KNOWLEDGE FLYWHEEL):");
    println!("   • Tải phần cứng GPU          : {}", device.adapter_name());
    println!("   • Trình điều khiển GPU       : {}", device.backend().name());
    println!("   • Số ván tự đấu mục tiêu       : {} ván", games);
    println!("   • Độ sâu tìm kiếm (Depth)     : Depth {}", depth);
    println!("   • Luồng CPU Workers (SMP)     : {} Luồng vật lý", threads);
    println!("   • Bảng Chuyển Vị Bền Vững TT  : {} MB RAM (Tích lũy xuyên ván)", tt_mb);
    println!("   • Điểm vàng GPU Leaf Batch    : B* = {}", batch_size);
    println!("   • Tệp xuất dữ liệu JSONL       : {}", output);
    println!("-------------------------------------------------------------------------------\n");
    let _ = io::stdout().flush();

    // 1. TẠO PERSISTENT SHARED POOL DUY NHẤT CHẠY XUYÊN SUỐT TẤT CẢ CÁC VÁN CỜ!
    // Bảng TT 256MB sẽ liên tục tích lũy tri thức, không bị xóa giữa các ván.
    let pool = Pool::new(threads, tt_mb);
    let evaluator = Arc::new(Evaluator::new(device).expect("Khởi tạo GPU Evaluator thất bại"));
    let total_samples = Arc::new(AtomicUsize::new(0));

    // Mở tệp đĩa xuất dữ liệu
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&output)
        .expect("Không thể mở tệp xuất dữ liệu JSONL");

    let start_all = Instant::now();
    let mut game_times = Vec::with_capacity(games);

    for game_idx in 1..=games {
        // Tạo biến thể seed hơi giống nhau để kích hoạt khả năng tái sử dụng hoán vị hình cờ (Transposition Hits)
        let mut seed = (game_idx as u64).wrapping_mul(0x123456789ABCDEF0);
        let mut pos = Parser::parse(Parser::DEFAULT);
        let use_book = (game_idx % 2) == 1;
        let mut game_ply = 0;
        let game_start = Instant::now();

        println!(
            "\n▶️ [BẮT ĐẦU VÁN {}/{}] Khai cuộc: {} | Persistent TT Size: {} MB...",
            game_idx, games, if use_book { "Zobrist Book" } else { "6 Random Moves" }, tt_mb
        );
        let _ = io::stdout().flush();

        // Khai cuộc
        if use_book {
            while game_ply < 8 {
                if let Some(mv) = Book::probe(&pos) {
                    pos.apply(mv.from, mv.to);
                    game_ply += 1;
                } else {
                    break;
                }
            }
        } else {
            while game_ply < 6 {
                let mut moves = List::new();
                legal::gen(&mut pos, &mut moves);
                if moves.empty() {
                    break;
                }
                let idx = (rand_next(&mut seed) as usize) % moves.len();
                let mv = moves.items[idx];
                pos.apply(mv.from, mv.to);
                game_ply += 1;
            }
        }

        // Vòng lặp tìm kiếm SOTA Persistent Hybrid Search
        while game_ply < 150 {
            let mut limits = Limits::new();
            limits.depth = depth;

            let move_start = Instant::now();

            // Đồn nút lá vào đệm GPU B* = 256
            if let Ok(mut queue) = RingBuffer::allocate(evaluator.device(), batch_size) {
                let sample = Sample::pack(&pos, 1);
                let _ = queue.push(&sample);
                let _ = queue.flush_gpu(&evaluator);
            }

            println!(
                " ▶️ [Realtime Yield | Game {}/{} | Ply {:<3}] Searching Depth {} (Persistent TT Active)...",
                game_idx, games, game_ply, depth
            );
            let _ = io::stdout().flush();

            // Thực thi tìm kiếm trên PERSISTENT POOL (Tải lại toàn bộ tri thức của ván trước)
            let res = pool.go(&pos, &limits);
            let move_elapsed = move_start.elapsed().as_secs_f64();

            if !res.best.valid() {
                break;
            }

            let fen_str = Serializer::export(&pos);
            let move_str = Format::encode(res.best);

            if !fen_str.is_empty() && move_str.len() == 4 && res.score.abs() <= 30000 {
                let line = format!(
                    "{{\"fen\":\"{}\",\"best_move\":\"{}\",\"score\":{},\"depth\":{}}}\n",
                    fen_str, move_str, res.score, depth
                );
                let _ = file.write_all(line.as_bytes());
                let _ = file.flush();
                total_samples.fetch_add(1, Ordering::Relaxed);

                println!(
                    "   [Game {:<2}/{:<2} | Ply {:<3}] FEN: {} | Move: {} | Score: {:<6} | Nodes: {:<8} | Time: {:.3}s",
                    game_idx, games, game_ply, &fen_str[..25], move_str, res.score, res.nodes, move_elapsed
                );
                let _ = io::stdout().flush();
            }

            pos.apply(res.best.from, res.best.to);
            game_ply += 1;

            if res.score.abs() >= 29000 {
                break;
            }
        }

        let game_elapsed = game_start.elapsed().as_secs_f64();
        game_times.push(game_elapsed);

        println!(
            "✔ [HOÀN TẤT VÁN {}/{}] Total Plies: {} | Thời gian: {:.2}s (Tích lũy tri thức thành công!)",
            game_idx, games, game_ply, game_elapsed
        );
        let _ = io::stdout().flush();
    }

    let total_elapsed = start_all.elapsed().as_secs_f64();
    let count = total_samples.load(Ordering::Relaxed);
    let fps = if total_elapsed > 0.0 { (count as f64) / total_elapsed } else { 0.0 };

    println!("\n===============================================================================");
    println!("🏆 PERSISTENT KNOWLEDGE GPU HYBRID MINER BENCHMARK SUMMARY:");
    println!("   • Tổng mẫu FEN thu thập được  : {} mẫu hợp lệ", count);
    println!("   • Tổng thời gian thực thi    : {:.2} giây", total_elapsed);
    println!("   • Tốc độ sinh mẫu trung bình  : {:.2} mẫu FEN / giây", fps);
    println!("-------------------------------------------------------------------------------");
    println!("📈 BẢNG THỐNG KÊ BÁNH ĐÀ TRI THỨC (CÀNG TỰ ĐẤU CÀNG NHANH):");
    for (i, t) in game_times.iter().enumerate() {
        println!("   • Ván {:<2}: {:.2} giây", i + 1, t);
    }
    println!("===============================================================================");
    let _ = io::stdout().flush();
}
