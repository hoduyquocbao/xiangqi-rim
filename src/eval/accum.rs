// ============================================================================
// MODULE ACCUM: BỘ TÍCH LŨY GIA TĂNG MẠNG NƠ-RON NNUE (INCREMENTAL ACCUMULATOR)
// ============================================================================
// `accum.rs` duy trì bộ tích lũy gia tăng 256 chiều cho 2 góc nhìn (0: Đỏ, 1: Đen):
// - Căn lề 64-byte `#[repr(C, align(64))]` đạt chuẩn L1 Cache line.
// - `reset()`: Tích lũy lại từ đầu toàn bộ các quân cờ trên bàn cờ $O(N)$.
// - `apply()`: Cập nhật gia tăng khi đi nước mới $O(1)$ (chỉ cộng/trừ vector của quân di chuyển và quân bị ăn).
// - `revert()`: Hoàn tác gia tăng khi lùi nước đi $O(1)$.
// - `rebuild()`: Tái thiết lập bộ tích lũy cho phe có Tướng di chuyển (do góc nhìn thay đổi).
// ============================================================================

use super::feature::Feature;
use super::weight::{Weight, DIM};
use crate::board::Position;

/// Struct `Accum` chứa 2 mảng $256$ phần tử $i16$ cho góc nhìn Đỏ (`vals[0]`) và Đen (`vals[1]`), căn lề 64-byte.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Accum {
    /// Mảng tích lũy 2 chiều: `vals[0]` cho phe Đỏ, `vals[1]` cho phe Đen
    pub vals: [[i16; DIM]; 2],
}

impl Default for Accum {
    /// Khởi tạo mặc định đối tượng Accum.
    fn default() -> Self {
        Self::new()
    }
}

impl Accum {
    /// Khởi tạo bộ tích lũy rỗng bằng 0.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            vals: [[0; DIM]; 2],
        }
    }

    /// Helper: Tái thiết lập duy nhất 1 góc nhìn (`side`) khi Tướng của phe đó di chuyển sang ô `king` mới.
    /// Tối ưu hóa SIMD Unrolling 64-way (AVX2 trên x86_64) và 32-way (NEON trên aarch64).
    #[inline(always)]
    fn add(dst: &mut [i16; DIM], src: &[i16; DIM]) {
        // Tăng tốc ARM64 NEON: Duỗi vòng lặp SIMD 32-way (4 thanh ghi int16x8_t per step)
        #[cfg(target_arch = "aarch64")]
        {
            use std::arch::aarch64::*;
            let mut i = 0;
            while i < DIM {
                unsafe {
                    // Nạp song song 4 khối 8 phần tử i16 (32 phần tử = 64 bytes)
                    let dst_ptr = dst.as_mut_ptr().add(i);
                    let src_ptr = src.as_ptr().add(i);

                    let d0 = vld1q_s16(dst_ptr);
                    let d1 = vld1q_s16(dst_ptr.add(8));
                    let d2 = vld1q_s16(dst_ptr.add(16));
                    let d3 = vld1q_s16(dst_ptr.add(24));

                    let s0 = vld1q_s16(src_ptr);
                    let s1 = vld1q_s16(src_ptr.add(8));
                    let s2 = vld1q_s16(src_ptr.add(16));
                    let s3 = vld1q_s16(src_ptr.add(24));

                    // Cộng dồn vector SIMD cho 4 thanh ghi
                    vst1q_s16(dst_ptr, vaddq_s16(d0, s0));
                    vst1q_s16(dst_ptr.add(8), vaddq_s16(d1, s1));
                    vst1q_s16(dst_ptr.add(16), vaddq_s16(d2, s2));
                    vst1q_s16(dst_ptr.add(24), vaddq_s16(d3, s3));
                }
                i += 32;
            }
            return;
        }

        // Tăng tốc x86_64 AVX2: Duỗi vòng lặp SIMD 64-way với lệnh nạp/ghi căn lề aligned _mm256_load_si256
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                use std::arch::x86_64::*;
                let mut i = 0;
                while i < DIM {
                    unsafe {
                        // Con trỏ căn lề 64-byte (aligned pointers) nạp 4 thanh ghi 256-bit (64 phần tử = 128 bytes)
                        let dst_ptr = dst.as_mut_ptr().add(i) as *mut __m256i;
                        let src_ptr = src.as_ptr().add(i) as *const __m256i;

                        let d0 = _mm256_load_si256(dst_ptr.add(0));
                        let d1 = _mm256_load_si256(dst_ptr.add(1));
                        let d2 = _mm256_load_si256(dst_ptr.add(2));
                        let d3 = _mm256_load_si256(dst_ptr.add(3));

                        let s0 = _mm256_loadu_si256(src_ptr.add(0));
                        let s1 = _mm256_loadu_si256(src_ptr.add(1));
                        let s2 = _mm256_loadu_si256(src_ptr.add(2));
                        let s3 = _mm256_loadu_si256(src_ptr.add(3));

                        // Thực thi cộng dồn 64-way song song và lưu lại vào bộ nhớ căn lề
                        _mm256_store_si256(dst_ptr.add(0), _mm256_add_epi16(d0, s0));
                        _mm256_store_si256(dst_ptr.add(1), _mm256_add_epi16(d1, s1));
                        _mm256_store_si256(dst_ptr.add(2), _mm256_add_epi16(d2, s2));
                        _mm256_store_si256(dst_ptr.add(3), _mm256_add_epi16(d3, s3));
                    }
                    i += 64;
                }
                return;
            }
        }

        // Vòng lặp dự phòng Scalar khi không có SIMD phần cứng
        let mut d = 0;
        while d < DIM {
            dst[d] += src[d];
            d += 1;
        }
    }

    /// Cập nhật cộng/trừ 2 vector đặc trưng cho bộ tích lũy Accumulator.
    #[inline(always)]
    fn update(dst: &mut [i16; DIM], add: &[i16; DIM], sub: &[i16; DIM]) {
        // Tăng tốc ARM64 NEON: Duỗi vòng lặp SIMD 32-way
        #[cfg(target_arch = "aarch64")]
        {
            use std::arch::aarch64::*;
            let mut i = 0;
            while i < DIM {
                unsafe {
                    let dst_ptr = dst.as_mut_ptr().add(i);
                    let add_ptr = add.as_ptr().add(i);
                    let sub_ptr = sub.as_ptr().add(i);

                    let d0 = vld1q_s16(dst_ptr);
                    let d1 = vld1q_s16(dst_ptr.add(8));
                    let d2 = vld1q_s16(dst_ptr.add(16));
                    let d3 = vld1q_s16(dst_ptr.add(24));

                    let a0 = vld1q_s16(add_ptr);
                    let a1 = vld1q_s16(add_ptr.add(8));
                    let a2 = vld1q_s16(add_ptr.add(16));
                    let a3 = vld1q_s16(add_ptr.add(24));

                    let s0 = vld1q_s16(sub_ptr);
                    let s1 = vld1q_s16(sub_ptr.add(8));
                    let s2 = vld1q_s16(sub_ptr.add(16));
                    let s3 = vld1q_s16(sub_ptr.add(24));

                    let r0 = vsubq_s16(vaddq_s16(d0, a0), s0);
                    let r1 = vsubq_s16(vaddq_s16(d1, a1), s1);
                    let r2 = vsubq_s16(vaddq_s16(d2, a2), s2);
                    let r3 = vsubq_s16(vaddq_s16(d3, a3), s3);

                    vst1q_s16(dst_ptr, r0);
                    vst1q_s16(dst_ptr.add(8), r1);
                    vst1q_s16(dst_ptr.add(16), r2);
                    vst1q_s16(dst_ptr.add(24), r3);
                }
                i += 32;
            }
            return;
        }

        // Tăng tốc x86_64 AVX2: Duỗi vòng lặp SIMD 64-way với aligned loads/stores
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                use std::arch::x86_64::*;
                let mut i = 0;
                while i < DIM {
                    unsafe {
                        let dst_ptr = dst.as_mut_ptr().add(i) as *mut __m256i;
                        let add_ptr = add.as_ptr().add(i) as *const __m256i;
                        let sub_ptr = sub.as_ptr().add(i) as *const __m256i;

                        let d0 = _mm256_load_si256(dst_ptr.add(0));
                        let d1 = _mm256_load_si256(dst_ptr.add(1));
                        let d2 = _mm256_load_si256(dst_ptr.add(2));
                        let d3 = _mm256_load_si256(dst_ptr.add(3));

                        let a0 = _mm256_loadu_si256(add_ptr.add(0));
                        let a1 = _mm256_loadu_si256(add_ptr.add(1));
                        let a2 = _mm256_loadu_si256(add_ptr.add(2));
                        let a3 = _mm256_loadu_si256(add_ptr.add(3));

                        let s0 = _mm256_loadu_si256(sub_ptr.add(0));
                        let s1 = _mm256_loadu_si256(sub_ptr.add(1));
                        let s2 = _mm256_loadu_si256(sub_ptr.add(2));
                        let s3 = _mm256_loadu_si256(sub_ptr.add(3));

                        let r0 = _mm256_sub_epi16(_mm256_add_epi16(d0, a0), s0);
                        let r1 = _mm256_sub_epi16(_mm256_add_epi16(d1, a1), s1);
                        let r2 = _mm256_sub_epi16(_mm256_add_epi16(d2, a2), s2);
                        let r3 = _mm256_sub_epi16(_mm256_add_epi16(d3, a3), s3);

                        _mm256_store_si256(dst_ptr.add(0), r0);
                        _mm256_store_si256(dst_ptr.add(1), r1);
                        _mm256_store_si256(dst_ptr.add(2), r2);
                        _mm256_store_si256(dst_ptr.add(3), r3);
                    }
                    i += 64;
                }
                return;
            }
        }

        // Vòng lặp dự phòng Scalar
        let mut d = 0;
        while d < DIM {
            dst[d] += add[d] - sub[d];
            d += 1;
        }
    }

    /// Điều chỉnh gia tăng bộ tích lũy khi có ăn quân (cộng vector mới, trừ vector di chuyển cũ và trừ vector quân bị ăn).
    #[inline(always)]
    fn modify(dst: &mut [i16; DIM], add: &[i16; DIM], sub1: &[i16; DIM], sub2: &[i16; DIM]) {
        // Tăng tốc ARM64 NEON: Duỗi vòng lặp SIMD 32-way
        #[cfg(target_arch = "aarch64")]
        {
            use std::arch::aarch64::*;
            let mut i = 0;
            while i < DIM {
                unsafe {
                    let dst_ptr = dst.as_mut_ptr().add(i);
                    let add_ptr = add.as_ptr().add(i);
                    let sub1_ptr = sub1.as_ptr().add(i);
                    let sub2_ptr = sub2.as_ptr().add(i);

                    let d0 = vld1q_s16(dst_ptr);
                    let d1 = vld1q_s16(dst_ptr.add(8));
                    let d2 = vld1q_s16(dst_ptr.add(16));
                    let d3 = vld1q_s16(dst_ptr.add(24));

                    let a0 = vld1q_s16(add_ptr);
                    let a1 = vld1q_s16(add_ptr.add(8));
                    let a2 = vld1q_s16(add_ptr.add(16));
                    let a3 = vld1q_s16(add_ptr.add(24));

                    let f0 = vld1q_s16(sub1_ptr);
                    let f1 = vld1q_s16(sub1_ptr.add(8));
                    let f2 = vld1q_s16(sub1_ptr.add(16));
                    let f3 = vld1q_s16(sub1_ptr.add(24));

                    let g0 = vld1q_s16(sub2_ptr);
                    let g1 = vld1q_s16(sub2_ptr.add(8));
                    let g2 = vld1q_s16(sub2_ptr.add(16));
                    let g3 = vld1q_s16(sub2_ptr.add(24));

                    let r0 = vsubq_s16(vsubq_s16(vaddq_s16(d0, a0), f0), g0);
                    let r1 = vsubq_s16(vsubq_s16(vaddq_s16(d1, a1), f1), g1);
                    let r2 = vsubq_s16(vsubq_s16(vaddq_s16(d2, a2), f2), g2);
                    let r3 = vsubq_s16(vsubq_s16(vaddq_s16(d3, a3), f3), g3);

                    vst1q_s16(dst_ptr, r0);
                    vst1q_s16(dst_ptr.add(8), r1);
                    vst1q_s16(dst_ptr.add(16), r2);
                    vst1q_s16(dst_ptr.add(24), r3);
                }
                i += 32;
            }
            return;
        }

        // Tăng tốc x86_64 AVX2: Duỗi vòng lặp SIMD 64-way với aligned loads/stores
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                use std::arch::x86_64::*;
                let mut i = 0;
                while i < DIM {
                    unsafe {
                        let dst_ptr = dst.as_mut_ptr().add(i) as *mut __m256i;
                        let add_ptr = add.as_ptr().add(i) as *const __m256i;
                        let sub1_ptr = sub1.as_ptr().add(i) as *const __m256i;
                        let sub2_ptr = sub2.as_ptr().add(i) as *const __m256i;

                        let d0 = _mm256_load_si256(dst_ptr.add(0));
                        let d1 = _mm256_load_si256(dst_ptr.add(1));
                        let d2 = _mm256_load_si256(dst_ptr.add(2));
                        let d3 = _mm256_load_si256(dst_ptr.add(3));

                        let a0 = _mm256_loadu_si256(add_ptr.add(0));
                        let a1 = _mm256_loadu_si256(add_ptr.add(1));
                        let a2 = _mm256_loadu_si256(add_ptr.add(2));
                        let a3 = _mm256_loadu_si256(add_ptr.add(3));

                        let f0 = _mm256_loadu_si256(sub1_ptr.add(0));
                        let f1 = _mm256_loadu_si256(sub1_ptr.add(1));
                        let f2 = _mm256_loadu_si256(sub1_ptr.add(2));
                        let f3 = _mm256_loadu_si256(sub1_ptr.add(3));

                        let g0 = _mm256_loadu_si256(sub2_ptr.add(0));
                        let g1 = _mm256_loadu_si256(sub2_ptr.add(1));
                        let g2 = _mm256_loadu_si256(sub2_ptr.add(2));
                        let g3 = _mm256_loadu_si256(sub2_ptr.add(3));

                        let r0 = _mm256_sub_epi16(_mm256_sub_epi16(_mm256_add_epi16(d0, a0), f0), g0);
                        let r1 = _mm256_sub_epi16(_mm256_sub_epi16(_mm256_add_epi16(d1, a1), f1), g1);
                        let r2 = _mm256_sub_epi16(_mm256_sub_epi16(_mm256_add_epi16(d2, a2), f2), g2);
                        let r3 = _mm256_sub_epi16(_mm256_sub_epi16(_mm256_add_epi16(d3, a3), f3), g3);

                        _mm256_store_si256(dst_ptr.add(0), r0);
                        _mm256_store_si256(dst_ptr.add(1), r1);
                        _mm256_store_si256(dst_ptr.add(2), r2);
                        _mm256_store_si256(dst_ptr.add(3), r3);
                    }
                    i += 64;
                }
                return;
            }
        }

        // Vòng lặp dự phòng Scalar
        let mut d = 0;
        while d < DIM {
            dst[d] += add[d] - sub1[d] - sub2[d];
            d += 1;
        }
    }

    /// Helper: Tái thiết lập duy nhất 1 góc nhìn (`side`) khi Tướng của phe đó di chuyển sang ô `king` mới.
    #[inline(always)]
    fn rebuild(&mut self, pos: &Position, side: usize, king: u8, from: u8, to: u8, weight: &Weight) {
        self.vals[side] = weight.bias;

        let mut sq = 0u8;
        while sq < 90 {
            let piece = if sq == from {
                14
            } else if sq == to {
                (side * 7) as u8
            } else {
                pos.grid[sq as usize]
            };

            if piece < 14 {
                let idx = Feature::index(king, piece, sq, side as u8, side as u8);
                let feat = weight.feature(idx);
                Self::add(&mut self.vals[side], feat);
            }
            sq += 1;
        }
    }

    /// Tích lũy lại từ đầu toàn bộ các quân cờ trên bàn cờ cho cả 2 phe Đỏ và Đen.
    #[inline(always)]
    pub fn reset(&mut self, pos: &Position, weight: &Weight) {
        self.vals[0] = weight.bias;
        self.vals[1] = weight.bias;

        let k0 = pos.king[0];
        let k1 = pos.king[1];

        let mut sq = 0u8;
        while sq < 90 {
            let p = pos.grid[sq as usize];
            if p < 14 {
                let idx0 = Feature::index(k0, p, sq, 0, 0);
                let feat0 = weight.feature(idx0);
                Self::add(&mut self.vals[0], feat0);

                let idx1 = Feature::index(k1, p, sq, 1, 1);
                let feat1 = weight.feature(idx1);
                Self::add(&mut self.vals[1], feat1);
            }
            sq += 1;
        }
    }

    /// Cập nhật gia tăng bộ tích lũy khi di chuyển nước đi từ `from` sang `to`.
    #[inline(always)]
    pub fn apply(
        &mut self,
        pos: &Position,
        from: u8,
        to: u8,
        moving: u8,
        captured: u8,
        weight: &Weight,
    ) {
        let king = (moving % 7) == 0;
        if king {
            let side = if moving < 7 { 0usize } else { 1usize };
            let other = side ^ 1;

            self.rebuild(pos, side, to, from, to, weight);

            let enemy = pos.king[other];
            let rem = Feature::index(enemy, moving, from, other as u8, other as u8);
            let add = Feature::index(enemy, moving, to, other as u8, other as u8);
            let old = weight.feature(rem);
            let new = weight.feature(add);

            if captured < 14 {
                let capidx = Feature::index(enemy, captured, to, other as u8, other as u8);
                let cap = weight.feature(capidx);
                Self::modify(&mut self.vals[other], new, old, cap);
            } else {
                Self::update(&mut self.vals[other], new, old);
            }
            return;
        }

        let k0 = pos.king[0];
        let k1 = pos.king[1];

        let rem0 = Feature::index(k0, moving, from, 0, 0);
        let add0 = Feature::index(k0, moving, to, 0, 0);
        let old0 = weight.feature(rem0);
        let new0 = weight.feature(add0);

        if captured < 14 {
            let cap0 = Feature::index(k0, captured, to, 0, 0);
            let c0 = weight.feature(cap0);
            Self::modify(&mut self.vals[0], new0, old0, c0);
        } else {
            Self::update(&mut self.vals[0], new0, old0);
        }

        let rem1 = Feature::index(k1, moving, from, 1, 1);
        let add1 = Feature::index(k1, moving, to, 1, 1);
        let old1 = weight.feature(rem1);
        let new1 = weight.feature(add1);

        if captured < 14 {
            let cap1 = Feature::index(k1, captured, to, 1, 1);
            let c1 = weight.feature(cap1);
            Self::modify(&mut self.vals[1], new1, old1, c1);
        } else {
            Self::update(&mut self.vals[1], new1, old1);
        }
    }

    /// Hoàn tác gia tăng bộ tích lũy (Revert move) quay trở lại trạng thái cũ.
    #[inline(always)]
    pub fn revert(
        &mut self,
        pos: &Position,
        from: u8,
        to: u8,
        moving: u8,
        captured: u8,
        weight: &Weight,
    ) {
        let king = (moving % 7) == 0;
        if king {
            let side = if moving < 7 { 0usize } else { 1usize };
            let other = side ^ 1;

            self.rebuild(pos, side, pos.king[side], from, from, weight);

            let enemy = pos.king[other];
            let rem = Feature::index(enemy, moving, from, other as u8, other as u8);
            let add = Feature::index(enemy, moving, to, other as u8, other as u8);
            let old = weight.feature(rem);
            let new = weight.feature(add);

            if captured < 14 {
                let capidx = Feature::index(enemy, captured, to, other as u8, other as u8);
                let cap = weight.feature(capidx);
                Self::update(&mut self.vals[other], old, new);
                Self::add(&mut self.vals[other], cap);
            } else {
                Self::update(&mut self.vals[other], old, new);
            }
            return;
        }

        let k0 = pos.king[0];
        let k1 = pos.king[1];

        let rem0 = Feature::index(k0, moving, from, 0, 0);
        let add0 = Feature::index(k0, moving, to, 0, 0);
        let old0 = weight.feature(rem0);
        let new0 = weight.feature(add0);

        if captured < 14 {
            let cap0 = Feature::index(k0, captured, to, 0, 0);
            let c0 = weight.feature(cap0);
            Self::update(&mut self.vals[0], old0, new0);
            Self::add(&mut self.vals[0], c0);
        } else {
            Self::update(&mut self.vals[0], old0, new0);
        }

        let rem1 = Feature::index(k1, moving, from, 1, 1);
        let add1 = Feature::index(k1, moving, to, 1, 1);
        let old1 = weight.feature(rem1);
        let new1 = weight.feature(add1);

        if captured < 14 {
            let cap1 = Feature::index(k1, captured, to, 1, 1);
            let c1 = weight.feature(cap1);
            Self::update(&mut self.vals[1], old1, new1);
            Self::add(&mut self.vals[1], c1);
        } else {
            Self::update(&mut self.vals[1], old1, new1);
        }
    }
}


