// ============================================================================
// MODULE CRDT: CONFLICT-FREE REPLICATED DATA TYPES KHÔNG KHÓA (LOCK-FREE CRDT)
// ============================================================================
// crdt.rs triển khai các cấu trúc dữ liệu không xảy ra xung đột khi hợp nhất đa luồng:
// 1. `PnCounter`: Bộ đếm tăng/giảm đồng thời (Positive-Negative Counter) sử dụng AtomicU64.
// 2. `LwwSet`: Tập hợp phần tử ghi đè theo độ sâu/thời gian cao nhất (Last-Write-Wins Element Set)
//    với quy tắc hợp nhất đơn điệu (Monotonic Merge) O(1) không khóa.
// 100% chú thích tiếng Việt từng dòng & 100% định danh từ đơn tiếng Anh.
// ============================================================================

use std::sync::atomic::{AtomicU64, Ordering};

/// Struct `PnCounter` (Positive-Negative Counter) quản lý đếm tăng giảm phân tán không khóa
#[repr(C, align(64))]
pub struct PnCounter {
    /// Tổng số lần tăng (Positive increments)
    pos: AtomicU64,
    /// Tổng số lần giảm (Negative decrements)
    neg: AtomicU64,
}

impl Default for PnCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl PnCounter {
    /// Khởi tạo một `PnCounter` mới với giá trị 0
    pub const fn new() -> Self {
        Self {
            pos: AtomicU64::new(0),
            neg: AtomicU64::new(0),
        }
    }

    /// Tăng bộ đếm thêm `value`
    #[inline(always)]
    pub fn add(&self, value: u64) {
        self.pos.fetch_add(value, Ordering::Relaxed);
    }

    /// Giảm bộ đếm đi `value`
    #[inline(always)]
    pub fn sub(&self, value: u64) {
        self.neg.fetch_add(value, Ordering::Relaxed);
    }

    /// Đọc giá trị thuần hiện tại
    #[inline(always)]
    pub fn get(&self) -> i64 {
        let p = self.pos.load(Ordering::Relaxed) as i64;
        let n = self.neg.load(Ordering::Relaxed) as i64;
        p - n
    }

    /// Hợp nhất với một `PnCounter` khác (Lấy max từng thành phần theo quy tắc CRDT)
    pub fn merge(&self, other: &PnCounter) {
        let op = other.pos.load(Ordering::Relaxed);
        let on = other.neg.load(Ordering::Relaxed);
        self.pos.fetch_max(op, Ordering::Relaxed);
        self.neg.fetch_max(on, Ordering::Relaxed);
    }
}

/// Bản ghi `Record` lưu trạng thái của một phần tử trong `LwwSet`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C, align(32))]
pub struct Record {
    /// Mã băm khóa thế cờ Zobrist
    pub key: u64,
    /// Nước đi tối thượng
    pub step: u16,
    /// Điểm số đánh giá Centipawn
    pub score: i16,
    /// Độ sâu tính toán
    pub depth: u8,
    /// Cờ tồn tại (1: Thêm vào, 0: Đã xóa)
    pub live: u8,
    /// Dấu thời gian logic hoặc thế hệ tính toán
    pub stamp: u64,
}

impl Record {
    /// Tạo bản ghi mới
    pub const fn new(key: u64, step: u16, score: i16, depth: u8, stamp: u64) -> Self {
        Self {
            key,
            step,
            score,
            depth,
            live: 1,
            stamp,
        }
    }

    /// Quy tắc hợp nhất CRDT Monotonic: Ưu tiên bản ghi có độ sâu cao hơn, hoặc dấu thời gian mới hơn
    #[inline(always)]
    pub fn merge(&mut self, other: &Record) {
        if other.depth > self.depth || (other.depth == self.depth && other.stamp > self.stamp) {
            *self = *other;
        }
    }
}
