// Chẩn đoán đánh giá NNUE trên nhiều thế cờ thực tế

use xiangrust::board::Parser;
use xiangrust::eval::Eval;

fn main() {
    println!("=== CHẨN ĐOÁN MẠNG NNUE WEIGHTS TRÊN CÁC THẾ CỜ ===");
    let mut eval = Eval::new();
    println!("NNUE loaded: {}", eval.nnue.loaded);
    let f0 = eval.nnue.weight.feature(0);
    println!("Feature(0)[0..8]: {:?}", &f0[0..8]);
    let f100 = eval.nnue.weight.feature(100);
    println!("Feature(100)[0..8]: {:?}", &f100[0..8]);
    let f5000 = eval.nnue.weight.feature(5000);
    println!("Feature(5000)[0..8]: {:?}", &f5000[0..8]);

    let test_fens = [
        ("Khai cuộc mặc định (Default)", Parser::DEFAULT),
        ("Đỏ hơn 2 Xe (Red +2 Rooks)", "1nbakabn1/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1"),
        ("Đỏ mất 2 Xe (Black +2 Rooks)", "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/1NBAKABN1 w - - 0 1"),
        ("Tàn cuộc Đỏ hơn Xe", "3ak4/4a4/4b4/9/9/9/9/9/4R4/4K4 w - - 0 1"),
    ];

    for (desc, fen) in &test_fens {
        let pos = Parser::parse(fen);
        eval.reset(&pos);

        println!("\n=======================================================");
        println!("📌 Thế cờ: {}", desc);
        println!("   FEN: {}", fen);

        // 1. Kiểm tra Accumulator
        println!("   Accum[0][0..8]: {:?}", &eval.accum.vals[0][0..8]);
        println!("   Accum[1][0..8]: {:?}", &eval.accum.vals[1][0..8]);

        // 2. Kiểm tra Transform
        let mut transform = xiangrust::eval::nnue::Transform::new();
        transform.active(&eval.accum.vals[0], &eval.accum.vals[1], pos.side);
        println!("   Transform own[0..8]: {:?}", &transform.active[0..8]);
        println!("   Transform opp[0..8]: {:?}", &transform.active[256..264]);

        // 3. Kiểm tra Hidden Layer (Affine)
        let mut hidden = [0i32; 32];
        eval.nnue.affine.forward(&transform.active, &mut hidden);
        println!("   Hidden[0..8]: {:?}", &hidden[0..8]);

        // 4. Kiểm tra Layer sau Clip32
        let mut layer = [0i8; 32];
        for i in 0..32 {
            let val = hidden[i] >> 6;
            layer[i] = if val < 0 { 0i8 } else if val > 127 { 127i8 } else { val as i8 };
        }
        println!("   Layer[0..8] (ClipReLU): {:?}", &layer[0..8]);

        // 5. Kiểm tra Output
        let dot_product = unsafe { xiangrust::simd::bytes(&layer, &eval.nnue.output.weight) };
        println!("   Output Dot Product: {}", dot_product);
        println!("   Output Bias: {}", eval.nnue.output.bias);
        let raw_sum = eval.nnue.output.bias + dot_product;
        let score_nnue = (raw_sum * 25) / 508;
        println!("   Raw Sum: {} -> Score: {:+6} cp", raw_sum, score_nnue);
    }
}
