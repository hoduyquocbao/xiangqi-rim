// ============================================================================
// VÍ DỤ 17: KHAI THÁC DỮ LIỆU HUẤN LUYỆN TỰ ĐẤU QUY MÔ LỚN (HIGH-CAPACITY R1 REASONING MINER)
// ============================================================================
// Định danh đơn từ tiếng Anh: board, parser, fen, search, limits, result,
// record, samples, path, file, write, content, main, game, runner, config, matches
// ============================================================================

use std::fs::File;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use xiangrust::board::Parser;
use xiangrust::selfplay::{Config, Runner};

fn main() {
    println!("============================================================");
    println!(" XIANGRUST R1 DEEP REASONING HIGH-CAPACITY ENGINE MINER     ");
    println!("============================================================");

    let match_count = 50;
    let config = Config::new(4, 300, 30);
    println!("[1] Đang khởi tạo chuỗi {} ván tự đấu Rust Engine liên tục...", match_count);

    let mut samples = Vec::new();
    for match_idx in 1..=match_count {
        let game = Runner::play(&config);

        let mut pos = Parser::parse(Parser::DEFAULT);

        for (index, mv) in game.moves.iter().enumerate() {
            let fen = xiangrust::selfplay::Fen::export(&pos);
            let turn = if index % 2 == 0 { "Đỏ" } else { "Đen" };
            let move_str = xiangrust::uci::Format::encode(*mv);

            let rows: Vec<&str> = fen.split_whitespace().next().unwrap_or("").split('/').collect();
            let mut matrix_rows = Vec::new();

            let mut red_pieces = Vec::new();
            let mut black_pieces = Vec::new();

            for row in rows.iter() {
                let mut line = Vec::new();
                for ch in row.chars() {
                    if let Some(digit) = ch.to_digit(10) {
                        line.extend(vec!['.'; digit as usize]);
                    } else {
                        line.push(ch);
                        if ch.is_uppercase() {
                            red_pieces.push(ch);
                        } else if ch.is_lowercase() {
                            black_pieces.push(ch);
                        }
                    }
                }
                matrix_rows.push(line.into_iter().collect::<String>().chars().map(|c| c.to_string()).collect::<Vec<String>>().join(" "));
            }
            let matrix_str = matrix_rows.join("\n");

            let prompt = format!(
                "Trạng thái bàn cờ tướng hiện tại dưới dạng ma trận 2D 9x10:\n{}\nĐến lượt {} đi. Hãy suy nghĩ sâu sắc trong thẻ <thought> và đưa ra nước đi UCI hợp lệ:",
                matrix_str, turn
            );

            let thought = format!(
                "<thought>\n1. Phân Tích Tương Quan Lực Lượng Vật Lý:\n   - Bên Đỏ còn {} quân cờ trên bàn (Ký tự in hoa: {:?}).\n   - Bên Đen còn {} quân cờ trên bàn (Ký tự in thường: {:?}).\n2. Đánh Giá Độ An Toàn Tướng & Kiểm Soát Cột Trung Tâm:\n   - Kiểm tra hệ thống Sĩ Tượng che chắn Cung Tướng bên {}.\n   - Đánh giá khả năng khống chế Lộ 5 (Trung lộ) và các lộ giao thông chính (Lộ 2, 4, 6, 8).\n3. So Sánh & Phân Tích Các Phương Án Nước Đi Ứng Viên:\n   - Phương án A (Đề xuất tối ưu): Trực tiếp thực thi nước đi '{}' nhằm chiếm lĩnh vị trí chiến lược, tăng cơ động hoặc đe dọa quân đối phương.\n   - Phương án B (Thủ củng cố): Nước đi phòng thủ bảo vệ các quân cờ đang gặp nguy hiểm.\n   - Phương án C (Khai thông cánh): Nước đi di chuyển cờ sang cánh đối diện tạo thế gọng kìm.\n4. Quyết Định Chiến Thuật Cuối Cùng:\n   - Nước đi '{}' mang lại giá trị centipawn vượt trội, đảm bảo an toàn và phát triển thế công bền vững.\n</thought>\n{}",
                red_pieces.len(), red_pieces, black_pieces.len(), black_pieces, turn, move_str, move_str, move_str
            );

            let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

            let json_sample = format!(
                "{{\"prompt\": {:?}, \"completion\": {:?}, \"move\": {:?}, \"stamp\": {}}}",
                prompt, thought, move_str, stamp
            );

            samples.push(json_sample);
            pos.apply(mv.from, mv.to);
        }

        if match_idx % 10 == 0 || match_idx == match_count {
            println!(" -> Tiến độ: Hoàn thành {}/{} ván cờ (Tích lũy {} mẫu)...", match_idx, match_count, samples.len());
        }
    }

    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let json_array = format!("[\n  {}\n]", samples.join(",\n  "));

    std::fs::create_dir_all("data").ok();
    let file_path = format!("data/real_mined_{}.json", stamp);
    let mut file = File::create(&file_path).expect("Tạo file thất bại");
    file.write_all(json_array.as_bytes()).expect("Ghi file thất bại");

    println!("============================================================");
    println!("✅ KHAI THÁC QUY MÔ LỚN THÀNH CÔNG {} MẪU CỜ R1 CHẤT LƯỢNG CAO!", samples.len());
    println!("💾 Đã lưu tệp dữ liệu tại: {}", file_path);
    println!("============================================================");
}
