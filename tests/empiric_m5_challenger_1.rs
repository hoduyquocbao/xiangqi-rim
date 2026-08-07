// Empirical test harness for M5 Lazy SMP ThreadPool & UCI Halt Responsiveness
// Single-word English identifiers for tests, 100% Vietnamese comments.

use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use xiangrust::board::Parser;
use xiangrust::search::limit::Limits;
use xiangrust::thread::Pool;

#[test]
fn test_pool_multi_thread_scaling() {
    let pos = Parser::parse(Parser::DEFAULT);
    let counts = [1, 2, 4, 8, 16];

    for &size in &counts {
        let pool = Pool::new(size, 16);
        let mut limits = Limits::new();
        limits.depth = 4;

        let start = Instant::now();
        let runner = pool;
        let result = runner.go(&pos, &limits);
        let elapsed = start.elapsed();

        let nps = if result.time > 0 {
            (result.nodes * 1000) / result.time
        } else {
            result.nodes * 1000
        };

        println!(
            "[SCALING TEST] Threads: {:2} | Depth: {} | Nodes: {:8} | Time: {:4}ms | NPS: {:10}",
            size, result.depth, result.nodes, result.time, nps
        );

        assert!(result.best.valid(), "Best move must be valid!");
        assert!(result.nodes > 0, "Nodes count must be positive!");
        assert!(elapsed < Duration::from_secs(5), "Search depth 4 took too long!");
    }
}

#[test]
fn test_pool_signal_halt_unlinked_defect() {
    let pos = Parser::parse(Parser::DEFAULT);
    let pool = Pool::new(4, 16);
    let mut limits = Limits::new();
    limits.depth = 8;

    let signal = Arc::clone(&pool.signal);
    let (tx, rx) = mpsc::channel();

    let runner = pool;
    thread::spawn(move || {
        let res = runner.go(&pos, &limits);
        let _ = tx.send(res);
    });

    thread::sleep(Duration::from_millis(2));
    let halt_start = Instant::now();
    signal.halt();

    let status = rx.recv_timeout(Duration::from_millis(50));
    let halt_lag = halt_start.elapsed();

    println!(
        "[HALT DEFECT TEST] Halt signal sent. Lag: {:?} | Terminated: {}",
        halt_lag,
        status.is_ok()
    );

    // Đã khắc phục liên kết tín hiệu: signal.halt() ngắt dừng tìm kiếm thành công.
    let terminated = status.is_ok();
    assert!(
        terminated,
        "Phản ứng dừng phải thành công khi gửi tín hiệu halt!"
    );
}
