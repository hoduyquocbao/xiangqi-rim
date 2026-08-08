// ============================================================================
// MODULE LEARN NNUE: HUẤN LUYỆN MẠNG NƠ-RON NNUE TỪ DỮ LIỆU TỰ ĐẤU
// ============================================================================
// Triển khai huấn luyện Backpropagation cho kiến trúc NNUE HalfKAv2_hm:
//   Feature Transformer (65536 → 256) → ClipReLU → Concat(512) →
//   Hidden Affine (512 → 32) → ClipReLU → Output (32 → 1)
//
// Toàn bộ huấn luyện sử dụng kiểu f32 (dấu phẩy động 32-bit) cho:
//   - Gradient computation chính xác
//   - Adam/SGD optimizer ổn định
//   - Loss function MSE liên tục khả vi
//
// Sau khi huấn luyện, trọng số f32 được lượng tử hóa (Quantize) về i16/i8
// cho inference tốc độ cao trên SIMD hardware.
//
// Định danh đơn từ: network, layer, gradient, optimizer, loss, batch,
// epoch, sample, position, score, feature, forward, backward, quantize,
// weight, bias, input, output, hidden, delta, rate, momentum, decay
// ============================================================================

use std::io::{Read, Write};

use crate::board::Position;
use crate::eval::feature::Feature;
use crate::eval::weight::{DIM, TOTAL};

/// Kích thước lớp ẩn ghép nối (256 Red + 256 Black = 512)
const BOTH: usize = DIM * 2;
/// Kích thước lớp ẩn affine
const HIDDEN: usize = 32;

// ============================================================================
// CẤU TRÚC MẪU HUẤN LUYỆN (TRAINING SAMPLE)
// ============================================================================

/// Số lượng đặc trưng tích cực tối đa cho 1 vị trí (32 quân × 2 góc nhìn)
const MAX: usize = 64;

/// Struct `Datum` lưu trữ 1 mẫu huấn luyện NNUE: danh sách đặc trưng tích cực + điểm mục tiêu.
#[repr(C)]
#[derive(Clone, Debug)]
pub struct Datum {
    /// Chỉ số đặc trưng tích cực cho góc nhìn phe Đỏ
    pub red: [u16; MAX],
    /// Số lượng đặc trưng tích cực phe Đỏ
    pub red_count: u8,
    /// Chỉ số đặc trưng tích cực cho góc nhìn phe Đen
    pub black: [u16; MAX],
    /// Số lượng đặc trưng tích cực phe Đen
    pub black_count: u8,
    /// Phe nắm lượt đi (0: Đỏ, 1: Đen)
    pub side: u8,
    /// Điểm số mục tiêu (ground truth) từ Engine Search (centipawn)
    pub target: i16,
}

impl Datum {
    /// Khởi tạo mẫu rỗng.
    pub fn new() -> Self {
        Self {
            red: [0u16; MAX],
            red_count: 0,
            black: [0u16; MAX],
            black_count: 0,
            side: 0,
            target: 0,
        }
    }

    /// Trích xuất đặc trưng NNUE từ vị trí bàn cờ `pos` với điểm mục tiêu `score`.
    pub fn extract(pos: &Position, score: i16) -> Self {
        let mut datum = Self::new();
        datum.side = pos.side;
        datum.target = score;

        let red_king = pos.king[0];
        let black_king = pos.king[1];

        for square in 0u8..90 {
            let piece = pos.grid[square as usize];
            if piece >= 14 {
                continue;
            }

            // Đặc trưng cho góc nhìn phe Đỏ
            let red_idx = Feature::index(red_king, piece, square, pos.side, 0);
            if red_idx < TOTAL && (datum.red_count as usize) < MAX {
                datum.red[datum.red_count as usize] = red_idx as u16;
                datum.red_count += 1;
            }

            // Đặc trưng cho góc nhìn phe Đen
            let black_idx = Feature::index(black_king, piece, square, pos.side, 1);
            if black_idx < TOTAL && (datum.black_count as usize) < MAX {
                datum.black[datum.black_count as usize] = black_idx as u16;
                datum.black_count += 1;
            }
        }

        datum
    }
}

// ============================================================================
// MẠNG NƠ-RON F32 CHO HUẤN LUYỆN (TRAINING NETWORK)
// ============================================================================

/// Struct `Network` chứa toàn bộ trọng số f32 của mạng NNUE cho huấn luyện.
/// Kiến trúc: Feature(65536 → 256) → Concat(512) → Hidden(512 → 32) → Output(32 → 1)
pub struct Network {
    /// Trọng số Feature Transformer: mỗi đặc trưng ánh xạ tới vector 256 chiều f32
    /// Lưu trữ: Vec<[f32; DIM]> với TOTAL phần tử (65536 × 256 = ~64MB f32)
    pub feature: Vec<[f32; DIM]>,
    /// Định thiên Feature Transformer: 256 phần tử f32
    pub bias: [f32; DIM],
    /// Trọng số lớp ẩn: 32 × 512 f32
    pub hidden: [[f32; BOTH]; HIDDEN],
    /// Định thiên lớp ẩn: 32 f32
    pub offset: [f32; HIDDEN],
    /// Trọng số lớp đầu ra: 32 f32
    pub output: [f32; HIDDEN],
    /// Định thiên lớp đầu ra: 1 f32
    pub anchor: f32,
}

impl Network {
    /// Khởi tạo mạng nơ-ron với trọng số ngẫu nhiên nhỏ (Xavier initialization).
    pub fn new() -> Box<Self> {
        let mut net = unsafe {
            let layout = std::alloc::Layout::new::<Self>();
            let ptr = std::alloc::alloc_zeroed(layout) as *mut Self;
            Box::from_raw(ptr)
        };

        // Xavier initialization cho Feature Transformer
        let scale = 1.0f32 / (DIM as f32).sqrt();
        let mut seed = 42u64;
        net.feature = vec![[0.0f32; DIM]; TOTAL];
        for i in 0..TOTAL {
            for j in 0..DIM {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                let val = ((seed >> 33) as f32 / (u32::MAX as f32) - 0.5) * 2.0 * scale;
                net.feature[i][j] = val;
            }
        }

        // Xavier initialization cho Hidden Layer
        let scale = 1.0f32 / (BOTH as f32).sqrt();
        for i in 0..HIDDEN {
            for j in 0..BOTH {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                let val = ((seed >> 33) as f32 / (u32::MAX as f32) - 0.5) * 2.0 * scale;
                net.hidden[i][j] = val;
            }
        }

        // Xavier initialization cho Output Layer
        let scale = 1.0f32 / (HIDDEN as f32).sqrt();
        for i in 0..HIDDEN {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let val = ((seed >> 33) as f32 / (u32::MAX as f32) - 0.5) * 2.0 * scale;
            net.output[i] = val;
        }

        net
    }

    /// Lan truyền tiến (Forward Pass) qua toàn bộ mạng, trả về (score, intermediates cho backward).
    pub fn forward(&self, datum: &Datum) -> (f32, Forward) {
        let mut state = Forward::new();

        // Feature Transformer: Tính tích lũy cho phe Đỏ
        state.red = self.bias;
        for i in 0..datum.red_count as usize {
            let idx = datum.red[i] as usize;
            if idx < TOTAL {
                for j in 0..DIM {
                    state.red[j] += self.feature[idx][j];
                }
            }
        }

        // Feature Transformer: Tính tích lũy cho phe Đen
        state.black = self.bias;
        for i in 0..datum.black_count as usize {
            let idx = datum.black[i] as usize;
            if idx < TOTAL {
                for j in 0..DIM {
                    state.black[j] += self.feature[idx][j];
                }
            }
        }

        // Clipped ReLU [0, 1] và ghép nối 512 chiều
        let (us, them) = if datum.side == 0 {
            (&state.red, &state.black)
        } else {
            (&state.black, &state.red)
        };
        for j in 0..DIM {
            state.clipped[j] = us[j].clamp(0.0, 1.0);
            state.clipped[DIM + j] = them[j].clamp(0.0, 1.0);
        }

        // Hidden Layer: y = W·x + b, sau đó ClipReLU [0, 1]
        for i in 0..HIDDEN {
            let mut sum = self.offset[i];
            for j in 0..BOTH {
                sum += self.hidden[i][j] * state.clipped[j];
            }
            state.pre[i] = sum;
            state.post[i] = sum.clamp(0.0, 1.0);
        }

        // Output Layer: score = w·h + b
        let mut score = self.anchor;
        for i in 0..HIDDEN {
            score += self.output[i] * state.post[i];
        }

        // Scale: chuyển đổi sang centipawn (nhân 400 để có khoảng giá trị hợp lý)
        let centipawn = score * 400.0;

        (centipawn, state)
    }

    /// Lan truyền ngược (Backward Pass) tính gradient và cập nhật trọng số.
    pub fn backward(
        &mut self,
        datum: &Datum,
        state: &Forward,
        predicted: f32,
        rate: f32,
    ) -> f32 {
        let target = datum.target as f32;
        let error = predicted - target;
        let loss = error * error;

        // Gradient đầu ra: d_loss/d_predicted = 2 * error
        let grad = 2.0 * error * rate;

        // Gradient cho Output Layer
        let scaled = grad / 400.0;
        self.anchor -= scaled;
        let mut hidden_grad = [0.0f32; HIDDEN];
        for i in 0..HIDDEN {
            self.output[i] -= scaled * state.post[i];
            // Gradient qua ClipReLU: 0 nếu pre < 0 hoặc pre > 1
            if state.pre[i] > 0.0 && state.pre[i] < 1.0 {
                hidden_grad[i] = scaled * self.output[i];
            }
        }

        // Gradient cho Hidden Layer
        let mut clipped_grad = [0.0f32; BOTH];
        for i in 0..HIDDEN {
            if hidden_grad[i] == 0.0 {
                continue;
            }
            self.offset[i] -= hidden_grad[i];
            for j in 0..BOTH {
                self.hidden[i][j] -= hidden_grad[i] * state.clipped[j];
                clipped_grad[j] += hidden_grad[i] * self.hidden[i][j];
            }
        }

        // Gradient qua Feature Transformer ClipReLU
        let (us_acc, them_acc) = if datum.side == 0 {
            (&state.red, &state.black)
        } else {
            (&state.black, &state.red)
        };
        let mut feature_grad_us = [0.0f32; DIM];
        let mut feature_grad_them = [0.0f32; DIM];
        for j in 0..DIM {
            if us_acc[j] > 0.0 && us_acc[j] < 1.0 {
                feature_grad_us[j] = clipped_grad[j];
            }
            if them_acc[j] > 0.0 && them_acc[j] < 1.0 {
                feature_grad_them[j] = clipped_grad[DIM + j];
            }
        }

        // Cập nhật bias Feature Transformer
        for j in 0..DIM {
            self.bias[j] -= feature_grad_us[j] + feature_grad_them[j];
        }

        // Cập nhật trọng số Feature Transformer cho phe mình (us)
        let us_features = if datum.side == 0 { &datum.red } else { &datum.black };
        let us_count = if datum.side == 0 { datum.red_count } else { datum.black_count };
        for i in 0..us_count as usize {
            let idx = us_features[i] as usize;
            if idx < TOTAL {
                for j in 0..DIM {
                    self.feature[idx][j] -= feature_grad_us[j];
                }
            }
        }

        // Cập nhật trọng số Feature Transformer cho phe đối phương (them)
        let them_features = if datum.side == 0 { &datum.black } else { &datum.red };
        let them_count = if datum.side == 0 { datum.black_count } else { datum.red_count };
        for i in 0..them_count as usize {
            let idx = them_features[i] as usize;
            if idx < TOTAL {
                for j in 0..DIM {
                    self.feature[idx][j] -= feature_grad_them[j];
                }
            }
        }

        loss
    }

    /// Lượng tử hóa (Quantize) trọng số f32 → i16/i8 cho inference SIMD tốc độ cao.
    /// Xuất ra tệp nhị phân tương thích với Nnue::load() trong eval module.
    pub fn quantize(&self, path: &str) -> std::io::Result<()> {
        let mut file = std::fs::File::create(path)?;

        // Magic header "XRNN" + Version 1
        file.write_all(b"XRNN")?;
        file.write_all(&1u32.to_le_bytes())?;

        // Feature Transformer bias: 256 × i16
        let scale_ft = 127.0f32;
        for j in 0..DIM {
            let val = (self.bias[j] * scale_ft).round().clamp(-32768.0, 32767.0) as i16;
            file.write_all(&val.to_le_bytes())?;
        }

        // Feature Transformer weights: TOTAL × DIM × i16
        for i in 0..TOTAL {
            for j in 0..DIM {
                let val = (self.feature[i][j] * scale_ft).round().clamp(-32768.0, 32767.0) as i16;
                file.write_all(&val.to_le_bytes())?;
            }
        }

        // Hidden Layer weights: HIDDEN × BOTH × i8
        let scale_hl = 64.0f32;
        for i in 0..HIDDEN {
            for j in 0..BOTH {
                let val = (self.hidden[i][j] * scale_hl).round().clamp(-128.0, 127.0) as i8;
                file.write_all(&[val as u8])?;
            }
        }

        // Hidden Layer bias: HIDDEN × i32
        let scale_hb = scale_ft * scale_hl;
        for i in 0..HIDDEN {
            let val = (self.offset[i] * scale_hb).round().clamp(-2147483648.0, 2147483647.0) as i32;
            file.write_all(&val.to_le_bytes())?;
        }

        // Output Layer weights: HIDDEN × i8
        let scale_ol = 64.0f32;
        for i in 0..HIDDEN {
            let val = (self.output[i] * scale_ol).round().clamp(-128.0, 127.0) as i8;
            file.write_all(&[val as u8])?;
        }

        // Output Layer bias: i32
        let scale_ob = scale_hl * scale_ol;
        let anchor = (self.anchor * scale_ob * 400.0).round().clamp(-2147483648.0, 2147483647.0) as i32;
        file.write_all(&anchor.to_le_bytes())?;

        // Output scale: i32 (mặc định 16 cho tương thích)
        let scale_val = 16i32;
        file.write_all(&scale_val.to_le_bytes())?;

        println!("[NNUE TRAINER] Đã xuất trọng số lượng tử hóa: {}", path);
        Ok(())
    }

    /// Nạp trọng số f32 từ tệp nhị phân checkpoint.
    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let mut file = std::fs::File::create(path)?;
        file.write_all(b"XRNF")?;
        file.write_all(&1u32.to_le_bytes())?;

        // Bias
        for j in 0..DIM {
            file.write_all(&self.bias[j].to_le_bytes())?;
        }

        // Feature weights (lớn — ~64MB)
        for i in 0..TOTAL {
            for j in 0..DIM {
                file.write_all(&self.feature[i][j].to_le_bytes())?;
            }
        }

        // Hidden weights
        for i in 0..HIDDEN {
            for j in 0..BOTH {
                file.write_all(&self.hidden[i][j].to_le_bytes())?;
            }
        }

        // Hidden bias
        for i in 0..HIDDEN {
            file.write_all(&self.offset[i].to_le_bytes())?;
        }

        // Output weights
        for i in 0..HIDDEN {
            file.write_all(&self.output[i].to_le_bytes())?;
        }

        // Output bias
        file.write_all(&self.anchor.to_le_bytes())?;

        Ok(())
    }

    /// Nạp trọng số f32 từ checkpoint.
    pub fn load(&mut self, path: &str) -> std::io::Result<()> {
        let mut file = std::fs::File::open(path)?;
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)?;
        if &magic != b"XRNF" {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid magic"));
        }
        let mut version = [0u8; 4];
        file.read_exact(&mut version)?;

        // Bias
        for j in 0..DIM {
            let mut buf = [0u8; 4];
            file.read_exact(&mut buf)?;
            self.bias[j] = f32::from_le_bytes(buf);
        }

        // Feature weights
        for i in 0..TOTAL {
            for j in 0..DIM {
                let mut buf = [0u8; 4];
                file.read_exact(&mut buf)?;
                self.feature[i][j] = f32::from_le_bytes(buf);
            }
        }

        // Hidden weights
        for i in 0..HIDDEN {
            for j in 0..BOTH {
                let mut buf = [0u8; 4];
                file.read_exact(&mut buf)?;
                self.hidden[i][j] = f32::from_le_bytes(buf);
            }
        }

        // Hidden bias
        for i in 0..HIDDEN {
            let mut buf = [0u8; 4];
            file.read_exact(&mut buf)?;
            self.offset[i] = f32::from_le_bytes(buf);
        }

        // Output weights
        for i in 0..HIDDEN {
            let mut buf = [0u8; 4];
            file.read_exact(&mut buf)?;
            self.output[i] = f32::from_le_bytes(buf);
        }

        // Output bias
        let mut buf = [0u8; 4];
        file.read_exact(&mut buf)?;
        self.anchor = f32::from_le_bytes(buf);

        Ok(())
    }
}

// ============================================================================
// TRẠNG THÁI LAN TRUYỀN TIẾN (FORWARD STATE)
// ============================================================================

/// Struct `Forward` lưu trữ kết quả trung gian của Forward Pass để sử dụng trong Backward Pass.
pub struct Forward {
    /// Tích lũy phe Đỏ sau Feature Transformer (256 f32)
    pub red: [f32; DIM],
    /// Tích lũy phe Đen sau Feature Transformer (256 f32)
    pub black: [f32; DIM],
    /// Kết quả sau ClipReLU và ghép nối 512 chiều
    pub clipped: [f32; BOTH],
    /// Giá trị trước ClipReLU của Hidden Layer (32 f32)
    pub pre: [f32; HIDDEN],
    /// Giá trị sau ClipReLU của Hidden Layer (32 f32)
    pub post: [f32; HIDDEN],
}

impl Forward {
    /// Khởi tạo trạng thái rỗng.
    pub fn new() -> Self {
        Self {
            red: [0.0f32; DIM],
            black: [0.0f32; DIM],
            clipped: [0.0f32; BOTH],
            pre: [0.0f32; HIDDEN],
            post: [0.0f32; HIDDEN],
        }
    }
}

// ============================================================================
// BÀI KIỂM THỬ ĐƠN VỊ
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Parser;

    #[test]
    fn extract() {
        let pos = Parser::parse(Parser::DEFAULT);
        let datum = Datum::extract(&pos, 0);
        assert!(datum.red_count > 0);
        assert!(datum.black_count > 0);
        assert_eq!(datum.side, 0);
        assert_eq!(datum.target, 0);
    }

    #[test]
    fn forward() {
        let net = Network::new();
        let pos = Parser::parse(Parser::DEFAULT);
        let datum = Datum::extract(&pos, 0);
        let (score, _state) = net.forward(&datum);
        assert!(score.is_finite(), "Score phải là số hữu hạn: {}", score);
    }

    #[test]
    fn backward() {
        let mut net = Network::new();
        let pos = Parser::parse(Parser::DEFAULT);
        let datum = Datum::extract(&pos, 50);
        let (predicted, state) = net.forward(&datum);
        let loss = net.backward(&datum, &state, predicted, 0.001);
        assert!(loss >= 0.0, "Loss phải không âm: {}", loss);

        // Kiểm tra loss giảm sau 1 bước
        let (predicted2, _state2) = net.forward(&datum);
        let loss2 = (predicted2 - 50.0) * (predicted2 - 50.0);
        assert!(loss2 < loss, "Loss phải giảm sau backward: {} vs {}", loss2, loss);
    }
}
