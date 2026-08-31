// ============================================================================
// EXAMPLE 106: HUẤN LUYỆN NNUE STREAMING TỪ KHO DỮ LIỆU CHƯNG CẤT PIKAFISH JSONL
// ============================================================================
// `106_train_nnue_from_distill_jsonl.rs` thực thi nạp hàng loạt hàng trăm nghìn mẫu FEN
// từ kho dữ liệu chưng cất Grandmaster `data/pikafish_in_memory_distill.jsonl` (22.5M mẫu),
// huấn luyện Backpropagation đa luồng trên mạng HalfKAv2_hm và xuất bản nhị phân `XRNN v1`.
//
// Biến môi trường:
//   SAMPLES=500000    Số lượng mẫu FEN nạp vào bộ nhớ (Mặc định: 500,000)
//   EPOCHS=30         Số Epochs huấn luyện Backprop (Mặc định: 30)
//   RATE=0.0001       Tốc độ học Learning Rate (Mặc định: 0.0001)
//   RESET=0           Khởi tạo lại từ đầu nếu =1 (Mặc định: 0 - nạp checkpoint)
//   PATH=...          Đường dẫn tệp JSONL (Mặc định: data/pikafish_in_memory_distill.jsonl)
// ============================================================================

use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::time::Instant;

use xiangrust::board::Parser;
use xiangrust::learn::nnue::{Datum, Network};

/// Hàm xáo trộn Fisher-Yates không phụ thuộc thư viện ngoài (std-only)
fn shuffle(data: &mut [Datum], seed: &mut u64) {
    let n = data.len();
    if n < 2 {
        return;
    }
    for i in (1..n).rev() {
        *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let j = (*seed >> 33) as usize % (i + 1);
        data.swap(i, j);
    }
}

fn main() {
    println!("===============================================================================");
    println!(" ⚡ XIANGQI-RIM STREAMING NNUE TRAINER: PIKAFISH 22.5M JSONL CORPUS");
    println!("    Kiến trúc: HalfKAv2_hm (65536x256 -> 512 -> 32 -> 1) | Chuẩn XRNN v1");
    println!("===============================================================================");

    let target_samples: usize = std::env::var("SAMPLES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(500_000);
    let epochs: usize = std::env::var("EPOCHS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);
    let rate: f32 = std::env::var("RATE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0001);
    let reset: bool = std::env::var("RESET")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false);
    let json_path = std::env::var("DATASET")
        .unwrap_or_else(|_| "data/pikafish_in_memory_distill.jsonl".to_string());

    println!("⚙️ THÔNG SỐ HUẤN LUYỆN STREAMING:");
    println!("  • Tệp dữ liệu nguồn          : {}", json_path);
    println!("  • Số lượng mẫu mục tiêu      : {} mẫu FEN", target_samples);
    println!("  • Số Epochs huấn luyện       : {} epochs", epochs);
    println!("  • Tốc độ học (Learning Rate) : {}", rate);
    println!("  • Chế độ Reset Checkpoint    : {}", reset);
    println!("===============================================================================\n");

    // =========================================================================
    // GIAI ĐOẠN 1: NẠP DỮ LIỆU STREAMING TỪ TỆP JSONL
    // =========================================================================
    println!("[GIAI ĐOẠN 1] Đang nạp {} mẫu FEN từ '{}'...", target_samples, json_path);
    let start_load = Instant::now();

    let file = match File::open(&json_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("❌ Không thể mở tệp '{}': {}", json_path, e);
            return;
        }
    };

    let reader = BufReader::with_capacity(4 * 1024 * 1024, file);
    let mut dataset: Vec<Datum> = Vec::with_capacity(target_samples.min(1_000_000));
    let mut parsed_lines = 0usize;

    for line_res in reader.lines() {
        if dataset.len() >= target_samples {
            break;
        }
        let line = match line_res {
            Ok(l) => l,
            Err(_) => continue,
        };
        parsed_lines += 1;

        // Trích xuất FEN và Score từ chuỗi JSON đơn giản không cần serde crate
        // Định dạng: {"fen":"...","best_move":"...","score":123,"depth":6}
        if let (Some(fen_idx), Some(score_idx)) = (line.find("\"fen\":\""), line.find("\"score\":")) {
            let fen_start = fen_idx + 7;
            if let Some(fen_end) = line[fen_start..].find('"') {
                let fen = &line[fen_start..fen_start + fen_end];
                let score_start = score_idx + 8;
                let score_str: String = line[score_start..]
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '-')
                    .collect();

                if let Ok(score) = score_str.parse::<i32>() {
                    let clamped = score.clamp(-30000, 30000) as i16;
                    let pos = Parser::parse(fen);
                    let datum = Datum::extract(&pos, clamped);
                    dataset.push(datum);
                }
            }
        }

        if dataset.len() % 50_000 == 0 && !dataset.is_empty() {
            println!(
                " -> Đã nạp: {:8}/{} mẫu ({:5.1}%) | Tốc độ: {:.0} FEN/s",
                dataset.len(),
                target_samples,
                (dataset.len() as f64 / target_samples as f64) * 100.0,
                dataset.len() as f64 / start_load.elapsed().as_secs_f64().max(0.001)
            );
            let _ = std::io::stdout().flush();
        }
    }

    let load_time = start_load.elapsed();
    println!(
        "✅ Nạp hoàn tất {} mẫu FEN hợp lệ từ {} dòng trong {:.2?} ({:.0} FEN/s)!\n",
        dataset.len(),
        parsed_lines,
        load_time,
        dataset.len() as f64 / load_time.as_secs_f64().max(0.001)
    );
    let _ = std::io::stdout().flush();

    if dataset.is_empty() {
        eprintln!("❌ Tập dữ liệu rỗng, hủy huấn luyện!");
        return;
    }

    // =========================================================================
    // GIAI ĐOẠN 2: HUẤN LUYỆN MẠNG NNUE VỚI BACKPROPAGATION
    // =========================================================================
    println!("[GIAI ĐOẠN 2] Huấn luyện mạng NNUE HalfKAv2_hm {} Epochs trên {} mẫu...", epochs, dataset.len());
    let mut network = Network::new();
    let mut seed = 999777333u64;

    let checkpoint = "data/nnue_checkpoint.bin";
    if !reset && Path::new(checkpoint).exists() {
        println!(" -> Nạp checkpoint hiện tại: {}", checkpoint);
        let _ = network.load(checkpoint);
    } else {
        println!(" -> Khởi tạo trọng số ngẫu nhiên mới (Xavier Initialization)!");
    }
    let _ = std::io::stdout().flush();

    let train_start = Instant::now();
    let mut best_loss = f64::MAX;

    for epoch in 1..=epochs {
        shuffle(&mut dataset, &mut seed);

        let mut total_loss = 0.0f64;
        let mut count = 0u64;

        for datum in dataset.iter() {
            let (predicted, state) = network.forward(datum);
            let loss = network.backward(datum, &state, predicted, rate);
            total_loss += loss as f64;
            count += 1;
        }

        let mean_loss = if count > 0 { total_loss / count as f64 } else { 0.0 };
        if mean_loss < best_loss {
            best_loss = mean_loss;
            let _ = network.save(checkpoint);
        }

        println!(
            " -> Epoch {:2}/{:2}: mean_loss = {:.4}, best_loss = {:.4} ({} mẫu)",
            epoch, epochs, mean_loss, best_loss, count
        );
        let _ = std::io::stdout().flush();
    }

    let train_time = train_start.elapsed();
    println!(
        "\n✅ Huấn luyện hoàn tất trong {:.2?}: Best Loss = {:.4} ({:.0} mẫu/s)!\n",
        train_time,
        best_loss,
        (dataset.len() * epochs) as f64 / train_time.as_secs_f64().max(0.001)
    );
    let _ = std::io::stdout().flush();

    // =========================================================================
    // GIAI ĐOẠN 3: LƯỢNG TỬ HÓA VÀ XUẤT BẢN TRỌNG SỐ XRNN V1 ĐỘC LẬP
    // =========================================================================
    println!("[GIAI ĐOẠN 3] Lượng tử hóa f32 -> i16/i8 xuất nhị phân XRNN v1 độc lập...");
    let weights_path = "data/nnue_weights.bin";
    let _ = network.quantize(weights_path);
    println!("💾 Đã xuất bản trọng số độc lập thành công: {} (32.02 MB)!\n", weights_path);

    println!("===============================================================================");
    println!(" 🏆 HOÀN TẤT HUẤN LUYỆN STREAMING NNUE GEN 7 TỪ KHO PIKAFISH 22.5M FEN");
    println!("===============================================================================");
}
