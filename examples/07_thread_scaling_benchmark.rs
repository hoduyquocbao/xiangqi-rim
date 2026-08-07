// ============================================================================
// VÍ DỤ 07: BÁO CÁO HIỆU NĂNG TÌM KIẾM ĐA LUỒNG DEPTH = 12
// ============================================================================
// Đo kiểm chính xác thời gian AI tìm kiếm nước đi ở độ sâu depth = 12
// trên các mức luồng 1, 4, 8, 16 threads.
// ============================================================================

use xiangrust::board::Parser;
use xiangrust::search::Limits;
use xiangrust::thread::Pool;

/// Struct `Record` lưu trữ dữ liệu báo cáo hiệu năng của một mức luồng.
#[repr(C, align(64))]
#[derive(Clone, Copy)]
pub struct Record {
    pub threads: usize,
    pub time: u64,
    pub nodes: u64,
    pub nps: u64,
    pub speedup: f64,
    pub efficiency: f64,
    pub pad: [u8; 16],
}

impl Record {
    #[inline(always)]
    pub const fn new(threads: usize) -> Self {
        Self {
            threads,
            time: 0,
            nodes: 0,
            nps: 0,
            speedup: 0.0,
            efficiency: 0.0,
            pad: [0u8; 16],
        }
    }
}

#[inline(always)]
fn run(threads: usize, depth: u8, mb: usize) -> Record {
    let pos = Parser::parse(Parser::DEFAULT);
    let mut limits = Limits::new();
    limits.depth = depth;

    let pool = Pool::new(threads, mb);
    let res = pool.go(&pos, &limits);

    let ms = res.time.max(1);
    let nps = (res.nodes * 1000) / ms;

    let mut rec = Record::new(threads);
    rec.time = ms;
    rec.nodes = res.nodes;
    rec.nps = nps;

    rec
}

fn main() {
    println!("===============================================================================");
    println!("  XIANGRUST AI ENGINE - THỬ NGHIỆM ĐO THỜI GIAN THỰC TẾ DEPTH = 12 ");
    println!("===============================================================================");

    let targets = vec![1usize, 4usize, 8usize, 16usize];
    let mb = 64usize;
    let depth = 12u8;

    println!("\n[1] Môi trường thử nghiệm AI Search Depth = 12:");
    println!(" -> Transposition Table (TT Sharded): {} MB", mb);
    println!(" -> Độ sâu mục tiêu (Target Depth): {}", depth);

    let mut records = Vec::with_capacity(targets.len());
    let mut base = 0u64;

    println!("\n-------------------------------------------------------------------------------");
    println!("[2] Đang chạy AI Engine sinh nước đi ở Depth = 12...");
    println!("-------------------------------------------------------------------------------");

    for &threads in &targets {
        let mut rec = run(threads, depth, mb);

        if threads == 1 || base == 0 {
            base = rec.nps;
            rec.speedup = 1.0;
            rec.efficiency = 100.0;
        } else {
            rec.speedup = (rec.nps as f64) / (base as f64);
            rec.efficiency = (rec.speedup / (threads as f64)) * 100.0;
        }

        println!(
            " -> [{:2} Threads] Time: {:6} ms ({:.2} s) | Nodes: {:10} | NPS: {:10} | Speedup: {:5.2}x",
            rec.threads, rec.time, (rec.time as f64) / 1000.0, rec.nodes, rec.nps, rec.speedup
        );

        records.push(rec);
    }

    println!("\n===============================================================================");
    println!("               BÁO CÁO THỜI GIAN AI SINH NƯỚC ĐỊ DEPTH = 12                    ");
    println!("===============================================================================");
    println!(
        " {:<8} | {:<14} | {:<14} | {:<16} | {:<10}",
        "Threads", "Time (sec)", "Total Nodes", "Speed (NPS)", "Speedup"
    );
    println!("-------------------------------------------------------------------------------");

    for rec in &records {
        println!(
            " {:<8} | {:<14.2} | {:<14} | {:<16} | {:<10.2}x",
            rec.threads,
            (rec.time as f64) / 1000.0,
            rec.nodes,
            rec.nps,
            rec.speedup
        );
    }

    println!("===============================================================================\n");
}
