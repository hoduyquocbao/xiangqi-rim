// ============================================================================
// MODULE PARTITION: PHÂN ĐẠO CỤM BẢNG BĂM SHARDING (TRANSPOSITION TABLE PARTITION)
// ============================================================================
// `partition.rs` triển khai cơ chế TT Cluster Sharding nhằm loại bỏ xung đột
// MESI Cache Line Bouncing O(N^2) khi nhiều luồng truy cập đồng thời vào Transposition Table.
// - Mỗi `Partition` là một mảng độc lập chứa các cụm `Cluster` 64-byte.
// - Cấu trúc `Partition` được căn lề bộ nhớ 64-byte (`#[repr(C, align(64))]`).
// - Định tuyến địa chỉ băm dựa trên phép băm kết hợp chỉ số luồng và khóa Zobrist.
// ============================================================================

use std::sync::atomic::Ordering;
use crate::movegen::types::Move;
use crate::tt::cluster::Cluster;
use crate::tt::item::Item;

/// Struct `Partition` đại diện cho một phân đoạn Shard độc lập trong Transposition Table (align 64-byte).
#[repr(C, align(64))]
pub struct Partition {
    /// Mảng lưu trữ danh sách các cụm Cluster độc lập trong phân đoạn
    pub items: Vec<Cluster>,
    /// Mặt nạ bitwise mask tra cứu nhanh `(count - 1)` trong phân đoạn
    pub mask: usize,
    /// Chỉ số phân đoạn Shard trong bảng băm toàn cục
    pub index: usize,
    /// Mảng đệm căn lề bộ nhớ vừa khít 64-byte
    pub pad: [u8; 24],
}

impl Partition {
    /// Khởi tạo một `Partition` mới với chỉ số `index` và số lượng cụm `count`.
    #[inline(always)]
    pub fn new(index: usize, mut count: usize) -> Self {
        if count == 0 {
            count = 1;
        }
        if !count.is_power_of_two() {
            count = count.next_power_of_two() >> 1;
            if count == 0 {
                count = 1;
            }
        }
        let mask = count - 1;
        let mut items = Vec::with_capacity(count);
        for _ in 0..count {
            items.push(Cluster::new());
        }
        Self {
            items,
            mask,
            index,
            pad: [0u8; 24],
        }
    }

    /// Tra cứu nguyên tử thế cờ `key: u64` trong phân đoạn Partition này.
    #[inline(always)]
    pub fn probe(&self, key: u64) -> Option<Item> {
        let idx = (key as usize) & self.mask;
        let cluster = unsafe { self.items.get_unchecked(idx) };

        if let Some(item) = cluster.slots[0].probe(key) {
            return Some(item);
        }
        if let Some(item) = cluster.slots[1].probe(key) {
            return Some(item);
        }
        if let Some(item) = cluster.slots[2].probe(key) {
            return Some(item);
        }
        if let Some(item) = cluster.slots[3].probe(key) {
            return Some(item);
        }
        None
    }

    /// Ghi nhận nguyên tử kết quả vào phân đoạn Partition theo chiến lược Two-Tier + Aging.
    #[inline(always)]
    pub fn save(&self, key: u64, depth: u8, bound: u8, step: Move, score: i16, age: u8) {
        let idx = (key as usize) & self.mask;
        let cluster = unsafe { self.items.get_unchecked(idx) };

        let mut empty = None;
        let mut victim = 0;
        let mut top = i32::MIN;

        let mut i = 0;
        while i < 4 {
            let slot = &cluster.slots[i];
            let data = slot.data.load(Ordering::Relaxed);
            if data == 0 {
                if empty.is_none() {
                    empty = Some(i);
                }
                i += 1;
                continue;
            }

            let xor = slot.key.load(Ordering::Acquire);
            if (xor ^ data) == key {
                let item = crate::tt::entry::Entry::unpack(key, data);
                if depth >= item.depth || bound == 1 || item.age != age {
                    let target = if step == Move::none() && item.step.valid() {
                        item.step
                    } else {
                        step
                    };
                    slot.save(key, depth, bound, target, score, age);
                }
                return;
            }

            let d = ((data >> 40) & 0xFF) as i32;
            let a = ((data >> 56) & 0xFF) as u8;
            let diff = (age.wrapping_sub(a)) as i32;
            let val = (diff * 256) - d;
            if val > top {
                top = val;
                victim = i;
            }
            i += 1;
        }

        let target = if let Some(sub) = empty {
            sub
        } else {
            victim
        };
        cluster.slots[target].save(key, depth, bound, step, score, age);
    }

    /// Xóa sạch toàn bộ dữ liệu phân đoạn về 0.
    #[inline(always)]
    pub fn clear(&self) {
        for cluster in &self.items {
            for slot in &cluster.slots {
                slot.data.store(0, Ordering::Release);
                slot.key.store(0, Ordering::Release);
            }
        }
    }

    /// Thu hoạch toàn bộ các cờ giá trị (depth >= 2) đã được GPU/CPU tính toán thành công.
    pub fn harvest(&self, out: &mut Vec<Item>) {
        for cluster in &self.items {
            for slot in &cluster.slots {
                let data = slot.data.load(Ordering::Relaxed);
                if data != 0 {
                    let xor = slot.key.load(Ordering::Acquire);
                    let target = xor ^ data;
                    let item = crate::tt::entry::Entry::unpack(target, data);
                    if item.key != 0 && item.depth >= 2 && item.step.valid() {
                        out.push(item);
                    }
                }
            }
        }
    }
}

// ----------------------------------------------------------------------------
// KHU VỰC BÀI KIỂM THỬ ĐƠN VỊ (UNIT TESTS) CHO PARTITION
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// Kiểm thử căn lề bộ nhớ 64-byte và kích thước struct `Partition`.
    #[test]
    fn alignment() {
        assert_eq!(std::mem::align_of::<Partition>(), 64);
        assert_eq!(std::mem::size_of::<Partition>(), 64);
    }

    /// Kiểm thử thao tác đọc/ghi trên 1 Partition đơn lẻ.
    #[test]
    fn execution() {
        let part = Partition::new(0, 16);
        let key = 0x1234_5678_9ABC_DEF0u64;
        let mv = Move::new(10, 20);

        part.save(key, 5, 1, mv, 100, 0);
        let probed = part.probe(key);
        assert!(probed.is_some());
        let item = probed.unwrap();
        assert_eq!(item.key, key);
        assert_eq!(item.score, 100);
        assert_eq!(item.step, mv);
    }
}
