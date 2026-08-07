// Integration test harness for M5 Challenger 2
// Empirical test for UCI v2 async interrupt and Format coordinate conversion

use xiangrust::uci::command::Command;
use xiangrust::uci::engine::Engine;
use xiangrust::uci::format::Format;
use xiangrust::movegen::types::Move;
use std::time::{Duration, Instant};

#[test]
fn interrupt() {
    let mut engine = Engine::new();

    // Complex midgame FEN
    let fen = "rnbakabnr/9/1c4c1/p1p1p1p1p/9/9/P1P1P1P1P/1C4C1/9/RNBAKABNR w - - 0 1";
    engine.exec(Command::Position {
        fen: fen.to_string(),
        moves: vec![],
    });

    let go = Command::Go {
        depth: 64,
        nodes: 0,
        infinite: true,
        span: 0,
        red: 0,
        black: 0,
        gain: 0,
        extra: 0,
    };

    // Run engine search in background thread
    engine.exec(go);

    // Allow search to run for 50ms on complex position
    std::thread::sleep(Duration::from_millis(50));

    // Send Stop command and measure latency
    let start = Instant::now();
    engine.exec(Command::Stop);
    let elapsed = start.elapsed().as_micros();
    let millis = elapsed as f64 / 1000.0;

    println!("[EMPIRICAL REPORT] Interrupt latency: {} us ({:.3} ms)", elapsed, millis);
    assert!(millis < 500.0, "Interrupt latency MUST be < 500ms, actual: {:.3} ms", millis);
    assert!(engine.handle.is_none(), "Engine task handle MUST be joined and cleared");
}

#[test]
fn repeat() {
    let mut engine = Engine::new();
    let fen = "2baakr2/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/2BAAKR2 w - - 0 1";
    engine.exec(Command::Position {
        fen: fen.to_string(),
        moves: vec![],
    });

    let mut max = 0u128;
    let mut sum = 0u128;
    let runs = 50usize;

    for _ in 0..runs {
        let go = Command::Go {
            depth: 64,
            nodes: 0,
            infinite: true,
            span: 0,
            red: 0,
            black: 0,
            gain: 0,
            extra: 0,
        };

        engine.exec(go);
        std::thread::sleep(Duration::from_millis(50));

        let start = Instant::now();
        engine.exec(Command::Stop);
        let elapsed = start.elapsed().as_micros();

        sum += elapsed;
        if elapsed > max {
            max = elapsed;
        }
        assert!(elapsed < 500000, "Interrupt latency exceeding 500ms limit: {} us", elapsed);
    }

    let avg = sum as f64 / runs as f64 / 1000.0;
    let peak = max as f64 / 1000.0;
    println!("[EMPIRICAL REPORT] Repeat interrupt over 50 runs: Avg = {:.3} ms, Peak = {:.3} ms", avg, peak);
    assert!(peak < 500.0, "Peak interrupt latency MUST be < 500ms");
}

#[test]
fn format() {
    let mut count = 0usize;
    for from in 0..90u8 {
        for to in 0..90u8 {
            if from == to {
                continue;
            }
            let mv = Move::new(from, to);
            let text = Format::encode(mv);
            assert_eq!(text.len(), 4, "Encoded text length must be 4 characters");

            let decoded = Format::decode(&text);
            assert_eq!(decoded.from, from, "Decoded from square mismatch");
            assert_eq!(decoded.to, to, "Decoded to square mismatch");
            count += 1;
        }
    }
    assert_eq!(count, 8010, "Total valid move pairs on 9x10 board MUST be exactly 8,010");
    println!("[EMPIRICAL REPORT] Stress test Format encode/decode across all {} square pairs PASSED", count);
}

#[test]
fn bounds() {
    let invalid = vec![
        "", "a0", "z9z9", "a-1a0", "i99i99", "a10", "1234", "%%%%", "a1a"
    ];
    for text in invalid {
        let decoded = Format::decode(text);
        assert_eq!(decoded, Move::none(), "Invalid UCI string '{}' must decode to Move::none()", text);
    }

    // Bug detection: strings longer than 4 chars like "a1a1a"
    let len_check_bug = Format::decode("a1a1a");
    if len_check_bug != Move::none() {
        println!("[EMPIRICAL REPORT - BUG FINDING] Format::decode(\"a1a1a\") failed length check: returned Move {{ from: {}, to: {} }} instead of Move::none()", len_check_bug.from, len_check_bug.to);
    }
}

#[test]
fn throughput() {
    let iterations = 1000usize;
    let start = Instant::now();

    for _ in 0..iterations {
        for from in 0..90u8 {
            for to in 0..90u8 {
                if from == to {
                    continue;
                }
                let mv = Move::new(from, to);
                let text = Format::encode(mv);
                let decoded = Format::decode(&text);
                assert_eq!(decoded.from, from);
            }
        }
    }

    let elapsed = start.elapsed();
    let ops = (iterations * 8010) as f64;
    let secs = elapsed.as_secs_f64();
    let rate = ops / secs;
    let nanos = elapsed.as_nanos() as f64 / ops;

    println!(
        "[EMPIRICAL REPORT] Format Throughput: {:.0} ops/sec, {:.2} ns/op (Total time: {:.3}s for {:.0} ops)",
        rate, nanos, secs, ops
    );
}
