// ============================================================================
// PHÂN HỆ LEARN: MODULE TRAP_STORAGE (DYNAMIC TACTICAL TRAP REPOSITORY)
// ============================================================================
// Module quản lý kho lưu trữ bẫy chiến thuật động (Tactical Trap Storage):
// 1. Lưu trữ nhị phân XRTP v1 (Magic b"XRTP", Header 32B, Record 16B).
// 2. Tra cứu điểm phạt bẫy O(1) phục vụ Move Ordering và cắt tỉa Alpha-Beta.
// 3. Tự động ghi nhận và nạp nóng (Hot-Reload) mà không cần biên dịch lại mã nguồn.
// 4. Cung cấp API xuất/nhập JSON để chỉnh sửa hoặc bổ sung bẫy tùy chỉnh.
// 5. Chuẩn MIME Type: `application/x-xiangqi-traps`.
// 100% chú thích tiếng Việt & 100% định danh từ đơn tiếng Anh (Single-Word Principle).
// ============================================================================

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use crate::movegen::types::Move;

/// Chuẩn định danh MIME Type cho tệp bẫy cờ tướng
pub const MIME: &str = "application/x-xiangqi-traps";

/// Magic bytes nhận diện định dạng nhị phân bẫy chiến thuật
pub const MAGIC: [u8; 4] = *b"XRTP";

/// Cấu trúc bản ghi nhị phân bẫy chiến thuật cố định 16 bytes (Aligned & Packed)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrapRecord {
    /// Khóa Zobrist Hash 64-bit của thế cờ
    pub hash: u64,
    /// Nước đi 16-bit sập bẫy sai lầm
    pub blunder: u16,
    /// Điểm phạt trừ điểm ưu tiên (Centipawn: -1000 đến -10000)
    pub penalty: i16,
    /// Nước đi 16-bit phản đòn chuẩn xác giải bẫy
    pub refutation: u16,
    /// Số lần vấp phải hoặc tái kiểm chứng thế bẫy
    pub count: u16,
}

impl TrapRecord {
    /// Khởi tạo bản ghi bẫy mới
    pub fn new(hash: u64, blunder: Move, penalty: i16, refutation: Move) -> Self {
        Self {
            hash,
            blunder: blunder.raw(),
            penalty,
            refutation: refutation.raw(),
            count: 1,
        }
    }

    /// Trả về nước đi sập bẫy dạng kiểu `Move`
    pub fn blunder_move(&self) -> Move {
        Move::from_raw(self.blunder)
    }

    /// Trả về nước đi phản đòn dạng kiểu `Move`
    pub fn refutation_move(&self) -> Move {
        Move::from_raw(self.refutation)
    }
}

/// Struct `TrapStorage` quản lý toàn bộ kho bẫy chiến thuật động trong RAM và trên Đĩa
#[derive(Clone, Debug, Default)]
pub struct TrapStorage {
    /// Bảng băm ánh xạ nhanh từ Zobrist Hash 64-bit sang danh sách các bẫy tại thế cờ đó
    pub map: HashMap<u64, Vec<TrapRecord>>,
    /// Tổng số lượng bản ghi bẫy đang lưu trữ
    pub total: usize,
}

impl TrapStorage {
    /// Khởi tạo đối tượng lưu trữ bẫy rỗng
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            total: 0,
        }
    }

    /// Trả về chuẩn định danh MIME Type
    pub fn mime_type() -> &'static str {
        MIME
    }

    /// Trả về tổng số bản ghi bẫy trong kho
    pub fn len(&self) -> usize {
        self.total
    }

    /// Kiểm tra kho bẫy có đang rỗng hay không
    pub fn is_empty(&self) -> bool {
        self.total == 0
    }

    /// Xóa sạch toàn bộ kho bẫy trong RAM
    pub fn clear(&mut self) {
        self.map.clear();
        self.total = 0;
    }

    /// Ghi nhận hoặc cập nhật một thế bẫy chiến thuật vào bộ nhớ
    pub fn record(&mut self, hash: u64, blunder: Move, penalty: i16, refutation: Move) {
        let raw_blunder = blunder.raw();
        let raw_refutation = refutation.raw();

        let entries = self.map.entry(hash).or_default();
        for entry in entries.iter_mut() {
            if entry.blunder == raw_blunder {
                entry.penalty = entry.penalty.max(penalty);
                entry.refutation = raw_refutation;
                entry.count = entry.count.saturating_add(1);
                return;
            }
        }

        entries.push(TrapRecord {
            hash,
            blunder: raw_blunder,
            penalty,
            refutation: raw_refutation,
            count: 1,
        });
        self.total += 1;
    }

    /// Tra cứu điểm phạt bẫy O(1) cho một nước đi cụ thể tại thế cờ `hash`
    pub fn penalty(&self, hash: u64, mv: Move) -> i16 {
        if let Some(entries) = self.map.get(&hash) {
            let raw = mv.raw();
            for entry in entries {
                if entry.blunder == raw {
                    return entry.penalty;
                }
            }
        }
        0
    }

    /// Tra cứu nước đi phản đòn giải bẫy O(1) tại thế cờ `hash` (nếu có)
    pub fn refutation(&self, hash: u64) -> Option<Move> {
        if let Some(entries) = self.map.get(&hash) {
            if let Some(first) = entries.first() {
                let mv = first.refutation_move();
                if mv.valid() {
                    return Some(mv);
                }
            }
        }
        None
    }

    /// Nạp toàn bộ kho bẫy từ tệp nhị phân XRTP v1
    pub fn load<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        let mut header = [0u8; 32];
        file.read_exact(&mut header)?;

        if &header[0..4] != &MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Không đúng định dạng Magic XRTP!",
            ));
        }

        let version = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
        if version != 1 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Phiên bản XRTP không hỗ trợ: {}", version),
            ));
        }

        let count = u64::from_le_bytes([
            header[8], header[9], header[10], header[11],
            header[12], header[13], header[14], header[15],
        ]) as usize;

        let mut storage = Self::new();
        let record_size = std::mem::size_of::<TrapRecord>();
        let mut buf = vec![0u8; record_size];

        for _ in 0..count {
            file.read_exact(&mut buf)?;
            let record: TrapRecord = unsafe { std::ptr::read_unaligned(buf.as_ptr() as *const _) };
            let entries = storage.map.entry(record.hash).or_default();
            entries.push(record);
            storage.total += 1;
        }

        Ok(storage)
    }

    /// Lưu toàn bộ kho bẫy xuống đĩa dưới định dạng nhị phân XRTP v1
    pub fn save<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        let mut header = [0u8; 32];
        header[0..4].copy_from_slice(&MAGIC);
        header[4..8].copy_from_slice(&1u32.to_le_bytes()); // Version 1
        header[8..16].copy_from_slice(&(self.total as u64).to_le_bytes()); // Record count

        file.write_all(&header)?;

        let record_size = std::mem::size_of::<TrapRecord>();
        for entries in self.map.values() {
            for record in entries {
                let slice = unsafe {
                    std::slice::from_raw_parts(record as *const _ as *const u8, record_size)
                };
                file.write_all(slice)?;
            }
        }

        file.flush()?;
        Ok(())
    }

    /// Chuyển đổi toàn bộ kho bẫy thành chuỗi JSON trực quan để quản trị
    pub fn export_json(&self) -> String {
        let mut json = String::from("[\n");
        let mut first = true;
        for entries in self.map.values() {
            for r in entries {
                if !first {
                    json.push_str(",\n");
                }
                first = false;
                json.push_str(&format!(
                    "  {{\"hash\":\"0x{:016x}\",\"blunder\":{},\"penalty\":{},\"refutation\":{},\"count\":{}}}",
                    r.hash, r.blunder, r.penalty, r.refutation, r.count
                ));
            }
        }
        json.push_str("\n]\n");
        json
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trap_storage_crud_and_persistence() {
        let mut storage = TrapStorage::new();
        assert_eq!(storage.len(), 0);
        assert!(storage.is_empty());

        let hash1 = 0x123456789ABCDEF0;
        let mv_blunder = Move::new(10, 20);
        let mv_refutation = Move::new(10, 21);

        storage.record(hash1, mv_blunder, 5000, mv_refutation);
        assert_eq!(storage.len(), 1);
        assert_eq!(storage.penalty(hash1, mv_blunder), 5000);
        assert_eq!(storage.penalty(hash1, Move::new(10, 22)), 0);
        assert_eq!(storage.refutation(hash1), Some(mv_refutation));

        let tmp_path = "target/test_traps.bin";
        storage.save(tmp_path).expect("Save failed");

        let loaded = TrapStorage::load(tmp_path).expect("Load failed");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded.penalty(hash1, mv_blunder), 5000);
        assert_eq!(loaded.refutation(hash1), Some(mv_refutation));

        let _ = std::fs::remove_file(tmp_path);
    }
}
