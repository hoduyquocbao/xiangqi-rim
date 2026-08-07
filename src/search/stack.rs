// ============================================================================
// MODULE STACK: KHUNG NGỮ CẢNH ĐỆ QUY TÌM KIẾM THEO ĐỘ SÂU PLY (SEARCH STACK FRAME)
// ============================================================================
// `Stack` lưu trữ toàn bộ trạng thái ngữ cảnh tại độ sâu ply hiện tại trong cây đệ quy PVS:
// - `pv`: Tuyến biến thể chính (PV Line) tại tầng ply này.
// - `killer`: Mảng 2 nước đi sát thủ (Killer Moves) gây cắt giảm Alpha-Beta nhanh.
// - `mv`: Nước đi đã thực thi dẫn tới ply này.
// - `eval`: Điểm đánh giá tĩnh (Static Evaluation).
// - `check`: Cờ đánh dấu vị trí có đang bị chiếu hay không.
// - `null`: Cờ đánh dấu có vừa thực hiện nước đi rỗng (Null Move) hay không.
// - Căn lề 64-byte `#[repr(C, align(64))]` tối ưu L1 Cache Line.
// ============================================================================

use crate::movegen::types::Move;
use crate::search::pv::Pv;

/// Struct `Stack` đại diện cho một khung đệ quy (Stack Frame) tìm kiếm tại ply nhất định, căn lề 64-byte.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct Stack {
    /// Khóa băm Zobrist hash tại ply này
    pub hash: u64,
    /// Tuyến biến thể chính PV tại ply hiện tại
    pub pv: Pv,
    /// Mảng chứa 2 nước đi sát thủ (Killer Moves) tại ply này
    pub killer: [Move; 2],
    /// Nước đi dẫn tới vị trí này
    pub mv: Move,
    /// Điểm đánh giá static eval
    pub eval: i32,
    /// Cờ vị trí có bị chiếu hay không
    pub check: bool,
    /// Cờ nước đi rỗng Null Move Pruning
    pub null: bool,
    /// Tầng độ sâu ply (0..MAX_PLY)
    pub ply: usize,
}

impl Default for Stack {
    /// Khởi tạo mặc định đối tượng Stack.
    fn default() -> Self {
        Self::new()
    }
}

impl Stack {
    /// Khởi tạo một đối tượng Stack rỗng ban đầu.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            hash: 0,
            pv: Pv::new(),
            killer: [Move::none(); 2],
            mv: Move::none(),
            eval: 0,
            check: false,
            null: false,
            ply: 0,
        }
    }
}

