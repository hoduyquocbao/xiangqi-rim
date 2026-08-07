// ============================================================================
// VÍ DỤ MẪU 15: HƯỚNG DẪN VẬN HÀNH GYM HUẤN LUYỆN NGẦM TỰ ĐẤU LŨY TIẾN ĐỘ SÂU
// ============================================================================
// Tệp ví dụ minh họa cách vận hành Môi trường GYM Tự huấn luyện ngầm tốc độ cao
// (Progressive Depth Curriculum Gym: Depth 4 -> 5 -> ... -> 12).
// 100% Clean Room std-only, 100% chú thích tiếng Việt & từ đơn tiếng Anh.
// ============================================================================

use std::thread;
use std::time::Duration;
use xiangrust::learn::Gym;

fn main() {
    println!("============================================================================");
    println!("  XIANGRUST PROGRESSIVE DEPTH GYM CURRICULUM TRAINER DEMO (DEPTH 4..12)");
    println!("============================================================================");

    let gym = Gym::new();
    println!("1. Khởi tạo Môi trường GYM. Trạng thái ban đầu:");
    let st = gym.status();
    println!(
        "   - Active: {}, Depth: {}, Finished: {}, Samples: {}, Synced GM: {}",
        st.active, st.depth, st.finished, st.samples, st.synced
    );

    println!("\n2. Kích hoạt luồng tự huấn luyện ngầm GYM...");
    let spawned = gym.spawn();
    println!("   - Kết quả spawn luồng ngầm: {}", spawned);

    println!("\n3. Theo dõi tiến trình tự huấn luyện trong 3 giây...");
    thread::sleep(Duration::from_secs(3));

    let st2 = gym.status();
    println!("   - Cập nhật Telemetry GYM sau 3 giây:");
    println!(
        "   - Active: {}, Depth: {}, Finished: {}, Partial: {}, Samples: {}, Synced GM: {}",
        st2.active, st2.depth, st2.finished, st2.partial, st2.samples, st2.synced
    );

    println!("\n4. Tạm dừng an toàn môi trường GYM...");
    gym.stop();
    thread::sleep(Duration::from_millis(500));

    let st3 = gym.status();
    println!("   - Trạng thái sau khi ngắt dừng:");
    println!(
        "   - Active: {}, Depth: {}, Finished: {}, Samples: {}, Synced GM: {}",
        st3.active, st3.depth, st3.finished, st3.samples, st3.synced
    );

    println!("\n[HOÀN TẤT] Ví dụ 15 Gym Curriculum Trainer đã chạy thành công 100%!");
}
