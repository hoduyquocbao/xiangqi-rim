// ============================================================================
// VÍ DỤ 91: XUẤT BẢN TRỌNG SỐ LƯỢNG TỬ HÓA XRNN V1 (NNUE QUANTIZER EXPORTER)
// ============================================================================
// Tệp thực thi nạp checkpoint f32 và xuất bản tệp nhị phân XRNN v1 chuẩn xác.
// 100% chú thích tiếng Việt & 100% định danh từ đơn tiếng Anh.
// ============================================================================

use xiangrust::learn::nnue::Network;

fn main() {
    println!("============================================================");
    println!(" 🚀 XUẤT BẢN TRỌNG SỐ LƯỢNG TỬ HÓA XRNN V1 (NNUE EXPORTER)");
    println!("============================================================");

    let checkpoint_path = "data/nnue_checkpoint.bin";
    let target_weights = "data/nnue_weights.bin";
    let target_weights_gpu = "data/nnue_weights_gpu.bin";

    if !std::path::Path::new(checkpoint_path).exists() {
        eprintln!("❌ Không tìm thấy tệp checkpoint: {}", checkpoint_path);
        return;
    }

    let mut net = Network::new();
    println!("📦 Đang nạp checkpoint f32 từ '{}'...", checkpoint_path);
    if let Err(e) = net.load(checkpoint_path) {
        eprintln!("❌ Lỗi nạp checkpoint: {}", e);
        return;
    }
    println!("✅ Đã nạp checkpoint f32 thành công!");

    println!("⚡ Đang lượng tử hóa và xuất bản sang '{}'...", target_weights);
    if let Err(e) = net.quantize(target_weights) {
        eprintln!("❌ Lỗi lượng tử hóa: {}", e);
        return;
    }
    println!("✅ Đã xuất bản '{}' thành công!", target_weights);

    println!("⚡ Đang sao chép sang '{}'...", target_weights_gpu);
    if let Err(e) = net.quantize(target_weights_gpu) {
        eprintln!("❌ Lỗi lượng tử hóa GPU: {}", e);
        return;
    }
    println!("✅ Đã xuất bản '{}' thành công!", target_weights_gpu);

    println!("============================================================");
    println!(" 🏆 HOÀN TẤT XUẤT BẢN TRỌNG SỐ LƯỢNG TỬ HÓA 100%!");
    println!("============================================================");
}
