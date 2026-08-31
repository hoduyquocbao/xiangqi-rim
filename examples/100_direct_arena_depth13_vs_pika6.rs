// ============================================================================
// VÍ DỤ 100: ĐẤU TRƯỜNG THỰC CHIẾN 10 VÁN: XIANGQI-RIM DEPTH 13 VS PIKAFISH DEPTH 6
// ============================================================================
// Đấu trường đỉnh cao 10 ván cờ đối đầu trực tiếp giữa:
// 1. Phe A: Xiangqi-RIM Rust SOTA (Depth 13, 4 Threads SMP, 64MB Global Shared TT,
//           TT Hardware Prefetch, Continuation History, Clamped Multi-Cut Pruning)
// 2. Phe B: Pikafish World Champion (Depth 6, C FFI in-memory 4 Threads SMP)
// - Luân phiên Đỏ / Đen: 5 ván Xiangqi-RIM cầm Đỏ, 5 ván Pikafish cầm Đỏ.
// - Bắt buộc in thời gian thực (realtime stdout flush) từng nước đi.
// - Ghi nhận kết quả Thắng/Thua/Hòa, số nước đi trung bình, và độ trễ phản hồi.
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

    /// Truy vấn nước đi tốt nhất và điểm số trực tiếp trong RAM
    pub fn probe(&self, moves: &[String], depth: i32) -> Option<(String, i32)> {
        let moves_str = moves.join(" ");
        let c_moves = CString::new(moves_str).ok()?;
        let mut best_buf = [0 as c_char; 32];
        let mut score: c_int = 0;

        let ret = unsafe {
            (self.query)(
                c_moves.as_ptr(),
                0, // depth-based search (không giới hạn movetime)
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
    println!("===============================================================================");
    println!(" ⚔️  ĐẤU TRƯỜNG THỰC CHIẾN 10 VÁN: XIANGQI-RIM DEPTH 13 VS PIKAFISH DEPTH 6");
    println!("    Phiên bản     : v11.5.0-sota-conthist-multicut-46x-speedup");
    println!("    Xiangqi-RIM   : Depth 13 | 4 Luồng SMP | 64MB Global Shared TT | SOTA Pack");
    println!("    Pikafish C FFI: Depth 6  | 4 Luồng SMP | In-Memory Zero-IPC C-ABI");
    println!("===============================================================================\n");
    let _ = std::io::stdout().flush();

    // 0. Nạp Sách Khai Cuộc Động Grandmaster XRBK v1
    if let Ok(count) = xiangrust::book::Loader::import("data/master_book.xrbk") {
        println!("📖 Đã nạp thành công {} biến thể khai cuộc từ data/master_book.xrbk", count);
    }

    // 1. Nạp động thư viện C FFI Pikafish
    let bridge = match Bridge::load("./libpika.dylib") {
        Some(b) => b,
        None => {
            eprintln!("❌ Không thể nạp libpika.dylib. Hãy kiểm tra tệp tại gốc workspace!");
            return;
        }
    };
    bridge.workers(4);

    // 2. Khởi tạo ThreadPool cho Xiangqi-RIM (4 luồng SMP, 64MB TT)
    let rim_pool = Pool::new(4, 64);

    let total_games = std::env::var("GAMES")
        .ok()
        .and_then(|g| g.parse().ok())
        .unwrap_or(10usize);
    let max_plies = 120usize;

    let mut rim_wins = 0usize;
    let mut pika_wins = 0usize;
    let mut draws = 0usize;
    let mut total_plies = 0usize;

    let mut rim_total_time_ms = 0u64;
    let mut rim_total_moves = 0usize;
    let mut pika_total_time_ms = 0u64;
    let mut pika_total_moves = 0usize;

    let match_start = Instant::now();

    for game_idx in 1..=total_games {
        let rim_is_red = (game_idx % 2) != 0;
        let mut pos = Parser::parse(Parser::DEFAULT);
        let mut move_history: Vec<String> = Vec::with_capacity(128);
        let mut past_hashes: Vec<u64> = Vec::with_capacity(128);
        past_hashes.push(pos.hash);
        let mut ply = 0usize;
        let mut result = "Draw (Timeout)";

        println!("-------------------------------------------------------------------------------");
        println!(
            " 🎲 VÁN {}/{}: [ĐỎ: {}] vs [ĐEN: {}]",
            game_idx,
            total_games,
            if rim_is_red { "Xiangqi-RIM (D13)" } else { "Pikafish (D6)" },
            if rim_is_red { "Pikafish (D6)" } else { "Xiangqi-RIM (D13)" }
        );
        println!("-------------------------------------------------------------------------------");
        let _ = std::io::stdout().flush();

        while ply < max_plies {
            let is_rim_turn = (pos.side == 0 && rim_is_red) || (pos.side == 1 && !rim_is_red);
            let (best_uci, move_score) = if is_rim_turn {
                let start = Instant::now();
                let book_move = if ply < 10 {
                    Book::probe(&pos)
                } else {
                    None
                };

                let (uci, score) = if let Some(bm) = book_move {
                    (Format::encode(bm), 0)
                } else {
                    let mut limits = Limits::new();
                    limits.depth = 13;
                    limits.exact = 1500; // 1.5s time budget
                    let res = rim_pool.trace(&pos, &limits, &past_hashes);
                    (Format::encode(res.best), res.score)
                };
                let elapsed_ms = start.elapsed().as_millis() as u64;
                rim_total_time_ms += elapsed_ms;
                rim_total_moves += 1;

                print!(" [RIM D13: {} ({:+4}cp, {:3}ms)]", uci, score, elapsed_ms);
                (uci, score)
            } else {
                let start = Instant::now();
                let book_move = if ply < 10 {
                    Book::probe(&pos)
                } else {
                    None
                };

                let (uci, score) = if let Some(bm) = book_move {
                    (Format::encode(bm), 0)
                } else if let Some((best, score)) = bridge.probe(&move_history, 6) {
                    (best, score)
                } else {
                    println!("\n  ⚠️ Pikafish không trả về nước đi hợp lệ!");
                    break;
                };
                let elapsed_ms = start.elapsed().as_millis() as u64;
                pika_total_time_ms += elapsed_ms;
                pika_total_moves += 1;

                print!(" [PIKA D6: {} ({:+4}cp, {:3}ms)]", uci, score, elapsed_ms);
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
                pos.apply(mv.from, mv.to);
                past_hashes.push(pos.hash);
                move_history.push(best_uci);
                ply += 1;

                if ply % 4 == 0 {
                    println!(" (Ply {})", ply);
                    let _ = std::io::stdout().flush();
                }

                // Kiểm tra điều kiện sát cục hoặc chênh lệch điểm số lớn
                if move_score.abs() >= 25000 || move_score.abs() >= 2000 {
                    if is_rim_turn {
                        if move_score >= 2000 {
                            result = "Xiangqi-RIM Thắng (Ưu thế áp đảo)";
                        } else {
                            result = "Pikafish Thắng (Ưu thế áp đảo)";
                        }
                    } else {
                        if move_score >= 2000 {
                            result = "Pikafish Thắng (Ưu thế áp đảo)";
                        } else {
                            result = "Xiangqi-RIM Thắng (Ưu thế áp đảo)";
                        }
                    }
                    if move_score.abs() >= 25000 {
                        result = if (move_score >= 25000 && is_rim_turn) || (move_score <= -25000 && !is_rim_turn) {
                            "Xiangqi-RIM Thắng (Sát Cục)"
                        } else {
                            "Pikafish Thắng (Sát Cục)"
                        };
                        break;
                    }
                }
            } else {
                println!("\n  ❌ Không thể giải mã nước đi UCI: {}", best_uci);
                break;
            }
        }

        total_plies += ply;
        if result.contains("Xiangqi-RIM Thắng") {
            rim_wins += 1;
        } else if result.contains("Pikafish Thắng") {
            pika_wins += 1;
        } else {
            draws += 1;
        }

        println!("\n  🏆 KẾT QUẢ VÁN {}: {}", game_idx, result);
        println!("     Số nước đi: {} plies | FEN cuối: {}\n", ply, Serializer::export(&pos));
        let _ = std::io::stdout().flush();
    }

    let match_elapsed = match_start.elapsed();

    println!("\n===============================================================================");
    println!(" 🏆 BẢNG TỔNG KẾT ĐẤU TRƯỜNG 10 VÁN THỰC CHIẾN");
    println!("===============================================================================");
    println!("  • Tổng thời gian giải đấu: {:.2} giây", match_elapsed.as_secs_f64());
    println!("  • Xiangqi-RIM (Depth 13) : {} Thắng", rim_wins);
    println!("  • Pikafish (Depth 6)     : {} Thắng", pika_wins);
    println!("  • Hòa cờ (Draws)         : {} Hòa", draws);
    println!(
        "  • Tỷ lệ điểm Xiangqi-RIM : {:.1}%",
        ((rim_wins as f64 + draws as f64 * 0.5) / total_games as f64) * 100.0
    );
    println!("  • Số nước trung bình/ván : {:.1} plies", total_plies as f64 / total_games as f64);
    println!(
        "  • Độ trễ TB Xiangqi-RIM  : {:.2} ms / nước đi",
        if rim_total_moves > 0 { rim_total_time_ms as f64 / rim_total_moves as f64 } else { 0.0 }
    );
    println!(
        "  • Độ trễ TB Pikafish     : {:.2} ms / nước đi",
        if pika_total_moves > 0 { pika_total_time_ms as f64 / pika_total_moves as f64 } else { 0.0 }
    );
    println!("===============================================================================\n");
}
