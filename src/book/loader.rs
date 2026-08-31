// ============================================================================
// MODULE LOADER: TRÌNH NẠP VÀ XUẤT BẢN SÁCH KHAI CUỘC ĐỘNG CHUẨN XRBK v1
// ============================================================================
// `loader.rs` chịu trách nhiệm giao tiếp I/O nạp sách khai cuộc từ bên ngoài:
// 1. Chuẩn nhị phân XRBK v1 (Xiangqi Rim Book v1): Zero-allocation, siêu gọn 16B/record.
// 2. Trình đọc tệp JSONL FEN/Moves từ các ván cờ Grandmaster chưng cất.
// 3. Nạp trực tiếp vào bảng bộ nhớ đệm động `DYNAMIC` mà không cần biên dịch lại mã nguồn.
//
// 100% chú thích tiếng Việt từng dòng & 100% định danh từ đơn tiếng Anh.
// ============================================================================

use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use crate::board::Parser;
use crate::book::opening::{Book, Entry};
use crate::uci::Format;

/// Cấu trúc phần đầu tệp nhị phân sách khai cuộc `XRBK v1`.
/// Căn lề bộ nhớ 16-byte (`#[repr(C, align(16))]`), kích thước đúng 16 bytes.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    /// Ký tự nhận diện ma thuật (Magic bytes: `b"XRBK"`)
    pub magic: [u8; 4],
    /// Số phiên bản định dạng nhị phân (Mặc định = 1)
    pub version: u32,
    /// Tổng số lượng bản ghi nước đi khai cuộc trong tệp
    pub count: u32,
    /// Cờ tính năng mở rộng bổ trợ (Flags)
    pub flags: u32,
}

impl Header {
    /// Ký tự ma thuật chuẩn hóa của định dạng sách khai cuộc Xiangqi Rim
    pub const MAGIC: [u8; 4] = *b"XRBK";
    /// Phiên bản định dạng hiện hành
    pub const VERSION: u32 = 1;

    /// Khởi tạo một đối tượng Header mới với số lượng bản ghi `count`.
    #[inline(always)]
    pub const fn new(count: u32) -> Self {
        Self {
            magic: Self::MAGIC,
            version: Self::VERSION,
            count,
            flags: 0,
        }
    }

    /// Thẩm định tính hợp lệ của Header.
    #[inline(always)]
    pub fn valid(&self) -> bool {
        self.magic == Self::MAGIC && self.version == Self::VERSION
    }
}

/// Bản ghi nhị phân thô của một nước đi khai cuộc trên đĩa (`Record`).
/// Kích thước vật lý đúng 16 bytes, tương thích bộ nhớ đệm CPU Cache.
#[repr(C, align(8))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Record {
    /// Khóa băm Zobrist Hash 64-bit của vị trí thế cờ
    pub hash: u64,
    /// Nước đi mã hóa 16-bit (`(from << 8) | to`)
    pub mv: u16,
    /// Trọng số ưu tiên / tần suất xuất hiện (0 - 65535)
    pub weight: u16,
    /// Điểm đánh giá thế trận centipawn hoặc tỷ lệ thắng
    pub score: i16,
    /// Đệm căn lề 2 bytes cho đủ 16 bytes
    pub pad: [u8; 2],
}

impl Record {
    /// Chuyển đổi một bản ghi `Record` trên đĩa thành `Entry` trong bộ nhớ.
    #[inline(always)]
    pub fn entry(&self) -> Entry {
        Entry::new(self.hash, self.mv, self.weight, "Dynamic Book")
    }

    /// Chuyển đổi một `Entry` trong bộ nhớ thành `Record` để lưu xuống đĩa.
    #[inline(always)]
    pub fn from_entry(entry: &Entry) -> Self {
        Self {
            hash: entry.hash,
            mv: entry.mv,
            weight: entry.weight,
            score: 0,
            pad: [0; 2],
        }
    }
}

/// Struct `Loader` cung cấp các hàm tĩnh nạp và chuyển đổi sách khai cuộc.
pub struct Loader;

impl Loader {
    /// Nạp danh sách các bản ghi khai cuộc từ tệp nhị phân chuẩn `XRBK v1`.
    pub fn load_xrbk(path: &str) -> std::io::Result<Vec<Entry>> {
        let mut file = File::open(path)?;
        let mut header_buf = [0u8; 16];
        file.read_exact(&mut header_buf)?;

        let magic = [header_buf[0], header_buf[1], header_buf[2], header_buf[3]];
        let version = u32::from_le_bytes([header_buf[4], header_buf[5], header_buf[6], header_buf[7]]);
        let count = u32::from_le_bytes([header_buf[8], header_buf[9], header_buf[10], header_buf[11]]) as usize;
        let flags = u32::from_le_bytes([header_buf[12], header_buf[13], header_buf[14], header_buf[15]]);

        let header = Header { magic, version, count: count as u32, flags };
        if !header.valid() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Tệp không đúng định dạng nhị phân XRBK v1 chuẩn!",
            ));
        }

        let mut entries = Vec::with_capacity(count);
        let mut record_buf = [0u8; 16];

        for _ in 0..count {
            file.read_exact(&mut record_buf)?;
            let hash = u64::from_le_bytes([
                record_buf[0], record_buf[1], record_buf[2], record_buf[3],
                record_buf[4], record_buf[5], record_buf[6], record_buf[7],
            ]);
            let mv = u16::from_le_bytes([record_buf[8], record_buf[9]]);
            let weight = u16::from_le_bytes([record_buf[10], record_buf[11]]);

            entries.push(Entry::new(hash, mv, weight, "Dynamic Book"));
        }

        Ok(entries)
    }

    /// Xuất bản mảng các bản ghi khai cuộc xuống tệp đĩa nhị phân chuẩn `XRBK v1`.
    pub fn save_xrbk(path: &str, entries: &[Entry]) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        let header = Header::new(entries.len() as u32);

        // 1. Ghi Header 16 bytes
        let mut header_buf = [0u8; 16];
        header_buf[0..4].copy_from_slice(&header.magic);
        header_buf[4..8].copy_from_slice(&header.version.to_le_bytes());
        header_buf[8..12].copy_from_slice(&header.count.to_le_bytes());
        header_buf[12..16].copy_from_slice(&header.flags.to_le_bytes());
        file.write_all(&header_buf)?;

        // 2. Ghi từng Record 16 bytes
        let mut record_buf = [0u8; 16];
        for entry in entries {
            record_buf[0..8].copy_from_slice(&entry.hash.to_le_bytes());
            record_buf[8..10].copy_from_slice(&entry.mv.to_le_bytes());
            record_buf[10..12].copy_from_slice(&entry.weight.to_le_bytes());
            record_buf[12..14].copy_from_slice(&0i16.to_le_bytes());
            record_buf[14..16].copy_from_slice(&[0u8; 2]);
            file.write_all(&record_buf)?;
        }

        file.flush()?;
        Ok(())
    }

    /// Nạp sách khai cuộc từ tệp JSONL chứa các thế cờ FEN và best_move.
    pub fn load_jsonl(path: &str) -> std::io::Result<Vec<Entry>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line_res in reader.lines() {
            let line = line_res?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Phân tích cú pháp JSON đơn giản tìm trường "fen" và "best_move"
            if let (Some(fen_start), Some(move_start)) = (trimmed.find("\"fen\":\""), trimmed.find("\"best_move\":\"")) {
                let fen_val_start = fen_start + 7;
                if let Some(fen_val_end) = trimmed[fen_val_start..].find('"') {
                    let fen_str = &trimmed[fen_val_start..fen_val_start + fen_val_end];
                    let move_val_start = move_start + 13;
                    if let Some(move_val_end) = trimmed[move_val_start..].find('"') {
                        let move_str = &trimmed[move_val_start..move_val_start + move_val_end];
                        
                        let pos = Parser::parse(fen_str);
                        let mv = Format::decode(move_str);
                        if mv.valid() {
                            let raw_mv = ((mv.from as u16) << 8) | (mv.to as u16);
                            entries.push(Entry::new(pos.hash, raw_mv, 500, "Grandmaster JSONL"));
                        }
                    }
                }
            }
        }

        Ok(entries)
    }

    /// Nạp và đồng bộ trực tiếp tệp sách khai cuộc (XRBK hoặc JSONL) vào Engine Runtime mà không cần restart.
    pub fn import(path: &str) -> std::io::Result<usize> {
        let entries = if path.ends_with(".jsonl") {
            Self::load_jsonl(path)?
        } else {
            Self::load_xrbk(path)?
        };

        let count = entries.len();
        Book::load_batch(entries);
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alignments() {
        assert_eq!(std::mem::align_of::<Header>(), 16);
        assert_eq!(std::mem::size_of::<Header>(), 16);
        assert_eq!(std::mem::align_of::<Record>(), 8);
        assert_eq!(std::mem::size_of::<Record>(), 16);
    }

    #[test]
    fn roundtrip() {
        let entries = vec![
            Entry::new(0x123456789ABCDEF0, 0x1316, 990, "Test 1"),
            Entry::new(0x0FEDCBA987654321, 0x5845, 950, "Test 2"),
        ];

        let path = "target/test_book.xrbk";
        Loader::save_xrbk(path, &entries).expect("Lưu XRBK thất bại!");
        let loaded = Loader::load_xrbk(path).expect("Nạp XRBK thất bại!");

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].hash, entries[0].hash);
        assert_eq!(loaded[0].mv, entries[0].mv);
        assert_eq!(loaded[0].weight, entries[0].weight);

        let _ = std::fs::remove_file(path);
    }
}
