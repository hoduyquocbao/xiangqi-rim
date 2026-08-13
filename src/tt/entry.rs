// ============================================================================
// MODULE ENTRY: PHẦN TỬ LƯU TRỮ NGUYÊN TỬ 16-BYTE LOCK-FREE (HYATT XOR ATOMIC ENTRY)
// ============================================================================
// Mỗi phần tử `Entry` chiếm đúng 16 bytes căn lề 16-byte (`#[repr(C, align(16))]`):
// - Gồm 2 trường nguyên tử `AtomicU64`: `key` và `data`.
// - Áp dụng thuật toán Hyatt XOR Signature Verification (`key_atomic = target_key ^ data_atomic`).
//   Thuật toán này đảm bảo không bao giờ bị đọc sai lệch giữa key và data (Torn Read)
//   mà KHÔNG cần dùng Mutex hay Spinlock!
// ============================================================================

use std::sync::atomic::{AtomicU64, Ordering};
use crate::movegen::types::Move;
use crate::tt::bound::Bound;
use crate::tt::item::Item;

/// Struct `Entry` là đơn vị lưu trữ nguyên tử 16-byte lock-free.
#[repr(C, align(16))]
pub struct Entry {
    /// Khóa XOR Verification (`target_key ^ data`) [AtomicU64]
    pub key: AtomicU64,
    /// Trường nén dữ liệu 64-bit chứa Move, Score, Depth, Bound, Age [AtomicU64]
    pub data: AtomicU64,
}

impl Default for Entry {
    /// Khởi tạo mặc định đối tượng `Entry`.
    fn default() -> Self {
        Self::empty()
    }
}

impl Entry {
    /// Khởi tạo một `Entry` rỗng với cả 2 trường nguyên tử đều bằng 0.
    #[inline(always)]
    pub const fn empty() -> Self {
        Self {
            key: AtomicU64::new(0),
            data: AtomicU64::new(0),
        }
    }

    /// Đóng gói các thành phần nước đi (`step`), điểm số (`score`), độ sâu (`depth`),
    /// cờ cận (`bound`), và tuổi (`age`) thành 1 số nguyên `u64` duy nhất.
    ///
    /// Sơ đồ Bitwise Packing:
    /// - Bits 0..15  (16 bits): Nước đi Move (from: 8b, to: 8b)
    /// - Bits 16..31 (16 bits): Điểm số Score (i16)
    /// - Bits 40..47 (8 bits) : Độ sâu Depth (u8)
    /// - Bits 48..55 (8 bits) : Cờ cận Bound (u8)
    /// - Bits 56..63 (8 bits) : Tuổi Age (u8)
    #[inline(always)]
    pub fn pack(step: Move, score: i16, depth: u8, bound: u8, age: u8) -> u64 {
        let m = (step.from as u64) | ((step.to as u64) << 8);
        let s = (score as u16 as u64) << 16;
        let d = (depth as u64) << 40;
        let b = (bound as u64) << 48;
        let a = (age as u64) << 56;
        m | s | d | b | a
    }

    /// Giải mã trường `data: u64` và `key: u64` thành đối tượng `Item` hoàn chỉnh.
    #[inline(always)]
    pub fn unpack(key: u64, data: u64) -> Item {
        let from = (data & 0xFF) as u8;
        let to = ((data >> 8) & 0xFF) as u8;
        let step = Move::new(from, to);
        let score = ((data >> 16) & 0xFFFF) as u16 as i16;
        let depth = ((data >> 40) & 0xFF) as u8;
        let bound = Bound::parse(((data >> 48) & 0xFF) as u8);
        let age = ((data >> 56) & 0xFF) as u8;
        Item::new(key, depth, bound, step, score, age)
    }

    /// Đọc nguyên tử (Probe) thế cờ `target: u64` từ `Entry`.
    /// Xác minh bằng Hyatt XOR Signature: `(key ^ data) == target`.
    #[inline(always)]
    pub fn probe(&self, target: u64) -> Option<Item> {
        let data = self.data.load(Ordering::Relaxed);
        if data == 0 {
            return None;
        }
        let xor = self.key.load(Ordering::Acquire);
        // Kiểm tra tính hợp lệ của chữ ký XOR Hyatt
        if (xor ^ data) == target {
            Some(Self::unpack(target, data))
        } else {
            None
        }
    }

    /// Ghi nguyên tử (Save) dữ liệu thế cờ vào `Entry` không khóa.
    #[inline(always)]
    pub fn save(&self, target: u64, depth: u8, bound: u8, step: Move, score: i16, age: u8) {
        let data = Self::pack(step, score, depth, bound, age);
        let xor = target ^ data;
        // Ghi dữ liệu data bằng Relaxed, sau đó ghi chữ ký xor key với mốc đồng bộ Ordering::Release
        self.data.store(data, Ordering::Relaxed);
        self.key.store(xor, Ordering::Release);
    }
}

