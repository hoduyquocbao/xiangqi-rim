// ============================================================================
// VÍ DỤ 120: CÔNG CỤ PHÂN TÍCH & GIẢI MÃ TRI THỨC NHỊ PHÂN BITWISE XRKB (INSPECTOR)
// ============================================================================
// Công cụ phân tích dành cho chuyên gia / con người:
// 1. Đọc và giải mã trực tiếp tệp nhị phân nén bitwise 64-byte `data/harvest_knowledge.xrk`.
// 2. Vẽ bàn cờ Unicode trực quan (帥, 仕, 相, 傌, 俥, 砲, 兵, 將, 士, 象, 馬, 車, 砲, 卒).
// 3. Thống kê toàn diện: Tổng số ván cờ, số thế cờ, tỷ lệ thắng/hòa/thua, phân bố điểm số.
// 4. Phân tích bẫy / bước ngoặt / nước đi sai lầm (Blunder & Turning Points Analysis).
// 5. Xuất sang định dạng JSONL hoặc FEN cho các mô hình AI / LLM bên ngoài khi cần.
//
// 100% chú thích tiếng Việt từng dòng & 100% định danh từ đơn tiếng Anh.
// ============================================================================

use std::env;
use std::time::Instant;
use xiangrust::learn::Archive;

fn main() {
    let args: Vec<String> = env::args().collect();
    let default_path = "data/harvest_knowledge.xrk".to_string();

    let file_path = args.iter()
        .position(|a| a == "--file" || a == "-f")
        .and_then(|i| args.get(i + 1))
        .unwrap_or(&default_path);

    println!("===============================================================================");
    println!(" 🔍 CÔNG CỤ PHÂN TÍCH & GIẢI MÃ TRI THỨC NÉN BITWISE 64-BYTE XRKB");
    println!("    Tệp nhị phân: {}", file_path);
    println!("===============================================================================");

    let start = Instant::now();
    let (header, frames) = match Archive::load(file_path) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("\n❌ Không thể đọc tệp nhị phân `{}`: {}", file_path, e);
            eprintln!("💡 Hãy chạy một giải đấu đối kháng (ví dụ `examples/119`) để tự động tích lũy tri thức!");
            return;
        }
    };
    let load_time = start.elapsed();

    // Kiểm tra chế độ xuất JSONL
    if let Some(pos) = args.iter().position(|a| a == "--export-jsonl") {
        if let Some(out_path) = args.get(pos + 1) {
            println!("\n📦 Đang giải nén {} thế cờ sang JSONL: {}...", frames.len(), out_path);
            match Archive::export_jsonl(file_path, out_path) {
                Ok(n) => println!("✅ Xuất bản thành công {} bản ghi JSONL trong {:.2}ms!", n, start.elapsed().as_secs_f64() * 1000.0),
                Err(e) => eprintln!("❌ Lỗi xuất JSONL: {}", e),
            }
            return;
        }
    }

    // Kiểm tra chế độ xem chi tiết 1 thế cờ
    if let Some(pos) = args.iter().position(|a| a == "--view" || a == "-v") {
        if let Some(idx_str) = args.get(pos + 1) {
            if let Ok(idx) = idx_str.parse::<usize>() {
                if idx < frames.len() {
                    let frame = &frames[idx];
                    println!("\n📋 CHI TIẾT THẾ CỜ [Index: {} / {}]:", idx, frames.len());
                    println!("{}", frame.render());
                    return;
                } else {
                    eprintln!("❌ Chỉ số thế cờ `{}` vượt quá giới hạn (0 .. {})!", idx, frames.len() - 1);
                    return;
                }
            }
        }
    }

    // Kiểm tra chế độ xem thế cờ cuối cùng
    if args.iter().any(|a| a == "--last" || a == "-l") {
        if let Some(frame) = frames.last() {
            println!("\n📋 CHI TIẾT THẾ CỜ CUỐI CÙNG [Index: {}]:", frames.len() - 1);
            println!("{}", frame.render());
            return;
        }
    }

    // Kiểm tra chế độ phân tích sai lầm (Blunders)
    if let Some(pos) = args.iter().position(|a| a == "--blunders" || a == "-b") {
        let threshold = args.get(pos + 1).and_then(|t| t.parse::<i32>().ok()).unwrap_or(200);
        println!("\n⚡ DANH SÁCH BƯỚC NGOẶT & NƯỚC CỜ SAI LẦM (Biến động >= {} cp):", threshold);
        println!("-------------------------------------------------------------------------------");
        let mut blunder_count = 0usize;
        for i in 1..frames.len() {
            let prev = &frames[i - 1];
            let curr = &frames[i];
            let diff = (curr.score - prev.score).abs() as i32;
            if diff >= threshold {
                blunder_count += 1;
                println!(
                    " #{:03} | Frame [{:04} -> {:04}] | Ply: {:02} | Nước: {} ({}) | Điểm: {:+5}cp -> {:+5}cp (Δ {:+5}cp) | {}",
                    blunder_count, i - 1, i, curr.ply, curr.uci(), curr.actor_name(), prev.score, curr.score, diff, curr.outcome_name()
                );
            }
        }
        println!("-------------------------------------------------------------------------------");
        println!("  • Tổng số bước ngoặt phát hiện: {} / {} thế cờ ({:.2}%)\n", blunder_count, frames.len(), (blunder_count as f64 / frames.len().max(1) as f64) * 100.0);
        return;
    }

    // MẶC ĐỊNH: BÁO CÁO TỔNG QUAN HỆ THỐNG TRI THỨC (SUMMARY REPORT)
    let file_size_bytes = std::fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);
    let mut red_wins = 0usize;
    let mut black_wins = 0usize;
    let mut draws = 0usize;
    let mut rim_moves = 0usize;
    let mut pika_moves = 0usize;

    let mut decisive_count = 0usize; // |score| >= 1000cp
    let mut tactical_count = 0usize; // 200cp <= |score| < 1000cp
    let mut balanced_count = 0usize; // |score| < 200cp

    for frame in &frames {
        match frame.outcome {
            1 => red_wins += 1,
            2 => black_wins += 1,
            3 => draws += 1,
            _ => (),
        }
        match frame.actor {
            0 => rim_moves += 1,
            1 => pika_moves += 1,
            _ => (),
        }
        let abs_score = (frame.score as i32).abs();
        if abs_score >= 1000 {
            decisive_count += 1;
        } else if abs_score >= 200 {
            tactical_count += 1;
        } else {
            balanced_count += 1;
        }
    }

    let total = frames.len().max(1);
    println!("\n📊 1. THÔNG SỐ VẬT LÝ & HIỆU NĂNG TỆP TIN:");
    println!("  • Tổng số thế cờ (Frames)  : {} thế cờ", header.count);
    println!("  • Tổng số ván cờ (Games)   : {} ván đấu", header.games);
    println!("  • Dung lượng tệp đĩa       : {:.2} KB ({:.3} MB) [Đúng 64 bytes/thế cờ]", file_size_bytes as f64 / 1024.0, file_size_bytes as f64 / (1024.0 * 1024.0));
    println!("  • Tốc độ đọc nạp RAM       : {:.3} ms (Tốc độ: {:.1} triệu frames/giây)", load_time.as_secs_f64() * 1000.0, (frames.len() as f64 / load_time.as_secs_f64()) / 1_000_000.0);

    println!("\n♟️ 2. PHÂN BỐ CHIẾN THUẬT & ĐIỂM SỐ CENTIPAWN:");
    println!("  • Thế cờ cân bằng (|score| < 200cp)  : {:5} ({:.1}%)", balanced_count, (balanced_count as f64 / total as f64) * 100.0);
    println!("  • Thế cờ chiến thuật (200..1000cp)   : {:5} ({:.1}%)", tactical_count, (tactical_count as f64 / total as f64) * 100.0);
    println!("  • Thế cờ áp đảo/sát cục (>= 1000cp)  : {:5} ({:.1}%)", decisive_count, (decisive_count as f64 / total as f64) * 100.0);

    println!("\n🏆 3. PHÂN BỐ KẾT QUẢ VÁN CỜ:");
    println!("  • Đỏ Thắng (Red Win)       : {:5} frames ({:.1}%)", red_wins, (red_wins as f64 / total as f64) * 100.0);
    println!("  • Đen Thắng (Black Win)     : {:5} frames ({:.1}%)", black_wins, (black_wins as f64 / total as f64) * 100.0);
    println!("  • Hòa cờ (Draw)             : {:5} frames ({:.1}%)", draws, (draws as f64 / total as f64) * 100.0);

    println!("\n🤖 4. THỐNG KÊ ĐỘNG CƠ & NƯỚC ĐI:");
    println!("  • Nước đi Xiangqi-RIM       : {:5} ({:.1}%)", rim_moves, (rim_moves as f64 / total as f64) * 100.0);
    println!("  • Nước đi Pikafish          : {:5} ({:.1}%)", pika_moves, (pika_moves as f64 / total as f64) * 100.0);

    println!("\n💡 5. HƯỚNG DẪN SỬ DỤNG LỆNH PHÂN TÍCH:");
    println!("  • Xem thế cờ cụ thể         : cargo run --release --example 120_inspect_harvest_knowledge -- --view <INDEX>");
    println!("  • Xem thế cờ mới nhất       : cargo run --release --example 120_inspect_harvest_knowledge -- --last");
    println!("  • Lọc các bước ngoặt sai lầm: cargo run --release --example 120_inspect_harvest_knowledge -- --blunders 200");
    println!("  • Giải nén sang JSONL       : cargo run --release --example 120_inspect_harvest_knowledge -- --export-jsonl data/output.jsonl\n");
}
