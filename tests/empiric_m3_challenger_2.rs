// Test suite kiểm thử Empirical & Adversarial cho Milestone 3 (Challenger 2)
// Kiểm tra Căn lề bộ nhớ align(64) và Invariants Accum::apply + Accum::revert vs Accum::reset

use xiangrust::board::{Parser, Position};
use xiangrust::eval::accum::Accum;
use xiangrust::eval::nnue::{Affine, Nnue, Output, Transform};
use xiangrust::eval::weight::Weight;
use xiangrust::eval::Eval;
use xiangrust::movegen::legal;
use xiangrust::movegen::types::List;
use std::mem::{align_of, size_of};

#[test]
fn test_challenger_m3_memory_alignments_and_padding() {
    println!("=== CHALLENGER M3: MEMORY ALIGNMENT AUDIT ===");

    // 1. Kiểm tra align_of = 64
    assert_eq!(align_of::<Accum>(), 64, "Accum MUST be 64-byte aligned!");
    assert_eq!(align_of::<Eval>(), 64, "Eval MUST be 64-byte aligned!");
    assert_eq!(align_of::<Weight>(), 64, "Weight MUST be 64-byte aligned!");
    assert_eq!(align_of::<Transform>(), 64, "Transform MUST be 64-byte aligned!");
    assert_eq!(align_of::<Affine<512, 32>>(), 64, "Affine MUST be 64-byte aligned!");
    assert_eq!(align_of::<Output<32>>(), 64, "Output MUST be 64-byte aligned!");
    assert_eq!(align_of::<Nnue>(), 64, "Nnue MUST be 64-byte aligned!");
    assert_eq!(align_of::<Position>(), 64, "Position MUST be 64-byte aligned!");

    // 2. Kiểm tra size_of là bội số của 64 bytes
    assert_eq!(size_of::<Accum>() % 64, 0, "Accum size ({}) MUST be multiple of 64!", size_of::<Accum>());
    assert_eq!(size_of::<Eval>() % 64, 0, "Eval size ({}) MUST be multiple of 64!", size_of::<Eval>());
    assert_eq!(size_of::<Weight>() % 64, 0, "Weight size ({}) MUST be multiple of 64!", size_of::<Weight>());
    assert_eq!(size_of::<Transform>() % 64, 0, "Transform size ({}) MUST be multiple of 64!", size_of::<Transform>());
    assert_eq!(size_of::<Affine<512, 32>>() % 64, 0, "Affine size ({}) MUST be multiple of 64!", size_of::<Affine<512, 32>>());
    assert_eq!(size_of::<Output<32>>() % 64, 0, "Output size ({}) MUST be multiple of 64!", size_of::<Output<32>>());
    assert_eq!(size_of::<Nnue>() % 64, 0, "Nnue size ({}) MUST be multiple of 64!", size_of::<Nnue>());
    assert_eq!(size_of::<Position>() % 64, 0, "Position size ({}) MUST be multiple of 64!", size_of::<Position>());

    // 3. Dynamic allocation alignment check
    let eval = Box::new(Eval::new());
    let ptr = &*eval as *const Eval as usize;
    assert_eq!(ptr % 64, 0, "Boxed Eval pointer ({:#x}) MUST be 64-byte aligned!", ptr);

    let accum = Box::new(Accum::new());
    let accum_ptr = &*accum as *const Accum as usize;
    assert_eq!(accum_ptr % 64, 0, "Boxed Accum pointer ({:#x}) MUST be 64-byte aligned!", accum_ptr);
}

#[test]
fn test_challenger_m3_accum_invariants_2000_random_sequences() {
    println!("=== CHALLENGER M3: ACCUMULATOR INVARIANT STRESS TEST (2,000+ MOVES) ===");

    let fen_samples = [
        Parser::DEFAULT,
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        "3ak4/9/4b4/p3p3p/2p6/6P2/P3P3P/4B4/9/3AK4 w - - 0 1",
        "2b1ka3/9/2c1c4/p1p1p1p1p/9/9/P1P1P1P1P/1C2C4/9/2B1KA3 w - - 0 1",
        "r3k2r/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/R3K2R w - - 0 1",
        "3ak4/9/9/9/9/9/9/9/9/4K4 w - - 0 1",
        "r1ba1ab1r/9/1c4nc1/p1p1p1p1p/9/9/P1P1P1P1P/1C4NC1/9/R1BA1AB1R w - - 0 1",
        "4k4/9/9/9/9/9/9/9/9/4K4 w - - 0 1",
        "crnka1b1r/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/CRNKA1B1R w - - 0 1",
        "r2akab2/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/R2AKAB2 w - - 0 1",
    ];

    let mut total_verified_moves = 0usize;
    let mut prng_state = 0xCAFEEFAC12345678u64;

    // Deterministic pseudo-random number generator
    let mut next_rand = || {
        prng_state = prng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        prng_state
    };

    for (fen_idx, fen) in fen_samples.iter().enumerate() {
        let mut pos = Parser::parse(fen);
        let mut eval = Eval::new();
        eval.reset(&pos);

        // Verify initial reset
        let mut fresh = Accum::new();
        fresh.reset(&pos, &eval.nnue.weight);
        assert_eq!(eval.accum, fresh, "Initial reset desync at FEN index {}", fen_idx);

        let moves_per_fen = 300;
        let mut stack: Vec<(xiangrust::board::state::State, u8, u8, u8, u8, Accum)> = Vec::new();

        for step in 0..moves_per_fen {
            let mut moves = List::new();
            legal::gen(&mut pos, &mut moves);

            if moves.count == 0 {
                // Reset to start FEN if no legal moves
                pos = Parser::parse(fen);
                eval.reset(&pos);
                stack.clear();
                continue;
            }

            // Decide whether to make a forward move or undo a move
            let do_undo = !stack.is_empty() && (next_rand() % 4 == 0);

            if do_undo {
                // Undo move test
                let (saved_state, from, to, moving, captured, accum_before) = stack.pop().unwrap();

                // Revert position
                pos.revert(from, to, &saved_state);

                // Revert accumulator
                eval.revert(&pos, from, to, moving, captured);

                // Assert 1: Reverted accum must match saved accum before the move
                assert_eq!(
                    eval.accum, accum_before,
                    "Accum revert failed to restore previous state at FEN {} step {}! move from {} to {}",
                    fen_idx, step, from, to
                );

                // Assert 2: Reverted accum must match clean reset from pos
                let mut fresh_reset = Accum::new();
                fresh_reset.reset(&pos, &eval.nnue.weight);
                assert_eq!(
                    eval.accum, fresh_reset,
                    "Accum revert desync with reset(&pos) at FEN {} step {}!",
                    fen_idx, step
                );

                total_verified_moves += 1;
            } else {
                // Forward move test
                let choice = (next_rand() as usize) % moves.count;
                let mv = moves.items[choice];

                let from = mv.from;
                let to = mv.to;
                let moving = pos.grid[from as usize];
                let captured = pos.grid[to as usize];
                let accum_before = eval.accum;

                // Apply accumulator change
                eval.apply(&pos, from, to, moving, captured);
                let accum_after_apply = eval.accum;

                // Apply position change
                let state = pos.apply(from, to);

                // Assert 1: accum_after_apply must match clean reset on new position
                let mut fresh_reset = Accum::new();
                fresh_reset.reset(&pos, &eval.nnue.weight);
                assert_eq!(
                    accum_after_apply, fresh_reset,
                    "Accum apply desync with reset(&pos) at FEN {} step {}! move from {} to {}, moving={}, captured={}",
                    fen_idx, step, from, to, moving, captured
                );

                // Push to stack for future revert test
                stack.push((state, from, to, moving, captured, accum_before));
                total_verified_moves += 1;
            }
        }

        // Unwind all remaining stack items to verify deep undo stack
        while let Some((saved_state, from, to, moving, captured, accum_before)) = stack.pop() {
            pos.revert(from, to, &saved_state);
            eval.revert(&pos, from, to, moving, captured);

            assert_eq!(eval.accum, accum_before, "Stack unwinding accum mismatch!");
            let mut fresh_reset = Accum::new();
            fresh_reset.reset(&pos, &eval.nnue.weight);
            assert_eq!(eval.accum, fresh_reset, "Stack unwinding reset mismatch!");
            total_verified_moves += 1;
        }
    }

    println!("SUCCESSFULLY VERIFIED {} RANDOM MOVES & UNDOS WITH 100% ACCUM MATCH!", total_verified_moves);
    assert!(total_verified_moves >= 2000, "Must test at least 2,000 move operations!");
}


#[test]
fn test_challenger_m3_king_move_accum_reset_invariant() {
    println!("=== CHALLENGER M3: KING MOVE ACCUMULATOR RESET INVARIANT ===");

    // Test specific scenario: King moves MUST trigger a clean reset of accumulator
    let mut pos = Parser::parse(Parser::DEFAULT);
    let mut eval = Eval::new();
    eval.reset(&pos);

    // Find a legal move for King (piece % 7 == 0)
    let mut moves = List::new();
    legal::gen(&mut pos, &mut moves);

    for i in 0..moves.count {
        let _mv = moves.items[i];
    }

    // In initial position, King has no legal moves. Let's create a custom FEN where King can move.
    let fen_king_can_move = "3k5/9/9/9/9/9/9/9/9/4K4 w - - 0 1";
    let mut pos = Parser::parse(fen_king_can_move);
    eval.reset(&pos);

    moves.clear();
    legal::gen(&mut pos, &mut moves);

    assert!(moves.count > 0, "King should have legal moves in custom FEN!");
    let mv = moves.items[0];
    let moving = pos.grid[mv.from as usize];
    assert_eq!(moving % 7, 0, "Selected piece must be King!");

    let from = mv.from;
    let to = mv.to;
    let captured = pos.grid[to as usize];

    // Apply accum
    eval.apply(&pos, from, to, moving, captured);
    let state = pos.apply(from, to);

    // Verify accum after King move matches clean reset
    let mut fresh = Accum::new();
    fresh.reset(&pos, &eval.nnue.weight);
    assert_eq!(eval.accum, fresh, "King move accum MUST match fresh reset 100%!");

    // Revert accum
    pos.revert(from, to, &state);
    eval.revert(&pos, from, to, moving, captured);

    let mut fresh_before = Accum::new();
    fresh_before.reset(&pos, &eval.nnue.weight);
    assert_eq!(eval.accum, fresh_before, "King move revert accum MUST match fresh reset before move!");
}
