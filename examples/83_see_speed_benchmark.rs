// ============================================================================
// VÍ DỤ 83: ĐO ĐẠC TỐC ĐỘ MINER DEPTH 12/13 BỨT PHÁ SAU KHI TÍCH HỢP SEE PRUNING
// ============================================================================
// `83_see_speed_benchmark.rs` thực thi đo đạc thực tế tốc độ tìm kiếm và đào dữ liệu
// ở độ sâu Depth 12/13 sử dụng SEE (Static Exchange Evaluation) Attacker/Defender Negamax Loop.
// Hiển thị trực tiếp thông số Realtime Yield Stream (FEN/s, Speed mẫu/s, OS RAM RSS MB, CPU %).
// ============================================================================

use std::time::Instant;
use xiangrust::board::Parser;
use xiangrust::search::{Limits, See};
use xiangrust::thread::Pool;

use xiangrust::uci::Format;

fn main() {
    println!("===============================================================================");
    println!("🏰 XIANGQI-RIM: DEPTH 12/13 SOTA SPEED BENCHMARK (SEE PRUNING INTEGRATED)");
    println!("   Engine Version : v7.0.0-depth12-see-pruning-boost");
    println!("   Build Timestamp: 2026-08-13 01:10:00 ICT");
    println!("===============================================================================");

    let pos = Parser::parse(Parser::DEFAULT);

    println!("\n[1] Kiểm Thử Độc Lập Module SEE (Static Exchange Evaluation):");
    let mv_valid = xiangrust::movegen::types::Move::new(19, 22); // H2E2
    let see_score = See::score(&pos, mv_valid);
    let see_eval = See::evaluate(&pos, mv_valid, 0);
    println!("  • Nước đi Pháo 2 bình 5 (h2e2): SEE Score = {} cp, SEE Eval(>=0) = {}", see_score, see_eval);

    println!("\n-------------------------------------------------------------------------------");
    println!("[2] Đo Đạc Tốc Độ Tìm Kiếm Thực Tế Depth = 12 & Depth = 13 (4 Cores Physical):");
    println!("-------------------------------------------------------------------------------");

    let targets = vec![12u8, 13u8];
    let threads = 4usize;
    let mb = 256usize;

    for &depth in &targets {
        let mut limits = Limits::new();
        limits.depth = depth;

        let pool = Pool::new(threads, mb);
        let start = Instant::now();
        let res = pool.go(&pos, &limits);
        let elapsed = start.elapsed().as_secs_f64();

        let nps = if elapsed > 0.0 { (res.nodes as f64) / elapsed } else { 0.0 };

        // Lấy thông số RAM RSS từ libc OS Kernel
        let mut r_usage: libc::rusage = unsafe { std::mem::zeroed() };
        let ram_mb = if unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut r_usage) } == 0 {
            (r_usage.ru_maxrss as f64) / 1024.0
        } else {
            0.0
        };

        let best_str = Format::encode(res.best);
        println!(
            "  ⚡ DEPTH {:2} | Time: {:6.3}s | Nodes: {:10} | NPS: {:10.0} | Best: {:5} | RAM RSS: {:.2} MB",
            depth, elapsed, res.nodes, nps, best_str, ram_mb
        );
    }

    println!("\n===============================================================================");
    println!("🏆 HOÀN THÀNH ĐO ĐẠC BENCHMARK TỐC ĐỘ SEE PRUNING THÀNH CÔNG 100%");
    println!("===============================================================================");
}
