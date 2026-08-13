// ============================================================================
// VÍ DỤ 91: ENGINE RE-EVALUATOR & UPGRADER DỮ LIỆU CŨ SANG FULL AUTHENTIC DEPTH 8
// ============================================================================
// `91_upgrade_legacy_fen_depth8.rs` đọc các tệp JSONL lịch sử, lấy từng thế cờ FEN,
// chạy lại tìm kiếm Full Authentic Depth 8 (cấm ngắt timer), và ghi bổ sung 5 trường
// Anti-Poisoning Schema (nodes, time_ms, nps, zobrist, engine) chuẩn hóa 100%!
// ============================================================================

use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use xiangrust::board::{Parser, Serializer};
use xiangrust::book::Book;
use xiangrust::gpu::{Device, Evaluator, RingBuffer, Sample};
use xiangrust::learn::replay::Sample as ReplaySample;
use xiangrust::learn::Shard;
use xiangrust::learn::store::Store as LearnStore;
use xiangrust::search::Limits;
use xiangrust::thread::Pool;
use xiangrust::uci::Format;

/// Hằng số phiên bản ứng dụng APP_VERSION
pub const APP_VERSION: &str = "v10.0.0-sota-platinum-upgrader-depth8";
/// Hằng số dấu thời gian đóng gói APP_BUILD_STAMP
pub const APP_BUILD_STAMP: &str = "2026-08-13 04:40:00 ICT";
/// Đường dẫn tệp nhị phân lưu giữ bộ nhớ kinh nghiệm vĩnh cửu (.agents/memory/experience_store.bin)
pub const ETERNAL_STORE_FILE: &str = ".agents/memory/experience_store.bin";

fn extract_fen(line: &str) -> Option<String> {
    if let Some(start) = line.find("\"fen\":\"") {
        let rest = &line[start + 7..];
        if let Some(end) = rest.find('\"') {
            return Some(rest[..end].to_string());
        }
    }
    if line.contains('/') && line.split_whitespace().count() >= 2 {
        return Some(line.trim().to_string());
    }
    None
}

fn main() {
    println!("===============================================================================");
    println!("💎 XIANGQI-RIM: LEGACY DATA RE-EVALUATOR & UPGRADER TO AUTHENTIC DEPTH 8");
    println!("   Engine Version : {}", APP_VERSION);
    println!("   Build Timestamp: {}", APP_BUILD_STAMP);
    println!("===============================================================================");
    let _ = io::stdout().flush();

    let input_path: String = std::env::var("INPUT").unwrap_or_else(|_| "data/selfplay_samples_gen6_depth4.jsonl".to_string());
    let output_path: String = std::env::var("OUTPUT").unwrap_or_else(|_| "data/upgraded_depth8_samples.jsonl".to_string());
    let target_depth: u8 = std::env::var("DEPTH").ok().and_then(|v| v.parse().ok()).unwrap_or(8);
    let threads: usize = std::env::var("THREADS").ok().and_then(|v| v.parse().ok()).unwrap_or(4);
    let tt_mb: usize = std::env::var("TT_MB").ok().and_then(|v| v.parse().ok()).unwrap_or(1024);
    let batch_size: usize = std::env::var("BATCH_SIZE").ok().and_then(|v| v.parse().ok()).unwrap_or(256);

    let input_file = match File::open(&input_path) {
        Ok(f) => f,
        Err(e) => {
            println!("❌ Không thể mở tệp đầu vào `{}`: {}", input_path, e);
            return;
        }
    };

    let device = Device::init();
    println!("\n⚡ THÔNG SỐ NÂNG CẤP DỮ LIỆU CŨ CỤC BỘ:");
    println!("   • Tệp đầu vào (Input)      : {}", input_path);
    println!("   • Tệp đầu ra (Output)     : {}", output_path);
    println!("   • Mục tiêu độ sâu (Depth) : Depth {}", target_depth);
    println!("   • Dung lượng Shared TT    : {} MB RAM", tt_mb);
    println!("   • GPU Hardware Adapter   : {}", device.adapter_name());
    println!("-------------------------------------------------------------------------------\n");
    let _ = io::stdout().flush();

    let pool = Pool::new(threads, tt_mb);
    let evaluator = Arc::new(Evaluator::new(device).expect("Khởi tạo GPU Evaluator thất bại"));
    let total_upgraded = Arc::new(AtomicUsize::new(0));

    // 🌟 CƠ CHẾ SMART CHECKPOINT RESUME: Đếm số mẫu đã nâng cấp trước đó để tự động resume
    let mut skipped_lines = 0usize;
    if let Ok(existing_file) = File::open(&output_path) {
        let existing_reader = BufReader::new(existing_file);
        skipped_lines = existing_reader.lines().count();
    }
    if skipped_lines > 0 {
        println!("✔ [SMART CHECKPOINT RESUME] Đã phát hiện {} mẫu đã nâng cấp trước đó trong `{}`! Tự động nhảy qua (skip) và tiếp tục từ mẫu #{}.", skipped_lines, output_path, skipped_lines + 1);
        let _ = io::stdout().flush();
    }

    let mut out_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&output_path)
        .expect("Không thể mở tệp xuất dữ liệu JSONL");

    let reader = BufReader::new(input_file);
    let start_all = Instant::now();

    let mut pending_replays: Vec<ReplaySample> = Vec::with_capacity(256);
    let mut pending_shards: Vec<(u64, u16, i16)> = Vec::with_capacity(256);
    let mut pending_books: Vec<(u64, u16, u16)> = Vec::with_capacity(256);

    for (idx, line_res) in reader.lines().enumerate() {
        if idx < skipped_lines {
            continue; // Nhảy qua các mẫu đã hoàn tất trước đó (Zero Wasted Compute)
        }
        let line = match line_res {
            Ok(l) => l,
            Err(_) => continue,
        };

        let fen = match extract_fen(&line) {
            Some(f) => f,
            None => continue,
        };

        let pos = Parser::parse(&fen);

        let mut limits = Limits::new();
        limits.depth = target_depth;
        limits.nodes = 0; // Full Authentic Search
        limits.exact = 0; // Full Authentic Search

        let move_start = Instant::now();

        if let Ok(mut queue) = RingBuffer::allocate(evaluator.device(), batch_size) {
            let sample = Sample::pack(&pos, 1);
            let _ = queue.push(&sample);
            let _ = queue.flush_gpu(&evaluator);
        }

        let res = pool.go(&pos, &limits);
        let move_elapsed = move_start.elapsed().as_secs_f64();

        if !res.best.valid() {
            continue;
        }

        let fen_str = Serializer::export(&pos);
        let move_str = Format::encode(res.best);

        if !fen_str.is_empty() && move_str.len() == 4 && res.score.abs() <= 30000 {
            let time_ms = (move_elapsed * 1000.0) as u64;
            let nps = if move_elapsed > 0.0 { (res.nodes as f64 / move_elapsed) as u64 } else { 0 };

            let out_line = format!(
                "{{\"fen\":\"{}\",\"best_move\":\"{}\",\"score\":{},\"depth\":{},\"nodes\":{},\"time_ms\":{},\"nps\":{},\"zobrist\":\"0x{:016X}\",\"engine\":\"{}\"}}\n",
                fen_str, move_str, res.score, res.depth, res.nodes, time_ms, nps, pos.hash, APP_VERSION
            );
            let _ = out_file.write_all(out_line.as_bytes());
            let _ = out_file.flush();
            total_upgraded.fetch_add(1, Ordering::Relaxed);

            let reward = (res.score as f32 / 1000.0).clamp(-1.0, 1.0);
            let mv_code = ((res.best.from as u16) << 8) | (res.best.to as u16);
            pending_replays.push(ReplaySample::new(pos.hash, mv_code, reward, 0, 0));
            pending_shards.push((pos.hash, mv_code, res.score as i16));
            if res.score > 50 {
                pending_books.push((pos.hash, mv_code, res.score.min(32767) as u16));
            }

            if idx % 10 == 0 || idx < 5 {
                println!(
                    " 🚀 [UPGRADE MẪU #{:<5}] FEN: {} | Move: {} | Score: {:<6} | TrueDepth: {} | Nodes: {:<8} | Time: {:.3}s",
                    idx + 1, &fen_str[..25], move_str, res.score, res.depth, res.nodes, move_elapsed
                );
                let _ = io::stdout().flush();
            }
        }

        if pending_replays.len() >= 200 {
            let shard = Shard::new("data/shards_10b");
            for r_sample in &pending_replays {
                let _ = LearnStore::append_sample(r_sample, ETERNAL_STORE_FILE);
            }
            for (h, mv, s) in pending_shards.drain(..) {
                let _ = shard.save(h, mv, s);
            }
            for (h, mv, s) in pending_books.drain(..) {
                Book::sync(h, mv, s);
            }
            pending_replays.clear();
        }
    }

    if !pending_replays.is_empty() {
        let shard = Shard::new("data/shards_10b");
        for r_sample in &pending_replays {
            let _ = LearnStore::append_sample(r_sample, ETERNAL_STORE_FILE);
        }
        for (h, mv, s) in pending_shards {
            let _ = shard.save(h, mv, s);
        }
        for (h, mv, s) in pending_books {
            Book::sync(h, mv, s);
        }
    }

    let total_elapsed = start_all.elapsed().as_secs_f64();
    let count = total_upgraded.load(Ordering::Relaxed);
    println!("\n===============================================================================");
    println!("🎉 NÂNG CẤP DỮ LIỆU CŨ THÀNH CÔNG:");
    println!("   • Số lượng mẫu đã nâng cấp Depth 8  : {} mẫu FEN", count);
    println!("   • Tổng thời gian thực thi            : {:.2} giây", total_elapsed);
    println!("   • Tệp đầu ra đã tích hợp Schema     : {}", output_path);
    println!("===============================================================================");
    let _ = io::stdout().flush();
}
