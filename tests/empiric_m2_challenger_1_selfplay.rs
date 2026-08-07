// ============================================================================
// EMPIRICAL CHALLENGER 1: SELF-PLAY ENGINE SPECIFICATION & CORRECTNESS TESTS
// ============================================================================

use xiangrust::board::Parser;
use xiangrust::movegen::{legal, List, Move};
use xiangrust::selfplay::{Config, Fen, Match, Outcome, Pgn, Runner, Side, Stats};
use xiangrust::uci::format::Format;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test match execution and replay move-by-move legal validation
    #[test]
    fn replay() {
        let config = Config::new(1, 50, 10);
        let result = Runner::play(&config);

        let mut pos = Parser::parse(Parser::DEFAULT);
        let mut list = List::new();

        let mut idx = 0;
        while idx < result.moves.len() {
            let mv = result.moves[idx];
            assert!(mv.valid(), "Move at step {} must be valid", idx);

            list.clear();
            legal(&mut pos, &mut list);

            let mut found = false;
            let mut i = 0;
            while i < list.count {
                let m = list.get(i);
                if m.from == mv.from && m.to == mv.to {
                    found = true;
                    break;
                }
                i += 1;
            }

            assert!(
                found,
                "Move {} ({}) at step {} is not in legal move list!",
                idx,
                Format::encode(mv),
                idx
            );

            pos.apply(mv.from, mv.to);
            idx += 1;
        }

        assert_eq!(
            pos.hash,
            result.history[result.history.len() - 1],
            "Final position hash must match last item in match history!"
        );
    }

    /// Test match limit outcome when limit reached
    #[test]
    fn limits() {
        let config = Config::new(1, 10, 15);
        let result = Runner::play(&config);

        if result.moves.len() == 15 {
            assert_eq!(
                result.outcome,
                Outcome::Limit,
                "Match reaching limit count must yield Outcome::Limit"
            );
        }
    }

    /// Test 3-fold repetition detection logic
    #[test]
    fn loopback() {
        let mut pos = Parser::parse(Parser::DEFAULT);
        let mut item = Match::new(20);
        item.history.push(pos.hash);

        // Knight back and forth simulation
        let m1 = Move::new(1, 18); // Red Knight b1-a3
        let m2 = Move::new(79, 62); // Black Knight b10-a8
        let m3 = Move::new(18, 1); // Red Knight a3-b1
        let m4 = Move::new(62, 79); // Black Knight a8-b10

        let mut outcome = Outcome::Draw;

        let mut turn = 0;
        while turn < 3 {
            pos.apply(m1.from, m1.to);
            item.history.push(pos.hash);
            item.moves.push(m1);

            pos.apply(m2.from, m2.to);
            item.history.push(pos.hash);
            item.moves.push(m2);

            pos.apply(m3.from, m3.to);
            item.history.push(pos.hash);
            item.moves.push(m3);

            pos.apply(m4.from, m4.to);
            item.history.push(pos.hash);
            item.moves.push(m4);

            let curr = pos.hash;
            let mut count = 0;
            let mut h = 0;
            while h < item.history.len() {
                if item.history[h] == curr {
                    count += 1;
                }
                h += 1;
            }

            if count >= 3 {
                outcome = Outcome::Loop;
                break;
            }

            turn += 1;
        }

        assert_eq!(outcome, Outcome::Loop, "Repetition must yield Outcome::Loop");
    }

    /// Test FEN export and roundtrip parsing
    #[test]
    fn fen() {
        let pos = Parser::parse(Parser::DEFAULT);
        let text = Fen::export(&pos);
        assert_eq!(text, Parser::DEFAULT, "Default position FEN export must equal DEFAULT string");

        let parsed = Parser::parse(&text);
        assert_eq!(parsed.hash, pos.hash, "Parsed FEN position hash must match original!");
    }

    /// Test PGN export header and outcome string formatting
    #[test]
    fn pgn() {
        let mut item = Match::new(10);
        item.moves.push(Move::new(19, 28)); // Red Cannon c2-c5
        item.moves.push(Move::new(64, 55)); // Black Cannon c7-c4
        item.outcome = Outcome::Win(Side::Red);
        item.stats.time = 200;
        item.stats.moves = 2;

        let text = Pgn::export(&item);

        assert!(text.contains("[Event \"Self-Play Match\"]"));
        assert!(text.contains("[Result \"1-0\"]"));
        assert!(text.contains("1. b2b3 b7b6"));

        // Black win result check
        item.outcome = Outcome::Win(Side::Black);
        let text_black = Pgn::export(&item);
        assert!(text_black.contains("[Result \"0-1\"]"));

        // Draw result check
        item.outcome = Outcome::Draw;
        let text_draw = Pgn::export(&item);
        assert!(text_draw.contains("[Result \"1/2-1/2\"]"));

        // Loop result check
        item.outcome = Outcome::Loop;
        let text_loop = Pgn::export(&item);
        assert!(text_loop.contains("[Result \"1/2-1/2\"]"));

        // Limit result check
        item.outcome = Outcome::Limit;
        let text_limit = Pgn::export(&item);
        assert!(text_limit.contains("[Result \"1/2-1/2\"]"));
    }

    /// Test Stats calculations correctness
    #[test]
    fn stats() {
        let mut stats = Stats::new();
        stats.nodes = 50000;
        stats.time = 250;
        stats.moves = 5;

        assert_eq!(stats.rate(), 200000, "NPS should be 50,000 * 1000 / 250 = 200,000");
        assert_eq!(stats.mean(), 10000, "Mean nodes/move should be 50,000 / 5 = 10,000");
        assert_eq!(stats.span(), 50, "Time span per move should be 250 / 5 = 50ms");
    }

    /// Test Side enum flip and index operations
    #[test]
    fn sides() {
        assert_eq!(Side::Red.flip(), Side::Black);
        assert_eq!(Side::Black.flip(), Side::Red);
        assert_eq!(Side::Red.index(), 0);
        assert_eq!(Side::Black.index(), 1);
    }
}
