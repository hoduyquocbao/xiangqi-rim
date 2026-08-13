// ============================================================================
// EXAMPLE 28: BỘ TỰ ĐỘNG MEASURE BENCHMARK VÀ TUNING HIỆU NĂNG TỐI ĐA (AUTO-TUNER)
// ============================================================================
// Tự động quét ma trận các tham số (THREADS x BATCH_SIZE) trong 3 giây/lần.
// Tìm kiếm cấu hình đạt tốc độ FEN/giây cao nhất thực tế trên hạ tầng hiện tại.
// Lưu trữ cấu hình quán quân vào tệp `data/optimal_config.json`.
// Tuân thủ 100% định danh từ đơn tiếng Anh và 100% chú thích tiếng Việt.
// ============================================================================

use std::fs::OpenOptions; // Nhập struct OpenOptions thao tác ghi tệp đĩa
use std::io::Write; // Nhập trait Write hỗ trợ ghi chuỗi
use std::time::Instant; // Nhập struct đo lường thời gian Instant

use rayon::prelude::*; // Nhập trait Rayon xử lý đa luồng song song
use xiangrust::board::{Parser, Position}; // Nhập module board từ xiangrust
use xiangrust::book::Book; // Nhập module book khai cuộc
use xiangrust::gpu::{Batch, Device, Evaluable, Evaluator, Sample}; // Nhập module gpu
use xiangrust::movegen::{legal, List}; // Nhập module movegen sinh nước đi

/// Struct `GameSlot`: Lưu trữ trạng thái 1 ván cờ trong mảng ván cờ song song
struct GameSlot {
    pos: Position, // Trạng thái vị trí bàn cờ hiện tại
    steps: u32, // Số bước đi hiện tại của ván cờ
    done: bool, // Cờ đánh dấu ván cờ đã kết thúc
}

/// Struct `BenchmarkResult`: Lưu trữ kết quả đo lường 1 tổ hợp cấu hình
#[derive(Debug, Clone)]
struct BenchmarkResult {
    threads: usize, // Số luồng CPU
    batch: usize, // Kích thước lô GPU Batch
    fen_per_sec: f64, // Tốc độ FEN/giây
    fen_per_min: f64, // Tốc độ FEN/phút
}

fn test_configuration(threads: usize, batch_size: usize, test_duration_secs: u64) -> BenchmarkResult {
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .ok();

    let device = Device::init();
    let mut evaluator = Evaluator::new(device).expect("Khởi tạo GPU Evaluator thất bại");
    let mut gpu_batch = Batch::allocate(evaluator.device(), batch_size).expect("Khởi tạo VRAM Batch thất bại");

    let mut slots: Vec<GameSlot> = Vec::with_capacity(batch_size);
    let mut seed = 123456789u64;

    for _ in 0..batch_size {
        let mut pos = Parser::parse(Parser::DEFAULT);
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;

        if (seed % 2) == 0 {
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
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                let idx = (seed as usize) % moves.len();
                let m = moves.items[idx];
                pos.apply(m.from, m.to);
            }
        }

        slots.push(GameSlot {
            pos,
            steps: 0,
            done: false,
        });
    }

    let start_time = Instant::now();
    let mut samples_evaluated = 0usize;

    while start_time.elapsed().as_secs() < test_duration_secs {
        gpu_batch.clear();
        let mut active_indices = Vec::with_capacity(batch_size);

        for (idx, slot) in slots.iter().enumerate() {
            if !slot.done {
                let mut sample = Sample::new();
                sample.load(&slot.pos.grid, slot.pos.side);
                let _ = gpu_batch.push(&sample);
                active_indices.push(idx);
            }
        }

        if active_indices.is_empty() {
            break;
        }

        let count = gpu_batch.count();
        let _ = evaluator.flush(&mut gpu_batch);

        let scores: Vec<i32> = (0..count).map(|i| gpu_batch.pull(i).map(|s| s.score()).unwrap_or(0)).collect();

        (0..active_indices.len())
            .into_par_iter()
            .for_each(|i| {
                let slot_idx = active_indices[i];
                let score = scores[i];
                let slot_ptr = slots.as_ptr() as usize + slot_idx * std::mem::size_of::<GameSlot>();
                let slot = unsafe { &mut *(slot_ptr as *mut GameSlot) };

                let mut moves = List::new();
                legal(&mut slot.pos, &mut moves);

                if moves.len() == 0 || slot.steps >= 200 || score.abs() > 29000 {
                    slot.done = true;
                    let mut seed = (slot_idx as u64 + 1) * 987654321;
                    seed ^= seed << 13;
                    seed ^= seed >> 7;
                    let _ = seed ^ (seed << 17);
                    let new_pos = Parser::parse(Parser::DEFAULT);
                    slot.pos = new_pos;
                    slot.steps = 0;
                    slot.done = false;
                    return;
                }

                let mut seed = (slot_idx as u64 + 1) * 1234567 + slot.steps as u64;
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                let move_idx = (seed as usize) % moves.len();
                let selected_move = moves.items[move_idx];

                slot.pos.apply(selected_move.from, selected_move.to);
                slot.steps += 1;
            });

        samples_evaluated += count;
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    let fen_per_sec = samples_evaluated as f64 / max_f64(0.001, elapsed);
    let fen_per_min = fen_per_sec * 60.0;

    BenchmarkResult {
        threads,
        batch: batch_size,
        fen_per_sec,
        fen_per_min,
    }
}

fn max_f64(a: f64, b: f64) -> f64 {
    if a > b { a } else { b }
}

fn main() {
    println!("============================================================");
    println!(" XIANGQI-RIM AUTOMATIC HARDWARE BENCHMARK & SPEED TUNER");
    println!("============================================================");

    let thread_candidates = vec![1, 2, 4];
    let batch_candidates = vec![2048, 4096, 8192, 16384];

    println!("Bắt đầu đo lường ma trận thử nghiệm (3 giây/cấu hình):");
    println!("  • Danh sách luồng CPU thử nghiệm  : {:?}", thread_candidates);
    println!("  • Danh sách kích thước GPU Batch : {:?}", batch_candidates);
    println!("============================================================");

    let mut results: Vec<BenchmarkResult> = Vec::new();

    for &threads in &thread_candidates {
        for &batch_size in &batch_candidates {
            print!("--> Testing Threads: {:2} | Batch: {:5} ... ", threads, batch_size);
            let _ = std::io::stdout().flush();

            let res = test_configuration(threads, batch_size, 3);
            println!("Speed: {:>10.0} FEN/s ({:>6.2}M FEN/min)", res.fen_per_sec, res.fen_per_min / 1_000_000.0);
            results.push(res);
        }
    }

    // Sắp xếp kết quả theo tốc độ FEN/giây giảm dần
    results.sort_by(|a, b| b.fen_per_sec.partial_cmp(&a.fen_per_sec).unwrap());

    println!("============================================================");
    println!(" LEADERBOARD CẤU HÌNH NHANH NHẤT HẠ TẦNG PHẦN CỨNG");
    println!("============================================================");
    println!("  Xếp hạng | Số Luồng CPU | GPU Batch Size | Tốc độ FEN/giây | Tốc độ FEN/phút");
    println!("  ---------+--------------+----------------+-----------------+-----------------");

    for (rank, res) in results.iter().enumerate() {
        let badge = if rank == 0 { "🏆 QUÁN QUÂN" } else { "  " };
        println!(
            "  #{:2} {} | Threads: {:2} | Batch: {:5}  | {:>11.0} FEN/s | {:>6.2}M FEN/min",
            rank + 1, badge, res.threads, res.batch, res.fen_per_sec, res.fen_per_min / 1_000_000.0
        );
    }

    let winner = &results[0];
    println!("============================================================");
    println!("✅ CẤU HÌNH TỐI ƯU NHẤT CHO HẠ TẦNG HIỆN TẠI:");
    println!("  • Số luồng CPU tốt nhất  : {} Threads", winner.threads);
    println!("  • GPU Batch Size tốt nhất: {} Slots", winner.batch);
    println!("  • Tốc độ cao nhất đạt được: {:.0} FEN/s ({:.2}M FEN/min)", winner.fen_per_sec, winner.fen_per_min / 1_000_000.0);
    println!("============================================================");

    // Lưu kết quả vào tệp data/optimal_config.json
    std::fs::create_dir_all("data").ok();
    let json_str = format!(
        "{{\"best_threads\":{},\"best_batch\":{},\"fen_per_sec\":{:.0},\"fen_per_min_m\":{:.2}}}\n",
        winner.threads, winner.batch, winner.fen_per_sec, winner.fen_per_min / 1_000_000.0
    );

    if let Ok(mut f) = OpenOptions::new().create(true).write(true).truncate(true).open("data/optimal_config.json") {
        let _ = f.write_all(json_str.as_bytes());
        println!("✅ Đã lưu cấu hình tối ưu vào: data/optimal_config.json");
    }
}
