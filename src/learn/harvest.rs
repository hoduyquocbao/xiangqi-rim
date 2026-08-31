// ============================================================================
// MODULE LEARN HARVEST: BỘ THU HOẠCH & BẢO TOÀN TRI THỨC TỰ ĐỘNG (ACTIVE HARVESTER)
// ============================================================================
// `Harvest` chịu trách nhiệm:
// 1. Tự động thu hoạch 100% nước đi, thế cờ FEN, điểm số, và kết quả thắng/thua/hòa
//    từ mọi trận đấu đối kháng (Xiangqi-RIM vs Pikafish, Self-Play, Arena).
// 2. Lưu trữ song song 3 tầng:
//    - Tầng 1: Tệp nhị phân nén bitwise 64-byte XRKB (`data/harvest_knowledge.xrk`) tốc độ tối đa > 20M/s.
//    - Tầng 2: 1,024 Shards NVMe nhị phân O(1) (`data/shards_10b/`) phục vụ huấn luyện NNUE.
//    - Tầng 3: Tệp luồng JSONL streaming (`data/depth20_harvest_knowledge.jsonl`) cho DeepSeek-R1 / Training.
// 3. Triệt tiêu 100% lãng phí tài nguyên tính toán và thời gian của Developer.
// ============================================================================

use std::fs::{self, OpenOptions};
use std::io::{BufWriter, Write};
use crate::board::Position;
use crate::learn::archive::Archive;
use crate::learn::frame::Frame;
use crate::learn::shard::Shard;
use crate::movegen::types::Move;

/// Struct `Harvest` quản lý việc thu hoạch và lưu trữ vĩnh cửu tri thức đối kháng
#[repr(C, align(64))]
pub struct Harvest {
    /// Đường dẫn tệp nhị phân nén bitwise XRKB
    pub bin: String,
    /// Đường dẫn tệp JSONL xuất bản tri thức
    pub jsonl: String,
    /// Bộ quản lý 1,024 phân mảnh Shards NVMe
    pub shard: Shard,
    /// Bộ đệm các khung thế cờ `Frame` trong ván đấu hiện tại
    pub buffer: Vec<Frame>,
    /// Mảng đệm căn lề 64-byte loại bỏ False Sharing
    pub pad: [u8; 8],
}

impl Harvest {
    /// Khởi tạo một đối tượng thu hoạch tri thức mới
    pub fn new(bin_path: &str, jsonl_path: &str, shard_root: &str) -> Self {
        if let Some(parent) = std::path::Path::new(bin_path).parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Some(parent) = std::path::Path::new(jsonl_path).parent() {
            let _ = fs::create_dir_all(parent);
        }
        let shard = Shard::new(shard_root);
        Self {
            bin: bin_path.to_string(),
            jsonl: jsonl_path.to_string(),
            shard,
            buffer: Vec::with_capacity(256),
            pad: [0u8; 8],
        }
    }

    /// Khởi tạo đối tượng thu hoạch mặc định cho thư mục data
    pub fn default() -> Self {
        Self::new(
            "data/harvest_knowledge.xrk",
            "data/depth20_harvest_knowledge.jsonl",
            "data/shards_10b",
        )
    }

    /// Ghi nhận 1 nước đi vừa được đánh giá vào bộ đệm của ván cờ
    pub fn push(
        &mut self,
        pos: &Position,
        mv: Move,
        score: i32,
        depth: u8,
        actor: &str,
        ply: usize,
    ) {
        let actor_code = match actor {
            "RIM" => 0u8,
            "PIKA" => 1u8,
            _ => 2u8,
        };
        let frame = Frame::pack(pos, mv, score, depth, actor_code, ply, 0);
        self.buffer.push(frame);
    }

    /// Lưu toàn bộ tri thức của ván cờ xuống ổ đĩa cùng kết quả chung cuộc (Flush Game Knowledge)
    pub fn flush(&mut self, result: &str) -> usize {
        let count = self.buffer.len();
        if count == 0 {
            return 0;
        }

        let outcome_code = if result.contains("Xiangqi-RIM Thắng") || result.contains("Red Win") || result.contains("Đỏ Thắng") {
            1u8
        } else if result.contains("Pikafish Thắng") || result.contains("Black Win") || result.contains("Đen Thắng") {
            2u8
        } else if result.contains("Hòa") || result.contains("Draw") {
            3u8
        } else {
            0u8
        };

        // Gán mã kết quả ván cờ cho tất cả các frames trong ván
        for frame in &mut self.buffer {
            frame.outcome = outcome_code;
        }

        // 1. Ghi tệp nhị phân nén bitwise 64-byte XRKB tốc độ tối đa
        let _ = Archive::append(&self.bin, &self.buffer);

        // 2. Chuyển đổi sang định dạng Shard NVMe nhị phân O(1)
        let mut shard_items: Vec<(u64, u16, i16)> = Vec::with_capacity(count);
        for item in &self.buffer {
            shard_items.push((item.hash, item.mv, item.score));
        }
        self.shard.batch(&shard_items);

        // 3. Ghi nối tiếp vào tệp JSONL streaming cho các công cụ AI / LLM
        if let Ok(file) = OpenOptions::new().create(true).append(true).open(&self.jsonl) {
            let mut writer = BufWriter::with_capacity(64 * 1024, file);
            for frame in &self.buffer {
                let json_line = format!(
                    "{{\"fen\":\"{}\",\"best_move\":\"{}\",\"score\":{},\"depth\":{},\"side\":{},\"ply\":{},\"actor\":\"{}\",\"result\":\"{}\"}}\n",
                    frame.fen(), frame.uci(), frame.score, frame.depth, frame.side, frame.ply, frame.actor_name(), result
                );
                let _ = writer.write_all(json_line.as_bytes());
            }
            let _ = writer.flush();
        }

        // Xóa bộ đệm sau khi đã bảo tồn xong
        self.buffer.clear();
        count
    }
}
