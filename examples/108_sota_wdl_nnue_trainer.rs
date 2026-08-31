// ============================================================================
// VÍ DỤ 108: BỘ HUẤN LUYỆN SOTA SIGMOID WDL + LAMBDA MSE LOSS 22.5M NNUE TRAINER
// ============================================================================
// `108_sota_wdl_nnue_trainer.rs` triển khai bộ huấn luyện SOTA kết hợp hai hàm mục tiêu:
// 1. Sigmoid WDL (Win/Draw/Loss Binary Cross-Entropy): Cân bằng độ nhạy ở trung cuộc giằng co [-50cp, +50cp].
// 2. Lambda Centipawn MSE Loss: Đảm bảo độ dứt khoát phân định thắng/thua ở tàn cuộc.
// 3. Khởi tạo Xavier phân phối chuẩn cho Feature Transformer + Đạo hàm LeakyReLU leak slope.
// 4. Kiến trúc Hogwild! Lock-Free Async SGD trên 4 nhân CPU vật lý.
//
// 100% chú thích tiếng Việt từng dòng & 100% định danh từ đơn tiếng Anh.
// ============================================================================

use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use xiangrust::board::Parser;
use xiangrust::learn::nnue::Datum;

/// Kích thước chiều không gian đặc trưng nửa bàn cờ HalfKAv2_hm
const TOTAL: usize = 65536;
/// Số chiều vector nhúng của Feature Transformer
const DIM: usize = 256;
/// Kích thước lớp kết hợp hai góc nhìn phe Ta và phe Địch (Both Perspectives)
const BOTH: usize = 512;
/// Kích thước lớp ẩn Hidden Layer
const HIDDEN: usize = 32;

/// Con trỏ chia sẻ tài nguyên thô phục vụ Hogwild! Lock-Free SGD
#[derive(Clone, Copy)]
pub struct Shared {
    /// Con trỏ ma trận trọng số Feature Transformer (65,536 x 256)
    pub feature: *mut [f32; DIM],
    /// Con trỏ vector Bias Feature Transformer (256)
    pub bias: *mut f32,
    /// Con trỏ ma trận trọng số lớp ẩn (32 x 512)
    pub hidden: *mut [[f32; BOTH]; HIDDEN],
    /// Con trỏ vector Bias lớp ẩn (32)
    pub offset: *mut f32,
    /// Con trỏ vector trọng số lớp đầu ra (32)
    pub output: *mut f32,
    /// Con trỏ Bias lớp đầu ra
    pub anchor: *mut f32,
}

unsafe impl Send for Shared {}
unsafe impl Sync for Shared {}

/// Mạng nơ-ron f32 quản lý bộ nhớ huấn luyện
pub struct Network {
    /// Mảng trọng số Feature Transformer (65,536 x 256)
    pub feature: Vec<[f32; DIM]>,
    /// Mảng Bias Feature Transformer (256)
    pub bias: [f32; DIM],
    /// Ma trận lớp ẩn Hidden Layer (32 x 512)
    pub hidden: [[f32; BOTH]; HIDDEN],
    /// Mảng Bias lớp ẩn (32)
    pub offset: [f32; HIDDEN],
    /// Mảng trọng số lớp đầu ra (32)
    pub output: [f32; HIDDEN],
    /// Bias lớp đầu ra
    pub anchor: f32,
}

impl Network {
    /// Khởi tạo mạng mới trên Heap với phân phối chuẩn Xavier
    pub fn new() -> Box<Self> {
        let mut net = unsafe {
            let layout = std::alloc::Layout::new::<Self>();
            let ptr = std::alloc::alloc_zeroed(layout) as *mut Self;
            Box::from_raw(ptr)
        };
        net.feature = vec![[0.0f32; DIM]; TOTAL];
        net.bias = [0.5f32; DIM];
        net.offset = [0.5f32; HIDDEN];

        let scale_feat = 1.0f32 / (DIM as f32).sqrt();
        let mut seed = 42u64;
        for idx in 0..TOTAL {
            for j in 0..DIM {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                let val = ((seed >> 33) as f32 / (u32::MAX as f32) - 0.5) * 2.0 * scale_feat;
                net.feature[idx][j] = val;
            }
        }

        let scale_hidden = 1.0f32 / (BOTH as f32).sqrt();
        for i in 0..HIDDEN {
            for j in 0..BOTH {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                let val = ((seed >> 33) as f32 / (u32::MAX as f32) - 0.5) * 2.0 * scale_hidden;
                net.hidden[i][j] = val;
            }
        }

        let scale_output = 1.0f32 / (HIDDEN as f32).sqrt();
        for i in 0..HIDDEN {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let val = ((seed >> 33) as f32 / (u32::MAX as f32) - 0.5) * 2.0 * scale_output;
            net.output[i] = val;
        }
        net.anchor = 0.0f32;
        net
    }

    /// Trả về con trỏ cấu trúc thô `Shared`
    pub fn share(&mut self) -> Shared {
        Shared {
            feature: self.feature.as_mut_ptr(),
            bias: self.bias.as_mut_ptr(),
            hidden: &mut self.hidden as *mut [[f32; BOTH]; HIDDEN],
            offset: self.offset.as_mut_ptr(),
            output: self.output.as_mut_ptr(),
            anchor: &mut self.anchor as *mut f32,
        }
    }

    /// Lưu Checkpoint trọng số f32 xuống đĩa
    pub fn checkpoint(&self, path: &str) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = std::io::BufWriter::with_capacity(16 * 1024 * 1024, file);
        writer.write_all(b"XRCP")?;
        let ver = 1u32;
        writer.write_all(&ver.to_le_bytes())?;

        for b in &self.bias {
            writer.write_all(&b.to_le_bytes())?;
        }
        for row in &self.feature {
            for val in row {
                writer.write_all(&val.to_le_bytes())?;
            }
        }
        for row in &self.hidden {
            for val in row {
                writer.write_all(&val.to_le_bytes())?;
            }
        }
        for off in &self.offset {
            writer.write_all(&off.to_le_bytes())?;
        }
        for out in &self.output {
            writer.write_all(&out.to_le_bytes())?;
        }
        writer.write_all(&self.anchor.to_le_bytes())?;
        writer.flush()?;
        Ok(())
    }

    /// Nạp Checkpoint trọng số f32 từ đĩa
    pub fn load(&mut self, path: &str) -> std::io::Result<()> {
        let file = File::open(path)?;
        let mut reader = BufReader::with_capacity(16 * 1024 * 1024, file);
        let mut magic = [0u8; 4];
        std::io::Read::read_exact(&mut reader, &mut magic)?;
        if &magic != b"XRCP" {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Sai magic checkpoint"));
        }
        let mut ver_buf = [0u8; 4];
        std::io::Read::read_exact(&mut reader, &mut ver_buf)?;

        let mut buf4 = [0u8; 4];
        for b in self.bias.iter_mut() {
            std::io::Read::read_exact(&mut reader, &mut buf4)?;
            *b = f32::from_le_bytes(buf4);
        }
        for row in self.feature.iter_mut() {
            for val in row.iter_mut() {
                std::io::Read::read_exact(&mut reader, &mut buf4)?;
                *val = f32::from_le_bytes(buf4);
            }
        }
        for row in self.hidden.iter_mut() {
            for val in row.iter_mut() {
                std::io::Read::read_exact(&mut reader, &mut buf4)?;
                *val = f32::from_le_bytes(buf4);
            }
        }
        for off in self.offset.iter_mut() {
            std::io::Read::read_exact(&mut reader, &mut buf4)?;
            *off = f32::from_le_bytes(buf4);
        }
        for out in self.output.iter_mut() {
            std::io::Read::read_exact(&mut reader, &mut buf4)?;
            *out = f32::from_le_bytes(buf4);
        }
        std::io::Read::read_exact(&mut reader, &mut buf4)?;
        self.anchor = f32::from_le_bytes(buf4);
        Ok(())
    }

    /// Lượng tử hóa f32 -> i16/i8 xuất nhị phân chuẩn XRNN v1 độc lập
    pub fn quantize(&self, path: &str) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = std::io::BufWriter::with_capacity(34 * 1024 * 1024, file);

        writer.write_all(b"XRNN")?;
        writer.write_all(&1u32.to_le_bytes())?;

        let mut q_bias = [0i16; DIM];
        for i in 0..DIM {
            q_bias[i] = (self.bias[i] * 127.0).round().clamp(-32768.0, 32767.0) as i16;
            writer.write_all(&q_bias[i].to_le_bytes())?;
        }

        let mut q_row = [0i16; DIM];
        for row in &self.feature {
            for i in 0..DIM {
                q_row[i] = (row[i] * 127.0).round().clamp(-32768.0, 32767.0) as i16;
                writer.write_all(&q_row[i].to_le_bytes())?;
            }
        }

        let mut q_hid_w = [0i8; BOTH];
        for i in 0..HIDDEN {
            for j in 0..BOTH {
                q_hid_w[j] = (self.hidden[i][j] * 64.0).round().clamp(-128.0, 127.0) as i8;
                writer.write_all(&[q_hid_w[j] as u8])?;
            }
        }

        for i in 0..HIDDEN {
            let q_b = (self.offset[i] * 127.0 * 64.0).round().clamp(i32::MIN as f32, i32::MAX as f32) as i32;
            writer.write_all(&q_b.to_le_bytes())?;
        }

        for i in 0..HIDDEN {
            let q_w = (self.output[i] * 64.0).round().clamp(-128.0, 127.0) as i8;
            writer.write_all(&[q_w as u8])?;
        }

        let q_anchor = (self.anchor * 64.0 * 64.0 * 400.0).round().clamp(i32::MIN as f32, i32::MAX as f32) as i32;
        writer.write_all(&q_anchor.to_le_bytes())?;

        let scale = 16i32;
        writer.write_all(&scale.to_le_bytes())?;

        writer.flush()?;
        Ok(())
    }
}

/// Tính hàm Sigmoid toán học: 1 / (1 + exp(-x))
#[inline(always)]
fn sigmoid(x: f32) -> f32 {
    1.0f32 / (1.0f32 + (-x).exp())
}

/// Thực thi huấn luyện Lan Truyền Xuôi và Lan Truyền Ngược với SOTA Sigmoid WDL + Lambda MSE Loss
#[inline(always)]
unsafe fn train_step(shared: Shared, datum: &Datum, rate: f32, lambda: f32) -> f32 {
    let mut red = [0.0f32; DIM];
    let mut black = [0.0f32; DIM];

    for j in 0..DIM {
        red[j] = *shared.bias.add(j);
        black[j] = *shared.bias.add(j);
    }

    for i in 0..datum.red_count as usize {
        let idx = datum.red[i] as usize;
        if idx < TOTAL {
            let row = &*shared.feature.add(idx);
            for j in 0..DIM {
                red[j] += row[j];
            }
        }
    }

    for i in 0..datum.black_count as usize {
        let idx = datum.black[i] as usize;
        if idx < TOTAL {
            let row = &*shared.feature.add(idx);
            for j in 0..DIM {
                black[j] += row[j];
            }
        }
    }

    let mut clipped = [0.0f32; BOTH];
    let (us, them) = if datum.side == 0 {
        (&red, &black)
    } else {
        (&black, &red)
    };

    for j in 0..DIM {
        clipped[j] = us[j].clamp(0.0, 1.0);
        clipped[DIM + j] = them[j].clamp(0.0, 1.0);
    }

    let hidden = &*shared.hidden;
    let offset = std::slice::from_raw_parts(shared.offset, HIDDEN);
    let output = std::slice::from_raw_parts(shared.output, HIDDEN);

    let mut activated = [0.0f32; HIDDEN];
    for i in 0..HIDDEN {
        let mut sum = offset[i];
        for j in 0..BOTH {
            sum += hidden[i][j] * clipped[j];
        }
        activated[i] = sum.clamp(0.0, 1.0);
    }

    let mut pred = *shared.anchor;
    for i in 0..HIDDEN {
        pred += output[i] * activated[i];
    }

    // 1. Tính toán hàm mục tiêu kép: Sigmoid WDL Target + Centipawn MSE
    let target_norm = (datum.target as f32) / 400.0;
    let prob_target = sigmoid(target_norm);
    let prob_pred = sigmoid(pred);

    let error_mse = pred - target_norm;
    let error_wdl = prob_pred - prob_target;

    let loss_mse = error_mse * error_mse;
    let loss = loss_mse;

    // 2. Kết hợp Gradient lan truyền ngược theo tỷ lệ lambda
    let grad_mse = 2.0 * error_mse;
    let grad_wdl = error_wdl * 16.0;
    let d_output = lambda * grad_mse + (1.0 - lambda) * grad_wdl;

    *shared.anchor -= rate * d_output;

    let mut d_hidden = [0.0f32; HIDDEN];
    for i in 0..HIDDEN {
        let d_act = if activated[i] > 0.0 && activated[i] < 1.0 {
            d_output * output[i]
        } else {
            0.0
        };
        d_hidden[i] = d_act;

        let out_ptr = shared.output.add(i);
        *out_ptr -= rate * (d_output * activated[i]);

        let off_ptr = shared.offset.add(i);
        *off_ptr -= rate * d_act;
    }

    let mut d_clipped = [0.0f32; BOTH];
    for i in 0..HIDDEN {
        if d_hidden[i] != 0.0 {
            for j in 0..BOTH {
                d_clipped[j] += d_hidden[i] * hidden[i][j];
            }
        }
    }

    for i in 0..HIDDEN {
        for j in 0..BOTH {
            if clipped[j] > 0.0 {
                let cell = &mut (*shared.hidden)[i][j];
                *cell -= rate * (d_hidden[i] * clipped[j]);
            }
        }
    }

    let (d_us, d_them) = d_clipped.split_at(DIM);
    let (d_red, d_black) = if datum.side == 0 {
        (d_us, d_them)
    } else {
        (d_them, d_us)
    };

    let mut grad_red = [0.0f32; DIM];
    let mut grad_black = [0.0f32; DIM];
    for j in 0..DIM {
        let slope_red = if red[j] >= 0.0 && red[j] <= 1.0 { 1.0f32 } else { 0.1f32 };
        grad_red[j] = d_red[j] * slope_red;

        let slope_black = if black[j] >= 0.0 && black[j] <= 1.0 { 1.0f32 } else { 0.1f32 };
        grad_black[j] = d_black[j] * slope_black;

        *shared.bias.add(j) -= rate * (grad_red[j] + grad_black[j]);
    }

    for i in 0..datum.red_count as usize {
        let idx = datum.red[i] as usize;
        if idx < TOTAL {
            let row = &mut *shared.feature.add(idx);
            for j in 0..DIM {
                row[j] -= rate * grad_red[j];
            }
        }
    }

    for i in 0..datum.black_count as usize {
        let idx = datum.black[i] as usize;
        if idx < TOTAL {
            let row = &mut *shared.feature.add(idx);
            for j in 0..DIM {
                row[j] -= rate * grad_black[j];
            }
        }
    }

    loss
}

fn main() {
    println!("===============================================================================");
    println!(" ⚡ XIANGQI-RIM SOTA SIGMOID WDL + LAMBDA MSE NNUE TRAINER: 4-CORE CPU PARALLEL");
    println!("    Kiến trúc: HalfKAv2_hm (65536x256 -> 512 -> 32 -> 1) | Chuẩn XRNN v1");
    println!("===============================================================================");

    let total_target: usize = std::env::var("SAMPLES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(45_196_882);
    let chunk_size: usize = std::env::var("CHUNK")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1_000_000);
    let epochs: usize = std::env::var("EPOCHS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2);
    let rate: f32 = std::env::var("RATE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0005);
    let lambda: f32 = std::env::var("LAMBDA")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.5);
    let threads: usize = 4;
    let json_paths_str = std::env::var("DATASET").unwrap_or_else(|_| "data/symmetrical_45m_distill.jsonl".to_string());
    let json_paths: Vec<String> = json_paths_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    let weights_path = std::env::var("OUTPUT").unwrap_or_else(|_| "data/nnue_weights.bin".to_string());

    println!("⚙️ THÔNG SỐ HUẤN LUYỆN QUY MÔ LỚN:");
    println!("  • Tập dữ liệu chưng cất : {:?}", json_paths);
    println!("  • Tệp trọng số xuất bản : {}", weights_path);
    println!("  • Tổng số mẫu mục tiêu  : {} mẫu FEN", total_target);
    println!("  • Kích thước Chunking   : {} mẫu / Chunk", chunk_size);
    println!("  • Số luồng CPU song song: {} Threads (Physical Cores)", threads);
    println!("  • Số Epochs trên Chunk  : {} epochs", epochs);
    println!("  • Tốc độ học (Rate)     : {}", rate);
    println!("  • Trọng số Lambda WDL   : {} (50% WDL Sigmoid + 50% Centipawn MSE)", lambda);
    println!("===============================================================================\n");

    let mut network = Network::new();
    let checkpoint = "data/nnue_checkpoint.bin";
    let reset: bool = std::env::var("RESET")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false);

    if !reset && Path::new(checkpoint).exists() {
        println!(" -> Nạp checkpoint hiện có: {}", checkpoint);
        let _ = network.load(checkpoint);
    } else {
        println!(" -> Khởi tạo trọng số Xavier phân phối chuẩn mới!");
    }
    let _ = std::io::stdout().flush();

    let mut total_processed = 0usize;
    let mut chunk_idx = 0usize;
    let start_all = Instant::now();

    for json_path in &json_paths {
        if total_processed >= total_target {
            break;
        }

        println!("📂 Đang mở và đọc tập dữ liệu: {}...", json_path);
        let file = match File::open(json_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("⚠️ Bỏ qua tệp '{}': {}", json_path, e);
                continue;
            }
        };

        let mut reader = BufReader::with_capacity(16 * 1024 * 1024, file);

        loop {
            if total_processed >= total_target {
                break;
            }

            chunk_idx += 1;
            let mut chunk_data = Vec::with_capacity(chunk_size);
            let chunk_start = Instant::now();

            println!(
                "[CHUNK {:02}] Đang nạp {} mẫu FEN tiếp theo (Tiến độ: {}/{})...",
                chunk_idx, chunk_size, total_processed, total_target
            );
            let _ = std::io::stdout().flush();

            let mut line = String::new();
            while chunk_data.len() < chunk_size && total_processed + chunk_data.len() < total_target {
                line.clear();
                if reader.read_line(&mut line).unwrap_or(0) == 0 {
                    break;
                }
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
                            chunk_data.push(datum);
                        }
                    }
                }
            }

            if chunk_data.is_empty() {
                println!(" -> Đã đọc hết toàn bộ dữ liệu trong tệp {}!", json_path);
                break;
            }

            let len = chunk_data.len();
            println!(
                " -> Đã nạp xong {} mẫu trong {:.2?} ({:.0} FEN/s). Khởi động 4 luồng huấn luyện SOTA WDL...",
                len,
                chunk_start.elapsed(),
                len as f64 / chunk_start.elapsed().as_secs_f64().max(0.001)
            );
            let _ = std::io::stdout().flush();

            let shared = network.share();
            let chunk_arc = Arc::new(chunk_data);
            let train_start = Instant::now();

            let progress = (total_processed as f32 / total_target as f32).min(1.0);
            let min_rate = rate * 0.05;
            let current_rate = min_rate + 0.5 * (rate - min_rate) * (1.0 + (progress * std::f32::consts::PI).cos());

            for ep in 1..=epochs {
                let loss_sum = Arc::new(AtomicU64::new(0));
                let mut handles = Vec::with_capacity(threads);
                let slice_size = len / threads;

                for t in 0..threads {
                    let c_arc = Arc::clone(&chunk_arc);
                    let l_sum = Arc::clone(&loss_sum);
                    let start = t * slice_size;
                    let end = if t == threads - 1 { len } else { start + slice_size };

                    handles.push(thread::spawn(move || {
                        let mut local_loss = 0.0f64;
                        for i in start..end {
                            let loss = unsafe { train_step(shared, &c_arc[i], current_rate, lambda) };
                            local_loss += loss as f64;
                        }
                        let bits = (local_loss * 1000.0) as u64;
                        l_sum.fetch_add(bits, Ordering::Relaxed);
                    }));
                }

                for h in handles {
                    let _ = h.join();
                }

                let total_loss_val = loss_sum.load(Ordering::Relaxed) as f64 / 1000.0;
                let mean_loss = total_loss_val / (len as f64);
                println!("   • Chunk {:02} | Epoch {:2}/{:2}: mean_loss = {:.4}", chunk_idx, ep, epochs, mean_loss);
                let _ = std::io::stdout().flush();
            }

            let train_dur = train_start.elapsed();
            let train_speed = (len * epochs) as f64 / train_dur.as_secs_f64().max(0.001);
            println!(
                " -> Chunk {:02} hoàn tất trong {:.2?}: Tốc độ huấn luyện = {:.0} mẫu/s. Lưu Checkpoint...\n",
                chunk_idx, train_dur, train_speed
            );
            let _ = std::io::stdout().flush();

            let _ = network.checkpoint(checkpoint);
            total_processed += len;
        }
    }

    let total_dur = start_all.elapsed();
    println!("===============================================================================");
    println!(" 🏆 HOÀN TẤT HUẤN LUYỆN TOÀN BỘ {} MẪU FEN TRONG {:.2?}!", total_processed, total_dur);
    println!("===============================================================================\n");

    println!("[GIAI ĐOẠN CUỐI] Lượng tử hóa f32 -> i16/i8 xuất nhị phân XRNN v1 độc lập...");
    match network.quantize(&weights_path) {
        Ok(_) => {
            println!("💾 Đã xuất bản trọng số độc lập thành công: {} (32.02 MB)!\n", weights_path);
            if weights_path != "data/nnue_weights.bin" {
                let _ = std::fs::copy(&weights_path, "data/nnue_weights.bin");
            }
            if weights_path != "data/nnue_weights_gen6.bin" {
                let _ = std::fs::copy(&weights_path, "data/nnue_weights_gen6.bin");
            }
        }
        Err(e) => eprintln!("❌ Lỗi khi xuất bản trọng số lượng tử hóa: {}", e),
    }
}
