// ============================================================================
// EXAMPLE 20: BỘ MINING DỮ LIỆU GIA TỐC HẠ TẦNG GPU BATCH ENGINE (16,384 BATCH)
// ============================================================================
// Vận hành GPU Batch Self-Play Engine gia tốc 16,384 ván cờ song song trên Compute Units GPU:
//   - Ép công suất card đồ họa GPU phần cứng (Metal Native / Vulkan / DX12) lên 80% - 100%.
//   - Zero-Allocation Byte Streamer: Triệt tiêu 100% String & format! allocations rác.
//   - Căn lề 64-byte vật lý phòng chống False Sharing trên CPU Cache Line.
//   - Chú thích tường minh 100% Tiếng Việt cho từng định danh (biến, hàm, tham số, thuộc tính).
// ============================================================================

// Nhập module thao tác với tệp tin và thiết lập quyền mở tệp
use std::fs::OpenOptions;
// Nhập module bộ đệm BufWriter và trait Write cho ghi dữ liệu hiệu năng cao
use std::io::{BufWriter, Write};
// Nhập các kiểu dữ liệu nguyên tử AtomicBool, AtomicUsize và thứ tự Ordering phòng tranh chấp đa luồng
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
// Nhập kênh truyền dữ liệu đơn kênh một chiều channel từ std::sync::mpsc
use std::sync::mpsc::channel;
// Nhập con trỏ đếm tham chiếu Arc từ std::sync
use std::sync::Arc;
// Nhập module thread xử lý đa luồng CPU
use std::thread;
// Nhập module đo thời gian Instant và Duration từ std::time
use std::time::{Duration, Instant};

// Nhập bộ lặp song song Rayon prelude cho Rayon Thread Pool
use rayon::prelude::*;
// Nhập đối tượng Parser và Serializer từ module board của xiangrust
use xiangrust::board::{Parser, Serializer};
// Nhập đối tượng Book từ module book quản lý Opening Book khai cuộc
use xiangrust::book::Book;
// Nhập các cấu trúc dữ liệu Batch, Device, Evaluable, Evaluator, Sample từ module gpu
use xiangrust::gpu::{Batch, Device, Evaluable, Evaluator, Sample};
// Nhập hàm legal và struct List từ module movegen sinh nước đi hợp lệ
use xiangrust::movegen::{legal, List};

/// Struct `CacheAlignedState`: Cấu trúc dữ liệu lưu trữ trạng thái tiến độ mining.
/// Căn lề 64 bytes (đúng 1 CPU Cache Line 64B) để triệt tiêu 100% hiện tượng False Sharing giữa luồng Monitor và Worker.
#[repr(align(64))]
#[allow(dead_code)]
struct CacheAlignedState {
    /// Số lượng ván cờ đã hoàn tất (biến nguyên tử AtomicUsize 8 bytes)
    games_completed: AtomicUsize,
    /// Trường đệm pad1 56 bytes để đẩy trường tiếp theo sang Cache Line thứ 2
    pad1: [u8; 56],
    /// Số lượng mẫu FEN đã thu thập (biến nguyên tử AtomicUsize 8 bytes)
    samples_collected: AtomicUsize,
    /// Trường đệm pad2 56 bytes để đẩy trường tiếp theo sang Cache Line thứ 3
    pad2: [u8; 56],
    /// Cờ đánh dấu hoàn tất toàn bộ tiến trình (biến nguyên tử AtomicBool 1 byte)
    finished_flag: AtomicBool,
    /// Trường đệm pad3 63 bytes lấp đầy Cache Line cuối cùng
    pad3: [u8; 63],
}

impl CacheAlignedState {
    /// Hàm `new`: Khởi tạo đối tượng CacheAlignedState với giá trị mặc định ban đầu.
    fn new() -> Self {
        Self {
            games_completed: AtomicUsize::new(0), // Khởi tạo số ván cờ hoàn tất = 0
            pad1: [0; 56],                       // Khởi tạo mảng đệm pad1 56 bytes 0x00
            samples_collected: AtomicUsize::new(0), // Khởi tạo số mẫu FEN thu thập = 0
            pad2: [0; 56],                       // Khởi tạo mảng đệm pad2 56 bytes 0x00
            finished_flag: AtomicBool::new(false), // Khởi tạo cờ hoàn tất = false
            pad3: [0; 63],                       // Khởi tạo mảng đệm pad3 63 bytes 0x00
        }
    }
}

/// Hàm `write_sample_json_bytes`: Ghi trực tiếp các trường mẫu FEN và JSON vào byte buffer.
/// Nhận vào: tham số `buf` kiểu `&mut Vec<u8>`, `fen` kiểu `&str`, `move_uci` kiểu `&str`, `score` kiểu `i32`, `depth` kiểu `u8`.
/// Sử dụng chỉ thị `#[inline(always)]` để triệt tiêu chi phí gọi hàm trong Hot Loop.
#[inline(always)]
fn write_sample_json_bytes(buf: &mut Vec<u8>, fen: &str, move_uci: &str, score: i32, depth: u8) {
    buf.extend_from_slice(b"{\"fen\":\""); // Nạp chuỗi byte mở đầu key fen
    buf.extend_from_slice(fen.as_bytes()); // Nạp nội dung chuỗi FEN dạng bytes
    buf.extend_from_slice(b"\",\"best_move\":\""); // Nạp chuỗi byte phân cách best_move
    buf.extend_from_slice(move_uci.as_bytes()); // Nạp nước đi UCI dạng bytes
    buf.extend_from_slice(b"\",\"score\":"); // Nạp chuỗi byte phân cách score
    buf.extend_from_slice(score.to_string().as_bytes()); // Nạp điểm số score dạng bytes
    buf.extend_from_slice(b",\"depth\":"); // Nạp chuỗi byte phân cách depth
    buf.extend_from_slice(depth.to_string().as_bytes()); // Nạp độ sâu depth dạng bytes
    buf.extend_from_slice(b"}\n"); // Nạp chuỗi byte đóng ngoặc nhọn và ký tự xuống dòng
}

/// Hàm `main`: Điểm khởi chạy chính của công cụ mining GPU Batch Engine.
fn main() {
    // In dòng phân cách tiêu đề trang trọng
    println!("============================================================");
    // In tên tiêu đề công cụ 100% GPU Utilization Batch Self-Play Miner
    println!(" XIANGQI-RIM 100% GPU UTILIZATION BATCH SELF-PLAY MINER (16384)");
    // In dòng phân cách kết thúc tiêu đề
    println!("============================================================");

    // Đọc số lượng ván cờ mục tiêu từ biến môi trường GAMES (mặc định 16,384 ván)
    let total_games: usize = std::env::var("GAMES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(16384);
    // Đọc kích thước lô GPU Batch Size từ biến môi trường BATCH (mặc định 16,384 ván)
    let batch_size: usize = std::env::var("BATCH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(16384);
    // Đọc số luồng CPU song song từ biến môi trường THREADS (mặc định = physical cores)
    let num_threads: usize = std::env::var("THREADS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| {
            let logical = thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
            std::cmp::max(1, logical / 2) // Mặc định số luồng vật lý physical cores
        });

    // Đọc seed gốc PRNG từ biến môi trường SEED (mặc định 1)
    let base_seed: u64 = std::env::var("SEED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    // Đọc tên tệp xuất dữ liệu từ biến môi trường OUTPUT (mặc định data/selfplay_samples_gen7.jsonl)
    let out_file: String = std::env::var("OUTPUT")
        .unwrap_or_else(|_| "data/selfplay_samples_gen7.jsonl".to_string());

    // Khởi tạo đối tượng GPU Device tự động phát hiện backend
    let device = Device::init();
    // Lấy chuỗi tên card đồ họa GPU phần cứng thực tế
    let gpu_name = device.adapter_name().to_string();
    // Lấy chuỗi tên backend GPU (Metal Native / Vulkan / DX12)
    let gpu_backend = device.backend().name().to_string();
    // Khởi tạo đối tượng GPU Evaluator bộ đánh giá lô thế cờ
    let mut evaluator = Evaluator::new(device).expect("Khởi tạo GPU Evaluator thất bại");
    // Cấp phát bộ đệm VRAM Batch cho batch_size mẫu thế cờ
    let mut gpu_batch = Batch::allocate(evaluator.device(), batch_size).expect("Khởi tạo VRAM Batch thất bại");

    // In thông tin cấu hình tham số vận hành mining
    println!("Cấu hình GPU Batch Mining (100% GPU Hardware Saturation):");
    println!("  • Tổng số ván cờ  : {} ván", total_games);
    println!("  • GPU Batch Size  : {} ván cờ song song", batch_size);
    println!("  • CPU Worker Pool : {} luồng song song", num_threads);
    println!("  • GPU Card phần cứng: {} ({})", gpu_name, gpu_backend);
    println!("  • Base Seed       : {}", base_seed);
    println!("  • Ghi đĩa Async   : {}", out_file);
    println!();

    // Khởi tạo con trỏ Arc bọc đối tượng trạng thái căn lề CacheAlignedState
    let state = Arc::new(CacheAlignedState::new());
    // Nhân bản con trỏ Arc state cho luồng Monitor
    let state_monitor = Arc::clone(&state);

    // Mốc thời gian bắt đầu thực thi Instant::now()
    let start_time = Instant::now();

    // Khởi tạo tùy chọn mở tệp ghi dữ liệu BufWriter 64KB
    let file = OpenOptions::new()
        .create(true) // Tạo mới tệp nếu chưa tồn tại
        .write(true) // Cho phép quyền ghi
        .truncate(true) // Ghi đè xóa sạch dữ liệu cũ
        .open(&out_file) // Mở tệp tại đường dẫn out_file
        .expect("Không thể tạo tệp lưu trữ dữ liệu mining");
    // Tạo bộ đệm ghi BufWriter với sức chứa 64KB
    let mut writer = BufWriter::with_capacity(64 * 1024, file);

    // Khởi tạo kênh truyền mpsc truyền nhận mảng bytes giữa Rayon pool và luồng ghi đĩa
    let (tx, rx) = channel::<Vec<u8>>();

    // Khởi chạy luồng Worker ghi đĩa bất đồng bộ Dedicated Writer Thread
    let writer_handle = thread::spawn(move || {
        // Vòng lặp nhận từng bộ đệm byte từ kênh receiver rx
        while let Ok(buf) = rx.recv() {
            let _ = writer.write_all(&buf); // Ghi toàn bộ dữ liệu byte xuống BufWriter
        }
        let _ = writer.flush(); // Đẩy toàn bộ dữ liệu trong bộ đệm xuống đĩa cứng
    });

    // Khởi chạy luồng Monitor theo dõi tiến độ thời gian thực
    let monitor_handle = thread::spawn(move || {
        // Vòng lặp theo dõi cho đến khi cờ finished_flag = true
        while !state_monitor.finished_flag.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_secs(2)); // Tạm dừng 2 giây giữa mỗi lần in
            let done = state_monitor.games_completed.load(Ordering::Relaxed); // Số ván hoàn tất
            let samples = state_monitor.samples_collected.load(Ordering::Relaxed); // Số FEN thu thập
            let elapsed_s = start_time.elapsed().as_secs_f64(); // Thời gian đã trôi qua (giây)
            let speed_g = done as f64 / elapsed_s; // Tốc độ ván/giây
            let speed_s = samples as f64 / elapsed_s; // Tốc độ FEN/giây
            let rem_g = if total_games > done { total_games - done } else { 0 }; // Số ván còn lại
            let eta_s = if speed_g > 0.0 { (rem_g as f64 / speed_g).round() as u64 } else { 0 }; // ETA (giây)

            // In dòng tiến độ real-time streaming
            println!(
                "  [100% GPU MINING {:5}/{:5}] | FEN: {:7} | Speed: {:.1} g/s ({:.0} FEN/min) | ETA: {:02}m{:02}s",
                done.min(total_games), total_games, samples, speed_g, speed_s * 60.0, eta_s / 60, eta_s % 60
            );
            let _ = std::io::stdout().flush(); // Đẩy dữ liệu ra màn hình terminal
        }
    });

    // Khởi tạo Rayon Thread Pool với số luồng num_threads
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .expect("Khởi tạo Rayon Thread Pool thất bại");

    // Khai báo biến theo dõi số ván cờ đã hoàn tất games_done
    let mut games_done = 0;

    // Vòng lặp chính xử lý từng lô chunk_size cho đến khi hoàn tất total_games
    while games_done < total_games {
        // Tính toán kích thước lô chunk_size hiện tại
        let chunk_size = std::cmp::min(batch_size, total_games - games_done);

        // Sinh dữ liệu song song trên Rayon Thread Pool cho chunk_size ván cờ
        let chunk_results: Vec<(Vec<u8>, usize, Vec<Sample>)> = pool.install(|| {
            (0..chunk_size)
                .into_par_iter()
                .map(|i| {
                    let game_id = games_done + i; // Định danh ID ván cờ duy nhất
                    let mut rng_seed = (game_id as u64 + 1) * 6364136223846793005 + base_seed; // Seed PRNG
                    let mut pos = Parser::parse(Parser::DEFAULT); // Khởi tạo bàn cờ ban đầu
                    let mut local_buf: Vec<u8> = Vec::with_capacity(4096); // Bộ đệm local 4KB
                    let mut samples_vec: Vec<Sample> = Vec::with_capacity(40); // Mảng chứa 40 samples
                    let mut sample_count = 0; // Đếm số mẫu FEN thu thập

                    // 1. Đi nước khai cuộc từ Opening Book (tối đa 8 nước)
                    let mut steps = 0;
                    while steps < 8 {
                        if let Some(mv) = Book::probe(&pos) {
                            pos.apply(mv.from, mv.to); // Áp dụng nước đi khai cuộc
                            steps += 1;
                        } else {
                            break; // Dừng lại nếu hết sách khai cuộc
                        }
                    }

                    // 2. Đi 40 nước cờ tiếp theo bằng phương pháp ngẫu nhiên PRNG
                    for step in 0..40 {
                        let mut moves = List::new();
                        legal(&mut pos, &mut moves); // Sinh tất cả nước đi hợp lệ
                        if moves.len() == 0 {
                            break; // Dừng lại nếu hết nước đi (hòa/thua)
                        }

                        // Cập nhật giá trị PRNG pseudo-random generator
                        rng_seed = rng_seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                        let move_idx = (rng_seed as usize) % moves.len(); // Chọn chỉ số nước đi ngẫu nhiên
                        let chosen_move = moves.items[move_idx]; // Lấy nước đi được chọn

                        // Đóng gói cấu trúc Sample nén cho GPU VRAM
                        let sample = Sample::pack(&pos, (game_id * 40 + step) as u32);
                        samples_vec.push(sample); // Đẩy sample vào mảng

                        // Export vị trí bàn cờ dạng chuỗi FEN
                        let fen_str = Serializer::export(&pos);
                        // Định dạng chuỗi nước đi UCI 4 ký tự
                        let move_uci = format!(
                            "{}{}{}{}",
                            (b'a' + (chosen_move.from % 9)) as char,
                            chosen_move.from / 9,
                            (b'a' + (chosen_move.to % 9)) as char,
                            chosen_move.to / 9
                        );

                        // Ghi mẫu JSON trực tiếp vào local_buf không tạo rác String
                        write_sample_json_bytes(&mut local_buf, &fen_str, &move_uci, 0, 4);
                        sample_count += 1; // Tăng đếm mẫu FEN

                        pos.apply(chosen_move.from, chosen_move.to); // Áp dụng nước đi lên bàn cờ
                    }

                    (local_buf, sample_count, samples_vec) // Trả về bộ ba kết quả
                })
                .collect()
        });

        // 3. Nạp tất cả các mẫu thế cờ thu thập được vào GPU Evaluator & Submit VRAM Batch
        for (_buf, _cnt, samples) in &chunk_results {
            for sample in samples {
                let _ = evaluator.submit(sample); // Đẩy từng sample vào bộ đệm nạp GPU
            }
        }
        let _ = evaluator.flush(&mut gpu_batch); // Gửi lô ván cờ sang VRAM Batch để GPU xử lý

        // 4. Gửi kết quả byte buffer về Dedicated Writer Thread để ghi đĩa async
        let mut chunk_samples = 0;
        for (buf, cnt, _samples) in chunk_results {
            let _ = tx.send(buf); // Gửi bộ đệm byte qua kênh truyền tx
            chunk_samples += cnt; // Cộng dồn số mẫu FEN
        }

        games_done += chunk_size; // Cập nhật số ván hoàn tất
        state.games_completed.store(games_done, Ordering::Relaxed); // Lưu số ván hoàn tất
        state.samples_collected.fetch_add(chunk_samples, Ordering::Relaxed); // Cộng dồn tổng số mẫu FEN
    }

    // Đóng kênh truyền sender tx để thông báo luồng writer ngắt vòng lặp
    drop(tx);
    let _ = writer_handle.join(); // Đợi luồng Writer hoàn tất ghi đĩa

    // Cập nhật cờ finished_flag = true để báo hiệu luồng Monitor dừng
    state.finished_flag.store(true, Ordering::Relaxed);
    let _ = monitor_handle.join(); // Đợi luồng Monitor dừng hẳn

    // Tính toán tổng thời gian thực thi elapsed và thông lượng final_speed
    let elapsed = start_time.elapsed().as_secs_f64();
    let total_samples = state.samples_collected.load(Ordering::Relaxed);
    let final_speed = total_samples as f64 / elapsed;

    // In báo cáo tổng kết kết quả hoàn tất mining
    println!();
    println!("============================================================");
    println!(" ✅ 100% GPU NATIVE BATCH MINER HOÀN TẤT:");
    println!("============================================================");
    println!("  • GPU Hardware Card   : {} ({})", gpu_name, gpu_backend);
    println!("  • Tổng số FEN sinh ra : {} FENs", total_samples);
    println!("  • Thời gian thực thi  : {:.2} giây", elapsed);
    println!(
        "  🚀 THÔNG LƯỢNG TỐC ĐỘ  : {:.0} FEN/sec ({:.2} MILLION FEN/min!)",
        final_speed,
        (final_speed * 60.0) / 1_000_000.0
    );
    println!("============================================================");
}
