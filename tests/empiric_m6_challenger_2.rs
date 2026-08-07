// Thử nghiệm thực nghiệm chuyển đổi trạng thái CircuitBreaker, phát hiện bất thường NNUE, fallback HCE và phục hồi Half -> Closed
// Tác giả: challenger_m6_2 (M6 CircuitBreaker Stress Challenger)
// 100% chú thích Tiếng Việt, 100% định danh mã nguồn từ đơn tiếng Anh (Single-Word English Identifiers).

use std::sync::Arc;
use std::thread;
use xiangrust::board::Parser;
use xiangrust::circuit::{Breaker, Check, State};
use xiangrust::eval::{Eval, Mode};

/// Thử nghiệm 1: Trạng thái Closed
/// Bơm 100 điểm số NNUE hợp lệ (ví dụ: score = 150),
/// xác nhận breaker.state() luôn luôn duy trì State::Closed.
#[test]
fn closed() {
    let breaker = Breaker::new();
    assert_eq!(breaker.state(), State::Closed);

    for i in 0..100 {
        let val = 150;
        let valid = Check::valid(val, -29999, 29999);
        assert!(valid, "Điểm số 150 phải là hợp lệ!");
        breaker.record(valid, i);

        assert_eq!(
            breaker.state(),
            State::Closed,
            "Trạng thái Breaker phải luôn là Closed khi bơm điểm số hợp lệ ở lần thứ {}!",
            i + 1
        );
    }
}

/// Thử nghiệm 2: Trạng thái Trip -> Open và Fallback sang HCE
/// Bơm 5 điểm số NNUE bất thường (score > 29999 hoặc score < -29999),
/// xác nhận breaker.state() lập tức trip sang State::Open và Eval::score() fallback 100% sang HCE.
#[test]
fn trip() {
    let mut eval = Eval::new();
    let pos = Parser::parse(Parser::DEFAULT);

    assert_eq!(eval.circuit.state(), State::Closed);

    // Điểm số HCE cơ bản
    eval.mode(Mode::Hce);
    let hce = eval.score(&pos);

    // Chuyển sang Mode::Auto để sử dụng NNUE + CircuitBreaker
    eval.mode(Mode::Auto);

    // Bơm 5 điểm số bất thường (> 29999 hoặc < -29999) vào circuit
    let invalid = [30000, -30000, 35000, -35000, 99999];
    for (idx, &score) in invalid.iter().enumerate() {
        let valid = Check::valid(score, -29999, 29999);
        assert!(!valid, "Điểm số {} phải bị coi là bất thường!", score);
        eval.circuit.record(valid, 100);

        if idx < 4 {
            assert_eq!(
                eval.circuit.state(),
                State::Closed,
                "Breaker chưa đủ 5 lần lỗi nên vẫn giữ Closed tại lần thứ {}!",
                idx + 1
            );
        }
    }

    // Sau 5 điểm số bất thường, breaker lập tức trip sang State::Open
    assert_eq!(
        eval.circuit.state(),
        State::Open,
        "Breaker phải lập tức trip sang State::Open sau 5 lần điểm số bất thường!"
    );

    // Khi ở State::Open, breaker.allow(100) trả về false
    assert!(
        !eval.circuit.allow(100),
        "breaker.allow phải trả về false khi ở trạng thái Open!"
    );

    // Xác nhận Eval::score() khi breaker bị Open trả về điểm số fallback HCE 100%
    let fallback = eval.score(&pos);
    assert_eq!(
        fallback, hce,
        "Eval::score() phải fallback 100% sang HCE evaluation khi Breaker ở State::Open!"
    );
}

/// Thử nghiệm 3: Trạng thái Recover -> Half -> Closed
/// Giả lập tick vượt qua span (10000 ticks), xác nhận chuyển sang State::Half.
/// Bơm 100 điểm số hợp lệ, xác nhận phục hồi hoàn toàn về State::Closed.
#[test]
fn recover() {
    let breaker = Breaker::new();

    // 1. Trip breaker sang State::Open tại tick = 1000
    for _ in 0..5 {
        breaker.record(false, 1000);
    }
    assert_eq!(breaker.state(), State::Open);
    assert!(!breaker.allow(1000));

    // 2. Giả lập tick tiến lên chưa đủ span (ví dụ tick = 5000 < 1000 + 10000)
    assert!(!breaker.allow(5000));
    assert_eq!(breaker.state(), State::Open);

    // 3. Giả lập tick vượt qua span (tick = 1000 + 10000 = 11000)
    assert!(
        breaker.allow(11000),
        "breaker.allow(11000) phải trả về true khi tick vượt span!"
    );
    assert_eq!(
        breaker.state(),
        State::Half,
        "Trạng thái Breaker phải chuyển sang State::Half sau khi hết span timeout!"
    );

    // 4. Bơm 99 điểm số hợp lệ tại State::Half -> vẫn giữ State::Half
    for i in 0..99 {
        let valid = Check::valid(100, -29999, 29999);
        breaker.record(valid, 11000 + i as u64);
        assert_eq!(
            breaker.state(),
            State::Half,
            "Trạng thái vẫn phải là Half trước khi đạt đủ 100 mẩu hợp lệ (lần thứ {})!",
            i + 1
        );
    }

    // 5. Bơm điểm số hợp lệ thứ 100 -> Breaker chuyển hẳn về State::Closed
    let valid = Check::valid(200, -29999, 29999);
    breaker.record(valid, 11100);
    assert_eq!(
        breaker.state(),
        State::Closed,
        "Trạng thái Breaker phải phục hồi hoàn toàn về State::Closed sau 100 mẩu hợp lệ!"
    );
}

/// Thử nghiệm nâng cao: Phản ứng khi ở State::Half nhưng gặp 1 điểm bất thường
/// Xác nhận Breaker lập tức quay lại State::Open ngay mà không cần đợi 5 lỗi.
#[test]
fn failhalf() {
    let breaker = Breaker::new();

    // Trip sang Open
    for _ in 0..5 {
        breaker.record(false, 100);
    }
    assert_eq!(breaker.state(), State::Open);

    // Chuyển sang Half bằng cách vượt span
    assert!(breaker.allow(10100));
    assert_eq!(breaker.state(), State::Half);

    // Bơm 10 điểm hợp lệ
    for _ in 0..10 {
        breaker.record(true, 10100);
    }
    assert_eq!(breaker.state(), State::Half);

    // Bơm 1 điểm bất thường duy nhất ở State::Half -> Lập tức re-trip sang Open!
    breaker.record(false, 10105);
    assert_eq!(
        breaker.state(),
        State::Open,
        "Ở State::Half chỉ cần 1 điểm bất thường là phải lập tức quay lại State::Open!"
    );
}

/// Stress test đa luồng: 16 luồng đồng thời gọi allow và record trên 1 Breaker dùng chung
#[test]
fn stress() {
    let breaker = Arc::new(Breaker::new());
    let threads = 16;
    let iterations = 10000;

    let mut handles = Vec::with_capacity(threads);
    for t in 0..threads {
        let breaker = Arc::clone(&breaker);
        let handle = thread::spawn(move || {
            for i in 0..iterations {
                let tick = (t * iterations + i) as u64;
                if breaker.allow(tick) {
                    let score = if i % 1000 == 999 { 35000 } else { 120 };
                    let valid = Check::valid(score, -29999, 29999);
                    breaker.record(valid, tick);
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Sau khi kết thúc stress test, reset breaker về trạng thái sạch ban đầu
    breaker.reset();
    assert_eq!(breaker.state(), State::Closed);
}
