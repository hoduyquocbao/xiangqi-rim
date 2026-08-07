// Empirical Performance & Stress Verification for Milestone 4 Final Audit.
// Author: m4_challenger_gen3_2

use xiangrust::board::Parser;
use xiangrust::book::endgame::Endgame;
use xiangrust::book::opening::Book;
use xiangrust::selfplay::engine::{Config, Runner};
use std::time::Instant;

#[test]
fn probe() {
    let pos = Parser::parse(Parser::DEFAULT);
    let total = 1_000_000usize;
    let start = Instant::now();
    let mut hits = 0usize;

    let mut i = 0usize;
    while i < total {
        if Book::probe(&pos).is_some() {
            hits += 1;
        }
        i += 1;
    }

    let elapsed = start.elapsed();
    let nanos = elapsed.as_nanos() as f64 / total as f64;
    println!("Executed {} probes in {:?} (avg {:.2} ns/probe)", total, elapsed, nanos);

    assert_eq!(hits, total, "1M probes must all hit");
    assert!(nanos < 1000.0, "Average latency must be under 1000ns (0ms)");
}

#[test]
fn endgame() {
    let pos = Parser::parse("4k4/9/9/9/9/9/9/9/9/4K4 w - - 0 1");
    let total = 1_000_000usize;
    let start = Instant::now();
    let mut draws = 0usize;

    let mut i = 0usize;
    while i < total {
        if Endgame::eval(&pos) == Some(0) {
            draws += 1;
        }
        i += 1;
    }

    let elapsed = start.elapsed();
    let nanos = elapsed.as_nanos() as f64 / total as f64;
    println!("Executed {} endgame evals in {:?} (avg {:.2} ns/eval)", total, elapsed, nanos);

    assert_eq!(draws, total, "1M endgame evals must all evaluate to draw");
    assert!(nanos < 1000.0, "Average latency must be under 1000ns");
}

#[test]
fn runner() {
    let config = Config::new(2, 50, 20);
    let start = Instant::now();
    let result = Runner::play(&config);
    let elapsed = start.elapsed();

    println!("Self-play match finished in {:?}, moves: {}, outcome: {:?}", elapsed, result.moves.len(), result.outcome);
    assert!(result.moves.len() <= 20, "Moves must not exceed limit");
}
