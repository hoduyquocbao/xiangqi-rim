// ============================================================================
// MODULE POSITION: QUẢN LÝ TRẠNG THÁI TOÀN CỤC CỦA BÀN CỜ (448 BYTES, ALIGN 64)
// ============================================================================
// Cấu trúc `Position` lưu trữ toàn bộ trạng thái vật lý của bàn cờ Cờ Tướng.
// Kích thước chính xác: 393 bytes dữ liệu thực tế + 55 bytes đệm (padding) = 448 bytes.
//
// Ý nghĩa căn lề `#[repr(C, align(64))]`:
// - Kích thước 448 bytes là bội số của 64 ($448 = 7 \times 64$).
// - Vừa khít đúng 7 dòng bộ nhớ đệm L1 Cache (L1 Data Cache Lines) của CPU.
// - Triệt tiêu hoàn toàn hiện tượng tranh chấp bộ đệm (False Sharing / True Sharing)
//   khi các luồng tìm kiếm Lazy SMP truy cập song song trên nhiều nhân CPU khác nhau.
// ============================================================================

use super::bitboard::Bitboard;
use super::square::Square;
use super::state::State;
use super::zobrist::KEYS;

/// Struct `Position` lưu trữ trạng thái bàn cờ Cờ Tướng với căn lề physical 64-byte.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    /// Mặt nạ Bitboard cho 2 phe chơi (index 0: Red, index 1: Black) [32 bytes]
    pub color: [Bitboard; 2],
    /// Mặt nạ Bitboard cho 14 loại quân cờ thực tế [224 bytes]
    pub piece: [Bitboard; 14],
    /// Mặt nạ Bitboard tổng hợp toàn bộ ô có quân cờ trên bàn [16 bytes]
    pub occupied: Bitboard,
    /// Khóa băm ngẫu nhiên Zobrist Hash 64-bit hiện tại của bàn cờ [8 bytes]
    pub hash: u64,
    /// Bộ đếm số nửa nước đi chưa ăn quân / chưa đi Tốt (Rule50 / Halfmove clock) [2 bytes]
    pub rule: u16,
    /// Số nửa nước đi hiện tại từ đầu ván cờ (Ply counter) [2 bytes]
    pub ply: u16,
    /// Phe nắm lượt đi hiện tại (0: Đỏ - Red, 1: Đen - Black) [1 byte]
    pub side: u8,
    /// Cờ đánh dấu vị trí Tướng đang bị chiếu [1 byte]
    pub check: u8,
    /// Mã quân cờ bị ăn ở nước đi gần nhất (14 nếu là nước đi không ăn quân) [1 byte]
    pub captured: u8,
    /// Vị trí ô cờ của Tướng Đỏ (index 0) và Tướng Đen (index 1) [2 bytes]
    pub king: [u8; 2],
    /// Mảng lưu số lượng tồn tại của từng loại quân cờ (0..13) [14 bytes]
    pub counts: [u8; 14],
    /// Mảng 90 ô lưu trực tiếp chỉ số quân cờ đang đứng (0..13, 14 là ô trống) [90 bytes]
    pub grid: [u8; 90],
    /// Mảng đệm padding để tổng kích thước struct đạt đúng 448 bytes (7x64B L1 Cache Lines) [55 bytes]
    pub pad: [u8; 55],
}

impl Default for Position {
    /// Khởi tạo mặc định đối tượng `Position` rỗng.
    fn default() -> Self {
        Self::empty()
    }
}

impl Position {
    /// Khởi tạo một bàn cờ `Position` rỗng hoàn toàn.
    #[inline(always)]
    pub const fn empty() -> Self {
        Self {
            color: [Bitboard::empty(); 2],
            piece: [Bitboard::empty(); 14],
            occupied: Bitboard::empty(),
            hash: 0,
            rule: 0,
            ply: 1,
            side: 0,
            check: 0,
            captured: 14,
            king: [255; 2],
            counts: [0; 14],
            grid: [14; 90],
            pad: [0; 55],
        }
    }

    /// Đặt lại toàn bộ dữ liệu bàn cờ về trạng thái rỗng ban đầu.
    #[inline(always)]
    pub fn clear(&mut self) {
        *self = Self::empty();
    }

    /// Truy xuất chỉ số quân cờ tại vị trí ô `square` (0..89). Trả về 14 (ô trống) nếu vượt biên.
    #[inline(always)]
    pub fn at(&self, square: u8) -> u8 {
        if square >= 90 {
            return 14;
        }
        self.grid[square as usize]
    }

    /// Đặt một quân cờ `piece` (0..13) vào vị trí ô `square` (0..89) trên bàn cờ.
    /// Tự động gỡ quân cờ cũ nếu ô đó đã có quân từ trước.
    #[inline(always)]
    pub fn put(&mut self, piece: u8, square: u8) {
        if piece >= 14 || square >= 90 {
            return;
        }
        let old = self.grid[square as usize];
        if old < 14 {
            self.take(square);
        }

        let sq = Square(square);
        let color = if piece < 7 { 0 } else { 1 };

        // Cập nhật mảng ô cờ tuyến tính
        self.grid[square as usize] = piece;
        // Cập nhật các mặt nạ Bitboard tương ứng
        self.piece[piece as usize].set(sq);
        self.color[color].set(sq);
        self.occupied.set(sq);
        // Tăng số lượng loại quân tương ứng
        self.counts[piece as usize] += 1;

        // Nếu quân đặt vào là Tướng (King - mã 0 hoặc 7), cập nhật vị trí Tướng
        if piece % 7 == 0 {
            self.king[color] = square;
        }
    }

    /// Gỡ bỏ quân cờ tại vị trí ô `square` khỏi bàn cờ và trả về chỉ số quân bị gỡ.
    #[inline(always)]
    pub fn take(&mut self, square: u8) -> u8 {
        if square >= 90 {
            return 14;
        }
        let piece = self.grid[square as usize];
        if piece < 14 {
            let sq = Square(square);
            let color = if piece < 7 { 0 } else { 1 };

            // Đánh dấu ô thành trống (mã 14)
            self.grid[square as usize] = 14;
            // Xóa bit khỏi các mặt nạ Bitboard
            self.piece[piece as usize].clear(sq);
            self.color[color].clear(sq);
            self.occupied.clear(sq);
            // Giảm số lượng loại quân tương ứng
            self.counts[piece as usize] -= 1;
        }
        piece
    }

    /// Tính toán lại toàn bộ khóa băm Zobrist Hash 64-bit từ trạng thái bàn cờ hiện tại.
    #[inline(always)]
    pub fn compute(&self) -> u64 {
        let mut hash = 0u64;
        let mut s = 0usize;
        while s < 90 {
            let piece = self.grid[s];
            if piece < 14 {
                // Phép XOR Zobrist key cho quân cờ tại ô s
                hash ^= KEYS.piece(piece as usize, s);
            }
            s += 1;
        }
        // Phép XOR Zobrist key đại diện cho bên Đen nắm lượt đi
        if self.side == 1 {
            hash ^= KEYS.side();
        }
        hash
    }

    /// Thực hiện nước đi (MakeMove) di chuyển quân từ ô `from` sang ô `to`.
    /// Trả về đối tượng `State` chứa thông tin lưu vết để khôi phục khi hoàn tác.
    #[inline(always)]
    pub fn apply(&mut self, from: u8, to: u8) -> State {
        if from >= 90 || to >= 90 || from == to {
            return State::new(14, self.captured, self.check, self.rule, self.ply, self.hash);
        }

        let moving = self.grid[from as usize];
        let captured = self.grid[to as usize];

        // Tạo đối tượng lưu vết State để phục vụ UndoMove
        let state = State::new(captured, self.captured, self.check, self.rule, self.ply, self.hash);

        // 1. Xóa quân bị ăn tại ô đích (nếu có)
        if captured < 14 {
            self.take(to);
            self.hash ^= KEYS.piece(captured as usize, to as usize);
            self.rule = 0; // Đặt lại bộ đếm rule50 khi có ăn quân
        } else if moving < 14 && moving % 7 == 6 {
            self.rule = 0; // Đặt lại bộ đếm rule50 khi di chuyển Tốt
        } else {
            self.rule += 1; // Tăng bộ đếm rule50
        }

        // 2. Di chuyển quân từ ô đi tới ô đến
        if moving < 14 {
            self.take(from);
            self.hash ^= KEYS.piece(moving as usize, from as usize);

            self.put(moving, to);
            self.hash ^= KEYS.piece(moving as usize, to as usize);
        }

        self.captured = captured;
        self.ply += 1;

        // 3. Đổi phe nắm lượt đi và cập nhật Zobrist hash phe đi
        self.side ^= 1;
        self.hash ^= KEYS.side();

        state
    }

    /// Hoàn tác nước đi (UndoMove) khôi phục bàn cờ về trạng thái trước đó từ `State`.
    #[inline(always)]
    pub fn revert(&mut self, from: u8, to: u8, state: &State) {
        if from >= 90 || to >= 90 || from == to {
            self.rule = state.rule;
            self.ply = state.ply;
            self.check = state.check;
            self.captured = state.prev;
            self.hash = state.hash;
            return;
        }

        // 1. Đổi lại phe nắm lượt đi
        self.side ^= 1;

        let moving = self.grid[to as usize];

        // 2. Đưa quân cờ từ ô đích về lại ô xuất phát
        if moving < 14 {
            self.take(to);
            self.put(moving, from);
        }

        // 3. Khôi phục quân cờ đã bị ăn tại ô đích (nếu có)
        if state.captured < 14 {
            self.put(state.captured, to);
        }

        // 4. Phục hồi toàn bộ các trường biến trạng thái cũ
        self.rule = state.rule;
        self.ply = state.ply;
        self.check = state.check;
        self.captured = state.prev;
        self.hash = state.hash;
    }
}

