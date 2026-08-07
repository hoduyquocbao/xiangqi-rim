// ============================================================================
// MODULE TT: BẢNG BĂM LƯU VẾT THẾ CỜ BẤT ĐỒNG BỘ KHÔNG KHÓA (TRANSPOSITION TABLE)
// ============================================================================
// Transposition Table (TT) lưu lại các thế cờ đã từng tìm kiếm để tái sử dụng kết quả,
// giúp tỉa nhánh hiệu quả gấp hàng trăm lần trong thuật toán PVS.
// - `entry`: Mỗi phần tử `Entry` chiếm 16 bytes căn lề 16-byte (`align(16)`), sử dụng
//   thao tác nguyên tử `AtomicU64` lock-free đảm bảo an toàn tuyệt đối khi đa luồng đọc/ghi.
// - `cluster`: Mỗi cụm `Cluster` chứa 4 `Entry` vừa khít 64 bytes (`align(64)`), khớp với 1 L1 Cache line.
// - `table`: Quản lý mảng các `Cluster` với kích thước là lũy thừa của 2, tra cứu bằng phép bitwise `key & mask`.
// ============================================================================

/// Module con `bound` định nghĩa cờ cận Alpha-Beta (Exact, Lower, Upper)
pub mod bound;
/// Module con `cluster` định nghĩa cụm 4 Entry trong 1 dòng bộ nhớ L1 Cache 64-byte
pub mod cluster;
/// Module con `entry` định nghĩa phần tử 16-byte AtomicU64 lock-free
pub mod entry;
/// Module con `item` định nghĩa cấu trúc kết quả tra cứu được giải mã
pub mod item;
/// Module con `partition` định nghĩa phân đoạn Shard trong Transposition Table 64-byte
pub mod partition;
/// Module con `table` quản lý bộ nhớ băm toàn cục và các hàm save/probe
pub mod table;

pub use bound::Bound;
pub use cluster::Cluster;
pub use entry::Entry;
pub use item::Item;
pub use partition::Partition;
pub use table::Table;


// ----------------------------------------------------------------------------
// KHU VỰC BÀI KIỂM THỬ ĐƠN VỊ (UNIT TESTS) CHO TRANSPOSITION TABLE
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::movegen::types::Move;

    /// Kiểm thử kích thước và căn lề bộ nhớ: `Entry` 16 bytes align 16, `Cluster` 64 bytes align 64.
    #[test]
    fn alignments() {
        assert_eq!(std::mem::align_of::<Entry>(), 16);
        assert_eq!(std::mem::size_of::<Entry>(), 16);
        assert_eq!(std::mem::align_of::<Cluster>(), 64);
        assert_eq!(std::mem::size_of::<Cluster>(), 64);
        assert_eq!(std::mem::align_of::<Table>(), 64);
    }

    /// Kiểm thử đọc/ghi nguyên tử (Atomic save/probe) khóa Zobrist, độ sâu depth, nước đi mv, điểm score.
    #[test]
    fn atomic() {
        let table = Table::new(1); // 1 MB Table
        let key = 0x1234_5678_9ABC_DEF0u64;
        let mv = Move::new(10, 20);
        let score = 150i16;
        let depth = 5u8;
        let bound = Bound::Exact.raw();

        table.save(key, depth, bound, mv, score);
        let probed = table.probe(key);
        assert!(probed.is_some());

        let item = probed.unwrap();
        assert_eq!(item.key, key);
        assert_eq!(item.depth, depth);
        assert_eq!(item.bound, Bound::Exact);
        assert_eq!(item.step, mv);
        assert_eq!(item.score, score);
    }

    /// Kiểm thử chiến lược thay thế (Replacement Strategy) khi cụm `Cluster` bị đầy 4 phần tử.
    #[test]
    fn replacement() {
        let table = Table::new(1);
        let index = 0usize;
        let mask = table.mask;

        // Lưu 4 items vào cùng 1 cluster
        for i in 0..4u64 {
            let key = (i * (mask as u64 + 1)) + (index as u64);
            let mv = Move::new(i as u8, (i + 1) as u8);
            table.save(key, i as u8 + 1, Bound::Exact.raw(), mv, 100);
        }

        // Kiểm tra probe cả 4 items thành công
        for i in 0..4u64 {
            let key = (i * (mask as u64 + 1)) + (index as u64);
            assert!(table.probe(key).is_some());
        }

        // Lưu item thứ 5 với depth cao hơn -> sẽ thay thế phần tử nạn nhân (victim) nông nhất
        let key5 = (4u64 * (mask as u64 + 1)) + (index as u64);
        table.save(key5, 10, Bound::Exact.raw(), Move::new(5, 6), 200);

        assert!(table.probe(key5).is_some());
    }
}

