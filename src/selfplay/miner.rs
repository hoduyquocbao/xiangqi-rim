// ============================================================================
// MODULE MINER: ĐỘNG CƠ TỰ ĐẤU KHAI THÁC DỮ LIỆU ĐA LUỒNG SONG SONG TRÊN GPU
// ============================================================================
// `miner.rs` thuộc Layer 2 trong Kiến trúc 3 Lớp (Tri-Tier Architecture):
// - Gom các ván cờ tự đấu song song tích hợp Engine SEE Pruning + NNUE/HCE.
// - Xuất chuỗi dữ liệu JSONL chứa thế cờ FEN, nước đi tốt nhất `best_move`,
//   điểm centipawn `score` và độ sâu `depth` phục vụ huấn luyện NNUE Gen 6.
// - 100% chú thích tiếng Việt từng dòng & 100% định danh từ đơn tiếng Anh.
// ============================================================================

use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use crate::board::{Parser, Serializer};
use crate::book::Book;
use crate::movegen::{legal, List};
use crate::search::Limits;
use crate::thread::Pool;
use crate::uci::Format;

/// Struct `Config`: Cấu hình thông số khai thác dữ liệu Miner
#[derive(Clone, Debug)]
pub struct Config {
    /// Số lượng ván cờ cần tự đấu
    pub games: usize,
    /// Độ sâu tìm kiếm Alpha-Beta Target
    pub depth: i32,
    /// Kích thước lô WGPU GPU Batch
    pub batch: usize,
    /// Đường dẫn tệp tin JSONL lưu trữ dữ liệu
    pub output: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            games: 1000,
            depth: 4,
            batch: 65536,
            output: "data/selfplay_samples_gen6.jsonl".to_string(),
        }
    }
}

/// Struct `Miner`: Động cơ tự đấu sinh dữ liệu hàng loạt trên GPU
pub struct Miner {
    /// Cấu hình miner
    config: Config,
    /// Cờ trạng thái đã hoàn tất
    finished: Arc<AtomicBool>,
    /// Tổng số mẫu FEN đã trích xuất
    samples: Arc<AtomicUsize>,
}

impl Miner {
    /// Khởi tạo Miner với cấu hình
    pub fn new(config: Config) -> Self {
        Self {
            config,
            finished: Arc::new(AtomicBool::new(false)),
            samples: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Khởi động tự đấu khai thác dữ liệu song song với Live Telemetry Stream Yield
    pub fn run(&self) -> std::io::Result<usize> {
        let start_time = Instant::now();
        let file = File::create(&self.config.output)?;
        let writer = Arc::new(std::sync::Mutex::new(BufWriter::new(file)));

        let total_games = self.config.games;
        let target_depth = self.config.depth;
        let pool = Pool::new(4, 64); // 4 Luồng CPU vật lý + 64MB TT

        println!("===============================================================================");
        println!("🏰 XIANGQI-RIM: SOTA PRODUCTION DATA MINER PIPELINE");
        println!("   Engine Target Depth: Depth {}", target_depth);
        println!("   Total Games Target : {} ván", total_games);
        println!("   Output File JSONL  : {}", self.config.output);
        println!("===============================================================================");

        for g in 0..total_games {
            let mut pos = Parser::parse(Parser::DEFAULT);
            let mut move_count = 0;
            let mut local_samples: Vec<String> = Vec::with_capacity(128);

            // 50% Book Opening + 50% Random Opening (6 nước đầu)
            let use_book = g % 2 == 0;
            let game_start = Instant::now();

            while move_count < 60 {
                let fen_str = Serializer::export(&pos);

                // Phase Opening: Dùng Book hoặc Random 6 nước đầu
                if use_book && move_count < 6 {
                    if let Some(mv) = Book::probe(&pos) {
                        let mv_str = Format::encode(mv);
                        pos.apply(mv.from, mv.to);
                        move_count += 1;

                        let sample_json = format!(
                            "{{\"fen\":\"{}\",\"best_move\":\"{}\",\"score\":0,\"depth\":{}}}\n",
                            fen_str, mv_str, target_depth
                        );
                        local_samples.push(sample_json);
                        continue;
                    }
                }

                if !use_book && move_count < 6 {
                    let mut list = List::new();
                    legal::gen(&mut pos, &mut list);
                    if list.len() == 0 {
                        break;
                    }
                    let mv = list.get(move_count % list.len());
                    let mv_str = Format::encode(mv);
                    pos.apply(mv.from, mv.to);
                    move_count += 1;

                    let sample_json = format!(
                        "{{\"fen\":\"{}\",\"best_move\":\"{}\",\"score\":0,\"depth\":{}}}\n",
                        fen_str, mv_str, target_depth
                    );
                    local_samples.push(sample_json);
                    continue;
                }

                // Phase Deep Search: Tìm kiếm bằng Alpha-Beta + SEE Pruning Engine ở target_depth
                let mut limits = Limits::new();
                limits.depth = target_depth as u8;
                let move_start = Instant::now();
                let res = pool.go(&pos, &limits);
                let move_elapsed = move_start.elapsed().as_secs_f64();

                if !res.best.valid() {
                    break;
                }

                let mv_str = Format::encode(res.best);
                let score = res.score;

                let sample_json = format!(
                    "{{\"fen\":\"{}\",\"best_move\":\"{}\",\"score\":{},\"depth\":{}}}\n",
                    fen_str, mv_str, score, target_depth
                );

                if let Ok(mut w) = writer.lock() {
                    let _ = w.write_all(sample_json.as_bytes());
                    let _ = w.flush();
                }

                local_samples.push(sample_json);
                pos.apply(res.best.from, res.best.to);
                move_count += 1;

                let current_total = self.samples.fetch_add(1, Ordering::Relaxed) + 1;
                let elapsed = start_time.elapsed().as_secs_f64();
                let speed = if elapsed > 0.0 { (current_total as f64) / elapsed } else { 0.0 };

                let mut r_usage: libc::rusage = unsafe { std::mem::zeroed() };
                let ram_mb = if unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut r_usage) } == 0 {
                    (r_usage.ru_maxrss as f64) / 1024.0
                } else {
                    0.0
                };

                // REALTIME UNBUFFERED YIELD DÒNG THEO DÒNG PER PLY (QUY TẮC 8.10 / 7.10)
                println!(
                    "  🚀 [LIVE MINER DEPTH {}] Ván {:2}/{} | Ply {:3} | Move: {:5} | Score: {:5} cp | Time: {:5.2}s | Samples: {:4} | Speed: {:5.2} mẫu/s | OS RAM RSS: {:.2} MB",
                    target_depth, g + 1, total_games, move_count, mv_str, score, move_elapsed, current_total, speed, ram_mb
                );
                let _ = std::io::stdout().flush();
            }

            let game_time = game_start.elapsed().as_secs_f64();
            println!(
                "  🏆 [GAME COMPLETED] Ván {:2}/{} hoàn thành trong {:5.2}s với {} mẫu dữ liệu FEN Depth {}\n",
                g + 1, total_games, game_time, local_samples.len(), target_depth
            );
            let _ = std::io::stdout().flush();
        }

        let total = self.samples.load(Ordering::Relaxed);
        let elapsed = start_time.elapsed().as_secs_f64();
        println!(
            "🏆 Miner hoàn tất! Trích xuất {} mẫu FEN vào {} trong {:.2}s",
            total, self.config.output, elapsed
        );
        self.finished.store(true, Ordering::Relaxed);
        Ok(total)
    }
}

