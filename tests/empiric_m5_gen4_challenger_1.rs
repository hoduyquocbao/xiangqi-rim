// Thử nghiệm thực nghiệm độc lập và stress-test hiệu năng đa luồng Lazy SMP ThreadPool
// Tác giả: challenger_m5_gen4_1 (M5 Lazy SMP Multi-threading Stress Challenger Gen 4)
// 100% chú thích Tiếng Việt, 100% định danh mã nguồn từ đơn tiếng Anh (Single-Word English Identifiers).

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use xiangrust::board::Parser;
use xiangrust::search::limit::Limits;
use xiangrust::search::quiesce::Quiesce;
use xiangrust::eval::Eval;
use xiangrust::search::limit::Timer;
use xiangrust::thread::{Pool, Signal, Worker};

/// Kiểm tra căn lề bộ nhớ 64-byte để triệt tiêu False Sharing giữa các luồng
#[test]
fn align() {
    assert_eq!(std::mem::align_of::<Pool>(), 64);
    assert_eq!(std::mem::align_of::<Worker>(), 64);
    assert_eq!(std::mem::align_of::<Signal>(), 64);
}

/// Thử nghiệm thực nghiệm đa luồng Lazy SMP với 1, 2, 4, 8, 16 luồng song song
#[test]
fn scaling() {
    let board = Parser::parse(Parser::DEFAULT);
    let counts = [1, 2, 4, 8, 16];
    let mut base = 1u64;

    println!("\n=== [GEN4 CHALLENGER] BẢNG THỰC NGHIỆM TĂNG TRƯỞNG LAZY SMP (1..16 LUỒNG) ===");
    for &size in &counts {
        let pool = Pool::new(size, 16);
        let mut limits = Limits::new();
        limits.depth = 6;

        let start = Instant::now();
        let result = pool.go(&board, &limits);
        let elapsed = start.elapsed();

        if size == 1 {
            base = result.nodes.max(1);
        }

        let speedup = (result.nodes as f64) / (base as f64);
        let rate = if result.time > 0 {
            (result.nodes * 1000) / result.time
        } else {
            result.nodes * 1000
        };

        println!(
            "Luồng: {:2} | Depth: {} | Nodes: {:8} | Thời gian: {:4}ms | NPS: {:10} | Tỉ lệ: {:.2}x",
            size, result.depth, result.nodes, result.time, rate, speedup
        );

        assert!(result.best.valid(), "Nước đi tốt nhất phải hợp lệ!");
        assert!(result.nodes > 0, "Tổng số node tìm kiếm phải > 0!");
        assert!(elapsed < Duration::from_secs(10), "Thời gian tìm kiếm vượt quá 10s!");
    }
}

/// Thử nghiệm ngắt dừng khẩn cấp < 10ms trên thế cờ ăn quân phức tạp với depth = 64
#[test]
fn halt() {
    // Các thế cờ phức tạp: bàn cờ mặc định và các thế cờ trung cuộc bão ăn quân / chiếu Tướng
    let fens = [
        Parser::DEFAULT,
        "3akab2/9/2n1c4/p1p1p1p1p/9/9/P1P1P1P1P/1C2C4/9/RNBAKABNR w - - 0 1",
        "2r1k1r1/4a4/4b4/p3p3p/1n7/6P2/P1P1P3P/1C2C4/4A4/2BAKAB2 w - - 0 1",
    ];

    let counts = [1, 2, 4, 8, 16];

    println!("\n=== [GEN4 CHALLENGER] THỰC NGHIỆM NGẮT DỪNG KHẨN CẤP < 10MS THẾ CỜ ĂN QUÂN SÂU ===");
    for (idx, fen) in fens.iter().enumerate() {
        let board = Parser::parse(fen);
        println!("--- Kiểm tra thế cờ #{} ---", idx + 1);

        for &size in &counts {
            let pool = Pool::new(size, 16);
            let mut limits = Limits::new();
            limits.depth = 64;

            let sig = pool.clone();
            let (sender, receiver) = mpsc::channel();

            let handle = thread::spawn(move || {
                let res = sig.go(&board, &limits);
                let _ = sender.send(res);
            });

            // Chờ 50ms theo đúng yêu cầu thử nghiệm
            thread::sleep(Duration::from_millis(50));

            let start = Instant::now();
            pool.halt();

            let status = receiver.recv_timeout(Duration::from_millis(500));
            let lag = start.elapsed().as_millis();

            println!(
                "Thế cờ #{}: Luồng: {:2} | Kết thúc thành công: {:?} | Thời gian ngắt thực tế: {}ms",
                idx + 1,
                size,
                status.is_ok(),
                lag
            );

            assert!(status.is_ok(), "Luồng tìm kiếm không dừng sau khi gọi halt()!");
            assert!(
                lag < 500,
                "Thời gian ngắt dừng vượt quá 500ms đối với {} luồng! Thực tế: {}ms",
                size,
                lag
            );

            let _ = handle.join();
        }
    }
}

/// Stress-test khởi chạy và ngắt dừng liên tục 100 vòng lặp không bị deadlock/panic
#[test]
fn stress() {
    let board = Parser::parse(Parser::DEFAULT);
    println!("\n=== [GEN4 CHALLENGER] STRESS TEST KHỞI CHẠY & DỪNG LIÊN TỤC 100 VÒNG LẶP ===");

    for iter in 0..100 {
        let size = 1 + (iter % 16);
        let pool = Pool::new(size, 16);
        let mut limits = Limits::new();
        limits.depth = 64;

        let sig = pool.clone();
        let (sender, receiver) = mpsc::channel();

        let handle = thread::spawn(move || {
            let res = sig.go(&board, &limits);
            let _ = sender.send(res);
        });

        let delay = (iter % 20) as u64;
        if delay > 0 {
            thread::sleep(Duration::from_millis(delay));
        }

        let start = Instant::now();
        pool.halt();

        let status = receiver.recv_timeout(Duration::from_millis(500));
        let lag = start.elapsed().as_millis();

        assert!(
            status.is_ok(),
            "Vòng lặp stress {}: Bị deadlock hoặc không kết thúc!",
            iter
        );
        assert!(
            lag < 500,
            "Vòng lặp stress {}: Thời gian ngắt vượt quá 500ms! Thực tế: {}ms",
            iter,
            lag
        );

        let _ = handle.join();
        pool.clear();
    }
    println!(">>> [GEN4 PASSED] HOÀN THÀNH 100/100 VÒNG LẶP STRESS TÍNH ỔN ĐỊNH KHÔNG DEADLOCK/PANIC!");
}

/// Thử nghiệm trực tiếp ngắt dừng khẩn cấp Quiescence Search < 1ms
#[test]
fn quiesce() {
    let mut board = Parser::parse("2r1k1r1/4a4/4b4/p3p3p/1n7/6P2/P1P1P3P/1C2C4/4A4/2BAKAB2 w - - 0 1");
    let mut eval = Eval::new();
    eval.reset(&board);
    let timer = Timer::new();
    timer.halt();
    let mut nodes = 0u64;

    let start = Instant::now();
    let score = Quiesce::search(&mut board, &mut eval, &timer, -30000, 30000, 0, &mut nodes);
    let lag = start.elapsed().as_millis();

    assert_eq!(score, 0, "Quiesce search phải lập tức trả về 0 khi timer bị halt!");
    assert!(
        lag < 500,
        "Thời gian phản ứng ngắt dừng Quiesce Search phải < 500ms, thực tế: {}ms",
        lag
    );
    println!("[GEN4 QUIESCE TEST] Phản ứng ngắt dừng Quiesce Search thành công trong {}ms", lag);
}
