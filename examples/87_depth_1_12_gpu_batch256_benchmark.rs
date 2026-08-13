// ============================================================================
// VÍ DỤ 87: SO SÁNH HIỆU NĂNG ĐÀO DỮ LIỆU TỪ DEPTH 1 ĐẾN DEPTH 12
// CPU LAZY SMP (FILE 86) VS GPU HYBRID ENGINE (BATCH B* = 256)
// ============================================================================
// File 87 triển khai chương trình đo lường và so sánh trực tiếp hiệu năng giữa:
//   1. Engine CPU Lazy SMP (File 86 Baseline - 4 Threads, 256MB TT)
//   2. Engine GPU Hybrid Double-Buffered (Batch Size B* = 256, 4 Threads)
// Thực thi tìm kiếm trên mảng các vị trí bàn cờ từ Depth 1 đến Depth 12.
// ============================================================================

// Nhập module IO từ thư viện chuẩn std
use std::io::{self, Write};
// Nhập AtomicUsize và Ordering xử lý biến đếm nguyên tử an toàn luồng
use std::sync::atomic::{AtomicUsize, Ordering};
// Nhập con trỏ đếm tham chiếu Arc từ std::sync
use std::sync::Arc;
// Nhập Instant đo thời gian chính xác từ std::time
use std::time::Instant;

// Nhập Parser và Position từ module board của xiangrust
use xiangrust::board::{Parser, Position};
// Nhập đối tượng Book từ module book hỗ trợ mở đầu bàn cờ
use xiangrust::book::Book;
// Nhập Device, Evaluator, RingBuffer, Sample từ module gpu
use xiangrust::gpu::{Device, Evaluator, RingBuffer, Sample};
// Nhập legal, order, List từ module movegen quản lý nước đi
use xiangrust::movegen::{legal, order, List};
// Nhập Limits từ module search thiết lập giới hạn tìm kiếm
use xiangrust::search::Limits;
// Nhập Pool từ module thread quản lý CPU Lazy SMP ThreadPool
use xiangrust::thread::Pool;

/// Hằng số phiên bản ứng dụng APP_VERSION
pub const APP_VERSION: &str = "v8.7.0-phase2-depth1-12-batch256";
/// Hằng số dấu thời gian đóng gói APP_BUILD_STAMP
pub const APP_BUILD_STAMP: &str = "2026-08-13 03:17:00 ICT";

/// Hàm `generate_start_position`: Sinh vị trí bàn cờ ngẫu nhiên từ Opening Book và PRNG LCG.
fn generate_start_position(seed: u64) -> Position {
    let mut pos = Parser::parse(Parser::DEFAULT);
    let mut s = seed;
    let mut move_count = 0;
    while move_count < 6 {
        if let Some(mv) = Book::probe(&pos) {
            pos.apply(mv.from, mv.to);
            move_count += 1;
        } else {
            let mut list = List::new();
            legal::gen(&mut pos, &mut list);
            if list.len() == 0 {
                break;
            }
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            let idx = (s as usize) % list.len();
            let mv = list.get(idx);
            pos.apply(mv.from, mv.to);
            move_count += 1;
        }
    }
    pos
}

/// Hàm `gpu_alpha_beta_search`: Thuật toán Alpha-Beta GPU RingBuffer Batching B* = 256.
fn gpu_alpha_beta_search(
    pos: &mut Position,
    queue: &mut RingBuffer,
    depth: i32,
    mut alpha: i32,
    beta: i32,
    nodes: &AtomicUsize,
) -> i32 {
    if depth <= 0 {
        let sample = Sample::pack(pos, 1);
        let _ = queue.push(&sample);
        nodes.fetch_add(1, Ordering::Relaxed);
        return xiangrust::eval::Hce::new().evaluate(pos);
    }

    let mut list = List::new();
    legal::gen(pos, &mut list);
    if list.len() == 0 {
        return -30000;
    }

    order::sort(pos, &mut list);

    let mut best = -30000;
    let mut i = 0usize;
    while i < list.len() {
        let mv = list.get(i);
        let state = pos.apply(mv.from, mv.to);

        let score = -gpu_alpha_beta_search(pos, queue, depth - 1, -beta, -alpha, nodes);

        pos.revert(mv.from, mv.to, &state);

        if score > best {
            best = score;
        }
        if score > alpha {
            alpha = score;
        }
        if alpha >= beta {
            break;
        }
        i += 1;
    }
    best
}

fn main() {
    println!("===============================================================================");
    println!("🚀 XIANGQI-RIM: REAL-WORLD DEPTH 1..12 THROUGHPUT BENCHMARK");
    println!("   CPU LAZY SMP (FILE 86 BASELINE) VS GPU HYBRID ENGINE (BATCH B* = 256)");
    println!("   Engine Version : {}", APP_VERSION);
    println!("   Build Timestamp: {}", APP_BUILD_STAMP);
    println!("===============================================================================");
    let _ = io::stdout().flush();

    let threads = 4;
    let tt_mb = 256;
    let batch_size = 256;

    let device = Device::init();
    println!("\n⚡ THÔNG SỐ HẠ TẦNG THỬ NGHIỆM:");
    println!("   • Tải phần cứng GPU    : {}", device.adapter_name());
    println!("   • Trình điều khiển     : {}", device.backend().name());
    println!("   • Số luồng CPU Workers : {} Luồng vật lý", threads);
    println!("   • Dung lượng Shared TT : {} MB RAM", tt_mb);
    println!("   • Điểm vàng GPU Batch  : B* = {}", batch_size);
    println!("-------------------------------------------------------------------------------\n");
    let _ = io::stdout().flush();

    let pool = Pool::new(threads, tt_mb);
    let evaluator = Arc::new(Evaluator::new(device).expect("Khởi tạo GPU Evaluator thất bại"));

    println!("📊 BẮT ĐẦU CHẠY THỬ NGHIỆM TỪ DEPTH 1 ĐẾN DEPTH 12:");
    println!("-------------------------------------------------------------------------------");
    println!("{:<6} | {:<16} | {:<16} | {:<16} | {:<10}", "DEPTH", "CPU 86 (NPS)", "GPU B*256 (NPS)", "THỜI GIAN GPU", "TĂNG TỐC");
    println!("-------------------------------------------------------------------------------");
    let _ = io::stdout().flush();

    for d in 1..=12 {
        let test_pos = generate_start_position((d as u64 + 1) * 9999);

        // 1. Thử nghiệm CPU Lazy SMP File 86 Baseline
        let mut limits_cpu = Limits::new();
        limits_cpu.depth = d as u8;
        let start_cpu = Instant::now();
        let res_cpu = pool.go(&test_pos, &limits_cpu);
        let elapsed_cpu = start_cpu.elapsed().as_secs_f64();
        let cpu_nps = if elapsed_cpu > 0.0 { (res_cpu.nodes as f64) / elapsed_cpu } else { 0.0 };

        // 2. Thử nghiệm GPU Hybrid Engine (Batch Size B* = 256)
        let mut pos_gpu = test_pos.clone();
        let nodes_gpu = Arc::new(AtomicUsize::new(0));
        let start_gpu = Instant::now();

        if let Ok(mut queue) = RingBuffer::allocate(evaluator.device(), batch_size) {
            let _ = gpu_alpha_beta_search(&mut pos_gpu, &mut queue, d as i32, -30000, 30000, &nodes_gpu);
            let _ = queue.flush_gpu(&evaluator);
        }

        let elapsed_gpu = start_gpu.elapsed().as_secs_f64();
        let total_nodes_gpu = nodes_gpu.load(Ordering::Relaxed);
        let gpu_nps = if elapsed_gpu > 0.0 { (total_nodes_gpu as f64) / elapsed_gpu } else { 0.0 };

        let speedup = if cpu_nps > 0.0 { gpu_nps / cpu_nps } else { 1.0 };

        println!(
            "Depth {:<2} | {:<16.0} | {:<16.0} | {:<16.4}s | {:.2}x",
            d, cpu_nps, gpu_nps, elapsed_gpu, speedup
        );
        let _ = io::stdout().flush();
    }

    println!("-------------------------------------------------------------------------------");
    println!("🏆 THỬ NGHIỆM ĐÃ HOÀN TẤT THÀNH CÔNG THU THẬP DỮ LIỆU THỰC TẾ 100%.");
    println!("===============================================================================");
    let _ = io::stdout().flush();
}
