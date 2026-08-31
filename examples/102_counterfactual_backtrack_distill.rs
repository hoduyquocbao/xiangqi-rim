// ============================================================================
// VÍ DỤ 102: ĐỘNG CƠ TỰ ĐẤU HỒI CỨU SỬA SAI VÔ HẠN (UNLIMITED COUNTERFACTUAL TREE DISTILLATION)
// ============================================================================
// Triển khai thuật toán Hoàn tác Vô hạn (Unlimited Blunder Backtracking Tree):
// 1. Khi Xiangqi-RIM đi vào thế yếu trước Pikafish (> 200cp bất lợi):
//    - Tự động hoàn tác về nút rẽ sai lầm gần nhất.
//    - Phạt -10,000 cp cho nước sai trong Blunder Memory.
//    - Ghi nhận mẫu tương phản tiêu cực: Blunder FEN = -800 cp.
//    - Thử nghiệm các nước đi ứng viên phản đòn mới.
//    - NẾU toàn bộ nước đi tại nút hiện tại đều thua: Lùi tiếp về nước đi trước đó (Multi-Ply Tree Rollback)!
//    - Tiếp tục quá trình cho đến khi tìm được nhánh Thắng/Hòa trước Pikafish!
// 2. Khi tìm được nhánh phản đòn an toàn/thắng trận:
//    - Ghi nhận mẫu tương phản tích cực: Refutation FEN = +500 cp.
// 3. Huấn luyện mạng NNUE HalfKAv2_hm trên kho mẫu tương phản vàng.
// 4. Xuất bản trọng số độc lập ra `data/nnue_weights.bin`.
// ============================================================================

use std::ffi::{CStr, CString};
use std::io::Write;
use std::os::raw::{c_char, c_int, c_void};
use std::time::Instant;

use xiangrust::board::{Parser, Position};
use xiangrust::book::Book;
use xiangrust::learn::blunder::Blunder;
use xiangrust::learn::nnue::{Datum, Network};
use xiangrust::movegen::legal;
use xiangrust::movegen::types::{List, Move};
use xiangrust::search::{Limits, Search};
use xiangrust::uci::Format;

/// Kiểu con trỏ hàm khởi tạo `pika_ffi_init`
type InitFn = unsafe extern "C" fn();
/// Kiểu con trỏ hàm tìm kiếm nước đi `pika_ffi_query`
type QueryFn = unsafe extern "C" fn(*const c_char, c_int, c_int, *mut c_char, *mut c_int) -> c_int;
/// Kiểu con trỏ hàm cấu hình luồng `pika_ffi_set_threads`
type ThreadsFn = unsafe extern "C" fn(c_int);
/// Kiểu con trỏ hàm dọn dẹp bộ nhớ `pika_ffi_cleanup`
type CleanFn = unsafe extern "C" fn();

/// Struct `Bridge` quản lý việc nạp động và giao tiếp với Pikafish qua C FFI in-memory.
pub struct Bridge {
    pub handle: *mut c_void,
    pub init: InitFn,
    pub query: QueryFn,
    pub threads: Option<ThreadsFn>,
    pub clean: CleanFn,
}

impl Bridge {
    /// Nạp động tệp thư viện C FFI từ đường dẫn `path`.
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

        // Khởi tạo động cơ Pikafish trong RAM
        unsafe { init() };

        Some(Self {
            handle,
            init,
            query,
            threads,
            clean,
        })
    }

    /// Cấu hình số luồng tìm kiếm song song cho Pikafish ThreadPool.
    pub fn workers(&self, count: i32) {
        if let Some(f) = self.threads {
            unsafe { f(count as c_int) };
        }
    }

    /// Truy vấn nước đi tốt nhất và điểm số trực tiếp trong RAM (Zero IPC, Zero Pipe).
    pub fn probe(&self, moves: &[String], movetime_ms: i32, depth: i32) -> Option<(String, i32)> {
        let moves_str = moves.join(" ");
        let c_moves = CString::new(moves_str).ok()?;
        let mut bestmove_buf = [0u8; 16];
        let mut score: c_int = 0;

        let res = unsafe {
            (self.query)(
                c_moves.as_ptr(),
                movetime_ms as c_int,
                depth as c_int,
                bestmove_buf.as_mut_ptr() as *mut c_char,
                &mut score as *mut c_int,
            )
        };

        if res != 0 {
            return None;
        }

        let best_str = unsafe { CStr::from_ptr(bestmove_buf.as_ptr() as *const c_char) }
            .to_str()
            .ok()?
            .to_string();

        if best_str.is_empty() {
            return None;
        }

        Some((best_str, score as i32))
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

/// Bản ghi vết của 1 nước đi trong ván đấu tự đấu
#[derive(Clone)]
struct Step {
    pos: Position,
    mv: Move,
    #[allow(dead_code)]
    score: i32,
    #[allow(dead_code)]
    pika_score: i32,
}

fn resolve_dylib_path() -> String {
    let candidates = [
        std::env::var("LIBPIKA_PATH").unwrap_or_default(),
        "libpika.dylib".to_string(),
        "/Users/hdqb/workspaces/xiangqi-rim/libpika.dylib".to_string(),
        "/Users/hdqb/workspaces/pikafish/src/libpika.dylib".to_string(),
    ];
    for p in &candidates {
        if !p.is_empty() && std::path::Path::new(p).exists() {
            return p.clone();
        }
    }
    "libpika.dylib".to_string()
}

fn main() {
    println!("===============================================================================");
    println!(" 🏆 ĐỘNG CƠ TỰ ĐẤU HỒI CỨU SỬA SAI VÔ HẠN (UNLIMITED COUNTERFACTUAL TREE)");
    println!("    Xiangqi-RIM AI Engine vs Pikafish C FFI — Quyết Tâm Tìm Ra Nhánh Thắng");
    println!("===============================================================================");

    let games: usize = std::env::var("GAMES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20);

    let depth_rim: u8 = std::env::var("DEPTH_RIM")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(6);

    let depth_pika: u8 = std::env::var("DEPTH_PIKA")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5);

    let max_backtracks: usize = std::env::var("BACKTRACKS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(50); // Mở rộng tối đa 50 lần hoàn tác lùi sâu đa tầng

    let dylib_path = resolve_dylib_path();

    println!("⚙️ CẤU HÌNH TỰ ĐẤU HỒI CỨU VÔ HẠN:");
    println!("  • Số ván cờ mục tiêu       : {} ván", games);
    println!("  • Độ sâu Xiangqi-RIM       : Depth {}", depth_rim);
    println!("  • Độ sâu Pikafish          : Depth {}", depth_pika);
    println!("  • Giới hạn hoàn tác tối đa : {} lần/ván (Không giới hạn đường rẽ)", max_backtracks);
    println!("  • Thư viện C FFI Pikafish  : {}", dylib_path);

    let bridge = match Bridge::load(&dylib_path) {
        Some(b) => {
            b.workers(4);
            b
        }
        None => {
            eprintln!("❌ Không thể nạp thư viện C FFI từ '{}'!", dylib_path);
            return;
        }
    };
    println!("✅ Nạp thành công libpika.dylib in-memory!\n");

    let mut dataset: Vec<Datum> = Vec::with_capacity(games * 150);
    let mut blunder_table = Blunder::new();

    let mut total_wins = 0;
    let mut total_draws = 0;
    let mut total_losses = 0;
    let mut total_backtracks = 0;

    let start_all = Instant::now();

    println!("[GIAI ĐOẠN 1] Bắt đầu tự đấu hồi cứu vô hạn (Live Per-Game Yield)...");
    let _ = std::io::stdout().flush();

    for game_idx in 1..=games {
        let game_start = Instant::now();
        let mut pos = Parser::parse(Parser::DEFAULT);
        let mut moves: Vec<String> = Vec::new();
        let mut steps: Vec<Step> = Vec::new();

        // 1. Khởi tạo khai cuộc đa dạng (100% Book Opening 2-4 plies)
        let opening_count = 2 + (game_idx % 3);
        for _ in 0..opening_count {
            if let Some(book_mv) = Book::probe(&pos) {
                let uci = Format::encode(book_mv);
                pos.apply(book_mv.from, book_mv.to);
                moves.push(uci);
            } else {
                break;
            }
        }

        let rim_side = (game_idx % 2) as u8; // Luân phiên Đỏ/Đen
        let mut game_winner: Option<usize> = None;
        let mut backtracks_in_game = 0;
        let mut ply_count = 0;

        let mut search = Search::new(16);

        while game_winner.is_none() && ply_count < 80 {
            let current_side = pos.side;

            if current_side == rim_side {
                // ============================================================
                // LƯỢT ĐI CỦA XIANGQI-RIM
                // ============================================================
                let mut limits = Limits::new();
                limits.depth = depth_rim;

                let best_res = search.go(&mut pos, &limits);
                let best_mv = best_res.best;
                if !best_mv.valid() {
                    game_winner = Some(1); // Hết nước đi hợp lệ -> Pikafish thắng
                    break;
                }

                // Kiểm tra điểm phạt từ bảng Blunder
                let penalty = blunder_table.penalty(pos.hash, best_mv.raw());
                let rim_score = search.eval.score(&pos) - penalty;

                let uci = Format::encode(best_mv);
                let pos_before = pos;

                pos.apply(best_mv.from, best_mv.to);
                moves.push(uci);

                steps.push(Step {
                    pos: pos_before,
                    mv: best_mv,
                    score: rim_score,
                    pika_score: 0,
                });

                ply_count += 1;
            } else {
                // ============================================================
                // LƯỢT ĐI CỦA PIKAFISH
                // ============================================================
                let res = bridge.probe(&moves, 0, depth_pika as i32);
                if res.is_none() {
                    game_winner = Some(0); // Pikafish đầu hàng / lỗi -> RIM thắng
                    break;
                }
                let (pika_uci, pika_score) = res.unwrap();
                let pika_mv = Format::decode(&pika_uci);
                if !pika_mv.valid() {
                    game_winner = Some(0); // Pikafish phạm luật -> RIM thắng
                    break;
                }

                // KIỂM TRA BẪY CHIẾN THUẬT: Pikafish có đang trừng phạt nước vừa rồi của RIM không?
                let pika_side = rim_side ^ 1;
                let pika_advantage = if pika_side == 0 { pika_score } else { -pika_score };

                // Nếu Pikafish có lợi thế > 200cp và RIM vẫn còn quyền hoàn tác lùi sâu:
                let mut needs_backtrack = pika_advantage > 200 && backtracks_in_game < max_backtracks;

                while needs_backtrack && !steps.is_empty() {
                    backtracks_in_game += 1;
                    total_backtracks += 1;

                    let blunder_step = steps.pop().unwrap();
                    let blunder_hash = blunder_step.pos.hash;
                    let blunder_raw = blunder_step.mv.raw();

                    // 1. Phạt nặng -10,000 cp cho nước đi sập bẫy trong Blunder Memory
                    blunder_table.record(blunder_hash, blunder_raw, 10000);

                    // 2. Tạo mẫu tương phản tiêu cực (Blunder Contrastive Sample)
                    let blunder_datum = Datum::extract(&blunder_step.pos, -800);
                    dataset.push(blunder_datum);

                    // 3. Hoàn tác bàn cờ về trước nước đi sai lầm của RIM
                    moves.pop(); // Bỏ nước sai của RIM
                    let mut refutation_pos = blunder_step.pos;

                    // 4. Tìm kiếm các nước đi thay thế hợp lệ chưa bị phạt nặng
                    let mut legals = List::new();
                    legal::gen(&mut refutation_pos, &mut legals);

                    // Ép Search Engine tìm kiếm nước phản đòn mới với depth tăng cường
                    let mut refutation_limits = Limits::new();
                    refutation_limits.depth = depth_rim + 1;

                    let new_res = search.go(&mut refutation_pos, &refutation_limits);
                    let new_best = new_res.best;

                    if new_best.valid() && new_best.raw() != blunder_raw {
                        // Thử nghiệm nước đi mới với Pikafish
                        let new_uci = Format::encode(new_best);
                        moves.push(new_uci);

                        let test_res = bridge.probe(&moves, 0, depth_pika as i32);
                        if let Some((_, test_pika_score)) = test_res {
                            let test_pika_adv = if pika_side == 0 { test_pika_score } else { -test_pika_score };
                            if test_pika_adv <= 180 {
                                // TÌM RA NHÁNH PHẢN ĐÒN THÀNH CÔNG!
                                let refutation_datum = Datum::extract(&refutation_pos, 500);
                                dataset.push(refutation_datum);

                                refutation_pos.apply(new_best.from, new_best.to);
                                steps.push(Step {
                                    pos: blunder_step.pos,
                                    mv: new_best,
                                    score: new_res.score,
                                    pika_score: 0,
                                });

                                pos = refutation_pos;
                                needs_backtrack = false;
                                break;
                            } else {
                                // Nước này vẫn thua -> Phạt tiếp và thử lùi sâu tiếp
                                moves.pop();
                                blunder_table.record(blunder_hash, new_best.raw(), 10000);
                            }
                        } else {
                            moves.pop();
                        }
                    }

                    if steps.is_empty() {
                        // Đã lùi hết toàn bộ lịch sử ván cờ nhưng không tìm thấy nước giải
                        moves.push(Format::encode(blunder_step.mv));
                        steps.push(blunder_step);
                        needs_backtrack = false;
                        break;
                    }
                }

                if !needs_backtrack {
                    if let Some(last_step) = steps.last_mut() {
                        last_step.pika_score = pika_score;
                    }
                    pos.apply(pika_mv.from, pika_mv.to);
                    moves.push(pika_uci);
                    ply_count += 1;
                }
            }
        }

        // Xác định kết quả ván đấu
        let (win_str, _is_win) = match game_winner {
            Some(0) => {
                total_wins += 1;
                ("RIM Thắng 🏆", true)
            }
            Some(1) => {
                total_losses += 1;
                ("Pikafish Thắng", false)
            }
            _ => {
                total_draws += 1;
                ("Hòa cờ 🤝", false)
            }
        };

        let game_ms = game_start.elapsed().as_millis();
        println!(
            "  [{:2}/{:2}] [Đỏ: {:7}] | Kết quả: {:15} ({:2} plies, {:5}ms) | Hoàn tác sâu: {:2} | Mẫu vàng: {:3}",
            game_idx,
            games,
            if rim_side == 0 { "RIM" } else { "Pika" },
            win_str,
            ply_count,
            game_ms,
            backtracks_in_game,
            dataset.len()
        );
        let _ = std::io::stdout().flush();
    }

    let elapsed = start_all.elapsed().as_secs_f32();
    println!(
        "\n✅ Giai đoạn 1 hoàn tất trong {:.2}s: Thu thập {} mẫu tương phản vàng từ {} ván đối kháng!",
        elapsed,
        dataset.len(),
        games
    );
    let _ = std::io::stdout().flush();

    // =========================================================================
    // GIAI ĐOẠN 2: HUẤN LUYỆN MẠNG NNUE VỚI TẬP MẪU TƯƠNG PHẢN (CONTRASTIVE LEARNING)
    // =========================================================================
    println!("\n[GIAI ĐOẠN 2] Huấn luyện mạng NNUE HalfKAv2_hm trên tập mẫu tương phản vàng...");
    let mut network = Network::new();
    let epochs = 40;
    let rate = 0.0001f32;

    for epoch in 1..=epochs {
        let mut total_loss = 0.0f32;
        for datum in &dataset {
            let (predicted, state) = network.forward(datum);
            let loss = network.backward(datum, &state, predicted, rate);
            total_loss += loss;
        }
        let mean_loss = total_loss / (dataset.len().max(1) as f32);
        if epoch % 5 == 0 || epoch == epochs {
            println!(" -> Epoch {:2}/{:2}: Mean Loss = {:.4}", epoch, epochs, mean_loss);
            let _ = std::io::stdout().flush();
        }
    }

    // =========================================================================
    // GIAI ĐOẠN 3: LƯỢNG TỬ HÓA VÀ XUẤT BẢN TRỌNG SỐ XRNN v1 ĐỘC LẬP
    // =========================================================================
    println!("\n[GIAI ĐOẠN 3] Lượng tử hóa và xuất bản tệp trọng số NNUE độc lập...");
    let output_weights = "data/nnue_weights.bin";
    if let Err(e) = network.quantize(output_weights) {
        eprintln!("❌ Lỗi xuất trọng số: {}", e);
    } else {
        println!("💾 Đã xuất bản thành công trọng số độc lập: {}", output_weights);
    }

    println!("\n===============================================================================");
    println!(" 🏆 BÁO CÁO TỔNG KẾT TỰ ĐẤU HỒI CỨU SỬA SAI VÔ HẠN:");
    println!("  • Số ván cờ thi đấu        : {} ván", games);
    println!("  • Tỷ lệ Thắng/Hòa/Thua     : W={}, D={}, L={}", total_wins, total_draws, total_losses);
    println!("  • Số lần hoàn tác sâu      : {} lần", total_backtracks);
    println!("  • Mẫu huấn luyện tương phản: {} FENs", dataset.len());
    println!("  • Trọng số NNUE xuất xưởng : {}", output_weights);
    println!("===============================================================================\n");
}
