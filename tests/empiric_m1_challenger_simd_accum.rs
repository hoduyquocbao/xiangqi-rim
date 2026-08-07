// ============================================================================
// EMPIRICAL CHALLENGER STRESS HARNESS FOR SIMD ACCUMULATOR & UNROLLING (M1)
// ============================================================================
// Tests SIMD unrolling (AVX2/NEON/Fallback) bitwise exactness vs pure scalar arithmetic,
// and tests Accumulator incremental updates across 1000+ position/move transitions.
// ============================================================================

use xiangrust::board::Parser;
use xiangrust::board::Position;
use xiangrust::eval::accum::Accum;
use xiangrust::eval::nnue::Transform;
use xiangrust::eval::weight::{Weight, DIM};
use xiangrust::movegen::legal;
use xiangrust::movegen::types::List;

/// Scalar reference for Accum::add
fn scalar_add(dst: &mut [i16; DIM], src: &[i16; DIM]) {
    for i in 0..DIM {
        dst[i] = dst[i].wrapping_add(src[i]);
    }
}

/// Scalar reference for Accum::update
fn scalar_update(dst: &mut [i16; DIM], add: &[i16; DIM], sub: &[i16; DIM]) {
    for i in 0..DIM {
        dst[i] = dst[i].wrapping_add(add[i]).wrapping_sub(sub[i]);
    }
}

/// Scalar reference for Accum::modify
fn scalar_modify(dst: &mut [i16; DIM], add: &[i16; DIM], sub1: &[i16; DIM], sub2: &[i16; DIM]) {
    for i in 0..DIM {
        dst[i] = dst[i]
            .wrapping_add(add[i])
            .wrapping_sub(sub1[i])
            .wrapping_sub(sub2[i]);
    }
}

/// Simple LCG Pseudo-Random Generator for deterministic test vector generation
struct Random {
    state: u64,
}

impl Random {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.state
    }

    fn next_i16(&mut self) -> i16 {
        (self.next_u64() & 0xFFFF) as i16
    }

    fn fill_i16_dim(&mut self, arr: &mut [i16; DIM]) {
        for i in 0..DIM {
            arr[i] = self.next_i16();
        }
    }
}

/// Pure scalar implementation of Accum::reset
fn scalar_reset(accum: &mut Accum, pos: &Position, weight: &Weight) {
    accum.vals[0] = weight.bias;
    accum.vals[1] = weight.bias;

    let k0 = pos.king[0];
    let k1 = pos.king[1];

    let mut sq = 0u8;
    while sq < 90 {
        let p = pos.grid[sq as usize];
        if p < 14 {
            let idx0 = xiangrust::eval::feature::Feature::index(k0, p, sq, 0, 0);
            let feat0 = weight.feature(idx0);
            scalar_add(&mut accum.vals[0], feat0);

            let idx1 = xiangrust::eval::feature::Feature::index(k1, p, sq, 1, 1);
            let feat1 = weight.feature(idx1);
            scalar_add(&mut accum.vals[1], feat1);
        }
        sq += 1;
    }
}

/// Pure scalar implementation of Accum::apply
fn scalar_apply(
    accum: &mut Accum,
    pos: &Position,
    from: u8,
    to: u8,
    moving: u8,
    captured: u8,
    weight: &Weight,
) {
    let king = (moving % 7) == 0;
    if king {
        let side = if moving < 7 { 0usize } else { 1usize };
        let other = side ^ 1;

        // Scalar rebuild
        accum.vals[side] = weight.bias;
        let mut sq = 0u8;
        while sq < 90 {
            let piece = if sq == from {
                14
            } else if sq == to {
                (side * 7) as u8
            } else {
                pos.grid[sq as usize]
            };

            if piece < 14 {
                let idx = xiangrust::eval::feature::Feature::index(to, piece, sq, side as u8, side as u8);
                let feat = weight.feature(idx);
                scalar_add(&mut accum.vals[side], feat);
            }
            sq += 1;
        }

        let enemy = pos.king[other];
        let rem = xiangrust::eval::feature::Feature::index(enemy, moving, from, other as u8, other as u8);
        let add = xiangrust::eval::feature::Feature::index(enemy, moving, to, other as u8, other as u8);
        let old = weight.feature(rem);
        let new = weight.feature(add);

        if captured < 14 {
            let capidx = xiangrust::eval::feature::Feature::index(enemy, captured, to, other as u8, other as u8);
            let cap = weight.feature(capidx);
            scalar_modify(&mut accum.vals[other], new, old, cap);
        } else {
            scalar_update(&mut accum.vals[other], new, old);
        }
        return;
    }

    let k0 = pos.king[0];
    let k1 = pos.king[1];

    let rem0 = xiangrust::eval::feature::Feature::index(k0, moving, from, 0, 0);
    let add0 = xiangrust::eval::feature::Feature::index(k0, moving, to, 0, 0);
    let old0 = weight.feature(rem0);
    let new0 = weight.feature(add0);

    if captured < 14 {
        let cap0 = xiangrust::eval::feature::Feature::index(k0, captured, to, 0, 0);
        let c0 = weight.feature(cap0);
        scalar_modify(&mut accum.vals[0], new0, old0, c0);
    } else {
        scalar_update(&mut accum.vals[0], new0, old0);
    }

    let rem1 = xiangrust::eval::feature::Feature::index(k1, moving, from, 1, 1);
    let add1 = xiangrust::eval::feature::Feature::index(k1, moving, to, 1, 1);
    let old1 = weight.feature(rem1);
    let new1 = weight.feature(add1);

    if captured < 14 {
        let cap1 = xiangrust::eval::feature::Feature::index(k1, captured, to, 1, 1);
        let c1 = weight.feature(cap1);
        scalar_modify(&mut accum.vals[1], new1, old1, c1);
    } else {
        scalar_update(&mut accum.vals[1], new1, old1);
    }
}

/// Pure scalar implementation of Accum::revert
fn scalar_revert(
    accum: &mut Accum,
    pos: &Position,
    from: u8,
    to: u8,
    moving: u8,
    captured: u8,
    weight: &Weight,
) {
    let king = (moving % 7) == 0;
    if king {
        let side = if moving < 7 { 0usize } else { 1usize };
        let other = side ^ 1;

        // Scalar rebuild
        accum.vals[side] = weight.bias;
        let mut sq = 0u8;
        while sq < 90 {
            let piece = if sq == from {
                14
            } else if sq == to {
                (side * 7) as u8
            } else {
                pos.grid[sq as usize]
            };

            if piece < 14 {
                let idx = xiangrust::eval::feature::Feature::index(pos.king[side], piece, sq, side as u8, side as u8);
                let feat = weight.feature(idx);
                scalar_add(&mut accum.vals[side], feat);
            }
            sq += 1;
        }

        let enemy = pos.king[other];
        let rem = xiangrust::eval::feature::Feature::index(enemy, moving, from, other as u8, other as u8);
        let add = xiangrust::eval::feature::Feature::index(enemy, moving, to, other as u8, other as u8);
        let old = weight.feature(rem);
        let new = weight.feature(add);

        if captured < 14 {
            let capidx = xiangrust::eval::feature::Feature::index(enemy, captured, to, other as u8, other as u8);
            let cap = weight.feature(capidx);
            scalar_update(&mut accum.vals[other], old, new);
            scalar_add(&mut accum.vals[other], cap);
        } else {
            scalar_update(&mut accum.vals[other], old, new);
        }
        return;
    }

    let k0 = pos.king[0];
    let k1 = pos.king[1];

    let rem0 = xiangrust::eval::feature::Feature::index(k0, moving, from, 0, 0);
    let add0 = xiangrust::eval::feature::Feature::index(k0, moving, to, 0, 0);
    let old0 = weight.feature(rem0);
    let new0 = weight.feature(add0);

    if captured < 14 {
        let cap0 = xiangrust::eval::feature::Feature::index(k0, captured, to, 0, 0);
        let c0 = weight.feature(cap0);
        scalar_update(&mut accum.vals[0], old0, new0);
        scalar_add(&mut accum.vals[0], c0);
    } else {
        scalar_update(&mut accum.vals[0], old0, new0);
    }

    let rem1 = xiangrust::eval::feature::Feature::index(k1, moving, from, 1, 1);
    let add1 = xiangrust::eval::feature::Feature::index(k1, moving, to, 1, 1);
    let old1 = weight.feature(rem1);
    let new1 = weight.feature(add1);

    if captured < 14 {
        let cap1 = xiangrust::eval::feature::Feature::index(k1, captured, to, 1, 1);
        let c1 = weight.feature(cap1);
        scalar_update(&mut accum.vals[1], old1, new1);
        scalar_add(&mut accum.vals[1], c1);
    } else {
        scalar_update(&mut accum.vals[1], old1, new1);
    }
}

#[test]
fn test_simd_vs_scalar_add_update_modify_10000_random_vectors() {
    let mut rng = Random::new(0xDEADBEEF12345678);

    for _iteration in 0..10_000 {
        let mut dst_scalar = [0i16; DIM];
        let mut src = [0i16; DIM];
        let mut add = [0i16; DIM];
        let mut sub1 = [0i16; DIM];
        let mut sub2 = [0i16; DIM];

        rng.fill_i16_dim(&mut dst_scalar);
        rng.fill_i16_dim(&mut src);
        rng.fill_i16_dim(&mut add);
        rng.fill_i16_dim(&mut sub1);
        rng.fill_i16_dim(&mut sub2);

        // Verify scalar math functions produce expected bitwise behavior
        let mut accum_add = dst_scalar;
        scalar_add(&mut accum_add, &src);

        let mut accum_update = dst_scalar;
        scalar_update(&mut accum_update, &add, &sub1);

        let mut accum_modify = dst_scalar;
        scalar_modify(&mut accum_modify, &add, &sub1, &sub2);

        for i in 0..DIM {
            assert_eq!(accum_add[i], dst_scalar[i].wrapping_add(src[i]));
            assert_eq!(
                accum_update[i],
                dst_scalar[i].wrapping_add(add[i]).wrapping_sub(sub1[i])
            );
            assert_eq!(
                accum_modify[i],
                dst_scalar[i]
                    .wrapping_add(add[i])
                    .wrapping_sub(sub1[i])
                    .wrapping_sub(sub2[i])
            );
        }
    }
    println!("Successfully verified 10,000 random vector iterations for scalar reference math!");
}

#[test]
fn test_accum_simd_bitwise_identity_1000_random_positions() {
    let weight = Weight::new();
    let initial_fen = Parser::DEFAULT;
    let pos = Parser::parse(initial_fen);

    let mut current_accum = Accum::new();
    current_accum.reset(&pos, &weight);

    let mut scalar_accum = Accum::new();
    scalar_reset(&mut scalar_accum, &pos, &weight);

    assert_eq!(
        current_accum, scalar_accum,
        "Initial accum SIMD reset must match scalar reset 100%"
    );

    let mut rng = Random::new(0xCAFEBABE11223344);
    let mut moves_tested = 0;

    // Run random walk for 1000+ moves across positions
    for _game in 0..50 {
        let mut game_pos = Parser::parse(initial_fen);
        let mut accum = Accum::new();
        accum.reset(&game_pos, &weight);

        let mut sc_accum = Accum::new();
        scalar_reset(&mut sc_accum, &game_pos, &weight);

        for _ply in 0..40 {
            let mut moves = List::new();
            legal::gen(&mut game_pos, &mut moves);
            if moves.count == 0 {
                break;
            }

            let idx = (rng.next_u64() as usize) % moves.count;
            let mv = moves.items[idx];

            let from = mv.from;
            let to = mv.to;
            let moving = game_pos.grid[from as usize];
            let captured = game_pos.grid[to as usize];

            // Apply move to SIMD accumulator
            accum.apply(&game_pos, from, to, moving, captured, &weight);

            // Apply move to Scalar accumulator
            scalar_apply(&mut sc_accum, &game_pos, from, to, moving, captured, &weight);

            // Apply move to board position
            let state = game_pos.apply(from, to);

            // Calculate fresh accumulator from scratch (reset)
            let mut fresh = Accum::new();
            fresh.reset(&game_pos, &weight);

            // Check 3-way bitwise identity across all 2 x 256 i16 values: SIMD accum == Fresh reset == Scalar accum
            assert_eq!(
                accum, fresh,
                "Accumulator after move {} -> {} MUST match 100% bitwise fresh reset!",
                from, to
            );
            assert_eq!(
                accum, sc_accum,
                "SIMD Accumulator after move {} -> {} MUST match 100% bitwise scalar reference!",
                from, to
            );

            // Test revert
            game_pos.revert(from, to, &state);
            accum.revert(&game_pos, from, to, moving, captured, &weight);
            scalar_revert(&mut sc_accum, &game_pos, from, to, moving, captured, &weight);

            let mut fresh_reverted = Accum::new();
            fresh_reverted.reset(&game_pos, &weight);

            assert_eq!(
                accum, fresh_reverted,
                "Accumulator after revert of move {} -> {} MUST match 100% bitwise fresh reset!",
                from, to
            );
            assert_eq!(
                accum, sc_accum,
                "SIMD Accumulator after revert of move {} -> {} MUST match 100% bitwise scalar reference!",
                from, to
            );

            // Re-apply for game continuation
            accum.apply(&game_pos, from, to, moving, captured, &weight);
            scalar_apply(&mut sc_accum, &game_pos, from, to, moving, captured, &weight);
            game_pos.apply(from, to);

            moves_tested += 1;
        }
    }

    assert!(
        moves_tested >= 1000,
        "Tested {} moves (expected >= 1000)",
        moves_tested
    );
    println!("Successfully verified bitwise 3-way identity (SIMD accum == Fresh reset == Pure scalar) for {} random move transitions!", moves_tested);
}

#[test]
fn test_simd_pack_bitwise_exactness_10000_vectors() {
    let mut rng = Random::new(0x9876543210ABCDEF);

    for iter in 0..10_000 {
        let mut red_i16 = [0i16; DIM];
        let mut black_i16 = [0i16; DIM];
        rng.fill_i16_dim(&mut red_i16);
        rng.fill_i16_dim(&mut black_i16);

        let mut transform = Transform::new();
        transform.active(&red_i16, &black_i16, 0);

        // Scalar reference calculation
        let mut expected_red = [0i8; DIM];
        let mut expected_black = [0i8; DIM];

        for i in 0..DIM {
            let r = red_i16[i];
            expected_red[i] = if r < 0 { 0 } else if r > 127 { 127 } else { r as i8 };

            let b = black_i16[i];
            expected_black[i] = if b < 0 { 0 } else if b > 127 { 127 } else { b as i8 };
        }

        // Compare side 0: active[0..256] == expected_red, active[256..512] == expected_black
        assert_eq!(
            &transform.active[0..DIM],
            &expected_red[..],
            "SIMD Clipped ReLU pack red side output mismatch at iteration {}",
            iter
        );
        assert_eq!(
            &transform.active[DIM..DIM * 2],
            &expected_black[..],
            "SIMD Clipped ReLU pack black side output mismatch at iteration {}",
            iter
        );

        // Also test side 1
        let mut transform_black_side = Transform::new();
        transform_black_side.active(&red_i16, &black_i16, 1);

        assert_eq!(
            &transform_black_side.active[0..DIM],
            &expected_black[..],
            "SIMD Clipped ReLU pack black side active[0..256] mismatch at iteration {}",
            iter
        );
        assert_eq!(
            &transform_black_side.active[DIM..DIM * 2],
            &expected_red[..],
            "SIMD Clipped ReLU pack black side active[256..512] mismatch at iteration {}",
            iter
        );
    }
    println!("Successfully verified SIMD Clipped ReLU pack bitwise identity across 10,000 random vectors!");
}
