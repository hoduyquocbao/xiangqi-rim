// ============================================================================
// MODULE LEARN STORE: LƯU TRỮ TRÍ NHỚ KINH NGHIỆM LÂU DÀI TRÊN Ổ ĐĨA (PERSISTENCE)
// ============================================================================
// Module `store` định nghĩa định dạng tệp nhị phân siêu nhẹ (Magic `XRLN`, Header 64B,
// Record 32B) lưu giữ các mẫu kinh nghiệm Replay xuống đĩa cứng và nạp lại khi khởi chạy.
// Thiết kế 100% Clean Room std-only, căn lề bộ nhớ 64-byte loại bỏ False Sharing,
// triệt tiêu 100% Memory UB padding, và tuân thủ Quy tắc Định danh Đơn Từ Tiếng Anh.
// ============================================================================

use std::fs::File;
use std::io::{Read, Write};
use crate::book::endgame::Endgame;
use crate::book::opening::Book;
use crate::learn::replay::{Replay, Sample};

/// Magic signature xác thực định dạng nhị phân XiangRust Learn Memory (`b"XRLN"`)
pub const MAGIC: [u8; 4] = *b"XRLN";
/// Phiên bản định dạng nhị phân hiện tại (Version 1)
pub const VERSION: u32 = 1;

/// Struct `Header` chứa thông số siêu dữ liệu tệp lưu trữ (64 bytes, `#[repr(C, align(64))]`).
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Header {
    /// Chuỗi chữ ký nhận dạng định dạng `b"XRLN"`
    pub magic: [u8; 4],
    /// Phiên bản tệp lưu giữ
    pub version: u32,
    /// Số lượng bản ghi kinh nghiệm có trong tệp
    pub count: u64,
    /// Mảng đệm căn lề 48-byte cho đủ 64 bytes vật lý
    pub pad: [u8; 48],
}

impl Header {
    /// Khởi tạo header mới với số lượng bản ghi `count`.
    #[inline(always)]
    pub fn new(count: u64) -> Self {
        Self {
            magic: MAGIC,
            version: VERSION,
            count,
            pad: [0u8; 48],
        }
    }

    /// Khởi tạo header rỗng mặc định.
    #[inline(always)]
    pub fn empty() -> Self {
        Self {
            magic: [0u8; 4],
            version: 0,
            count: 0,
            pad: [0u8; 48],
        }
    }
}

impl Default for Header {
    fn default() -> Self {
        Self::empty()
    }
}

/// Struct `Record` đại diện cho 1 phần tử bản ghi nhị phân lưu ổ đĩa (32 bytes, `#[repr(C, align(16))]`).
/// Thứ tự các trường được sắp xếp theo căn lề giảm dần để triệt tiêu 100% uninitialized padding UB.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Record {
    /// Mã băm Zobrist của thế cờ s (8 bytes, align 8)
    pub hash: u64,
    /// Mã băm Zobrist của thế cờ s' (8 bytes, align 8)
    pub next: u64,
    /// Phần thưởng r (4 bytes, align 4)
    pub reward: f32,
    /// Mã nước đi a (2 bytes, align 2)
    pub mv: u16,
    /// Cờ kết thúc (1 byte, align 1)
    pub done: u8,
    /// Đệm 9-byte cho đủ 32 bytes vật lý (align 1)
    pub pad: [u8; 9],
}

impl Record {
    /// Chuyển đổi từ `Sample` sang `Record` lưu đĩa.
    #[inline(always)]
    pub fn from(sample: &Sample) -> Self {
        Self {
            hash: sample.hash,
            next: sample.next,
            reward: sample.reward,
            mv: sample.mv,
            done: sample.done,
            pad: [0u8; 9],
        }
    }

    /// Chuyển đổi từ `Record` lưu đĩa sang `Sample`.
    #[inline(always)]
    pub fn sample(&self) -> Sample {
        Sample::new(self.hash, self.mv, self.reward, self.next, self.done)
    }
}

/// Struct `Store` thực thi thao tác lưu và nạp đĩa nhị phân persistence (align 64).
#[repr(C, align(64))]
pub struct Store {
    /// Tiêu đề header tệp
    pub header: Header,
    /// Số bản ghi đã xử lý
    pub count: usize,
    /// Mảng đệm căn lề 48-byte
    pub pad: [u8; 48],
}

impl Store {
    /// Khởi tạo đối tượng `Store` mới.
    pub fn new() -> Self {
        Self {
            header: Header::empty(),
            count: 0,
            pad: [0u8; 48],
        }
    }

    /// Lưu toàn bộ mẫu kinh nghiệm từ `replay` xuống tệp nhị phân tại đường dẫn `path`.
    pub fn save(replay: &Replay, path: &str) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        let header = Header::new(replay.len() as u64);

        // 1. Ghi Header 64 bytes
        let hdr = unsafe {
            std::slice::from_raw_parts(&header as *const Header as *const u8, std::mem::size_of::<Header>())
        };
        file.write_all(hdr)?;

        // 2. Ghi từng Record 32 bytes
        for i in 0..replay.len() {
            if let Some(sample) = replay.get(i) {
                let rec = Record::from(sample);
                let raw = unsafe {
                    std::slice::from_raw_parts(&rec as *const Record as *const u8, std::mem::size_of::<Record>())
                };
                file.write_all(raw)?;
            }
        }

        file.flush()?;
        Ok(())
    }

    /// Nạp mẫu kinh nghiệm từ tệp nhị phân `path` vào bộ đệm `replay`. Trả về số mẫu nạp thành công.
    pub fn load(replay: &mut Replay, path: &str) -> std::io::Result<usize> {
        let mut file = File::open(path)?;
        let mut header = Header::empty();

        // 1. Đọc Header 64 bytes
        let hdr = unsafe {
            std::slice::from_raw_parts_mut(&mut header as *mut Header as *mut u8, std::mem::size_of::<Header>())
        };
        file.read_exact(hdr)?;

        if header.magic != MAGIC || header.version != VERSION {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Lỗi chữ ký hoặc phiên bản tệp nhị phân không hợp lệ!",
            ));
        }

        replay.clear();
        let mut loaded = 0usize;
        let mut rec = Record {
            hash: 0,
            next: 0,
            reward: 0.0,
            mv: 0,
            done: 0,
            pad: [0u8; 9],
        };

        let raw = unsafe {
            std::slice::from_raw_parts_mut(&mut rec as *mut Record as *mut u8, std::mem::size_of::<Record>())
        };

        // 2. Đọc từng Record 32 bytes
        while loaded < (header.count as usize) {
            if file.read_exact(raw).is_err() {
                break;
            }
            replay.push(rec.sample());
            loaded += 1;
        }

        Ok(loaded)
    }

    /// Đồng bộ tự động các nước đi có tỷ lệ thắng cao (win rate >= 65%, reward >= 0.65)
    /// từ bộ đệm `replay` trực tiếp vào Opening Book và Endgame Memory Table.
    /// Trả về số lượng mẫu kinh nghiệm xuất sắc đã được đồng bộ thành công.
    pub fn sync(replay: &Replay) -> usize {
        let mut count = 0usize;
        let total = replay.len();
        let mut i = 0usize;
        while i < total {
            if let Some(sample) = replay.get(i) {
                // Ngưỡng tỷ lệ thắng cao: win rate >= 65% (reward >= 0.65)
                if sample.reward >= 0.65 {
                    let weight = (sample.reward.min(1.0) * 1000.0) as u16;
                    let book = Book::sync(sample.hash, sample.mv, weight);

                    let score = if sample.reward >= 0.9 {
                        crate::book::endgame::WIN
                    } else {
                        (sample.reward * 15000.0) as i32
                    };
                    let endgame = Endgame::sync(sample.hash, score);

                    if book || endgame {
                        count += 1;
                    }
                }
            }
            i += 1;
        }
        count
    }

    /// Hàm bọc hỗ trợ gọi đồng bộ kinh nghiệm khai cuộc cho tương thích bộ kiểm thử cũ.
    #[deprecated(note = "Sử dụng Store::sync để tuân thủ quy tắc định danh đơn từ")]
    #[inline(always)]
    pub fn sync_book(replay: &Replay) -> usize {
        Self::sync(replay)
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

// ----------------------------------------------------------------------------
// KHU VỰC BÀI KIỂM THỬ ĐƠN VỊ (UNIT TESTS) CHO PERSISTENT STORE
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// Kiểm thử căn lề bộ nhớ và dung lượng struct Header & Record
    #[test]
    fn alignments() {
        assert_eq!(std::mem::align_of::<Header>(), 64);
        assert_eq!(std::mem::size_of::<Header>(), 64);
        assert_eq!(std::mem::align_of::<Record>(), 16);
        assert_eq!(std::mem::size_of::<Record>(), 32);
        assert_eq!(std::mem::align_of::<Store>(), 64);
    }

    /// Kiểm thử lưu và nạp tệp nhị phân kinh nghiệm
    #[test]
    fn storage() {
        let path = "/tmp/test_experience_m2_remediation.bin";
        let mut replay = Replay::capacity(10);
        replay.push(Sample::new(111, 10, 0.5, 222, 0));
        replay.push(Sample::new(333, 20, 1.0, 444, 1));

        // Lưu xuống ổ đĩa
        let res = Store::save(&replay, path);
        assert!(res.is_ok());

        // Nạp lại từ ổ đĩa
        let mut target = Replay::capacity(10);
        let loaded = Store::load(&mut target, path);
        assert!(loaded.is_ok());
        assert_eq!(loaded.unwrap(), 2);
        assert_eq!(target.len(), 2);
        assert_eq!(target.get(0).unwrap().hash, 111);
        assert_eq!(target.get(1).unwrap().reward, 1.0);

        // Dọn dẹp tệp tạm
        let _ = std::fs::remove_file(path);
    }

    /// Kiểm thử tính năng đồng bộ tự động sync với các mẫu kinh nghiệm có tỷ lệ thắng cao (reward >= 0.65).
    #[test]
    fn sync() {
        Book::clear();
        Endgame::clear();

        let mut replay = Replay::capacity(10);
        // Sample có win-rate 0.5 (dưới 65%) -> Không sync
        replay.push(Sample::new(100, 0x1316, 0.5, 200, 0));
        // Sample có win-rate 0.8 (trên 65%) -> Đồng bộ thành công
        replay.push(Sample::new(300, 0x0114, 0.8, 400, 0));

        let synced = Store::sync(&replay);
        assert_eq!(synced, 1);
        assert_eq!(Book::count(), 1);
        assert_eq!(Endgame::count(), 1);

        // Kiểm tra tra cứu sau khi đồng bộ
        let book = Book::default();
        let probed_mv = book.find(300);
        assert!(probed_mv.is_some());
        assert_eq!(probed_mv.unwrap().raw(), 0x0114);

        let probed_score = Endgame::probe(300);
        assert!(probed_score.is_some());

        Book::clear();
        Endgame::clear();
    }
}
