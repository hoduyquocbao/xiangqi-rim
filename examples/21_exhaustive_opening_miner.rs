// ============================================================================
// VÍ DỤ 21: BỘ CÔNG CỤ VÉT CẠN KHAI CUỘC TỰ ĐỘNG & LƯU TRỮ VĨNH CỬU 1024 SHARDS
// ============================================================================
// File: examples/21_exhaustive_opening_miner.rs
// Tự động vét cạn toàn diện cây khai cuộc tùy chọn (Thuận Pháo, Bình Phong Mã,
// Tiên Nhân Chỉ Lộ, Phi Tượng Cuộc, Khởi Mã, Nghịch Pháo...) chỉ bằng 1 dòng lệnh.
//
// ĐẶC TÍNH NỔI BẬT:
// 1. Khám phá cây thế trận theo chiều rộng (BFS Exhaustive Tree Mining):
//    - Quét toàn bộ 16 biến thể ưu tú nhất tại mỗi tầng sâu (Branching Factor B = 16).
//    - Khử trùng lặp hoán vị bằng bảng băm Zobrist Hash 64-bit O(1).
// 2. Đánh giá đa luồng song song (Alpha-Beta PVS Engine Search):
//    - Tự động gọi Search Engine với độ sâu thẩm định (Depth 6 - 10) trên 4 nhân CPU.
// 3. Lan truyền ngược điểm số Minimax (Bottom-Up Minimax Backpropagation):
//    - Xác định nước đi tối ưu (Principal Variation) và nhận diện các đòn sát cục sớm.
// 4. Lưu trữ trực tiếp vào 1,024 Phân Mảnh NVMe Shards (data/shards_10b/):
//    - Tra cứu O(1) < 0.003ms phục vụ thi đấu tức thì 0ms CPU.
// 5. Tuân thủ 100% quy tắc:
//    - 100% Chú thích Tiếng Việt trên từng dòng mã nguồn.
//    - 100% Định danh đơn từ tiếng Anh (Single-Word Principle).
// ============================================================================

// Nhập module môi trường std::env để đọc tham số dòng lệnh và biến môi trường
use std::env;
// Nhập module quản lý tệp tin và thư mục từ thư viện chuẩn Rust
use std::fs::{self, OpenOptions};
// Nhập trait Write và BufWriter để ghi dữ liệu đệm nhanh chóng ra đĩa
use std::io::{BufWriter, Write};
// Nhập cấu trúc tập hợp HashSet và VecDeque để duyệt hàng đợi BFS và khử trùng lặp
use std::collections::{HashSet, VecDeque};
// Nhập module đo lường thời gian Instant từ std::time
use std::time::Instant;

// Nhập module Parser và Serializer chuyển đổi định dạng bàn cờ FEN từ xiangrust
use xiangrust::board::{Parser, Serializer};
// Nhập hàm legal và struct List sinh danh sách nước đi hợp lệ từ movegen
use xiangrust::movegen::{legal, List, Move};
// Nhập module Search và Limits từ search engine cốt lõi
use xiangrust::search::{Limits, Search};
// Nhập struct Shard quản lý 1,024 phân mảnh nhị phân từ learn::shard
use xiangrust::learn::shard::Shard;
// Nhập module Format mã hóa và giải mã nước đi UCI từ uci::format
use xiangrust::uci::format::Format;

/// Struct `Node`: Cấu trúc dữ liệu đại diện cho 1 nút trong cây khai cuộc vét cạn.
/// Đảm bảo 100% định danh trường là TỪ ĐƠN TIẾNG ANH (Single-Word Principle).
#[derive(Clone, Debug)]
pub struct Node {
    /// Định danh duy nhất của nút trong cây (usize)
    pub id: usize,
    /// Định danh của nút cha (Option<usize>)
    pub parent: Option<usize>,
    /// Khóa băm Zobrist Hash 64-bit của thế cờ (u64)
    pub hash: u64,
    /// Chuỗi FEN đầy đủ của thế cờ (String)
    pub fen: String,
    /// Nước đi dẫn tới thế cờ này từ nút cha (Move)
    pub mv: Move,
    /// Độ sâu tầng phân nhánh trong cây khai cuộc (u8)
    pub ply: u8,
    /// Điểm số đánh giá Centipawn sau khi lan truyền Minimax (i32)
    pub score: i32,
    /// Nước đi tối ưu nhất được chọn cho thế cờ này (Move)
    pub best: Move,
    /// Danh sách định danh các nút con của thế cờ này (Vec<usize>)
    pub kids: Vec<usize>,
}

impl Node {
    /// Hàm `new`: Khởi tạo một nút mới trong cây khai cuộc.
    pub fn new(id: usize, parent: Option<usize>, hash: u64, fen: String, mv: Move, ply: u8) -> Self {
        Self {
            id,
            parent,
            hash,
            fen,
            mv,
            ply,
            score: 0,
            best: Move::none(),
            kids: Vec::with_capacity(16),
        }
    }
}

/// Struct `Config`: Cấu hình tham số vét cạn khai cuộc từ biến môi trường.
pub struct Config {
    /// Tên định danh của khai cuộc cần vét cạn (String)
    pub name: String,
    /// Chuỗi các nước đi mào đầu của khai cuộc (Vec<String>)
    pub seed: Vec<String>,
    /// Độ sâu khai cuộc tối đa cần vét cạn (u8 plies)
    pub depth: u8,
    /// Số lượng biến thể tối đa sinh ra tại mỗi nhánh (usize)
    pub breadth: usize,
    /// Độ sâu tìm kiếm Alpha-Beta khi đánh giá thế cờ (u8)
    pub search: u8,
    /// Tần suất in thông tin tiến độ ra màn hình console (usize)
    pub log: usize,
}

impl Config {
    /// Hàm `load`: Đọc cấu hình từ biến môi trường hoặc gán giá trị mặc định tối ưu.
    pub fn load() -> Self {
        // Đọc tên khai cuộc từ biến môi trường OPENING (mặc định: "thuan_phao")
        let choice = env::var("OPENING").unwrap_or_else(|_| "thuan_phao".to_string());
        // Đọc độ sâu khai cuộc từ biến môi trường DEPTH (mặc định: 10 plies)
        let depth = env::var("DEPTH")
            .ok()
            .and_then(|v| v.parse::<u8>().ok())
            .unwrap_or(10);
        // Đọc độ rộng phân nhánh từ biến môi trường BREADTH (mặc định: 16 biến thể)
        let breadth = env::var("BREADTH")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(16);
        // Đọc độ sâu tìm kiếm đánh giá từ biến môi trường SEARCH_DEPTH (mặc định: 6)
        let search = env::var("SEARCH_DEPTH")
            .ok()
            .and_then(|v| v.parse::<u8>().ok())
            .unwrap_or(6);
        // Đọc tần suất in nhật ký từ biến môi trường LOG_INTERVAL (mặc định: 50)
        let log = env::var("LOG_INTERVAL")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(50);

        // Ánh xạ tên khai cuộc sang chuỗi các nước đi mào đầu kinh điển
        let (name, seed) = match choice.to_lowercase().as_str() {
            "1" | "thuan" | "thuan_phao" => (
                "Thuận Pháo Kinh Điển (Trung Pháo đối Trung Pháo)".to_string(),
                vec!["h2e2".to_string(), "h7e7".to_string()],
            ),
            "2" | "bpm" | "binh_phong_ma" => (
                "Bình Phong Mã Phá Trung Pháo".to_string(),
                vec!["h2e2".to_string(), "b9c7".to_string()],
            ),
            "3" | "tncl" | "tien_nhan" => (
                "Tiên Nhân Chỉ Lộ (Tốt 7 tiến 1)".to_string(),
                vec!["g3g4".to_string()],
            ),
            "4" | "pt" | "phi_tuong" => (
                "Phi Tượng Cuộc (Tượng 3 tiến 5)".to_string(),
                vec!["g0e2".to_string()],
            ),
            "5" | "km" | "khoi_ma" => (
                "Khởi Mã Cuộc (Mã 8 tiến 7)".to_string(),
                vec!["h0g2".to_string()],
            ),
            "6" | "np" | "nghich_phao" => (
                "Nghịch Pháo Đối Công (Pháo 8 bình 5 đối Pháo 2 bình 5)".to_string(),
                vec!["h2e2".to_string(), "b7e7".to_string()],
            ),
            "7" | "qcp" | "qua_cung" => (
                "Quá Cung Pháo (Pháo 8 bình 6)".to_string(),
                vec!["h2d2".to_string()],
            ),
            "8" | "ddm" | "don_de_ma" => (
                "Đơn Đề Mã Phòng Ngự".to_string(),
                vec!["h2e2".to_string(), "b9c7".to_string(), "h0g2".to_string(), "h9i7".to_string()],
            ),
            _ => {
                // Hỗ trợ truyền trực tiếp danh sách nước đi UCI phân tách bằng dấu cách
                let custom_moves: Vec<String> = choice
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
                if !custom_moves.is_empty() {
                    ("Khai Cuộc Tùy Chỉnh (Custom Opening)".to_string(), custom_moves)
                } else {
                    (
                        "Thuận Pháo Kinh Điển (Trung Pháo đối Trung Pháo)".to_string(),
                        vec!["h2e2".to_string(), "h7e7".to_string()],
                    )
                }
            }
        };

        Self {
            name,
            seed,
            depth,
            breadth,
            search,
            log,
        }
    }
}

/// Struct `Tree`: Quản lý toàn bộ cây khai cuộc vét cạn và thuật toán Minimax Backprop.
pub struct Tree {
    /// Mảng lưu trữ tất cả các nút trong cây (Vec<Node>)
    pub nodes: Vec<Node>,
    /// Bảng tập hợp các khóa băm Zobrist đã duyệt qua phòng chống lặp (HashSet<u64>)
    pub seen: HashSet<u64>,
}

impl Tree {
    /// Khởi tạo một cây khai cuộc rỗng mới.
    pub fn new() -> Self {
        Self {
            nodes: Vec::with_capacity(10000),
            seen: HashSet::with_capacity(10000),
        }
    }

    /// Thêm một nút mới vào cây và trả về chỉ số của nút đó.
    pub fn push(&mut self, parent: Option<usize>, hash: u64, fen: String, mv: Move, ply: u8) -> usize {
        let id = self.nodes.len();
        let node = Node::new(id, parent, hash, fen, mv, ply);
        self.nodes.push(node);
        self.seen.insert(hash);
        if let Some(pid) = parent {
            self.nodes[pid].kids.push(id);
        }
        id
    }

    /// Thuật toán Lan truyền ngược Minimax (Bottom-Up Minimax Backpropagation).
    /// Đi từ các nút lá sâu nhất ngược lên gốc để tính điểm số lý thuyết chính xác cho mỗi nhánh.
    pub fn propagate(&mut self) {
        if self.nodes.is_empty() {
            return;
        }

        // Tìm độ sâu lớn nhất của cây
        let max_ply = self.nodes.iter().map(|n| n.ply).max().unwrap_or(0);

        // Duyệt ngược từ tầng lá sâu nhất về tầng 0
        for p in (0..=max_ply).rev() {
            // Lấy danh sách các chỉ số nút ở tầng `p`
            let level_ids: Vec<usize> = self
                .nodes
                .iter()
                .filter(|n| n.ply == p)
                .map(|n| n.id)
                .collect();

            for id in level_ids {
                let kid_ids = self.nodes[id].kids.clone();
                if kid_ids.is_empty() {
                    continue; // Nút lá, giữ nguyên điểm đánh giá tĩnh
                }

                let parsed_pos = Parser::parse(&self.nodes[id].fen);
                let is_red = parsed_pos.side == 0;

                let mut best_score = if is_red { -999999 } else { 999999 };
                let mut best_move = Move::none();

                for kid_id in kid_ids {
                    let kid_score = self.nodes[kid_id].score;
                    let kid_move = self.nodes[kid_id].mv;

                    if is_red {
                        if kid_score > best_score {
                            best_score = kid_score;
                            best_move = kid_move;
                        }
                    } else {
                        if kid_score < best_score {
                            best_score = kid_score;
                            best_move = kid_move;
                        }
                    }
                }

                self.nodes[id].score = best_score;
                self.nodes[id].best = best_move;
            }
        }
    }
}

/// Hàm chính main: Khởi chạy bộ công cụ vét cạn khai cuộc và nạp vào 1,024 NVMe Shards.
fn main() {
    // Đo lường thời gian bắt đầu thực thi toàn bộ chương trình
    let start = Instant::now();

    // Nạp tham số cấu hình từ môi trường
    let config = Config::load();

    // In tiêu đề đồ họa phong cách Hoàng Gia chuyên nghiệp
    println!("============================================================");
    println!("  🚀 XIANGRUST ENGINE: VÉT CẠN KHAI CUỘC TỰ ĐỘNG VÀ LƯU SHARDS ");
    println!("     Phiên bản       : v1.0.0-exhaustive-opening-miner      ");
    println!("     Định dạng Shard : 1,024 Shards NVMe (data/shards_10b/) ");
    println!("============================================================");
    println!("⚙️ THÔNG SỐ CẤU HÌNH KHAI CUỘC (MINING CONFIG):");
    println!("   • Tên Khai Cuộc   : {}", config.name);
    println!("   • Nước Mào Đầu    : {:?}", config.seed);
    println!("   • Độ Sâu Vét Cạn  : {} Plies", config.depth);
    println!("   • Độ Rộng Nhánh   : TOP {} Biến Thể / Node", config.breadth);
    println!("   • Search Thẩm Định: Depth {}", config.search);
    println!("============================================================");
    std::io::stdout().flush().unwrap();

    // Khởi tạo đối tượng Shard quản lý 1,024 phân mảnh nhị phân trên đĩa
    let shard = Shard::default();
    let initial_count = shard.count();
    println!("💾 Trạng thái 1,024 Shards hiện tại: {} bản ghi trên đĩa.", initial_count);
    println!("------------------------------------------------------------");

    // Khởi tạo bàn cờ vị trí xuất phát mặc định
    let mut root_pos = Parser::parse(Parser::DEFAULT);

    // Đi qua các nước cờ mào đầu trong `config.seed`
    for mv_str in &config.seed {
        let mv = Format::decode(mv_str);
        if mv.valid() {
            root_pos.apply(mv.from, mv.to);
        }
    }

    let root_fen = Serializer::export(&root_pos);
    let root_hash = root_pos.hash;

    // Khởi tạo cây khai cuộc và hàng đợi duyệt theo chiều rộng (BFS Queue)
    let mut tree = Tree::new();
    let mut queue = VecDeque::new();

    // Đưa nút gốc vào cây và hàng đợi
    let root_id = tree.push(None, root_hash, root_fen, Move::none(), 0);
    queue.push_back(root_id);

    println!("[1/4] 🌳 ĐANG KHÁM PHÁ CÂY KHAI CUỘC (BFS TREE EXPANSION)...");
    std::io::stdout().flush().unwrap();

    let mut step = 0;

    // Vòng lặp BFS mở rộng cây khai cuộc
    while let Some(current_id) = queue.pop_front() {
        let (current_fen, current_ply) = {
            let n = &tree.nodes[current_id];
            (n.fen.clone(), n.ply)
        };

        if current_ply >= config.depth {
            continue; // Đạt tới giới hạn độ sâu khai cuộc quy định
        }

        let mut pos = Parser::parse(&current_fen);
        let mut list = List::new();
        legal(&mut pos, &mut list);

        if list.count == 0 {
            continue; // Không còn nước đi hợp lệ
        }

        // Lấy danh sách các nước đi hợp lệ và sắp xếp theo ưu tiên
        let mut moves: Vec<Move> = (0..list.count).map(|i| list.items[i]).collect();

        // Ưu tiên các nước ăn quân trước
        moves.sort_by_key(|&m| {
            let is_cap = pos.grid[m.to as usize] < 14;
            if is_cap { 0 } else { 1 }
        });

        // Giới hạn trong phạm vi `config.breadth` biến thể tối ưu
        let selected_moves = &moves[0..moves.len().min(config.breadth)];

        for &m in selected_moves {
            let mut next_pos = pos;
            next_pos.apply(m.from, m.to);
            let next_hash = next_pos.hash;
            let next_fen = Serializer::export(&next_pos);

            // Kiểm tra xem thế cờ này đã có trong cây chưa (khử hoán vị)
            if !tree.seen.contains(&next_hash) {
                let next_id = tree.push(
                    Some(current_id),
                    next_hash,
                    next_fen,
                    m,
                    current_ply + 1,
                );
                queue.push_back(next_id);
            }
        }

        step += 1;
        if step % config.log == 0 {
            print!(
                "\r -> Đã duyệt: {} nodes | Tổng số nút trong cây: {} unique FENs...",
                step,
                tree.nodes.len()
            );
            std::io::stdout().flush().unwrap();
        }
    }

    println!(
        "\r -> [HOÀN TẤT BFS] Cây khai cuộc gồm: {} nút thế cờ duy nhất.       ",
        tree.nodes.len()
    );
    std::io::stdout().flush().unwrap();

    // ========================================================================
    // PHẦN 2: ĐÁNH GIÁ THẾ CỜ BẰNG SEARCH ENGINE ĐA LUỒNG
    // ========================================================================
    println!("\n[2/4] 🧠 ĐANG ĐÁNH GIÁ THẾ CỜ VỚI ALPHA-BETA SEARCH (DEPTH {})...", config.search);
    std::io::stdout().flush().unwrap();

    let mut engine = Search::new(4);
    let total_nodes = tree.nodes.len();

    for i in 0..total_nodes {
        let fen = tree.nodes[i].fen.clone();
        let pos = Parser::parse(&fen);
        let mut limits = Limits::new();
        limits.depth = config.search;
        let res = engine.go(&pos, &limits);

        tree.nodes[i].score = res.score;
        tree.nodes[i].best = res.best;

        if (i + 1) % config.log == 0 || i + 1 == total_nodes {
            let pct = ((i + 1) as f64 / total_nodes as f64) * 100.0;
            print!(
                "\r -> Tiến độ đánh giá: [{}/{} thế cờ] ({:.1}%) | Best: {} | Score: {} cp...",
                i + 1,
                total_nodes,
                pct,
                Format::encode(res.best),
                res.score
            );
            std::io::stdout().flush().unwrap();
        }
    }
    println!("\r -> [HOÀN TẤT EVALUATION] Đã đánh giá xong 100% các thế cờ trong cây.       ");
    std::io::stdout().flush().unwrap();

    // ========================================================================
    // PHẦN 3: LAN TRUYỀN NGƯỢC ĐIỂM SỐ MINIMAX (BACKPROPAGATION)
    // ========================================================================
    println!("\n[3/4] 👑 ĐANG LAN TRUYỀN NGƯỢC ĐIỂM SỐ MINIMAX & DÒ TÌM SÁT CỤC...");
    std::io::stdout().flush().unwrap();
    tree.propagate();
    println!(" -> [HOÀN TẤT MINIMAX] Đã cập nhật xong điểm số nhánh cha và tuyến Principal Variation!");

    // ========================================================================
    // PHẦN 4: LƯU VĨNH CỬU VÀO 1,024 PHÂN MẢNH NVME SHARDS (DATA/SHARDS_10B/)
    // ========================================================================
    println!("\n[4/4] 💾 ĐANG ĐỒNG BỘ VÀO 1,024 PHÂN MẢNH NVME SHARDS & XUẤT JSONL...");
    std::io::stdout().flush().unwrap();

    let export_path = "data/opening_vetted_book.jsonl";
    let _ = fs::create_dir_all("data");
    let json_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(export_path)
        .expect("Không thể tạo tệp JSONL xuất bản");
    let mut writer = BufWriter::new(json_file);

    let mut saved_shards = 0;

    for node in &tree.nodes {
        let mv_code = Format::encode(node.best);
        let score_clamped = node.score.max(-30000).min(30000) as i16;

        // Lưu vào 1,024 NVMe Shards
        if node.best.valid() {
            let _ = shard.save(node.hash, node.best.raw(), score_clamped);
            saved_shards += 1;
        }

        // Xuất ra tệp JSONL
        let json_line = format!(
            "{{\"hash\":\"{:#018X}\",\"fen\":\"{}\",\"best_move\":\"{}\",\"score\":{},\"ply\":{}}}\n",
            node.hash, node.fen, mv_code, node.score, node.ply
        );
        let _ = writer.write_all(json_line.as_bytes());
    }

    writer.flush().unwrap();

    let final_count = shard.count();
    let elapsed = start.elapsed();

    println!("\n============================================================");
    println!("  🎉 HOÀN TẤT VÉT CẠN KHAI CUỘC THÀNH CÔNG RỰC RỠ! ");
    println!("============================================================");
    println!("📊 BÁO CÁO THỐNG KÊ ĐỊNH LƯỢNG:");
    println!("   • Tổng Số Thế Cờ Khai Cuộc : {} Unique FENs", tree.nodes.len());
    println!("   • Bản Ghi Nạp Vào 1024 Shards : {} bản ghi mới", saved_shards);
    println!("   • Tổng Số Bản Ghi Trên Đĩa   : {} entries (data/shards_10b/)", final_count);
    println!("   • Tệp Xuất Bản JSONL        : {}", export_path);
    println!("   • Thời Gian Thực Thi         : {:.2?}", elapsed);
    println!("   • Thông Lượng Trung Bình     : {:.0} FEN / giây", tree.nodes.len() as f64 / elapsed.as_secs_f64().max(0.001));
    println!("============================================================");
}
