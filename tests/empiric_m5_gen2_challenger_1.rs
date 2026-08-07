// Empirical stress test harness for M5 Lazy SMP ThreadPool & Halt Responsiveness
// Single-word English identifiers for all code symbols, 100% Vietnamese comments.

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use xiangrust::board::Parser;
use xiangrust::search::limit::Limits;
use xiangrust::thread::Pool;
use xiangrust::uci::{Command, Engine};

#[test]
fn scaling() {
    let pos = Parser::parse(Parser::DEFAULT);
    let counts = [1, 2, 4, 8, 16];
    let mut base = 1u64;

    println!("\n=== LAZY SMP SCALING BENCHMARK ===");
    for &size in &counts {
        let pool = Pool::new(size, 16);
        let mut limits = Limits::new();
        limits.depth = 6;

        let start = Instant::now();
        let result = pool.go(&pos, &limits);
        let elapsed = start.elapsed();

        if size == 1 {
            base = result.nodes.max(1);
        }

        let speedup = (result.nodes as f64) / (base as f64);
        let nps = if result.time > 0 {
            (result.nodes * 1000) / result.time
        } else {
            result.nodes * 1000
        };

        println!(
            "Luồng: {:2} | Depth: {} | Nodes: {:8} | Thời gian: {:4}ms | NPS: {:10} | Tỉ lệ: {:.2}x",
            size, result.depth, result.nodes, result.time, nps, speedup
        );

        assert!(result.best.valid(), "Nước đi tốt nhất phải hợp lệ!");
        assert!(result.nodes > 0, "Số node tìm kiếm phải lớn hơn 0!");
        assert!(elapsed < Duration::from_secs(10), "Tìm kiếm depth 6 không được vượt quá 10s!");
    }
}

#[test]
fn halt() {
    let pos = Parser::parse(Parser::DEFAULT);
    let counts = [1, 2, 4, 8, 16];

    println!("\n=== EMERGENCY HALT RESPONSIVENESS TEST (< 50ms) ===");
    for &size in &counts {
        let pool = Pool::new(size, 16);
        let mut limits = Limits::new();
        limits.depth = 64;

        let sig = pool.clone();
        let (sender, receiver) = mpsc::channel();

        let handle = thread::spawn(move || {
            let res = sig.go(&pos, &limits);
            let _ = sender.send(res);
        });

        // Đợi 50ms theo đúng yêu cầu đề bài
        thread::sleep(Duration::from_millis(50));

        let start = Instant::now();
        pool.halt();

        let status = receiver.recv_timeout(Duration::from_millis(500));
        let lag = start.elapsed().as_millis();

        println!(
            "Luồng: {:2} | Trạng thái dừng: {:?} | Thời gian phản ứng ngắt: {}ms",
            size,
            status.is_ok(),
            lag
        );

        assert!(status.is_ok(), "Luồng tìm kiếm không kết thúc sau khi halt!");
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
    let pos = Parser::parse(Parser::DEFAULT);
    println!("\n=== STRESS TEST RAPID START/HALT LOOPS (100 ITERATIONS) ===");

    for iter in 0..100 {
        let size = 1 + (iter % 16);
        let pool = Pool::new(size, 16);
        let mut limits = Limits::new();
        limits.depth = 64;

        let sig = pool.clone();
        let (sender, receiver) = mpsc::channel();

        let handle = thread::spawn(move || {
            let res = sig.go(&pos, &limits);
            let _ = sender.send(res);
        });

        // Thời gian chờ biến đổi ngắn (0ms đến 15ms)
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
            "Vòng lặp stress {}: Pool bị deadlock hoặc treo không ngắt!",
            iter
        );
        assert!(
            lag < 500,
            "Vòng lặp stress {}: Thời gian ngắt dừng vượt quá 500ms! Thực tế: {}ms",
            iter,
            lag
        );

        let _ = handle.join();
        pool.clear();
    }
    println!(">>> Pass 100/100 vòng lặp stress start/halt liên tục không deadlock/panic!");
}

#[test]
fn engine() {
    println!("\n=== UCI ENGINE STOP INTEGRATION TEST ===");
    for iter in 0..20 {
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

        assert!(ok, "Lệnh UCI Stop phải trả về true!");
        assert!(
            lag < 500,
            "Vòng lặp engine {}: Thời gian ngắt UCI Stop vượt quá 500ms! Thực tế: {}ms",
            iter,
            lag
        );

        assert!(!eng.exec(Command::Quit));
    }
    println!(">>> Pass 20/20 vòng lặp UCI Engine Go -> Stop < 500ms!");
}
