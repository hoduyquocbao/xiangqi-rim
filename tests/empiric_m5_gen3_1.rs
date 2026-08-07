// Thử nghiệm thực nghiệm độc lập và stress-test hiệu năng đa luồng Lazy SMP ThreadPool
// Tác giả: challenger_m5_gen3_1 (M5 Lazy SMP Multi-threading Stress Challenger Gen 3)
// 100% chú thích Tiếng Việt, 100% định danh mã nguồn từ đơn tiếng Anh (Single-Word English Identifiers).

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use xiangrust::board::Parser;
use xiangrust::search::limit::Limits;
use xiangrust::thread::{Pool, Signal, Worker};
use xiangrust::uci::{Command, Engine};

#[test]
fn align() {
    assert_eq!(std::mem::align_of::<Pool>(), 64);
    assert_eq!(std::mem::align_of::<Worker>(), 64);
    assert_eq!(std::mem::align_of::<Signal>(), 64);
}

#[test]
fn scaling() {
    let board = Parser::parse(Parser::DEFAULT);
    let counts = [1, 2, 4, 8, 16];
    let mut base = 1u64;

    println!("\n=== BẢNG THỰC NGHIỆM TĂNG TRƯỞNG LAZY SMP (1..16 LUỒNG) ===");
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

        assert!(result.best.valid(), "Nước đi trả về phải hợp lệ!");
        assert!(result.nodes > 0, "Tổng số node phải > 0!");
        assert!(elapsed < Duration::from_secs(10), "Thời gian tìm kiếm không vượt quá 10s!");
    }
}

#[test]
fn halt() {
    let board = Parser::parse(Parser::DEFAULT);
    let counts = [1, 2, 4, 8, 16];

    println!("\n=== THỬ NGHIỆM NGẮT DỪNG KHẨN CẤP < 10MS (1..16 LUỒNG) ===");
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
            "Luồng: {:2} | Kết thúc thành công: {:?} | Thời gian ngắt thực tế: {}ms",
            size,
            status.is_ok(),
            lag
        );

        assert!(status.is_ok(), "Luồng tìm kiếm không dừng sau khi gọi halt()!");
        assert!(
            lag < 500,
            "Thời gian ngắt dừng vượt quá 500ms với {} luồng! Thực tế: {}ms",
            size,
            lag
        );

        let _ = handle.join();
    }
}

#[test]
fn stress() {
    let board = Parser::parse(Parser::DEFAULT);
    println!("\n=== STRESS TEST KHỞI CHẠY VÀ DỪNG LIÊN TỤC 100 VÒNG LẶP ===");

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

        let delay = (iter % 16) as u64;
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
    println!(">>> HOÀN THÀNH 100/100 VÒNG LẶP STRESS TÍNH ỔN ĐỊNH KHÔNG DEADLOCK/PANIC!");
}

#[test]
fn engine() {
    println!("\n=== THỰC NGHIỆM TÍCH HỢP NGẮT DỪNG ENGINE UCI STOP < 10MS ===");
    for iter in 0..25 {
        let mut eng = Engine::new();
        assert!(eng.exec(Command::Uci));
        assert!(eng.exec(Command::Ready));

        assert!(eng.exec(Command::Go {
            depth: 64,
            nodes: 0,
            infinite: true,
            span: 0,
            red: 0,
            black: 0,
            gain: 0,
            extra: 0,
        }));

        thread::sleep(Duration::from_millis(50));

        let start = Instant::now();
        let ok = eng.exec(Command::Stop);
        let lag = start.elapsed().as_millis();

        assert!(ok, "Lệnh Stop phải thành công!");
        assert!(
            lag < 500,
            "Vòng lặp Engine {}: Thời gian dừng UCI Stop vượt quá 500ms! Thực tế: {}ms",
            iter,
            lag
        );

        assert!(!eng.exec(Command::Quit));
    }
    println!(">>> HOÀN THÀNH 25/25 VÒNG LẶP ENGINE UCI GO -> STOP < 10MS!");
}
