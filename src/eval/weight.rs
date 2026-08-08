// ============================================================================
// MODULE WEIGHT: MẢNG TRỌNG SỐ TĨNH VÀ ĐỊNH THIÊN CHO MẠNG NNUE (NNUE WEIGHT MATRIX)
// ============================================================================
// `weight.rs` quản lý các mảng trọng số ma trận cho kiến trúc NNUE HalfKAv2_hm:
// - `TOTAL`: Tổng số đặc trưng $65,536$ đặc trưng vị trí ($90 \times 14 \times 52$).
// - `DIM`: Kích thước ẩn bộ tích lũy Accumulator ($256$ chiều int16).
// - Căn lề 64-byte `#[repr(C, align(64))]` khớp L1 Cache Line tối ưu SIMD vectorization.
// - Hỗ trợ nạp trọng số huấn luyện từ tệp nhị phân format XRNN.
// ============================================================================

/// Tổng số đặc trưng vị trí HalfKAv2_hm NNUE = 65,536
pub const TOTAL: usize = 65536;
/// Kích thước không gian ẩn bộ tích lũy Accumulator = 256
pub const DIM: usize = 256;

/// Struct `Weight` lưu trữ ma trận định thiên bias và trọng số, căn lề 64-byte.
/// Feature weights được lưu trên Heap (~32MB) thông qua `Box<[[i16; DIM]]>`.
#[repr(C, align(64))]
#[derive(Clone, Debug)]
pub struct Weight {
    /// Mảng định thiên bias 256 phần tử i16 [512 bytes]
    pub bias: [i16; DIM],
    /// Ma trận trọng số đặc trưng TOTAL×DIM trên Heap (~32MB)
    /// `None` khi chưa nạp trọng số (sử dụng mảng 0 mặc định)
    pub matrix: Option<Box<Vec<[i16; DIM]>>>,
}

impl Default for Weight {
    /// Khởi tạo mặc định đối tượng Weight.
    fn default() -> Self {
        Self::new()
    }
}

/// Struct bao bọc mảng 0 tĩnh đảm bảo căn lề 64-byte phần cứng.
#[repr(C, align(64))]
struct Aligned([i16; DIM]);

/// Mảng 0 tĩnh căn lề 64-byte tuyệt đối chống lỗi #GP(0) trên x86_64 SIMD.
static ZEROES: Aligned = Aligned([0; DIM]);

impl Weight {
    /// Khởi tạo mảng định thiên rỗng bằng 0 (chưa nạp feature weights).
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            bias: [0; DIM],
            matrix: None,
        }
    }

    /// Truy xuất mảng trọng số đặc trưng theo chỉ số đặc trưng `index: 0..65535`.
    /// Nếu feature weights đã được nạp từ tệp, trả về trọng số thực.
    /// Nếu chưa nạp, trả về mảng 0 (hành vi mặc định tương thích ngược).
    #[inline(always)]
    pub fn feature(&self, index: usize) -> &[i16; DIM] {
        if let Some(ref matrix) = self.matrix {
            if index < matrix.len() {
                return &matrix[index];
            }
        }
        &ZEROES.0
    }

    /// Kiểm tra xem trọng số đặc trưng đã được nạp hay chưa.
    #[inline(always)]
    pub fn loaded(&self) -> bool {
        self.matrix.is_some()
    }
}
