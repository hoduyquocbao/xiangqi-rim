// ============================================================================
// EMPIRICAL CHALLENGER 2: SELF-PLAY ENGINE HIGH-LOAD & STRESS HARNESS
// ============================================================================

use std::mem::{align_of, size_of};

use xiangrust::selfplay::{Config, Fen, Match, Pgn, Runner, Stats};

#[cfg(test)]
mod layout {
    use super::*;

    #[test]
    fn alignments() {
        assert_eq!(align_of::<Config>(), 64, "Config alignment must be 64");
        assert_eq!(size_of::<Config>(), 64, "Config size must be 64");

        assert_eq!(align_of::<Runner>(), 64, "Runner alignment must be 64");
        assert_eq!(size_of::<Runner>(), 64, "Runner size must be 64");

        assert_eq!(align_of::<Stats>(), 64, "Stats alignment must be 64");
        assert_eq!(size_of::<Stats>(), 64, "Stats size must be 64");

        assert_eq!(align_of::<Pgn>(), 64, "Pgn alignment must be 64");
        assert_eq!(size_of::<Pgn>(), 64, "Pgn size must be 64");

        assert_eq!(align_of::<Fen>(), 64, "Fen alignment must be 64");
        assert_eq!(size_of::<Fen>(), 64, "Fen size must be 64");

        assert_eq!(align_of::<Match>(), 64, "Match alignment must be 64");
        assert_eq!(size_of::<Match>() % 64, 0, "Match size must be a multiple of 64");

        println!("Match size_of = {}", size_of::<Match>());
        println!("Search struct size_of = {}", size_of::<xiangrust::search::Search>());
        println!("Eval struct size_of = {}", size_of::<xiangrust::eval::Eval>());
    }
}
