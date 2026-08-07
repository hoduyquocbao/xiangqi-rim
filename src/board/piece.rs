// ============================================================================
// MODULE PIECE: BIỂU DIỄN PHE CHƠI (COLOR), LOẠI QUÂN (ROLE) VÀ QUÂN CỜ (PIECE)
// ============================================================================
// Trong Cờ Tướng:
// - Có 2 phe chơi: Đỏ (Red - đi trước) và Đen (Black - đi sau).
// - Có 7 loại quân cờ: Tướng (King), Sĩ (Advisor), Tượng (Bishop/Elephant),
//   Mã (Knight), Xe (Rook), Pháo (Cannon), Tốt (Pawn).
// - Tổng cộng 14 loại quân cờ thực tế + 1 định danh ô trống (Empty = 14).
// ============================================================================

/// Enum `Color` đại diện cho phe chơi trong ván cờ.
/// `Red = 0` (Bên Đỏ), `Black = 1` (Bên Đen).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Color {
    Red = 0,
    Black = 1,
}

impl Color {
    /// Đảo ngược lượt đi / phe chơi (Đỏ thành Đen, Đen thành Đỏ).
    #[inline(always)]
    pub const fn flip(self) -> Self {
        match self {
            Self::Red => Self::Black,
            Self::Black => Self::Red,
        }
    }

    /// Lấy chỉ số nguyên của phe chơi (0: Đỏ, 1: Đen) dùng cho truy xuất mảng.
    #[inline(always)]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Ký tự đại diện cho phe chơi trong định dạng FEN ('w' đại diện cho Red, 'b' đại diện cho Black).
    #[inline(always)]
    pub const fn char(self) -> char {
        match self {
            Self::Red => 'w',
            Self::Black => 'b',
        }
    }
}

/// Enum `Role` đại diện cho 7 loại quân cờ Tướng.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Role {
    King = 0,    // Tướng / Soái
    Advisor = 1, // Sĩ
    Bishop = 2,  // Tượng / Voi
    Knight = 3,  // Mã / Ngựa
    Rook = 4,    // Xe
    Cannon = 5,  // Pháo
    Pawn = 6,    // Tốt / Binh
}

impl Role {
    /// Lấy chỉ số loại quân (từ 0 đến 6) dùng cho mảng tra cứu.
    #[inline(always)]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Chuyển đổi loại quân thành ký tự FEN in hoa tiêu chuẩn ('K', 'A', 'B', 'N', 'R', 'C', 'P').
    #[inline(always)]
    pub const fn char(self) -> char {
        match self {
            Self::King => 'K',
            Self::Advisor => 'A',
            Self::Bishop => 'B',
            Self::Knight => 'N',
            Self::Rook => 'R',
            Self::Cannon => 'C',
            Self::Pawn => 'P',
        }
    }
}

/// Struct `Piece` bọc số nguyên `u8` mã hóa quân cờ:
/// - Từ 0 đến 6: Quân Đỏ (King..Pawn)
/// - Từ 7 đến 13: Quân Đen (King..Pawn)
/// - Từ 14 trở lên: Ô trống (Empty / None)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Piece(pub u8);

impl Piece {
    /// Khởi tạo đối tượng `Piece` mới từ phe chơi (`color`) và loại quân (`role`).
    /// Mã quân cờ = `color_index * 7 + role_index`.
    #[inline(always)]
    pub const fn new(color: Color, role: Role) -> Self {
        Self((color.index() * 7 + role.index()) as u8)
    }

    /// Khởi tạo `Piece` trực tiếp từ mã nguyên `u8` (0..14).
    #[inline(always)]
    pub const fn make(code: u8) -> Self {
        Self(code)
    }

    /// Trả về đối tượng `Piece` đại diện cho ô trống (mã 14).
    #[inline(always)]
    pub const fn none() -> Self {
        Self(14)
    }

    /// Lấy chỉ số mã quân cờ dưới dạng `usize` (0..14).
    #[inline(always)]
    pub const fn index(self) -> usize {
        self.0 as usize
    }

    /// Kiểm tra xem ô cờ có phải ô trống hay không (mã >= 14).
    #[inline(always)]
    pub const fn empty(self) -> bool {
        self.0 >= 14
    }

    /// Kiểm tra xem ô cờ có chứa quân cờ hợp lệ hay không (mã < 14).
    #[inline(always)]
    pub const fn valid(self) -> bool {
        self.0 < 14
    }

    /// Trả về phe chơi của quân cờ (`Some(Color::Red)`, `Some(Color::Black)`, hoặc `None` nếu trống).
    #[inline(always)]
    pub fn color(self) -> Option<Color> {
        if self.0 < 7 {
            Some(Color::Red)
        } else if self.0 < 14 {
            Some(Color::Black)
        } else {
            None
        }
    }

    /// Trả về loại quân cờ (`Some(Role)`, hoặc `None` nếu ô trống).
    #[inline(always)]
    pub fn role(self) -> Option<Role> {
        match self.0 % 7 {
            0 if self.0 < 14 => Some(Role::King),
            1 if self.0 < 14 => Some(Role::Advisor),
            2 if self.0 < 14 => Some(Role::Bishop),
            3 if self.0 < 14 => Some(Role::Knight),
            4 if self.0 < 14 => Some(Role::Rook),
            5 if self.0 < 14 => Some(Role::Cannon),
            6 if self.0 < 14 => Some(Role::Pawn),
            _ => None,
        }
    }

    /// Trả về ký tự hiển thị FEN của quân cờ:
    /// - Quân Đỏ: In hoa ('K', 'A', 'B', 'N', 'R', 'C', 'P')
    /// - Quân Đen: In thường ('k', 'a', 'b', 'n', 'r', 'c', 'p')
    /// - Ô trống: Dấu chấm ('.')
    #[inline(always)]
    pub fn char(self) -> char {
        if self.0 >= 14 {
            return '.';
        }
        let role = match self.role() {
            Some(r) => r.char(),
            None => return '.',
        };
        if self.0 < 7 {
            role
        } else {
            role.to_ascii_lowercase()
        }
    }

    /// Phân tích ký tự FEN thành đối tượng `Piece` tương ứng.
    /// Ép buộc inlining `#[inline(always)]` tăng tốc giải mã FEN.
    #[inline(always)]
    pub fn parse(char: char) -> Self {
        match char {
            'K' => Self::new(Color::Red, Role::King),
            'A' => Self::new(Color::Red, Role::Advisor),
            'B' => Self::new(Color::Red, Role::Bishop),
            'N' => Self::new(Color::Red, Role::Knight),
            'R' => Self::new(Color::Red, Role::Rook),
            'C' => Self::new(Color::Red, Role::Cannon),
            'P' => Self::new(Color::Red, Role::Pawn),
            'k' => Self::new(Color::Black, Role::King),
            'a' => Self::new(Color::Black, Role::Advisor),
            'b' => Self::new(Color::Black, Role::Bishop),
            'n' => Self::new(Color::Black, Role::Knight),
            'r' => Self::new(Color::Black, Role::Rook),
            'c' => Self::new(Color::Black, Role::Cannon),
            'p' => Self::new(Color::Black, Role::Pawn),
            _ => Self::none(),
        }
    }
}

