// ============================================================================
// VÍ DỤ 119: BIÊN DỊCH SÁCH KHAI CUỘC ĐẠI KIỆN TƯỚNG SIÊU CẤP MASTER BOOK XRBK v1
// ============================================================================
// `119_build_grandmaster_super_book.rs` trích xuất và hợp nhất toàn bộ các thế cờ khai cuộc:
// 1. Nguồn 1: `data/opening_vetted_book.jsonl` (994 biến thể khai cuộc thẩm định chuẩn).
// 2. Nguồn 2: `data/master_grandmaster_distill.jsonl` (Kỳ phổ Đại Kiện Tướng).
// 3. Nguồn 3: `data/production_massive_depth6_5000.jsonl` (Trích xuất chuỗi nước đi từ FEN liên tiếp 5,000 ván Depth 6).
// 4. Lọc bỏ các nước đi phạm luật, tính toán khóa băm Zobrist Hash O(1), gán trọng số ưu tiên cao.
// 5. Xuất bản tệp nhị phân chuẩn XRBK v1: `data/master_book.xrbk`.
//
// 100% chú thích tiếng Việt từng dòng & 100% định danh từ đơn tiếng Anh.
// ============================================================================

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::time::Instant;

use xiangrust::board::{Parser, Position};
use xiangrust::book::opening::Entry;
use xiangrust::book::Loader;
use xiangrust::movegen::legal;
use xiangrust::movegen::types::Move;
use xiangrust::uci::Format;

/// Suy diễn nước đi từ 2 thế cờ FEN kế tiếp nhau
fn deduce_move(prev: &Position, curr: &Position) -> Option<Move> {
    let side = prev.side as usize;
    let mut from_sq = None;
    let mut moving_piece = 255u8;

    for sq in 0..90 {
        let p_prev = prev.grid[sq];
        let p_curr = curr.grid[sq];
        if p_prev < 14 && (p_prev as usize / 7) == side && p_curr != p_prev {
            from_sq = Some(sq as u8);
            moving_piece = p_prev;
            break;
        }
    }

    let from = from_sq?;
    for sq in 0..90 {
        if sq as u8 != from && curr.grid[sq] == moving_piece && prev.grid[sq] != moving_piece {
            return Some(Move::new(from, sq as u8));
        }
    }

    None
}

fn main() {
    println!("===============================================================================");
    println!(" 📚 BIÊN DỊCH SÁCH KHAI CUỘC ĐẠI KIỆN TƯỚNG MASTER BOOK XRBK v1");
    println!("===============================================================================\n");
    let _ = std::io::stdout().flush();

    let start = Instant::now();
    let mut book_map: HashMap<u64, (u16, u16, &'static str)> = HashMap::with_capacity(300_000);

    // 1. Nạp từ data/opening_vetted_book.jsonl
    let vetted_path = "data/opening_vetted_book.jsonl";
    if std::path::Path::new(vetted_path).exists() {
        if let Ok(file) = File::open(vetted_path) {
            let reader = BufReader::new(file);
            let mut count = 0usize;
            for line_res in reader.lines().flatten() {
                let trimmed = line_res.trim();
                if let (Some(f_start), Some(m_start)) = (trimmed.find("\"fen\":\""), trimmed.find("\"best_move\":\"")) {
                    let f_end = trimmed[f_start + 7..].find('"').unwrap_or(0);
                    let fen_str = &trimmed[f_start + 7..f_start + 7 + f_end];
                    let m_end = trimmed[m_start + 13..].find('"').unwrap_or(0);
                    let move_str = &trimmed[m_start + 13..m_start + 13 + m_end];

                    let mut pos = Parser::parse(fen_str);
                    let mv = Format::decode(move_str);
                    if mv.valid() && legal::valid(&mut pos, mv) {
                        let raw_mv = ((mv.from as u16) << 8) | (mv.to as u16);
                        book_map.insert(pos.hash, (raw_mv, 990, "Vetted Grandmaster"));
                        count += 1;
                    }
                }
            }
            println!("  ✅ Đã nạp {} biến thể từ {}", count, vetted_path);
        }
    }

    // 2. Nạp từ data/master_grandmaster_distill.jsonl
    let distill_path = "data/master_grandmaster_distill.jsonl";
    if std::path::Path::new(distill_path).exists() {
        if let Ok(file) = File::open(distill_path) {
            let reader = BufReader::new(file);
            let mut count = 0usize;
            for line_res in reader.lines().flatten() {
                let trimmed = line_res.trim();
                if let (Some(f_start), Some(m_start)) = (trimmed.find("\"fen\":\""), trimmed.find("\"best_move\":\"")) {
                    let f_end = trimmed[f_start + 7..].find('"').unwrap_or(0);
                    let fen_str = &trimmed[f_start + 7..f_start + 7 + f_end];
                    let m_end = trimmed[m_start + 13..].find('"').unwrap_or(0);
                    let move_str = &trimmed[m_start + 13..m_start + 13 + m_end];

                    let mut pos = Parser::parse(fen_str);
                    let mv = Format::decode(move_str);
                    if mv.valid() && legal::valid(&mut pos, mv) {
                        let raw_mv = ((mv.from as u16) << 8) | (mv.to as u16);
                        book_map.entry(pos.hash).or_insert((raw_mv, 950, "Distill Grandmaster"));
                        count += 1;
                    }
                }
            }
            println!("  ✅ Đã nạp {} biến thể từ {}", count, distill_path);
        }
    }

    // 3. Nạp và suy diễn nước đi từ 5,000 ván cờ tự đấu Depth 6 (Chỉ lấy các thế cờ ply <= 24)
    let massive_path = "data/production_massive_depth6_5000.jsonl";
    if std::path::Path::new(massive_path).exists() {
        if let Ok(file) = File::open(massive_path) {
            let reader = BufReader::new(file);
            let mut count = 0usize;
            let mut prev_pos: Option<Position> = None;
            let mut ply_in_game = 0usize;

            for line_res in reader.lines().flatten() {
                let trimmed = line_res.trim();
                if let Some(f_start) = trimmed.find("\"fen\":\"") {
                    let f_end = trimmed[f_start + 7..].find('"').unwrap_or(0);
                    let fen_str = &trimmed[f_start + 7..f_start + 7 + f_end];
                    let curr_pos = Parser::parse(fen_str);

                    if let Some(mut prev) = prev_pos {
                        // Nếu là nước tiếp theo trong cùng ván cờ (side đảo ngược)
                        if curr_pos.side != prev.side && ply_in_game <= 24 {
                            if let Some(mv) = deduce_move(&prev, &curr_pos) {
                                if mv.valid() && legal::valid(&mut prev, mv) {
                                    let raw_mv = ((mv.from as u16) << 8) | (mv.to as u16);
                                    book_map.entry(prev.hash).or_insert((raw_mv, 850, "Depth 6 Tactical"));
                                    count += 1;
                                }
                            }
                            ply_in_game += 1;
                        } else {
                            // Bắt đầu ván mới
                            ply_in_game = 1;
                        }
                    } else {
                        ply_in_game = 1;
                    }

                    prev_pos = Some(curr_pos);
                }
            }
            println!("  ✅ Đã trích xuất & nạp {} biến thể khai cuộc từ {}", count, massive_path);
        }
    }

    // 4. Chuyển đổi sang Vec<Entry> và sắp xếp theo khóa băm Zobrist Hash
    let mut entries: Vec<Entry> = book_map
        .into_iter()
        .map(|(hash, (mv, weight, name))| Entry::new(hash, mv, weight, name))
        .collect();

    entries.sort_unstable_by_key(|e| e.hash);
    entries.dedup_by_key(|e| e.hash);

    let output_path = "data/master_book.xrbk";
    Loader::save_xrbk(output_path, &entries).expect("Lưu master_book.xrbk thất bại!");

    let elapsed = start.elapsed();
    println!("\n===============================================================================");
    println!(" 🏆 XUẤT BẢN THÀNH CÔNG: {} BIẾN THỂ KHAI CUỘC XRBK v1!", entries.len());
    println!("    Đường dẫn tệp : {}", output_path);
    println!("    Thời gian xử lý: {:.2}s", elapsed.as_secs_f64());
    println!("===============================================================================\n");
}
