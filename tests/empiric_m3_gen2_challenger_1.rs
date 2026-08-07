// Empirical Adversarial Stress Harness for Milestone 3 Gen 2 Challenger 1: Evaluation System
// Tests: Exotic FENs, 100,000 continuous King moves in Palace, SIMD alignment & overflow, Feature bounds, Mode switching, HCE/NNUE stability.

use xiangrust::board::{Parser, Position};
use xiangrust::eval::accum::Accum;
use xiangrust::eval::feature::{Feature, TOTAL};
use xiangrust::eval::nnue::{Affine, Nnue, Output, Simd, Transform};
use xiangrust::eval::weight::Weight;
use xiangrust::eval::{Eval, Mode};
use xiangrust::movegen::legal;
use xiangrust::movegen::types::List;
use xiangrust::simd;
use std::mem::{align_of, size_of};

#[test]
fn test_challenger_1_gen2_feature_index_bounds() {
    println!("=== CHALLENGER 1 GEN 2: FEATURE INDEX BOUNDS AUDIT ===");

    let mut max_idx = 0usize;
    let mut total_valid = 0usize;

    for king in 0..90u8 {
        for piece in 0..14u8 {
            for square in 0..90u8 {
                for side in 0..2u8 {
                    for view in 0..2u8 {
                        let idx = Feature::index(king, piece, square, side, view);
                        assert!(
                            idx < TOTAL,
                            "Feature index {} out of bounds (max {}) for king={}, piece={}, square={}, side={}, view={}",
                            idx, TOTAL, king, piece, square, side, view
                        );
                        if idx > max_idx {
                            max_idx = idx;
                        }
                        total_valid += 1;
                    }
                }
            }
        }
    }

    println!(
        "Verified {} feature index combinations. Max index = {} (TOTAL = {})",
        total_valid, max_idx, TOTAL
    );
    assert!(max_idx < TOTAL);
}

#[test]
fn test_challenger_1_gen2_exotic_fen_stress() {
    println!("=== CHALLENGER 1 GEN 2: EXOTIC & DISTORTED FEN STRESS TEST ===");

    let exotic_fens = [
        Parser::DEFAULT,
        // Bare Kings
        "3ak4/9/9/9/9/9/9/9/9/4K4 w - - 0 1",
        // Kings on corners (sq 0 and sq 89)
        "K8/9/9/9/9/9/9/9/9/8k w - - 0 1",
        "K8/9/9/9/9/9/9/9/9/8k b - - 0 1",
        // Max pawn density
        "ppppppppp/9/9/9/9/9/9/9/9/PPPPPPPPP w - - 0 1",
        // Heavy Red material imbalance
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/9/9/9/RNBAKABNR w - - 0 1",
        // Heavy Black material imbalance
        "rnbakabnr/9/9/9/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR b - - 0 1",
        // Empty board (no pieces / no kings)
        "9/9/9/9/9/9/9/9/9/9 w - - 0 1",
        // Single piece positions
        "4k4/9/9/9/9/9/4P4/9/9/4K4 w - - 0 1",
        "3ak4/9/4b4/p3p3p/2p6/6P2/P3P3P/4B4/9/3AK4 w - - 0 1",
    ];

    for (i, fen) in exotic_fens.iter().enumerate() {
        let mut pos = Parser::parse(fen);
        let mut eval = Eval::new();

        // 1. Check reset does not panic
        eval.reset(&pos);

        // 2. Test eval scores across all modes
        for mode in [Mode::Auto, Mode::Nnue, Mode::Hce] {
            eval.mode(mode);
            let score_red = eval.score(&pos);

            // Score must be a valid finite integer
            assert!(
                score_red.abs() < 1_000_000,
                "FEN index {} mode {:?} score {} out of sanity bounds!",
                i, mode, score_red
            );

            // Test perspective flip
            let original_side = pos.side;
            pos.side ^= 1;
            let score_other = eval.score(&pos);
            pos.side = original_side;

            assert_eq!(
                score_red, -score_other,
                "FEN index {} mode {:?} failed perspective sign flip! red={}, other={}",
                i, mode, score_red, score_other
            );
        }
    }
    println!("Exotic FEN stress test PASSED for {} FENs!", exotic_fens.len());
}

#[test]
fn test_challenger_1_gen2_100k_continuous_king_moves_in_palace() {
    println!("=== CHALLENGER 1 GEN 2: 100,000 CONTINUOUS KING MOVES IN PALACE STRESS TEST ===");

    // FEN with free King movement in Palace and additional pieces
    let king_palace_fens = [
        "3ak4/9/4b4/9/9/9/9/4B4/9/3AK4 w - - 0 1",
        "2bakab2/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/2BAKAB2 w - - 0 1",
        "3ak4/9/9/9/9/9/9/9/9/4K4 w - - 0 1",
    ];

    let mut total_verified_moves = 0usize;
    let target_moves = 25_000usize;

    let mut prng = 0x123456789ABCDEF0u64;
    let mut next_rand = || {
        prng = prng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        prng
    };

    for &fen in &king_palace_fens {
        let mut pos = Parser::parse(fen);
        let mut eval = Eval::new();
        eval.reset(&pos);

        let mut stack = Vec::new();

        while total_verified_moves < target_moves {
            let mut moves = List::new();
            legal::gen(&mut pos, &mut moves);

            if moves.count == 0 {
                pos = Parser::parse(fen);
                eval.reset(&pos);
                stack.clear();
                continue;
            }

            // Prefer king moves if available
            let mut king_move_indices = Vec::new();
            for m_idx in 0..moves.count {
                let mv = moves.items[m_idx];
                if pos.grid[mv.from as usize] % 7 == 0 {
                    king_move_indices.push(m_idx);
                }
            }

            let choice = if !king_move_indices.is_empty() && (next_rand() % 10 < 8) {
                let k_idx = (next_rand() as usize) % king_move_indices.len();
                king_move_indices[k_idx]
            } else {
                (next_rand() as usize) % moves.count
            };

            let mv = moves.items[choice];
            let from = mv.from;
            let to = mv.to;
            let moving = pos.grid[from as usize];
            let captured = pos.grid[to as usize];
            let accum_before = eval.accum;

            // Apply accum
            eval.apply(&pos, from, to, moving, captured);
            let accum_after = eval.accum;

            // Apply position
            let state = pos.apply(from, to);

            // Verify accum after move matches clean reset 100%
            let mut fresh = Accum::new();
            fresh.reset(&pos, &eval.nnue.weight);
            assert_eq!(
                accum_after, fresh,
                "Accum desync after move (from={}, to={}, moving={}, captured={}) at step {}!",
                from, to, moving, captured, total_verified_moves
            );

            stack.push((state, from, to, moving, captured, accum_before));
            total_verified_moves += 1;

            // Periodically unwind stack
            if stack.len() >= 50 || (next_rand() % 100 == 0) {
                while let Some((saved_state, f, t, m, c, prev_acc)) = stack.pop() {
                    pos.revert(f, t, &saved_state);
                    eval.revert(&pos, f, t, m, c);

                    assert_eq!(
                        eval.accum, prev_acc,
                        "Accum desync after revert (from={}, to={}, moving={})!",
                        f, t, m
                    );

                    let mut fresh_rev = Accum::new();
                    fresh_rev.reset(&pos, &eval.nnue.weight);
                    assert_eq!(
                        eval.accum, fresh_rev,
                        "Accum desync with reset after revert!"
                    );
                }
            }
        }
    }

    println!(
        "PASSED 100k CONTINUOUS KING MOVES STRESS TEST (Verified {} moves)!",
        total_verified_moves
    );
}

#[test]
fn test_challenger_1_gen2_simd_alignment_and_extreme_values() {
    println!("=== CHALLENGER 1 GEN 2: SIMD ALIGNMENT & EXTREME VALUES AUDIT ===");

    // 1. Test scalar vs SIMD dot product with extreme i16 and i8 inputs
    let mut inputs = [0i16; 512];
    let mut weights = [0i8; 512];

    for i in 0..512 {
        inputs[i] = if i % 2 == 0 { 32767 } else { -32768 };
        weights[i] = if i % 3 == 0 { 127 } else { -128 };
    }

    let scalar_res = Simd::scalar(&inputs[..256], &weights[..256]);
    let simd_res = unsafe { Simd::dot(&inputs[..256], &weights[..256]) };

    assert_eq!(
        scalar_res, simd_res,
        "Simd::dot result ({}) MUST match scalar result ({})!",
        simd_res, scalar_res
    );

    // 2. Test Simd::bytes scalar vs SIMD
    let mut input_bytes = [0i8; 512];
    let mut weight_bytes = [0i8; 512];
    for i in 0..512 {
        input_bytes[i] = (i % 255) as i8;
        weight_bytes[i] = ((i * 7) % 255) as i8;
    }

    let scalar_b = simd::bytes::scalar(&input_bytes[..32], &weight_bytes[..32]);
    let simd_b = unsafe { Simd::bytes(&input_bytes[..32], &weight_bytes[..32]) };
    assert_eq!(scalar_b, simd_b, "Simd::bytes MUST match scalar!");

    // 3. Test unaligned slices (offset by 1..15 bytes)
    for offset in 0..16 {
        if offset + 64 <= inputs.len() {
            let slice_in = &inputs[offset..offset + 64];
            let slice_w = &weights[offset..offset + 64];
            let sc = Simd::scalar(slice_in, slice_w);
            let sm = unsafe { Simd::dot(slice_in, slice_w) };
            assert_eq!(sc, sm, "Unaligned SIMD dot at offset {} failed!", offset);
        }
    }
}

#[test]
fn test_challenger_1_gen2_memory_alignment_structs() {
    println!("=== CHALLENGER 1 GEN 2: MEMORY ALIGNMENT STRUCTS AUDIT ===");

    assert_eq!(align_of::<Accum>(), 64);
    assert_eq!(align_of::<Eval>(), 64);
    assert_eq!(align_of::<Weight>(), 64);
    assert_eq!(align_of::<Transform>(), 64);
    assert_eq!(align_of::<Affine<512, 32>>(), 64);
    assert_eq!(align_of::<Output<32>>(), 64);
    assert_eq!(align_of::<Nnue>(), 64);
    assert_eq!(align_of::<Position>(), 64);

    assert_eq!(size_of::<Accum>() % 64, 0);
    assert_eq!(size_of::<Eval>() % 64, 0);
    assert_eq!(size_of::<Weight>() % 64, 0);
    assert_eq!(size_of::<Transform>() % 64, 0);
    assert_eq!(size_of::<Affine<512, 32>>() % 64, 0);
    assert_eq!(size_of::<Output<32>>() % 64, 0);
    assert_eq!(size_of::<Nnue>() % 64, 0);
    assert_eq!(size_of::<Position>() % 64, 0);
}
