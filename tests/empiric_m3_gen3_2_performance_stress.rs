// ============================================================================
// EMPIRICAL PERFORMANCE & MEMORY STRESS HARNESS - MILESTONE 3 GEN3_2
// ============================================================================
// Author: m3_challenger_gen3_2
// Purpose: Empirical verification of performance claims and memory layout:
// 1. Probe timing benchmark of Book::probe (0ms / sub-microsecond O(1) probe)
// 2. Memory alignment, size, stack frame footprint of Stack & Pv at line 333
// 3. Execution verification of opening book and endgame knowledge structures
// Single-word identifiers rule strictly enforced for Rust code.
// ============================================================================

use xiangrust::board::{Parser, Position};
use xiangrust::book::opening::ENTRIES;
use xiangrust::book::Book;
use xiangrust::search::stack::Stack;
use xiangrust::search::pv::Pv;
use std::time::Instant;

#[test]
fn probe() {
    let pos = Parser::parse(Parser::DEFAULT);
    let count = 1_000_000usize;
    let start = Instant::now();
    let mut hits = 0usize;

    let mut i = 0usize;
    while i < count {
        let probe = Book::probe(&pos);
        if probe.is_some() {
            hits += 1;
        }
        i += 1;
    }

    let elapsed = start.elapsed();
    let nanos = elapsed.as_nanos() as f64 / count as f64;
    println!("Executed {} probes in {:?} (avg {:.2} ns/probe)", count, elapsed, nanos);

    assert_eq!(hits, count, "All probes on default position must hit!");
    assert!(nanos < 1000.0, "Average probe latency must be sub-microsecond (< 1us / 0ms)!");
}

#[test]
fn layout() {
    let size_pv = std::mem::size_of::<Pv>();
    let align_pv = std::mem::align_of::<Pv>();
    let size_stack = std::mem::size_of::<Stack>();
    let align_stack = std::mem::align_of::<Stack>();
    let size_total = size_stack * 128;

    println!("Pv struct size: {} bytes, align: {} bytes", size_pv, align_pv);
    println!("Stack struct size: {} bytes, align: {} bytes", size_stack, align_stack);
    println!("128 Stack array total size: {} bytes ({:.2} KB)", size_total, size_total as f64 / 1024.0);

    assert_eq!(align_pv, 64, "Pv must be aligned to 64 bytes for SIMD L1 cache line");
    assert_eq!(align_stack, 64, "Stack must be aligned to 64 bytes for SIMD L1 cache line");
    assert!(size_total > 40000, "128 Stack frames total size should be ~49-57 KB, requiring heap allocation");
}

#[test]
fn misses() {
    let count = 100_000usize;
    let start = Instant::now();
    let mut _misses = 0usize;

    let mut i = 0usize;
    while i < count {
        let mut pos = Position::empty();
        pos.hash = 0xDEADBEEF12345678u64.wrapping_add(i as u64);
        let probe = Book::probe(&pos);
        if probe.is_none() {
            _misses += 1;
        }
        i += 1;
    }

    let elapsed = start.elapsed();
    let nanos = elapsed.as_nanos() as f64 / count as f64;
    println!("Executed {} miss probes in {:?} (avg {:.2} ns/probe)", count, elapsed, nanos);

    assert_eq!(_misses, count, "All random hashes must miss!");
    assert!(nanos < 1000.0, "Miss probe latency must also be sub-microsecond (< 1us / 0ms)!");
}

#[test]
fn entries() {
    assert_eq!(std::mem::align_of::<Book>(), 64);
    assert_eq!(std::mem::size_of::<Book>(), 64);
    assert!(ENTRIES.len() >= 1000, "Book entries must contain at least 1,000 opening records");
}
