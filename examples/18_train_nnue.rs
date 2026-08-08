// ============================================================================
// EXAMPLE 18: HUẤN LUYỆN NNUE TỪ DỮ LIỆU TỰ ĐẤU ENGINE
// ============================================================================
// Pipeline hoàn chỉnh:
//   1. GYM Self-Play Engine Depth 4-8 sinh dữ liệu (position, search_score)
//   2. Trích xuất đặc trưng NNUE HalfKAv2_hm từ mỗi vị trí
//   3. Huấn luyện mạng f32 bằng Backpropagation + SGD
//   4. Lượng tử hóa f32 → i16/i8 xuất tệp nhị phân cho inference
//
// Sử dụng: cargo run --release --example 18_train_nnue
// Biến môi trường:
//   EPOCHS=100        Số epoch huấn luyện (mặc định 100)
//   GAMES=5000        Số ván tự đấu sinh dữ liệu (mặc định 5000)
//   DEPTH=6           Độ sâu tìm kiếm Engine (mặc định 6)
//   RATE=0.001        Tốc độ học (mặc định 0.001)
// ============================================================================

use xiangrust::board::Parser;
use xiangrust::learn::nnue::{Datum, Network};
use xiangrust::search::{Limits, Search};

/// Tạo dữ liệu huấn luyện từ 1 ván tự đấu Engine.
/// Trả về danh sách Datum (position features + search score).
fn generate(depth: u8, limit: u32) -> Vec<Datum> {
    let mut pos = Parser::parse(Parser::DEFAULT);
    let mut search = Search::new(16);
    let mut limits = Limits::new();
    limits.depth = depth;
    let mut data = Vec::with_capacity(128);
    let mut steps = 0u32;

    while steps < limit {
        // Engine Search tìm nước đi tốt nhất
        let result = search.go(&pos, &limits);
        if !result.best.valid() {
            break;
        }

        let score = result.score as i16;

        // Trích xuất đặc trưng NNUE cho vị trí hiện tại
        let datum = Datum::extract(&pos, score);
        data.push(datum);

        // Thực hiện nước đi
        pos.apply(result.best.from, result.best.to);
        steps += 1;

        // Phát hiện lặp nước 2-fold

        // Kiểm tra đơn giản: nếu quá nhiều nước đi mà score gần 0, dừng sớm
        if steps > 200 && score.abs() < 50 {
            break;
        }
    }

    data
}

/// Xáo trộn mảng Fisher-Yates shuffle không cần rand crate.
fn shuffle(data: &mut Vec<Datum>, seed: &mut u64) {
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
    println!("============================================================");
    println!(" XIANGQI-RIM NNUE TRAINING ENGINE (GEN 1)                  ");
    println!("============================================================");

    // Đọc cấu hình từ biến môi trường
    let epochs: usize = std::env::var("EPOCHS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);
    let games: usize = std::env::var("GAMES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5000);
    let depth: u8 = std::env::var("DEPTH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(6);
    let rate: f32 = std::env::var("RATE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.001);
    let limit: u32 = 300; // Giới hạn nước đi tối đa mỗi ván

    println!("Cấu hình: epochs={}, games={}, depth={}, rate={}, limit={}", epochs, games, depth, rate, limit);
    println!();

    // =========================================================================
    // GIAI ĐOẠN 1: SINH DỮ LIỆU HUẤN LUYỆN TỪ TỰ ĐẤU ENGINE
    // =========================================================================
    println!("[GIAI ĐOẠN 1] Sinh dữ liệu huấn luyện từ {} ván tự đấu Engine depth={}...", games, depth);
    let begin = std::time::Instant::now();

    let mut data: Vec<Datum> = Vec::with_capacity(games * 60);
    for g in 0..games {
        let positions = generate(depth, limit);
        data.extend(positions);

        if (g + 1) % 100 == 0 || g + 1 == games {
            print!("\r -> Tiến độ: {}/{} ván ({} mẫu)", g + 1, games, data.len());
        }
    }
    println!();

    let elapsed = begin.elapsed();
    println!(
        "✅ Sinh dữ liệu hoàn tất: {} mẫu từ {} ván ({:.1}s, {:.0} mẫu/s)",
        data.len(),
        games,
        elapsed.as_secs_f64(),
        data.len() as f64 / elapsed.as_secs_f64()
    );
    println!();

    // =========================================================================
    // GIAI ĐOẠN 2: HUẤN LUYỆN MẠNG NƠ-RON NNUE
    // =========================================================================
    println!("[GIAI ĐOẠN 2] Huấn luyện NNUE {} epochs trên {} mẫu...", epochs, data.len());
    let mut network = Network::new();
    let mut seed = 12345u64;

    // Kiểm tra xem có checkpoint không
    let checkpoint = "data/nnue_checkpoint.bin";
    if std::path::Path::new(checkpoint).exists() {
        println!(" -> Nạp checkpoint từ {}", checkpoint);
        if let Err(e) = network.load(checkpoint) {
            println!(" -> Lỗi nạp checkpoint: {}, bắt đầu từ đầu", e);
        }
    }


    let mut best = f64::MAX;

    for epoch in 0..epochs {
        // Xáo trộn dữ liệu mỗi epoch
        shuffle(&mut data, &mut seed);

        let mut sum = 0.0f64;
        let mut count = 0u64;

        for datum in data.iter() {
            let (predicted, state) = network.forward(datum);
            let loss = network.backward(datum, &state, predicted, rate);
            sum += loss as f64;
            count += 1;
        }

        let mean = if count > 0 { sum / count as f64 } else { 0.0 };

        if (epoch + 1) % 10 == 0 || epoch == 0 || epoch + 1 == epochs {
            println!(
                " -> Epoch {}/{}: mean_loss={:.4}, samples={}",
                epoch + 1,
                epochs,
                mean,
                count
            );
        }

        // Lưu checkpoint mỗi 20 epoch nếu loss cải thiện
        if mean < best {
            best = mean;
            if (epoch + 1) % 20 == 0 {
                if let Err(e) = network.save(checkpoint) {
                    eprintln!(" -> Lỗi lưu checkpoint: {}", e);
                }
            }
        }
    }

    // Lưu checkpoint cuối cùng
    println!();
    if let Err(e) = network.save(checkpoint) {
        eprintln!("[LỖI] Không thể lưu checkpoint: {}", e);
    } else {
        println!("💾 Checkpoint f32: {}", checkpoint);
    }

    // =========================================================================
    // GIAI ĐOẠN 3: LƯỢNG TỬ HÓA VÀ XUẤT TRỌNG SỐ INFERENCE
    // =========================================================================
    println!();
    println!("[GIAI ĐOẠN 3] Lượng tử hóa f32 → i16/i8 cho inference SIMD...");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let output = format!("data/nnue_weights_{}.bin", stamp);

    if let Err(e) = network.quantize(&output) {
        eprintln!("[LỖI] Không thể xuất trọng số: {}", e);
    } else {
        // Kiểm tra dung lượng tệp
        if let Ok(meta) = std::fs::metadata(&output) {
            let mb = meta.len() as f64 / (1024.0 * 1024.0);
            println!("💾 Trọng số NNUE inference: {} ({:.1} MB)", output, mb);
        }
    }

    // =========================================================================
    // GIAI ĐOẠN 4: XÁC MINH CHẤT LƯỢNG
    // =========================================================================
    println!();
    println!("[GIAI ĐOẠN 4] Xác minh chất lượng trên 100 mẫu...");
    let sample_size = 100.min(data.len());
    let mut total_error = 0.0f64;
    let mut max_error = 0.0f64;

    for i in 0..sample_size {
        let (predicted, _) = network.forward(&data[i]);
        let target = data[i].target as f64;
        let err = (predicted as f64 - target).abs();
        total_error += err;
        if err > max_error {
            max_error = err;
        }
    }

    let mean_error = total_error / sample_size as f64;
    println!("  Mean Absolute Error: {:.1} centipawn", mean_error);
    println!("  Max Absolute Error:  {:.1} centipawn", max_error);

    println!();
    println!("============================================================");
    println!("✅ NNUE TRAINING HOÀN TẤT!");
    println!("   Best loss: {:.6}", best);
    println!("   Tệp trọng số: {}", output);
    println!("============================================================");
}
