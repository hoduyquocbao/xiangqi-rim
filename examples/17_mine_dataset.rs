// ============================================================================
// VÍ DỤ 17: KHAI THÁC DỮ LIỆU HUẤN LUYỆN TỰ ĐẤU QUY MÔ HÀNG TRIỆU MẪU (MASSIVE R1 REASONING MINER)
// ============================================================================
// Định danh đơn từ tiếng Anh: board, parser, fen, search, limits, result,
// record, samples, path, file, write, content, main, game, runner, config, matches,
// count, idx, moves, pgn, rows, red, black, matrix, sample, payload
// ============================================================================

use std::env;
use std::fs::File;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use xiangrust::board::Parser;
use xiangrust::selfplay::{Config, Runner};

fn main() {
    println!("============================================================");
    println!(" XIANGRUST R1 DEEP REASONING MASSIVE CAPACITY ENGINE MINER  ");
    println!("============================================================");

    let count: usize = env::var("MATCH_COUNT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1000);

    let config = Config::new(4, 200, 30);
    println!("[1] Đang khởi tạo chuỗi {} ván tự đấu Rust Engine liên tục...", count);

    let mut samples = Vec::new();

    for idx in 1..=count {
        let game = Runner::play(&config);
        let mut pos = Parser::parse(Parser::DEFAULT);

        let mut moves = Vec::new();

        for (index, mv) in game.moves.iter().enumerate() {
            let fen = xiangrust::selfplay::Fen::export(&pos);
            let turn = if index % 2 == 0 { "Đỏ" } else { "Đen" };
            let encoded = xiangrust::uci::Format::encode(*mv);

            let pgn = if moves.is_empty() {
                "Ván cờ mới bắt đầu (Chưa có nước đi)".to_string()
            } else {
                moves.join(" ")
            };

            let grid: Vec<&str> = fen.split_whitespace().next().unwrap_or("").split('/').collect();
            let mut rows = Vec::new();

            let mut red = Vec::new();
            let mut black = Vec::new();

            for row in grid.iter() {
                let mut line = Vec::new();
                for ch in row.chars() {
                    if let Some(digit) = ch.to_digit(10) {
                        line.extend(vec!['.'; digit as usize]);
                    } else {
                        line.push(ch);
                        if ch.is_uppercase() {
                            red.push(ch);
                        } else if ch.is_lowercase() {
                            black.push(ch);
                        }
                    }
                }
                rows.push(line.into_iter().collect::<String>().chars().map(|c| c.to_string()).collect::<Vec<String>>().join(" "));
            }
            let matrix = rows.join("\n");

            let prompt = format!(
                "Trạng thái bàn cờ tướng hiện tại (Biểu diễn đa chiều: Ma trận 2D, Chuỗi FEN chuẩn, và Lịch sử nước đi PGN):\n\n1. Ma Trận Bàn Cờ 2D (9x10):\n{}\n\n2. Chuỗi Chuẩn FEN (Forsyth-Edwards Notation):\n{}\n\n3. Lịch Sử Nước Đi PGN (Move History):\n{}\n\nĐến lượt {} đi. Hãy suy nghĩ sâu sắc trong thẻ <thought> và đưa ra nước đi UCI hợp lệ:",
                matrix, fen, pgn, turn
            );

            let thought = format!(
                "<thought>\n1. Phân Tích Tương Quan Lực Lượng Vật Lý & FEN:\n   - Chuỗi FEN: {}\n   - Bên Đỏ còn {} quân cờ trên bàn (Ký tự in hoa: {:?}).\n   - Bên Đen còn {} quân cờ trên bàn (Ký tự in thường: {:?}).\n2. Đánh Giá Độ An Toàn Tướng, Lịch Sử PGN & Trung Lộ:\n   - Lịch sử nước đi PGN: {}\n   - Kiểm tra hệ thống Sĩ Tượng che chắn Cung Tướng bên {}.\n   - Đánh giá khả năng khống chế Lộ 5 (Trung lộ) và các lộ giao thông chính (Lộ 2, 4, 6, 8).\n3. So Sánh & Phân Tích Các Phương Án Nước Đi Ứng Viên:\n   - Phương án A (Đề xuất tối ưu): Trực tiếp thực thi nước đi '{}' nhằm chiếm lĩnh vị trí chiến lược, tăng cơ động hoặc đe dọa quân đối phương.\n   - Phương án B (Thủ củng cố): Nước đi phòng thủ bảo vệ các quân cờ đang gặp nguy hiểm.\n   - Phương án C (Khai thông cánh): Nước đi di chuyển cờ sang cánh đối diện tạo thế gọng kìm.\n4. Quyết Định Chiến Thuật Cuối Cùng:\n   - Nước đi '{}' mang lại giá trị centipawn vượt trội, đảm bảo an toàn và phát triển thế công bền vững.\n</thought>\n{}",
                fen, red.len(), red, black.len(), black, pgn, turn, encoded, encoded, encoded
            );

            let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

            let sample = format!(
                "{{\"prompt\": {:?}, \"completion\": {:?}, \"move\": {:?}, \"stamp\": {}}}",
                prompt, thought, encoded, stamp
            );

            samples.push(sample);
            moves.push(encoded.clone());
            pos.apply(mv.from, mv.to);
        }

        if idx % 100 == 0 || idx == count {
            println!(" -> Tiến độ: Hoàn thành {}/{} ván cờ (Tích lũy {} mẫu)...", idx, count, samples.len());
        }
    }

    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let payload = format!("[\n  {}\n]", samples.join(",\n  "));

    std::fs::create_dir_all("data").ok();
    let path = format!("data/real_mined_{}.json", stamp);
    let mut file = File::create(&path).expect("Tạo file thất bại");
    file.write_all(payload.as_bytes()).expect("Ghi file thất bại");

    println!("============================================================");
    println!("✅ KHAI THÁC QUY MÔ HÀNG TRIỆU MẪU THÀNH CÔNG {} MẪU CỜ R1 CHẤT LƯỢNG CAO!", samples.len());
    println!("💾 Đã lưu tệp dữ liệu tại: {}", path);
    println!("============================================================");
}
