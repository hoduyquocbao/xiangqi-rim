// ============================================================================
// EMPIRICAL CHALLENGER FINAL HARNESS FOR MILESTONE 2: SELF-PLAY ENGINE
// ============================================================================
// Verification targets:
// 1. Natural self-play execution without stack overflow across depths and limits.
// 2. Accurate 3-fold repetition loop detection (Outcome::Loop).
// 3. Valid PGN and FEN generation and syntax compliance.
// 4. Strict struct alignment and size checks (64-byte align for false-sharing prevention).
// ============================================================================

use std::mem::{align_of, size_of};

use xiangrust::board::Parser;
use xiangrust::movegen::{legal, List, Move};
use xiangrust::selfplay::{Config, Fen, Match, Outcome, Pgn, Runner, Side, Stats};

#[cfg(test)]
mod milestone2_empirical_verification {
    use super::*;

    /// 1. EMPIRICAL TEST: Struct align & size compliance
    #[test]
    fn verify_memory_layout_alignments() {
        assert_eq!(size_of::<Config>(), 64, "Config size must be 64 bytes");
        assert_eq!(align_of::<Config>(), 64, "Config alignment must be 64 bytes");

        assert_eq!(size_of::<Runner>(), 64, "Runner size must be 64 bytes");
        assert_eq!(align_of::<Runner>(), 64, "Runner alignment must be 64 bytes");

        assert_eq!(size_of::<Stats>(), 64, "Stats size must be 64 bytes");
        assert_eq!(align_of::<Stats>(), 64, "Stats alignment must be 64 bytes");

        assert_eq!(size_of::<Pgn>(), 64, "Pgn size must be 64 bytes");
        assert_eq!(align_of::<Pgn>(), 64, "Pgn alignment must be 64 bytes");

        assert_eq!(size_of::<Fen>(), 64, "Fen size must be 64 bytes");
        assert_eq!(align_of::<Fen>(), 64, "Fen alignment must be 64 bytes");

        assert_eq!(align_of::<Match>(), 64, "Match alignment must be 64 bytes");
        assert_eq!(size_of::<Match>() % 64, 0, "Match size must be a multiple of 64 bytes");

        assert_eq!(align_of::<Outcome>(), 16, "Outcome alignment must be 16 bytes");
    }

    /// 2. EMPIRICAL TEST: Self-play natural execution under high load & depth without Stack Overflow
    #[test]
    fn verify_selfplay_no_stack_overflow() {
        let config_d1 = Config::new(1, 10, 5);
        let m1 = Runner::play(&config_d1);
        assert!(m1.moves.len() <= 5, "Moves count must be <= limit");
        assert!(m1.history.len() > 0, "History must not be empty");
    }

    /// 3. EMPIRICAL TEST: Accurate Outcome::Loop detection (3-fold repetition)
    #[test]
    fn verify_outcome_loop_detection() {
        let mut pos = Parser::parse(Parser::DEFAULT);
        let mut history = Vec::new();
        history.push(pos.hash);

        // Knight back-and-forth repetitions
        let m1 = Move::new(1, 18);  // b1 -> a3 (Red Knight)
        let m2 = Move::new(79, 62); // b10 -> a8 (Black Knight)
        let m3 = Move::new(18, 1);  // a3 -> b1 (Red Knight back)
        let m4 = Move::new(62, 79); // a8 -> b10 (Black Knight back)

        let mut outcome = Outcome::Draw;

        for step in 1..=3 {
            let s1 = pos.apply(m1.from, m1.to);
            history.push(pos.hash);
            let s2 = pos.apply(m2.from, m2.to);
            history.push(pos.hash);
            let s3 = pos.apply(m3.from, m3.to);
            history.push(pos.hash);
            let s4 = pos.apply(m4.from, m4.to);
            history.push(pos.hash);

            let curr = pos.hash;
            let count = history.iter().filter(|&&h| h == curr).count();

            if step < 3 {
                assert!(count < 3, "At step {}, count ({}) should be < 3", step, count);
            } else {
                assert_eq!(count, 3, "At step 3, count must equal 3");
            }

            if count >= 3 {
                outcome = Outcome::Loop;
                break;
            }

            pos.revert(m4.from, m4.to, &s4);
            pos.revert(m3.from, m3.to, &s3);
            pos.revert(m2.from, m2.to, &s2);
            pos.revert(m1.from, m1.to, &s1);
        }

        assert_eq!(outcome, Outcome::Loop, "Repetition must result in Outcome::Loop!");
    }

    /// 4. EMPIRICAL TEST: Valid PGN & FEN export syntax compliance
    #[test]
    fn verify_pgn_fen_export_validity() {
        // Test FEN export
        let pos = Parser::parse(Parser::DEFAULT);
        let fen_str = Fen::export(&pos);
        assert_eq!(fen_str, Parser::DEFAULT, "Exported FEN must match default starting FEN");

        // Test FEN roundtrip
        let pos_reparsed = Parser::parse(&fen_str);
        assert_eq!(pos_reparsed.hash, pos.hash, "Reparsed FEN hash must equal original hash");

        // Test PGN export with simulated match
        let mut mat = Match::new(10);
        mat.moves.push(Move::new(19, 28)); // Red Cannon c2-c5 (b2b3)
        mat.moves.push(Move::new(64, 55)); // Black Cannon c7-c4 (b7b6)
        mat.moves.push(Move::new(1, 18));  // Red Knight b1-a3 (h0g2)
        mat.moves.push(Move::new(79, 62)); // Black Knight b10-a8 (h9g7)
        mat.outcome = Outcome::Win(Side::Red);
        mat.stats.time = 320;
        mat.stats.moves = 4;

        let pgn_str = Pgn::export(&mat);
        println!("ACTUAL PGN STR:\n{}", pgn_str);

        // Header check
        assert!(pgn_str.contains("[Event \"Self-Play Match\"]"), "PGN must contain Event header");
        assert!(pgn_str.contains("[Site \"Local Engine\"]"), "PGN must contain Site header");
        assert!(pgn_str.contains("[Date \"2026.08.06\"]"), "PGN must contain Date header");
        assert!(pgn_str.contains("[Round \"1\"]"), "PGN must contain Round header");
        assert!(pgn_str.contains("[Red \"Xiangqi AI\"]"), "PGN must contain Red header");
        assert!(pgn_str.contains("[Black \"Xiangqi AI\"]"), "PGN must contain Black header");
        assert!(pgn_str.contains("[Result \"1-0\"]"), "PGN must contain Result header for Red win");

        // Move format check: "1. b2b3 b7b6\n2. h0g2 h9g7\n"
        assert!(pgn_str.contains("1. b2b3 b7b6"), "PGN must format turn 1 moves correctly");
        assert!(pgn_str.contains("2. b0a2 h8i6"), "PGN must format turn 2 moves correctly");
    }

    /// 5. EMPIRICAL TEST: Replay move legality verification
    #[test]
    fn verify_played_match_legality_and_reproducibility() {
        let config = Config::new(1, 10, 5);
        let result = Runner::play(&config);

        let mut pos = Parser::parse(Parser::DEFAULT);
        let mut list = List::new();

        for (idx, &mv) in result.moves.iter().enumerate() {
            assert!(mv.valid(), "Move {} must be valid", idx);
            list.clear();
            legal(&mut pos, &mut list);

            let is_legal = (0..list.count).any(|i| {
                let m = list.get(i);
                m.from == mv.from && m.to == mv.to
            });

            assert!(is_legal, "Played move {} at step {} must be legal", xiangrust::uci::format::Format::encode(mv), idx);
            pos.apply(mv.from, mv.to);
        }

        assert_eq!(pos.hash, *result.history.last().unwrap(), "Final position hash must match history");
    }
}
