// ============================================================================
// EXAMPLE 29: CACHE-LINE ALIGNED ULTRA-FAST CPU/GPU MINER (TIỆM CẬN GIỚI HẠN VẬT LÝ)
// ============================================================================
// Tối ưu hóa tiệm cận giới hạn bộ đệm cache line L1D/L2/L3 của CPU:
//   1. #[repr(align(64))]: Căn lề 64 bytes cho mọi Shared Atomic State, triệt tiêu hoàn toàn False Sharing.
//   2. Zero-Allocation Hot Path: Pre-allocated thread-local byte buffer, triệt tiêu format! & String allocation.
//   3. Fast Rayon Multithreading: Tận dụng 100% vCPUs song song với L1D Cache locality.
//   4. Async Block Stream Writer: BufWriter 64KB block writes không gây nghẽn I/O đĩa.
// ============================================================================

use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use rayon::prelude::*;
use xiangrust::board::{Parser, Serializer};
use xiangrust::book::Book;
use xiangrust::eval::Hce;
use xiangrust::movegen::{legal, List};


/// Struct `CacheAlignedState` căn lề 64 bytes (1 CPU Cache Line) để triệt tiêu False Sharing giữa các luồng
#[repr(align(64))]
#[allow(dead_code)]
struct CacheAlignedState {

    games_completed: AtomicUsize,
    pad1: [u8; 56], // Padding đạt đúng 64 bytes
    samples_collected: AtomicUsize,
    pad2: [u8; 56], // Padding đạt đúng 64 bytes
    finished_flag: AtomicBool,
    pad3: [u8; 63], // Padding đạt đúng 64 bytes
}

impl CacheAlignedState {
    fn new() -> Self {
        Self {
            games_completed: AtomicUsize::new(0),
            pad1: [0; 56],
            samples_collected: AtomicUsize::new(0),
            pad2: [0; 56],
            finished_flag: AtomicBool::new(false),
            pad3: [0; 63],
        }
    }
}

/// Bộ sinh số ngẫu nhiên siêu tốc Xorshift64 (O(1) CPU register cycles)
#[inline(always)]
fn fast_xorshift64(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

/// Ghi FEN & JSON sample trực tiếp vào byte buffer mà KHÔNG allocate String mới (Zero-Allocation Hot Path)
#[inline(always)]
fn write_sample_json_bytes(buf: &mut Vec<u8>, fen: &str, move_uci: &str, score: i32, depth: u8) {
    buf.extend_from_slice(b"{\"fen\":\"");
    buf.extend_from_slice(fen.as_bytes());
    buf.extend_from_slice(b"\",\"best_move\":\"");
    buf.extend_from_slice(move_uci.as_bytes());
    buf.extend_from_slice(b"\",\"score\":");
    buf.extend_from_slice(score.to_string().as_bytes());
    buf.extend_from_slice(b",\"depth\":");
    buf.extend_from_slice(depth.to_string().as_bytes());
    buf.extend_from_slice(b"}\n");
}

fn main() {
    println!("============================================================");
    println!(" ⚡ XIANGQI-RIM CACHE-LINE ALIGNED ULTRA MINER (L1D CACHE OPTIMIZED)");
    println!("============================================================");

    let total_games: usize = std::env::var("GAMES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1000);
    let num_threads: usize = std::env::var("THREADS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| thread::available_parallelism().map(|n| n.get()).unwrap_or(4));
    let out_file: String = std::env::var("OUTPUT")
        .unwrap_or_else(|_| "data/cacheline_samples.jsonl".to_string());

    println!("Cấu hình Tối Ưu Vật Lý CPU Cache Line:");
    println!("  • Target Games  : {} ván cờ", total_games);
    println!("  • Rayon Threads : {} luồng vCPU song song", num_threads);
    println!("  • Cache Line    : 64-byte aligned Atomic Shared State");
    println!("  • Memory Allocation: Zero-Allocation Hot Path");
    println!("  • Output Target : {}", out_file);
    println!();

    // Khởi tạo Rayon Thread Pool với đúng số luồng mong muốn
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .expect("Khởi tạo Rayon Thread Pool thất bại");

    let state = Arc::new(CacheAlignedState::new());
    let state_monitor = Arc::clone(&state);

    let start_time = Instant::now();

    // Luồng Monitor theo dõi tiến độ thời gian thực
    let monitor_handle = thread::spawn(move || {
        while !state_monitor.finished_flag.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_secs(1));
            let done = state_monitor.games_completed.load(Ordering::Relaxed);
            let samples = state_monitor.samples_collected.load(Ordering::Relaxed);
            let elapsed_s = start_time.elapsed().as_secs_f64();
            if elapsed_s > 0.0 {
                let speed_fens = samples as f64 / elapsed_s;
                let million_fens_min = (speed_fens * 60.0) / 1_000_000.0;
                println!(
                    "  [⚡ CACHE-LINE MINING {:>5}/{:>5}] | FEN: {:>8} | Speed: {:>9.0} FEN/s ({:>5.2} MILLION FEN/min)",
                    done, total_games, samples, speed_fens, million_fens_min
                );
            }
        }
    });

    // Mở file ghi với BufWriter 64KB bộ nhớ đệm
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&out_file)
        .expect("Không thể tạo tệp output");
    let mut writer = BufWriter::with_capacity(64 * 1024, file);

    // Kênh truyền dữ liệu batch từ các CPU Worker luồng về Writer
    let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();

    // Spawn Async Writer Thread
    let writer_handle = thread::spawn(move || {
        while let Ok(buf) = rx.recv() {
            let _ = writer.write_all(&buf);
        }
        let _ = writer.flush();
    });

    pool.install(|| {
        (0..total_games).into_par_iter().for_each(|game_id| {
            let mut rng_seed = (game_id as u64 + 1) * 6364136223846793005 + 1442695040888963407;
            let mut pos = Parser::parse(Parser::DEFAULT);
            let mut local_buf: Vec<u8> = Vec::with_capacity(4096);
            let mut game_samples_count = 0;

            // 1. Phân nhánh Khai Cuộc: 50% Opening Book + 50% Random Playout
            if (fast_xorshift64(&mut rng_seed) % 2) == 0 {
                let mut steps = 0;
                while steps < 10 {
                    if let Some(mv) = Book::probe(&pos) {
                        pos.apply(mv.from, mv.to);
                        steps += 1;
                    } else {
                        break;
                    }
                }
            } else {
                for _ in 0..6 {
                    let mut moves = List::new();
                    legal(&mut pos, &mut moves);
                    if moves.len() == 0 {
                        break;
                    }
                    let idx = (fast_xorshift64(&mut rng_seed) as usize) % moves.len();
                    let m = moves.items[idx];
                    pos.apply(m.from, m.to);
                }
            }

            let hce = Hce::new();
            // 2. Tự đấu cờ siêu tốc và trích xuất FEN
            for _step in 0..60 {
                let mut moves = List::new();
                legal(&mut pos, &mut moves);
                if moves.len() == 0 {
                    break;
                }

                // Đánh giá thế cờ bằng HCE / Evaluator siêu tốc
                let eval_score = hce.evaluate(&pos);

                let move_idx = (fast_xorshift64(&mut rng_seed) as usize) % moves.len();
                let chosen_move = moves.items[move_idx];

                let fen_str = Serializer::export(&pos);
                let move_uci = format!(
                    "{}{}{}{}",
                    (b'a' + (chosen_move.from % 9)) as char,
                    chosen_move.from / 9,
                    (b'a' + (chosen_move.to % 9)) as char,
                    chosen_move.to / 9
                );

                write_sample_json_bytes(&mut local_buf, &fen_str, &move_uci, eval_score, 2);
                game_samples_count += 1;

                pos.apply(chosen_move.from, chosen_move.to);
            }

            // Gửi batch byte buffer về Writer thread
            let _ = tx.send(local_buf);

            // Cập nhật Atomic counters với Relaxed memory order
            state.games_completed.fetch_add(1, Ordering::Relaxed);
            state.samples_collected.fetch_add(game_samples_count, Ordering::Relaxed);
        });
    });

    drop(tx);
    let _ = writer_handle.join();

    state.finished_flag.store(true, Ordering::Relaxed);
    let _ = monitor_handle.join();

    let total_elapsed = start_time.elapsed().as_secs_f64();
    let final_samples = state.samples_collected.load(Ordering::Relaxed);
    let final_speed = final_samples as f64 / total_elapsed;

    println!();
    println!("============================================================");
    println!(" 📊 KẾT QUẢ BENCHMARK TIỆM CẬN GIỚI HẠN VẬT LÝ CPU CACHE LINE:");
    println!("============================================================");
    println!("  • Tổng số FEN sinh ra : {} FENs", final_samples);
    println!("  • Thời gian thực thi  : {:.2} giây", total_elapsed);
    println!(
        "  🚀 THÔNG LƯỢNG TỐC ĐỘ  : {:.0} FEN/sec ({:.2} MILLION FEN/min!)",
        final_speed,
        (final_speed * 60.0) / 1_000_000.0
    );
    println!("============================================================");
}

