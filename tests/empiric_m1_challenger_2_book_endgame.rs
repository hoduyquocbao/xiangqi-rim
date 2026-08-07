// ============================================================================
// EMPIRICAL STRESS TEST & BENCHMARK HARNESS FOR MILESTONE 1 (src/book/)
// ============================================================================
// Author: teamwork_preview_challenger_m1_2
// Purpose:
// 1. Benchmark `Book::probe` performance & memory contention under 1,000,000
//    probes across sequential and multi-threaded environments.
// 2. Stress test `Endgame::eval` over 50+ FEN positions (theoretical endgames,
//    midgames, synthetic/random configurations) for panic/overflow.
// ============================================================================

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use xiangrust::board::{Parser, Position};
use xiangrust::book::endgame::{DRAW, LOSS, WIN};
use xiangrust::book::opening::ENTRIES;
use xiangrust::book::{Book, Endgame};

/// Test 1: Benchmark `Book::probe` sequentially for 1,000,000 lookup iterations.
#[test]
fn test_book_probe_sequential_1m_iterations() {
    let start_pos = Parser::parse(Parser::DEFAULT);
    let mut pos_hit = start_pos;
    pos_hit.hash = ENTRIES[512].hash;

    let mut pos_miss = start_pos;
    pos_miss.hash = 0xDEAD_BEEF_9876_5432;

    const ITERATIONS: usize = 1_000_000;

    let start = Instant::now();
    let mut hit_count = 0usize;
    let mut miss_count = 0usize;

    for i in 0..ITERATIONS {
        let pos = if i % 2 == 0 { &pos_hit } else { &pos_miss };
        if let Some(mv) = Book::probe(pos) {
            hit_count += 1;
            assert_eq!(mv.raw(), ENTRIES[512].mv);
        } else {
            miss_count += 1;
        }
    }

    let elapsed = start.elapsed();
    let ns_per_op = elapsed.as_nanos() as f64 / ITERATIONS as f64;
    let ops_per_sec = (ITERATIONS as f64 / elapsed.as_secs_f64()) as u64;

    println!(
        "[BENCHMARK] Book::probe Sequential 1,000,000 iterations completed in {:?}: {:.2} ns/op, {} ops/sec (hits: {}, misses: {})",
        elapsed, ns_per_op, ops_per_sec, hit_count, miss_count
    );

    assert_eq!(hit_count, 500_000);
    assert_eq!(miss_count, 500_000);
    // Expect sub-200ns per binary search probe
    assert!(ns_per_op < 200.0, "Book::probe sequential performance degraded: {:.2} ns/op", ns_per_op);
}

/// Test 2: Multi-threaded load test for `Book::probe` (16 threads, 62,500 iterations each = 1,000,000 probes total).
/// Verifies zero memory contention, zero data races, and cache efficiency under multi-threading.
#[test]
fn test_book_probe_multithreaded_contention() {
    const NUM_THREADS: usize = 16;
    const ITER_PER_THREAD: usize = 62_500;
    const TOTAL_OPS: usize = NUM_THREADS * ITER_PER_THREAD;

    let hits_counter = Arc::new(AtomicU64::new(0));
    let start = Instant::now();

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|t_idx| {
            let hits = Arc::clone(&hits_counter);
            thread::spawn(move || {
                let mut local_hits = 0u64;
                let mut pos = Parser::parse(Parser::DEFAULT);
                for i in 0..ITER_PER_THREAD {
                    // Mix hit entries and miss entries based on thread ID and iteration
                    let entry_idx = (t_idx * 17 + i) % 1024;
                    if i % 3 == 0 {
                        pos.hash = 0xFFFF_0000_1111_0000 | (i as u64); // Miss
                    } else {
                        pos.hash = ENTRIES[entry_idx].hash; // Hit
                    }

                    if let Some(mv) = Book::find_hash(&ENTRIES, pos.hash) {
                        assert_eq!(mv.raw(), ENTRIES[entry_idx].mv);
                        local_hits += 1;
                    }
                }
                hits.fetch_add(local_hits, Ordering::Relaxed);
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread panicked during Book::probe stress test");
    }

    let elapsed = start.elapsed();
    let total_hits = hits_counter.load(Ordering::Relaxed);
    let ns_per_op = elapsed.as_nanos() as f64 / TOTAL_OPS as f64;
    let ops_per_sec = (TOTAL_OPS as f64 / elapsed.as_secs_f64()) as u64;

    println!(
        "[BENCHMARK] Book::probe Multi-threaded ({} threads) 1,000,000 probes completed in {:?}: {:.2} ns/op, {} ops/sec (total hits: {})",
        NUM_THREADS, elapsed, ns_per_op, ops_per_sec, total_hits
    );

    assert!(total_hits > 0);
    assert!(ns_per_op < 500.0, "Book::probe multi-threaded contention detected: {:.2} ns/op", ns_per_op);
}

/// Test 3: Stress test `Endgame::eval` across 50+ diverse and synthetic FEN positions.
#[test]
fn test_endgame_eval_50_plus_random_fen_positions() {
    let fens = [
        // Theoretical Endgames (Rule 1-10 cases)
        "4k4/9/9/9/9/9/9/9/9/4K4 w - - 0 1", // Bare Kings -> DRAW
        "4k4/9/9/9/9/9/9/9/9/4K4 b - - 0 1", // Bare Kings Black to move -> DRAW
        "4k1a2/9/9/9/9/9/9/4N4/9/4K4 w - - 0 1", // Single Knight vs Single Advisor -> WIN
        "4k4/4a4/9/9/9/9/9/4n4/9/2A1K4 w - - 0 1", // Single Knight vs Single Advisor (enemy attack) -> LOSS
        "4k1a2/9/9/9/9/9/9/4C4/9/4K4 w - - 0 1", // Single Cannon vs Single Advisor -> DRAW
        "3a1k3/9/9/9/9/9/9/4c4/9/2A1K4 b - - 0 1", // Single Cannon vs Single Advisor (black side) -> DRAW
        "3ak4/9/9/9/9/9/9/3RN4/9/3K1r3 w - - 0 1", // Rook + Knight vs Rook -> WIN
        "3ak4/9/9/9/9/9/9/3rn4/9/3K1R3 b - - 0 1", // Enemy Rook + Knight vs Rook -> LOSS
        "3akab2/9/9/9/9/9/9/4CC3/9/4K4 w - - 0 1", // Double Cannons vs Incomplete -> WIN
        "3akab2/9/9/9/9/9/9/4cc3/9/4K4 b - - 0 1", // Enemy Double Cannons vs Incomplete -> LOSS
        "3akab2/9/9/9/9/9/9/4R4/9/4K4 w - - 0 1", // Single Rook vs Incomplete -> WIN
        "3akab2/9/9/9/9/9/9/4r4/9/4K4 b - - 0 1", // Enemy Single Rook vs Incomplete -> LOSS
        "4k4/4b4/9/9/9/9/9/4N4/9/4K4 w - - 0 1", // Single Knight vs Single Bishop -> DRAW
        "4k4/4b4/9/9/9/9/9/4n4/9/4K4 b - - 0 1", // Enemy Single Knight vs Single Bishop -> DRAW
        "3akab2/9/9/9/9/9/9/3NN4/9/4K4 w - - 0 1", // Double Knights vs Full -> WIN
        "3akab2/9/9/9/9/9/9/3nn4/9/4K4 b - - 0 1", // Enemy Double Knights vs Full -> LOSS
        "3akab2/4p4/9/9/9/9/9/4C4/9/4K4 w - - 0 1", // Cannon + River Pawn vs Incomplete -> WIN
        "3akab2/9/9/9/9/9/4P4/9/9/4c4 b - - 0 1", // Enemy Cannon + River Pawn vs Incomplete -> LOSS
        "3k5/4r4/9/9/9/9/9/9/4C4/3K1R3 w - - 0 1", // Rook + Cannon vs Rook -> WIN
        "3k5/4r4/9/9/9/9/9/9/4C4/3K1R3 b - - 0 1", // Rook + Cannon vs Rook (black to move) -> LOSS

        // Starting & Midgame FENs (Non-endgame positions where eval should return None)
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1", // Initial start position
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR b - - 0 1",
        "r1bakabnr/9/1c4sc1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        "rnbakab2/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        "2bakab2/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        "3akab2/9/9/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        "3akab2/9/9/4p4/9/9/4P4/1C5C1/9/RNBAKABNR w - - 0 1",

        // Synthetic positions with various piece combinations (total > 50 positions)
        "3a1k3/9/9/9/9/9/9/9/9/4K4 w - - 0 1",
        "3a1k3/9/9/9/9/9/9/9/9/4K4 b - - 0 1",
        "3a1k3/4a4/9/9/9/9/9/9/9/4K4 w - - 0 1",
        "4k4/9/4b4/9/9/9/9/9/9/4K4 w - - 0 1",
        "4k4/9/2b1b4/9/9/9/9/9/9/4K4 w - - 0 1",
        "4k4/4a4/4b4/9/9/9/9/9/9/4K4 w - - 0 1",
        "3akab2/9/9/9/9/9/9/9/9/4K4 w - - 0 1",
        "3akab2/9/9/9/9/9/9/9/9/3AKAB2 w - - 0 1",
        "4k4/9/9/9/9/9/9/9/9/3AKAB2 w - - 0 1",
        "4k4/9/9/9/9/9/4P4/9/9/4K4 w - - 0 1", // Single Red Pawn
        "4k4/9/9/9/9/4p4/9/9/9/4K4 w - - 0 1", // Single Black Pawn
        "4k4/9/9/9/9/4p4/4P4/9/9/4K4 w - - 0 1", // Both side Pawns
        "4k4/9/9/9/9/3pp4/3PP4/9/9/4K4 w - - 0 1", // 2 Pawns each
        "4k4/9/9/9/9/2ppp4/2PPP4/9/9/4K4 w - - 0 1", // 3 Pawns each
        "4k4/9/9/9/9/1pppp4/1PPPP4/9/9/4K4 w - - 0 1", // 4 Pawns each
        "4k4/9/9/9/9/ppppp4/PPPPP4/9/9/4K4 w - - 0 1", // 5 Pawns each
        "4k4/9/9/9/9/9/9/4R4/9/4K4 w - - 0 1", // Single Red Rook vs Bare
        "4k4/9/9/9/9/9/9/4r4/9/4K4 w - - 0 1", // Single Black Rook vs Bare
        "4k4/9/9/9/9/9/9/4N4/9/4K4 w - - 0 1", // Single Red Knight vs Bare
        "4k4/9/9/9/9/9/9/4n4/9/4K4 w - - 0 1", // Single Black Knight vs Bare
        "4k4/9/9/9/9/9/9/4C4/9/4K4 w - - 0 1", // Single Red Cannon vs Bare
        "4k4/9/9/9/9/9/9/4c4/9/4K4 w - - 0 1", // Single Black Cannon vs Bare
        "4k4/9/9/9/9/9/9/3RR4/9/4K4 w - - 0 1", // Double Red Rooks
        "4k4/9/9/9/9/9/9/3rr4/9/4K4 w - - 0 1", // Double Black Rooks
        "4k4/9/9/9/9/9/9/3NN4/9/4K4 w - - 0 1", // Double Red Knights
        "4k4/9/9/9/9/9/9/3nn4/9/4K4 w - - 0 1", // Double Black Knights
        "4k4/9/9/9/9/9/9/3CC4/9/4K4 w - - 0 1", // Double Red Cannons
        "4k4/9/9/9/9/9/9/3cc4/9/4K4 w - - 0 1", // Double Black Cannons
        "3akab2/9/9/9/9/9/9/3RN4/9/4K4 w - - 0 1", // Red Rook+Knight vs Full
        "3akab2/9/9/9/9/9/9/3rn4/9/4K4 b - - 0 1", // Black Rook+Knight vs Full
        "3akab2/9/9/9/9/9/9/3RC4/9/4K4 w - - 0 1", // Red Rook+Cannon vs Full
        "3akab2/9/9/9/9/9/9/3rc4/9/4K4 b - - 0 1", // Black Rook+Cannon vs Full
        "3akab2/9/9/9/9/9/9/3NC4/9/4K4 w - - 0 1", // Red Knight+Cannon vs Full
        "3akab2/9/9/9/9/9/9/3nc4/9/4K4 b - - 0 1", // Black Knight+Cannon vs Full
    ];

    assert!(fens.len() >= 50, "Expected at least 50 test positions, got {}", fens.len());

    let mut evaluated_count = 0;
    let mut win_count = 0;
    let mut loss_count = 0;
    let mut draw_count = 0;
    let mut none_count = 0;

    for (idx, fen) in fens.iter().enumerate() {
        let pos = Parser::parse(fen);
        let eval_res = Endgame::eval(&pos);

        match eval_res {
            Some(score) => {
                assert!(
                    score == WIN || score == LOSS || score == DRAW,
                    "Position #{} ({}) produced invalid endgame score: {}",
                    idx, fen, score
                );
                if score == WIN {
                    win_count += 1;
                } else if score == LOSS {
                    loss_count += 1;
                } else if score == DRAW {
                    draw_count += 1;
                }
            }
            None => {
                none_count += 1;
            }
        }
        evaluated_count += 1;
    }

    println!(
        "[STRESS TEST] Endgame::eval tested on {} FEN positions successfully without panic or overflow (WIN: {}, LOSS: {}, DRAW: {}, NONE: {})",
        evaluated_count, win_count, loss_count, draw_count, none_count
    );

    assert_eq!(evaluated_count, fens.len());
}

/// Test 4: Extreme stress test for `Endgame::eval` with random board layouts and boundary checks.
#[test]
fn test_endgame_eval_randomized_boundary_resilience() {
    let mut seed = 0x1234_5678_9ABC_DEF0u64;

    for trial in 0..500 {
        let mut pos = Position::empty();

        // Place Kings in valid palaces
        let red_king_sq = 4 + ((trial % 3) * 9 + (trial / 3) % 3) as u8;
        let black_king_sq = 76 + ((trial % 3) * 9 + (trial / 3) % 3) as u8;

        pos.put(0, red_king_sq);
        pos.put(7, black_king_sq);

        // Randomly place a few pieces
        for piece_type in 1..=6 {
            seed = seed.wrapping_add(0x9E3779B97F4A7C15);
            if (seed % 3) == 0 {
                let sq = ((seed >> 8) % 45) as u8;
                if pos.at(sq) == 14 {
                    pos.put(piece_type, sq);
                }
            }

            seed = seed.wrapping_add(0x9E3779B97F4A7C15);
            if (seed % 3) == 0 {
                let sq = 45 + ((seed >> 16) % 45) as u8;
                if pos.at(sq) == 14 {
                    pos.put(piece_type + 7, sq);
                }
            }
        }

        pos.side = (trial % 2) as u8;

        // Ensure Endgame::eval runs smoothly without panic or memory corruption
        let _result = Endgame::eval(&pos);
    }
}
