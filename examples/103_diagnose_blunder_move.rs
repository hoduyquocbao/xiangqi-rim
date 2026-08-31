// ============================================================================
// CHẨN ĐOÁN LỖI TẦM NHÌN: TẠI SAO RIM CHỌN NƯỚC ĐI TẶNG QUÂN (BLUNDER MOVE)
// ============================================================================

use xiangrust::board::Parser;
use xiangrust::search::core::Core;
use xiangrust::search::limit::{Limits, Timer};
use xiangrust::search::order::{History, Killer};
use xiangrust::tt::Table;
use xiangrust::uci::Format;

fn main() {
    println!("===============================================================================");
    println!(" 🔍 CHẨN ĐOÁN CHI TIẾT NƯỚC ĐI BLUNDER: b2e2 h9g7 b0c2 -> i9h9 ???");
    println!("===============================================================================");

    // Khởi tạo bàn cờ sau 3 nước: 1. b2e2 h9g7 2. b0c2
    let mut pos = Parser::parse(Parser::DEFAULT);
    let moves = ["b2e2", "h9g7", "b0c2"];
    for m_str in &moves {
        let mv = Format::decode(m_str);
        pos.apply(mv.from, mv.to);
    }

    println!("♟️ Trạng thái bàn cờ hiện tại:");
    println!("  • Phe đến lượt: {}", if pos.side == 0 { "ĐỎ" } else { "ĐEN" });
    println!("  • Hash: 0x{:016X}", pos.hash);

    let mut eval = xiangrust::eval::Eval::new();
    if std::path::Path::new("data/nnue_weights.bin").exists() {
        let _ = eval.load("data/nnue_weights.bin");
    }
    eval.reset(&pos);

    let tt = Table::new(64);
    let mut history = History::new();
    let mut killer = Killer::new();

    println!("\n[KIỂM TRA TÌM KIẾM ĐỆ QUY TỪNG ĐỘ SÂU]:");
    for d in 1..=13 {
        let mut timer = Timer::new();
        let mut limits = Limits::new();
        limits.depth = d;
        timer.init(&limits, pos.side);

        eval.reset(&pos);

        let (best, score, nodes, completed_depth) = Core::iterate(
            &mut pos,
            &mut eval,
            Some(&tt),
            &mut history,
            &mut killer,
            &timer,
            None,
            None,
        );

        println!(
            "  -> Depth {:2} (hoàn tất {:2}): Best = {} | Score = {:+5} cp | Nodes = {:8}",
            d,
            completed_depth,
            Format::encode(best),
            score,
            nodes
        );
    }
}
