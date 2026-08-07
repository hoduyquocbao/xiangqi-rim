// Empirical verification benchmark for Cache Coherency and Atomic Line Bouncing (Milestone 16)

use std::sync::Arc;
use std::time::Instant;
use std::thread;

use xiangrust::movegen::types::Move;
use xiangrust::tt::table::Table;

#[test]
fn test_cache_bouncing_scaling() {
    let table = Arc::new(Table::new(16)); // 16 MB TT Table
    let ops_per_thread = 1_000_000usize;

    println!("\n=== EMPIRICAL ATOMIC TT THROUGHPUT AND SCALING BENCHMARK ===");

    for &thread_count in &[1, 2, 4, 8, 16] {
        let start = Instant::now();
        let mut handles = Vec::with_capacity(thread_count);

        for tid in 0..thread_count {
            let tt = table.clone();
            let handle = thread::spawn(move || {
                let mut state = (tid as u64).wrapping_add(1).wrapping_mul(0x9E3779B97F4A7C15);
                for _ in 0..ops_per_thread {
                    state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                    let key = if state == 0 { 1 } else { state };
                    let from = ((state >> 16) % 90) as u8;
                    let to = ((state >> 24) % 90) as u8;
                    let step = Move::new(from, to);
                    let score = (state as i16) % 10000;
                    let depth = ((state >> 32) % 32) as u8 + 1;
                    let bound = ((state >> 40) % 3 + 1) as u8;

                    tt.save(key, depth, bound, step, score);
                    let _ = tt.probe(key);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let elapsed = start.elapsed();
        let total_ops = (thread_count * ops_per_thread * 2) as f64; // save + probe = 2 ops
        let mops = (total_ops / elapsed.as_secs_f64()) / 1_000_000.0;
        let ns_per_op = (elapsed.as_nanos() as f64) / total_ops;

        println!(
            "Threads: {:2} | Total Ops: {:10} | Time: {:8.3?} | Throughput: {:7.2} MOPS | Latency: {:6.2} ns/op",
            thread_count,
            thread_count * ops_per_thread * 2,
            elapsed,
            mops,
            ns_per_op
        );
    }
}
