// File tests/adversarial_board.rs - Stress harness & empirical test suite for Board Module

use xiangrust::board::{Bitboard, Color, Parser, Piece, Position, Serializer, Square};

#[test]
fn test_same_square_apply_revert_desync() {
    let fen = Parser::DEFAULT;
    let mut pos = Parser::parse(fen);
    let orig_pos = pos;

    // Apply move where from == to (e.g., sq 19 where Red Cannon is)
    let sq = 19u8;
    let state = pos.apply(sq, sq);

    let recomputed_hash = pos.compute();
    println!("Same square apply: pos.hash = {}, pos.compute() = {}", pos.hash, recomputed_hash);

    // Check invariant: hash should match compute
    let hash_matches = pos.hash == recomputed_hash;

    // Revert move
    pos.revert(sq, sq, &state);

    let count_matches = pos.counts[5] == orig_pos.counts[5];
    println!("Revert same square: orig counts[5] = {}, reverted counts[5] = {}", orig_pos.counts[5], pos.counts[5]);

    assert!(hash_matches, "CRITICAL: apply(sq, sq) caused Zobrist hash desync!");
    assert!(count_matches, "CRITICAL: revert(sq, sq) corrupted counts array!");
    assert_eq!(pos, orig_pos, "CRITICAL: position after revert(sq, sq) does not match original!");
}

#[test]
fn test_put_occupied_square_corruption() {
    let mut pos = Position::empty();

    // Put Red Rook (4) at sq 0
    pos.put(4, 0);
    assert_eq!(pos.counts[4], 1);
    assert!(pos.piece[4].test(Square(0)));

    // Put Red Cannon (5) at sq 0 without calling take(0)
    pos.put(5, 0);

    // Check corruptions
    let rook_bit_set = pos.piece[4].test(Square(0));
    let rook_count = pos.counts[4];
    let cannon_count = pos.counts[5];
    let occupied_count = pos.occupied.count();

    println!("Put on occupied sq: rook_bit_set = {}, rook_count = {}, cannon_count = {}, occupied_count = {}",
        rook_bit_set, rook_count, cannon_count, occupied_count);

    assert!(!rook_bit_set, "CRITICAL: pos.put on occupied square left old piece bit in piece[4] bitboard!");
    assert_eq!(rook_count, 0, "CRITICAL: pos.put on occupied square did not decrement old piece count!");
}

#[test]
fn test_square_flip_out_of_bounds_underflow() {
    // Square(90) has rank 90 / 9 = 10.
    // flip() calculates 9 - rank = 9 - 10, causing u8 underflow.
    let sq = Square(90);
    println!("Testing Square(90).flip()...");
    let flipped = sq.flip();
    println!("Flipped square: {:?}", flipped);
}

#[test]
fn test_apply_out_of_bounds_panic() {
    let mut pos = Parser::parse(Parser::DEFAULT);
    // Apply move with square >= 90
    println!("Testing pos.apply(90, 19)...");
    let _state = pos.apply(90, 19);
}

#[test]
fn test_fen_boundary_strings() {
    // 1. All 90 squares occupied is rejected by Validator (multiple kings & river violation)
    let full_fen = "rnbakabnr/rnbakabnr/rnbakabnr/rnbakabnr/rnbakabnr/RNBAKABNR/RNBAKABNR/RNBAKABNR/RNBAKABNR/RNBAKABNR w - - 0 1";
    let pos_full = Parser::parse(full_fen);
    assert_eq!(pos_full.occupied.count(), 0);

    // 2. Empty board FEN
    let empty_fen = "9/9/9/9/9/9/9/9/9/9 w - - 0 1";
    let pos_empty = Parser::parse(empty_fen);
    assert_eq!(pos_empty.occupied.count(), 0);

    // 3. Roundtrip test for empty board
    let exported_empty = Serializer::export(&pos_empty);
    assert_eq!(empty_fen, exported_empty);

    // 4. Invalid FEN characters and spaces
    let junk_fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR_INVALID_TOKENS!!! b";
    let pos_junk = Parser::parse(junk_fen);
    assert_eq!(pos_junk.occupied.count(), 0);
}

#[test]
fn test_randomized_apply_revert_invariants() {
    let mut pos = Parser::parse(Parser::DEFAULT);
    let mut rng = 123456789u64;

    let mut next_u64 = || {
        rng = rng.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    };

    let mut violations = 0;

    for step in 0..10_000 {
        let from = (next_u64() % 90) as u8;
        let to = (next_u64() % 90) as u8;

        let before_pos = pos;
        let state = pos.apply(from, to);

        // Check hash sync
        if pos.hash != pos.compute() {
            println!("Step {}: Hash desync on move from {} to {}! pos.hash = {:x}, compute = {:x}", step, from, to, pos.hash, pos.compute());
            violations += 1;
        }

        // Check bitboard occupied count vs color sum
        let color_sum = pos.color[0].count() + pos.color[1].count();
        if pos.occupied.count() != color_sum {
            println!("Step {}: Occupied count ({}) != color sum ({})", step, pos.occupied.count(), color_sum);
            violations += 1;
        }

        // Check bitboard counts vs counts array
        let mut total_piece_bits = 0;
        for p in 0..14 {
            let bb_count = pos.piece[p].count() as u8;
            if bb_count != pos.counts[p] {
                println!("Step {}: Piece {} count mismatch: bb={} vs counts={}", step, p, bb_count, pos.counts[p]);
                violations += 1;
            }
            total_piece_bits += bb_count;
        }

        if pos.occupied.count() != total_piece_bits as u32 {
            println!("Step {}: Occupied count ({}) != total piece bits ({})", step, pos.occupied.count(), total_piece_bits);
            violations += 1;
        }

        // Revert move
        pos.revert(from, to, &state);

        if pos != before_pos {
            println!("Step {}: Revert failed to restore exact position on move from {} to {}", step, from, to);
            violations += 1;
            break;
        }

        if violations > 20 {
            break;
        }
    }

    assert_eq!(violations, 0, "Found {} invariant violations during randomized apply/revert stress testing!", violations);
}

#[test]
fn test_deep_move_stack_apply_revert() {
    let mut pos = Parser::parse(Parser::DEFAULT);
    let orig_pos = pos;
    let mut rng = 9876543210123456789u64;

    let mut next_u64 = || {
        rng = rng.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    };

    let mut history = Vec::new();

    // Perform 100,000 operations (randomly push move or pop move)
    for step in 0..100_000 {
        let op = next_u64() % 100;
        if op < 60 || history.is_empty() {
            // Apply move
            let from = (next_u64() % 90) as u8;
            let to = (next_u64() % 90) as u8;
            let state = pos.apply(from, to);
            assert_eq!(pos.hash, pos.compute(), "Hash mismatch at step {} after apply({}, {})", step, from, to);
            history.push((from, to, state));
        } else {
            // Revert move
            let (from, to, state) = history.pop().unwrap();
            pos.revert(from, to, &state);
            assert_eq!(pos.hash, pos.compute(), "Hash mismatch at step {} after revert({}, {})", step, from, to);
        }
    }

    // Revert all remaining moves in history
    while let Some((from, to, state)) = history.pop() {
        pos.revert(from, to, &state);
        assert_eq!(pos.hash, pos.compute(), "Hash mismatch during final rewind after apply({}, {})", from, to);
    }

    assert_eq!(pos, orig_pos, "Position mismatch after 100,000 random apply/revert stack operations!");
    assert_eq!(pos.hash, orig_pos.hash, "Hash mismatch after 100,000 random apply/revert stack operations!");
}

#[test]
fn test_piece_enum_and_conversions() {
    // Test all piece codes 0..14
    for code in 0..14u8 {
        let piece = Piece::make(code);
        assert!(piece.valid());
        assert!(!piece.empty());
        assert_eq!(piece.index(), code as usize);

        let color = piece.color().expect("Piece 0..13 must have a color");
        let role = piece.role().expect("Piece 0..13 must have a role");

        if code < 7 {
            assert_eq!(color, Color::Red);
            assert_eq!(color.index(), 0);
            assert_eq!(color.char(), 'w');
        } else {
            assert_eq!(color, Color::Black);
            assert_eq!(color.index(), 1);
            assert_eq!(color.char(), 'b');
        }

        assert_eq!(role.index(), (code % 7) as usize);

        let ch = piece.char();
        let parsed = Piece::parse(ch);
        assert_eq!(parsed, piece, "Piece char roundtrip failed for code {}", code);
    }

    // Test empty piece (code 14)
    let empty_piece = Piece::make(14);
    assert!(!empty_piece.valid());
    assert!(empty_piece.empty());
    assert_eq!(empty_piece.color(), None);
    assert_eq!(empty_piece.role(), None);
    assert_eq!(empty_piece.char(), '.');

    // Test invalid characters
    assert_eq!(Piece::parse('X'), Piece::none());
    assert_eq!(Piece::parse(' '), Piece::none());
    assert_eq!(Piece::parse('\0'), Piece::none());

    // Test color flip
    assert_eq!(Color::Red.flip(), Color::Black);
    assert_eq!(Color::Black.flip(), Color::Red);
}

#[test]
fn test_bitboard_operations_stress() {
    let mut bb = Bitboard::empty();
    assert_eq!(bb.count(), 0);
    assert!(!bb.active());
    assert_eq!(bb.lsb(), None);
    assert_eq!(bb.pop(), None);

    // Set all 90 bits
    for i in 0..90 {
        let sq = Square(i);
        assert!(!bb.test(sq));
        bb.set(sq);
        assert!(bb.test(sq));
    }
    assert_eq!(bb.count(), 90);
    assert!(bb.active());
    assert_eq!(bb.lsb(), Some(Square(0)));

    // Clear even bits
    for i in (0..90).step_by(2) {
        let sq = Square(i);
        bb.clear(sq);
        assert!(!bb.test(sq));
    }
    assert_eq!(bb.count(), 45);

    // Pop all remaining odd bits
    let mut count = 0;
    while let Some(sq) = bb.pop() {
        assert_eq!(sq.0 % 2, 1);
        count += 1;
    }
    assert_eq!(count, 45);
    assert_eq!(bb.count(), 0);
    assert!(!bb.active());
}

#[test]
fn test_fen_parser_malformed_and_extreme_inputs() {
    // Malformed FEN 1: Empty string
    let pos1 = Parser::parse("");
    assert_eq!(pos1.occupied.count(), 0);

    // Malformed FEN 2: Only 1 rank provided
    let pos2 = Parser::parse("rnbakabnr");
    assert_eq!(pos2.occupied.count(), 0);

    // Malformed FEN 3: Unknown characters and digits exceeding 9
    let pos3 = Parser::parse("999999999/9/9/9/9/9/9/9/9/9 b - - 50 100");
    assert_eq!(pos3.occupied.count(), 0);

    // Malformed FEN 4: FEN with invalid turn indicator
    let pos4 = Parser::parse("9/9/9/9/9/9/9/9/9/9 x");
    assert_eq!(pos4.occupied.count(), 0);
}
