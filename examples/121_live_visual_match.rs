// ============================================================================
// VÍ DỤ 121: ĐẤU TRƯỜNG TRỰC QUAN HÓA THỜI GIAN THỰC (LIVE VISUAL MATCH ARENA)
// ============================================================================
// Chương trình chạy trận đấu trực tiếp giữa Xiangqi-RIM và đối thủ (Pikafish / Tự đấu)
// và TRỰC QUAN HÓA BÀN CỜ ASCII GRID CHUẨN ĐỒ HỌA TOÁN HỌC SAU MỖI NƯỚC ĐI!
//
// 100% chú thích tiếng Việt từng dòng & 100% định danh từ đơn tiếng Anh.
// ============================================================================

use std::ffi::{CStr, CString};
use std::io::Write;
use std::os::raw::{c_char, c_int, c_void};
use std::time::Instant;

use xiangrust::board::{Parser, Position};
use xiangrust::book::Book;
use xiangrust::movegen::legal;
use xiangrust::movegen::types::Move;
use xiangrust::search::Limits;
use xiangrust::system::{TimeSystem, Weights};
use xiangrust::thread::Pool;
use xiangrust::uci::Format;
use xiangrust::learn::frame::Frame;

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
    pub handle: *mut c_void,
    pub init: InitFn,
    pub query: QueryFn,
    pub threads: Option<ThreadsFn>,
    pub clean: CleanFn,
}

impl Bridge {
    /// Nạp động thư viện Pikafish nếu có sẵn
    pub fn load(path: &str) -> Option<Self> {
        let c_path = CString::new(path).ok()?;
        let handle = unsafe { libc::dlopen(c_path.as_ptr(), libc::RTLD_NOW) };
        if handle.is_null() {
            return None;
        }

        let init_sym = unsafe { libc::dlsym(handle, b"pika_ffi_init\0".as_ptr() as *const c_char) };
        let query_sym = unsafe { libc::dlsym(handle, b"pika_ffi_query\0".as_ptr() as *const c_char) };
        let threads_sym = unsafe { libc::dlsym(handle, b"pika_ffi_set_threads\0".as_ptr() as *const c_char) };
        let clean_sym = unsafe { libc::dlsym(handle, b"pika_ffi_cleanup\0".as_ptr() as *const c_char) };

        if init_sym.is_null() || query_sym.is_null() || clean_sym.is_null() {
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

    /// Truy vấn nước đi tốt nhất từ Pikafish
    pub fn probe(&self, moves: &[String], movetime_ms: i32, depth: i32) -> Option<(String, i32)> {
        let moves_str = moves.join(" ");
        let c_moves = CString::new(moves_str).ok()?;
        let mut out_move = vec![0 as c_char; 32];
        let mut out_score: c_int = 0;

        let safe_movetime = movetime_ms.clamp(100, 60_000);
        let ret = unsafe {
            (self.query)(
                c_moves.as_ptr(),
                depth as c_int,
                safe_movetime as c_int,
                out_move.as_mut_ptr(),
                &mut out_score,
            )
        };

        if ret != 0 {
            return None;
        }

        let move_str = unsafe { CStr::from_ptr(out_move.as_ptr()).to_str().ok()? }.to_string();
        Some((move_str, out_score as i32))
    }
}

impl Drop for Bridge {
    fn drop(&mut self) {
        unsafe {
            (self.clean)();
            libc::dlclose(self.handle);
        }
    }
}

/// Hàm vẽ giao diện bàn cờ ASCII trực quan kèm thông số nước đi
fn render_board(pos: &Position, mv: Move, score: i32, depth: u8, actor: u8, ply: usize) {
    let frame = Frame::pack(pos, mv, score, depth, actor, ply, 0);
    let rendered = frame.render();
    println!("{}", rendered);
    let _ = std::io::stdout().flush();
}

fn main() {
    println!("===============================================================================");
    println!(" ⚔️  ĐẤU TRƯỜNG TRỰC QUAN HÓA THỜI GIAN THỰC (LIVE VISUAL MATCH ARENA) ⚔️");
    println!("===============================================================================");

    let depth = std::env::var("DEPTH").ok().and_then(|d| d.parse().ok()).unwrap_or(8);
    let max_plies = std::env::var("MAX_PLIES").ok().and_then(|p| p.parse().ok()).unwrap_or(30);

    // Khởi tạo Xiangqi-RIM Engine Pool
    let pool = Pool::new(4, 32);

    // Thử nạp Pikafish Bridge nếu có
    let pika_path = "pikafish/src/libpikafish_arm64.dylib";
    let bridge = Bridge::load(pika_path);
    let has_pika = bridge.is_some();

    println!(" • Xiangqi-RIM Engine : 4 Cores, 32MB TT, Depth: {}", depth);
    if has_pika {
        println!(" • Đối thủ            : Pikafish World Champion (Depth: {})", depth);
    } else {
        println!(" • Đối thủ            : Xiangqi-RIM Self-Play Clone (Depth: {})", depth);
    }
    println!(" • Giới hạn nước đi   : Tối đa {} plies", max_plies);
    println!("-------------------------------------------------------------------------------");

    let mut pos = Parser::parse(Parser::DEFAULT);
    let mut past_hashes = Vec::with_capacity(128);
    let mut move_history = Vec::with_capacity(128);
    let weights = Weights::grandmaster();
    let mut prev_score = 0i32;
    past_hashes.push(pos.hash);

    println!("\n📋 THẾ CỜ KHỞI ĐẦU:");
    render_board(&pos, Move::new(0, 0), 0, 0, 0, 0);

    let mut ply = 0;
    while ply < max_plies {
        let is_red = pos.side == 0;
        let actor_code = if is_red { 0u8 } else { if has_pika { 1u8 } else { 0u8 } };
        let actor = if is_red { "Xiangqi-RIM (ĐỎ)" } else { if has_pika { "Pikafish (ĐEN)" } else { "Xiangqi-RIM (ĐEN)" } };

        println!("-------------------------------------------------------------------------------");
        println!(" ♟️ LƯỢT {}: {} suy nghĩ...", ply + 1, actor);
        let _ = std::io::stdout().flush();

        let start = Instant::now();
        let (best_uci, score) = if is_red || !has_pika {
            // Xiangqi-RIM tìm kiếm với Dynamic Critical Time Allocator
            let book_move = if ply < 6 { Book::probe(&pos) } else { None };
            if let Some(bm) = book_move {
                (Format::encode(bm), 0)
            } else {
                let dynamic_ms = TimeSystem::allocate(&pos, &weights, prev_score, prev_score, ply);
                let mut limits = Limits::new();
                limits.depth = depth as u8;
                limits.exact = dynamic_ms;
                let res = pool.trace(&pos, &limits, &past_hashes);
                (Format::encode(res.best), res.score)
            }
        } else {
            // Pikafish tìm kiếm
            let book_move = if ply < 6 { Book::probe(&pos) } else { None };
            if let Some(bm) = book_move {
                (Format::encode(bm), 0)
            } else if let Some(ref b) = bridge {
                b.probe(&move_history, 5000, depth).unwrap_or_else(|| ("a0a1".to_string(), 0))
            } else {
                ("a0a1".to_string(), 0)
            }
        };

        prev_score = score;

        let elapsed = start.elapsed().as_millis();
        let mv = Format::decode(&best_uci);

        if !mv.valid() || !legal::valid(&mut pos, mv) {
            println!(" ❌ Nước đi không hợp lệ: {}", best_uci);
            break;
        }

        println!(" 🎯 NƯỚC ĐI: {} | Đánh giá: {:+4} cp | Thời gian: {}ms", best_uci, score, elapsed);

        // Áp dụng nước đi
        pos.apply(mv.from, mv.to);
        past_hashes.push(pos.hash);
        move_history.push(best_uci);
        ply += 1;

        // Trực quan hóa bàn cờ sau nước đi
        render_board(&pos, mv, score, depth as u8, actor_code, ply);

        if score.abs() >= 28000 {
            println!("\n 🏆 TRẬN ĐẤU KẾT THÚC BẰNG SÁT CỤC!");
            break;
        }
    }

    println!("===============================================================================");
    println!(" ✅ HOÀN TẤT TRẬN ĐẤU THỬ NGHIỆM TRỰC QUAN HÓA ({} Plies)", ply);
    println!("===============================================================================");
}
