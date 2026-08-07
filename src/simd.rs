// ============================================================================
// MODULE SIMD: TĂNG TỐC TÍNH TOÁN ĐẠI SỐ TUYẾN TÍNH TRÊN VECTOR CPU INTRINSICS
// ============================================================================
// Cung cấp các phép toán nhân tích vô hướng (Dot Product) tối ưu trên tập thanh ghi
// Vector SIMD phần cứng:
// - AVX2 (`_mm256_madd_epi16`, `_mm256_cvtepi8_epi16`) trên CPU x86_64 Intel/AMD.
// - NEON (`vmlal_s16`, `vmovl_s8`, `vaddvq_s32`) trên CPU ARM64 Apple Silicon/Ampere.
// - Scalar Loop Fallback trên các hệ thống không hỗ trợ SIMD hardware acceleration.
// ============================================================================

/// Trả về khoảng cách căn lề bộ nhớ (alignment boundary bytes) của kiểu dữ liệu `T`.
#[inline(always)]
pub fn align<T>() -> usize {
    std::mem::align_of::<T>()
}

/// Phép tính Tích vô hướng (Dot Product) giữa mảng kích hoạt `input: &[i16]` và trọng số `weight: &[i8]`.
/// Dùng cho lớp Feature Transformer của mạng nơ-ron NNUE.
///
/// Tự động phát hiện tính năng vi xử lý phần cứng tại runtime:
/// - x86_64: Gọi `avx()` nếu phát hiện AVX2 support.
/// - aarch64: Gọi `neon()` trên kiến trúc ARM64.
/// - Fallback: Gọi `scalar()` nếu không có phần cứng SIMD.
#[inline(always)]
pub unsafe fn dot(input: &[i16], weight: &[i8]) -> i32 {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            return avx(input, weight);
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        return neon(input, weight);
    }

    scalar(input, weight)
}

/// Thuật toán nhân tích vô hướng Scalar thuần túy (dự phòng khi không có SIMD).
/// Phép toán: $sum = \sum_{i} (input[i] \times weight[i])$.
#[inline(always)]
pub fn scalar(input: &[i16], weight: &[i8]) -> i32 {
    let mut sum: i32 = 0;
    let mut idx = 0usize;
    while idx < input.len() {
        sum += (input[idx] as i32) * (weight[idx] as i32);
        idx += 1;
    }
    sum
}

/// Phép toán Tích vô hướng tăng tốc AVX2 trên x86_64 (Xử lý 16 phần tử 16-bit / 8-bit song song per loop iteration).
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn avx(input: &[i16], weight: &[i8]) -> i32 {
    use std::arch::x86_64::*;
    // 1. Tải thanh ghi 256-bit tích lũy rỗng = 0
    let mut acc = _mm256_setzero_si256();
    let mut idx = 0usize;

    // 2. Vòng lặp chính xử lý khối 16 phần tử song song
    while idx + 16 <= input.len() {
        // Nạp không căn lề (unaligned load) 16 phần tử i16 (32 bytes) vào thanh ghi 256-bit `__m256i`
        let vec = _mm256_loadu_si256(input.as_ptr().add(idx) as *const __m256i);
        // Nạp 16 phần tử i8 (16 bytes) vào thanh ghi 128-bit `__m128i`
        let byte = _mm_loadu_si128(weight.as_ptr().add(idx) as *const __m128i);
        // Mở rộng kiểu dữ liệu 16 phần tử i8 thành 16 phần tử i16 trong thanh ghi 256-bit
        let word = _mm256_cvtepi8_epi16(byte);
        // Nhận từng cặp 16-bit nhân với nhau và cộng dồn theo cặp thành 8 phần tử i32
        let prod = _mm256_madd_epi16(vec, word);
        // Cộng dồn kết quả i32 vào thanh ghi tích lũy `acc`
        acc = _mm256_add_epi32(acc, prod);
        idx += 16;
    }

    // 3. Thu gọn (Reduce sum) từ thanh ghi vector 256-bit về 1 giá trị i32 duy nhất
    let val128 = _mm_add_epi32(
        _mm256_extracti128_si256(acc, 0),
        _mm256_extracti128_si256(acc, 1),
    );
    let shuf = _mm_shuffle_epi32(val128, 0b10_11_00_01);
    let sums = _mm_add_epi32(val128, shuf);
    let shuf2 = _mm_shuffle_epi32(sums, 0b00_00_10_10);
    let sum = _mm_add_epi32(sums, shuf2);
    let mut rem = _mm_cvtsi128_si32(sum);

    // 4. Xử lý phần dư còn lại không đủ khối 16 phần tử bằng vòng lặp Scalar
    while idx < input.len() {
        rem += (input[idx] as i32) * (weight[idx] as i32);
        idx += 1;
    }
    rem
}

/// Phép toán Tích vô hướng tăng tốc ARM NEON trên aarch64 (xử lý khối 8 phần tử song song).
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub unsafe fn neon(input: &[i16], weight: &[i8]) -> i32 {
    use std::arch::aarch64::*;
    // Tải thanh ghi vector 128-bit tích lũy rỗng
    let mut acc = vdupq_n_s32(0);
    let mut idx = 0usize;

    // Vòng lặp xử lý khối 8 phần tử
    while idx + 8 <= input.len() {
        let vec = vld1q_s16(input.as_ptr().add(idx));
        let byte = vld1_s8(weight.as_ptr().add(idx));
        // Mở rộng i8 -> i16
        let word = vmovl_s8(byte);
        // Nhân cộng dồn i16 x i16 -> i32 cho nửa thấp
        acc = vmlal_s16(acc, vget_low_s16(vec), vget_low_s16(word));
        // Nhân cộng dồn i16 x i16 -> i32 cho nửa cao
        acc = vmlal_high_s16(acc, vec, word);
        idx += 8;
    }
    // Thu gọn tổng ngangvector (horizontal sum)
    let mut rem = vaddvq_s32(acc);
    while idx < input.len() {
        rem += (input[idx] as i32) * (weight[idx] as i32);
        idx += 1;
    }
    rem
}

/// Phép tính Tích vô hướng (Dot Product) giữa mảng 8-bit `input: &[i8]` và trọng số `weight: &[i8]`.
/// Dùng cho lớp Affine Layer và Output Layer của NNUE.
#[inline(always)]
pub unsafe fn bytes(input: &[i8], weight: &[i8]) -> i32 {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            return bytes::avx(input, weight);
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        return bytes::neon(input, weight);
    }

    bytes::scalar(input, weight)
}

/// Module con quản lý các phép toán SIMD tích vô hướng 8-bit.
pub mod bytes {
    /// Thuật toán nhân tích vô hướng 8-bit Scalar dự phòng.
    #[inline(always)]
    pub fn scalar(input: &[i8], weight: &[i8]) -> i32 {
        let mut sum: i32 = 0;
        let mut idx = 0usize;
        while idx < input.len() {
            sum += (input[idx] as i32) * (weight[idx] as i32);
            idx += 1;
        }
        sum
    }

    /// Phép toán Tích vô hướng 8-bit tăng tốc AVX2 trên x86_64 (Xử lý 32 phần tử i8 song song).
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    pub unsafe fn avx(input: &[i8], weight: &[i8]) -> i32 {
        use std::arch::x86_64::*;
        let mut acc = _mm256_setzero_si256();
        let mut idx = 0usize;

        // Vòng lặp xử lý khối 32 phần tử i8
        while idx + 32 <= input.len() {
            let vec = _mm256_loadu_si256(input.as_ptr().add(idx) as *const __m256i);
            let weight = _mm256_loadu_si256(weight.as_ptr().add(idx) as *const __m256i);

            // Mở rộng nửa thấp 16 i8 -> 16 i16 và nhân cộng dồn
            let low = _mm256_cvtepi8_epi16(_mm256_castsi256_si128(vec));
            let base = _mm256_cvtepi8_epi16(_mm256_castsi256_si128(weight));
            let left = _mm256_madd_epi16(low, base);
            acc = _mm256_add_epi32(acc, left);

            // Mở rộng nửa cao 16 i8 -> 16 i16 và nhân cộng dồn
            let high = _mm256_cvtepi8_epi16(_mm256_extracti128_si256(vec, 1));
            let top = _mm256_cvtepi8_epi16(_mm256_extracti128_si256(weight, 1));
            let right = _mm256_madd_epi16(high, top);
            acc = _mm256_add_epi32(acc, right);

            idx += 32;
        }

        // Thu gọn kết quả từ 256-bit về 32-bit scalar
        let val128 = _mm_add_epi32(
            _mm256_extracti128_si256(acc, 0),
            _mm256_extracti128_si256(acc, 1),
        );
        let shuf = _mm_shuffle_epi32(val128, 0b10_11_00_01);
        let sums = _mm_add_epi32(val128, shuf);
        let shuf2 = _mm_shuffle_epi32(sums, 0b00_00_10_10);
        let sum = _mm_add_epi32(sums, shuf2);
        let mut rem = _mm_cvtsi128_si32(sum);

        while idx < input.len() {
            rem += (input[idx] as i32) * (weight[idx] as i32);
            idx += 1;
        }
        rem
    }

    /// Phép toán Tích vô hướng 8-bit tăng tốc ARM NEON trên aarch64.
    #[cfg(target_arch = "aarch64")]
    #[target_feature(enable = "neon")]
    pub unsafe fn neon(input: &[i8], weight: &[i8]) -> i32 {
        use std::arch::aarch64::*;
        let mut acc = vdupq_n_s32(0);
        let mut idx = 0usize;
        while idx + 8 <= input.len() {
            let vec = vld1_s8(input.as_ptr().add(idx));
            let weight = vld1_s8(weight.as_ptr().add(idx));
            let wide = vmovl_s8(vec);
            let word = vmovl_s8(weight);
            acc = vmlal_s16(acc, vget_low_s16(wide), vget_low_s16(word));
            acc = vmlal_high_s16(acc, wide, word);
            idx += 8;
        }
        let mut rem = vaddvq_s32(acc);
        while idx < input.len() {
            rem += (input[idx] as i32) * (weight[idx] as i32);
            idx += 1;
        }
        rem
    }
}

