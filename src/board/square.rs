// ============================================================================
// MODULE SQUARE: BIỂU DIỄN VÀ QUẢN LÝ TỌA ĐỘ BÀN CỜ CỜ TƯỚNG (90 Ô CỜ)
// ============================================================================
// Bàn cờ Cờ Tướng tiêu chuẩn quốc tế có kích thước 9 cột (Files: 0..8 tương ứng a..i)
// và 10 hàng (Ranks: 0..9 tương ứng 1..10).
// Tổng số ô cờ trên bàn cờ là: 9 cột * 10 hàng = 90 ô (chỉ số tuyến tính 0..89).
//
// Mô hình tọa độ tuyến tính 1D (Single-word memory index):
//   index = rank * 9 + file
//   file  = index % 9
//   rank  = index / 9
// ============================================================================

/// Struct `Square` bọc một giá trị kiểu `u8` đại diện cho một ô trên bàn cờ (0..89).
/// Sử dụng `#[repr(transparent)]` ngầm hiểu bộ nhớ bằng đúng 1 byte duy nhất của `u8`,
/// giúp triệt tiêu hoàn toàn overhead truyền tham trị trong CPU registers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Square(pub u8);

impl Square {
    /// Khởi tạo ô bàn cờ mới từ tọa độ cột (`file`: 0..8) và hàng (`rank`: 0..9).
    ///
    /// - `file`: Chỉ số cột từ 0 (cột a) đến 8 (cột i).
    /// - `rank`: Chỉ số hàng từ 0 (hàng 1) đến 9 (hàng 10).
    /// - Trả về: Đối tượng `Square` mang chỉ số tuyến tính `rank * 9 + file`.
    ///
    /// Chỉ thị `#[inline(always)]` ép trình biên dịch Rust nhúng thẳng phép toán vào hot path,
    /// loại bỏ overhead gọi hàm (function call instruction overhead) trong CPU.
    #[inline(always)]
    pub const fn new(file: u8, rank: u8) -> Self {
        // Công thức tính vị trí tuyến tính trên mảng 1D 90 phần tử
        Self(rank * 9 + file)
    }

    /// Trả về chỉ số cột (file index) của ô cờ (từ 0 đến 8).
    ///
    /// Phép chia lấy dư `% 9` trích xuất vị trí nằm trên cột nào trong hàng 9 cột.
    #[inline(always)]
    pub const fn file(self) -> u8 {
        self.0 % 9
    }

    /// Trả về chỉ số hàng (rank index) của ô cờ (từ 0 đến 9).
    ///
    /// Phép chia nguyên `/ 9` xác định ô cờ đang thuộc hàng thứ mấy từ đáy bàn cờ.
    #[inline(always)]
    pub const fn rank(self) -> u8 {
        self.0 / 9
    }

    /// Trả về vị trí ô cờ dưới dạng chỉ số mảng `usize` (0..89) dùng cho truy xuất mảng.
    ///
    /// Chuyển đổi an toàn từ `u8` sang `usize` không tốn chi phí xung nhịp CPU.
    #[inline(always)]
    pub const fn index(self) -> usize {
        self.0 as usize
    }

    /// Kiểm tra ô cờ có nằm trong phạm vi bàn cờ hợp lệ (0..89) hay không.
    ///
    /// Trả về `true` nếu chỉ số ô cờ nhỏ hơn 90, ngược lại trả về `false`.
    #[inline(always)]
    pub const fn valid(self) -> bool {
        self.0 < 90
    }

    /// Lật đối xứng ô cờ theo phương ngang bàn cờ theo góc nhìn phe Đen (Black Perspective).
    ///
    /// Hàng mới = `9 - rank_hiện_tại`. Giúp tính toán điểm số NNUE và PST đối xứng giữa 2 bên.
    #[inline(always)]
    pub const fn flip(self) -> Self {
        // Nếu ô cờ không hợp lệ, giữ nguyên không thực hiện phép tính
        if self.0 >= 90 {
            return self;
        }
        // Giữ nguyên cột (file), đảo ngược hàng (9 - rank)
        Self::new(self.file(), 9 - self.rank())
    }
}

