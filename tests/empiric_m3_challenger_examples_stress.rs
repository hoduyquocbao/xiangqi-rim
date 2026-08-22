// ============================================================================
// ADVERSARIAL STRESS & CORRECTNESS HARNESS: MILESTONE 3 EXAMPLES
// ============================================================================
// Harness kiểm thử đối kháng empirical cho Ví dụ 12 (Self-Play Simulation)
// và Ví dụ 13 (Opening & Endgame Book).
// Thẩm định an toàn bộ nhớ, 0-panic, 0-crash, và tính chính xác của các API.
// ============================================================================

use xiangrust::board::Parser;
use xiangrust::book::endgame::{DRAW, LOSS, WIN};
use xiangrust::book::opening::ENTRIES;
use xiangrust::book::{Book, Endgame};
use xiangrust::selfplay::{Config, Fen, Outcome, Pgn, Runner, Side, Stats};
use xiangrust::uci::format::Format;

#[test]
fn test_example_12_self_play_simulation_correctness() {
    let depth = 2u8;
    let time = 50u64;
    let limit = 10u32;

    let config = Config::new(depth, time, limit);
    assert_eq!(config.depth, 2);
    assert_eq!(config.time, 50);
    assert_eq!(config.limit, 10);

    let game = Runner::play(&config);

    // 1. Thống kê số nước đi không vượt quá giới hạn
    assert!(game.moves.len() <= limit as usize);
    assert!(game.stats.moves <= limit);
    assert!(!game.history.is_empty());

    // 2. Kiểm tra chỉ số hiệu năng Stats
    let stats: Stats = game.stats;
    assert!(stats.nodes > 0, "Tổng số nút duyệt phải > 0");
    assert!(stats.mean() > 0, "Trung bình số nút/nước phải > 0");

    // 3. Phân tích kết quả Outcome
    match game.outcome {
        Outcome::Win(Side::Red) => {}
        Outcome::Win(Side::Black) => {}
        Outcome::Draw => {}
        Outcome::Loop => {}
        Outcome::Limit => {}
    }

    // 4. Reconstruct position & Export FEN
    let mut pos = Parser::parse(Parser::DEFAULT);
    for mv in &game.moves {
        pos.apply(mv.from, mv.to);
    }
    let fen = Fen::export(&pos);
    assert!(!fen.is_empty(), "Chuỗi FEN không được rỗng");
    assert!(fen.contains(' '), "Chuỗi FEN phải có các trường phân tách bằng khoảng trắng");

    // 5. Export PGN
    let pgn = Pgn::export(&game);
    assert!(pgn.contains("[Event \"Self-Play Match\"]"));
    assert!(pgn.contains("[Site \"Local Engine\"]"));
    assert!(pgn.contains("[Result "));
}

#[test]
fn test_example_13_opening_and_endgame_book_correctness() {
    Endgame::clear();
    // 1. Opening Book Entry Count
    let book = Book::default();
    assert!(book.count >= 1000, "Opening book phải chứa >= 1,000 bản ghi");

    // 2. Probe initial position
    let pos = Parser::parse(Parser::DEFAULT);
    let probe = Book::probe(&pos);
    if let Some(mv) = probe {
        assert!(mv.valid(), "Nước đi khai cuộc gợi ý phải hợp lệ");
        let code = Format::encode(mv);
        assert_eq!(code.len(), 4, "Mã UCI nước đi khai cuộc phải đúng 4 ký tự");
    }

    // 3. Probe direct Zobrist Hash
    let target = ENTRIES[100];
    let probe_hash = Book::find_hash(&ENTRIES, target.hash);
    assert!(probe_hash.is_some(), "Tra cứu băm Zobrist bản ghi 100 phải thành công");
    let mv_hash = probe_hash.unwrap();
    assert_eq!(mv_hash.raw(), target.mv, "Mã nước đi tra cứu phải trùng với target.mv");

    // 4. Endgame Book Evaluation
    let endgame = Endgame::new();
    assert_eq!(endgame.total, 10, "Bộ tri thức tàn cuộc chứa đúng 10 quy tắc");

    // Case 1: Bare Kings (Draw)
    let pos_bare = Parser::parse("4k4/9/9/9/9/9/9/9/9/4K4 w - - 0 1");
    assert_eq!(Endgame::eval(&pos_bare), Some(DRAW));

    // Case 2: Single Knight vs Single Advisor (Red Win)
    let pos_knight = Parser::parse("3k1a3/9/9/9/9/9/9/4N4/9/4K4 w - - 0 1");
    assert_eq!(Endgame::eval(&pos_knight), Some(WIN));

    // Case 3: Rook + Cannon vs Single Rook (Red Win)
    let pos_rc = Parser::parse("3k5/4r4/9/9/9/9/9/9/4C4/4K1R2 w - - 0 1");
    assert_eq!(Endgame::eval(&pos_rc), Some(WIN));

    // Case 4: Single Cannon vs Single Advisor (Draw)
    let pos_cannon = Parser::parse("3k1a3/9/9/9/9/9/9/9/4C4/4K4 w - - 0 1");
    assert_eq!(Endgame::eval(&pos_cannon), Some(DRAW));

    // Case 5: Double Cannons vs Incomplete Advisors/Bishops (Red Win)
    let pos_2c = Parser::parse("3k1a3/9/9/9/9/9/9/9/4C1C2/4K4 w - - 0 1");
    assert_eq!(Endgame::eval(&pos_2c), Some(WIN));

    // Case 6: Single Knight vs Single Advisor - Black Turn (Black Loss)
    let pos_black_knight = Parser::parse("3k1a3/9/9/9/9/9/9/4N4/9/4K4 b - - 0 1");
    assert_eq!(Endgame::eval(&pos_black_knight), Some(LOSS));
}
