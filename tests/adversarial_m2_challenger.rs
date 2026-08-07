// Adversarial test suite created by Challenger 1 (m2_gen2_1) to stress-test MoveGen correctness.

use xiangrust::board::{Parser, Position, Square};
use xiangrust::movegen::{legal, List};

// 1. Stress test Pawn check logic in all directions for Red King
#[test]
fn test_pawn_check_red_king() {
    // Red King at e0 (4)
    // Black Pawn at d0 (3): sideways check -> MUST BE CHECK
    let fen1 = "4k4/9/9/9/9/9/9/9/9/3pK4 w - - 0 1";
    let pos1 = Parser::parse(fen1);
    assert!(legal::check(&pos1, 0), "Red King at e0 MUST be in check from Black Pawn at d0!");

    // Black Pawn at f0 (5): sideways check -> MUST BE CHECK
    let fen2 = "4k4/9/9/9/9/9/9/9/9/4Kp3 w - - 0 1";
    let pos2 = Parser::parse(fen2);
    assert!(legal::check(&pos2, 0), "Red King at e0 MUST be in check from Black Pawn at f0!");

    // Black Pawn at e1 (13): forward check -> MUST BE CHECK
    let fen3 = "4k4/9/9/9/9/9/9/9/4p4/4K4 w - - 0 1";
    let pos3 = Parser::parse(fen3);
    assert!(legal::check(&pos3, 0), "Red King at e0 MUST be in check from Black Pawn at e1!");

    // Red King at d1 (12). Black Pawn at d0 (3): behind King -> NOT CHECK (Black pawns move down)
    let fen4 = "4k4/9/9/9/9/9/9/9/3K4/3p5 w - - 0 1";
    let pos4 = Parser::parse(fen4);
    assert!(!legal::check(&pos4, 0), "Red King at d1 should NOT be in check from Black Pawn at d0 behind it!");

    // Red King at d1 (12). Black Pawn at c1 (11): sideways check -> MUST BE CHECK
    let fen5 = "4k4/9/9/9/9/9/9/9/2pK5/9 w - - 0 1";
    let pos5 = Parser::parse(fen5);
    assert!(legal::check(&pos5, 0), "Red King at d1 MUST be in check from Black Pawn at c1!");
}

// 2. Stress test Pawn check logic in all directions for Black King
#[test]
fn test_pawn_check_black_king() {
    // Black King at e9 (85)
    // Red Pawn at d9 (84): sideways check -> MUST BE CHECK
    let fen1 = "3Pk4/9/9/9/9/9/9/9/9/4K4 b - - 0 1";
    let pos1 = Parser::parse(fen1);
    assert!(legal::check(&pos1, 1), "Black King at e9 MUST be in check from Red Pawn at d9!");

    // Red Pawn at f9 (86): sideways check -> MUST BE CHECK
    let fen2 = "4kP3/9/9/9/9/9/9/9/9/4K4 b - - 0 1";
    let pos2 = Parser::parse(fen2);
    assert!(legal::check(&pos2, 1), "Black King at e9 MUST be in check from Red Pawn at f9!");

    // Red Pawn at e8 (76): forward check -> MUST BE CHECK
    let fen3 = "4k4/4P4/9/9/9/9/9/9/9/4K4 b - - 0 1";
    let pos3 = Parser::parse(fen3);
    assert!(legal::check(&pos3, 1), "Black King at e9 MUST be in check from Red Pawn at e8!");

    // Black King at d8 (75). Red Pawn at d9 (84): behind King -> NOT CHECK (Red pawns move up)
    let fen4 = "3P4/3k5/9/9/9/9/9/9/9/4K4 b - - 0 1";
    let pos4 = Parser::parse(fen4);
    assert!(!legal::check(&pos4, 1), "Black King at d8 should NOT be in check from Red Pawn at d9 behind it!");
}

// 3. Stress test Knight leg blocking in all 8 jump directions to Red King at d1 (12)
#[test]
fn test_knight_check_8_directions() {
    // Target King at d1 (file 3, rank 1, square 12).
    // Direction 1: Knight at c3 (file 2, rank 3, sq 29). Leg is c2 (file 2, rank 2, sq 20).
    // Unblocked:
    let mut pos = Position::empty();
    pos.put(0, 12);  // Red King at d1
    pos.put(7, 85);  // Black King at e9
    pos.put(10, 29); // Black Knight at c3
    assert!(legal::check(&pos, 0), "Knight at c3 should check Red King at d1 when c2 is empty");
    // Blocked by Red Pawn at c2 (20):
    pos.put(6, 20);  // Red Pawn at c2
    assert!(!legal::check(&pos, 0), "Knight at c3 should NOT check Red King at d1 when c2 is blocked");

    // Direction 2: Knight at e3 (file 4, rank 3, sq 31). Leg is e2 (file 4, rank 2, sq 22).
    let mut pos2 = Position::empty();
    pos2.put(0, 12);  // Red King at d1
    pos2.put(7, 85);  // Black King at e9
    pos2.put(10, 31); // Black Knight at e3
    assert!(legal::check(&pos2, 0), "Knight at e3 should check Red King at d1 when e2 is empty");
    pos2.put(6, 22);  // Red Pawn at e2
    assert!(!legal::check(&pos2, 0), "Knight at e3 should NOT check Red King at d1 when e2 is blocked");

    // Direction 3: Knight at b2 (file 1, rank 2, sq 19). Leg is c2 (file 2, rank 2, sq 20).
    let mut pos3 = Position::empty();
    pos3.put(0, 12);  // Red King at d1
    pos3.put(7, 85);  // Black King at e9
    pos3.put(10, 19); // Black Knight at b2
    assert!(legal::check(&pos3, 0), "Knight at b2 should check Red King at d1 when c2 is empty");
    pos3.put(6, 20);  // Red Pawn at c2
    assert!(!legal::check(&pos3, 0), "Knight at b2 should NOT check Red King at d1 when c2 is blocked");

    // Direction 4: Knight at f2 (file 5, rank 2, sq 23). Leg is e2 (file 4, rank 2, sq 22).
    let mut pos4 = Position::empty();
    pos4.put(0, 12);  // Red King at d1
    pos4.put(7, 85);  // Black King at e9
    pos4.put(10, 23); // Black Knight at f2
    assert!(legal::check(&pos4, 0), "Knight at f2 should check Red King at d1 when e2 is empty");
    pos4.put(6, 22);  // Red Pawn at e2
    assert!(!legal::check(&pos4, 0), "Knight at f2 should NOT check Red King at d1 when e2 is blocked");

    // Direction 5: Knight at b0 (file 1, rank 0, sq 1). Leg is c0 (file 2, rank 0, sq 2).
    let mut pos5 = Position::empty();
    pos5.put(0, 12);  // Red King at d1
    pos5.put(7, 85);  // Black King at e9
    pos5.put(10, 1);  // Black Knight at b0
    assert!(legal::check(&pos5, 0), "Knight at b0 should check Red King at d1 when c0 is empty");
    pos5.put(6, 2);   // Red Pawn at c0
    assert!(!legal::check(&pos5, 0), "Knight at b0 should NOT check Red King at d1 when c0 is blocked");

    // Direction 6: Knight at f0 (file 5, rank 0, sq 5). Leg is e0 (file 4, rank 0, sq 4).
    let mut pos6 = Position::empty();
    pos6.put(0, 12);  // Red King at d1
    pos6.put(7, 85);  // Black King at e9
    pos6.put(10, 5);  // Black Knight at f0
    assert!(legal::check(&pos6, 0), "Knight at f0 should check Red King at d1 when e0 is empty");
    pos6.put(6, 4);   // Red Pawn at e0
    assert!(!legal::check(&pos6, 0), "Knight at f0 should NOT check Red King at d1 when e0 is blocked");
}

// 4. Stress test pinned pieces exposing King to check
#[test]
fn test_pinned_piece_behavior() {
    // Red King at e0 (4). Red Rook at e1 (13). Black Rook at e8 (76). Black King at a9 (81).
    // Red Rook at e1 is pinned along file 4.
    // Moving Red Rook off file 4 (e.g. e1 -> d1 or e1 -> f1) exposes King to check.
    // Moving Red Rook along file 4 (e.g. e1 -> e2, e1 -> e8 capturing Black Rook) is LEGAL!
    let mut pos = Position::empty();
    pos.put(0, 4);   // Red King at e0 (4)
    pos.put(4, 13);  // Red Rook at e1 (13)
    pos.put(11, 76); // Black Rook at e8 (76)
    pos.put(7, 81);  // Black King at a9 (81)
    pos.side = 0;

    let mut moves = List::new();
    legal::gen(&mut pos, &mut moves);

    let off_file_moves = moves.items[..moves.count].iter().any(|m| m.from == 13 && Square(m.to).file() != 4);
    let along_file_moves = moves.items[..moves.count].iter().any(|m| m.from == 13 && Square(m.to).file() == 4);

    assert!(!off_file_moves, "Pinned Red Rook MUST NOT be allowed to move off file 4!");
    assert!(along_file_moves, "Pinned Red Rook MUST be allowed to move along file 4 (including capture)!");
}

// 5. Stress test Cannon pin with double screen
#[test]
fn test_cannon_pin_double_screen() {
    // Red King at e0 (4). Red Advisor at e1 (13). Red Pawn at e2 (22). Black Cannon at e9 (85). Black King at a9 (81).
    // There are 2 screens (Advisor at e1, Pawn at e2) between Red King and Black Cannon.
    // If Advisor at e1 moves off file 4 (e.g. e1 -> d0), there is STILL 1 screen (Pawn at e2) left!
    // Therefore, moving Advisor at e1 off file 4 EXPOSES Red King to check from Black Cannon!
    // So Advisor moves off file 4 MUST be rejected as illegal!
    let mut pos = Position::empty();
    pos.put(0, 4);   // Red King at e0 (4)
    pos.put(1, 13);  // Red Advisor at e1 (13)
    pos.put(6, 22);  // Red Pawn at e2 (22)
    pos.put(12, 85); // Black Cannon at e9 (85)
    pos.put(7, 81);  // Black King at a9 (81)
    pos.side = 0;

    let mut moves = List::new();
    legal::gen(&mut pos, &mut moves);

    let advisor_moves_off_file = moves.items[..moves.count].iter().any(|m| m.from == 13 && Square(m.to).file() != 4);
    assert!(
        !advisor_moves_off_file,
        "Advisor move from e1 off file 4 MUST be rejected because it leaves 1 screen for Black Cannon, putting Red King in check!"
    );
}

// 6. Stress test King moving directly into check
#[test]
fn test_king_cannot_step_into_check() {
    // Red King at e0 (4). Black Pawn at d1 (12). Black King at a9 (81).
    // Black Pawn at d1 attacks d0 (3) (sideways) and d0/e1/c1 depending on movement.
    // Red King at e0 moving to d0 (3) puts King adjacent to Black Pawn at d1!
    // Red King at e0 moving to e1 (13) puts King adjacent to Black Pawn at d1!
    let mut pos = Position::empty();
    pos.put(0, 4);   // Red King at e0 (4)
    pos.put(13, 12); // Black Pawn at d1 (12)
    pos.put(7, 81);  // Black King at a9 (81)
    pos.side = 0;

    let mut moves = List::new();
    legal::gen(&mut pos, &mut moves);

    let king_moves_to_d0 = moves.items[..moves.count].iter().any(|m| m.from == 4 && m.to == 3);
    let king_moves_to_e1 = moves.items[..moves.count].iter().any(|m| m.from == 4 && m.to == 13);

    assert!(!king_moves_to_d0, "Red King MUST NOT move to d0 (into Pawn check)!");
    assert!(!king_moves_to_e1, "Red King MUST NOT move to e1 (into Pawn check)!");
}
