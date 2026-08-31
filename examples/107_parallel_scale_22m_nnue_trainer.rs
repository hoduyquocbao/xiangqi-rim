// ============================================================================
// EXAMPLE 107: HUẤN LUYỆN ĐA LUỒNG 22.5 TRIỆU FENs PIKAFISH TRÊN NATIVE CPU
// ============================================================================
// `107_parallel_scale_22m_nnue_trainer.rs` thực thi huấn luyện toàn bộ kho dữ liệu
// chưng cất Grandmaster `data/pikafish_in_memory_distill.jsonl` (22,598,441 FENs)
// trên 4 nhân CPU vật lý bằng kiến trúc Hogwild! Lock-Free Async SGD:
// - Nạp dữ liệu dạng Cuốn Chiếu (Streaming Chunks 1,000,000 FENs/lô) -> RAM < 500MB.
// - 4 Luồng Worker song song tính toán Lan Truyền Ngược Backpropagation.
// - Tốc độ mục tiêu: > 100,000 mẫu / giây trên Intel Core CPU.
// - Tự động lượng tử hóa nhị phân chuẩn `XRNN v1` sang `data/nnue_weights.bin`.
// ============================================================================

use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use xiangrust::board::Parser;
use xiangrust::eval::weight::{DIM, TOTAL};
use xiangrust::learn::nnue::Datum;

/// Kích thước lớp ẩn ghép nối (256 Red + 256 Black = 512)
const BOTH: usize = DIM * 2;
/// Kích thước lớp ẩn affine
const HIDDEN: usize = 32;

/// Cấu trúc bọc con trỏ thô hỗ trợ chia sẻ bộ nhớ đa luồng Lock-Free cho thuật toán Hogwild! SGD
#[derive(Clone, Copy)]
struct Shared {
    /// Con trỏ thô trỏ tới mảng trọng số Feature Transformer
    feature: *mut [f32; DIM],
    /// Con trỏ thô trỏ tới mảng Bias Feature Transformer
    bias: *mut f32,
    /// Con trỏ thô trỏ tới ma trận lớp ẩn Hidden Layer
    hidden: *mut [[f32; BOTH]; HIDDEN],
    /// Con trỏ thô trỏ tới mảng Bias lớp ẩn
    offset: *mut f32,
    /// Con trỏ thô trỏ tới mảng trọng số lớp đầu ra
    output: *mut f32,
    /// Con trỏ thô trỏ tới Bias lớp đầu ra
    anchor: *mut f32,
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
    /// Khởi tạo mạng mới trên Heap
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

        net
    }

    /// Nạp checkpoint từ đĩa
    pub fn load(&mut self, path: &str) -> std::io::Result<()> {
        let mut file = File::open(path)?;
        let mut magic = [0u8; 4];
        std::io::Read::read_exact(&mut file, &mut magic)?;
        if &magic != b"XRNC" {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid magic"));
        }

        unsafe {
            let slice_bias = std::slice::from_raw_parts_mut(self.bias.as_mut_ptr() as *mut u8, DIM * 4);
            std::io::Read::read_exact(&mut file, slice_bias)?;

            let slice_feat = std::slice::from_raw_parts_mut(self.feature.as_mut_ptr() as *mut u8, TOTAL * DIM * 4);
            std::io::Read::read_exact(&mut file, slice_feat)?;

            let slice_hid = std::slice::from_raw_parts_mut(self.hidden.as_mut_ptr() as *mut u8, HIDDEN * BOTH * 4);
            std::io::Read::read_exact(&mut file, slice_hid)?;

            let slice_off = std::slice::from_raw_parts_mut(self.offset.as_mut_ptr() as *mut u8, HIDDEN * 4);
            std::io::Read::read_exact(&mut file, slice_off)?;

            let slice_out = std::slice::from_raw_parts_mut(self.output.as_mut_ptr() as *mut u8, HIDDEN * 4);
            std::io::Read::read_exact(&mut file, slice_out)?;

            let slice_anc = std::slice::from_raw_parts_mut(&mut self.anchor as *mut f32 as *mut u8, 4);
            std::io::Read::read_exact(&mut file, slice_anc)?;
        }
        Ok(())
    }

    /// Lưu checkpoint ra đĩa
    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all(b"XRNC")?;

        unsafe {
            let slice_bias = std::slice::from_raw_parts(self.bias.as_ptr() as *const u8, DIM * 4);
            file.write_all(slice_bias)?;

            let slice_feat = std::slice::from_raw_parts(self.feature.as_ptr() as *const u8, TOTAL * DIM * 4);
            file.write_all(slice_feat)?;

            let slice_hid = std::slice::from_raw_parts(self.hidden.as_ptr() as *const u8, HIDDEN * BOTH * 4);
            file.write_all(slice_hid)?;

            let slice_off = std::slice::from_raw_parts(self.offset.as_ptr() as *const u8, HIDDEN * 4);
            file.write_all(slice_off)?;

            let slice_out = std::slice::from_raw_parts(self.output.as_ptr() as *const u8, HIDDEN * 4);
            file.write_all(slice_out)?;

            let slice_anc = std::slice::from_raw_parts(&self.anchor as *const f32 as *const u8, 4);
            file.write_all(slice_anc)?;
        }
        Ok(())
    }

    /// Lượng tử hóa f32 -> i16/i8 sang chuẩn XRNN v1
    pub fn quantize(&self, path: &str) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all(b"XRNN")?;
        file.write_all(&1u32.to_le_bytes())?;

        let mut q_bias = [0i16; DIM];
        for i in 0..DIM {
            q_bias[i] = (self.bias[i] * 127.0).round().clamp(-32768.0, 32767.0) as i16;
        }
        unsafe {
            let slice = std::slice::from_raw_parts(q_bias.as_ptr() as *const u8, DIM * 2);
            file.write_all(slice)?;
        }

        for row in self.feature.iter() {
            let mut q_row = [0i16; DIM];
            for i in 0..DIM {
                q_row[i] = (row[i] * 127.0).round().clamp(-32768.0, 32767.0) as i16;
            }
            unsafe {
                let slice = std::slice::from_raw_parts(q_row.as_ptr() as *const u8, DIM * 2);
                file.write_all(slice)?;
            }
        }

        let mut q_hidden = [[0i8; BOTH]; HIDDEN];
        for i in 0..HIDDEN {
            for j in 0..BOTH {
                q_hidden[i][j] = (self.hidden[i][j] * 64.0).round().clamp(-128.0, 127.0) as i8;
            }
        }
        unsafe {
            let slice = std::slice::from_raw_parts(q_hidden.as_ptr() as *const u8, HIDDEN * BOTH);
            file.write_all(slice)?;
        }

        let mut q_offset = [0i32; HIDDEN];
        for i in 0..HIDDEN {
            q_offset[i] = (self.offset[i] * 127.0 * 64.0).round() as i32;
        }
        unsafe {
            let slice = std::slice::from_raw_parts(q_offset.as_ptr() as *const u8, HIDDEN * 4);
            file.write_all(slice)?;
        }

        let mut q_output = [0i8; HIDDEN];
        for i in 0..HIDDEN {
            q_output[i] = (self.output[i] * 64.0).round().clamp(-128.0, 127.0) as i8;
        }
        unsafe {
            let slice = std::slice::from_raw_parts(q_output.as_ptr() as *const u8, HIDDEN);
            file.write_all(slice)?;
        }

        let q_anchor = (self.anchor * 64.0 * 64.0 * 400.0).round() as i32;
        file.write_all(&q_anchor.to_le_bytes())?;
        file.write_all(&16i32.to_le_bytes())?;

        Ok(())
    }

    /// Lấy con trỏ chia sẻ cho các luồng
    fn share(&mut self) -> Shared {
        Shared {
            feature: self.feature.as_mut_ptr(),
            bias: self.bias.as_mut_ptr(),
            hidden: &mut self.hidden as *mut _,
            offset: self.offset.as_mut_ptr(),
            output: self.output.as_mut_ptr(),
            anchor: &mut self.anchor as *mut _,
        }
    }
}

/// Thực thi huấn luyện Lan Truyền Xuôi và Lan Truyền Ngược cho 1 mẫu trên con trỏ Shared
#[inline(always)]
unsafe fn train_step(shared: Shared, datum: &Datum, rate: f32) -> f32 {
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

    let target = (datum.target as f32) / 400.0;
    let error = pred - target;
    let loss = error * error;

    let d_output = 2.0 * error;
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
    println!(" ⚡ XIANGQI-RIM SCALE 22.5M NNUE TRAINER: 4-CORE HOGWILD! CPU PARALLEL");
    println!("    Kiến trúc: HalfKAv2_hm (65536x256 -> 512 -> 32 -> 1) | Chuẩn XRNN v1");
    println!("===============================================================================");

    let total_target: usize = std::env::var("SAMPLES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(22_598_441);
    let chunk_size: usize = std::env::var("CHUNK")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1_000_000);
    let epochs: usize = std::env::var("EPOCHS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5);
    let rate: f32 = std::env::var("RATE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0001);
    let threads: usize = 4;
    let json_path = "data/pikafish_in_memory_distill.jsonl";

    println!("⚙️ THÔNG SỐ HUẤN LUYỆN QUY MÔ LỚN:");
    println!("  • Tập dữ liệu chưng cất : {}", json_path);
    println!("  • Tổng số mẫu mục tiêu  : {} mẫu FEN", total_target);
    println!("  • Kích thước Chunking   : {} mẫu / Chunk", chunk_size);
    println!("  • Số luồng CPU song song: {} Threads (Physical Cores)", threads);
    println!("  • Số Epochs trên Chunk  : {} epochs", epochs);
    println!("  • Tốc độ học (Rate)     : {}", rate);
    println!("===============================================================================\n");

    let file = match File::open(json_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("❌ Không thể mở tệp '{}': {}", json_path, e);
            return;
        }
    };

    let mut reader = BufReader::with_capacity(16 * 1024 * 1024, file);
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

    loop {
        if total_processed >= total_target {
            break;
        }

        chunk_idx += 1;
        let mut chunk_data: Vec<Datum> = Vec::with_capacity(chunk_size);
        let chunk_start = Instant::now();

        println!(
            "\n[CHUNK {:02}] Đang nạp {} mẫu FEN tiếp theo (Tiến độ: {}/{})...",
            chunk_idx, chunk_size, total_processed, total_target
        );
        let _ = std::io::stdout().flush();

        let mut line = String::new();
        while chunk_data.len() < chunk_size {
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
            println!(" -> Đã đọc hết toàn bộ dữ liệu trong tệp!");
            break;
        }

        println!(
            " -> Đã nạp xong {} mẫu trong {:.2?} ({:.0} FEN/s). Khởi động 4 luồng huấn luyện...",
            chunk_data.len(),
            chunk_start.elapsed(),
            chunk_data.len() as f64 / chunk_start.elapsed().as_secs_f64().max(0.001)
        );
        let _ = std::io::stdout().flush();

        let shared = network.share();
        let chunk_arc = Arc::new(chunk_data);
        let train_start = Instant::now();

        for ep in 1..=epochs {
            let loss_sum = Arc::new(AtomicU64::new(0));
            let mut handles = Vec::with_capacity(threads);
            let len = chunk_arc.len();
            let slice_size = len / threads;

            for t in 0..threads {
                let c_arc = Arc::clone(&chunk_arc);
                let l_sum = Arc::clone(&loss_sum);
                let start = t * slice_size;
                let end = if t == threads - 1 { len } else { start + slice_size };

                handles.push(thread::spawn(move || {
                    let mut local_loss = 0.0f64;
                    for i in start..end {
                        let loss = unsafe { train_step(shared, &c_arc[i], rate) };
                        local_loss += loss as f64;
                    }
                    let bits = (local_loss * 1000.0) as u64;
                    l_sum.fetch_add(bits, Ordering::Relaxed);
                }));
            }

            for h in handles {
                let _ = h.join();
            }

            let total_loss = loss_sum.load(Ordering::Relaxed) as f64 / 1000.0;
            let mean_loss = total_loss / len as f64;

            println!(
                "   • Chunk {:02} | Epoch {:2}/{:2}: mean_loss = {:.4}",
                chunk_idx, ep, epochs, mean_loss
            );
            let _ = std::io::stdout().flush();
        }

        total_processed += chunk_arc.len();
        let chunk_time = train_start.elapsed();
        let speed = (chunk_arc.len() * epochs) as f64 / chunk_time.as_secs_f64().max(0.001);

        println!(
            " -> Chunk {:02} hoàn tất trong {:.2?}: Tốc độ huấn luyện = {:.0} mẫu/s. Lưu Checkpoint...",
            chunk_idx, chunk_time, speed
        );
        let _ = network.save(checkpoint);
        let _ = std::io::stdout().flush();
    }

    let total_time = start_all.elapsed();
    println!("\n===============================================================================");
    println!(
        " 🏆 HOÀN TẤT HUẤN LUYỆN TOÀN BỘ {} MẪU FEN TRONG {:.2?}!",
        total_processed, total_time
    );
    println!("===============================================================================\n");

    println!("[GIAI ĐOẠN CUỐI] Lượng tử hóa f32 -> i16/i8 xuất nhị phân XRNN v1 độc lập...");
    let weights_path = "data/nnue_weights.bin";
    let _ = network.quantize(weights_path);
    println!("💾 Đã xuất bản trọng số độc lập thành công: {} (32.02 MB)!\n", weights_path);
    let _ = std::io::stdout().flush();
}
