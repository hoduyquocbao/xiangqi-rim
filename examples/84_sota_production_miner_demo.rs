// ============================================================================
// VÍ DỤ 84: ĐỘNG CƠ TỰ ĐẤU KHAI THÁC DỮ LIỆU THỰC TẾ DEPTH 12 (MINER V7.1.0 SOTA)
// ============================================================================
// `84_sota_production_miner_demo.rs` vận hành Miner class bản nâng cấp với
// Search Engine SEE Pruning thực ở Depth 12, xuất dữ liệu JSONL chuẩn hóa và
// in thông số Live Telemetry Stream Realtime (Speed mẫu/s, RAM RSS MB).
// ============================================================================

use xiangrust::selfplay::miner::{Config, Miner};

fn main() {
    println!("===============================================================================");
    println!("🏰 XIANGQI-RIM: SOTA PRODUCTION DATA MINER DEMO (DEPTH 12 SEE ENGINE)");
    println!("===============================================================================");

    let config = Config {
        games: 3,
        depth: 12,
        batch: 256,
        output: "data/depth12_mined_samples.jsonl".to_string(),
    };

    let miner = Miner::new(config);
    match miner.run() {
        Ok(samples) => {
            println!("\n✅ Khai thác thành công {} mẫu dữ liệu Depth 12 vào tệp data/depth12_mined_samples.jsonl!", samples);
        }
        Err(e) => {
            eprintln!("\n❌ Lỗi trong quá trình khai thác dữ liệu: {}", e);
        }
    }
}
