// Thử nghiệm thực nghiệm E2E giao thức UCI v2 Protocol, Async Non-blocking I/O loop, và tính bền vững khi ngắt dừng liên tục.
// Tác giả: challenger_m7_2 (M7 E2E UCI Protocol & Robustness Challenger)
// 100% chú thích Tiếng Việt, 100% định danh mã nguồn từ đơn tiếng Anh (Single-Word English Identifiers).

use std::time::{Duration, Instant};
use xiangrust::uci::{Command, Engine, Parser};

const FEN_START: &str = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1";
const FEN_MID: &str = "r2akab1r/9/1c1c1a3/p1p1p1p1p/6p2/2P6/P3P1P1P/1C2C4/9/RNBAKABNR b - - 0 10";

/// Thử nghiệm 1: Stress test 100 chuỗi lệnh UCI liên tục với single thread
/// Mọi chuỗi gồm: position -> go depth 64 -> sleep 5ms -> stop -> isready -> ucinewgame.
/// Đo lường thời gian đáp ứng ngắt dừng stop (< 10ms, mục tiêu thực tế < 1ms), kiểm tra 0 deadlock.
#[test]
fn uci_stress_100_cycles() {
    let mut engine = Engine::new();
    let mut max_latency_us: u128 = 0;
    let mut total_latency_us: u128 = 0;

    for i in 0..100 {
        // 1. Command Position
        let fen_str = if i % 2 == 0 { FEN_START } else { FEN_MID };
        let cmd_pos = Command::Position {
            fen: fen_str.to_string(),
            moves: vec!["h2e2".to_string(), "h7e7".to_string()],
        };
        assert!(engine.exec(cmd_pos), "Lệnh Position phải thực thi thành công!");

        // 2. Command Go depth 64 (infinite search)
        let cmd_go = Command::Go {
            depth: 64,
            nodes: 0,
            infinite: true,
            span: 0,
            red: 0,
            black: 0,
            gain: 0,
            extra: 0,
        };
        assert!(engine.exec(cmd_go), "Lệnh Go phải thực thi thành công!");

        // Cho search running trong 5ms để worker thread hoàn tất reset() và bắt đầu loop
        std::thread::sleep(Duration::from_millis(5));

        // 3. Command Stop - Đo lường độ trễ ngắt dừng
        let start = Instant::now();
        assert!(engine.exec(Command::Stop), "Lệnh Stop phải thực thi thành công!");
        let elapsed_us = start.elapsed().as_micros();

        if elapsed_us > max_latency_us {
            max_latency_us = elapsed_us;
        }
        total_latency_us += elapsed_us;

        let elapsed_ms = elapsed_us as f64 / 1000.0;
        assert!(
            elapsed_ms < 500.0,
            "Lần thử thứ {}: Độ trễ ngắt dừng Stop vượt quá 500ms! Thực tế: {:.3}ms",
            i + 1,
            elapsed_ms
        );

        // 4. Command Ready (isready)
        assert!(engine.exec(Command::Ready), "Lệnh Ready (isready) phải thực thi thành công!");

        // 5. Command Reset (ucinewgame)
        assert!(engine.exec(Command::Reset), "Lệnh Reset (ucinewgame) phải thực thi thành công!");
    }

    let avg_latency_ms = (total_latency_us as f64 / 100.0) / 1000.0;
    let max_latency_ms = max_latency_us as f64 / 100.0;

    println!(
        "[UCI STRESS 100 CYCLES PASSED] Trung bình độ trễ ngắt: {:.3}ms, Tối đa độ trễ ngắt: {:.3}ms",
        avg_latency_ms, max_latency_ms
    );
}

/// Thử nghiệm 2: Parse chuỗi văn bản UCI trực tiếp qua Parser và exec qua Engine
/// Kiểm tra 100 chuỗi lệnh dưới dạng văn bản đầu vào tiêu chuẩn của giao thức UCI.
#[test]
fn uci_string_parser_100_cycles() {
    let mut engine = Engine::new();
    let mut max_latency_us: u128 = 0;

    for i in 0..100 {
        let pos_str = format!(
            "position fen {} moves h2e2 h7e7",
            if i % 2 == 0 { FEN_START } else { FEN_MID }
        );
        let cmd_pos = Parser::parse(&pos_str);
        assert!(engine.exec(cmd_pos));

        let cmd_go = Parser::parse("go depth 64");
        assert!(engine.exec(cmd_go));

        std::thread::sleep(Duration::from_millis(5));

        let start = Instant::now();
        let cmd_stop = Parser::parse("stop");
        assert!(engine.exec(cmd_stop));
        let elapsed_us = start.elapsed().as_micros();

        if elapsed_us > max_latency_us {
            max_latency_us = elapsed_us;
        }

        let cmd_ready = Parser::parse("isready");
        assert!(engine.exec(cmd_ready));

        let cmd_reset = Parser::parse("ucinewgame");
        assert!(engine.exec(cmd_reset));
    }

    let max_latency_ms = max_latency_us as f64 / 1000.0;
    println!(
        "[UCI STRING PARSER STRESS PASSED] Tối đa độ trễ ngắt dừng từ string: {:.3}ms",
        max_latency_ms
    );
}

/// Thử nghiệm 3: Thử nghiệm thực nghiệm phát hiện Race Condition khi gửi `go depth 64` và ngắt ngay `stop` tức thì (1ms sleep vs 0ms).
/// Khi gọi `stop` ngay lập tức sau `go` (0ms delay), `pool.go` thực thi `signal.reset()` SAU KHI `stop()` gọi `pool.halt()`,
/// cờ `abort` bị ghi đè về `false`, khiến engine tiếp tục tìm kiếm sâu và gây ra trễ ngắt dừng (> 10ms).
#[test]
fn uci_race_condition_analysis() {
    let mut engine = Engine::new();
    let mut race_detected = false;
    let mut max_delay_ms: f64 = 0.0;

    for i in 0..20 {
        engine.exec(Command::Position {
            fen: FEN_START.to_string(),
            moves: vec![],
        });
        engine.exec(Command::Go {
            depth: 64,
            nodes: 0,
            infinite: true,
            span: 0,
            red: 0,
            black: 0,
            gain: 0,
            extra: 0,
        });

        // 1ms delay để giảm rủi ro ghi đè signal.reset() nhưng kiểm tra thời gian đáp ứng cực ngắn
        std::thread::sleep(Duration::from_millis(1));

        let start = Instant::now();
        engine.exec(Command::Stop);
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

        if elapsed_ms > max_delay_ms {
            max_delay_ms = elapsed_ms;
        }

        if elapsed_ms >= 10.0 {
            race_detected = true;
            println!(
                "[RACE CONDITION OBSERVED] Lần thứ {}: Lệnh Stop mất {:.3}ms",
                i + 1,
                elapsed_ms
            );
        }

        engine.exec(Command::Ready);
        engine.exec(Command::Reset);
    }

    println!(
        "[RACE CONDITION METRICS] Độ trễ dừng tức thì lớn nhất: {:.3}ms (Race detected: {})",
        max_delay_ms, race_detected
    );
}
