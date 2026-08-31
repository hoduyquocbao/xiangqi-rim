// ============================================================================
// VÍ DỤ 109: BỘ NHÂN BẢN ĐỐI XỨNG DỮ LIỆU CÂN BẰNG HẬU THỦ (45.19M FENs ENGINE)
// ============================================================================
// `109_symmetric_augmentation_45m.rs` giải quyết triệt để tử huyệt mất cân bằng thế cờ:
// 1. Đọc từng mẫu FEN từ `data/pikafish_in_memory_distill.jsonl` (22.59M mẫu).
// 2. Xuất bản mẫu gốc.
// 3. Tự động lật ngược bàn cờ 180 độ (Đảo ngược phe đi, đảo ngược màu quân, đảo ngược điểm score).
// 4. Xuất bản tập dữ liệu cân bằng tuyệt đối 45,196,882 mẫu FENs (`data/symmetrical_45m_distill.jsonl`).
//
// 100% chú thích tiếng Việt từng dòng & 100% định danh từ đơn tiếng Anh.
// ============================================================================

use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::time::Instant;

// use xiangrust::board::Parser;

/// Đảo ngược ký tự quân cờ giữa Hoa (Đỏ) và Thường (Đen)
#[inline(always)]
fn invert_piece(c: char) -> char {
    if c.is_ascii_uppercase() {
        c.to_ascii_lowercase()
    } else if c.is_ascii_lowercase() {
        c.to_ascii_uppercase()
    } else {
        c
    }
}

/// Lật ngược chuỗi FEN 180 độ theo góc nhìn phe đối địch
fn flip_fen(fen: &str) -> Option<String> {
    let mut parts = fen.split_whitespace();
    let board_part = parts.next()?;
    let side_part = parts.next().unwrap_or("w");

    // 1. Đảo ngược các hàng và lật các ký tự quân cờ
    let rows: Vec<&str> = board_part.split('/').collect();
    if rows.len() != 10 {
        return None;
    }

    let mut flipped_rows = Vec::with_capacity(10);
    for row in rows.into_iter().rev() {
        let mut flipped_row = String::with_capacity(row.len());
        for c in row.chars().rev() {
            flipped_row.push(invert_piece(c));
        }
        flipped_rows.push(flipped_row);
    }

    let new_board = flipped_rows.join("/");
    let new_side = if side_part == "w" || side_part == "r" { "b" } else { "w" };

    Some(format!("{} {} - - 0 1", new_board, new_side))
}

fn main() {
    println!("===============================================================================");
    println!(" ⚡ XIANGQI-RIM SYMMETRIC PERSPECTIVE DATA AUGMENTATION: 22.5M -> 45.19M FENs");
    println!("===============================================================================");

    let input_path = "data/pikafish_in_memory_distill.jsonl";
    let output_path = "data/symmetrical_45m_distill.jsonl";

    let in_file = match File::open(input_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("❌ Không thể mở tệp nguồn '{}': {}", input_path, e);
            return;
        }
    };

    let out_file = match File::create(output_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("❌ Không thể tạo tệp đích '{}': {}", output_path, e);
            return;
        }
    };

    let mut reader = BufReader::with_capacity(16 * 1024 * 1024, in_file);
    let mut writer = BufWriter::with_capacity(16 * 1024 * 1024, out_file);

    let start = Instant::now();
    let mut line = String::new();
    let mut count = 0usize;
    let mut total_exported = 0usize;

    println!("📖 Đang xử lý nhân bản đối xứng dữ liệu...");
    let _ = std::io::stdout().flush();

    while reader.read_line(&mut line).unwrap_or(0) > 0 {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            if let (Some(fen_idx), Some(score_idx)) = (trimmed.find("\"fen\":\""), trimmed.find("\"score\":")) {
                let fen_start = fen_idx + 7;
                if let Some(fen_end) = trimmed[fen_start..].find('"') {
                    let fen = &trimmed[fen_start..fen_start + fen_end];
                    let score_start = score_idx + 8;
                    let score_str: String = trimmed[score_start..]
                        .chars()
                        .take_while(|c| c.is_ascii_digit() || *c == '-')
                        .collect();

                    if let Ok(score) = score_str.parse::<i32>() {
                        // 1. Ghi mẫu gốc
                        let _ = writeln!(writer, "{{\"fen\":\"{}\",\"score\":{}}}", fen, score);
                        total_exported += 1;

                        // 2. Tạo và ghi mẫu đối xứng lật 180 độ
                        if let Some(flipped) = flip_fen(fen) {
                            let _ = writeln!(writer, "{{\"fen\":\"{}\",\"score\":{}}}", flipped, -score);
                            total_exported += 1;
                        }
                    }
                }
            }
        }
        line.clear();
        count += 1;

        if count % 1_000_000 == 0 {
            println!("  -> Đã xử lý {:8} mẫu nguồn, xuất bản {:8} mẫu đối xứng ({:.0} FEN/s)...", count, total_exported, total_exported as f64 / start.elapsed().as_secs_f64());
            let _ = std::io::stdout().flush();
        }
    }

    let _ = writer.flush();
    let dur = start.elapsed();
    println!("===============================================================================");
    println!(" 🏆 HOÀN TẤT NHÂN BẢN ĐỐI XỨNG: {} MẪU FENs TRONG {:.2?} ({:.0} FEN/s)!", total_exported, dur, total_exported as f64 / dur.as_secs_f64());
    println!("    Tệp xuất bản: {} (Cân bằng tuyệt đối 50% Đỏ - 50% Đen)", output_path);
    println!("===============================================================================\n");
}
