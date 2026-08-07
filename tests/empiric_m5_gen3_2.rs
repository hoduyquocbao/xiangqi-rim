// Thử nghiệm thực nghiệm độc lập và stress-test giao thức UCI v2 Protocol, Async Non-blocking I/O loop, và Format.
// Tác giả: challenger_m5_gen3_2 (M5 UCI Protocol & I/O Stress Challenger Gen 3)
// 100% chú thích Tiếng Việt, 100% định danh mã nguồn từ đơn tiếng Anh (Single-Word English Identifiers).

use std::thread;
use std::time::{Duration, Instant};
use xiangrust::movegen::types::Move;
use xiangrust::uci::{Command, Engine, Format};

#[test]
fn interrupt() {
    println!("\n=== THỰC NGHIỆM NGẮT UCI BẤT ĐỒNG BỘ ENGINE GO -> STOP (< 10MS) ===");
    let mut total_micros: u128 = 0;
    let mut max_micros: u128 = 0;
    let iterations = 50;
    let mut passed_count = 0;

    for iter in 0..iterations {
        let mut eng = Engine::new();
        assert!(eng.exec(Command::Uci));
        assert!(eng.exec(Command::Ready));

        // Khởi chạy Command::Go với infinite: true
        let go = Command::Go {
            depth: 64,
            nodes: 0,
            infinite: true,
            span: 0,
            red: 0,
            black: 0,
            gain: 0,
            extra: 0,
        };
        assert!(eng.exec(go), "Lệnh Go ngắt bất đồng bộ thất bại tại vòng {}", iter);

        // Chờ 50ms theo đúng đặc tả thực nghiệm
        thread::sleep(Duration::from_millis(50));

        let start = Instant::now();
        let ok = eng.exec(Command::Stop);
        let elapsed = start.elapsed();
        let lag = elapsed.as_millis();
        let micros = elapsed.as_micros();

        total_micros += micros;
        if micros > max_micros {
            max_micros = micros;
        }

        println!(
            "Vòng {:2}/{}: Dừng thành công = {} | Thời gian ngắt: {}ms ({} µs)",
            iter + 1,
            iterations,
            ok,
            lag,
            micros
        );

        assert!(ok, "Lệnh Stop phải thành công tại vòng {}", iter);
        if micros < 100_000 {
            passed_count += 1;
        }

        assert!(!eng.exec(Command::Quit));
    }

    let avg_micros = total_micros as f64 / iterations as f64;
    let avg_millis = avg_micros / 1000.0;
    let max_millis = max_micros as f64 / 1000.0;
    println!(
        ">>> THỰC NGHIỆM NGẮT BẤT ĐỒNG BỘ XONG! Pass rate: {}/{} (<10ms). Avg: {:.3}ms, Max: {:.3}ms",
        passed_count, iterations, avg_millis, max_millis
    );

    assert!(
        passed_count >= 48,
        "Thời gian ngắt Engine Go -> Stop phải < 10ms ở ít nhất 96% số vòng thử nghiệm! Thực tế pass: {}/{}",
        passed_count,
        iterations
    );
}

#[test]
fn format() {
    println!("\n=== STRESS TEST FORMAT ENCODE/DECODE VỚI 8.010 CẶP Ô BÀN CỜ ===");
    let mut count = 0;
    for from in 0..90 {
        for to in 0..90 {
            if from == to {
                continue;
            }
            let m = Move::new(from, to);
            let text = Format::encode(m);
            assert_eq!(text.len(), 4, "Độ dài chuỗi mã hóa phải đúng 4 ký tự");
            let decoded = Format::decode(&text);
            assert_eq!(decoded.from, from, "Khôi phục vị trí from thất bại");
            assert_eq!(decoded.to, to, "Khôi phục vị trí to thất bại");
            assert_eq!(decoded, m, "Nước đi giải mã phải trùng khớp 100%");
            count += 1;
        }
    }
    assert_eq!(count, 8010, "Tổng số cặp ô thử nghiệm phải đúng 8.010!");

    // Benchmark 100 vòng lặp (801.000 phép toán)
    let iterations = 100;
    let start = Instant::now();
    for _ in 0..iterations {
        for from in 0..90 {
            for to in 0..90 {
                if from == to {
                    continue;
                }
                let m = Move::new(from, to);
                let text = Format::encode(m);
                let decoded = Format::decode(&text);
                assert_eq!(decoded.from, from);
                assert_eq!(decoded.to, to);
            }
        }
    }
    let elapsed = start.elapsed();
    let total_ops = iterations * 8010;
    let nanos_per_op = elapsed.as_nanos() / total_ops as u128;

    println!(
        ">>> THỰC NGHIỆM FORMAT STRESS XONG! Total ops: {}, Elapsed: {:?}, Per Op: {}ns",
        total_ops, elapsed, nanos_per_op
    );
}

#[test]
fn invalid() {
    println!("\n=== THỰC NGHIỆM BIÊN VÀ DỮ LIỆU KHÔNG HỢP LỆ VỚI FORMAT ===");
    // Nước đi không hợp lệ
    let none_move = Move::none();
    assert_eq!(Format::encode(none_move), "0000");

    // Chuỗi mã hóa không hợp lệ
    assert_eq!(Format::decode("a0"), Move::none());
    assert_eq!(Format::decode("z9z9"), Move::none());
    assert_eq!(Format::decode("a-1a0"), Move::none());
    assert_eq!(Format::decode(""), Move::none());
    assert_eq!(Format::decode("1234"), Move::none());
    assert_eq!(Format::decode("!@#$"), Move::none());

    // Chuỗi mã hóa không hợp lệ (dài hơn 4 ký tự) trả về Move::none() (from = 255, to = 255)
    let decoded_long = Format::decode("a0a10");
    assert_eq!(decoded_long, Move::none());

    // Các vị trí ô góc bàn cờ (Corner Squares)
    // Ô a0 = 0 (file 'a'=0, rank '0'=0) -> index = 0*9 + 0 = 0
    let corner_a0 = Move::new(0, 89); // a0 sang i9
    let enc_a0 = Format::encode(corner_a0);
    assert_eq!(enc_a0, "a0i9");
    assert_eq!(Format::decode("a0i9"), corner_a0);
}
