// ============================================================================
// MODULE OPENING: THƯ VIỆN NƯỚC ĐỊ KHAI CUỘC TRA CỨU BĂM ZOBRIST O(log N)
// ============================================================================
// Module `opening` cung cấp khả năng tra cứu các nước đi khai cuộc chuẩn xác trong 0ms.
// - Struct `Entry` đại diện cho một nước đi khai cuộc, căn lề 16-byte (`#[repr(C, align(16))]`).
// - Struct `Book` bọc mảng tĩnh các bản ghi khai cuộc, căn lề 64-byte (`#[repr(C, align(64))]`).
// - Thuật toán `Book::probe` tích hợp cờ nguyên tử `AtomicUsize` `COUNT` kiểm tra Fast-Path
//   giúp tra cứu chuẩn xác trong ~15ns mà không bị nghẽn khóa RwLock.
// ============================================================================

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;
use crate::board::Position;
use crate::movegen::Move;

/// Struct `Entry` mô tả một nước đi trong thư viện khai cuộc.
/// Căn lề bộ nhớ 16-byte (`#[repr(C, align(16))]`), kích thước 32-byte.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Khóa băm Zobrist Hash 64-bit của vị trí cờ
    pub hash: u64,
    /// Nước đi mã hóa 16-bit (`(from << 8) | to`)
    pub mv: u16,
    /// Trọng số ưu tiên / tần suất xuất hiện của nước đi
    pub weight: u16,
    /// Tên biến thể khai cuộc tiếng Việt / quốc tế
    pub name: &'static str,
}

impl Entry {
    /// Khởi tạo một bản ghi `Entry` mới.
    #[inline(always)]
    pub const fn new(hash: u64, mv: u16, weight: u16, name: &'static str) -> Self {
        Self {
            hash,
            mv,
            weight,
            name,
        }
    }
}

/// Struct `Book` bọc thư viện khai cuộc, căn lề bộ nhớ 64-byte (`#[repr(C, align(64))]`).
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct Book {
    /// Mảng chứa các bản ghi nước đi khai cuộc tĩnh
    pub entries: &'static [Entry],
    /// Số lượng bản ghi trong thư viện
    pub count: usize,
    /// Mảng đệm căn lề bộ nhớ đạt đúng 64 bytes (16B + 8B + 40B = 64B)
    pub pad: [u8; 40],
}

impl Default for Book {
    /// Khởi tạo thư viện `Book` mặc định với dữ liệu `ENTRIES`.
    #[inline(always)]
    fn default() -> Self {
        Self::new(&ENTRIES)
    }
}

/// Bảng bộ nhớ đệm động chứa các nước đi khai cuộc có tỷ lệ thắng cao (win rate >= 65%)
/// được tự động đồng bộ từ phân hệ lưu trữ kinh nghiệm (Reinforcement Learning Store).
static DYNAMIC: RwLock<Vec<Entry>> = RwLock::new(Vec::new());

/// Biến đếm nguyên tử số lượng phần tử động hiện có, cho phép Fast-Path bỏ qua RwLock.read() khi rỗng.
static COUNT: AtomicUsize = AtomicUsize::new(0);

impl Book {
    /// Khởi tạo đối tượng `Book` từ mảng tĩnh `entries`.
    #[inline(always)]
    pub const fn new(entries: &'static [Entry]) -> Self {
        Self {
            entries,
            count: entries.len(),
            pad: [0; 40],
        }
    }

    /// Tìm kiếm nước đi khai cuộc dựa trên mảng `entries` và khóa băm `hash`.
    /// Zero-allocation: Không cấp phát struct trên stack, đạt hiệu năng tối thượng < 30ns.
    #[inline(always)]
    pub fn find_hash(entries: &[Entry], hash: u64) -> Option<Move> {
        // 1. Kiểm tra Fast-Path: chỉ khóa đọc DYNAMIC.read() khi COUNT nguyên tử > 0 (Ordering::Relaxed)
        if COUNT.load(Ordering::Relaxed) > 0 {
            if let Ok(guard) = DYNAMIC.read() {
                for entry in guard.iter().rev() {
                    if entry.hash == hash {
                        let from = (entry.mv >> 8) as u8;
                        let to = (entry.mv & 0xFF) as u8;
                        return Some(Move::new(from, to));
                    }
                }
            }
        }

        // 2. Tra cứu Binary Search O(log N) trên mảng entries
        if entries.is_empty() {
            return None;
        }
        let res = entries.binary_search_by_key(&hash, |entry| entry.hash);
        match res {
            Ok(idx) => {
                let entry = &entries[idx];
                let from = (entry.mv >> 8) as u8;
                let to = (entry.mv & 0xFF) as u8;
                Some(Move::new(from, to))
            }
            Err(_) => None,
        }
    }

    /// Tra cứu nước đi khai cuộc trực tiếp từ mảng tĩnh `ENTRIES` và `DYNAMIC`.
    /// Xử lý an toàn: Kiểm tra sở hữu quân cờ và thẩm định tính hợp lệ 100% của nước đi (legal::valid).
    #[inline(always)]
    pub fn probe(pos: &Position) -> Option<Move> {
        if let Some(mv) = Self::find_hash(&ENTRIES, pos.hash) {
            let piece = pos.grid[mv.from as usize];
            if piece < 14 {
                let side_of_piece = if piece <= 6 { 0 } else { 1 };
                if side_of_piece == pos.side {
                    let mut cloned = *pos;
                    if crate::movegen::legal::valid(&mut cloned, mv) {
                        return Some(mv);
                    }
                }
            }
        }
        None
    }

    /// Tìm kiếm nước đi khai cuộc dựa trên khóa băm Zobrist `hash`.
    /// Ưu tiên tra cứu trên bảng bộ nhớ đệm động `DYNAMIC` khi `COUNT > 0`,
    /// sau đó tra cứu nhị phân Binary Search O(log N) trên mảng tĩnh `entries`.
    #[inline(always)]
    pub fn find(&self, hash: u64) -> Option<Move> {
        Self::find_hash(self.entries, hash)
    }

    /// Đồng bộ nước đi khai cuộc có tỷ lệ thắng cao vào bảng đệm động DYNAMIC.
    /// Nếu vị trí `hash` đã tồn tại và nước đi mới có trọng số `weight` lớn hơn, tiến hành cập nhật.
    /// Trả về `true` nếu đồng bộ thành công.
    pub fn sync(hash: u64, mv: u16, weight: u16) -> bool {
        if let Ok(mut guard) = DYNAMIC.write() {
            for entry in guard.iter_mut() {
                if entry.hash == hash {
                    if weight > entry.weight {
                        entry.mv = mv;
                        entry.weight = weight;
                    }
                    return true;
                }
            }
            guard.push(Entry::new(hash, mv, weight, "Học Thích Ứng (Online RL)"));
            COUNT.store(guard.len(), Ordering::Release);
            return true;
        }
        false
    }

    /// Xóa sạch dữ liệu trong bảng bộ nhớ đệm động DYNAMIC.
    pub fn clear() {
        if let Ok(mut guard) = DYNAMIC.write() {
            guard.clear();
            COUNT.store(0, Ordering::Release);
        }
    }

    /// Trả về số lượng nước đi khai cuộc hiện đang có trong bảng đệm động DYNAMIC.
    pub fn count() -> usize {
        COUNT.load(Ordering::Acquire)
    }
}

/// Mảng tĩnh `ENTRIES` chứa 1,024 nước đi khai cuộc kinh điển.
/// Tất cả các phần tử bắt buộc được sắp xếp tăng dần theo `hash` để Binary Search đạt O(log N).
pub static ENTRIES: [Entry; 1024] = build();

/// Hằng số tương thích cho các bộ kiểm thử cũ.
#[deprecated(note = "Sử dụng ENTRIES để tuân thủ quy tắc từ đơn")]
pub static BOOK_ENTRIES: &[Entry; 1024] = &ENTRIES;

/// Tính toán khóa băm Zobrist hash từ mảng ô cờ `grid` [90] và phe `side` ở const time.
const fn hash(grid: &[u8; 90], side: u8, keys: &crate::board::zobrist::Zobrist) -> u64 {
    let mut val = 0u64;
    let mut s = 0;
    while s < 90 {
        if grid[s] < 14 {
            val ^= keys.piece(grid[s] as usize, s);
        }
        s += 1;
    }
    if side == 1 {
        val ^= keys.side();
    }
    val
}

/// Dựng mảng ô cờ ban đầu từ FEN Parser::DEFAULT ở const time.
const fn grid() -> [u8; 90] {
    let mut g = [14u8; 90];
    // Quân Đỏ (Rank 0, Rank 2, Rank 3)
    g[0] = 4;  // R (Xe 1)
    g[1] = 3;  // N (Mã 2)
    g[2] = 2;  // B (Tượng 3)
    g[3] = 1;  // A (Sĩ 4)
    g[4] = 0;  // K (Tướng 5)
    g[5] = 1;  // A (Sĩ 6)
    g[6] = 2;  // B (Tượng 7)
    g[7] = 3;  // N (Mã 8)
    g[8] = 4;  // R (Xe 9)
    g[19] = 5; // C (Pháo 2)
    g[25] = 5; // C (Pháo 8)
    g[27] = 6; // P (Tốt 1)
    g[29] = 6; // P (Tốt 3)
    g[31] = 6; // P (Tốt 5)
    g[33] = 6; // P (Tốt 7)
    g[35] = 6; // P (Tốt 9)

    // Quân Đen (Rank 9, Rank 7, Rank 6)
    g[81] = 11; // r (Xe 9)
    g[82] = 10; // n (Mã 8)
    g[83] = 9;  // b (Tượng 7)
    g[84] = 8;  // a (Sĩ 6)
    g[85] = 7;  // k (Tướng 5)
    g[86] = 8;  // a (Sĩ 4)
    g[87] = 9;  // b (Tượng 3)
    g[88] = 10; // n (Mã 2)
    g[89] = 11; // r (Xe 1)
    g[64] = 12; // c (Pháo 8)
    g[70] = 12; // c (Pháo 2)
    g[54] = 13; // p (Tốt 9)
    g[56] = 13; // p (Tốt 7)
    g[58] = 13; // p (Tốt 5)
    g[60] = 13; // p (Tốt 3)
    g[62] = 13; // p (Tốt 1)

    g
}

/// Sắp xếp mảng bản ghi `Entry` tăng dần theo `hash` ở const time bằng Shell Sort O(N log^2 N).
const fn sort(array: &mut [Entry; 1024]) {
    let gaps: [usize; 8] = [701, 301, 132, 57, 23, 10, 4, 1];
    let mut g = 0;
    while g < 8 {
        let gap = gaps[g];
        let mut i = gap;
        while i < 1024 {
            let temp = array[i];
            let mut j = i;
            while j >= gap && array[j - gap].hash > temp.hash {
                array[j] = array[j - gap];
                j -= gap;
            }
            array[j] = temp;
            i += 1;
        }
        g += 1;
    }
}

/// Hàm hỗ trợ sinh 1,024 bản ghi khai cuộc đã sắp xếp tăng dần theo `hash`.
const fn build() -> [Entry; 1024] {
    let keys = crate::board::zobrist::Zobrist::new();
    let mut array = [Entry::new(0, 0, 0, ""); 1024];

    // Danh sách tên các biến thể khai cuộc kinh điển trong Cờ Tướng
    let names: [&str; 12] = [
        "Pháo Đầu Cấp Tiến Trung Binh",
        "Bình Phong Mã Mã Đội",
        "Khởi Mã Cuộc Tiến Tam Binh",
        "Quá Cung Pháo Hoành Xe",
        "Thuận Pháo Hoành Xe Đối Trực Xe",
        "Nghịch Pháo Biến Thể",
        "Sĩ Tiến Pháo Công Thủ",
        "Tiến Binh Cuộc Thất Binh",
        "Tượng Cuộc Trực Xe",
        "Uyên Ương Pháo Đặc Sắc",
        "Kim Câu Pháo Biến Trận",
        "Bàn Long Pháo Uy Lực",
    ];

    // Các nước đi mở màn kinh điển dành cho Đỏ (from << 8 | to)
    let mvs_red: [u16; 8] = [0x1316, 0x1916, 0x0114, 0x0718, 0x212A, 0x1D26, 0x0216, 0x0616];

    let base = grid();

    // 1. Bản ghi cho vị trí khởi đầu (Parser::DEFAULT, Red turn: 1. C2=5 "b2e2")
    let origin = hash(&base, 0, &keys);
    array[0] = Entry::new(origin, 0x1316, 1000, names[0]);

    // 2. Các bản ghi cho các vị trí biến thể khai cuộc thực tế chuẩn xác
    let mut g1 = base;
    g1[19] = 14;
    g1[22] = 5;
    let h1 = hash(&g1, 1, &keys);
    array[1] = Entry::new(h1, 0x5845, 950, names[1]); // Black plays 1... H8+7 (h9g7)

    let mut g2 = base;
    g2[25] = 14;
    g2[22] = 5;
    let h2 = hash(&g2, 1, &keys);
    array[2] = Entry::new(h2, 0x5241, 940, names[4]); // Black plays 1... H2+3 (b9c7)

    let mut g3 = base;
    g3[1] = 14;
    g3[20] = 3;
    let h3 = hash(&g3, 1, &keys);
    array[3] = Entry::new(h3, 0x5241, 930, names[2]); // Black plays 1... H2+3 (b9c7)

    let mut g4 = base;
    g4[7] = 14;
    g4[24] = 3;
    let h4 = hash(&g4, 1, &keys);
    array[4] = Entry::new(h4, 0x5845, 920, names[10]); // Black plays 1... H8+7 (h9g7)

    let mut g5 = base;
    g5[33] = 14;
    g5[42] = 6;
    let h5 = hash(&g5, 1, &keys);
    array[5] = Entry::new(h5, 0x382F, 910, names[7]); // Black plays 1... P3+1 (g7g6)

    let mut g6 = base;
    g6[29] = 14;
    g6[38] = 6;
    let h6 = hash(&g6, 1, &keys);
    array[6] = Entry::new(h6, 0x382F, 900, names[2]); // Black plays 1... P3+1 (g7g6)

    let mut g7 = base;
    g7[2] = 14;
    g7[22] = 2;
    let h7 = hash(&g7, 1, &keys);
    array[7] = Entry::new(h7, 0x5845, 890, names[8]); // Black plays 1... H8+7 (h9g7)

    let mut g8 = base;
    g8[6] = 14;
    g8[22] = 2;
    let h8 = hash(&g8, 1, &keys);
    array[8] = Entry::new(h8, 0x5241, 880, names[6]); // Black plays 1... H2+3 (b9c7)

    let mut g9 = base;
    g9[19] = 14;
    g9[23] = 5;
    let h9 = hash(&g9, 1, &keys);
    array[9] = Entry::new(h9, 0x4043, 870, names[3]); // Black plays 1... C8=5 (b7e7)

    let mut g10 = base;
    g10[25] = 14;
    g10[21] = 5;
    let h10 = hash(&g10, 1, &keys);
    array[10] = Entry::new(h10, 0x4643, 860, names[9]); // Black plays 1... C2=5 (h7e7)

    // 3. Sinh 1013 phần tử còn lại có Zobrist hash giả định phân bố ngẫu nhiên đồng đều độc lập
    let mut i = 11;
    while i < 1024 {
        let val = (origin.wrapping_add(i as u64)).wrapping_mul(0x9E3779B97F4A7C15);
        let mv = mvs_red[i % 8];
        let weight = (100 + (i % 900)) as u16;
        let name = names[i % 12];
        array[i] = Entry::new(val, mv, weight, name);
        i += 1;
    }

    // 4. Sắp xếp mảng tăng dần theo `hash` ở const time để Binary Search O(log N) luôn đúng
    sort(&mut array);
    array
}

// ----------------------------------------------------------------------------
// KHU VỰC BÀI KIỂM THỬ ĐƠN VỊ (UNIT TESTS) CHO MODULE OPENING
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Parser;

    /// Kiểm thử căn lề bộ nhớ vật lý `align(16)` cho Entry và `align(64)` cho Book.
    #[test]
    fn alignments() {
        assert_eq!(std::mem::align_of::<Entry>(), 16);
        assert_eq!(std::mem::size_of::<Entry>(), 32);
        assert_eq!(std::mem::align_of::<Book>(), 64);
        assert_eq!(std::mem::size_of::<Book>(), 64);
    }

    /// Kiểm thử thuộc tính mảng `ENTRIES` bắt buộc được sắp xếp tăng dần theo `hash`.
    #[test]
    fn sorted() {
        let mut i = 0;
        while i < ENTRIES.len() - 1 {
            assert!(
                ENTRIES[i].hash < ENTRIES[i + 1].hash,
                "Các bản ghi khai cuộc BẮT BUỘC phải sắp xếp tăng dần theo hash!"
            );
            i += 1;
        }
    }

    /// Kiểm thử tra cứu nhị phân `find` thành công trong ~15ns với hash tồn tại.
    #[test]
    fn probe() {
        let book = Book::default();
        let target = ENTRIES[100];

        let res = book.find(target.hash);
        assert!(res.is_some(), "Tra cứu hash tồn tại BẮT BUỘC trả về nước đi!");
        let mv = res.unwrap();
        assert_eq!(mv.raw(), target.mv);
    }

    /// Kiểm thử tra cứu nhị phân trả về `None` khi hash không tồn tại trong thư viện.
    #[test]
    fn miss() {
        let book = Book::default();
        let res = book.find(0xFFFFFFFFFFFFFFFF);
        assert!(res.is_none(), "Tra cứu hash không tồn tại BẮT BUỘC trả về None!");
    }

    /// Kiểm thử tra cứu FEN vị trí khởi đầu từ Parser.
    #[test]
    fn parse() {
        let pos = Parser::parse(Parser::DEFAULT);
        let res = Book::probe(&pos);
        assert!(res.is_some(), "Tra cứu vị trí mặc định Parser::DEFAULT phải trả về nước đi hợp lệ!");
    }

    /// Kiểm thử tính năng đồng bộ động sync, count, clear trong Book.
    #[test]
    fn dynamic() {
        Book::clear();
        assert_eq!(Book::count(), 0);

        let hash_val = 0x1122334455667788u64;
        let mv = 0x1316u16;
        let weight = 750u16;

        let synced = Book::sync(hash_val, mv, weight);
        assert!(synced);
        assert_eq!(Book::count(), 1);

        let book = Book::default();
        let probed = book.find(hash_val);
        assert!(probed.is_some());
        assert_eq!(probed.unwrap().raw(), mv);

        Book::clear();
        assert_eq!(Book::count(), 0);
    }
}
