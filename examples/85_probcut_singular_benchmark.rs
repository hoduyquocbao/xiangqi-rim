// ============================================================================
// VÍ DỤ 85: BÁO CÁO REALTIME STREAM YIELD ENGINE V7.2.0 (PROBCUT & SINGULAR EXT)
// ============================================================================
// `85_probcut_singular_benchmark.rs` thực thi đo đạc thời gian tìm kiếm từng Ply (1..13),
// số nút duyệt (nodes), NPS thực tế và dung lượng RAM RSS.
// Tuân thủ 100% Quy tắc 8.10 Live Realtime Stream Yield (in dòng theo dòng + io::stdout().flush()).
// ============================================================================

use std::io::{self, Write};
use std::time::Instant;
use xiangrust::board::fen::Validator;
use xiangrust::board::Parser;
use xiangrust::search::Limits;
use xiangrust::thread::Pool;
use xiangrust::uci::Format;

fn main() {
    println!("===============================================================================");
    println!("🏰 XIANGQI-RIM: REALTIME STREAM YIELD BENCHMARK (PROBCUT + SINGULAR EXT)");
    println!("   Engine Version : v7.2.0-probcut-singular-boost");
    println!("   Build Timestamp: 2026-08-13 01:20:00 ICT");
    println!("===============================================================================");
    let _ = io::stdout().flush();

    // Thế cờ Trung cuộc thực sự ngoài Opening Book (0ms Book miss -> kích hoạt PVS Search Engine 100%)
    let fen = "r1ba1ab1r/4k4/1cn1c4/p1p1p1p1p/9/9/P1P1P1P1P/1C2C4/4K4/RNBA1ABNR w - - 0 1";
    println!("🔍 [DEBUG FEN AUDIT]: {:?}", Validator::audit(fen));
    let pos = Parser::parse(fen);
    println!("🔍 [DEBUG POS HASH]: {:x}", pos.hash);
    let threads = 4usize;
    let mb = 256usize;

    println!("\n-------------------------------------------------------------------------------");
    println!("[1] Đang Realtime Live Stream thông số từ Depth 1 đến Depth 13 (4 Physical Cores):");
    println!("-------------------------------------------------------------------------------");
    let _ = io::stdout().flush();

    let pool = Pool::new(threads, mb);
    let start_all = Instant::now();

    for depth in 1u8..=13u8 {
        let mut limits = Limits::new();
        limits.depth = depth;

        let start = Instant::now();
        let res = pool.go(&pos, &limits);
        let elapsed = start.elapsed().as_secs_f64();

        let nps = if elapsed > 0.0 { (res.nodes as f64) / elapsed } else { 0.0 };

        let mut r_usage: libc::rusage = unsafe { std::mem::zeroed() };
        let ram_mb = if unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut r_usage) } == 0 {
            (r_usage.ru_maxrss as f64) / 1024.0
        } else {
            0.0
        };

        let best_str = Format::encode(res.best);
        println!(
            "  🚀 [LIVE PLY {:2}] Time: {:6.3}s | Nodes: {:10} | NPS: {:10.0} | Score: {:5} cp | Best: {:5} | OS RAM RSS: {:.2} MB",
            depth, elapsed, res.nodes, nps, res.score, best_str, ram_mb
        );
        let _ = io::stdout().flush();
    }

    let total_time = start_all.elapsed().as_secs_f64();
    println!("\n===============================================================================");
    println!("🏆 HOÀN THÀNH REALTIME STREAM YIELD BENCHMARK TRONG {:.2}s", total_time);
    println!("===============================================================================");
    let _ = io::stdout().flush();
}

