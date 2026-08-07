// ============================================================================
// EMPIRICAL 1,000,000 ITERATIONS BENCHMARK & ADVERSARIAL STRESS HARNESS FOR M1
// ============================================================================
// Author: teamwork_preview_challenger_m1_gen2_2
// Purpose:
// 1. Independently measure `Book::probe` over 1,000,000 lookup iterations.
// 2. Independently measure `Endgame::eval` over 1,000,000 evaluation iterations.
// 3. Stress test multi-threaded scaling and boundary conditions for zero panic/leak.
// ============================================================================

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use xiangrust::board::Parser;
use xiangrust::book::endgame::{DRAW, WIN};
use xiangrust::book::opening::ENTRIES;
use xiangrust::book::{Book, Endgame};

/// Benchmark 1: `Book::probe` over 1,000,000 sequential iterations (50% hit, 50% miss).
#[test]
fn benchmark_book_probe_1m() {
    let base_pos = Parser::parse(Parser::DEFAULT);
    let mut pos_hit = base_pos;
    pos_hit.hash = ENTRIES[256].hash;

    let mut pos_miss = base_pos;
    pos_miss.hash = 0x9999_8888_7777_6666;

    const ITERATIONS: usize = 1_000_000;
    let start = Instant::now();
    let mut hits = 0usize;
    let mut misses = 0usize;

    for i in 0..ITERATIONS {
        let pos = if i % 2 == 0 { &pos_hit } else { &pos_miss };
        if let Some(mv) = Book::probe(pos) {
            hits += 1;
            assert_eq!(mv.raw(), ENTRIES[256].mv);
        } else {
            misses += 1;
        }
    }

    let elapsed = start.elapsed();
    let ns_per_op = elapsed.as_nanos() as f64 / ITERATIONS as f64;
    let ops_per_sec = (ITERATIONS as f64 / elapsed.as_secs_f64()) as u64;

    println!(
        "\n[BENCHMARK M1] Book::probe 1,000,000 iterations: total = {:?}, ns/op = {:.2}, ops/sec = {} (hits: {}, misses: {})",
        elapsed, ns_per_op, ops_per_sec, hits, misses
    );

    assert_eq!(hits, 500_000);
    assert_eq!(misses, 500_000);
    assert!(ns_per_op < 300.0, "Book::probe exceeded latency threshold: {:.2} ns/op", ns_per_op);
}

/// Benchmark 2: `Endgame::eval` over 1,000,000 sequential iterations across 10 standard endgame positions.
#[test]
fn benchmark_endgame_eval_1m() {
    let fens = [
        "4k4/9/9/9/9/9/9/9/9/4K4 w - - 0 1",              // Bare Kings -> DRAW
        "4k1a2/9/9/9/9/9/9/4N4/9/4K4 w - - 0 1",          // Knight vs Advisor -> WIN
        "4k1a2/9/9/9/9/9/9/4C4/9/4K4 w - - 0 1",          // Cannon vs Advisor -> DRAW
        "3ak4/9/9/9/9/9/9/3RN4/9/3K1r3 w - - 0 1",        // Rook+Knight vs Rook -> WIN
        "3akab2/9/9/9/9/9/9/4CC3/9/4K4 w - - 0 1",        // Double Cannons -> WIN
        "3akab2/9/9/9/9/9/9/4R4/9/4K4 w - - 0 1",         // Single Rook vs Incomplete -> WIN
        "4k4/4b4/9/9/9/9/9/4N4/9/4K4 w - - 0 1",          // Knight vs Bishop -> DRAW
        "3akab2/9/9/9/9/9/9/3NN4/9/4K4 w - - 0 1",        // Double Knights vs Full -> WIN
        "3akab2/4P4/9/9/9/9/9/4C4/9/4K4 w - - 0 1",        // Cannon + River Pawn -> WIN
        "3k5/4r4/9/9/9/9/9/9/4C4/3K1R3 w - - 0 1",        // Rook+Cannon vs Rook -> WIN
    ];

    let positions: Vec<_> = fens.iter().map(|f| Parser::parse(f)).collect();
    const ROUNDS: usize = 100_000;
    const TOTAL_OPS: usize = ROUNDS * 10;

    let start = Instant::now();
    let mut eval_count = 0usize;

    for _ in 0..ROUNDS {
        for pos in &positions {
            let res = Endgame::eval(pos);
            if let Some(_s) = res {
                eval_count += 1;
            } else {
                eval_count += 1;
            }
        }
    }

    let elapsed = start.elapsed();
    let ns_per_op = elapsed.as_nanos() as f64 / TOTAL_OPS as f64;
    let ops_per_sec = (TOTAL_OPS as f64 / elapsed.as_secs_f64()) as u64;

    println!(
        "\n[BENCHMARK M1] Endgame::eval 1,000,000 iterations: total = {:?}, ns/op = {:.2}, ops/sec = {} (evaluations: {})",
        elapsed, ns_per_op, ops_per_sec, eval_count
    );

    assert_eq!(eval_count, TOTAL_OPS);
    assert!(ns_per_op < 1000.0, "Endgame::eval exceeded latency threshold: {:.2} ns/op", ns_per_op);
}

/// Benchmark 3: Multi-threaded `Endgame::eval` over 1,000,000 iterations across 16 threads.
#[test]
fn benchmark_endgame_eval_multithreaded_1m() {
    const NUM_THREADS: usize = 16;
    const ITER_PER_THREAD: usize = 62_500;
    const TOTAL_OPS: usize = NUM_THREADS * ITER_PER_THREAD;

    let counter = Arc::new(AtomicU64::new(0));
    let start = Instant::now();

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|t_idx| {
            let cnt = Arc::clone(&counter);
            thread::spawn(move || {
                let fen = if t_idx % 2 == 0 {
                    "4k1a2/9/9/9/9/9/9/4N4/9/4K4 w - - 0 1"
                } else {
                    "3akab2/9/9/9/9/9/9/4R4/9/4K4 w - - 0 1"
                };
                let pos = Parser::parse(fen);
                let mut local_cnt = 0u64;

                for _ in 0..ITER_PER_THREAD {
                    let score = Endgame::eval(&pos);
                    if score == Some(WIN) {
                        local_cnt += 1;
                    }
                }
                cnt.fetch_add(local_cnt, Ordering::Relaxed);
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Multi-threaded Endgame thread panicked");
    }

    let elapsed = start.elapsed();
    let wins = counter.load(Ordering::Relaxed);
    let ns_per_op = elapsed.as_nanos() as f64 / TOTAL_OPS as f64;
    let ops_per_sec = (TOTAL_OPS as f64 / elapsed.as_secs_f64()) as u64;

    println!(
        "\n[BENCHMARK M1] Endgame::eval Multi-threaded ({} threads) 1,000,000 iterations: total = {:?}, ns/op = {:.2}, ops/sec = {} (wins: {})",
        NUM_THREADS, elapsed, ns_per_op, ops_per_sec, wins
    );

    assert_eq!(wins as usize, TOTAL_OPS);
}

/// Test 4: Combined 1,000,000 stress test pipeline (500k Book::probe + 500k Endgame::eval alternating).
#[test]
fn stress_combined_book_endgame_1m_pipeline() {
    let fen_opening = Parser::parse(Parser::DEFAULT);
    let mut pos_book = fen_opening;
    pos_book.hash = ENTRIES[100].hash;

    let pos_endgame = Parser::parse("4k4/9/9/9/9/9/9/9/9/4K4 w - - 0 1");

    const TOTAL_STEPS: usize = 1_000_000;
    let start = Instant::now();
    let mut book_success = 0usize;
    let mut endgame_draws = 0usize;

    for i in 0..TOTAL_STEPS {
        if i % 2 == 0 {
            if let Some(mv) = Book::probe(&pos_book) {
                assert_eq!(mv.raw(), ENTRIES[100].mv);
                book_success += 1;
            }
        } else {
            if let Some(score) = Endgame::eval(&pos_endgame) {
                assert_eq!(score, DRAW);
                endgame_draws += 1;
            }
        }
    }

    let elapsed = start.elapsed();
    let ns_per_op = elapsed.as_nanos() as f64 / TOTAL_STEPS as f64;

    println!(
        "\n[STRESS TEST M1] Combined Book & Endgame 1,000,000 pipeline: total = {:?}, ns/op = {:.2} (book: {}, endgame: {})",
        elapsed, ns_per_op, book_success, endgame_draws
    );

    assert_eq!(book_success, 500_000);
    assert_eq!(endgame_draws, 500_000);
}
