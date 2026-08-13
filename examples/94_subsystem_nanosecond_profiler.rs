// ============================================================================
// VÍ DỤ 94: SUBSYSTEM NANOSECOND BOTTLENECK PROFILER V1.0.0
// VI CHẨN ĐOÁN THỜI GIAN THỰC THI CHO TỪNG MODULE CỐT LÕI TỚI ĐỘ CHÍNH XÁC NANOSECOND
// ============================================================================
// `94_subsystem_nanosecond_profiler.rs` giải phẫu từng mắt xích độc lập:
//   1. FEN Parser vs FEN Serializer Export Bytes
//   2. Move Generator (legal::gen)
//   3. Position Apply & Unapply Move
//   4. Static Evaluation (HCE vs Accumulator)
//   5. Search Alpha-Beta per Depth (Depth 1..5)
// ============================================================================

use std::time::Instant;

use xiangrust::board::{Parser, Serializer};
use xiangrust::movegen::{legal, List};
use xiangrust::search::{Limits, Search};
use xiangrust::uci::Format;

const APP_VERSION: &str = "v1.0.0-nanosecond-profiler";
const TEST_ITERATIONS: usize = 100_000;

fn main() {
    println!("===============================================================================");
    println!("🔬 XIANGQI-RIM: SUBSYSTEM NANOSECOND BOTTLENECK PROFILER ({})", APP_VERSION);
    println!("   🔥 ĐO ĐẠC HIỆU NĂNG THỰC TẾ CHI TIẾT TỚI NANOSECOND ({} ITERATIONS)", TEST_ITERATIONS);
    println!("===============================================================================\n");

    let pos_start = Parser::parse(Parser::DEFAULT);

    // -------------------------------------------------------------------------
    // TEST 1: FEN PARSING & SERIALIZATION BENCHMARK
    // -------------------------------------------------------------------------
    println!("🧪 [TEST 1] PHÂN TÍCH FEN PARSING & SERIALIZATION EXPORT BYTES...");
    let t_start = Instant::now();
    for _ in 0..TEST_ITERATIONS {
        let _p = Parser::parse(Parser::DEFAULT);
    }
    let dur_parse = t_start.elapsed();
    let ns_per_parse = dur_parse.as_nanos() as f64 / TEST_ITERATIONS as f64;

    let mut buf = [0u8; 96];
    let t_start = Instant::now();
    for _ in 0..TEST_ITERATIONS {
        let _len = Serializer::export_bytes(&pos_start, &mut buf);
    }
    let dur_export = t_start.elapsed();
    let ns_per_export = dur_export.as_nanos() as f64 / TEST_ITERATIONS as f64;

    println!("   • FEN Parsing (`Parser::parse`)       : {:.2} ns/op ({:.2} ops/s)", ns_per_parse, 1e9 / ns_per_parse);
    println!("   • FEN Serializer (`export_bytes`)    : {:.2} ns/op ({:.2} ops/s)", ns_per_export, 1e9 / ns_per_export);
    println!("   $\rightarrow$ Serializer nhanh hơn Parse gấp {:.1}x lần!\n", ns_per_parse / ns_per_export);

    // -------------------------------------------------------------------------
    // TEST 2: MOVE GENERATION & ENCODING BENCHMARK
    // -------------------------------------------------------------------------
    println!("🧪 [TEST 2] PHÂN TÍCH MOVE GENERATION (`legal::gen`) & UCI MOVE ENCODING...");
    let mut moves = List::new();
    let mut pos_mut = pos_start.clone();
    let t_start = Instant::now();
    for _ in 0..TEST_ITERATIONS {
        moves.clear();
        legal::gen(&mut pos_mut, &mut moves);
    }
    let dur_gen = t_start.elapsed();
    let ns_per_gen = dur_gen.as_nanos() as f64 / TEST_ITERATIONS as f64;

    let sample_move = moves.items[0];
    let t_start = Instant::now();
    for _ in 0..TEST_ITERATIONS {
        let _b = Format::encode_bytes(sample_move);
    }
    let dur_encode = t_start.elapsed();
    let ns_per_encode = dur_encode.as_nanos() as f64 / TEST_ITERATIONS as f64;

    println!("   • Move Generator (`legal::gen`)      : {:.2} ns/op ({:.2} ops/s | {} nước đi)", ns_per_gen, 1e9 / ns_per_gen, moves.len());
    println!("   • Move Encoder (`encode_bytes`)     : {:.2} ns/op ({:.2} ops/s)", ns_per_encode, 1e9 / ns_per_encode);
    println!("   $\rightarrow$ Tốc độ sinh nước đi: {:.2} triệu nước/giây!\n", (moves.len() as f64 * 1e9 / ns_per_gen) / 1e6);

    // -------------------------------------------------------------------------
    // TEST 3: BOARD APPLY & REVERT MOVE LATENCY
    // -------------------------------------------------------------------------
    println!("🧪 [TEST 3] PHÂN TÍCH BOARD APPLY & REVERT MOVE LATENCY...");
    let mut test_pos = pos_start.clone();
    let t_start = Instant::now();
    for _ in 0..TEST_ITERATIONS {
        let state = test_pos.apply(sample_move.from, sample_move.to);
        test_pos.revert(sample_move.from, sample_move.to, &state);
    }
    let dur_apply = t_start.elapsed();
    let ns_per_apply = dur_apply.as_nanos() as f64 / TEST_ITERATIONS as f64;

    println!("   • Apply + Revert Move                : {:.2} ns/op ({:.2} ops/s)\n", ns_per_apply, 1e9 / ns_per_apply);

    // -------------------------------------------------------------------------
    // TEST 4: SEARCH LATENCY BENCHMARK PER DEPTH (DEPTH 1..5)
    // -------------------------------------------------------------------------
    println!("🧪 [TEST 4] PHÂN TÍCH THỜI GIAN SEARCH THEO TỪNG ĐỘ SÂU (DEPTH 1..5)...");
    let mut engine = Search::new(256);

    for d in 1..=5 {
        let mut limits = Limits::new();
        limits.depth = d;

        let iterations = match d {
            1 => 10_000,
            2 => 5_000,
            3 => 1_000,
            4 => 500,
            5 => 100,
            _ => 10,
        };

        let t_start = Instant::now();
        let mut total_nodes = 0u64;

        for _ in 0..iterations {
            let res = engine.go(&pos_start, &limits);
            total_nodes += res.nodes as u64;
        }
        let dur_search = t_start.elapsed();
        let ms_per_search = dur_search.as_secs_f64() * 1000.0 / iterations as f64;
        let avg_nodes = total_nodes / iterations as u64;
        let nps = if dur_search.as_secs_f64() > 0.0 {
            total_nodes as f64 / dur_search.as_secs_f64()
        } else {
            0.0
        };

        println!(
            "   • Depth {} Search : {:>8.3} ms/search | {:>5} nodes/search | NPS: {:>10.0} NPS",
            d, ms_per_search, avg_nodes, nps
        );
    }

    println!("\n===============================================================================");
    println!("🟢 HOÀN THÀNH TOÀN BỘ VI CHẨN ĐOÁN NANOSECOND");
    println!("===============================================================================");
}
