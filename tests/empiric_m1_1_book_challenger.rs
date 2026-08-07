// ============================================================================
// EMPIRIC M1.1 BOOK & ENDGAME ADVERSARIAL CHALLENGER HARNESS
// ============================================================================
// File kiểm thử đối kháng tự động cho Zobrist Opening Book và Endgame Engine.
// Tuân thủ quy tắc Đơn Từ 100% (Single-Word Principle) cho định danh mã nguồn.
// ============================================================================

use std::time::Instant;
use xiangrust::board::Parser;
use xiangrust::book::endgame::{Endgame, DRAW, LOSS, WIN};
use xiangrust::book::opening::{Book, ENTRIES};

#[test]
fn sorted() {
    let mut i = 0;
    while i < ENTRIES.len() - 1 {
        let first = ENTRIES[i].hash;
        let second = ENTRIES[i + 1].hash;
        assert!(
            first < second,
            "Mảng ENTRIES BẮT BUỘC phải sắp xếp tăng dần 100% theo hash!"
        );
        i += 1;
    }
}

#[test]
fn collision() {
    let limit = ENTRIES.len();
    let mut i = 0;
    while i < limit {
        let mut j = i + 1;
        while j < limit {
            assert_ne!(
                ENTRIES[i].hash,
                ENTRIES[j].hash,
                "Phát hiện trùng lặp Zobrist hash giữa hai bản ghi khai cuộc!"
            );
            j += 1;
        }
        i += 1;
    }

    let book = Book::default();
    let mut k = 0;
    while k < limit {
        let entry = &ENTRIES[k];
        let res = book.find(entry.hash);
        assert!(res.is_some(), "Binary search không tìm thấy entry hợp lệ!");
        let mv = res.unwrap();
        assert_eq!(mv.raw(), entry.mv, "Xung đột nước đi thu được từ Zobrist probe!");
        k += 1;
    }
}

#[test]
fn latency() {
    let book = Book::default();
    let limit = ENTRIES.len();

    let start = Instant::now();
    let mut rounds = 0;
    while rounds < 100 {
        let mut idx = 0;
        while idx < limit {
            let hash = ENTRIES[idx].hash;
            let res = book.find(hash);
            assert!(res.is_some());
            idx += 1;
        }
        rounds += 1;
    }
    let elapsed = start.elapsed();

    // 100 vòng probe x 1,024 entries = 102,400 lượt probe
    let total = elapsed.as_secs_f64() * 1000.0;
    println!("Tổng thời gian probe 102,400 lượt: {:.4} ms", total);

    // Thời gian probe 1,024 entries phải nhỏ hơn 1ms
    let per = total / 100.0;
    println!("Thời gian probe 1,024 entries: {:.4} ms", per);
    assert!(
        per < 1.0,
        "Thời gian Book::probe 1,024 entries vượt quá ngưỡng 1ms!"
    );
}

#[test]
fn knight() {
    // 1. Red Hero Win: Đỏ đi, Đỏ có Mã vs Đen có Sĩ -> WIN (+15000)
    let fen = "4k1a2/9/9/9/9/9/9/4N4/9/4K4 w - - 0 1";
    let pos = Parser::parse(fen);
    let score = Endgame::eval(&pos);
    assert_eq!(score, Some(WIN), "Đỏ đi, Đỏ có Mã vs Đen có Sĩ phải là WIN!");

    // 2. Red Hero Loss: Đỏ đi, Đen có Mã vs Đỏ có Sĩ -> LOSS (-15000)
    let fen = "4k4/9/4n4/9/9/9/9/9/4A4/4K4 w - - 0 1";
    let pos = Parser::parse(fen);
    let score = Endgame::eval(&pos);
    assert_eq!(score, Some(LOSS), "Đỏ đi, Đen có Mã vs Đỏ có Sĩ phải là LOSS!");

    // 3. Black Hero Win: Đen đi, Đen có Mã vs Đỏ có Sĩ -> WIN (+15000)
    let fen = "4k4/9/4n4/9/9/9/9/9/4A4/4K4 b - - 0 1";
    let pos = Parser::parse(fen);
    let score = Endgame::eval(&pos);
    assert_eq!(score, Some(WIN), "Đen đi, Đen có Mã vs Đỏ có Sĩ phải là WIN!");

    // 4. Black Hero Loss: Đen đi, Đỏ có Mã vs Đen có Sĩ -> LOSS (-15000)
    let fen = "4k1a2/9/9/9/9/9/9/4N4/9/4K4 b - - 0 1";
    let pos = Parser::parse(fen);
    let score = Endgame::eval(&pos);
    assert_eq!(score, Some(LOSS), "Đen đi, Đỏ có Mã vs Đen có Sĩ phải là LOSS!");
}

#[test]
fn cannon() {
    // Đơn Pháo Khuyết Tượng vs Đơn Sĩ (Lượt Đỏ) -> DRAW (0)
    let fen = "4k1a2/9/9/9/9/9/9/4C4/9/4K4 w - - 0 1";
    let pos = Parser::parse(fen);
    let score = Endgame::eval(&pos);
    assert_eq!(score, Some(DRAW), "Đơn Pháo Khuyết Tượng hòa Đơn Sĩ!");

    // Đơn Pháo Khuyết Tượng vs Đơn Sĩ (Lượt Đen) -> DRAW (0)
    let fen = "4k1a2/9/9/9/9/9/9/4C4/9/4K4 b - - 0 1";
    let pos = Parser::parse(fen);
    let score = Endgame::eval(&pos);
    assert_eq!(score, Some(DRAW), "Đơn Pháo Khuyết Tượng hòa Đơn Sĩ!");
}

#[test]
fn rook() {
    // Xe Mã vs Xe Sĩ Tượng (Lượt Đỏ) -> WIN (+15000)
    let fen = "2bak4/9/4r4/9/9/9/9/4N4/4R4/4K4 w - - 0 1";
    let pos = Parser::parse(fen);
    let score = Endgame::eval(&pos);
    assert_eq!(score, Some(WIN), "Xe Mã thắng Xe Sĩ Tượng (Đỏ đi)!");

    // Xe Mã vs Xe Sĩ Tượng (Lượt Đen - Red attack, Black hero) -> LOSS (-15000)
    let fen = "2bak4/9/4r4/9/9/9/9/4N4/4R4/4K4 b - - 0 1";
    let pos = Parser::parse(fen);
    let score = Endgame::eval(&pos);
    assert_eq!(score, Some(LOSS), "Xe Mã thắng Xe Sĩ Tượng (Đen bị LOSS)!");
}

#[test]
fn cannons() {
    // 1. Red Hero Win: Đỏ 2 Pháo vs Đen Khuyết Sĩ Tượng (1 Sĩ 1 Tượng) -> WIN (+15000)
    let fen = "2b1ka3/9/9/9/9/9/9/4C4/4C4/4K4 w - - 0 1";
    let pos = Parser::parse(fen);
    let score = Endgame::eval(&pos);
    assert_eq!(score, Some(WIN), "Hai Pháo thắng Khuyết Sĩ Tượng!");

    // 2. Red Hero Win: Đỏ 2 Pháo vs Đen Khuyết Tượng (2 Sĩ 1 Tượng) -> WIN (+15000)
    let fen = "2b1ka2a/9/9/9/9/9/9/4C4/4C4/4K4 w - - 0 1";
    let pos = Parser::parse(fen);
    let score = Endgame::eval(&pos);
    assert_eq!(score, Some(WIN), "Hai Pháo thắng Khuyết Tượng!");

    // 3. Red Hero Win: Đỏ 2 Pháo vs Đen Khuyết Sĩ (1 Sĩ 2 Tượng) -> WIN (+15000)
    let fen = "2b1ka1b1/9/9/9/9/9/9/4C4/4C4/4K4 w - - 0 1";
    let pos = Parser::parse(fen);
    let score = Endgame::eval(&pos);
    assert_eq!(score, Some(WIN), "Hai Pháo thắng Khuyết Sĩ!");

    // 4. Black Hero Win: Đen 2 Pháo vs Đỏ Khuyết Sĩ Tượng -> WIN (+15000)
    let fen = "4k4/4c4/4c4/9/9/9/9/9/2B1KA3/2B6 b - - 0 1";
    let pos = Parser::parse(fen);
    let score = Endgame::eval(&pos);
    assert_eq!(score, Some(WIN), "Đen 2 Pháo thắng Đỏ Khuyết Sĩ Tượng phải là WIN cho Đen!");

    // 5. Black Hero Loss: Đen Khuyết Sĩ Tượng vs Đỏ 2 Pháo (Đen đi) -> LOSS (-15000)
    let fen = "2b1ka3/9/9/9/9/9/9/4C4/4C4/4K4 b - - 0 1";
    let pos = Parser::parse(fen);
    let score = Endgame::eval(&pos);
    assert_eq!(score, Some(LOSS), "Đen Khuyết Sĩ Tượng vs Đỏ 2 Pháo phải là LOSS cho Đen!");
}

#[test]
fn bare() {
    // Không còn quân công ở cả 2 bên -> DRAW (0)
    let fen = "4k4/9/9/9/9/9/9/9/9/4K4 w - - 0 1";
    let pos = Parser::parse(fen);
    let score = Endgame::eval(&pos);
    assert_eq!(score, Some(DRAW), "Hai bên trần Tướng hòa!");
}

#[test]
fn bishop() {
    // Đơn Mã hòa Đơn Tượng -> DRAW (0)
    let fen = "4k1b2/9/9/9/9/9/9/4N4/9/4K4 w - - 0 1";
    let pos = Parser::parse(fen);
    let score = Endgame::eval(&pos);
    assert_eq!(score, Some(DRAW), "Đơn Mã hòa Đơn Tượng!");
}

#[test]
fn knights() {
    // Hai Mã thắng Sĩ Tượng Toàn -> WIN (+15000)
    let fen = "2bakab2/9/9/9/9/9/9/4N4/4N4/4K4 w - - 0 1";
    let pos = Parser::parse(fen);
    let score = Endgame::eval(&pos);
    assert_eq!(score, Some(WIN), "Hai Mã thắng Sĩ Tượng Toàn!");
}

#[test]
fn pawn() {
    // Pháo Tốt qua sông thắng Khuyết Sĩ Tượng -> WIN (+15000)
    let fen = "2b1ka3/9/9/9/3P5/9/9/4C4/9/4K4 w - - 0 1";
    let pos = Parser::parse(fen);
    let score = Endgame::eval(&pos);
    assert_eq!(score, Some(WIN), "Pháo Tốt qua sông thắng Khuyết Sĩ Tượng!");
}
