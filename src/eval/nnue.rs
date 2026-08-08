// ============================================================================
// MODULE NNUE: MẠNG NƠ-RON ĐÁNH GIÁ ĐẶC TRƯNG HỮU HẠN (EVALUATION NNUE ENGINE)
// ============================================================================
// `nnue.rs` quản lý quá trình lan truyền tiến (Forward Propagation) của mạng nơ-ron NNUE:
// - `Simd`: Wrapper thực thi tích vô hướng SIMD (AVX2/NEON) siêu tốc.
// - `Clip`: Hàm kích hoạt Clipped ReLU `[0, 127]`.
// - `Sculu`: Hàm kích hoạt phi tuyến tính SCULU (Squared Clipped Linear Unit).
// - `Transform`: Lớp chuyển đổi đặc trưng tích lũy $2 \times 256 \rightarrow 512$ chiều $i8$.
// - `Affine<IN, OUT>`: Lớp liên kết đầy đủ (Fully Connected Layer) $512 \rightarrow 32$ chiều.
// - `Output<IN>`: Lớp đầu ra cho điểm số thế cờ $32 \rightarrow 1$ centipawn score.
// - `Nnue`: Struct điều phối tổng thể với căn lề 64-byte `#[repr(C, align(64))]`.
// ============================================================================

use std::mem::MaybeUninit;
use super::accum::Accum;
use super::weight::{Weight, DIM};

/// Kích thước một nửa không gian đặc trưng = 256
pub const HALF: usize = DIM;
/// Kích thước toàn bộ không gian đặc trưng ghép nối 2 phe = 512
pub const BOTH: usize = HALF * 2;

/// Struct `Simd` bọc các phép toán tích vô hướng tăng tốc phần cứng.
pub struct Simd;

impl Simd {
    /// Phép nhân vô hướng (dot product) giữa `i16` và `i8` trả về `i32` với SIMD.
    #[inline(always)]
    pub unsafe fn dot(input: &[i16], weight: &[i8]) -> i32 {
        crate::simd::dot(input, weight)
    }

    /// Phép nhân vô hướng hai mảng `i8` với `i8` trả về `i32` với SIMD.
    #[inline(always)]
    pub unsafe fn bytes(input: &[i8], weight: &[i8]) -> i32 {
        crate::simd::bytes(input, weight)
    }

    /// Phép nhân vô hướng dự phòng trên CPU scalar (không SIMD).
    #[inline(always)]
    pub fn scalar(input: &[i16], weight: &[i8]) -> i32 {
        crate::simd::scalar(input, weight)
    }
}

/// Struct `Clip` thực hiện hàm kích hoạt Clipped ReLU `[min, max]`.
pub struct Clip;

impl Clip {
    /// Kẹp giá trị `val` nằm trong khoảng $[min, max]$.
    #[inline(always)]
    pub fn clamp(val: i16, min: i16, max: i16) -> i8 {
        if val < min {
            min as i8
        } else if val > max {
            max as i8
        } else {
            val as i8
        }
    }
}

/// Struct `Sculu` thực hiện hàm kích hoạt bình phương Clipped ReLU (SCULU).
pub struct Sculu;

impl Sculu {
    /// Tính toán giá trị kích hoạt SCULU: $\text{SCULU}(x) = \frac{\text{Clamp}(x, 0, max)^2}{scale}$.
    #[inline(always)]
    pub fn active(val: i16, max: i16, scale: i32) -> i8 {
        let clamped = if val < 0 {
            0i32
        } else if val > max {
            max as i32
        } else {
            val as i32
        };
        let squared = clamped * clamped;
        let scaled = squared / scale;
        if scaled > 127 {
            127i8
        } else {
            scaled as i8
        }
    }
}

/// Struct `Transform` lưu trữ mảng kích hoạt 512 chiều $i8$, căn lề 64-byte.
#[repr(C, align(64))]
#[derive(Clone, Debug)]
pub struct Transform {
    /// Mảng kích hoạt 512 phần tử $i8$
    pub active: [i8; BOTH],
}

impl Default for Transform {
    /// Khởi tạo mặc định đối tượng Transform.
    fn default() -> Self {
        Self::new()
    }
}

impl Transform {
    /// Khởi tạo Transform với mảng 0.
    #[inline(always)]
    pub const fn new() -> Self {
        Self { active: [0; BOTH] }
    }

    /// Helper SIMD Vector Packing cho Clipped ReLU: Chuyển đổi mảng 256 phần tử `i16` thành `i8` kẹp `[0, 127]`.
    /// Triệt tiêu hoàn toàn 512 bước lặp scalar element-by-element bằng SIMD vector instructions.
    #[inline(always)]
    fn pack(src: &[i16; HALF], dst: &mut [i8]) {
        // Tăng tốc x86_64 AVX2: Đóng gói 32 phần tử i16 thành i8 mỗi vòng lặp
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                use std::arch::x86_64::*;
                let zero = unsafe { _mm256_setzero_si256() };
                let ceiling = unsafe { _mm256_set1_epi16(127) };
                let mut i = 0;
                while i < HALF {
                    unsafe {
                        let src_ptr = src.as_ptr().add(i) as *const __m256i;
                        let v0 = _mm256_load_si256(src_ptr.add(0));
                        let v1 = _mm256_load_si256(src_ptr.add(1));

                        // Thực hiện Kẹp Clipped ReLU [0, 127] trên 2 thanh ghi 256-bit
                        let c0 = _mm256_min_epi16(_mm256_max_epi16(v0, zero), ceiling);
                        let c1 = _mm256_min_epi16(_mm256_max_epi16(v1, zero), ceiling);

                        // Đóng gói saturated pack i16 -> i8
                        let packed = _mm256_packs_epi16(c0, c1);
                        // Hoán vị 4x64-bit qwords khôi phục thứ tự tuyến tính [c0_low, c0_high, c1_low, c1_high]
                        let permuted = _mm256_permute4x64_epi64(packed, 0b11_01_10_00);

                        let dst_ptr = dst.as_mut_ptr().add(i) as *mut __m256i;
                        _mm256_store_si256(dst_ptr, permuted);
                    }
                    i += 32;
                }
                return;
            }
        }

        // Tăng tốc ARM64 NEON: Đóng gói 32 phần tử i16 thành i8 mỗi vòng lặp
        #[cfg(target_arch = "aarch64")]
        {
            use std::arch::aarch64::*;
            let zero = unsafe { vdupq_n_s16(0) };
            let ceiling = unsafe { vdupq_n_s16(127) };
            let mut i = 0;
            while i < HALF {
                unsafe {
                    let src_ptr = src.as_ptr().add(i);
                    let v0 = vld1q_s16(src_ptr);
                    let v1 = vld1q_s16(src_ptr.add(8));
                    let v2 = vld1q_s16(src_ptr.add(16));
                    let v3 = vld1q_s16(src_ptr.add(24));

                    let c0 = vminq_s16(vmaxq_s16(v0, zero), ceiling);
                    let c1 = vminq_s16(vmaxq_s16(v1, zero), ceiling);
                    let c2 = vminq_s16(vmaxq_s16(v2, zero), ceiling);
                    let c3 = vminq_s16(vmaxq_s16(v3, zero), ceiling);

                    let pack0 = vcombine_s8(vmovn_s16(c0), vmovn_s16(c1));
                    let pack1 = vcombine_s8(vmovn_s16(c2), vmovn_s16(c3));

                    let dst_ptr = dst.as_mut_ptr().add(i);
                    vst1q_s8(dst_ptr, pack0);
                    vst1q_s8(dst_ptr.add(16), pack1);
                }
                i += 32;
            }
            return;
        }

        // Vòng lặp dự phòng Scalar khi không có SIMD
        let mut idx = 0usize;
        while idx < HALF {
            dst[idx] = Clip::clamp(src[idx], 0, 127);
            idx += 1;
        }
    }

    /// Biến đổi và áp dụng Clipped ReLU trên 2 mảng tích lũy `red` và `black` theo phe `side`.
    /// Đã tối ưu hóa bằng SIMD Clipped ReLU Vector Packing.
    #[inline(always)]
    pub fn active(&mut self, red: &[i16; HALF], black: &[i16; HALF], side: u8) {
        let (us, them) = if side == 0 {
            (red, black)
        } else {
            (black, red)
        };

        // Áp dụng SIMD Clipped ReLU đóng gói 256 phần tử cho phe mình
        Self::pack(us, &mut self.active[0..HALF]);

        // Áp dụng SIMD Clipped ReLU đóng gói 256 phần tử cho phe đối thủ
        Self::pack(them, &mut self.active[HALF..BOTH]);
    }
}

/// Struct `Affine<IN, OUT>` đại diện cho lớp ma trận tuyến tính $IN \rightarrow OUT$, căn lề 64-byte.
#[repr(C, align(64))]
#[derive(Clone, Debug)]
pub struct Affine<const IN: usize, const OUT: usize> {
    /// Ma trận trọng số $OUT \times IN$ kiểu $i8$
    pub weight: [[i8; IN]; OUT],
    /// Vector định thiên $OUT$ phần tử kiểu $i32$
    pub bias: [i32; OUT],
}

impl<const IN: usize, const OUT: usize> Default for Affine<IN, OUT> {
    /// Khởi tạo mặc định cho Affine.
    fn default() -> Self {
        Self::new()
    }
}

impl<const IN: usize, const OUT: usize> Affine<IN, OUT> {
    /// Khởi tạo lớp ma trận 0.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            weight: [[0; IN]; OUT],
            bias: [0; OUT],
        }
    }

    /// Tính toán lan truyền tiến ma trận $y = W \cdot x + b$ bằng SIMD.
    #[inline(always)]
    pub fn forward(&self, input: &[i8; IN], output: &mut [i32; OUT]) {
        let mut o = 0usize;
        while o < OUT {
            let sum = self.bias[o] + unsafe { Simd::bytes(input, &self.weight[o]) };
            output[o] = sum;
            o += 1;
        }
    }
}

/// Struct `Output<IN>` đại diện cho lớp đầu ra điểm số của mạng nơ-ron, căn lề 64-byte.
#[repr(C, align(64))]
#[derive(Clone, Debug)]
pub struct Output<const IN: usize> {
    /// Trọng số lớp đầu ra $IN$ phần tử $i8$
    pub weight: [i8; IN],
    /// Định thiên đầu ra kiểu $i32$
    pub bias: i32,
    /// Tỷ lệ chia điểm (scale factor, mặc định 16)
    pub scale: i32,
}

impl<const IN: usize> Default for Output<IN> {
    /// Khởi tạo mặc định lớp Output.
    fn default() -> Self {
        Self::new(16)
    }
}

impl<const IN: usize> Output<IN> {
    /// Khởi tạo lớp Output với tỷ lệ `scale`.
    #[inline(always)]
    pub const fn new(scale: i32) -> Self {
        Self {
            weight: [0; IN],
            bias: 0,
            scale,
        }
    }

    /// Đánh giá điểm số cuối cùng của vị trí (centipawn score).
    #[inline(always)]
    pub fn evaluate(&self, input: &[i8; IN]) -> i32 {
        let sum = self.bias + unsafe { Simd::bytes(input, &self.weight) };
        if self.scale != 0 {
            sum / self.scale
        } else {
            sum
        }
    }
}

/// Struct `Nnue` tập hợp các lớp nơ-ron NNUE tổng thể, căn lề 64-byte.
#[repr(C, align(64))]
#[derive(Clone, Debug)]
pub struct Nnue {
    /// Trọng số đặc trưng
    pub weight: Weight,
    /// Lớp tuyến tính ẩn $512 \rightarrow 32$
    pub affine: Box<Affine<BOTH, 32>>,
    /// Lớp đầu ra $32 \rightarrow 1$
    pub output: Output<32>,
    /// Cờ đánh dấu mạng nơ-ron đã nạp thành công
    pub loaded: bool,
}

impl Default for Nnue {
    /// Khởi tạo mặc định đối tượng Nnue.
    fn default() -> Self {
        Self::new()
    }
}

impl Nnue {
    /// Khởi tạo mạng nơ-ron NNUE mặc định với 0-byte stack overhead cho Affine layer.
    pub fn new() -> Self {
        let affine = unsafe {
            let layout = std::alloc::Layout::new::<Affine<BOTH, 32>>();
            let ptr = std::alloc::alloc_zeroed(layout) as *mut Affine<BOTH, 32>;
            Box::from_raw(ptr)
        };
        Self {
            weight: Weight::new(),
            affine,
            output: Output::new(16),
            loaded: false,
        }
    }

    /// Thực thi đánh giá vị trí qua toàn bộ mạng nơ-ron từ bộ tích lũy `accum` cho lượt đi `side`.
    /// Ép buộc inlining `#[inline(always)]` triệt tiêu chi phí gọi hàm trên hot path tìm kiếm.
    /// Tối ưu hóa: MaybeUninit bỏ qua khởi tạo 512-byte lãng phí cho Transform,
    /// và SIMD vectorize ClipReLU trên hidden layer 32 phần tử i32→i8.
    #[inline(always)]
    pub fn evaluate(&self, accum: &Accum, side: u8) -> i32 {
        // Sử dụng MaybeUninit bỏ qua zero-init 512 bytes cho Transform
        // vì hàm active() sẽ ghi đè 100% mảng ngay lập tức.
        // Tiết kiệm ~512 CPU cycles mỗi nút đánh giá.
        let mut uninit: MaybeUninit<Transform> = MaybeUninit::uninit();
        let transform = unsafe {
            let ptr = uninit.as_mut_ptr();
            // active() sẽ ghi đè toàn bộ mảng active[0..BOTH] qua SIMD pack()
            (*ptr).active(&accum.vals[0], &accum.vals[1], side);
            // An toàn vì active() đã khởi tạo toàn bộ trường duy nhất `active`
            uninit.assume_init_ref()
        };

        // Tính toán lớp ẩn (Hidden Layer) $y = W \cdot x + b$
        let mut hidden = [0i32; 32];
        self.affine.forward(&transform.active, &mut hidden);

        // SIMD vectorize ClipReLU: Chuyển đổi 32 phần tử i32 → i8 kẹp [0, 127]
        // thay vì vòng lặp scalar 32 bước
        let mut layer = [0i8; 32];
        Self::clip32(&hidden, &mut layer);

        self.output.evaluate(&layer) * 4
    }

    /// SIMD vectorize ClipReLU cho hidden layer 32 phần tử i32 → i8 kẹp [0, 127].
    /// ARM64 NEON: Xử lý 32 phần tử trong 2 bước (16 phần tử/bước) thay vì 32 bước scalar.
    /// x86_64 AVX2: Xử lý 32 phần tử trong 1 bước (32 phần tử/bước).
    #[inline(always)]
    fn clip32(src: &[i32; 32], dst: &mut [i8; 32]) {
        // Tăng tốc ARM64 NEON: 32 phần tử i32 → i8 kẹp [0, 127]
        #[cfg(target_arch = "aarch64")]
        {
            use std::arch::aarch64::*;
            unsafe {
                let zero = vdupq_n_s32(0);
                let ceil = vdupq_n_s32(127);

                // Xử lý 8 phần tử i32 thành 8 phần tử i16 (lặp 4 lần = 32 phần tử)
                let mut i = 0usize;
                while i < 32 {
                    // Nạp 4 phần tử i32 và kẹp [0, 127]
                    let v0 = vminq_s32(vmaxq_s32(vld1q_s32(src.as_ptr().add(i)), zero), ceil);
                    let v1 = vminq_s32(vmaxq_s32(vld1q_s32(src.as_ptr().add(i + 4)), zero), ceil);
                    // Thu hẹp i32 → i16 (2 thanh ghi × 4 = 8 phần tử i16)
                    let narrow16 = vcombine_s16(vmovn_s32(v0), vmovn_s32(v1));
                    // Thu hẹp i16 → i8 (8 phần tử → 8 phần tử i8)
                    let narrow8 = vmovn_s16(narrow16);
                    // Ghi 8 bytes vào đích
                    vst1_s8(dst.as_mut_ptr().add(i), narrow8);
                    i += 8;
                }
            }
            return;
        }

        // Tăng tốc x86_64 AVX2: 32 phần tử i32 → i8 kẹp [0, 127]
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                use std::arch::x86_64::*;
                unsafe {
                    let zero = _mm256_setzero_si256();
                    let ceil = _mm256_set1_epi32(127);

                    // Nạp và kẹp 4 khối × 8 phần tử i32
                    let v0 = _mm256_min_epi32(_mm256_max_epi32(
                        _mm256_loadu_si256(src.as_ptr().add(0) as *const __m256i), zero), ceil);
                    let v1 = _mm256_min_epi32(_mm256_max_epi32(
                        _mm256_loadu_si256(src.as_ptr().add(8) as *const __m256i), zero), ceil);
                    let v2 = _mm256_min_epi32(_mm256_max_epi32(
                        _mm256_loadu_si256(src.as_ptr().add(16) as *const __m256i), zero), ceil);
                    let v3 = _mm256_min_epi32(_mm256_max_epi32(
                        _mm256_loadu_si256(src.as_ptr().add(24) as *const __m256i), zero), ceil);

                    // Thu hẹp i32 → i16 (pack saturated)
                    let p01 = _mm256_packs_epi32(v0, v1); // 16 phần tử i16
                    let p23 = _mm256_packs_epi32(v2, v3); // 16 phần tử i16
                    // Thu hẹp i16 → i8 (pack saturated)
                    let packed = _mm256_packs_epi16(p01, p23); // 32 phần tử i8
                    // Hoán vị qwords khôi phục thứ tự tuyến tính
                    let result = _mm256_permutevar8x32_epi32(packed,
                        _mm256_setr_epi32(0, 4, 1, 5, 2, 6, 3, 7));

                    _mm256_storeu_si256(dst.as_mut_ptr() as *mut __m256i, result);
                }
                return;
            }
        }

        // Vòng lặp dự phòng Scalar khi không có SIMD
        let mut i = 0usize;
        while i < 32 {
            let val = src[i];
            dst[i] = if val < 0 {
                0i8
            } else if val > 127 {
                127i8
            } else {
                val as i8
            };
            i += 1;
        }
    }
    /// Nạp trọng số NNUE từ tệp nhị phân format XRNN (output của `learn::nnue::Network::quantize()`).
    /// Binary layout:
    ///   Magic "XRNN" (4B) + Version u32 LE (4B)
    ///   FT Bias:     256 × i16    =     512 bytes
    ///   FT Weights:  65536×256 × i16 = 33,554,432 bytes
    ///   Hidden:      32×512 × i8  =     16,384 bytes
    ///   Hidden Bias: 32 × i32     =       128 bytes
    ///   Output:      32 × i8      =        32 bytes
    ///   Output Bias: i32          =         4 bytes
    ///   Output Scale: i32         =         4 bytes
    pub fn load(&mut self, path: &str) -> Result<(), String> {
        use std::io::Read;

        let handle = std::fs::File::open(path)
            .map_err(|e| format!("Không thể mở tệp NNUE: {}", e))?;
        let mut file = std::io::BufReader::with_capacity(33 * 1024 * 1024, handle);

        // Đọc và xác minh magic header
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)
            .map_err(|e| format!("Lỗi đọc magic: {}", e))?;
        if &magic != b"XRNN" {
            return Err(format!("Magic header không hợp lệ: {:?} (kỳ vọng XRNN)", magic));
        }

        // Đọc version
        let mut version = [0u8; 4];
        file.read_exact(&mut version)
            .map_err(|e| format!("Lỗi đọc version: {}", e))?;
        let ver = u32::from_le_bytes(version);
        if ver != 1 {
            return Err(format!("Version không hỗ trợ: {} (kỳ vọng 1)", ver));
        }

        // Đọc Feature Transformer bias: 256 × i16
        let mut buf2 = [0u8; 2];
        for j in 0..HALF {
            file.read_exact(&mut buf2)
                .map_err(|e| format!("Lỗi đọc FT bias[{}]: {}", j, e))?;
            self.weight.bias[j] = i16::from_le_bytes(buf2);
        }

        // Đọc Feature Transformer weights: 65536 × 256 × i16 (~32MB)
        let total = super::weight::TOTAL;
        let mut matrix: Vec<[i16; HALF]> = Vec::with_capacity(total);
        for i in 0..total {
            let mut row = [0i16; HALF];
            for j in 0..HALF {
                file.read_exact(&mut buf2)
                    .map_err(|e| format!("Lỗi đọc FT weight[{}][{}]: {}", i, j, e))?;
                row[j] = i16::from_le_bytes(buf2);
            }
            matrix.push(row);
        }
        self.weight.matrix = Some(Box::new(matrix));

        // Đọc Hidden Layer weights: 32 × 512 × i8
        let mut buf1 = [0u8; 1];
        for i in 0..32 {
            for j in 0..BOTH {
                file.read_exact(&mut buf1)
                    .map_err(|e| format!("Lỗi đọc hidden[{}][{}]: {}", i, j, e))?;
                self.affine.weight[i][j] = buf1[0] as i8;
            }
        }

        // Đọc Hidden Layer bias: 32 × i32
        let mut buf4 = [0u8; 4];
        for i in 0..32 {
            file.read_exact(&mut buf4)
                .map_err(|e| format!("Lỗi đọc hidden bias[{}]: {}", i, e))?;
            self.affine.bias[i] = i32::from_le_bytes(buf4);
        }

        // Đọc Output Layer weights: 32 × i8
        for i in 0..32 {
            file.read_exact(&mut buf1)
                .map_err(|e| format!("Lỗi đọc output weight[{}]: {}", i, e))?;
            self.output.weight[i] = buf1[0] as i8;
        }

        // Đọc Output Layer bias: i32
        file.read_exact(&mut buf4)
            .map_err(|e| format!("Lỗi đọc output bias: {}", e))?;
        self.output.bias = i32::from_le_bytes(buf4);

        // Đọc Output Scale: i32
        file.read_exact(&mut buf4)
            .map_err(|e| format!("Lỗi đọc output scale: {}", e))?;
        self.output.scale = i32::from_le_bytes(buf4);

        self.loaded = true;
        Ok(())
    }
}


