// ============================================================================
// EMPIRICAL CHALLENGER GEN2_2: SELF-PLAY ENGINE HIGH-LOAD & STRESS TEST HARNESS
// ============================================================================

use std::mem::{align_of, size_of};
use std::time::Instant;

use xiangrust::board::Parser;
use xiangrust::selfplay::{Config, Fen, Match, Outcome, Pgn, Runner, Side, Stats};

#[cfg(test)]
mod tests {
    use super::*;

    /// Kiểm thử nghiêm ngặt căn lề bộ nhớ và dung lượng vật lý theo yêu cầu đặc tả
    #[test]
    fn alignments() {
        assert_eq!(align_of::<Outcome>(), 16usize, "Outcome alignment must be 16!");
        assert_eq!(size_of::<Match>(), 128usize, "Match size must be exactly 128!");
        assert_eq!(align_of::<Match>(), 64usize, "Match alignment must be 64!");

        assert_eq!(size_of::<Config>(), 64usize, "Config size must be 64!");
        assert_eq!(align_of::<Config>(), 64usize, "Config alignment must be 64!");

        assert_eq!(size_of::<Runner>(), 64usize, "Runner size must be 64!");
        assert_eq!(align_of::<Runner>(), 64usize, "Runner alignment must be 64!");

        assert_eq!(size_of::<Stats>(), 64usize, "Stats size must be 64!");
        assert_eq!(align_of::<Stats>(), 64usize, "Stats alignment must be 64!");

        assert_eq!(size_of::<Pgn>(), 64usize, "Pgn size must be 64!");
        assert_eq!(align_of::<Pgn>(), 64usize, "Pgn alignment must be 64!");

        assert_eq!(size_of::<Fen>(), 64usize, "Fen size must be 64!");
        assert_eq!(align_of::<Fen>(), 64usize, "Fen alignment must be 64!");
    }

    /// Kiểm thử tải cao và tốc độ tính toán NPS trên hàng loạt ván mô phỏng Self-Play
    #[test]
    fn stress() {
        let config = Config::new(1, 10, 20);
        let start = Instant::now();
        let total = 50usize;

        let mut count = 0usize;
        while count < total {
            let result = Runner::play(&config);
            assert!(result.moves.len() <= 20, "Moves count must not exceed limit");
            assert!(result.history.len() > 0, "History must not be empty");
            count += 1;
        }

        let elapsed = start.elapsed();
        println!("Completed {} self-play matches in {:?}", total, elapsed);
    }

    /// Kiểm thử đa luồng đồng thời (Concurrency & Thread Safety) chạy Runner::play
    #[test]
    fn concurrency() {
        let mut handles = Vec::new();

        let mut index = 0usize;
        while index < 4 {
            let handle = std::thread::spawn(move || {
                let config = Config::new(1, 10, 10);
                let result = Runner::play(&config);
                assert!(result.history.len() > 0);
            });
            handles.push(handle);
            index += 1;
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    /// Kiểm thử chính xác công thức NPS rate(), mean(), span() trong Stats
    #[test]
    fn rate() {
        let mut stats = Stats::new();
        stats.nodes = 1_000_000;
        stats.time = 500;
        stats.moves = 20;

        let rate = stats.rate();
        assert_eq!(rate, 2_000_000, "1M nodes in 500ms must yield 2M NPS");
        assert_eq!(stats.mean(), 50_000, "1M nodes / 20 moves must yield 50k nodes/move");
        assert_eq!(stats.span(), 25, "500ms / 20 moves must yield 25ms/move");

        // Edge case division by zero
        let mut empty = Stats::new();
        assert_eq!(empty.rate(), 0);
        assert_eq!(empty.mean(), 0);
        assert_eq!(empty.span(), 0);
    }

    /// Kiểm thử Side index và flip logic
    #[test]
    fn sides() {
        assert_eq!(Side::Red.flip(), Side::Black);
        assert_eq!(Side::Black.flip(), Side::Red);
        assert_eq!(Side::Red.index(), 0);
        assert_eq!(Side::Black.index(), 1);
    }

    /// Kiểm thử xuất FEN và PGN
    #[test]
    fn formatters() {
        let pos = Parser::parse(Parser::DEFAULT);
        let fen_text = Fen::export(&pos);
        assert_eq!(fen_text, Parser::DEFAULT);

        let mut item = Match::new(10);
        item.moves.push(xiangrust::movegen::Move::new(19, 28));
        item.moves.push(xiangrust::movegen::Move::new(64, 55));
        item.outcome = Outcome::Win(Side::Red);
        item.stats.time = 100;
        item.stats.moves = 2;

        let pgn_text = Pgn::export(&item);
        assert!(pgn_text.contains("[Result \"1-0\"]"));
        assert!(pgn_text.contains("1. b2b3 b7b6"));
    }
}
