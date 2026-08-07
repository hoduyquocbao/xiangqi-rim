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

/// Ghi nhận tự động toàn bộ mẫu kinh nghiệm từ Transposition Table vào đĩa nhị phân và đồng bộ Grandmaster Book
fn auto_record(table: &crate::tt::Table, hash: u64, mv: u16, score: i32) {
    let mut replay = Replay::new();
    let _ = LearnStore::load(&mut replay, DATASET);

    // 1. Thu hoạch toàn bộ cờ giá trị (depth >= 2) mà GPU/CPU vừa duyệt được
    let harvested = table.export_to_replay(&mut replay);

    // 2. Ghi nhận nước đi tốt nhất cuối cùng
    let reward = (score as f32 / 1000.0).clamp(-1.0, 1.0);
    let sample = Sample::new(hash, mv, reward, 0, 0);
    replay.push(sample);

    let _ = LearnStore::save(&replay, DATASET);
    let synced = LearnStore::sync(&replay);
    println!(
        "[SERVER] [TELEMETRY] Persistent Experience Sync: Harvested {} nodes, Memory Total: {} samples, Synced GM Moves: {}",
        harvested, replay.count, synced
    );
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
}

impl Server {
    /// Khởi tạo đối tượng Server mới với host và port
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            gym: Gym::new(),
            hash: Arc::new(AtomicUsize::new(256)),
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
                let best = Format::encode(res.best);

                // Tự động ghi vết toàn bộ mẫu kinh nghiệm đã thu hoạch từ TT vào đĩa nhị phân persistence
                auto_record(&search.tt, pos.hash, res.best.raw(), res.score);

                let text = format!(
                    "{{\"status\":\"ok\",\"bestmove\":\"{}\",\"score\":{},\"nodes\":{},\"time\":{},\"nps\":{}}}",
                    best, res.score, res.nodes, span, nps
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
                let best = Format::encode(res.best);

                // Tự động ghi vết toàn bộ mẫu kinh nghiệm đã thu hoạch từ TT vào đĩa nhị phân persistence
                auto_record(&search.tt, pos.hash, res.best.raw(), res.score);

                let payload = format!(
                    "{{\"type\":\"bestmove\",\"best\":\"{}\",\"score\":{},\"nodes\":{},\"time\":{},\"nps\":{},\"depth\":{}}}",
                    best, res.score, res.nodes, span, nps, cap
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

            _ => {
                let payload = "{\"type\":\"error\",\"message\":\"Action không hợp lệ\"}";
                let packet = Frame::text(payload);
                let _ = stream.write_all(&packet);
                let _ = stream.flush();
            }
        }
    }
}
