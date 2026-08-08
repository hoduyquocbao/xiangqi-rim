// ============================================================================
// MODULE TABLE: TOÀN CỤC BẢNG BĂM LƯU VẾT TRANSPOSITION TABLE MB-SIZED SHARDED
// ============================================================================
// `Table` quản lý mảng các `Partition` Shards phân đoạn được cấp phát bộ nhớ động theo MB:
// - Tra cứu siêu tốc $O(1)$ thông qua phép định tuyến băm partition hash.
// - Phép băm định tuyến: `partition_idx = ((key >> 48) as usize ^ (index & mask)) & (partition_count - 1)`.
// - Phân tách các luồng ghi vào các Partition căn lề 64-byte hoàn toàn độc lập,
//   triệt tiêu triệt để hiện tượng MESI Cache Line Bouncing $O(N^2)$ giữa các luồng.
// ============================================================================

use crate::movegen::types::Move;
use crate::tt::item::Item;
use crate::tt::partition::Partition;

/// Struct `Table` bọc danh sách các phân đoạn Partition, căn lề bộ nhớ 64-byte (`#[repr(C, align(64))]`).
#[repr(C, align(64))]
pub struct Table {
    /// Mảng chứa các phân đoạn Partition Shards bộ nhớ độc lập
    pub partitions: Vec<Partition>,
    /// Mặt nạ bitwise mask tra cứu số lượng phân đoạn `(partition_count - 1)`
    pub mask: usize,
    /// Bộ đếm tuổi thế cờ (Aging generation counter)
    pub age: u8,
    /// Tổng số lượng cụm Cluster trong toàn bộ bảng băm
    pub count: usize,
    /// Mảng đệm căn lề vừa khít 64-byte
    pub pad: [u8; 16],
}

impl Table {
    /// Số lượng phân đoạn Partition Shards mặc định cho Lazy SMP (16 partitions)
    pub const SHARDS: usize = 16;

    /// Khởi tạo một `Table` mới với dung lượng bộ nhớ `mb` Megabytes.
    #[inline(always)]
    pub fn new(mb: usize) -> Self {
        let bytes = mb * 1024 * 1024;
        let unit = std::mem::size_of::<crate::tt::cluster::Cluster>();
        let mut total_clusters = bytes / unit;
        if total_clusters == 0 {
            total_clusters = 1;
        }

        let shards = Self::SHARDS;
        let mut per_shard = (total_clusters / shards).max(1);
        if !per_shard.is_power_of_two() {
            per_shard = per_shard.next_power_of_two() >> 1;
            if per_shard == 0 {
                per_shard = 1;
            }
        }

        let mut partitions = Vec::with_capacity(shards);
        for idx in 0..shards {
            partitions.push(Partition::new(idx, per_shard));
        }

        let mask = shards - 1;
        let count = per_shard * shards;

        Self {
            partitions,
            mask,
            age: 0,
            count,
            pad: [0u8; 16],
        }
    }

    /// Định tuyến chỉ số phân đoạn Partition dựa trên Zobrist key và thread index.
    #[inline(always)]
    pub fn route(&self, key: u64, index: usize) -> usize {
        ((key >> 48) as usize ^ (index & self.mask)) & self.mask
    }

    /// Tra cứu nguyên tử thế cờ `key: u64` với chỉ số luồng `index`.
    #[inline(always)]
    pub fn probe_with(&self, key: u64, index: usize) -> Option<Item> {
        let p_idx = self.route(key, index);
        self.partitions[p_idx].probe(key)
    }

    /// Tra cứu nguyên tử thế cờ `key: u64` mặc định (với luồng 0).
    #[inline(always)]
    pub fn probe(&self, key: u64) -> Option<Item> {
        self.probe_with(key, 0)
    }

    /// Ghi nhận nguyên tử kết quả tìm kiếm với chỉ số luồng `index`.
    #[inline(always)]
    pub fn save_with(&self, key: u64, depth: u8, bound: u8, step: Move, score: i16, index: usize) {
        let p_idx = self.route(key, index);
        self.partitions[p_idx].save(key, depth, bound, step, score, self.age);
    }

    /// Ghi nhận nguyên tử kết quả tìm kiếm mặc định (với luồng 0).
    #[inline(always)]
    pub fn save(&self, key: u64, depth: u8, bound: u8, step: Move, score: i16) {
        self.save_with(key, depth, bound, step, score, 0);
    }

    /// Xóa sạch toàn bộ bảng băm về 0.
    #[inline(always)]
    pub fn clear(&self) {
        for partition in &self.partitions {
            partition.clear();
        }
    }

    /// Tăng tuổi thế cờ của bảng băm khi bắt đầu nước đi mới.
    #[inline(always)]
    pub fn advance(&mut self) {
        self.age = self.age.wrapping_add(1);
    }

    /// Trả về dung lượng bộ nhớ thực tế đã cấp phát tính theo Megabytes (MB).
    #[inline(always)]
    pub fn size(&self) -> usize {
        let total: usize = self.partitions.iter().map(|p| p.items.len()).sum();
        (total * std::mem::size_of::<crate::tt::cluster::Cluster>()) / (1024 * 1024)
    }

    /// Cấp phát động Transposition Table với dung lượng RAM `mb` Megabytes.
    /// Tự động quy đổi số MB RAM thành $2^k$ entries trong khoảng [16MB, 8192MB].
    #[inline(always)]
    pub fn allocate(mb: usize) -> Self {
        let size = mb.clamp(16, 49152); // Nâng giới hạn TT tối đa 48GB cho hệ thống 64GB+
        Self::new(size)
    }

    /// Thu hoạch toàn bộ các thế cờ có giá trị (depth >= 2) từ bảng băm Transposition Table.
    pub fn harvest(&self) -> Vec<Item> {
        let mut out = Vec::new();
        for partition in &self.partitions {
            partition.harvest(&mut out);
        }
        out
    }

    /// Xuất và nạp toàn bộ các cờ đã thu hoạch từ Transposition Table vào bộ đệm Replay Memory kinh nghiệm.
    pub fn export_to_replay(&self, replay: &mut crate::learn::replay::Replay) -> usize {
        let items = self.harvest();
        let mut added = 0;
        for item in items {
            let reward = (item.score as f32 / 1000.0).clamp(-1.0, 1.0);
            let sample = crate::learn::replay::Sample::new(item.key, item.step.raw(), reward, 0, 0);
            replay.push(sample);
            added += 1;
        }
        added
    }

    /// Tự động nạp bộ nhớ kinh nghiệm từ Replay Memory bền vững vào Transposition Table (0ms Hash Preheat).
    pub fn populate(&self, replay: &crate::learn::replay::Replay) -> usize {
        let mut count = 0;
        for i in 0..replay.count {
            let sample = replay.samples[i];
            if sample.hash != 0 && sample.mv != 0 {
                let score = (sample.reward * 1000.0) as i16;
                let from = (sample.mv >> 8) as u8;
                let to = (sample.mv & 0xFF) as u8;
                let step = crate::movegen::types::Move::new(from, to);
                if step.valid() {
                    self.save(sample.hash, 12, 1, step, score);
                    count += 1;
                }
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Kiểm thử căn lề bộ nhớ 64-byte cho Table.
    #[test]
    fn alignment() {
        assert_eq!(std::mem::align_of::<Table>(), 64);
        assert_eq!(std::mem::size_of::<Table>(), 64);
    }

    /// Kiểm thử cấp phát động `allocate` với các mốc dung lượng RAM biên và bình thường.
    #[test]
    fn allocate() {
        let mbs = [0, 1, 15, 16, 128, 255, 256];
        for mb in mbs {
            let table = Table::allocate(mb);
            let entries = table.count * 4;
            // Xác nhận tổng số lượng entries luôn là lũy thừa của 2
            assert!(entries.is_power_of_two(), "Số entries {} cho {}MB phải là 2^k", entries, mb);
            // Xác nhận mặt nạ table mask luôn là partitions.len() - 1
            assert_eq!(table.mask, table.partitions.len() - 1, "Mặt nạ table mask phải bằng partitions.len() - 1");
            // Xác nhận mặt nạ partition mask luôn hợp lệ (count - 1) với count là 2^k
            for partition in &table.partitions {
                let count = partition.items.len();
                assert!(count.is_power_of_two(), "Số cụm partition {} cho {}MB phải là 2^k", count, mb);
                assert_eq!(partition.mask, count - 1, "Mặt nạ partition mask phải bằng count - 1");
            }
        }

        // Kiểm thử clamp giới hạn nhỏ hơn 16MB (0, 1, 15 MB) và lớn hơn 16MB (256 MB)
        let small_zero = Table::allocate(0);
        assert_eq!(small_zero.size(), 16);
        let small_one = Table::allocate(1);
        assert_eq!(small_one.size(), 16);
        let small_fifteen = Table::allocate(15);
        assert_eq!(small_fifteen.size(), 16);
        let normal_size = Table::allocate(256);
        assert_eq!(normal_size.size(), 256);
    }
}

