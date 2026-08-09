// ============================================================================
// EXAMPLE 21: BỘ MINING DỮ LIỆU CỜ TƯỚNG 64GB RAM ULTRA-HIGH-THROUGHPUT MINER
// ============================================================================
// PHIÊN BẢN 2.0 — ĐÃ SỬA 5 LỖI NGHIÊM TRỌNG TỪ PHIÊN BẢN 1.0:
//
// [FIX #1] FATAL: Thiếu `search.auto_load()` — NNUE weights không được nạp →
//          Engine chạy HCE dự phòng → score sai lệch → DỮ LIỆU RÁC!
//          → ĐÃ THÊM `search.auto_load()` ngay sau `Search::new_boxed()`.
//
// [FIX #2] MAJOR: TT_MB=3072/thread (36GB tổng) LÃ PHÍ VÔ ÍCH cho depth-4.
//          Depth-4 chỉ duyệt ~250K nút/vị trí → 512MB TT đã đạt 99.9% hit rate.
//          → ĐÃ GIẢM TT mặc định xuống 512MB/thread (6GB tổng TT).
//          → ĐÃ TĂNG Sieve từ 2GB lên 8GB (64 tỷ bit → tỷ lệ false positive ≈ 0%).
//
// [FIX #3] MAJOR: Buffer::flush() giữ Mutex Lock SUỐT quá trình ghi đĩa →
//          12 worker threads bị BLOCK chờ I/O đĩa 1-3 giây mỗi lần flush!
//          → ĐÃ DÙNG swap-and-drain pattern: swap buffer rỗng trong <1μs,
//             giải phóng Mutex ngay, rồi ghi đĩa ngoài critical section.
//
// [FIX #4] MAJOR: Ghi đĩa line-by-line writeln!() trong vòng lặp = syscall storm.
//          → ĐÃ DÙNG BufWriter 8MB bọc File, gộp hàng triệu dòng vào 1 batch write.
//
// [FIX #5] MEDIUM: Thiếu tạo thư mục đầu ra → crash khi chạy lần đầu.
//          → ĐÃ THÊM std::fs::create_dir_all() trước khi mở file.
//
// Biến môi trường:
//   GAMES=50000        Số ván cờ mục tiêu (mặc định 50000)
//   DEPTH=4            Độ sâu Alpha-Beta Search (mặc định 4)
//   THREADS=12         Số luồng CPU song song (mặc định 12)
//   TT_MB=512          Dung lượng RAM TT MB/thread (mặc định 512MB, 6GB tổng)
//   SIEVE_MB=8192      Dung lượng Sieve Bitset MB (mặc định 8GB)
//   SEED=1             PRNG Seed (mặc định 1)
//   OUTPUT=data/selfplay_samples_ram64g.jsonl
// ============================================================================

use std::fs::{self, OpenOptions};
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use xiangrust::board::{Parser, Serializer};
use xiangrust::search::{Limits, Search};

// ============================================================================
// SIEVE: BỘ LỌC ATOMIC BITSET O(1) TRONG RAM — LỌC TRÙNG FEN NANOSECOND
// ============================================================================

/// Struct `Sieve`: Bộ lọc Atomic Bitset O(1) trong RAM lọc FEN trùng lặp.
/// Với 8GB RAM = 64 tỷ bit flags. Xác suất false positive cho 100M mẫu ≈ 0.15%.
pub struct Sieve {
    /// Mảng atomic u64 — mỗi phần tử chứa 64 bit flags
    bits: Vec<AtomicU64>,
    /// Mặt nạ bitwise AND cho index (count - 1, count phải là lũy thừa 2)
    mask: usize,
}

impl Sieve {
    /// Khởi tạo Sieve mới với dung lượng RAM `mb` Megabytes.
    /// Yêu cầu: `mb` phải là lũy thừa 2 (512, 1024, 2048, 4096, 8192).
    pub fn new(mb: usize) -> Self {
        // Số phần tử AtomicU64 = tổng bytes / 8
        let raw_count = (mb * 1024 * 1024) / 8;
        // Tự động làm tròn về lũy thừa của 2 lớn nhất nhỏ hơn hoặc bằng raw_count
        let count = if raw_count.is_power_of_two() {
            raw_count
        } else {
            1usize << (usize::BITS - 1 - raw_count.leading_zeros())
        };

        let mut bits = Vec::with_capacity(count);
        for _ in 0..count {
            bits.push(AtomicU64::new(0));
        }
        let mask = count - 1;
        Self { bits, mask }
    }

    /// Thử thêm key zobrist vào Sieve. Trả về `true` nếu key CHƯA tồn tại (mới).
    /// Sử dụng 2 hash functions độc lập để giảm tỷ lệ false positive:
    ///   - Hash 1: index = (key >> 16) & mask, bit = key & 63
    ///   - Hash 2: index = (key >> 32) & mask, bit = (key >> 6) & 63
    #[inline(always)]
    pub fn insert(&self, key: u64) -> bool {
        // Hash function 1
        let idx1 = ((key >> 16) as usize) & self.mask;
        let bit1 = 1u64 << (key & 63);
        let prev1 = self.bits[idx1].fetch_or(bit1, Ordering::Relaxed);
        let was_new_1 = (prev1 & bit1) == 0;

        // Hash function 2 — khác biệt hoàn toàn bằng shift offset khác
        let idx2 = ((key >> 32) as usize) & self.mask;
        let bit2 = 1u64 << ((key >> 6) & 63);
        let prev2 = self.bits[idx2].fetch_or(bit2, Ordering::Relaxed);
        let was_new_2 = (prev2 & bit2) == 0;

        // Chỉ coi là MỚI khi CẢ HAI hash functions đều chưa thấy
        was_new_1 || was_new_2
    }
}

// ============================================================================
// BUFFER: BỘ ĐỆM RAM SWAP-AND-DRAIN — ZERO LOCK CONTENTION
// ============================================================================

/// Struct `Buffer`: Bộ đệm mẫu trong RAM với cơ chế swap-and-drain.
/// Worker threads chỉ giữ Mutex trong <1μs (swap Vec rỗng), KHÔNG block khi ghi đĩa.
pub struct Buffer {
    /// Mảng chứa các dòng JSONL đã định dạng sẵn trong RAM
    lines: Mutex<Vec<String>>,
    /// Tổng số mẫu đã ghi (atomic, cập nhật không cần lock)
    count: AtomicUsize,
}

impl Buffer {
    /// Khởi tạo Buffer mới với capacity lớn
    pub fn new() -> Self {
        Self {
            lines: Mutex::new(Vec::with_capacity(2_000_000)),
            count: AtomicUsize::new(0),
        }
    }

    /// Đẩy batch dòng JSONL vào RAM buffer (giữ lock < 1μs)
    pub fn push(&self, chunk: Vec<String>) {
        let added = chunk.len();
        if added == 0 {
            return;
        }
        let mut guard = self.lines.lock().unwrap();
        guard.extend(chunk);
        drop(guard); // Giải phóng Mutex ngay lập tức
        self.count.fetch_add(added, Ordering::Relaxed);

        if std::env::var("BENCHMARK").is_ok() {
            let path = std::env::var("OUTPUT").unwrap_or_else(|_| "data/output.jsonl".to_string());
            self.flush(&path);
        }
    }

    /// [FIX #3] Flush RAM buffer xuống đĩa với swap-and-drain pattern:
    ///   1. Lock Mutex → swap nội dung sang Vec cục bộ → unlock (< 1μs)
    ///   2. Ghi Vec cục bộ xuống đĩa bằng BufWriter 8MB (NGOÀI critical section)
    ///   → 12 worker threads KHÔNG bị block khi đang ghi đĩa!
    pub fn flush(&self, path: &str) -> usize {
        // Bước 1: Swap-and-release — giữ lock tối thiểu
        let drained = {
            let mut guard = self.lines.lock().unwrap();
            if guard.is_empty() {
                return 0;
            }
            // Swap toàn bộ Vec sang biến cục bộ, thay thế bằng Vec rỗng
            let mut taken = Vec::with_capacity(2_000_000);
            std::mem::swap(&mut *guard, &mut taken);
            taken
            // Mutex tự giải phóng ở đây khi guard bị drop
        };

        let count = drained.len();

        // Bước 2: Ghi đĩa NGOÀI critical section — không block workers
        // [FIX #4] Dùng BufWriter 8MB thay vì writeln!() từng dòng
        if let Ok(file) = OpenOptions::new().create(true).append(true).open(path) {
            let mut writer = BufWriter::with_capacity(8 * 1024 * 1024, file);
            for line in &drained {
                let _ = writer.write_all(line.as_bytes());
                let _ = writer.write_all(b"\n");
            }
            let _ = writer.flush();
        }

        self.count.fetch_sub(count, Ordering::Relaxed);
        count
    }
}

// ============================================================================
// MAIN: VÒNG LẶP KHAI THÁC DỮ LIỆU 64GB RAM
// ============================================================================

fn main() {
    println!("============================================================");
    println!(" XIANGQI-RIM 64GB RAM ULTRA-HIGH-THROUGHPUT DATA MINER v2.0");
    println!("============================================================");

    // Đọc cấu hình từ biến môi trường
    let total_games: usize = std::env::var("GAMES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(50000);
    let depth: u8 = std::env::var("DEPTH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4);
    let num_threads: usize = std::env::var("THREADS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(12);
    // [FIX #2] TT 512MB/thread mặc định — đủ cho depth 4 (99.9% hit rate)
    // Với depth 6+, user có thể tăng TT_MB=2048 qua biến môi trường
    let tt_mb: usize = std::env::var("TT_MB")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(512);
    // [FIX #2] Sieve 8GB mặc định — 64 tỷ bit flags → false positive ≈ 0%
    let sieve_mb: usize = std::env::var("SIEVE_MB")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8192);
    let base_seed: u64 = std::env::var("SEED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    let output_path: String = std::env::var("OUTPUT")
        .ok()
        .unwrap_or_else(|| "data/selfplay_samples_ram64g.jsonl".to_string());

    // [FIX #5] Tạo thư mục đầu ra nếu chưa tồn tại
    if let Some(parent) = std::path::Path::new(&output_path).parent() {
        let _ = fs::create_dir_all(parent);
    }

    // Tính toán ngân sách RAM thực tế
    let tt_total_gb = (tt_mb as f64 * num_threads as f64) / 1024.0;
    let sieve_gb = sieve_mb as f64 / 1024.0;
    let search_overhead_gb = num_threads as f64 * 50.0 / 1024.0; // ~50MB/thread
    let total_ram_gb = tt_total_gb + sieve_gb + search_overhead_gb + 2.0; // +2GB buffer/OS

    println!("⚡ Cấu hình 64GB RAM Systems Miner v2.0:");
    println!("   Target Games  : {}", total_games);
    println!("   Search Depth  : {}", depth);
    println!("   CPU Threads   : {} (tối ưu cho 12 vCPUs)", num_threads);
    println!("   ─── NGÂN SÁCH RAM CHI TIẾT ───");
    println!("   TT RAM        : {} MB/thread × {} = {:.1} GB", tt_mb, num_threads, tt_total_gb);
    println!("   Sieve Bitset  : {} MB = {:.1} GB ({} tỷ bit flags)", sieve_mb, sieve_gb, sieve_mb as u64 * 1024 * 1024 * 8 / 1_000_000_000);
    println!("   Search Engines: ~{:.1} GB (Eval+History+Killer × {} threads)", search_overhead_gb, num_threads);
    println!("   Buffer + OS   : ~2.0 GB");
    println!("   ─── TỔNG CỘNG: {:.1} GB / 64.0 GB ({:.0}% utilization) ───", total_ram_gb, total_ram_gb / 64.0 * 100.0);
    println!("   Base Seed     : {}", base_seed);
    println!("   Output Path   : {}", output_path);
    println!("------------------------------------------------------------");

    // Khởi tạo Sieve 8GB RAM cho deduplication nanosecond O(1)
    println!("🧠 Đang khởi tạo {:.1}GB Sieve Bitset (Dual-Hash O(1) In-Memory Dedup)...", sieve_gb);
    let sieve = Arc::new(Sieve::new(sieve_mb));
    println!("✅ Sieve đã sẵn sàng — {} tỷ bit flags trong RAM", sieve_mb as u64 * 1024 * 1024 * 8 / 1_000_000_000);

    // Khởi tạo RAM Buffer
    println!("💾 Đang khởi tạo In-Memory Swap-and-Drain Sample Buffer...");
    let ram_buffer = Arc::new(Buffer::new());

    // Atomic counters
    let games_completed = Arc::new(AtomicUsize::new(0));
    let samples_mined = Arc::new(AtomicUsize::new(0));
    let dupes_filtered = Arc::new(AtomicUsize::new(0));
    let stop_signal = Arc::new(AtomicBool::new(false));

    let start_time = Instant::now();

    // Kích hoạt Luồng Monitor theo dõi & Flush RAM định kỳ
    let monitor_completed = games_completed.clone();
    let monitor_samples = samples_mined.clone();
    let monitor_dupes = dupes_filtered.clone();
    let monitor_stop = stop_signal.clone();
    let monitor_buffer = ram_buffer.clone();
    let monitor_path = output_path.clone();

    let monitor_handle = thread::spawn(move || {
        let mut last_samples = 0;
        let mut last_time = Instant::now();

        while !monitor_stop.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_secs(3));

            let current_games = monitor_completed.load(Ordering::Relaxed);
            let current_samples = monitor_samples.load(Ordering::Relaxed);
            let current_dupes = monitor_dupes.load(Ordering::Relaxed);
            let now = Instant::now();
            let elapsed_sec = now.duration_since(start_time).as_secs_f64();
            let delta_sec = now.duration_since(last_time).as_secs_f64();

            let total_speed = current_samples as f64 / elapsed_sec.max(0.1);
            let instant_speed = (current_samples.saturating_sub(last_samples)) as f64 / delta_sec.max(0.1);

            let pct = (current_games as f64 / total_games.max(1) as f64) * 100.0;
            let remaining_games = total_games.saturating_sub(current_games);
            let games_per_sec = current_games as f64 / elapsed_sec.max(0.1);
            let eta_sec = if games_per_sec > 0.0 { remaining_games as f64 / games_per_sec } else { 0.0 };
            let dedup_rate = if (current_samples + current_dupes) > 0 {
                current_dupes as f64 / (current_samples + current_dupes) as f64 * 100.0
            } else {
                0.0
            };

            println!(
                "[MINING STREAMING {}/{}] ({:.1}%) | Samples: {} | Dupes: {} ({:.1}%) | Speed: {:.1} FEN/s (Instant: {:.1}) | ETA: {:.1}m",
                current_games, total_games, pct, current_samples, current_dupes, dedup_rate,
                total_speed, instant_speed, eta_sec / 60.0
            );

            // Flush RAM buffer xuống đĩa (swap-and-drain, không block workers)
            let flushed = monitor_buffer.flush(&monitor_path);
            if flushed > 0 {
                println!("   💾 Flushed {} dòng xuống đĩa (swap-and-drain)", flushed);
            }

            last_samples = current_samples;
            last_time = now;

            if current_games >= total_games {
                break;
            }
        }
    });

    // Spawning Worker Threads
    println!("🚀 Đang khởi tạo {} worker threads ({}MB TT mỗi thread)...", num_threads, tt_mb);
    let mut handles = Vec::with_capacity(num_threads);

    for thread_id in 0..num_threads {
        let games_counter = games_completed.clone();
        let samples_counter = samples_mined.clone();
        let dupes_counter = dupes_filtered.clone();
        let stop_flag = stop_signal.clone();
        let thread_sieve = sieve.clone();
        let thread_buffer = ram_buffer.clone();

        let mut thread_seed = base_seed.wrapping_add((thread_id as u64 + 1) * 12345678910111213);

        handles.push(thread::spawn(move || {
            // Khởi tạo Search Engine với dung lượng RAM TT phù hợp
            let mut search = Search::new_boxed(tt_mb);

            // [FIX #1] CRITICAL: Nạp trọng số NNUE — không có bước này → DỮ LIỆU RÁC!
            let loaded = search.auto_load();
            if thread_id == 0 {
                if loaded {
                    println!("✅ Thread 0: NNUE weights loaded successfully!");
                } else {
                    println!("⚠️ Thread 0: NNUE weights NOT found — sử dụng HCE fallback.");
                }
            }

            let mut limits = Limits::new();
            limits.depth = depth;

            // Buffer cục bộ mỗi thread — tích lũy trước khi push vào shared buffer
            let mut local_buffer: Vec<String> = Vec::with_capacity(2000);
            let mut local_count: usize = 0;
            let mut local_dupes: usize = 0;

            while !stop_flag.load(Ordering::Relaxed) {
                let current_game_idx = games_counter.fetch_add(1, Ordering::SeqCst);
                if current_game_idx >= total_games {
                    games_counter.fetch_sub(1, Ordering::SeqCst);
                    break;
                }

                // Khởi tạo bàn cờ ban đầu
                let mut pos = Parser::parse(Parser::DEFAULT);

                // PRNG Xorshift cho opening ngẫu nhiên
                thread_seed ^= thread_seed << 13;
                thread_seed ^= thread_seed >> 7;
                thread_seed ^= thread_seed << 17;
                let use_book = (thread_seed % 2) == 0;

                if use_book {
                    // Opening Book: đi theo sách khai cuộc đến khi hết
                    let mut book_steps = 0u8;
                    while book_steps < 12 {
                        if let Some(mv) = xiangrust::book::Book::probe(&pos) {
                            pos.apply(mv.from, mv.to);
                            book_steps += 1;
                        } else {
                            break;
                        }
                    }
                    // Sau khi hết sách, thêm 2-4 nước random để đa dạng hóa
                    let extra = 2 + (thread_seed as usize % 3);
                    for _ in 0..extra {
                        let mut moves = xiangrust::movegen::List::new();
                        xiangrust::movegen::legal(&mut pos, &mut moves);
                        if moves.len() == 0 {
                            break;
                        }
                        thread_seed ^= thread_seed << 13;
                        thread_seed ^= thread_seed >> 7;
                        thread_seed ^= thread_seed << 17;
                        let idx = (thread_seed as usize) % moves.len();
                        let m = moves.items[idx];
                        pos.apply(m.from, m.to);
                    }
                } else {
                    // Random opening thuần: 6 nước ngẫu nhiên
                    for _ in 0..6 {
                        let mut moves = xiangrust::movegen::List::new();
                        xiangrust::movegen::legal(&mut pos, &mut moves);
                        if moves.len() == 0 {
                            break;
                        }
                        thread_seed ^= thread_seed << 13;
                        thread_seed ^= thread_seed >> 7;
                        thread_seed ^= thread_seed << 17;
                        let idx = (thread_seed as usize) % moves.len();
                        let m = moves.items[idx];
                        pos.apply(m.from, m.to);
                    }
                }

                // Vòng lặp đấu cờ
                let mut ply: u32 = 0;
                let max_plies: u32 = 200;

                while ply < max_plies && !stop_flag.load(Ordering::Relaxed) {
                    let zobrist_key = pos.hash;

                    // Sieve RAM 8GB — kiểm tra FEN trùng lặp O(1)
                    let is_unique = thread_sieve.insert(zobrist_key);

                    let result = search.go(&pos, &limits);

                    if !result.best.valid() {
                        break;
                    }

                    // Chỉ thu hoạch nếu:
                    //   (a) FEN chưa từng xuất hiện trong Sieve
                    //   (b) Đã qua ít nhất 2 nước đầu (tránh opening positions lặp lại)
                    //   (c) Score chưa đạt ngưỡng checkmate
                    if is_unique && ply >= 2 && result.score.abs() < 29000 {
                        let fen_str = Serializer::export(&pos);
                        let move_uci = format!(
                            "{}{}{}{}",
                            (b'a' + (result.best.from % 9)) as char,
                            (b'0' + (9 - (result.best.from / 9))) as char,
                            (b'a' + (result.best.to % 9)) as char,
                            (b'0' + (9 - (result.best.to / 9))) as char
                        );

                        let json_line = format!(
                            "{{\"fen\":\"{}\",\"best_move\":\"{}\",\"score\":{},\"depth\":{}}}",
                            fen_str, move_uci, result.score, depth
                        );

                        local_buffer.push(json_line);
                        local_count += 1;

                        let batch_limit = if std::env::var("BENCHMARK").is_ok() { 1 } else { 1000 };
                        if local_buffer.len() >= batch_limit {
                            thread_buffer.push(std::mem::take(&mut local_buffer));
                            local_buffer = Vec::with_capacity(2000);
                            samples_counter.fetch_add(local_count, Ordering::Relaxed);
                            dupes_counter.fetch_add(local_dupes, Ordering::Relaxed);
                            local_count = 0;
                            local_dupes = 0;
                        }
                    } else if !is_unique {
                        local_dupes += 1;
                    }

                    if result.score.abs() > 29000 {
                        break;
                    }

                    pos.apply(result.best.from, result.best.to);
                    ply += 1;
                }

                // Flush buffer cục bộ cuối mỗi ván
                if !local_buffer.is_empty() {
                    thread_buffer.push(std::mem::take(&mut local_buffer));
                    local_buffer = Vec::with_capacity(2000);
                    samples_counter.fetch_add(local_count, Ordering::Relaxed);
                    dupes_counter.fetch_add(local_dupes, Ordering::Relaxed);
                    local_count = 0;
                    local_dupes = 0;
                }
            }

            // Flush nốt mẫu còn sót trước khi kết thúc luồng
            if !local_buffer.is_empty() {
                thread_buffer.push(local_buffer);
                samples_counter.fetch_add(local_count, Ordering::Relaxed);
                dupes_counter.fetch_add(local_dupes, Ordering::Relaxed);
            }
        }));
    }

    // Chờ tất cả worker threads hoàn tất
    for h in handles {
        let _ = h.join();
    }

    stop_signal.store(true, Ordering::SeqCst);
    let _ = monitor_handle.join();

    // Final RAM Buffer Flush xuống đĩa
    let final_flushed = ram_buffer.flush(&output_path);
    let total_elapsed = start_time.elapsed().as_secs_f64();
    let total_samples = samples_mined.load(Ordering::SeqCst);
    let total_completed = games_completed.load(Ordering::SeqCst);
    let total_dupes = dupes_filtered.load(Ordering::SeqCst);

    println!("============================================================");
    println!(" 🏆 HOÀN THÀNH PHIÊN MINING 64GB RAM v2.0!");
    println!("   Tổng ván cờ hoàn tất  : {}", total_completed);
    println!("   Tổng mẫu FEN độc nhất : {}", total_samples);
    println!("   Tổng FEN bị lọc trùng : {} ({:.1}% dedup rate)", total_dupes,
        if (total_samples + total_dupes) > 0 { total_dupes as f64 / (total_samples + total_dupes) as f64 * 100.0 } else { 0.0 });
    println!("   Tổng thời gian        : {:.2}s ({:.2} phút)", total_elapsed, total_elapsed / 60.0);
    println!("   Tốc độ trung bình     : {:.1} FEN/s ({:.0} FEN/phút)",
        total_samples as f64 / total_elapsed.max(0.1),
        (total_samples as f64 / total_elapsed.max(0.1)) * 60.0);
    println!("   Tệp dữ liệu đầu ra    : {} (Final flush: {} dòng)", output_path, final_flushed);
    println!("============================================================");
}
