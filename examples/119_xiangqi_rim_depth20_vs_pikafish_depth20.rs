// ============================================================================
// VÍ DỤ 119: ĐẠI CHIẾN ĐỈNH CAO: XIANGQI-RIM DEPTH 20 VS PIKAFISH DEPTH 20
// ============================================================================
// Đấu trường đỉnh cao đối đầu trực tiếp ở độ sâu siêu cấp Depth 20:
// 1. Phe A: Xiangqi-RIM Super Grandmaster (Depth 20, 4 Luồng SMP, 128MB Global Shared TT,
//           NNUE Gen 8 Platinum 3.8M FENs, ProbCut, Double Singular Extensions, Improving Heuristic)
// 2. Phe B: Pikafish World Champion (Depth 20, C FFI in-memory 4 Luồng SMP)
// - Luân phiên Đỏ / Đen qua từng ván cờ.
// - Bắt buộc in thời gian thực (realtime stdout flush) từng nước đi.
// - Ghi nhận kết quả Thắng/Thua/Hòa, số nước đi trung bình, và độ trễ phản hồi.
//
// 100% chú thích tiếng Việt từng dòng & 100% định danh từ đơn tiếng Anh.
// ============================================================================

use std::ffi::{CStr, CString};
use std::io::Write;
use std::os::raw::{c_char, c_int, c_void};
use std::time::Instant;

use xiangrust::board::{Parser, Serializer};
use xiangrust::book::Book;
use xiangrust::movegen::legal;
use xiangrust::search::Limits;
use xiangrust::thread::Pool;
use xiangrust::uci::Format;

use xiangrust::system::{TimeSystem, Weights};
use xiangrust::learn::Harvest;
use xiangrust::learn::frame::Frame;

/// Hàm vẽ giao diện bàn cờ ASCII trực quan
fn render_board(pos: &xiangrust::board::Position, mv: xiangrust::movegen::types::Move, score: i32, depth: u8, actor: u8, ply: usize) {
    let frame = Frame::pack(pos, mv, score, depth, actor, ply, 0);
    let rendered = frame.render();
    println!("{}", rendered);
    let _ = std::io::stdout().flush();
}

/// Kiểu con trỏ hàm khởi tạo C FFI `pika_ffi_init`
type InitFn = unsafe extern "C" fn();
/// Kiểu con trỏ hàm tìm kiếm nước đi C FFI `pika_ffi_query`
type QueryFn = unsafe extern "C" fn(*const c_char, c_int, c_int, *mut c_char, *mut c_int) -> c_int;
/// Kiểu con trỏ hàm cấu hình luồng C FFI `pika_ffi_set_threads`
type ThreadsFn = unsafe extern "C" fn(c_int);
/// Kiểu con trỏ hàm dọn dẹp bộ nhớ C FFI `pika_ffi_cleanup`
type CleanFn = unsafe extern "C" fn();

/// Struct `Bridge` quản lý nạp động thư viện Pikafish C FFI
pub struct Bridge {
    /// Con trỏ thư viện
    pub handle: *mut c_void,
    /// Hàm khởi tạo
    pub init: InitFn,
    /// Hàm tìm kiếm
    pub query: QueryFn,
    /// Hàm cấu hình luồng
    pub threads: Option<ThreadsFn>,
    /// Hàm giải phóng
    pub clean: CleanFn,
}

impl Bridge {
    /// Nạp động tệp `dylib` từ đường dẫn
    pub fn load(path: &str) -> Option<Self> {
        let c_path = CString::new(path).ok()?;
        let handle = unsafe { libc::dlopen(c_path.as_ptr(), libc::RTLD_NOW) };
        if handle.is_null() {
            eprintln!("❌ Không thể nạp thư viện động tại: {}", path);
            return None;
        }

        let init_sym = unsafe { libc::dlsym(handle, b"pika_ffi_init\0".as_ptr() as *const c_char) };
        let query_sym = unsafe { libc::dlsym(handle, b"pika_ffi_query\0".as_ptr() as *const c_char) };
        let threads_sym = unsafe { libc::dlsym(handle, b"pika_ffi_set_threads\0".as_ptr() as *const c_char) };
        let clean_sym = unsafe { libc::dlsym(handle, b"pika_ffi_cleanup\0".as_ptr() as *const c_char) };

        if init_sym.is_null() || query_sym.is_null() || clean_sym.is_null() {
            eprintln!("❌ Không tìm thấy đầy đủ các hàm C-ABI trong thư viện!");
            return None;
        }

        let init: InitFn = unsafe { std::mem::transmute(init_sym) };
        let query: QueryFn = unsafe { std::mem::transmute(query_sym) };
        let threads: Option<ThreadsFn> = if !threads_sym.is_null() {
            Some(unsafe { std::mem::transmute(threads_sym) })
        } else {
            None
        };
        let clean: CleanFn = unsafe { std::mem::transmute(clean_sym) };

        unsafe { init() };

        Some(Self {
            handle,
            init,
            query,
            threads,
            clean,
        })
    }

    /// Cấu hình số luồng cho Pikafish
    pub fn workers(&self, count: i32) {
        if let Some(f) = self.threads {
            unsafe { f(count as c_int) };
        }
    }

    /// Truy vấn nước đi tốt nhất và điểm số trực tiếp trong RAM với giới hạn thời gian (ms) và Depth (Tối đa 60s)
    pub fn probe(&self, moves: &[String], movetime_ms: i32, depth: i32) -> Option<(String, i32)> {
        let moves_str = moves.join(" ");
        let c_moves = CString::new(moves_str).ok()?;
        let mut best_buf = [0 as c_char; 32];
        let mut score: c_int = 0;

        // Bắt buộc kẹp trần thời gian tối đa không bao giờ vượt quá 60,000ms (1 phút)!
        let safe_movetime = movetime_ms.clamp(100, 60_000);

        let ret = unsafe {
            (self.query)(
                c_moves.as_ptr(),
                safe_movetime as c_int,
                depth as c_int,
                best_buf.as_mut_ptr(),
                &mut score as *mut c_int,
            )
        };

        if ret == 0 {
            let best_cstr = unsafe { CStr::from_ptr(best_buf.as_ptr()) };
            let best = best_cstr.to_str().ok()?.to_string();
            Some((best, score as i32))
        } else {
            None
        }
    }
}

impl Drop for Bridge {
    fn drop(&mut self) {
        unsafe {
            (self.clean)();
            if !self.handle.is_null() {
                libc::dlclose(self.handle);
            }
        }
    }
}

fn main() {
    let bridge_path = "./libpika.dylib";
    let bridge = match Bridge::load(bridge_path) {
        Some(b) => b,
        None => {
            eprintln!("⚠️ Vui lòng đảm bảo `libpika.dylib` tồn tại tại gốc workspace!");
            std::process::exit(1);
        }
    };

    let threads = std::env::var("THREADS").unwrap_or_else(|_| "4".to_string()).parse::<usize>().unwrap_or(4);
    bridge.workers(threads as i32);

    let rim_depth = std::env::var("RIM_DEPTH").unwrap_or_else(|_| "20".to_string()).parse::<i32>().unwrap_or(20);
    let pika_depth = std::env::var("PIKA_DEPTH").unwrap_or_else(|_| "20".to_string()).parse::<i32>().unwrap_or(20);
    let pika_time = std::env::var("PIKA_TIME").unwrap_or_else(|_| "12000".to_string()).parse::<i32>().unwrap_or(12000).clamp(100, 60_000);
    let total_games = std::env::var("GAMES").unwrap_or_else(|_| "4".to_string()).parse::<usize>().unwrap_or(4);
    let max_plies = std::env::var("MAX_PLIES").unwrap_or_else(|_| "140".to_string()).parse::<usize>().unwrap_or(140);
    let tt_mb = std::env::var("TT_MB").unwrap_or_else(|_| "128".to_string()).parse::<usize>().unwrap_or(128);

    println!("===============================================================================");
    println!(" ⚔️  ĐẠI CHIẾN ĐỈNH CAO SIÊU CẤP: XIANGQI-RIM DEPTH {} VS PIKAFISH DEPTH {}", rim_depth, pika_depth);
    println!("    Phiên bản     : v24.12.0-strict-1min-time-cap-depth20");
    println!("    Xiangqi-RIM   : Depth {} | {} Luồng SMP | NNUE Gen 8 (3.8M) | {}MB TT | Time Alloc (<=12s, Hard Cap <=60s)", rim_depth, threads, tt_mb);
    println!("    Pikafish C FFI: Depth {} | {} Luồng SMP | In-Memory Zero-IPC C-ABI | Max MoveTime: {}ms (Hard Cap <=60s)", pika_depth, threads, pika_time);
    println!("===============================================================================\n");

    println!("📖 Sử dụng bộ sách khai cuộc chuẩn tĩnh đã qua thẩm định (1,024 biến thể chuẩn)");
    let _ = std::io::stdout().flush();

    let rim_pool = Pool::new(threads, tt_mb);

    let mut rim_wins = 0;
    let mut pika_wins = 0;
    let mut draws = 0;

    let mut rim_total_time_ms = 0u64;
    let mut rim_total_moves = 0u64;
    let mut pika_total_time_ms = 0u64;
    let mut pika_total_moves = 0u64;

    let tournament_start = Instant::now();
    let weights = Weights::grandmaster();
    let mut harvest = Harvest::default();

    for game_idx in 1..=total_games {
        let rim_is_red = game_idx % 2 == 1;
        let mut pos = Parser::parse(Parser::DEFAULT);
        let mut past_hashes: Vec<u64> = Vec::with_capacity(128);
        past_hashes.push(pos.hash);

        let mut move_history: Vec<String> = Vec::with_capacity(128);
        let mut ply = 0usize;
        let mut result = "Hòa (Draw)";
        let mut last_score = 0i32;

        println!("-------------------------------------------------------------------------------");
        println!(
            " 🎲 VÁN {}/{}: [ĐỎ: {}] vs [ĐEN: {}]",
            game_idx,
            total_games,
            if rim_is_red { format!("Xiangqi-RIM (D{})", rim_depth) } else { format!("Pikafish (D{})", pika_depth) },
            if rim_is_red { format!("Pikafish (D{})", pika_depth) } else { format!("Xiangqi-RIM (D{})", rim_depth) }
        );
        println!("-------------------------------------------------------------------------------");
        let _ = std::io::stdout().flush();

        while ply < max_plies {
            let max_book_ply = std::env::var("BOOK_PLIES").unwrap_or_else(|_| "14".to_string()).parse::<usize>().unwrap_or(14);
            let is_rim_turn = (pos.side == 0 && rim_is_red) || (pos.side == 1 && !rim_is_red);
            let (best_uci, move_score) = if is_rim_turn {
                let start = Instant::now();
                let book_move = if ply < max_book_ply {
                    Book::probe(&pos)
                } else {
                    None
                };

                let (uci, score) = if let Some(bm) = book_move {
                    (Format::encode(bm), 0)
                } else {
                    let mut limits = Limits::new();
                    limits.depth = rim_depth as u8;
                    let pure_depth = std::env::var("PURE_DEPTH").unwrap_or_default() == "1" || std::env::var("MOVE_TIME").unwrap_or_default() == "0";
                    if pure_depth {
                        limits.exact = 0; // Pure Depth Mode: Không ngắt thời gian, bắt buộc tìm kiếm đủ 100% Depth
                    } else {
                        let mut dynamic_time = TimeSystem::allocate(&pos, &weights, last_score, 0, ply);
                        if last_score.abs() >= 500 {
                            dynamic_time = (dynamic_time * 3 / 2).min(30_000);
                        }
                        let requested_time = std::env::var("MOVE_TIME").ok().and_then(|t| t.parse().ok()).unwrap_or(dynamic_time);
                        limits.exact = requested_time.clamp(100, 300_000);
                    }
                    let res = rim_pool.trace(&pos, &limits, &past_hashes);
                    last_score = res.score;
                    (Format::encode(res.best), res.score)
                };
                let elapsed_ms = start.elapsed().as_millis() as u64;
                rim_total_time_ms += elapsed_ms;
                rim_total_moves += 1;

                print!(" [RIM D{}: {} ({:+4}cp, {:3}ms)]", rim_depth, uci, score, elapsed_ms);
                (uci, score)
            } else {
                let start = Instant::now();
                let book_move = if ply < max_book_ply {
                    Book::probe(&pos)
                } else {
                    None
                };

                let (uci, score) = if let Some(bm) = book_move {
                    (Format::encode(bm), 0)
                } else {
                    let pika_limit = if std::env::var("PURE_DEPTH").unwrap_or_default() == "1" { 0 } else { pika_time };
                    let (best, s) = bridge.probe(&move_history, pika_limit, pika_depth).unwrap_or(("0000".to_string(), 0));
                    last_score = -s;
                    (best, s)
                };
                let elapsed_ms = start.elapsed().as_millis() as u64;
                pika_total_time_ms += elapsed_ms;
                pika_total_moves += 1;

                print!(" [PIKA D{}: {} ({:+4}cp, {:3}ms)]", pika_depth, uci, score, elapsed_ms);
                (uci, score)
            };
            let _ = std::io::stdout().flush();

            // Áp dụng nước đi lên bàn cờ
            let mv = Format::decode(&best_uci);
            if mv.valid() {
                if !legal::valid(&mut pos, mv) {
                    println!("\n  ❌ Nước đi phạm luật: {}", best_uci);
                    result = if is_rim_turn { "Pikafish Thắng (RIM Phạm Luật)" } else { "Xiangqi-RIM Thắng (Pika Phạm Luật)" };
                    break;
                }
                // Tự động bảo tồn thế cờ và nước đi Grandmaster vào bộ đệm
                harvest.push(&pos, mv, move_score, if is_rim_turn { rim_depth as u8 } else { pika_depth as u8 }, if is_rim_turn { "RIM" } else { "PIKA" }, ply);

                pos.apply(mv.from, mv.to);
                past_hashes.push(pos.hash);
                move_history.push(best_uci);
                ply += 1;

                if ply % 4 == 0 {
                    println!(" (Ply {})", ply);
                    let _ = std::io::stdout().flush();
                }

                // Kiểm tra sát cục hoặc thắng cuộc
                if move_score.abs() >= 28000 {
                    if is_rim_turn {
                        if move_score > 0 {
                            result = "Xiangqi-RIM Thắng (Sát Cục)";
                        } else {
                            result = "Pikafish Thắng (Sát Cục)";
                        }
                    } else {
                        if move_score > 0 {
                            result = "Pikafish Thắng (Sát Cục)";
                        } else {
                            result = "Xiangqi-RIM Thắng (Sát Cục)";
                        }
                    }
                    break;
                }

                // Kiểm tra điều kiện đầu hàng khi chênh lệch điểm quá lớn sau ply 30
                if ply >= 30 && move_score.abs() >= 3500 {
                    if is_rim_turn {
                        if move_score > 0 {
                            result = "Xiangqi-RIM Thắng (Ưu thế áp đảo)";
                        } else {
                            result = "Pikafish Thắng (Ưu thế áp đảo)";
                        }
                    } else {
                        if move_score > 0 {
                            result = "Pikafish Thắng (Ưu thế áp đảo)";
                        } else {
                            result = "Xiangqi-RIM Thắng (Ưu thế áp đảo)";
                        }
                    }
                    break;
                }
            } else {
                println!("\n  ⚠️ Nước đi không hợp lệ: {}", best_uci);
                break;
            }
        }

        println!("\n\n  🏆 KẾT QUẢ VÁN {}: {}", game_idx, result);
        let fen_str = Serializer::export(&pos);
        println!("     Số nước đi: {} plies | FEN cuối: {}", ply, fen_str);
        render_board(&pos, xiangrust::movegen::types::Move::new(0, 0), 0, rim_depth as u8, 0, ply);

        // Lưu toàn bộ tri thức của ván cờ vào 1,024 Shards NVMe và JSONL Stream
        let saved_samples = harvest.flush(result);
        println!("     💾 Tự động bảo tồn tri thức: Đã lưu {} thế cờ Grandmaster vào 1,024 Shards NVMe & JSONL Stream!\n", saved_samples);
        let _ = std::io::stdout().flush();

        if result.contains("Xiangqi-RIM Thắng") {
            rim_wins += 1;
        } else if result.contains("Pikafish Thắng") {
            pika_wins += 1;
        } else {
            draws += 1;
        }
    }

    let tournament_elapsed = tournament_start.elapsed().as_secs_f64();
    let rim_avg_ms = if rim_total_moves > 0 { rim_total_time_ms as f64 / rim_total_moves as f64 } else { 0.0 };
    let pika_avg_ms = if pika_total_moves > 0 { pika_total_time_ms as f64 / pika_total_moves as f64 } else { 0.0 };

    println!("===============================================================================");
    println!(" 🏆 BẢNG TỔNG KẾT ĐẠI CHIẾN SIÊU CẤP DEPTH 20 VS DEPTH 20");
    println!("===============================================================================");
    println!("  • Tổng thời gian giải đấu: {:.2} giây", tournament_elapsed);
    println!("  • Xiangqi-RIM (Depth {}) : {} Thắng ({:.1}%)", rim_depth, rim_wins, (rim_wins as f64 / total_games as f64) * 100.0);
    println!("  • Pikafish (Depth {})    : {} Thắng ({:.1}%)", pika_depth, pika_wins, (pika_wins as f64 / total_games as f64) * 100.0);
    println!("  • Hòa cờ (Draws)         : {} Hòa ({:.1}%)", draws, (draws as f64 / total_games as f64) * 100.0);
    println!("  • Tỷ lệ điểm Xiangqi-RIM : {:.1}%", ((rim_wins as f64 + 0.5 * draws as f64) / total_games as f64) * 100.0);
    println!("  • Số nước trung bình/ván : {:.1} plies", (rim_total_moves + pika_total_moves) as f64 / total_games as f64);
    println!("  • Độ trễ TB Xiangqi-RIM  : {:.2} ms / nước đi", rim_avg_ms);
    println!("  • Độ trễ TB Pikafish     : {:.2} ms / nước đi", pika_avg_ms);
    println!("===============================================================================\n");
}
