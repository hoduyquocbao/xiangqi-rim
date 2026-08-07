// ============================================================================
// EMPIRICAL M3 GATE VERIFICATION TEST HARNESS
// ============================================================================
// Verification harness written by m3_challenger_gen3_1 to empirically test:
// 1. `Book::probe(&pos)` for initial position `board::Parser::DEFAULT` -> returns valid `Some(Move)`.
// 2. `Endgame::eval(&pos)` for various theoretical endgame positions (WIN, DRAW, LOSS, None).
// 3. Alignments and data structure sizes (`Entry`, `Book`, `Count`, `Rule`, `Endgame`).
// 4. Sorting of `BOOK_ENTRIES` and O(1) Zobrist lookup behavior.
// ============================================================================

use std::time::Instant;
use xiangrust::board::{Parser, Position};
use xiangrust::book::endgame::{Count, Endgame, Rule, DRAW, LOSS, WIN};
use xiangrust::book::opening::{Book, Entry, ENTRIES};
use xiangrust::uci::format::Format;

#[test]
fn test_opening_book_probe_default_position() {
    let pos = Parser::parse(Parser::DEFAULT);
    
    let start = Instant::now();
    let probed = Book::probe(&pos);
    let elapsed = start.elapsed();
    
    println!("[EMPIRICAL] Book::probe(DEFAULT) time: {:?}", elapsed);
    assert!(probed.is_some(), "Book::probe must return Some(Move) for Parser::DEFAULT!");
    
    let mv = probed.unwrap();
    let uci_str = Format::encode(mv);
    println!("[EMPIRICAL] Book::probe(DEFAULT) returned move: {} (raw: {:#06X})", uci_str, mv.raw());
    
    // Check if move is 0x1316 (Pháo 2 bình 5: sq 19 -> 22)
    assert_eq!(mv.raw(), 0x1316, "Default move should be Pháo 2 Bình 5 (0x1316 / c2c5)!");
}

#[test]
fn test_opening_book_count_and_sorted() {
    let book = Book::default();
    assert!(book.count >= 1000, "Book::count must be >= 1000, actual: {}", book.count);
    assert_eq!(ENTRIES.len(), 1024);
    
    // Verify strict ascending order of Zobrist hashes in ENTRIES
    for i in 0..(ENTRIES.len() - 1) {
        assert!(
            ENTRIES[i].hash < ENTRIES[i + 1].hash,
            "ENTRIES must be strictly sorted by hash! Violation at index {}: {:#X} >= {:#X}",
            i, ENTRIES[i].hash, ENTRIES[i + 1].hash
        );
    }
}

#[test]
fn test_opening_book_binary_search_probe_all_entries() {
    let book = Book::default();
    
    for (idx, entry) in ENTRIES.iter().enumerate() {
        let mut pos = Position::empty();
        pos.hash = entry.hash;
        
        let found = book.find(entry.hash);
        assert!(found.is_some(), "find failed for entry #{} with hash {:#X}", idx, entry.hash);
        assert_eq!(found.unwrap().raw(), entry.mv);
        
        let probed = Book::find_hash(&ENTRIES, pos.hash);
        assert_eq!(probed, Some(found.unwrap()));
    }
}

#[test]
fn test_endgame_eval_theoretical_positions() {
    // 1. Bare Kings -> DRAW (0)
    let pos_bare = Parser::parse("4k4/9/9/9/9/9/9/9/9/4K4 w - - 0 1");
    assert_eq!(Endgame::eval(&pos_bare), Some(DRAW), "Bare kings must evaluate to DRAW");
    
    // 2. Single Knight vs Single Advisor -> WIN (+15000) for Red, LOSS (-15000) for Black
    let pos_knight_red = Parser::parse("4k1a2/9/9/9/9/9/9/4N4/9/4K4 w - - 0 1");
    assert_eq!(Endgame::eval(&pos_knight_red), Some(WIN), "Knight vs Advisor (Red turn) must evaluate to WIN");
    let pos_knight_black = Parser::parse("4k1a2/9/9/9/9/9/9/4N4/9/4K4 b - - 0 1");
    assert_eq!(Endgame::eval(&pos_knight_black), Some(LOSS), "Knight vs Advisor (Black turn) must evaluate to LOSS");

    // 3. Single Cannon vs Single Advisor -> DRAW (0)
    let pos_cannon = Parser::parse("4k1a2/9/9/9/9/9/9/9/4C4/4K4 w - - 0 1");
    assert_eq!(Endgame::eval(&pos_cannon), Some(DRAW), "Single Cannon vs Advisor must evaluate to DRAW");

    // 4. Rook + Cannon vs Single Rook -> WIN (+15000) for Red, LOSS (-15000) for Black
    let pos_rc_red = Parser::parse("3k5/4r4/9/9/9/9/9/9/4C4/3K1R3 w - - 0 1");
    assert_eq!(Endgame::eval(&pos_rc_red), Some(WIN), "Rook+Cannon vs Rook (Red turn) must evaluate to WIN");
    let pos_rc_black = Parser::parse("3k5/4r4/9/9/9/9/9/9/4C4/3K1R3 b - - 0 1");
    assert_eq!(Endgame::eval(&pos_rc_black), Some(LOSS), "Rook+Cannon vs Rook (Black turn) must evaluate to LOSS");

    // 5. Double Cannons vs Incomplete Defense -> WIN (+15000)
    let pos_2c = Parser::parse("4k1a2/9/9/9/9/9/9/9/4C1C2/4K4 w - - 0 1");
    assert_eq!(Endgame::eval(&pos_2c), Some(WIN), "Double Cannons vs Incomplete Defense must evaluate to WIN");

    // 6. Single Rook vs Incomplete Defense -> WIN (+15000)
    let pos_1r = Parser::parse("4k1a2/9/9/9/9/9/9/9/4R4/4K4 w - - 0 1");
    assert_eq!(Endgame::eval(&pos_1r), Some(WIN), "Single Rook vs Incomplete Defense must evaluate to WIN");

    // 7. Single Knight vs Single Bishop -> DRAW (0)
    let pos_kb = Parser::parse("4k1b2/9/9/9/9/9/9/4N4/9/4K4 w - - 0 1");
    assert_eq!(Endgame::eval(&pos_kb), Some(DRAW), "Single Knight vs Bishop must evaluate to DRAW");

    // 8. Double Knights vs Defense -> WIN (+15000)
    let pos_2n = Parser::parse("4ka3/9/4b4/9/9/9/9/4N1N2/9/4K4 w - - 0 1");
    assert_eq!(Endgame::eval(&pos_2n), Some(WIN), "Double Knights vs Defense must evaluate to WIN");

    // 9. Cannon + River Pawn vs Incomplete Defense -> WIN (+15000)
    let pos_cp = Parser::parse("4k1a2/9/9/9/4P4/9/9/9/4C4/4K4 w - - 0 1");
    assert_eq!(Endgame::eval(&pos_cp), Some(WIN), "Cannon + River Pawn vs Incomplete Defense must evaluate to WIN");

    // 10. Initial Board (full game) -> None (not a theoretical endgame position)
    let pos_initial = Parser::parse(Parser::DEFAULT);
    assert_eq!(Endgame::eval(&pos_initial), None, "Initial full position should return None from Endgame::eval");
}

#[test]
fn test_memory_alignments_and_sizes() {
    assert_eq!(std::mem::align_of::<Entry>(), 16);
    assert_eq!(std::mem::size_of::<Entry>(), 32);

    assert_eq!(std::mem::align_of::<Book>(), 64);
    assert_eq!(std::mem::size_of::<Book>(), 64);

    assert_eq!(std::mem::align_of::<Count>(), 16);
    assert_eq!(std::mem::size_of::<Count>(), 16);

    assert_eq!(std::mem::align_of::<Rule>(), 16);
    assert_eq!(std::mem::size_of::<Rule>(), 32);

    assert_eq!(std::mem::align_of::<Endgame>(), 64);
    assert_eq!(std::mem::size_of::<Endgame>(), 64);
}
