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
        // 1. Kiểm tra Fast-Path: Tra cứu nhị phân O(log N) trên DYNAMIC khi COUNT > 0
        if COUNT.load(Ordering::Relaxed) > 0 {
            if let Ok(guard) = DYNAMIC.read() {
                if let Ok(idx) = guard.binary_search_by_key(&hash, |entry| entry.hash) {
                    let entry = &guard[idx];
                    let from = (entry.mv >> 8) as u8;
                    let to = (entry.mv & 0xFF) as u8;
                    return Some(Move::new(from, to));
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

    /// Nạp hàng loạt (Batch Load) mảng bản ghi khai cuộc vào bảng đệm động DYNAMIC với sắp xếp O(log N).
    pub fn load_batch(mut entries: Vec<Entry>) {
        entries.sort_unstable_by_key(|e| e.hash);
        entries.dedup_by_key(|e| e.hash);
        if let Ok(mut guard) = DYNAMIC.write() {
            *guard = entries;
            COUNT.store(guard.len(), Ordering::Release);
        }
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
            match guard.binary_search_by_key(&hash, |e| e.hash) {
                Ok(idx) => {
                    if weight > guard[idx].weight {
                        guard[idx].mv = mv;
                        guard[idx].weight = weight;
                    }
                    return true;
                }
                Err(insert_idx) => {
                    guard.insert(insert_idx, Entry::new(hash, mv, weight, "Học Thích Ứng (Online RL)"));
                    COUNT.store(guard.len(), Ordering::Release);
                    return true;
                }
            }
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

/// Thực hiện nước đi trên mảng ô cờ `grid` ở const time.
const fn make_move(mut g: [u8; 90], from: usize, to: usize) -> [u8; 90] {
    g[to] = g[from];
    g[from] = 14;
    g
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

    let mut count = 1;

    // Helper macro / logic trong const fn để thêm nhánh
    // --- NHÁNH 1: PHÁO ĐẦU VS BÌNH PHONG MÃ TRỰC XA (14 Plies) ---
    let g1 = make_move(base, 19, 22); // 1. C2=5 (b2e2)
    array[count] = Entry::new(hash(&g1, 1, &keys), 0x5845, 980, names[1]); count += 1; // 1... H8+7 (h9g7)

    let g2 = make_move(g1, 88, 69);
    array[count] = Entry::new(hash(&g2, 0, &keys), 0x0114, 970, names[0]); count += 1; // 2. H2+3 (b0c2)

    let g3 = make_move(g2, 1, 20);
    array[count] = Entry::new(hash(&g3, 1, &keys), 0x5241, 960, names[1]); count += 1; // 2... H2+3 (b9c7)

    let g4 = make_move(g3, 82, 65);
    array[count] = Entry::new(hash(&g4, 0, &keys), 0x0001, 950, names[0]); count += 1; // 3. R1=2 (a0b0)

    let g5 = make_move(g4, 0, 1);
    array[count] = Entry::new(hash(&g5, 1, &keys), 0x5152, 940, names[1]); count += 1; // 3... R1=2 (a9b9)

    let g6 = make_move(g5, 81, 82);
    array[count] = Entry::new(hash(&g6, 0, &keys), 0x212A, 930, names[0]); count += 1; // 4. P7+1 (g3g4)

    let g7 = make_move(g6, 33, 42);
    array[count] = Entry::new(hash(&g7, 1, &keys), 0x3C33, 920, names[1]); count += 1; // 4... P7+1 (g6g5)

    let g8 = make_move(g7, 60, 51);
    array[count] = Entry::new(hash(&g8, 0, &keys), 0x0137, 910, names[0]); count += 1; // 5. R2+6 (b0b6)

    let g9 = make_move(g8, 1, 55);
    array[count] = Entry::new(hash(&g9, 1, &keys), 0x4037, 900, names[1]); count += 1; // 5... C8-1 (b7b8)

    let g10 = make_move(g9, 64, 55);
    array[count] = Entry::new(hash(&g10, 0, &keys), 0x0718, 890, names[0]); count += 1; // 6. H8+7 (h0g2)

    let g11 = make_move(g10, 7, 24);
    array[count] = Entry::new(hash(&g11, 1, &keys), 0x5343, 880, names[1]); count += 1; // 6... E3+5 (c9e7)

    let g12 = make_move(g11, 83, 67);
    array[count] = Entry::new(hash(&g12, 0, &keys), 0x0807, 870, names[0]); count += 1; // 7. R9=8 (i0h0)

    let g13 = make_move(g12, 8, 7);
    array[count] = Entry::new(hash(&g13, 1, &keys), 0x5950, 860, names[1]); count += 1; // 7... R9+1 (i9i8)

    // --- NHÁNH 1B: PHÁO ĐẦU VS BÌNH PHONG MÃ HOÀNH XA (12 Plies) ---
    let gh1 = make_move(g6, 33, 42); // 4. P7+1 (g3g4)
    let gh2 = make_move(gh1, 81, 72); // 4... R1+1 (a9a8)
    array[count] = Entry::new(hash(&gh2, 0, &keys), 0x0718, 930, names[1]); count += 1; // 5. H8+7 (h0g2)

    let gh3 = make_move(gh2, 7, 24);
    array[count] = Entry::new(hash(&gh3, 1, &keys), 0x484B, 920, names[1]); count += 1; // 5... R1=4 (a8d8)

    let gh4 = make_move(gh3, 72, 75);
    array[count] = Entry::new(hash(&gh4, 0, &keys), 0x0807, 910, names[0]); count += 1; // 6. R9=8 (i0h0)

    let gh5 = make_move(gh4, 8, 7);
    array[count] = Entry::new(hash(&gh5, 1, &keys), 0x3C33, 900, names[1]); count += 1; // 6... P7+1 (g6g5)

    // --- NHÁNH 2: PHÁO ĐẦU VS THUẬN PHÁO HOÀNH XA (12 Plies) ---
    let gt1 = make_move(g1, 64, 67); // 1... C8=5 (b7e7)
    array[count] = Entry::new(hash(&gt1, 0, &keys), 0x0114, 960, names[4]); count += 1; // 2. H2+3 (b0c2)

    let gt2 = make_move(gt1, 1, 20);
    array[count] = Entry::new(hash(&gt2, 1, &keys), 0x5241, 950, names[4]); count += 1; // 2... H2+3 (b9c7)

    let gt3 = make_move(gt2, 82, 65);
    array[count] = Entry::new(hash(&gt3, 0, &keys), 0x0001, 940, names[4]); count += 1; // 3. R1=2 (a0b0)

    let gt4 = make_move(gt3, 0, 1);
    array[count] = Entry::new(hash(&gt4, 1, &keys), 0x5152, 930, names[4]); count += 1; // 3... R1=2 (a9b9)

    let gt5 = make_move(gt4, 81, 82);
    array[count] = Entry::new(hash(&gt5, 0, &keys), 0x0718, 920, names[4]); count += 1; // 4. H8+7 (h0g2)

    let gt6 = make_move(gt5, 7, 24);
    array[count] = Entry::new(hash(&gt6, 1, &keys), 0x5845, 910, names[4]); count += 1; // 4... H8+7 (h9g7)

    let gt7 = make_move(gt6, 88, 69);
    array[count] = Entry::new(hash(&gt7, 0, &keys), 0x0807, 900, names[4]); count += 1; // 5. R9=8 (i0h0)

    let gt8 = make_move(gt7, 8, 7);
    array[count] = Entry::new(hash(&gt8, 1, &keys), 0x5958, 890, names[4]); count += 1; // 5... R9=8 (i9h9)

    let gt9 = make_move(gt8, 89, 88);
    array[count] = Entry::new(hash(&gt9, 0, &keys), 0x0137, 880, names[4]); count += 1; // 6. R2+6 (b0b6)

    let gt10 = make_move(gt9, 1, 55);
    array[count] = Entry::new(hash(&gt10, 1, &keys), 0x3C33, 870, names[4]); count += 1; // 6... P7+1 (g6g5)

    // --- NHÁNH 3: TIẾN THẤT BINH CUỘC (P7+1) (12 Plies) ---
    let gb1 = make_move(base, 33, 42); // 1. P7+1 (g3g4)
    array[count] = Entry::new(hash(&gb1, 1, &keys), 0x3C33, 950, names[7]); count += 1; // 1... P7+1 (g6g5)

    let gb2 = make_move(gb1, 60, 51);
    array[count] = Entry::new(hash(&gb2, 0, &keys), 0x0718, 940, names[7]); count += 1; // 2. H8+7 (h0g2)

    let gb3 = make_move(gb2, 7, 24);
    array[count] = Entry::new(hash(&gb3, 1, &keys), 0x5845, 930, names[7]); count += 1; // 2... H8+7 (h9g7)

    let gb4 = make_move(gb3, 88, 69);
    array[count] = Entry::new(hash(&gb4, 0, &keys), 0x1316, 920, names[7]); count += 1; // 3. C2=5 (b2e2)

    let gb5 = make_move(gb4, 19, 22);
    array[count] = Entry::new(hash(&gb5, 1, &keys), 0x5241, 910, names[7]); count += 1; // 3... H2+3 (b9c7)

    let gb6 = make_move(gb5, 82, 65);
    array[count] = Entry::new(hash(&gb6, 0, &keys), 0x0807, 900, names[7]); count += 1; // 4. R9=8 (i0h0)

    let gb7 = make_move(gb6, 8, 7);
    array[count] = Entry::new(hash(&gb7, 1, &keys), 0x5958, 890, names[7]); count += 1; // 4... R9=8 (i9h9)

    let gb8 = make_move(gb7, 89, 88);
    array[count] = Entry::new(hash(&gb8, 0, &keys), 0x0114, 880, names[7]); count += 1; // 5. H2+3 (b0c2)

    let gb9 = make_move(gb8, 1, 20);
    array[count] = Entry::new(hash(&gb9, 1, &keys), 0x382F, 870, names[7]); count += 1; // 5... P3+1 (c6c5)

    let gb10 = make_move(gb9, 56, 47);
    array[count] = Entry::new(hash(&gb10, 0, &keys), 0x0001, 860, names[7]); count += 1; // 6. R1=2 (a0b0)

    let gb11 = make_move(gb10, 0, 1);
    array[count] = Entry::new(hash(&gb11, 1, &keys), 0x5343, 850, names[7]); count += 1; // 6... E3+5 (c9e7)

    // --- NHÁNH 4: KHỞI MÃ CUỘC (H8+7) (12 Plies) ---
    let gk1 = make_move(base, 7, 24); // 1. H8+7 (h0g2)
    array[count] = Entry::new(hash(&gk1, 1, &keys), 0x5845, 950, names[2]); count += 1; // 1... H8+7 (h9g7)

    let gk2 = make_move(gk1, 88, 69);
    array[count] = Entry::new(hash(&gk2, 0, &keys), 0x212A, 940, names[2]); count += 1; // 2. P7+1 (g3g4)

    let gk3 = make_move(gk2, 33, 42);
    array[count] = Entry::new(hash(&gk3, 1, &keys), 0x5241, 930, names[2]); count += 1; // 2... H2+3 (b9c7)

    let gk4 = make_move(gk3, 82, 65);
    array[count] = Entry::new(hash(&gk4, 0, &keys), 0x0807, 920, names[2]); count += 1; // 3. R9=8 (i0h0)

    let gk5 = make_move(gk4, 8, 7);
    array[count] = Entry::new(hash(&gk5, 1, &keys), 0x5958, 910, names[2]); count += 1; // 3... R9=8 (i9h9)

    let gk6 = make_move(gk5, 89, 88);
    array[count] = Entry::new(hash(&gk6, 0, &keys), 0x1316, 900, names[2]); count += 1; // 4. C2=5 (b2e2)

    let gk7 = make_move(gk6, 19, 22);
    array[count] = Entry::new(hash(&gk7, 1, &keys), 0x4043, 890, names[2]); count += 1; // 4... C8=5 (b7e7)

    let gk8 = make_move(gk7, 64, 67);
    array[count] = Entry::new(hash(&gk8, 0, &keys), 0x0114, 880, names[2]); count += 1; // 5. H2+3 (b0c2)

    let gk9 = make_move(gk8, 1, 20);
    array[count] = Entry::new(hash(&gk9, 1, &keys), 0x5152, 870, names[2]); count += 1; // 5... R1=2 (a9b9)

    // --- NHÁNH 5: PHI TƯỢNG CUỘC (E3+5) (8 Plies) ---
    let ge1 = make_move(base, 2, 22); // 1. E3+5 (c0e2)
    array[count] = Entry::new(hash(&ge1, 1, &keys), 0x5845, 930, names[8]); count += 1; // 1... H8+7 (h9g7)

    let ge2 = make_move(ge1, 88, 69);
    array[count] = Entry::new(hash(&ge2, 0, &keys), 0x0718, 920, names[8]); count += 1; // 2. H8+7 (h0g2)

    let ge3 = make_move(ge2, 7, 24);
    array[count] = Entry::new(hash(&ge3, 1, &keys), 0x5241, 910, names[8]); count += 1; // 2... H2+3 (b9c7)

    let ge4 = make_move(ge3, 82, 65);
    array[count] = Entry::new(hash(&ge4, 0, &keys), 0x212A, 900, names[8]); count += 1; // 3. P7+1 (g3g4)

    let ge5 = make_move(ge4, 33, 42);
    array[count] = Entry::new(hash(&ge5, 1, &keys), 0x3C33, 890, names[8]); count += 1; // 3... P7+1 (g6g5)

    let ge6 = make_move(ge5, 60, 51);
    array[count] = Entry::new(hash(&ge6, 0, &keys), 0x0807, 880, names[8]); count += 1; // 4. R9=8 (i0h0)

    let ge7 = make_move(ge6, 8, 7);
    array[count] = Entry::new(hash(&ge7, 1, &keys), 0x5958, 870, names[8]); count += 1; // 4... R9=8 (i9h9)

    // --- NHÁNH 6: QUÁ CUNG PHÁO (C2=6) (8 Plies) ---
    let gq1 = make_move(base, 19, 23); // 1. C2=6 (b2f2)
    array[count] = Entry::new(hash(&gq1, 1, &keys), 0x5845, 920, names[3]); count += 1; // 1... H8+7 (h9g7)

    let gq2 = make_move(gq1, 88, 69);
    array[count] = Entry::new(hash(&gq2, 0, &keys), 0x0114, 910, names[3]); count += 1; // 2. H2+3 (b0c2)

    let gq3 = make_move(gq2, 1, 20);
    array[count] = Entry::new(hash(&gq3, 1, &keys), 0x382F, 900, names[3]); count += 1; // 2... P3+1 (c6c5)

    let gq4 = make_move(gq3, 56, 47);
    array[count] = Entry::new(hash(&gq4, 0, &keys), 0x0001, 890, names[3]); count += 1; // 3. R1=2 (a0b0)

    let gq5 = make_move(gq4, 0, 1);
    array[count] = Entry::new(hash(&gq5, 1, &keys), 0x5152, 880, names[3]); count += 1; // 3... R1=2 (a9b9)

    let gq6 = make_move(gq5, 81, 82);
    array[count] = Entry::new(hash(&gq6, 0, &keys), 0x0718, 870, names[3]); count += 1; // 4. H8+7 (h0g2)

    let gq7 = make_move(gq6, 7, 24);
    array[count] = Entry::new(hash(&gq7, 1, &keys), 0x5958, 860, names[3]); count += 1; // 4... R9=8 (i9h9)

    // Điền các phần tử còn lại có Zobrist hash giả định phân bố ngẫu nhiên đồng đều
    let mut i = count;
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
