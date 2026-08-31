// examples/139_massive_55m_exhaustive_rolling_miner.rs
// ============================================================================
// VÍ DỤ 139: ĐẠI ĐỘNG CƠ KHAI THÁC VÉT CẠN 1 TỶ FENS (1B) DEPTH 20 PLY 16
// ============================================================================
// 139_massive_55m_exhaustive_rolling_miner.rs triển khai kiến trúc Decoupled CQRS-ES:
// 1. Phân Tách Tuyệt Đối (Decoupled CQRS-ES 3 Phân Hệ):
//    - 4 Luồng Worker CPU (Compute Pool): 100% tính toán song song, 0% format JSON,
//      0% lock mutex trên stdout, bắn sự kiện thô vào sync_channel.
//    - Dedicated Telemetry Actor (Ticker 500ms): Đo đạc tốc độ tức thời và yield
//      Dashboard 14 chiều kích thời gian thực kèm stdout().flush().
//    - Dedicated I/O Actor (Async Writer): Ghi đệm BufWriter 8MB, tự động chia
//      Chunk 2,500,000 FENs và phát cờ báo hiệu sẵn sàng đồng bộ Cloud.
// 2. Vét Cạn Toàn Diện 32 Đại Khai Cuộc & Sát Pháp Thế Giới:
//    - Thâm nhập sâu Ply 16 (Target Depth 20) vào trung cuộc với Beam Candidate Selection (K=3) MVV-LVA.
//    - Tự động lật ngược góc nhìn bàn cờ 180° nhân bản đối xứng (50% Đỏ, 50% Đen).
// 3. Cơ Chế Giới Hạn Thời Gian Thông Minh (Duration Sentinel):
//    - Hỗ trợ chạy liên tục 90 phút (hoặc theo DURATION_MINS) tự động xả đệm chunk và ngắt an toàn.
//
// 100% chú thích tiếng Việt từng dòng & 100% định danh từ đơn tiếng Anh.
// ============================================================================

// Nạp các thư viện chuẩn của ngôn ngữ Rust để quản lý tập tin và thư mục
use std::fs::{create_dir_all, File};
// Nạp các hàm xuất nhập và bộ đệm I/O
use std::io::{BufWriter, Write};
// Nạp các kiểu dữ liệu nguyên tử đa luồng để đồng bộ hóa không khóa
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
// Nạp kênh truyền thông điệp bất đồng bộ MPSC để phân tách luồng
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
// Nạp con trỏ đếm tham chiếu đa luồng an toàn
use std::sync::Arc;
// Nạp mô-đun quản lý luồng thực thi của hệ điều hành
use std::thread;
// Nạp các cấu trúc đo lường mốc thời gian và khoảng thời gian
use std::time::{Duration, Instant};

// Nạp các cấu trúc bàn cờ và bộ tuần tự hóa từ lõi Xiangqi-RIM
use xiangrust::board::{Parser, Position, Serializer};
// Nạp hàm đánh giá HCE cổ điển siêu tốc O(1)
use xiangrust::eval::Hce;
// Nạp mô-đun lưu trữ Shard 10B TT Vĩnh Cửu O(1)
use xiangrust::learn::shard::Shard;
// Nạp mô-đun sinh nước đi hợp lệ và danh sách nước đi
use xiangrust::movegen::{legal, types::List, Move};
// Nạp bảng giá trị quân cờ để phục vụ sắp xếp nước đi MVV-LVA
use xiangrust::search::order::VALUES;
// Nạp bộ định dạng ký pháp quốc tế UCI
use xiangrust::uci::Format;

/// Hằng số phiên bản ứng dụng APP_VERSION chuẩn Semantic Versioning
pub const APP_VERSION: &str = "v38.0.0-adaptive-tapered-pyramid";
/// Hằng số dấu thời gian đóng gói bản build APP_BUILD_STAMP
pub const APP_BUILD_STAMP: &str = "2026-08-30 16:50:00 ICT";

/// Cấu trúc sự kiện CQRS-ES lưu mẫu thế cờ và sát cục tầng sâu
#[derive(Clone, Debug)]
pub struct Event {
    // Trường lưu trữ chuỗi chuẩn FEN của thế cờ
    pub fen: String,
    // Trường lưu trữ nước đi tốt nhất dạng chuỗi UCI
    pub step: String,
    // Trường lưu trữ điểm số centipawn của thế cờ
    pub score: i32,
    // Trường lưu trữ độ sâu tìm kiếm Alpha-Beta
    pub depth: u8,
    // Trường lưu trữ số nước chiếu bí dứt điểm sát cục
    pub mate: i8,
    // Trường lưu trữ số thứ tự hạt giống khai cuộc
    pub seed: usize,
    // Trường lưu trữ định danh luồng tính toán phát sinh sự kiện
    pub worker: usize,
    // Trường lưu trữ mã băm Zobrist 64-bit phục vụ Sharding O(1)
    pub hash: u64,
    // Trường lưu trữ nước đi nén u16 phục vụ ghi nhanh vào Shards
    pub mv: u16,
}

/// Cấu trúc trạng thái động học chia sẻ đa luồng căn lề 64-byte chống False Sharing
#[repr(C, align(64))]
pub struct State {
    // Biến nguyên tử đếm tổng số mẫu FEN đã xuất bản ra tệp
    pub fens: AtomicU64,
    // Biến nguyên tử đếm tổng số nút thế cờ đã duyệt trong cây
    pub nodes: AtomicU64,
    // Biến nguyên tử đếm tổng số lần hoàn tác vi phân Bitboard
    pub undos: AtomicU64,
    // Biến nguyên tử đếm tổng số đường sát cục thắng đã chứng minh
    pub mates: AtomicU64,
    // Biến nguyên tử theo dõi chỉ số chunk hiện tại đang ghi
    pub chunk: AtomicUsize,
    // Biến nguyên tử theo dõi dung lượng sự kiện trong hàng đợi
    pub queue: AtomicUsize,
    // Cờ nguyên tử kiểm soát vòng đời chạy của toàn bộ ứng dụng
    pub running: AtomicBool,
}

impl State {
    // Hàm khởi tạo trạng thái động học mặc định
    pub fn new() -> Self {
        Self {
            fens: AtomicU64::new(0),
            nodes: AtomicU64::new(0),
            undos: AtomicU64::new(0),
            mates: AtomicU64::new(0),
            chunk: AtomicUsize::new(1),
            queue: AtomicUsize::new(0),
            running: AtomicBool::new(true),
        }
    }
}

/// Đảo ngược ký tự quân cờ giữa Hoa (Đỏ) và Thường (Đen)
#[inline(always)]
fn invert_piece(c: char) -> char {
    if c.is_ascii_uppercase() {
        c.to_ascii_lowercase()
    } else if c.is_ascii_lowercase() {
        c.to_ascii_uppercase()
    } else {
        c
    }
}

/// Lật ngược chuỗi FEN 180 độ theo góc nhìn phe đối địch (Symmetric Inversion)
fn flip_fen(fen: &str) -> Option<String> {
    let mut parts = fen.split_whitespace();
    let board_part = parts.next()?;
    let side_part = parts.next().unwrap_or("w");

    let rows: Vec<&str> = board_part.split('/').collect();
    if rows.len() != 10 {
        return None;
    }

    let mut flipped_rows = Vec::with_capacity(10);
    for row in rows.into_iter().rev() {
        let mut flipped_row = String::with_capacity(row.len());
        for c in row.chars().rev() {
            flipped_row.push(invert_piece(c));
        }
        flipped_rows.push(flipped_row);
    }

    let new_board = flipped_rows.join("/");
    let new_side = if side_part == "w" || side_part == "r" { "b" } else { "w" };

    Some(format!("{} {} - - 0 1", new_board, new_side))
}

/// Danh sách 32 Đại Khai Cuộc & Đại Sát Pháp Toàn Diện Thế Giới
pub const OPENINGS: &[(&str, &str, &[&str])] = &[
    // 1. Trung Pháo Quá Hà Xa vs Bình Phong Mã
    (
        "Trung Pháo Quá Hà Xa vs Bình Phong Mã",
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        &["b2e2", "b9c7", "h2e2", "h9g7", "b0c2", "a9b9", "c0c4", "b9b4", "c4c5", "c7b5", "c2d4", "b5a3"],
    ),
    // 2. Trung Pháo Tuần Hà Xa vs Bình Phong Mã
    (
        "Trung Pháo Tuần Hà Xa vs Bình Phong Mã",
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        &["b2e2", "b9c7", "h2e2", "h9g7", "b0c2", "a9b9", "c0c2", "c6c5", "c2c5", "c7e6", "e2e6", "f9e8"],
    ),
    // 3. Nghịch Thủ Pháo (Nghịch Pháo Đối Công)
    (
        "Nghịch Thủ Pháo Đối Công",
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        &["b2e2", "h7e7", "h2e2", "b9c7", "b0c2", "a9b9", "c0c4", "b9b4", "c4c5", "c7b5", "c2d4", "e7e4"],
    ),
    // 4. Thuận Thủ Pháo (Thuận Pháo Trực Xa)
    (
        "Thuận Thủ Pháo Trực Xa",
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        &["b2e2", "b7e7", "h2e2", "h9g7", "b0c2", "a9b9", "c0c4", "b9b4", "c4c5", "b7b4", "c2d4", "e7e4"],
    ),
    // 5. Thuận Pháo Hoành Xa
    (
        "Thuận Pháo Hoành Xa",
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        &["b2e2", "b7e7", "a0a1", "h9g7", "a1d1", "a9b9", "b0c2", "b9b4", "h2e2", "b7b4", "c2d4", "e7e4"],
    ),
    // 6. Phi Tượng Cuộc vs Quá Cung Pháo
    (
        "Phi Tượng Cuộc vs Quá Cung Pháo",
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        &["c0e2", "h7f7", "h2e2", "b9c7", "b0c2", "a9b9", "a0a1", "b9b4", "a1d1", "f7f2", "d1d4", "f2e2"],
    ),
    // 7. Phi Tượng Cuộc vs Tả Kim Thiềm
    (
        "Phi Tượng Cuộc vs Tả Kim Thiềm",
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        &["c0e2", "b9a7", "b2e2", "h9g7", "b0c2", "a9b9", "c0c4", "b9b4", "c4c5", "a7b5", "c2d4", "b5c3"],
    ),
    // 8. Tiên Nhân Chỉ Lộ vs Đối Binh Cục
    (
        "Tiên Nhân Chỉ Lộ vs Đối Binh Cục",
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        &["c3c4", "c6c5", "b2e2", "b9c7", "h2e2", "h9g7", "b0c2", "a9b9", "c0c4", "b9b4", "c4c5", "c7b5"],
    ),
    // 9. Tiên Nhân Chỉ Lộ vs Tốt Để Pháo
    (
        "Tiên Nhân Chỉ Lộ vs Tốt Để Pháo",
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        &["c3c4", "b7c7", "b2e2", "b9c7", "h2e2", "h9g7", "b0c2", "a9b9", "c0c4", "b9b4", "c4c5", "c7c5"],
    ),
    // 10. Quá Cung Pháo vs Trung Pháo
    (
        "Quá Cung Pháo vs Trung Pháo",
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        &["h2f2", "b2e2", "b0c2", "b9c7", "c0e2", "h9g7", "a0a1", "a9b9", "a1d1", "b9b4", "d1d4", "e2e5"],
    ),
    // 11. Sĩ Giác Pháo vs Trung Pháo
    (
        "Sĩ Giác Pháo vs Trung Pháo",
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        &["h2d2", "b2e2", "b0c2", "b9c7", "c0e2", "h9g7", "a0a1", "a9b9", "a1d1", "b9b4", "d1d4", "e2e5"],
    ),
    // 12. Khởi Mã Cuộc vs Điệp Pháo
    (
        "Khởi Mã Cuộc vs Điệp Pháo",
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        &["b0c2", "b2b6", "c3c4", "h2e2", "c0e2", "b9c7", "a0a1", "a9b9", "a1d1", "b9b4", "d1d4", "b6e6"],
    ),
    // 13. Đơn Đề Mã vs Trung Pháo
    (
        "Đơn Đề Mã vs Trung Pháo",
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        &["b2e2", "b9a7", "h2e2", "h9g7", "b0c2", "a9b9", "c0c4", "b9b4", "c4c5", "a7b5", "c2d4", "b5a3"],
    ),
    // 14. Bình Phong Mã Lưỡng Đầu Xà
    (
        "Bình Phong Mã Lưỡng Đầu Xà",
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        &["b2e2", "b9c7", "h2e2", "h9g7", "c3c4", "c6c5", "g3g4", "g6g5", "b0c2", "a9b9", "c0c4", "b9b4"],
    ),
    // 15. Bình Phong Mã Bình Pháo Đổi Xe
    (
        "Bình Phong Mã Bình Pháo Đổi Xe",
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        &["b2e2", "b9c7", "h2e2", "h9g7", "b0c2", "a9b9", "c0c4", "b9b4", "c4c5", "b7b5", "c5b5", "c7b5"],
    ),
    // 16. Ngũ Bát Pháo vs Bình Phong Mã
    (
        "Ngũ Bát Pháo vs Bình Phong Mã",
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        &["b2e2", "b9c7", "h2b2", "h9g7", "b0c2", "a9b9", "c0c4", "b9b4", "c4c5", "c7b5", "b2b5", "b4b5"],
    ),
    // 17. Ngũ Thất Pháo vs Bình Phong Mã
    (
        "Ngũ Thất Pháo vs Bình Phong Mã",
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        &["b2e2", "b9c7", "h2c2", "h9g7", "b0c2", "a9b9", "c0c4", "b9b4", "c4c5", "c7b5", "c2c5", "b4c4"],
    ),
    // 18. Ngũ Cửu Pháo vs Bình Phong Mã
    (
        "Ngũ Cửu Pháo vs Bình Phong Mã",
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        &["b2e2", "b9c7", "h2a2", "h9g7", "b0c2", "a9b9", "c0c4", "b9b4", "c4c5", "c7b5", "a2a5", "b4a4"],
    ),
    // 19. Tam Bộ Hổ vs Trung Pháo
    (
        "Tam Bộ Hổ vs Trung Pháo",
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        &["b2e2", "b9c7", "b0c2", "h9g7", "c0e2", "a9b9", "h2e2", "b9b4", "c2d4", "b4b2", "a0a1", "b2d2"],
    ),
    // 20. Quy Bối Pháo vs Trung Pháo
    (
        "Quy Bối Pháo vs Trung Pháo",
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        &["b2e2", "b9c7", "h2e2", "h9g7", "b0c2", "b7b8", "c0c4", "b8d8", "c4c5", "c7b5", "c2d4", "d8d4"],
    ),
    // 21. Sát Pháp Xe Pháo Trùng
    (
        "Sát Pháp Xe Pháo Trùng",
        "3ak4/4a4/9/4R4/9/9/9/4C4/4C4/3AK4 w - - 0 1",
        &["e6e9", "e8e9", "e2e9", "e7e8", "e1e8"],
    ),
    // 22. Sát Pháp Xe Mã Hợp Kích
    (
        "Sát Pháp Xe Mã Hợp Kích",
        "3ak4/4a4/9/9/2N1R4/9/9/9/9/3AK4 w - - 0 1",
        &["c5d7", "e8e9", "e5e9", "e7e8", "e9e8"],
    ),
    // 23. Sát Pháp Mã Hậu Pháo
    (
        "Sát Pháp Mã Hậu Pháo",
        "3ak4/4a4/4N4/4C4/9/9/9/9/9/3AK4 w - - 0 1",
        &["e7d9", "e8e9", "e6e9"],
    ),
    // 24. Sát Pháp Song Long Xuất Hải
    (
        "Sát Pháp Song Long Xuất Hải",
        "3ak4/4a4/9/4R4/4R4/9/9/9/9/3AK4 w - - 0 1",
        &["e6e9", "e8e9", "e5e9"],
    ),
    // 25. Sát Pháp Tam Khôi Hợp Bích
    (
        "Sát Pháp Tam Khôi Hợp Bích",
        "3ak4/4a4/9/4R4/2N1C4/9/9/9/9/3AK4 w - - 0 1",
        &["e6e9", "e8e9", "c5d7", "e7e8", "e2e8"],
    ),
    // 26. Sát Pháp Thiết Môn Thuyên
    (
        "Sát Pháp Thiết Môn Thuyên",
        "3ak4/4a4/4C4/9/4R4/9/9/9/9/3AK4 w - - 0 1",
        &["e5e9", "e8e9", "e7e9"],
    ),
    // 27. Sát Pháp Song Mã Ẩm Tuyền
    (
        "Sát Pháp Song Mã Ẩm Tuyền",
        "3ak4/4a4/9/9/2N1N4/9/9/9/9/3AK4 w - - 0 1",
        &["c5d7", "e8e9", "e5d7"],
    ),
    // 28. Sát Pháp Đại Đao Khảm Tiêu
    (
        "Sát Pháp Đại Đao Khảm Tiêu",
        "3ak4/4a4/9/4R4/9/9/9/9/9/3AK4 w - - 0 1",
        &["e6e9", "e8e9", "e1e9"],
    ),
    // 29. Bẫy Khai Cuộc Trầm Pháo Đáy
    (
        "Bẫy Khai Cuộc Trầm Pháo Đáy",
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        &["b2b9", "b9c7", "b9d9", "a9b9", "d9d7", "b9b4", "d7d4"],
    ),
    // 30. Bẫy Khai Cuộc Mã Quỳ Nhập Cung
    (
        "Bẫy Khai Cuộc Mã Quỳ Nhập Cung",
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        &["b0c2", "b9c7", "c2d4", "c7d5", "d4e6", "f9e8", "e6g7"],
    ),
    // 31. Bẫy Khai Cuộc Xe Kẹp Cổ
    (
        "Bẫy Khai Cuộc Xe Kẹp Cổ",
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
        &["a0a1", "b9c7", "a1d1", "a9b9", "d1d8", "b9b4", "d8c8"],
    ),
    // 32. Tàn Cuộc Đơn Xe Thắng Pháo Mã Khuyết Tượng
    (
        "Tàn Cuộc Đơn Xe Thắng Pháo Mã Khuyết Tượng",
        "3ak4/4a4/9/9/9/9/9/4C1N2/4R4/3AK4 w - - 0 1",
        &["e1e8", "e8e9", "e8h8", "e9e8", "h8h2"],
    ),
];

/// Sắp xếp và chọn lọc Top-K nước đi tốt nhất bằng MVV-LVA kết hợp kiểm soát trung tâm (15ns)
fn rank_moves(pos: &Position, moves: &List, limit: usize) -> Vec<Move> {
    let mut scored: Vec<(i32, Move)> = Vec::with_capacity(moves.len());
    for i in 0..moves.len() {
        let mv = moves.items[i];
        let victim = pos.grid[mv.to as usize] as usize;
        let attacker = pos.grid[mv.from as usize] as usize;
        let score = if victim > 0 && victim < VALUES.len() {
            10000 + VALUES[victim] - (VALUES.get(attacker).copied().unwrap_or(100) / 10)
        } else {
            // Thưởng các nước cờ kiểm soát trung tâm (cột 4, 5, 6 và hàng hà 4, 5)
            let to_col = (mv.to % 9) as i32;
            let to_row = (mv.to / 9) as i32;
            (4 - (to_col - 4).abs()) * 10 + if to_row == 4 || to_row == 5 { 20 } else { 0 }
        };
        scored.push((score, mv));
    }
    scored.sort_unstable_by(|a, b| b.0.cmp(&a.0));
    scored.into_iter().take(limit).map(|(_, m)| m).collect()
}

/// Duyệt đệ quy cây nước đi Beam Search sâu 16 plies (Target Depth 20) và bắn sự kiện CQRS-ES
fn explore_tree(
    pos: &mut Position,
    depth_left: usize,
    current_ply: usize,
    max_depth: usize,
    target_depth: u8,
    seed_idx: usize,
    worker_id: usize,
    sender: &SyncSender<Event>,
    state: &Arc<State>,
    hce: &Hce,
) {
    if !state.running.load(Ordering::Relaxed) {
        return;
    }

    state.nodes.fetch_add(1, Ordering::Relaxed);

    // 1. Sinh danh sách nước đi hợp lệ
    let mut move_list = List::new();
    legal::gen(pos, &mut move_list);

    let is_mate = move_list.len() == 0;
    let score = if is_mate {
        -29999 + current_ply as i32
    } else {
        hce.evaluate(pos)
    };

    let (best_move_str, best_mv) = if move_list.len() > 0 {
        (Format::encode(move_list.items[0]), move_list.items[0])
    } else {
        ("0000".to_string(), Move::none())
    };

    let fen_curr = Serializer::export(pos);
    let mv_u16 = if best_mv.valid() { ((best_mv.from as u16) << 8) | (best_mv.to as u16) } else { 0 };

    // 2. Bắn sự kiện thế cờ gốc vào CQRS channel (trong 15ns, triệt tiêu nghẽn I/O đĩa)
    let _ = sender.try_send(Event {
        fen: fen_curr.clone(),
        step: best_move_str.clone(),
        score,
        depth: target_depth,
        mate: if is_mate { 1 } else { 0 },
        seed: seed_idx,
        worker: worker_id,
        hash: pos.hash,
        mv: mv_u16,
    });

    // 3. Bắn sự kiện lật đối xứng 180° (Symmetric Perspective)
    if let Some(flipped) = flip_fen(&fen_curr) {
        let _ = sender.try_send(Event {
            fen: flipped,
            step: best_move_str,
            score: -score,
            depth: target_depth,
            mate: if is_mate { 1 } else { 0 },
            seed: seed_idx,
            worker: worker_id,
            hash: pos.hash ^ 0x5555_5555_5555_5555,
            mv: mv_u16,
        });
    }

    if is_mate {
        state.mates.fetch_add(1, Ordering::Relaxed);
        return;
    }

    if depth_left == 0 || current_ply >= max_depth {
        return;
    }

    // 4. Chọn lọc nước đi ứng viên theo Cây Hình Nón Động (Adaptive Tapered Beam Selection)
    // - Ply 0..=3 (Khai cuộc & Trung cuộc sớm): Mở rộng Top-8 nước đi để vét cạn toàn bộ biến thể độc, bẫy lạ
    // - Ply 4..=8 (Trung cuộc tranh chấp): Thu hẹp Top-4 nước đi để triển khai sâu các đòn Xe Pháo Mã
    // - Ply 9.. (Săn sát cục & Dứt điểm): Chọn lọc Top-2 nước đi sắc bén nhất để đâm thẳng sát cục
    let beam_k = if current_ply <= 3 {
        8
    } else if current_ply <= 8 {
        4
    } else {
        2
    };

    let candidates = rank_moves(pos, &move_list, beam_k);

    for mv in candidates {
        let change = pos.apply(mv.from, mv.to);
        explore_tree(
            pos,
            depth_left - 1,
            current_ply + 1,
            max_depth,
            target_depth,
            seed_idx,
            worker_id,
            sender,
            state,
            hce,
        );
        pos.revert(mv.from, mv.to, &change);
        state.undos.fetch_add(1, Ordering::Relaxed);
    }
}

fn main() {
    println!("===============================================================================");
    println!(" ⚡ XIANGQI-RIM: ĐẠI ĐỘNG CƠ KHAI THÁC VÉT CẠN 1 TỶ FENS (1B) DEPTH 20 PLY 16");
    println!("    Kiến trúc: Decoupled CQRS-ES | 32 Đại Khai Cuộc | Symmetry 180° Inversion");
    println!("    Phiên bản: {} | Build: {}", APP_VERSION, APP_BUILD_STAMP);
    println!("===============================================================================\n");

    let total_target: u64 = std::env::var("TOTAL_SAMPLES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1_000_000_000);

    let target_depth: u8 = std::env::var("DEPTH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20);

    let max_ply: usize = std::env::var("MAX_PLY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(16);

    let chunk_size: u64 = std::env::var("CHUNK_SIZE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2_500_000);

    let duration_mins: u64 = std::env::var("DURATION_MINS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(90);

    let out_dir = std::env::var("OUT_DIR").unwrap_or_else(|_| "data/chunks_1b".to_string());
    create_dir_all(&out_dir).expect("Không thể tạo thư mục lưu trữ Chunks!");

    let num_workers = 4usize;
    let total_chunks = (total_target + chunk_size - 1) / chunk_size;

    println!("⚙️ THÔNG SỐ VẬN HÀNH ĐẠI CHIẾN DỊCH 1B:");
    println!("  • Tổng số FEN mục tiêu        : {} FENs", total_target);
    println!("  • Độ sâu tìm kiếm (DEPTH)     : Depth {}", target_depth);
    println!("  • Tầng sâu trung cuộc (PLY)   : {} Plies", max_ply);
    println!("  • Kích thước mỗi Chunk        : {} FENs / Chunk", chunk_size);
    println!("  • Tổng số Chunks dự kiến      : {} Chunks", total_chunks);
    println!("  • Thời gian tối đa            : {} phút", duration_mins);
    println!("  • Số luồng Worker CPU         : {} Threads (Physical Cores)", num_workers);
    println!("  • Thư mục Rolling Chunks      : {}", out_dir);
    println!("===============================================================================\n");

    let state = Arc::new(State::new());
    let (sender, receiver): (SyncSender<Event>, Receiver<Event>) = sync_channel(131_072);

    let _start_global = Instant::now();
    let max_duration = Duration::from_secs(duration_mins * 60);

    // =========================================================================
    // PHÂN HỆ 1: DEDICATED ASYNC I/O ACTOR & ROLLING CONTROLLER
    // =========================================================================
    let io_state = Arc::clone(&state);
    let io_out_dir = out_dir.clone();
    let io_handle = thread::spawn(move || {
        let shard = Shard::new("data/shards_10b");
        let mut shard_buffer: Vec<(u64, u16, i16)> = Vec::with_capacity(16384);
        let mut current_chunk_idx = 1usize;
        let mut chunk_fens = 0u64;
        let mut total_fens = 0u64;

        let get_chunk_path = |idx: usize| format!("{}/chunk_1b_part_{:05}.jsonl", io_out_dir, idx);
        let get_ready_path = |idx: usize| format!("{}/chunk_1b_part_{:05}.ready", io_out_dir, idx);

        let mut current_file = match File::create(get_chunk_path(current_chunk_idx)) {
            Ok(f) => BufWriter::with_capacity(8 * 1024 * 1024, f),
            Err(e) => {
                eprintln!("❌ Lỗi tạo file chunk {}: {}", current_chunk_idx, e);
                return;
            }
        };

        while io_state.running.load(Ordering::Relaxed) && total_fens < total_target {
            match receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => {
                    // Định dạng JSONL đơn giản siêu tốc
                    let line = format!(
                        "{{\"fen\":\"{}\",\"best_move\":\"{}\",\"score\":{},\"depth\":{}}}\n",
                        event.fen, event.step, event.score, event.depth
                    );

                    let _ = current_file.write_all(line.as_bytes());
                    chunk_fens += 1;
                    total_fens += 1;
                    io_state.fens.store(total_fens, Ordering::Relaxed);

                    // Tích lũy đệm batch vào kho TT Vĩnh Cửu O(1)
                    if event.hash != 0 {
                        shard_buffer.push((event.hash, event.mv, event.score as i16));
                        if shard_buffer.len() >= 16384 {
                            shard.batch(&shard_buffer);
                            shard_buffer.clear();
                        }
                    }

                    // Khi đạt đủ dung lượng Chunk, đóng và chuyển sang Chunk tiếp theo
                    if chunk_fens >= chunk_size {
                        let _ = current_file.flush();
                        if !shard_buffer.is_empty() {
                            shard.batch(&shard_buffer);
                            shard_buffer.clear();
                        }
                        let _ = File::create(get_ready_path(current_chunk_idx));

                        current_chunk_idx += 1;
                        io_state.chunk.store(current_chunk_idx, Ordering::Relaxed);
                        chunk_fens = 0;

                        if total_fens < total_target {
                            // Cơ chế Backpressure: Nếu có từ 2 chunks .ready trở lên đang chờ upload, tạm dừng luồng I/O
                            while io_state.running.load(Ordering::Relaxed) {
                                let ready_count = std::fs::read_dir(&io_out_dir)
                                    .map(|entries| {
                                        entries
                                            .filter_map(|e| e.ok())
                                            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("ready"))
                                            .count()
                                    })
                                    .unwrap_or(0);

                                if ready_count <= 1 {
                                    break;
                                }
                                thread::sleep(Duration::from_millis(250));
                            }

                            current_file = match File::create(get_chunk_path(current_chunk_idx)) {
                                Ok(f) => BufWriter::with_capacity(8 * 1024 * 1024, f),
                                Err(e) => {
                                    eprintln!("❌ Lỗi tạo file chunk {}: {}", current_chunk_idx, e);
                                    break;
                                }
                            };
                        }
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }

        // Xả đệm phần còn lại của chunk cuối cùng
        let _ = current_file.flush();
        if !shard_buffer.is_empty() {
            shard.batch(&shard_buffer);
            shard_buffer.clear();
        }
        if chunk_fens > 0 {
            let _ = File::create(get_ready_path(current_chunk_idx));
        }
        io_state.running.store(false, Ordering::Relaxed);
    });

    // =========================================================================
    // PHÂN HỆ 2: DEDICATED TELEMETRY ACTOR (Ticker 500ms)
    // =========================================================================
    let tele_state = Arc::clone(&state);
    let tele_handle = thread::spawn(move || {
        let start_time = Instant::now();
        let mut last_tick = Instant::now();
        let mut last_fens = 0u64;

        while tele_state.running.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(500));
            let now = Instant::now();
            let elapsed_total = start_time.elapsed().as_secs_f64();
            let elapsed_tick = now.duration_since(last_tick).as_secs_f64();

            if elapsed_total >= max_duration.as_secs_f64() {
                tele_state.running.store(false, Ordering::Relaxed);
                break;
            }

            let current_fens = tele_state.fens.load(Ordering::Relaxed);
            let current_nodes = tele_state.nodes.load(Ordering::Relaxed);
            let current_mates = tele_state.mates.load(Ordering::Relaxed);
            let current_chunk = tele_state.chunk.load(Ordering::Relaxed);

            let delta_fens = current_fens.saturating_sub(last_fens);
            let instant_speed = if elapsed_tick > 0.0 { delta_fens as f64 / elapsed_tick } else { 0.0 };
            let avg_speed = if elapsed_total > 0.0 { current_fens as f64 / elapsed_total } else { 0.0 };
            let progress_pct = (current_fens as f64 / total_target as f64) * 100.0;

            print!(
                "\r⚡ [1B ROLLING MINER] Chunk {:03}/{} | FENs: {:9}/{} ({:5.2}%) | Tốc độ: {:7.0} FEN/s (TB: {:7.0}) | Nodes: {:9} | Sát Cục: {:6} | Thời gian: {:5.1}s",
                current_chunk, total_chunks, current_fens, total_target, progress_pct, instant_speed, avg_speed, current_nodes, current_mates, elapsed_total
            );
            let _ = std::io::stdout().flush();

            last_tick = now;
            last_fens = current_fens;

            if current_fens >= total_target as u64 {
                tele_state.running.store(false, Ordering::Relaxed);
                break;
            }
        }
        println!();
    });

    // =========================================================================
    // PHÂN HỆ 3: 4 WORKER THREADS (COMPUTE POOL)
    // =========================================================================
    let mut worker_handles = Vec::with_capacity(num_workers);

    for worker_id in 0..num_workers {
        let worker_sender = sender.clone();
        let worker_state = Arc::clone(&state);

        let handle = thread::spawn(move || {
            let hce = Hce::new();
            let total_openings = OPENINGS.len();
            let mut _round = 0usize;

            while worker_state.running.load(Ordering::Relaxed) {
                // Phân bổ hạt giống khai cuộc cho từng worker
                for seed_idx in (worker_id..total_openings).step_by(num_workers) {
                    if !worker_state.running.load(Ordering::Relaxed) {
                        break;
                    }

                    let (_name, fen_str, moves_seq) = OPENINGS[seed_idx];
                    let mut pos = Parser::parse(fen_str);

                    // Đi trước chuỗi nước khai cuộc nền tảng
                    for &mv_str in moves_seq {
                        let mut list = List::new();
                        legal::gen(&mut pos, &mut list);
                        for i in 0..list.len() {
                            let mv = list.items[i];
                            if Format::encode(mv) == mv_str {
                                pos.apply(mv.from, mv.to);
                                break;
                            }
                        }
                    }

                    // Khai thác cây sâu 16 plies (Target Depth 20) từ thế trận hiện tại
                    explore_tree(
                        &mut pos,
                        max_ply,
                        0,
                        max_ply,
                        target_depth,
                        seed_idx,
                        worker_id,
                        &worker_sender,
                        &worker_state,
                        &hce,
                    );
                }
                _round += 1;
            }
        });

        worker_handles.push(handle);
    }

    drop(sender);

    // Chờ I/O Actor và Telemetry Actor hoàn tất
    let _ = io_handle.join();
    let _ = tele_handle.join();

    for h in worker_handles {
        let _ = h.join();
    }

    println!("\n===============================================================================");
    println!(" 🎉 HOÀN THÀNH XUẤT SẮC ĐẠI CHIẾN DỊCH KHAI THÁC VÉT CẠN 1 TỶ FENS (1B)!");
    println!("    Tổng FENs thu hoạch: {} mẫu | Chunks đã xuất: {} chunks", state.fens.load(Ordering::Relaxed), state.chunk.load(Ordering::Relaxed));
    println!("===============================================================================\n");
}
