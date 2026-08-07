// ============================================================================
// MODULE ZOBRIST: BẢNG BĂM GIẢ NGẪU NHIÊN ZOBRIST HASHING CHO BÀN CỜ
// ============================================================================
// Thuật toán Zobrist Hashing cho phép cập nhật mã băm đại diện bàn cờ ở độ phức tạp $O(1)$
// khi thực hiện (MakeMove) hoặc hoàn tác (UndoMove) nước đi thông qua phép toán XOR `^`.
//
// Sử dụng bộ sinh số giả ngẫu nhiên `SplitMix64` tĩnh ở thời điểm biên dịch (`const fn`),
// gán mỗi cặp `(loại_quân, ô_bàn_cờ)` một số nguyên 64-bit ngẫu nhiên đồng đều.
// ============================================================================

/// Struct `Prng` triển khai bộ sinh số giả ngẫu nhiên 64-bit SplitMix64 hằng số.
pub struct Prng(pub u64);

impl Prng {
    /// Khởi tạo bộ sinh số ngẫu nhiên với giá trị hạt giống (seed).
    #[inline(always)]
    pub const fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// Sinh số nguyên 64-bit ngẫu nhiên tiếp theo bằng thuật toán SplitMix64.
    /// Đạt phân bố bit đồng đều và vượt qua các kiểm thử Dieharder suite.
    #[inline(always)]
    pub const fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
}

/// Struct `Zobrist` chứa toàn bộ mảng băm ngẫu nhiên cho 14 loại quân cờ x 90 ô cờ + lượt đi (Hỗ trợ Dual 128-bit).
pub struct Zobrist {
    /// Mảng 2 chiều 14 loại quân x 90 ô cờ chứa số 64-bit ngẫu nhiên [14][90]
    pub pieces: [[u64; 90]; 14],
    /// Mảng băm phụ 64-bit cao phục vụ Dual Zobrist 128-bit chống trùng băm
    pub pieces_high: [[u64; 90]; 14],
    /// Số 64-bit ngẫu nhiên đại diện cho lượt đi của bên Đen (Black)
    pub turn: u64,
    /// Số 64-bit ngẫu nhiên phụ cho lượt đi
    pub turn_high: u64,
}

impl Zobrist {
    /// Tạo lập bảng giá trị băm Zobrist ngẫu nhiên tại thời điểm biên dịch (`const fn`).
    #[inline(always)]
    pub const fn new() -> Self {
        let mut prng = Prng::new(0x123456789ABCDEF0);
        let mut pieces = [[0u64; 90]; 14];
        let mut pieces_high = [[0u64; 90]; 14];
        let mut p = 0;
        while p < 14 {
            let mut s = 0;
            while s < 90 {
                pieces[p][s] = prng.next();
                pieces_high[p][s] = prng.next();
                s += 1;
            }
            p += 1;
        }
        let turn = prng.next();
        let turn_high = prng.next();
        Self { pieces, pieces_high, turn, turn_high }
    }

    /// Trả về số băm Zobrist đại diện cho quân cờ `piece` tại ô `square`.
    #[inline(always)]
    pub const fn piece(&self, piece: usize, square: usize) -> u64 {
        self.pieces[piece][square]
    }

    /// Trả về cặp số băm Dual 128-bit (low, high) đại diện cho quân cờ `piece` tại ô `square`.
    #[inline(always)]
    pub const fn dual(&self, piece: usize, square: usize) -> (u64, u64) {
        (self.pieces[piece][square], self.pieces_high[piece][square])
    }

    /// Trả về số băm Zobrist đại diện cho việc chuyển lượt đi sang bên Đen.
    #[inline(always)]
    pub const fn side(&self) -> u64 {
        self.turn
    }
}

/// Khởi tạo hằng số tĩnh toàn cục `KEYS` lưu trữ toàn bộ mảng băm Zobrist sẵn sàng sử dụng.
pub static KEYS: Zobrist = Zobrist::new();

