// Test suite stress-testing edge cases in MoveGen (`src/movegen/`)

use xiangrust::board::Parser;
use xiangrust::movegen::{legal, pseudo, List};

#[test]
fn test_red_sideways_pawn_check_detection() {
    // Red King at e0 (square 4: file 4, rank 0).
    // Black King at e9 (square 85: file 4, rank 9).
    // Black Pawn at d0 (square 3: file 3, rank 0) - right next to Red King!
    // FEN: 4k4/9/9/9/9/9/9/9/9/3pK4 w - - 0 1
    let fen = "4k4/9/9/9/9/9/9/9/9/3pK4 w - - 0 1";
    let pos = Parser::parse(fen);

    let is_in_check = legal::check(&pos, 0); // 0 = Red
    println!("Red king in check from d0 Black Pawn: {}", is_in_check);
    assert!(
        is_in_check,
        "BUG CONFIRMED: Red King at e0 MUST be in check from Black Pawn at d0!"
    );
}

#[test]
fn test_black_sideways_pawn_check_detection() {
    // Black King at e9 (square 85: file 4, rank 9).
    // Red King at e0 (square 4: file 4, rank 0).
    // Red Pawn at d9 (square 84: file 3, rank 9) - right next to Black King!
    // FEN: 3Pk4/9/9/9/9/9/9/9/9/4K4 b - - 0 1
    let fen = "3Pk4/9/9/9/9/9/9/9/9/4K4 b - - 0 1";
    let pos = Parser::parse(fen);

    let is_in_check = legal::check(&pos, 1); // 1 = Black
    println!("Black king in check from d9 Red Pawn: {}", is_in_check);
    assert!(
        is_in_check,
        "BUG CONFIRMED: Black King at e9 MUST be in check from Red Pawn at d9!"
    );
}

#[test]
fn test_knight_check_leg_blocking() {
    // Red King at e0 (square 4: file 4, rank 0).
    // Black King at e9 (square 85: file 4, rank 9).
    // Black Knight at d2 (square 21: file 3, rank 2).
    // Delta from d2 to e0 is (file +1, rank -2).
    // Knight's leg when moving from d2 to e0 is d1 (square 12: file 3, rank 1).
    // Put a Red Advisor at d1 (square 12).
    // Now Black Knight's leg d1 is BLOCKED by Red Advisor at d1!
    // Therefore, Black Knight CANNOT attack e0, so Red King is NOT in check!
    // FEN: 4k4/9/9/9/9/9/9/3n5/3A5/4K4 w - - 0 1
    let fen = "4k4/9/9/9/9/9/9/3n5/3A5/4K4 w - - 0 1";
    let pos = Parser::parse(fen);

    let is_in_check = legal::check(&pos, 0);
    println!("Red king in check from blocked d2 Knight: {}", is_in_check);
    assert!(
        !is_in_check,
        "BUG CONFIRMED: Red King at e0 should NOT be in check when Black Knight's leg d1 is blocked!"
    );
}

#[test]
fn test_knight_check_unblocked_leg() {
    // Red King at e0 (square 4: file 4, rank 0).
    // Black King at e9 (square 85: file 4, rank 9).
    // Black Knight at d2 (square 21: file 3, rank 2).
    // Leg at d1 (square 12) is EMPTY.
    // Put a piece at e1 (square 13: file 4, rank 1).
    // FEN: 4k4/9/9/9/9/9/9/3n5/4A4/4K4 w - - 0 1
    let fen = "4k4/9/9/9/9/9/9/3n5/4A4/4K4 w - - 0 1";
    let pos = Parser::parse(fen);

    let is_in_check = legal::check(&pos, 0);
    println!("Red king in check from unblocked d2 Knight (piece at e1): {}", is_in_check);
    assert!(
        is_in_check,
        "BUG CONFIRMED: Red King at e0 MUST be in check from Black Knight at d2 when leg d1 is empty (even if e1 is occupied)!"
    );
}

#[test]
fn test_pinned_piece_exposing_pawn_check() {
    // Red King at e0 (square 4). Red Advisor at d0 (square 3). Black Rook at c0 (square 2).
    // Black King at e9 (square 85).
    // FEN: 4k4/9/9/9/9/9/9/9/9/2rAK4 w - - 0 1
    let fen = "4k4/9/9/9/9/9/9/9/9/2rAK4 w - - 0 1";
    let mut pos = Parser::parse(fen);

    let mut moves = List::new();
    legal::gen(&mut pos, &mut moves);

    // Check if move d0 -> e1 (from 3 to 13) is generated as legal.
    // It MUST NOT be generated because moving Advisor from d0 exposes Red King to Black Rook at c0!
    let exposes_check_move = moves.items[..moves.count].iter().any(|m| m.from == 3 && m.to == 13);
    println!("Illegal move d0->e1 exposing check generated: {}", exposes_check_move);
    assert!(
        !exposes_check_move,
        "BUG CONFIRMED: Advisor move d0->e1 MUST NOT be legal because it exposes King to check!"
    );
}

#[test]
fn test_elephant_eye_blocking() {
    // Red Elephant at c0 (square 2). Target e2 (square 20). Eye is d1 (square 11).
    // Put a piece at d1 (square 11).
    // FEN: 4k4/9/9/9/9/9/9/9/3P5/2B1K4 w - - 0 1
    let fen = "4k4/9/9/9/9/9/9/9/3P5/2B1K4 w - - 0 1";
    let pos = Parser::parse(fen);

    let mut moves = List::new();
    pseudo::elephant(&pos, &mut moves);

    let blocked_move = moves.items[..moves.count].iter().any(|m| m.from == 2 && m.to == 20);
    assert!(
        !blocked_move,
        "Elephant move c0->e2 MUST be blocked by piece at d1!"
    );
}

#[test]
fn test_pawn_river_crossing_movement() {
    // Red Pawn before river at e3 (square 31: file 4, rank 3).
    // Red Pawn after river at e5 (square 49: file 4, rank 5).
    // FEN: 4k4/9/9/9/4P4/9/4P4/9/9/4K4 w - - 0 1
    let fen = "4k4/9/9/9/4P4/9/4P4/9/9/4K4 w - - 0 1";
    let pos = Parser::parse(fen);

    let mut moves = List::new();
    pseudo::pawn(&pos, &mut moves);

    // Pawn at e3 (31) can ONLY move to e4 (40). (1 move)
    // Pawn at e5 (49) can move to e6 (58), d5 (48), f5 (50). (3 moves)
    let e3_moves: Vec<_> = moves.items[..moves.count].iter().filter(|m| m.from == 31).collect();
    let e5_moves: Vec<_> = moves.items[..moves.count].iter().filter(|m| m.from == 49).collect();

    assert_eq!(e3_moves.len(), 1, "Pawn before river should have exactly 1 move (forward)");
    assert_eq!(e3_moves[0].to, 40, "Pawn at e3 must move to e4");

    assert_eq!(e5_moves.len(), 3, "Pawn after river should have 3 moves (forward, left, right)");
}

#[test]
fn test_flying_general_prevention() {
    // Red King at e0 (square 4). Red Advisor at e1 (square 13). Black King at e9 (square 85).
    // FEN: 4k4/9/9/9/9/9/9/9/4A4/4K4 w - - 0 1
    let fen = "4k4/9/9/9/9/9/9/9/4A4/4K4 w - - 0 1";
    let mut pos = Parser::parse(fen);

    let mut moves = List::new();
    legal::gen(&mut pos, &mut moves);

    // Advisor at e1 (13) can move to d0 (3), f0 (5), d2 (21), f2 (23).
    // But e1->d0 and e1->f0 leave file 4 empty, exposing Red King to Black King (Flying General)!
    // So e1->d0 and e1->f0 MUST be filtered out!
    let exposes_flying_gen = moves.items[..moves.count].iter().any(|m| m.from == 13 && (m.to == 3 || m.to == 5));
    assert!(
        !exposes_flying_gen,
        "Advisor moves e1->d0 or e1->f0 MUST be illegal due to Flying General!"
    );
}
