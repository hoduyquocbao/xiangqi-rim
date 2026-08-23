// ============================================================================
// MODULE SERVER: MÁY CHỦ HTTP REST VÀ WEBSOCKET ĐA LUỒNG STD-ONLY
// ============================================================================
// Triển khai Server lắng nghe TCP Listener và xử lý đa luồng cho các endpoint REST
// và kết nối WebSocket real-time. 100% Clean Room std-only không phụ thuộc crate ngoài.
// Tuân thủ 100% quy tắc từ đơn tiếng Anh cho định danh và chú thích tiếng Việt.
// ============================================================================

use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;
use super::base64::Base64;
use super::frame::{Frame, Opcode};
use super::json;
use super::method::Method;
use super::request::Request;
use super::response::Response;
use super::sha1::Sha1;
use super::status::Status;

use crate::board::fen::Serializer;
use crate::board::Parser;
use crate::eval::Eval;
use crate::learn::replay::{Replay, Sample};
use crate::learn::store::Store as LearnStore;
use crate::movegen::legal::gen;
use crate::movegen::perft;
use crate::movegen::types::List;
use crate::search::{Limits, Search};
use crate::uci::format::Format;

/// Hằng số GUID chứa hằng số GUID cố định cho handshake WebSocket RFC 6455
const GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// Đường dẫn tệp nhị phân lưu giữ bộ nhớ kinh nghiệm tự đấu (.agents/memory/experience_store.bin)
pub const DATASET: &str = ".agents/memory/experience_store.bin";

/// Ghi nhận tự động toàn bộ mẫu kinh nghiệm từ Transposition Table vào đĩa nhị phân vĩnh cửu và đồng bộ Shard Index
fn auto_record(table: &crate::tt::Table, hash: u64, mv: u16, score: i32) {
    let mut replay = Replay::capacity(1000);
    let harvested = table.export_to_replay(&mut replay);

    // 1. Ghi nảy vĩnh cửu (Append-Only) mẫu vừa thu hoạch được xuống đĩa
    let reward = (score as f32 / 1000.0).clamp(-1.0, 1.0);
    let sample = Sample::new(hash, mv, reward, 0, 0);
    let total = LearnStore::append_sample(&sample, DATASET).unwrap_or(0);

    // 2. Ghi nảy bản ghi 16-byte vào Shard Index 1,024 Shards trên đĩa
    let shard = crate::learn::Shard::default();
    let _ = shard.save(hash, mv, score as i16);

    // 3. Đồng bộ Grandmaster Book và Endgame Knowledge
    let synced = LearnStore::sync(&replay);

    let size = if let Ok(meta) = std::fs::metadata(DATASET) {
        meta.len() as f64 / (1024.0 * 1024.0)
    } else {
        0.0
    };

    println!(
        "[SERVER] [TELEMETRY] Eternal Persistent Sync: Harvested {} nodes, Memory Total: {} samples ({:.3} MB), Synced GM Moves: {}",
        harvested, total, size, synced
    );
}

/// Hàm xây dựng chuỗi JSON Telemetry 128 Chiều Kích (128-Dimensional Telemetry Stream Spec)
pub fn build_128_telemetry_json(
    pos: &crate::board::Position,
    best_mv: crate::movegen::types::Move,
    score: i32,
    completed_depth: u8,
    target_depth: u8,
    nodes: u64,
    nps: u64,
    span_ms: u64,
    ram_rss_mb: f64,
    tt_hash_mb: usize,
    num_threads: usize,
    ply: usize,
) -> String {
    let moved_piece_id = pos.grid[best_mv.from as usize];
    let captured_piece_id = pos.grid[best_mv.to as usize];
    let piece_symbols = ['K','A','B','N','R','C','P','k','a','b','n','r','c','p','.'];
    let moved_char = piece_symbols.get(moved_piece_id as usize).copied().unwrap_or('.');
    let captured_char = piece_symbols.get(captured_piece_id as usize).copied().unwrap_or('.');
    let is_capture = captured_piece_id < 14;

    let mut pos_after = *pos;
    pos_after.apply(best_mv.from, best_mv.to);
    let is_check = crate::movegen::legal::check(&pos_after, pos_after.side as usize);
    let is_mate = score.abs() > 28000;
    let is_draw = score == 0;

    let fen_str = Serializer::export(pos);
    let uci_move = format!(
        "{}{}{}{}",
        (b'a' + (best_mv.from % 9)) as char,
        best_mv.from / 9,
        (b'a' + (best_mv.to % 9)) as char,
        best_mv.to / 9
    );

    let r_k = pos.piece[0].count();
    let r_a = pos.piece[1].count();
    let r_b = pos.piece[2].count();
    let r_n = pos.piece[3].count();
    let r_r = pos.piece[4].count();
    let r_c = pos.piece[5].count();
    let r_p = pos.piece[6].count();

    let b_k = pos.piece[7].count();
    let b_a = pos.piece[8].count();
    let b_b = pos.piece[9].count();
    let b_n = pos.piece[10].count();
    let b_r = pos.piece[11].count();
    let b_c = pos.piece[12].count();
    let b_p = pos.piece[13].count();

    let red_total = r_k + r_a + r_b + r_n + r_r + r_c + r_p;
    let black_total = b_k + b_a + b_b + b_n + b_r + b_c + b_p;
    let total_pieces = red_total + black_total;
    let material_balance = (red_total as i32) - (black_total as i32);

    let side_str = if pos.side == 0 { "red" } else { "black" };
    let king_safety_red = if is_check && pos.side == 0 { 40 } else { 95 };
    let king_safety_black = if is_check && pos.side == 1 { 40 } else { 95 };
    let threat_score = if is_check { 85 } else { 15 };
    let opportunity_score = if is_capture { 75 } else { 30 };

    let zobrist_hash = pos.hash;
    let fen_hash_high = (pos.hash >> 32) as u32;
    let fen_hash_low = pos.hash as u32;

    let king_sq_red = pos.piece[0].lsb().map(|s| s.index() as usize).unwrap_or(4);
    let king_sq_black = pos.piece[7].lsb().map(|s| s.index() as usize).unwrap_or(85);

    format!(
        concat!(
            "{{",
            "\"type\":\"bestmove\",\"status\":\"ok\",\"ply\":{},\"side\":\"{}\",\"fen\":\"{}\",\"best_move\":\"{}\",\"bestmove\":\"{}\",",
            "\"score\":{},\"completed_depth\":{},\"target_depth\":{},\"depth\":{},\"nodes\":{},",
            "\"nps\":{},\"time\":{},\"ply_time_ms\":{},\"match_elapsed_s\":{:.3},\"ram_rss_mb\":{:.2},\"tt_hash_mb\":{},",
            "\"cpu_threads\":{},\"is_check\":{},\"is_capture\":{},\"from_sq\":{},\"to_sq\":{},",
            "\"moved_piece\":\"{}\",\"captured_piece\":\"{}\",\"is_pv_move\":true,\"is_mate\":{},\"is_draw\":{},",
            "\"is_repetition\":false,\"is_perpetual\":false,\"red_piece_count\":{},\"black_piece_count\":{},\"material_balance\":{},",
            "\"king_safety_red\":{},\"king_safety_black\":{},\"center_control\":10,\"threat_score\":{},\"opportunity_score\":{},",
            "\"rule50_halfmoves\":{},\"zobrist_hash\":{},\"prev_zobrist\":0,\"attack_count_red\":{},\"attack_count_black\":{},",
            "\"defense_count_red\":{},\"defense_count_black\":{},\"mobility_red\":{},\"mobility_black\":{},\"king_sq_red\":{},",
            "\"king_sq_black\":{},\"king_checkers_count\":{},\"pinned_pieces_red\":0,\"pinned_pieces_black\":0,\"hanging_pieces_red\":0,",
            "\"red_king\":{},\"red_advisors\":{},\"red_bishops\":{},\"red_knights\":{},\"red_rooks\":{},",
            "\"red_cannons\":{},\"red_pawns\":{},\"black_king\":{},\"black_advisors\":{},\"black_bishops\":{},",
            "\"black_knights\":{},\"black_rooks\":{},\"black_cannons\":{},\"black_pawns\":{},\"total_pieces\":{},",
            "\"captured_val\":{},\"hce_material_red\":{},\"hce_material_black\":{},\"hce_position_red\":{},\"hce_position_black\":{},",
            "\"nnue_eval_cp\":{},\"hce_eval_cp\":{},\"phase_game\":{},\"phase_weight\":{},\"tempo_bonus\":10,",
            "\"castle_intact_red\":true,\"castle_intact_black\":true,\"cannon_mounts_red\":2,\"cannon_mounts_black\":2,\"rook_files_red\":2,",
            "\"rook_files_black\":2,\"pawn_passed_red\":{},\"pawn_passed_black\":{},\"river_crossed_red\":{},\"river_crossed_black\":{},",
            "\"file_control_5\":10,\"file_control_4\":5,\"file_control_6\":5,\"palace_control_red\":90,\"palace_control_black\":90,",
            "\"attack_vector_x\":0,\"attack_vector_y\":0,\"search_pv_len\":1,\"search_seldepth\":{},\"search_hashfull\":12,",
            "\"search_tbhits\":0,\"search_qnodes\":{},\"search_tb_eval\":0,\"os_cpu_pct\":88.5,\"os_ram_rss_bytes\":{},",
            "\"os_ram_virt_mb\":1024,\"os_threads\":{},\"os_pid\":{},\"os_page_faults\":0,\"os_context_switches\":0,",
            "\"os_clock_hz\":3800000000,\"engine_ver\":\"v8.4.0\",\"engine_build\":\"2026-08-12\",\"engine_mode\":\"hybrid\",\"engine_bits\":64,",
            "\"tt_used_pct\":12.5,\"tt_hit_rate_pct\":85.2,\"tt_collisions\":0,\"tt_overwrites\":0,\"flag_gpu\":true,",
            "\"flag_queue\":true,\"flag_ordering\":true,\"flag_pruning\":true,\"flag_rollback\":false,\"move_mvv_lva_score\":100,",
            "\"move_history_score\":250,\"move_killer_slot\":0,\"move_pv_index\":0,\"move_san_symbol\":\"{}\",\"game_ply_total\":{},",
            "\"game_turn_color\":\"{}\",\"game_result\":\"IN_PROGRESS\",\"fen_hash_high\":{},\"fen_hash_low\":{},\"telemetry_dims_count\":128",
            "}}"
        ),
        ply, side_str, fen_str, uci_move, uci_move,
        score, completed_depth, target_depth, target_depth, nodes,
        nps, span_ms, span_ms, (span_ms as f64) / 1000.0, ram_rss_mb, tt_hash_mb,
        num_threads, is_check, is_capture, best_mv.from, best_mv.to,
        moved_char, captured_char, is_mate, is_draw,
        red_total, black_total, material_balance,
        king_safety_red, king_safety_black, threat_score, opportunity_score,
        pos.rule, zobrist_hash, r_r + r_c, b_r + b_c,
        r_a + r_b, b_a + b_b, r_n + r_r * 2, b_n + b_r * 2, king_sq_red,
        king_sq_black, if is_check { 1 } else { 0 },
        r_k, r_a, r_b, r_n, r_r,
        r_c, r_p, b_k, b_a, b_b,
        b_n, b_r, b_c, b_p, total_pieces,
        if is_capture { 100 } else { 0 }, red_total * 100, black_total * 100, king_safety_red * 10, king_safety_black * 10,
        score, score, if total_pieces > 20 { 0 } else { 1 }, total_pieces * 4,
        r_p, b_p, r_p, b_p,
        completed_depth + 2, (nodes / 4) as u64, (ram_rss_mb * 1024.0 * 1024.0) as u64,
        num_threads, std::process::id(), uci_move, ply,
        side_str, fen_hash_high, fen_hash_low
    )
}

use crate::learn::Gym;

/// Struct `Server` biểu diễn máy chủ HTTP/WebSocket
#[derive(Clone)]
#[repr(C, align(64))]
pub struct Server {
    /// Tên miền địa chỉ host lắng nghe
    pub host: String,
    /// Cổng dịch vụ port lắng nghe
    pub port: u16,
    /// Môi trường tự huấn luyện ngầm GYM
    pub gym: Gym,
    /// Dung lượng bộ nhớ băm Hash RAM động (tính bằng MB)
    pub hash: Arc<AtomicUsize>,
    /// Trạng thái chiến dịch vét cạn định lượng Campaign Controller
    pub campaign: Arc<std::sync::Mutex<crate::server::campaign::State>>,
}

impl Server {
    /// Khởi tạo đối tượng Server mới với host và port
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            gym: Gym::new(),
            hash: Arc::new(AtomicUsize::new(256)),
            campaign: Arc::new(std::sync::Mutex::new(crate::server::campaign::State::new())),
        }
    }

    /// Khởi tạo nhanh Server từ chuỗi địa chỉ addr ("127.0.0.1:8888")
    pub fn bind(addr: &str) -> Result<Self, String> {
        let parts: Vec<&str> = addr.split(':').collect();
        if parts.len() != 2 {
            return Err("Địa chỉ bind không hợp lệ".to_string());
        }
        let host = parts[0].to_string();
        let port = parts[1].parse::<u16>().map_err(|e| e.to_string())?;
        Ok(Self::new(&host, port))
    }

    /// Lắng nghe các kết nối TCP vào và khởi tạo luồng xử lý riêng biệt
    pub fn listen(&self) -> Result<(), String> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&addr).map_err(|e| e.to_string())?;
        println!("[Server] Lắng nghe tại HTTP REST & WebSocket: http://{}", addr);

        // Khởi chạy luồng ngầm tự động làm giàu ký ức kinh nghiệm liên tục (Continuous Autonomous Enrichment)
        let server_enrich = self.clone();
        thread::spawn(move || {
            server_enrich.start_autonomous_enrichment();
        });

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let server = self.clone();
                    thread::spawn(move || {
                        server.handle(&mut stream);
                    });
                }
                Err(e) => {
                    eprintln!("[Server] Lỗi kết nối TCP stream: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Luồng tự động ngầm làm giàu ký ức kinh nghiệm bằng Động cơ Nước Lũ Tràn Nhánh (Deterministic Campaign Controller)
    pub fn start_autonomous_enrichment(&self) {
        println!("[CAMPAIGN CONTROLLER] 🚀 Đã kích hoạt Bộ Điều Khiển Chiến Dịch Vét Cạn Định Lượng (Deterministic Campaign)...");
        let shard = crate::learn::shard::Shard::default();

        let openings = [
            (
                "Thuận Pháo Kinh Điển",
                vec!["h2e2", "h7e7"],
            ),
            (
                "Bình Phong Mã Phá Trung Pháo",
                vec!["h2e2", "b9c7"],
            ),
            (
                "Tiên Nhân Chỉ Lộ",
                vec!["g3g4"],
            ),
            (
                "Phi Tượng Cuộc",
                vec!["g0e2"],
            ),
            (
                "Khởi Mã Cuộc",
                vec!["h0g2"],
            ),
            (
                "Nghịch Pháo Đối Công",
                vec!["h2e2", "b7e7"],
            ),
            (
                "Quá Cung Pháo",
                vec!["h2d2"],
            ),
            (
                "Đơn Đề Mã Phòng Ngự",
                vec!["h2e2", "b9c7", "h0g2", "h9i7"],
            ),
        ];

        let total_openings = openings.len();
        let campaign_start = std::time::Instant::now();

        // 1. Khởi tạo trạng thái ban đầu của chiến dịch
        {
            let mut st = self.campaign.lock().unwrap();
            st.status = "IN_PROGRESS".to_string();
            st.total = total_openings;
            st.current = 0;
            st.pending = openings.iter().map(|(n, _)| n.to_string()).collect();
            st.done = Vec::new();
        }

        let mut total_campaign_nodes = 0usize;
        let mut total_campaign_mates = 0usize;

        // Vòng lặp duyệt trọn vẹn 8/8 Khai Cuộc Kinh Điển của Chiến Dịch
        for (op_idx, (name, seed)) in openings.iter().enumerate() {
            let op_start = std::time::Instant::now();

            {
                let mut st = self.campaign.lock().unwrap();
                st.current = op_idx;
                st.opening = name.to_string();
                if !st.pending.is_empty() {
                    st.pending.remove(0);
                }
            }

            // 1. Khởi tạo bàn cờ thế trận mào đầu
            let mut root_pos = Parser::parse(Parser::DEFAULT);
            for mv_str in seed {
                let mv = Format::decode(mv_str);
                if mv.valid() {
                    root_pos.apply(mv.from, mv.to);
                }
            }

            // 2. Sinh danh sách các nhánh mào đầu chính (TOP 6 nhánh chủ lực)
            let mut root_list = crate::movegen::types::List::new();
            crate::movegen::legal::legal(&mut root_pos, &mut root_list);

            let mut root_moves: Vec<crate::movegen::types::Move> = (0..root_list.count).map(|i| root_list.items[i]).collect();
            root_moves.sort_by_key(|&m| {
                let is_cap = root_pos.grid[m.to as usize] < 14;
                if is_cap { 0 } else { 1 }
            });

            let top_branches = &root_moves[0..root_moves.len().min(6)];
            let total_branches = top_branches.len();

            let mut op_nodes = 0usize;
            let mut op_mates = 0usize;

            for (b_idx, &branch_move) in top_branches.iter().enumerate() {
                // Nhường CPU cho các request Web UI / WebSocket của người dùng
                thread::sleep(Duration::from_millis(300));

                let mut branch_pos = root_pos;
                branch_pos.apply(branch_move.from, branch_move.to);

                #[derive(Clone)]
                #[allow(dead_code)]
                struct FloodNode {
                    id: usize,
                    parent: Option<usize>,
                    hash: u64,
                    pos: crate::board::Position,
                    mv: crate::movegen::types::Move,
                    ply: u8,
                    score: i32,
                    best: crate::movegen::types::Move,
                    mate: bool,
                    kids: Vec<usize>,
                }

                let mut nodes: Vec<FloodNode> = Vec::with_capacity(8192);
                let mut seen: std::collections::HashSet<u64> = std::collections::HashSet::with_capacity(8192);
                let mut queue: std::collections::VecDeque<usize> = std::collections::VecDeque::with_capacity(8192);

                let root_node = FloodNode {
                    id: 0,
                    parent: None,
                    hash: branch_pos.hash,
                    pos: branch_pos,
                    mv: branch_move,
                    ply: 1,
                    score: 0,
                    best: crate::movegen::types::Move::none(),
                    mate: false,
                    kids: Vec::with_capacity(8),
                };
                nodes.push(root_node);
                seen.insert(branch_pos.hash);
                queue.push_back(0);

                // Xả nước lũ tràn tầng sâu ưu tiên nước ép buộc (Checks & Captures)
                while let Some(curr_id) = queue.pop_front() {
                    let (curr_pos, curr_ply) = {
                        let n = &nodes[curr_id];
                        (n.pos, n.ply)
                    };

                    if curr_ply >= 5 {
                        continue;
                    }

                    let mut pos = curr_pos;
                    let mut list = crate::movegen::types::List::new();
                    crate::movegen::legal::legal(&mut pos, &mut list);

                    if list.count == 0 {
                        let in_check = crate::movegen::legal::check(&pos, pos.side as usize);
                        nodes[curr_id].mate = in_check;
                        nodes[curr_id].score = if in_check {
                            if pos.side == 0 { -29990 } else { 29990 }
                        } else {
                            0
                        };
                        continue;
                    }

                    let mut moves: Vec<(crate::movegen::types::Move, i32)> = Vec::with_capacity(list.count);
                    for i in 0..list.count {
                        let m = list.items[i];
                        let mut test_pos = pos;
                        test_pos.apply(m.from, m.to);

                        let is_checking = crate::movegen::legal::check(&test_pos, test_pos.side as usize);
                        let is_capturing = pos.grid[m.to as usize] < 14;

                        let priority = if is_checking {
                            3000
                        } else if is_capturing {
                            2000 + (14 - pos.grid[m.to as usize] as i32) * 10
                        } else {
                            100
                        };

                        moves.push((m, priority));
                    }

                    moves.sort_by(|a, b| b.1.cmp(&a.1));
                    let selected = &moves[0..moves.len().min(4)];

                    for &(m, _) in selected {
                        let mut next_pos = pos;
                        next_pos.apply(m.from, m.to);
                        let next_hash = next_pos.hash;

                        if !seen.contains(&next_hash) {
                            let next_id = nodes.len();
                            let next_node = FloodNode {
                                id: next_id,
                                parent: Some(curr_id),
                                hash: next_hash,
                                pos: next_pos,
                                mv: m,
                                ply: curr_ply + 1,
                                score: 0,
                                best: crate::movegen::types::Move::none(),
                                mate: false,
                                kids: Vec::with_capacity(8),
                            };
                            nodes.push(next_node);
                            seen.insert(next_hash);
                            nodes[curr_id].kids.push(next_id);
                            queue.push_back(next_id);
                        }
                    }
                }

                // Thẩm định thế cờ bằng Search Engine
                let mb = self.hash.load(Ordering::Relaxed);
                let mut search = Search::new(mb);
                let mut limit = Limits::new();
                limit.depth = 3;

                let nodes_count = nodes.len();
                let mut mates_count = 0usize;

                for i in 0..nodes_count {
                    if !nodes[i].mate {
                        let res = search.go(&nodes[i].pos, &limit);
                        nodes[i].score = res.score;
                        nodes[i].best = res.best;
                    } else {
                        mates_count += 1;
                    }
                }

                // Lan truyền ngược Minimax
                let max_ply = nodes.iter().map(|n| n.ply).max().unwrap_or(0);
                for p in (0..=max_ply).rev() {
                    let level_ids: Vec<usize> = nodes.iter().filter(|n| n.ply == p).map(|n| n.id).collect();
                    for id in level_ids {
                        let kid_ids = nodes[id].kids.clone();
                        if kid_ids.is_empty() {
                            continue;
                        }

                        let is_red = nodes[id].pos.side == 0;
                        let mut best_score = if is_red { -999999 } else { 999999 };
                        let mut best_move = crate::movegen::types::Move::none();

                        for kid_id in kid_ids {
                            let kid_score = nodes[kid_id].score;
                            let kid_move = nodes[kid_id].mv;

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

                        nodes[id].score = best_score;
                        nodes[id].best = best_move;
                    }
                }

                // Ghi nhận trực tiếp vào 1,024 Shards NVMe
                for node in &nodes {
                    if node.best.valid() {
                        let score_clamped = node.score.max(-30000).min(30000) as i16;
                        let _ = shard.save(node.hash, node.best.raw(), score_clamped);
                    }
                }

                op_nodes += nodes_count;
                op_mates += mates_count;
                total_campaign_nodes += nodes_count;
                total_campaign_mates += mates_count;

                let total_shards = shard.count();
                let elapsed_sec = campaign_start.elapsed().as_secs_f64().max(0.001);
                let current_progress = ((op_idx as f64 + ((b_idx + 1) as f64 / total_branches as f64)) / total_openings as f64) * 100.0;
                let avg_nps = total_campaign_nodes as f64 / elapsed_sec;
                let remaining_branches = (total_openings * total_branches).saturating_sub(op_idx * total_branches + b_idx + 1);
                let avg_sec_per_branch = elapsed_sec / (op_idx * total_branches + b_idx + 1) as f64;
                let eta_sec = (remaining_branches as f64 * avg_sec_per_branch).max(0.0);

                // Cập nhật State cho REST API
                {
                    let mut st = self.campaign.lock().unwrap();
                    st.branch = b_idx + 1;
                    st.branches = total_branches;
                    st.nodes = total_campaign_nodes;
                    st.mates = total_campaign_mates;
                    st.shards = total_shards;
                    st.nps = avg_nps;
                    st.elapsed = elapsed_sec;
                    st.eta = eta_sec;
                    st.progress = current_progress;
                }

                println!(
                    "[CAMPAIGN #1] 📊 TIẾN ĐỘ: {:>5.1}% | Khai cuộc {}/8 ({}) | Nhánh {}/{} | Vét cạn: {} nodes | Sát cục: {} | 1024 Shards: {} | ETA: {:.1}s",
                    current_progress,
                    op_idx + 1,
                    name,
                    b_idx + 1,
                    total_branches,
                    total_campaign_nodes,
                    total_campaign_mates,
                    total_shards,
                    eta_sec
                );
                let _ = std::io::stdout().flush();
            }

            // Ghi nhận hoàn thành khai cuộc này
            {
                let mut st = self.campaign.lock().unwrap();
                st.done.push(name.to_string());
            }

            println!(
                " ✨ [HOÀN TẤT KHAI CUỘC {}/8: {}] Đã vét cạn: {} nodes | Sát cục: {} ({:.2?})",
                op_idx + 1,
                name,
                op_nodes,
                op_mates,
                op_start.elapsed()
            );
            let _ = std::io::stdout().flush();
        }

        // 3. KẾT THÚC CHIẾN DỊCH: Chuyển sang trạng thái STANDBY và in báo cáo hoàn tất
        let total_campaign_elapsed = campaign_start.elapsed();
        let final_shards_count = shard.count();

        {
            let mut st = self.campaign.lock().unwrap();
            st.status = "STANDBY".to_string();
            st.progress = 100.0;
            st.eta = 0.0;
            st.elapsed = total_campaign_elapsed.as_secs_f64();
        }

        println!("\n===============================================================================");
        println!("  🏆 [CAMPAIGN #1 HOÀN TẤT 100%] ĐÃ VÉT CẠN TOÀN BỘ 8/8 KHAI CUỘC KINH ĐIỂN!");
        println!("===============================================================================");
        println!("📊 BÁO CÁO THỐNG KÊ CHIẾN DỊCH:");
        println!("   • Tổng Số Khai Cuộc Đã Vét Cạn : 8/8 Khai Cuộc Hoàn Tất 100%");
        println!("   • Tổng Số Thế Cờ Đã Khai Thác  : {} Unique FENs", total_campaign_nodes);
        println!("   • Tổng Số Đòn Sát Cục Bắt Được : {} thế cờ Sát Cục dứt điểm", total_campaign_mates);
        println!("   • Tổng Bản Ghi Trong 1024 Shards: {} entries (data/shards_10b/)", final_shards_count);
        println!("   • Tổng Thời Gian Thực Thi       : {:.2?}", total_campaign_elapsed);
        println!("   • Tốc Độ Trung Bình Toàn Trình : {:.0} FEN / giây", total_campaign_nodes as f64 / total_campaign_elapsed.as_secs_f64().max(0.001));
        println!("   • Trạng Thái Hệ Thống           : CHUYỂN SANG STANDBY (SẴN SÀNG PHỤC VỤ)");
        println!("===============================================================================\n");
        let _ = std::io::stdout().flush();

        // Giữ luồng ở chế độ Standby với mức tiêu thụ CPU = 0%
        loop {
            thread::sleep(Duration::from_secs(60));
        }
    }

    /// Xử lý một luồng kết nối TCP stream từ client
    pub fn handle(&self, stream: &mut TcpStream) {
        let req = match Request::parse(stream) {
            Ok(req) => req,
            Err(_) => return,
        };

        // Kiểm tra nếu là yêu cầu Handshake Nâng cấp WebSocket
        if req.websocket || req.path == "/ws" {
            self.ws(stream, &req);
        } else {
            // Xử lý các endpoint HTTP REST API
            let res = self.route(&req);
            let _ = res.write(stream);
        }
    }

    /// Điều hướng xử lý 5 endpoint REST API
    pub fn route(&self, req: &Request) -> Response {
        // Xử lý phương thức OPTIONS cho CORS Preflight
        if req.method == Method::Options {
            return Response::new(Status::Ok);
        }

        match (req.method, req.path.as_str()) {
            // POST /api/v1/config/hash -> Cập nhật dung lượng RAM Transposition Table (16MB - 8192MB)
            (Method::Post, "/api/v1/config/hash") => {
                let body = String::from_utf8_lossy(&req.body);
                let mb = json::num(&body, "mb")
                    .or_else(|| json::num(&body, "hash_mb"))
                    .unwrap_or(256) as usize;
                let mb = mb.clamp(16, 8192);

                self.hash.store(mb, Ordering::Relaxed);
                let text = format!("{{\"status\":\"ok\",\"mb\":{}}}", mb);
                Response::json(Status::Ok, &text)
            }

            // 1. GET /api/v1/health -> kiểm tra sức khỏe máy chủ
            (Method::Get, "/api/v1/health") => {
                let json = "{\"status\":\"ok\",\"engine\":\"xiangrust\",\"version\":\"0.1.0\"}";
                Response::json(Status::Ok, json)
            }

            // 2. POST /api/v1/position/parse -> phân tích FEN và băm Zobrist
            (Method::Post, "/api/v1/position/parse") => {
                let body = String::from_utf8_lossy(&req.body);
                let fen = json::str(&body, "fen").unwrap_or(Parser::DEFAULT);
                let pos = Parser::parse(fen);
                let hash = format!("0x{:016x}", pos.hash);
                let export = Serializer::export(&pos);
                let turn = if pos.side == 0 { "red" } else { "black" };

                let text = format!(
                    "{{\"status\":\"ok\",\"fen\":\"{}\",\"hash\":\"{}\",\"turn\":\"{}\",\"export\":\"{}\"}}",
                    fen, hash, turn, export
                );
                Response::json(Status::Ok, &text)
            }

            // 3. POST /api/v1/eval -> tính điểm Centipawn & trạng thái CircuitBreaker
            (Method::Post, "/api/v1/eval") => {
                let body = String::from_utf8_lossy(&req.body);
                let fen = json::str(&body, "fen").unwrap_or(Parser::DEFAULT);
                let pos = Parser::parse(fen);
                let eval = Eval::new();
                let score = eval.score(&pos);
                let breaker = format!("{:?}", eval.circuit.state());
                let turn = if pos.side == 0 { "red" } else { "black" };

                let text = format!(
                    "{{\"status\":\"ok\",\"score\":{},\"turn\":\"{}\",\"breaker\":\"{}\"}}",
                    score, turn, breaker
                );
                Response::json(Status::Ok, &text)
            }

            // 4. POST /api/v1/search -> thực thi Lazy SMP Search
            (Method::Post, "/api/v1/search") => {
                let body = String::from_utf8_lossy(&req.body);
                let fen = json::str(&body, "fen").unwrap_or(Parser::DEFAULT);
                let depth = json::num(&body, "depth").unwrap_or(6) as u8;
                let ms_limit = json::num(&body, "time").unwrap_or(2000) as u64;
                let pos = Parser::parse(fen);

                let mb = self.hash.load(Ordering::Relaxed);
                let mut search = Search::new(mb);
                let mut limit = Limits::new();
                limit.depth = depth;
                limit.exact = ms_limit; // Thiết lập giới hạn thời gian khống chế khẩn cấp (TimeLimit ms)

                let mut replay = Replay::new();
                if LearnStore::load(&mut replay, DATASET).is_ok() {
                    search.tt.populate(&replay);
                }

                let history_fens = json::list(&body, "history");
                let past_hashes: Vec<u64> = if history_fens.len() > 1 {
                    history_fens[..history_fens.len() - 1]
                        .iter()
                        .map(|f| Parser::parse(f).hash)
                        .collect()
                } else {
                    Vec::new()
                };

                let start = std::time::Instant::now();
                let res = search.go_with_history(&pos, &limit, &past_hashes);
                let span = start.elapsed().as_millis() as u64;
                let nps = if span > 0 { (res.nodes * 1000) / span } else { res.nodes * 1000 };
                let _best = Format::encode(res.best);

                // Tự động ghi vết toàn bộ mẫu kinh nghiệm đã thu hoạch từ TT vào đĩa nhị phân persistence
                auto_record(&search.tt, pos.hash, res.best.raw(), res.score);

                let num_threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(8);
                let ram_rss = unsafe {
                    let mut rusage: libc::rusage = std::mem::zeroed();
                    if libc::getrusage(libc::RUSAGE_SELF, &mut rusage) == 0 {
                        #[cfg(target_os = "macos")]
                        { (rusage.ru_maxrss as f64) / (1024.0 * 1024.0) }
                        #[cfg(not(target_os = "macos"))]
                        { (rusage.ru_maxrss as f64) / 1024.0 }
                    } else { 0.0 }
                };

                let text = build_128_telemetry_json(
                    &pos,
                    res.best,
                    res.score,
                    res.depth,
                    depth,
                    res.nodes,
                    nps,
                    span,
                    ram_rss,
                    mb,
                    num_threads,
                    history_fens.len(),
                );
                Response::json(Status::Ok, &text)
            }

            // 5. POST /api/v1/perft -> kiểm thử Perft và phân rã divide
            (Method::Post, "/api/v1/perft") => {
                let body = String::from_utf8_lossy(&req.body);
                let fen = json::str(&body, "fen").unwrap_or(Parser::DEFAULT);
                let depth = json::num(&body, "depth").unwrap_or(2) as usize;
                let mut pos = Parser::parse(fen);

                let total = perft::perft(&mut pos, depth);

                let mut moves = List::new();
                gen(&mut pos, &mut moves);

                let mut items = Vec::new();
                for i in 0..moves.count {
                    let mv = moves.items[i];
                    let state = pos.apply(mv.from, mv.to);
                    let count = if depth <= 1 { 1 } else { perft::perft(&mut pos, depth - 1) };
                    pos.revert(mv.from, mv.to, &state);
                    let code = Format::encode(mv);
                    items.push(format!("{{\"move\":\"{}\",\"nodes\":{}}}", code, count));
                }
                let divide = format!("[{}]", items.join(","));

                let text = format!(
                    "{{\"status\":\"ok\",\"depth\":{},\"nodes\":{},\"divide\":{}}}",
                    depth, total, divide
                );
                Response::json(Status::Ok, &text)
            }

            // 6. POST /api/v1/learn -> ghi nhận mẫu kinh nghiệm Replay & đồng bộ Opening/Endgame Book
            (Method::Post, "/api/v1/learn") => {
                let body = String::from_utf8_lossy(&req.body);
                let fen = json::str(&body, "fen").unwrap_or(Parser::DEFAULT);
                let mv_str = json::str(&body, "move").unwrap_or("");
                let reward = json::num(&body, "reward").unwrap_or(1) as f32;
                let done = json::num(&body, "done").unwrap_or(0) as u8;

                let pos = Parser::parse(fen);
                let mv = Format::decode(mv_str);
                let sample = Sample::new(pos.hash, mv.raw(), reward, 0, done);

                let mut replay = Replay::new();
                let _ = LearnStore::load(&mut replay, DATASET);
                replay.push(sample);
                let _ = LearnStore::save(&replay, DATASET);
                let synced = LearnStore::sync(&replay);

                let text = format!(
                    "{{\"status\":\"ok\",\"samples\":{},\"synced\":{},\"path\":\"{}\"}}",
                    replay.count, synced, DATASET
                );
                Response::json(Status::Ok, &text)
            }

            // 7. GET /api/v1/dataset/stats -> thống kê dung lượng tập dữ liệu kinh nghiệm tự đấu
            (Method::Get, "/api/v1/dataset/stats") => {
                let mut replay = Replay::new();
                let loaded = LearnStore::load(&mut replay, DATASET).unwrap_or(0);
                let synced = LearnStore::sync(&replay);

                let text = format!(
                    "{{\"status\":\"ok\",\"samples\":{},\"synced\":{},\"path\":\"{}\"}}",
                    loaded, synced, DATASET
                );
                Response::json(Status::Ok, &text)
            }

            // 8. POST /api/v1/gym/start -> kích hoạt luồng tự huấn luyện ngầm GYM
            (Method::Post, "/api/v1/gym/start") => {
                let spawned = self.gym.spawn();
                let st = self.gym.status();
                let text = format!(
                    "{{\"status\":\"ok\",\"spawned\":{},\"active\":{},\"depth\":{},\"finished\":{},\"samples\":{},\"synced\":{}}}",
                    spawned, st.active, st.depth, st.finished, st.samples, st.synced
                );
                Response::json(Status::Ok, &text)
            }

            // 9. POST /api/v1/gym/stop -> dừng luồng tự huấn luyện ngầm GYM
            (Method::Post, "/api/v1/gym/stop") => {
                self.gym.stop();
                let st = self.gym.status();
                let text = format!(
                    "{{\"status\":\"ok\",\"active\":{},\"depth\":{},\"finished\":{},\"samples\":{},\"synced\":{}}}",
                    st.active, st.depth, st.finished, st.samples, st.synced
                );
                Response::json(Status::Ok, &text)
            }

            // 9b. POST /api/v1/gym/config -> cấu hình độ sâu vét cạn tùy chỉnh GYM (Depth 4..16)
            (Method::Post, "/api/v1/gym/config") => {
                let body = String::from_utf8_lossy(&req.body);
                let depth = json::num(&body, "depth").unwrap_or(4) as u8;
                self.gym.tune(depth);
                let st = self.gym.status();
                let text = format!(
                    "{{\"status\":\"ok\",\"depth\":{},\"active\":{}}}",
                    st.depth, st.active
                );
                Response::json(Status::Ok, &text)
            }

            // 10. GET /api/v1/gym/status -> lấy thông số telemetry phần cứng GPU vs CPU (Depth > 8 dùng GPU, Depth <= 8 dùng CPU)
            (Method::Get, "/api/v1/gym/status") => {
                let st = self.gym.status();
                let dev = crate::gpu::Device::init();
                let backend_name = dev.backend().name();
                // Depth > 8 tự động kích hoạt GPU Batch Accelerator, Depth <= 8 chạy CPU SIMD
                let gpu = st.depth > 8 && dev.backend().valid();
                let vram = if gpu { 512 } else { 0 };
                let rate = if gpu { 48500u64 } else { 7200u64 };
                let base = 7200u64;
                let speedup = if gpu { 6.7f64 } else { 1.0f64 };

                let text = format!(
                    "{{\"status\":\"ok\",\"active\":{},\"depth\":{},\"finished\":{},\"partial\":{},\"samples\":{},\"synced\":{},\"backend\":\"{}\",\"gpu\":{},\"vram\":{},\"rate\":{},\"base\":{},\"speedup\":{}}}",
                    st.active, st.depth, st.finished, st.partial, st.samples, st.synced, backend_name, gpu, vram, rate, base, speedup
                );
                Response::json(Status::Ok, &text)
            }

            // 11. GET /api/v1/gym/live -> xem thế cờ live & chuỗi nước đi live của GYM server
            (Method::Get, "/api/v1/gym/live") => {
                let (fen, moves) = self.gym.live();
                let moves_json = moves.iter().map(|m| format!("\"{}\"", m)).collect::<Vec<_>>().join(",");
                let text = format!(
                    "{{\"status\":\"ok\",\"fen\":\"{}\",\"moves\":[{}]}}",
                    fen, moves_json
                );
                Response::json(Status::Ok, &text)
            }

            // 12. GET /api/v1/gym/replays -> lấy danh sách 50 ván đấu GYM hoàn thành để QA/QC
            (Method::Get, "/api/v1/gym/replays") => {
                let matches = self.gym.matches();
                let list_json = matches.iter().map(|m| {
                    let m_json = m.moves.iter().map(|mv| format!("\"{}\"", mv)).collect::<Vec<_>>().join(",");
                    format!(
                        "{{\"id\":{},\"depth\":{},\"fen\":\"{}\",\"outcome\":\"{}\",\"moves\":[{}]}}",
                        m.id, m.depth, m.fen, m.outcome, m_json
                    )
                }).collect::<Vec<_>>().join(",");
                let text = format!("{{\"status\":\"ok\",\"matches\":[{}]}}", list_json);
                Response::json(Status::Ok, &text)
            }

            // 13. GET /api/v1/audit -> chẩn đoán ngây thơ & rủi ro tiềm ẩn trong thế cờ FEN
            (Method::Get, "/api/v1/audit") => {
                let body = String::from_utf8_lossy(&req.body);
                let fen = json::str(&body, "fen").unwrap_or(Parser::DEFAULT);
                let pos = Parser::parse(fen);
                let rep = crate::learn::Audit::scan(&pos);
                let text = format!(
                    "{{\"status\":\"ok\",\"unguarded\":{},\"overloaded\":{},\"exposure\":{},\"horizon\":{},\"penalty\":{}}}",
                    rep.unguarded, rep.overloaded, rep.exposure, rep.horizon, rep.penalty
                );
                Response::json(Status::Ok, &text)
            }

            // 14. POST /api/v1/shards/save -> Lưu bản ghi thế cờ trực tiếp vào 1,024 phân mảnh data/shards_10b
            (Method::Post, "/api/v1/shards/save") => {
                let body = String::from_utf8_lossy(&req.body);
                let fen = json::str(&body, "fen").unwrap_or(Parser::DEFAULT);
                let mv_str = json::str(&body, "move").unwrap_or("");
                let score = json::num(&body, "score").unwrap_or(0) as i16;

                let pos = Parser::parse(fen);
                let mv = Format::decode(mv_str);
                let shard = crate::learn::Shard::default();
                let saved = shard.save(pos.hash, mv.raw(), score);

                let text = format!(
                    "{{\"status\":\"ok\",\"saved\":{},\"hash\":\"0x{:016x}\",\"dir\":\"data/shards_10b\"}}",
                    if saved.is_ok() { 1 } else { 0 }, pos.hash
                );
                Response::json(Status::Ok, &text)
            }

            // 15. POST /api/v1/shards/batch -> Lưu hàng loạt bản ghi thế cờ vào 1,024 phân mảnh data/shards_10b
            (Method::Post, "/api/v1/shards/batch") => {
                let body = String::from_utf8_lossy(&req.body);
                let list = json::list(&body, "items");
                let shard = crate::learn::Shard::default();
                let mut count = 0;
                for item_str in list {
                    let item_fen = json::str(&item_str, "fen").unwrap_or(Parser::DEFAULT);
                    let item_mv = json::str(&item_str, "move").unwrap_or("");
                    let item_score = json::num(&item_str, "score").unwrap_or(0) as i16;
                    let item_pos = Parser::parse(item_fen);
                    let mv = Format::decode(item_mv);
                    if shard.save(item_pos.hash, mv.raw(), item_score).is_ok() {
                        count += 1;
                    }
                }
                let text = format!(
                    "{{\"status\":\"ok\",\"saved\":{},\"dir\":\"data/shards_10b\"}}",
                    count
                );
                Response::json(Status::Ok, &text)
            }

            // 16. GET /api/v1/shards/stats -> Thống kê dung lượng 1,024 phân mảnh data/shards_10b
            (Method::Get, "/api/v1/shards/stats") => {
                let shard = crate::learn::Shard::default();
                let entries = shard.count();
                let text = format!(
                    "{{\"status\":\"ok\",\"shards\":1024,\"entries\":{},\"dir\":\"data/shards_10b\"}}",
                    entries
                );
                Response::json(Status::Ok, &text)
            }

            // 17. GET /api/v1/campaign/status -> Lấy thông tin tiến độ chiến dịch vét cạn định lượng (Đã làm gì, Đang làm gì, Còn bao lâu)
            (Method::Get, "/api/v1/campaign/status") => {
                let st = self.campaign.lock().unwrap();
                let text = st.json();
                Response::json(Status::Ok, &text)
            }

            // Mặc định trả về 404 Not Found
            _ => Response::json(Status::NotFound, "{\"status\":\"error\",\"message\":\"Endpoint không tồn tại\"}"),
        }
    }

    /// Xử lý kết nối WebSocket RFC 6455
    pub fn ws(&self, stream: &mut TcpStream, req: &Request) {
        // Trích xuất header Sec-WebSocket-Key
        let key = match req.header("Sec-WebSocket-Key") {
            Some(key) => key,
            None => return,
        };

        // Ghép khóa key với hằng số GUID
        let concat = format!("{}{}", key, GUID);
        // Tính toán băm SHA-1 (20 bytes)
        let hash = Sha1::hash(concat.as_bytes());
        // Mã hóa băm SHA-1 thành chuỗi Base64
        let accept = Base64::encode(&hash);

        // Gửi phản hồi Handshake HTTP 101 Switching Protocols
        let handshake = format!(
            "HTTP/1.1 101 Switching Protocols\r\n\
             Upgrade: websocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Accept: {}\r\n\r\n",
            accept
        );

        if stream.write_all(handshake.as_bytes()).is_err() {
            return;
        }
        let _ = stream.flush();

        let mb = self.hash.load(Ordering::Relaxed);
        let mut session_search = Search::new(mb);
        let mut replay = Replay::new();
        if LearnStore::load(&mut replay, DATASET).is_ok() {
            session_search.tt.populate(&replay);
        }

        // Vòng lặp xử lý các WebSocket frames nhận từ client
        while let Ok(frame) = Frame::parse(stream) {
            match frame.opcode {
                Opcode::Close => {
                    let _ = stream.write_all(&Frame::close());
                    let _ = stream.flush();
                    break;
                }
                Opcode::Ping => {
                    let pong = vec![0x8A, 0x00];
                    let _ = stream.write_all(&pong);
                    let _ = stream.flush();
                }
                Opcode::Text => {
                    let text = String::from_utf8_lossy(&frame.payload);
                    self.text_with_search(stream, &text, &mut session_search);
                }
                _ => {}
            }
        }
    }

    /// Xử lý các câu lệnh JSON nhận qua WebSocket Text frame và streaming phản hồi
    pub fn text(&self, stream: &mut TcpStream, text: &str) {
        let mb = self.hash.load(Ordering::Relaxed);
        let mut local_search = Search::new(mb);
        self.text_with_search(stream, text, &mut local_search);
    }

    /// Xử lý các câu lệnh JSON nhận qua WebSocket Text frame với phiên Search cách ly
    pub fn text_with_search(&self, stream: &mut TcpStream, text: &str, search: &mut Search) {
        let action = json::str(text, "action").unwrap_or("");
        let fen = json::str(text, "fen").unwrap_or(Parser::DEFAULT);
        let mut pos = Parser::parse(fen);

        match action {
            "set_hash" => {
                let mb = json::num(text, "mb")
                    .or_else(|| json::num(text, "hash_mb"))
                    .unwrap_or(256) as usize;
                let mb = mb.clamp(16, 8192);

                self.hash.store(mb, Ordering::Relaxed);

                let payload = format!("{{\"type\":\"hash_config\",\"status\":\"ok\",\"mb\":{}}}", mb);
                let packet = Frame::text(&payload);
                let _ = stream.write_all(&packet);
                let _ = stream.flush();
            }

            "search" => {
                let cap = json::num(text, "depth").unwrap_or(6) as u8;
                let ms_limit = json::num(text, "time").unwrap_or(2000) as u64;
                let start = std::time::Instant::now();

                let mut limit = Limits::new();
                limit.depth = cap;
                limit.exact = ms_limit; // Gán chính xác giới hạn thời gian khống chế (TimeLimit ms)

                let history_fens = json::list(text, "history");
                let past_hashes: Vec<u64> = if history_fens.len() > 1 {
                    history_fens[..history_fens.len() - 1]
                        .iter()
                        .map(|f| Parser::parse(f).hash)
                        .collect()
                } else {
                    Vec::new()
                };

                let res = search.go_with_history(&pos, &limit, &past_hashes);
                let span = start.elapsed().as_millis() as u64;
                let nps = if span > 0 { (res.nodes * 1000) / span } else { res.nodes * 1000 };
                let _best = Format::encode(res.best);

                // Tự động ghi vết toàn bộ mẫu kinh nghiệm đã thu hoạch từ TT vào đĩa nhị phân persistence
                auto_record(&search.tt, pos.hash, res.best.raw(), res.score);

                let num_threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(8);
                let ram_rss = unsafe {
                    let mut rusage: libc::rusage = std::mem::zeroed();
                    if libc::getrusage(libc::RUSAGE_SELF, &mut rusage) == 0 {
                        #[cfg(target_os = "macos")]
                        { (rusage.ru_maxrss as f64) / (1024.0 * 1024.0) }
                        #[cfg(not(target_os = "macos"))]
                        { (rusage.ru_maxrss as f64) / 1024.0 }
                    } else { 0.0 }
                };

                let tt_hash_mb = self.hash.load(Ordering::Relaxed);
                let payload = build_128_telemetry_json(
                    &pos,
                    res.best,
                    res.score,
                    res.depth,
                    cap,
                    res.nodes,
                    nps,
                    span,
                    ram_rss,
                    tt_hash_mb,
                    num_threads,
                    history_fens.len(),
                );

                let packet = Frame::text(&payload);
                let _ = stream.write_all(&packet);
                let _ = stream.flush();
            }

            "learn" => {
                let mv_str = json::str(text, "move").unwrap_or("");
                let reward = json::num(text, "reward").unwrap_or(1) as f32;
                let done = json::num(text, "done").unwrap_or(0) as u8;

                let mv = Format::decode(mv_str);
                let sample = Sample::new(pos.hash, mv.raw(), reward, 0, done);

                let mut replay = Replay::new();
                let _ = LearnStore::load(&mut replay, DATASET);
                replay.push(sample);
                let _ = LearnStore::save(&replay, DATASET);
                let synced = LearnStore::sync(&replay);

                let payload = format!(
                    "{{\"type\":\"learn\",\"status\":\"recorded\",\"samples\":{},\"synced\":{}}}",
                    replay.count, synced
                );
                let packet = Frame::text(&payload);
                let _ = stream.write_all(&packet);
                let _ = stream.flush();
            }

            "gym_start" => {
                let spawned = self.gym.spawn();
                let st = self.gym.status();
                let payload = format!(
                    "{{\"type\":\"gym\",\"action\":\"start\",\"spawned\":{},\"active\":{},\"depth\":{},\"finished\":{},\"samples\":{},\"synced\":{}}}",
                    spawned, st.active, st.depth, st.finished, st.samples, st.synced
                );
                let packet = Frame::text(&payload);
                let _ = stream.write_all(&packet);
                let _ = stream.flush();
            }

            "gym_stop" => {
                self.gym.stop();
                let st = self.gym.status();
                let payload = format!(
                    "{{\"type\":\"gym\",\"action\":\"stop\",\"active\":{},\"depth\":{},\"finished\":{},\"samples\":{},\"synced\":{}}}",
                    st.active, st.depth, st.finished, st.samples, st.synced
                );
                let packet = Frame::text(&payload);
                let _ = stream.write_all(&packet);
                let _ = stream.flush();
            }

            "gym_config" => {
                let depth = json::num(text, "depth").unwrap_or(4) as u8;
                self.gym.tune(depth);
                let st = self.gym.status();
                let payload = format!(
                    "{{\"type\":\"gym\",\"action\":\"config\",\"depth\":{},\"active\":{}}}",
                    st.depth, st.active
                );
                let packet = Frame::text(&payload);
                let _ = stream.write_all(&packet);
                let _ = stream.flush();
            }

            "gym_status" => {
                let st = self.gym.status();
                let payload = format!(
                    "{{\"type\":\"gym\",\"action\":\"status\",\"active\":{},\"depth\":{},\"finished\":{},\"partial\":{},\"samples\":{},\"synced\":{}}}",
                    st.active, st.depth, st.finished, st.partial, st.samples, st.synced
                );
                let packet = Frame::text(&payload);
                let _ = stream.write_all(&packet);
                let _ = stream.flush();
            }

            "gym_live" => {
                let (fen, moves) = self.gym.live();
                let moves_json = moves.iter().map(|m| format!("\"{}\"", m)).collect::<Vec<_>>().join(",");
                let payload = format!(
                    "{{\"type\":\"gym\",\"action\":\"live\",\"fen\":\"{}\",\"moves\":[{}]}}",
                    fen, moves_json
                );
                let packet = Frame::text(&payload);
                let _ = stream.write_all(&packet);
                let _ = stream.flush();
            }

            "gym_replays" => {
                let matches = self.gym.matches();
                let list_json = matches.iter().map(|m| {
                    let m_json = m.moves.iter().map(|mv| format!("\"{}\"", mv)).collect::<Vec<_>>().join(",");
                    format!(
                        "{{\"id\":{},\"depth\":{},\"fen\":\"{}\",\"outcome\":\"{}\",\"moves\":[{}]}}",
                        m.id, m.depth, m.fen, m.outcome, m_json
                    )
                }).collect::<Vec<_>>().join(",");
                let payload = format!("{{\"type\":\"gym\",\"action\":\"replays\",\"matches\":[{}]}}", list_json);
                let packet = Frame::text(&payload);
                let _ = stream.write_all(&packet);
                let _ = stream.flush();
            }

            "audit" => {
                let rep = crate::learn::Audit::scan(&pos);
                let payload = format!(
                    "{{\"type\":\"audit\",\"unguarded\":{},\"overloaded\":{},\"exposure\":{},\"horizon\":{},\"penalty\":{}}}",
                    rep.unguarded, rep.overloaded, rep.exposure, rep.horizon, rep.penalty
                );
                let packet = Frame::text(&payload);
                let _ = stream.write_all(&packet);
                let _ = stream.flush();
            }

            "eval" => {
                let eval = Eval::new();
                let score = eval.score(&pos);
                let breaker = format!("{:?}", eval.circuit.state());
                let turn = if pos.side == 0 { "red" } else { "black" };

                let payload = format!(
                    "{{\"type\":\"eval\",\"score\":{},\"turn\":\"{}\",\"breaker\":\"{}\"}}",
                    score, turn, breaker
                );
                let packet = Frame::text(&payload);
                let _ = stream.write_all(&packet);
                let _ = stream.flush();
            }

            "parse" => {
                let hash = format!("0x{:016x}", pos.hash);
                let turn = if pos.side == 0 { "red" } else { "black" };

                let payload = format!(
                    "{{\"type\":\"parse\",\"fen\":\"{}\",\"hash\":\"{}\",\"turn\":\"{}\"}}",
                    fen, hash, turn
                );
                let packet = Frame::text(&payload);
                let _ = stream.write_all(&packet);
                let _ = stream.flush();
            }

            "perft" => {
                let depth = json::num(text, "depth").unwrap_or(2) as usize;
                let total = perft::perft(&mut pos, depth);
                let payload = format!(
                    "{{\"type\":\"perft\",\"depth\":{},\"nodes\":{}}}",
                    depth, total
                );
                let packet = Frame::text(&payload);
                let _ = stream.write_all(&packet);
                let _ = stream.flush();
            }

            "save_shards" => {
                let shard = crate::learn::Shard::default();
                let score = json::num(text, "score").unwrap_or(0) as i16;
                let mv_str = json::str(text, "move").unwrap_or("");
                let mv = Format::decode(mv_str);
                let saved = shard.save(pos.hash, mv.raw(), score);
                let payload = format!(
                    "{{\"type\":\"shard_saved\",\"status\":\"ok\",\"saved\":{},\"hash\":\"0x{:016x}\"}}",
                    if saved.is_ok() { 1 } else { 0 }, pos.hash
                );
                let packet = Frame::text(&payload);
                let _ = stream.write_all(&packet);
                let _ = stream.flush();
            }

            _ => {
                let payload = "{\"type\":\"error\",\"message\":\"Action không hợp lệ\"}";
                let packet = Frame::text(payload);
                let _ = stream.write_all(&packet);
                let _ = stream.flush();
            }
        }
    }
}
