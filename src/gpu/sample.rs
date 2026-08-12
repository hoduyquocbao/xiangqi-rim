// ============================================================================
// XIANGTI ENGINE: ĐẠI DIỆN VỊ TRÍ THẾ CỜ / VECTƠ ĐẶC TRƯNG MẪU (SAMPLE)
// ============================================================================
// Struct `Sample` đại diện cho một thế cờ Xiangqi hoặc vectơ tích lũy đặc trưng
// truyền tới GPU theo lô (batch). Căn lề 64-byte vật lý phòng chống False Sharing.
// Tuân thủ 100% định danh từ đơn tiếng Anh và 100% chú thích tiếng Việt.
// ============================================================================

use super::status::Status; // Nhập kiểu enum Status từ module status
use crate::board::Position; // Nhập kiểu struct Position từ module board

/// Trait `Sampleable`: Định nghĩa khả năng mã hóa và tương tác với dữ liệu thế cờ mẫu.
pub trait Sampleable { // Định nghĩa trait Sampleable
    /// Phương thức `encode`: Mã hóa bàn cờ mảng 90 ô và phe lượt đi vào mẫu thế cờ.
    fn encode(&mut self, grid: &[u8; 90], side: u8) -> Status; // Chữ ký hàm encode
    /// Phương thức `score`: Truy vấn điểm số đánh giá centipawn sau tính toán.
    fn score(&self) -> i32; // Chữ ký hàm score
    /// Phương thức `valid`: Kiểm tra tính hợp lệ của mẫu thế cờ.
    fn valid(&self) -> bool; // Chữ ký hàm valid
    /// Phương thức `clear`: Xóa dữ liệu mẫu thế cờ về trạng thái mặc định.
    fn clear(&mut self); // Chữ ký hàm clear
} // Kết thúc trait Sampleable

/// Struct `Sample`: Đại diện dữ liệu 1 thế cờ căn lề 64-byte (128 bytes total).
#[repr(C, align(64))] // Căn lề 64-byte phòng False Sharing trên CPU Cache Line
#[derive(Clone, Copy, Debug, PartialEq, Eq)] // Tự động derive các trait cơ bản
pub struct Sample { // Định nghĩa struct Sample
    /// Mảng 90 ô cờ trên bàn cờ Xiangqi (90 bytes, offset 0..90)
    grid: [u8; 90], // Trường mảng bàn cờ grid
    /// Vị trí ô Tướng Đỏ và Tướng Đen (2 bytes, offset 90..92)
    king: [u8; 2], // Trường vị trí tướng king
    /// Phe nắm lượt đi (0: Đỏ, 1: Đen) (1 byte, offset 92)
    side: u8, // Trường phe lượt đi side
    /// Trạng thái chiếu/kiểm tra bàn cờ (1 byte, offset 93)
    state: u8, // Trường trạng thái state
    /// Điểm số đánh giá centipawn sau khi GPU tính toán (4 bytes, offset 96..100)
    score: i32, // Trường điểm số score
    /// Chỉ số thứ tự của thế cờ trong lô (4 bytes, offset 100..104)
    index: u32, // Trường chỉ số index
    /// Khóa băm Zobrist của thế cờ (8 bytes, offset 104..112)
    hash: u64, // Trường khóa băm hash
    /// Mảng đệm 16 byte làm tròn kích thước struct lên đúng 128 bytes (2 cache lines) (16 bytes, offset 112..128)
    pad: [u8; 16], // Trường đệm pad căn lề 128 bytes
} // Kết thúc struct Sample

impl Sample { // Khối triển khai các phương thức cho Sample
    /// Khởi tạo mẫu Sample rỗng mặc định.
    #[inline(always)] // Inline hàm tạo mặc định hot path
    pub const fn new() -> Self { // Hàm new tạo bản thể mặc định
        Self { // Trả về struct Sample mặc định
            grid: [14u8; 90], // Mảng ô cờ 14 (ô trống EMPTY)
            king: [0u8; 2], // Vị trí tướng zero
            side: 0, // Phe mặc định 0 (Đỏ)
            state: 0, // Trạng thái 0
            score: 0, // Điểm số 0
            index: 0, // Chỉ số 0
            hash: 0, // Khóa băm 0
            pad: [0u8; 16], // Mảng đệm zero
        } // Kết thúc struct Sample
    } // Kết thúc hàm new

    #[inline(always)]
    pub fn pack(pos: &Position, index: u32) -> Self {
        Self {
            grid: pos.grid,
            king: pos.king,
            side: pos.side,
            state: 1,
            score: 0,
            index,
            hash: pos.hash,
            pad: [0u8; 16],
        }
    }

    /// Trả về điểm số centipawn của mẫu thế cờ.
    #[inline(always)] // Inline phương thức đọc điểm số
    pub fn score(&self) -> i32 { // Hàm score trả về i32
        self.score // Trả về score
    } // Kết thúc hàm score

    /// Lưu trữ kết quả điểm số centipawn vào mẫu thế cờ.
    #[inline(always)] // Inline phương thức ghi điểm số
    pub fn store(&mut self, score: i32) { // Hàm store lưu điểm số
        self.score = score; // Gán điểm số mới
    } // Kết thúc hàm store

    /// Trả về phe nắm lượt đi.
    #[inline(always)] // Inline phương thức đọc phe
    pub fn side(&self) -> u8 { // Hàm side trả về u8
        self.side // Trả về side
    } // Kết thúc hàm side

    /// Trả về tham chiếu tới mảng 90 ô cờ trên bàn cờ.
    #[inline(always)] // Inline phương thức đọc mảng ô cờ
    pub fn grid(&self) -> &[u8; 90] { // Hàm grid trả về &[u8; 90]
        &self.grid // Trả về tham chiếu &self.grid
    } // Kết thúc hàm grid

    /// Nạp dữ liệu mảng ô cờ và phe lượt đi vào mẫu thế cờ.
    pub fn load(&mut self, grid: &[u8; 90], side: u8) { // Hàm load nạp dữ liệu
        self.grid.copy_from_slice(grid); // Sao chép 90 ô cờ
        self.side = side; // Gán phe lượt đi
        self.state = 1; // Đặt trạng thái 1
    } // Kết thúc hàm load


    /// Trả về chỉ số thứ tự của mẫu trong lô.
    #[inline(always)] // Inline phương thức đọc chỉ số
    pub fn index(&self) -> u32 { // Hàm index trả về u32
        self.index // Trả về index
    } // Kết thúc hàm index

    /// Trả về khóa băm Zobrist.
    #[inline(always)] // Inline phương thức đọc băm
    pub fn hash(&self) -> u64 { // Hàm hash trả về u64
        self.hash // Trả về hash
    } // Kết thúc hàm hash

    /// Kiểm tra mẫu thế cờ có hợp lệ không.
    #[inline(always)] // Inline phương thức kiểm tra hợp lệ
    pub fn valid(&self) -> bool { // Hàm valid trả về bool
        self.side < 2 // Kiểm tra phe hợp lệ (< 2)
    } // Kết thúc hàm valid

    /// Xóa dữ liệu mẫu về trạng thái 0.
    #[inline(always)] // Inline phương thức xóa dữ liệu
    pub fn clear(&mut self) { // Hàm clear xóa dữ liệu
        self.grid = [14u8; 90]; // Xóa mảng grid về 14 (EMPTY)
        self.king = [0u8; 2]; // Xóa mảng king về 0
        self.side = 0; // Đặt side về 0
        self.state = 0; // Đặt state về 0
        self.score = 0; // Đặt score về 0
        self.index = 0; // Đặt index về 0
        self.hash = 0; // Đặt hash về 0
    } // Kết thúc hàm clear
} // Kết thúc khối impl Sample

impl Sampleable for Sample { // Triển khai trait Sampleable cho Sample
    fn encode(&mut self, grid: &[u8; 90], side: u8) -> Status { // Triển khai encode
        if side > 1 { // Nếu phe không hợp lệ (> 1)
            return Status::Fault; // Trả về lỗi Fault
        } // Kết thúc kiểm tra side
        self.grid.copy_from_slice(grid); // Sao chép mảng bàn cờ vào grid
        self.side = side; // Gán phe side
        self.state = 1; // Đánh dấu trạng thái state = 1
        Status::Ready // Trả về trạng thái Ready
    } // Kết thúc phương thức encode

    fn score(&self) -> i32 { // Triển khai score
        self.score // Trả về điểm số centipawn
    } // Kết thúc phương thức score

    fn valid(&self) -> bool { // Triển khai valid
        self.valid() // Gọi hàm valid nội tại
    } // Kết thúc phương thức valid

    fn clear(&mut self) { // Triển khai clear
        self.clear(); // Gọi hàm clear nội tại
    } // Kết thúc phương thức clear
} // Kết thúc impl Sampleable for Sample

impl Default for Sample { // Triển khai trait Default cho Sample
    fn default() -> Self { // Hàm default khởi tạo mặc định
        Self::new() // Gọi hàm new
    } // Kết thúc hàm default
} // Kết thúc impl Default for Sample

#[cfg(test)] // Module kiểm thử unit tests cho Sample
mod tests { // Cấu hình module tests
    use super::*; // Nhập tất cả đối tượng từ module cha

    #[test] // Đánh dấu hàm kiểm thử cấu trúc và căn lề bộ nhớ 128-byte
    fn test_sample_struct_layout_and_alignment() { // Hàm test layout Sample
        assert_eq!(std::mem::size_of::<Sample>(), 128); // Kiểm tra size_of đúng 128 bytes
        assert_eq!(std::mem::align_of::<Sample>(), 64); // Kiểm tra align_of đúng 64 bytes
    } // Kết thúc hàm test_sample_struct_layout_and_alignment

    #[test] // Đánh dấu hàm kiểm thử mã hóa và lưu trữ điểm số
    fn test_sample_encoding_and_operations() { // Hàm test encode và thao tác Sample
        let mut sample = Sample::new(); // Khởi tạo mẫu thế cờ rỗng
        assert_eq!(sample.score(), 0); // Điểm số ban đầu bằng 0
        assert!(sample.valid()); // Phe ban đầu 0 là hợp lệ

        let grid = [1u8; 90]; // Mảng cờ thử nghiệm với các ô cờ bằng 1
        let res = sample.encode(&grid, 0); // Mã hóa bàn cờ với phe Đỏ (0)
        assert_eq!(res, Status::Ready); // Kết quả trả về Status::Ready
        assert_eq!(sample.side(), 0); // Phe là 0

        sample.store(350); // Ghi điểm số 350 centipawns
        assert_eq!(sample.score(), 350); // Điểm số cập nhật bằng 350

        sample.clear(); // Xóa mẫu thế cờ về 0
        assert_eq!(sample.score(), 0); // Điểm số sau clear bằng 0
    } // Kết thúc hàm test_sample_encoding_and_operations
} // Kết thúc module tests
