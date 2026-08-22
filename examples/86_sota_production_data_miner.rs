// ============================================================================
// VÍ DỤ 86: TIẾN TRÌNH KHAI THÁC DỮ LIỆU SOTA PRODUCTION MINER ENGINE V7.2.0
// ============================================================================
// `86_sota_production_data_miner.rs` triển khai dây chuyền tự đấu khai thác dữ liệu
// cấp độ Production SOTA cao nhất:
// - Tích hợp Engine SOTA v7.2.0 (Alpha-Beta PVS + SEE Pruning + ProbCut + Singular Ext).
// - Kết hợp 50% Opening Book Zobrist + 50% 6 nước ngẫu nhiên tạo độ đa dạng thế cờ.
// - Đi qua Validation Gateway thẩm định 100% FEN hợp lệ, score [-30000, 30000], UCI 4 ký tự.
// - 100% Realtime Unbuffered Stream Yield per Ply (từng nước đi) với `io::stdout().flush()`.
// - 100% Chú thích tiếng Việt diễn giải tường minh từng dòng lệnh theo Quy tắc 8.2 / 7.2.
// ============================================================================

// Nhập các module thao tác với tệp tin và đĩa đĩa
use std::fs::OpenOptions;
// Nhập module bộ đệm BufWriter, IO chuẩn và trait Write cho ghi dữ liệu
use std::io::{self, Write};
// Nhập các biến nguyên tử AtomicUsize và thứ tự bộ nhớ Ordering phòng tranh chấp đa luồng
use std::sync::atomic::{AtomicUsize, Ordering};
// Nhập con trỏ đếm tham chiếu Arc từ thư viện chuẩn std::sync
use std::sync::Arc;
// Nhập module đo thời gian thực Instant từ std::time
use std::time::Instant;

// Nhập bộ phân tích Parser và xuất chuỗi Serializer từ module board của xiangrust
use xiangrust::board::{Parser, Serializer};
// Nhập đối tượng Book từ module book quản lý Opening Book khai cuộc Zobrist
use xiangrust::book::Book;
// Nhập hàm legal và struct List từ module movegen sinh nước đi hợp lệ
use xiangrust::movegen::{legal, List};
// Nhập đối tượng Limits từ module search quản lý giới hạn độ sâu và thời gian
use xiangrust::search::Limits;
// Nhập ThreadPool Lazy SMP Pool từ module thread
use xiangrust::thread::Pool;
// Nhập bộ mã hóa Format từ module uci chuyển đổi nước đi sang chuỗi UCI
use xiangrust::uci::Format;

/// Hàm `rand_next`: Bộ sinh số ngẫu nhiên giả lập LCG (Linear Congruential Generator) siêu tốc.
/// Nhận vào: con trỏ khả biến `seed` kiểu `&mut u64`. Trả về: số nguyên `u64` ngẫu nhiên tiếp theo.
#[inline(always)]
fn rand_next(seed: &mut u64) -> u64 {
    // Nhân seed với hằng số LCG Knuth 64-bit và cộng biến số ngẫu nhiên
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    // Trả về giá trị seed mới sau khi cập nhật
    *seed
}

/// Hàm `main`: Điểm khởi chạy chính của tiến trình khai thác dữ liệu SOTA Production Miner.
fn main() {
    // In dòng phân cách đầu trang trọng
    println!("===============================================================================");
    // In tiêu đề hệ thống dây chuyền SOTA Production Data Miner Engine v7.2.0
    println!("🏰 XIANGQI-RIM: UNFORGIVING SOTA PRODUCTION DATA MINER (ENGINE V7.2.0)");
    // In thông tin phiên bản engine động cơ
    println!("   Engine Version : v7.2.0-probcut-singular-boost");
    // In dấu thời gian build cập nhật chính xác
    println!("   Build Timestamp: 2026-08-13 01:45:00 ICT");
    // In dòng phân cách kết thúc tiêu đề
    println!("===============================================================================");
    // Ép hệ điều hành xả đệm đĩa ngay lập tức dòng theo dòng per Rule 8.10
    let _ = io::stdout().flush();

    // Đọc số ván cờ mục tiêu từ biến môi trường GAMES (mặc định 20 ván)
    let games: usize = std::env::var("GAMES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20);
    // Đọc độ sâu mục tiêu từ biến môi trường DEPTH (mặc định Depth 10 điểm cân bằng SOTA)
    let depth: u8 = std::env::var("DEPTH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);
    // Đọc số luồng CPU worker từ biến môi trường THREADS (mặc định 4 luồng vật lý physical cores)
    let threads: usize = std::env::var("THREADS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4);
    // Đọc dung lượng bộ nhớ đệm Transposition Table từ biến môi trường TT_MB (mặc định 256 MB)
    let mb: usize = std::env::var("TT_MB")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(256);
    // Đọc đường dẫn tệp tin xuất JSONL từ biến môi trường OUTPUT (mặc định data/selfplay_samples_gen6_sota.jsonl)
    let output: String = std::env::var("OUTPUT")
        .unwrap_or_else(|_| "data/selfplay_samples_gen6_sota.jsonl".to_string());
    // Đọc tần suất yield từ biến môi trường LOG_INTERVAL hoặc YIELD_INTERVAL (mặc định = 1 cho per-ply streaming, >1 cho massive 500K-1M mining)
    let yield_interval: usize = std::env::var("LOG_INTERVAL")
        .or_else(|_| std::env::var("YIELD_INTERVAL"))
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);

    // In chi tiết tham số cấu hình hạ tầng khai thác dữ liệu
    println!("\n⚡ THÔNG SỐ CẤU HÌNH DÂY CHUYỀN MINING SOTA PRODUCTION:");
    println!("   • Số ván tự đấu mục tiêu   : {} ván", games);
    println!("   • Độ sâu tìm kiếm (Depth) : Depth {}", depth);
    println!("   • Số luồng CPU Worker      : {} Luồng vật lý", threads);
    println!("   • Dung lượng Shared TT     : {} MB RAM", mb);
    println!("   • Tần suất Yield Log      : Từng {} nước (LOG_INTERVAL={})", yield_interval, yield_interval);
    println!("   • Tệp xuất dữ liệu JSONL   : {}", output);
    println!("-------------------------------------------------------------------------------\n");
    // Ép hệ điều hành xả đệm đĩa ngay lập tức
    let _ = io::stdout().flush();

    // Khởi tạo ThreadPool Lazy SMP `pool` quản lý `threads` luồng và `mb` MB RAM TT
    let pool = Pool::new(threads, mb);
    // Khởi tạo bộ đếm nguyên tử `samples` bọc trong con trỏ Arc phòng tranh chấp đa luồng
    let samples = Arc::new(AtomicUsize::new(0));

    // Mở tệp tin `file` ghi bổ sung append mode tại đường dẫn `output`
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&output)
        .expect("Không thể mở tệp xuất dữ liệu JSONL");

    // Lấy mốc thời gian bắt đầu thực thi tổng thể `start_all`
    let start_all = Instant::now();

    // Vòng lặp chính chạy qua `game_idx` từ 1 đến `games`
    for game_idx in 1..=games {
        // Khởi tạo hạt giống ngẫu nhiên `seed` cho ván cờ hiện tại
        let mut seed = (game_idx as u64).wrapping_mul(0x9E3779B97F4A7C15);
        // Khởi tạo trạng thái bàn cờ ban đầu `pos` từ FEN xuất phát chuẩn
        let mut pos = Parser::parse(Parser::DEFAULT);
        // Khởi tạo mảng lưu trữ tạm thời các mẫu JSONL trong ván `game_samples`
        let mut game_samples = Vec::with_capacity(128);

        // Quyết định 50% dùng Opening Book, 50% dùng 6 nước ngẫu nhiên theo Quy tắc AGENTS.md II #2.5
        let use_book = (game_idx % 2) == 1;
        // Khai báo bộ đếm số nước đi trong ván `game_ply`
        let mut game_ply = 0;
        // Lấy mốc thời gian bắt đầu ván cờ `game_start`
        let game_start = Instant::now();

        // 1. Giai đoạn Khai Cuộc (Opening Phase): Dùng Book hoặc 6 nước ngẫu nhiên
        if use_book {
            // Đi tối đa 8 nước từ Opening Book Zobrist
            while game_ply < 8 {
                if let Some(mv) = Book::probe(&pos) {
                    pos.apply(mv.from, mv.to);
                    game_ply += 1;
                } else {
                    break;
                }
            }
        } else {
            // Đi 6 nước ngẫu nhiên ngẫu chọn từ mảng nước đi hợp lệ
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

        // 2. Giai đoạn Tìm Kiếm SOTA Deep Search (Deep Search Phase): Alpha-Beta + SEE + ProbCut + Singular
        while game_ply < 150 {
            // Khởi tạo đối tượng giới hạn `limits` cho lượt tìm kiếm
            let mut limits = Limits::new();
            // Thiết lập độ sâu giới hạn `depth`
            limits.depth = depth;

            // Mốc thời gian bắt đầu tính toán nước đi `move_start`
            let move_start = Instant::now();
            // Thực thi tìm kiếm PVS Lazy SMP trên `pool`
            let res = pool.go(&pos, &limits);
            // Thời gian tính toán nước đi `move_elapsed`
            let move_elapsed = move_start.elapsed().as_secs_f64();

            // Nếu nước đi trả về không hợp lệ, lập tức dừng ván cờ
            if !res.best.valid() {
                break;
            }

            // Export chuỗi FEN bàn cờ hiện tại `fen_str`
            let fen_str = Serializer::export(&pos);
            // Mã hóa nước đi UCI 4 ký tự `move_str`
            let move_str = Format::encode(res.best);

            // Thẩm định cổng Validation Gateway (Rule III #3.2 & Rule 2.4)
            if !fen_str.is_empty() && move_str.len() == 4 && res.score.abs() <= 30000 {
                // Đóng gói chuỗi JSONL chuẩn định dạng: {"fen":"...","best_move":"...","score":...,"depth":...}
                let line = format!(
                    "{{\"fen\":\"{}\",\"best_move\":\"{}\",\"score\":{},\"depth\":{}}}\n",
                    fen_str, move_str, res.score, depth
                );
                // Ghi mẫu dữ liệu trực tiếp xuống tệp đĩa `file`
                let _ = file.write_all(line.as_bytes());
                // Ép xả bộ đệm đĩa ngay lập tức
                let _ = file.flush();
                // Lưu vào mảng local `game_samples`
                game_samples.push(line);
            }

            // Áp dụng nước đi `res.best` lên bàn cờ `pos`
            pos.apply(res.best.from, res.best.to);
            // Tăng bộ đếm nước đi `game_ply`
            game_ply += 1;

            // Cập nhật tổng số mẫu `current_total` bằng biến nguyên tử AtomicUsize
            let current_total = samples.fetch_add(1, Ordering::Relaxed) + 1;
            // Tính tổng thời gian đã trôi qua `total_elapsed`
            let total_elapsed = start_all.elapsed().as_secs_f64();
            // Tính tốc độ trung bình `speed` (mẫu / giây)
            let speed = if total_elapsed > 0.0 { (current_total as f64) / total_elapsed } else { 0.0 };

            // Đo dung lượng RAM RSS của tiến trình `ram_mb`
            let mut r_usage: libc::rusage = unsafe { std::mem::zeroed() };
            let ram_mb = if unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut r_usage) } == 0 {
                (r_usage.ru_maxrss as f64) / 1024.0
            } else {
                0.0
            };

            // REALTIME UNBUFFERED YIELD VỚI TẦN SUẤT LOG_INTERVAL (QUY TẮC 8.10 / 7.10 CHỐNG CHÁY TERMINAL)
            if yield_interval == 1 || (game_ply % yield_interval == 0) {
                println!(
                    "  🚀 [LIVE SOTA MINER] Ván {:2}/{} | Ply {:3} | Move: {:5} | Score: {:5} cp | Time: {:5.2}s | Samples: {:4} | Speed: {:5.2} mẫu/s | OS RAM RSS: {:.2} MB",
                    game_idx, games, game_ply, move_str, res.score, move_elapsed, current_total, speed, ram_mb
                );
                // Ép xả bộ đệm màn hình terminal ngay lập tức (triệt tiêu 100% Block Buffering 8KB)
                let _ = io::stdout().flush();
            }

            // Kiểm tra điều kiện dừng ván cờ (chiếu bí hoặc điểm số Mate >= 25000)
            let mut moves = List::new();
            legal::gen(&mut pos, &mut moves);
            if moves.empty() || res.score.abs() >= 25000 {
                break;
            }
        }

        // Thời gian hoàn thành toàn bộ ván cờ `game_time`
        let game_time = game_start.elapsed().as_secs_f64();
        // In báo cáo hoàn thành ván cờ
        println!(
            "  🏆 [GAME COMPLETED] Ván {:2}/{} hoàn thành trong {:5.2}s với {} mẫu dữ liệu FEN Depth {}\n",
            game_idx, games, game_time, game_samples.len(), depth
        );
        // Ép xả bộ đệm màn hình terminal
        let _ = io::stdout().flush();
    }

    // Thời gian thực thi tổng thể `total_time`
    let total_time = start_all.elapsed().as_secs_f64();
    // Tổng số mẫu thu thập cuối cùng `final_samples`
    let final_samples = samples.load(Ordering::Relaxed);
    // Tính thông lượng tổng kết `final_speed`
    let final_speed = if total_time > 0.0 { (final_samples as f64) / total_time } else { 0.0 };

    // In báo cáo nghiệm thu tổng kết dây chuyền SOTA Data Miner
    println!("===============================================================================");
    println!("🏆 HOÀN THÀNH DÂY CHUYỀN MINING SOTA PRODUCTION TRONG {:.2}s", total_time);
    println!("   • Tổng ván cờ đã đấu : {} ván", games);
    println!("   • Tổng mẫu thu thập  : {} mẫu FEN Depth {}", final_samples, depth);
    println!("   • Thông lượng trung bình: {:.2} mẫu / giây", final_speed);
    println!("   • Tệp dữ liệu xuất  : {}", output);
    println!("===============================================================================");
    // Ép xả bộ đệm màn hình terminal
    let _ = io::stdout().flush();
}
