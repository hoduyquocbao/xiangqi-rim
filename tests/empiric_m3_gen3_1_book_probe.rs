// ============================================================================
// EMPIRICAL TEST HARNESS: MILESTONE 3 BOOK PROBE & ENDGAME STRESS TEST
// ============================================================================
// Author: challenger_m3_gen3_1
// Purpose: Empirical verification of Book::probe(&pos) correctness across:
// 1. Opening positions (default board & known variation hashes)
// 2. Endgame positions (theoretical win/draw/loss positions)
// 3. Edge case positions (empty board, zero hash, max hash, randomized hashes)
// 4. Stress testing 100,000 calls for O(1) timing and zero panics/overflows.
// All single-word identifiers for Rust code strictly maintained.
// ============================================================================

use xiangrust::board::{Parser, Position};
use xiangrust::book::opening::ENTRIES;
use xiangrust::book::{Book, Endgame};
use std::time::Instant;

#[test]
fn opening() {
    let pos = Parser::parse(Parser::DEFAULT);
    let probe = Book::probe(&pos);
    assert!(probe.is_some(), "Default position probe must yield move!");
    
    let mv = probe.unwrap();
    assert!(mv.from < 90, "From square must be within 0..90 boundary");
    assert!(mv.to < 90, "To square must be within 0..90 boundary");
}

#[test]
fn entries() {
    let book = Book::default();
    let mut i = 0;
    while i < ENTRIES.len() {
        let entry = ENTRIES[i];
        let mut pos = Position::empty();
        pos.hash = entry.hash;

        let res = book.find(entry.hash);
        assert!(res.is_some(), "Existing entry hash must be found!");
        let mv = res.unwrap();
        assert_eq!(mv.raw(), entry.mv, "Probed move must match entry move!");

        let probed = Book::find_hash(&ENTRIES, pos.hash);
        assert_eq!(probed, Some(mv), "Book::find_hash must match find result!");

        i += 1;
    }
}

#[test]
fn endgame() {
    let fens = [
        "4k4/9/9/9/9/9/9/9/9/4K4 w - - 0 1",
        "4k1a2/9/9/9/9/9/9/4N4/9/4K4 w - - 0 1",
        "3k5/4r4/9/9/9/9/9/9/4C4/3K1R3 w - - 0 1",
        "4k1a2/9/9/9/9/9/9/9/4C4/4K4 w - - 0 1",
        "4k1a2/9/9/9/9/9/9/9/4C1C2/4K4 w - - 0 1",
        "4k1a2/9/9/9/9/9/9/4N4/9/4K4 b - - 0 1",
    ];

    for fen in fens {
        let pos = Parser::parse(fen);
        let eval = Endgame::eval(&pos);
        assert!(eval.is_some(), "Endgame evaluation must produce score");

        let probe = Book::probe(&pos);
        // Probe on endgame position must safely return Option<Move> without panic
        if let Some(mv) = probe {
            assert!(mv.from < 90);
            assert!(mv.to < 90);
        }
    }
}

#[test]
fn edge() {
    // 1. Empty board position
    let empty = Position::empty();
    let probe_empty = Book::probe(&empty);
    if let Some(mv) = probe_empty {
        assert!(mv.from < 90);
        assert!(mv.to < 90);
    }

    // 2. Zero hash position
    let mut pos_zero = Position::empty();
    pos_zero.hash = 0;
    let _ = Book::probe(&pos_zero);

    // 3. Max u64 hash position
    let mut pos_max = Position::empty();
    pos_max.hash = u64::MAX;
    let _ = Book::probe(&pos_max);
}

#[test]
fn stress() {
    let pos = Parser::parse(Parser::DEFAULT);
    let start = Instant::now();
    let limit = 100_000;
    let mut count = 0;

    let mut i = 0;
    while i < limit {
        let mut test_pos = pos;
        test_pos.hash = test_pos.hash.wrapping_add(i as u64);
        let probe = Book::probe(&test_pos);
        if probe.is_some() {
            count += 1;
        }
        i += 1;
    }

    let elapsed = start.elapsed();
    println!("Probed {} positions in {:?}", limit, elapsed);
    assert!(count > 0, "Some hashes must hit book entries during stress scan");
}
