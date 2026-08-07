// ============================================================================
// ĐIỂM THỰC THI CHÍNH (ENTRY POINT) CỦA ỨNG DỤNG ENGINE CỜ TƯỚNG XIANGRUST
// ============================================================================
// Khởi chạy vòng lặp xử lý giao thức chuẩn UCI v2 (Universal Chess Interface)
// hoặc khởi chạy máy chủ Backend REST API & WebSocket Server nếu có cờ `--serve`.
// Tuân thủ 100% quy tắc định danh từ đơn tiếng Anh và chú thích tiếng Việt.
// ============================================================================

use std::sync::atomic::Ordering;
use xiangrust::server::Server;
use xiangrust::uci::Engine;

/// Hàm `main` - Điểm khởi đầu thực thi của chương trình binary `xiangrust`
fn main() {
    // 1. Phân tích tham số dòng lệnh std::env::args()
    let args: Vec<String> = std::env::args().collect();
    let mut mb = 64usize;
    let mut threads = 1usize;
    let mut serve = false;
    let mut addr = "0.0.0.0:8888".to_string();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--hash-mb" | "--hash" => {
                if i + 1 < args.len() {
                    if let Ok(val) = args[i + 1].parse::<usize>() {
                        mb = val.clamp(16, 8192);
                    }
                    i += 1;
                }
            }
            "--threads" | "-t" => {
                if i + 1 < args.len() {
                    if let Ok(val) = args[i + 1].parse::<usize>() {
                        threads = val.clamp(1, 128);
                    }
                    i += 1;
                }
            }
            "--serve" | "-s" => {
                serve = true;
            }
            "--bind" | "-b" => {
                if i + 1 < args.len() {
                    addr = args[i + 1].clone();
                    serve = true;
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    // 2. Kiểm tra cờ --serve để quyết định chế độ chạy Backend Server hay UCI Engine
    if serve {
        println!("[Main] Khởi chạy XiangRust Server tại {} với Hash RAM {}MB...", addr, mb);
        let server = Server::bind(&addr).expect("Không thể bind địa chỉ máy chủ server");
        server.hash.store(mb, Ordering::Relaxed);
        server.listen().expect("Lỗi thực thi máy chủ server");
    } else {
        // Khởi tạo Engine UCI mặc định
        let mut engine = Engine::new();
        engine.exec(xiangrust::uci::Command::Option {
            name: "Hash".to_string(),
            value: mb.to_string(),
        });
        if threads > 1 {
            engine.exec(xiangrust::uci::Command::Option {
                name: "Threads".to_string(),
                value: threads.to_string(),
            });
        }
        engine.run();
    }
}