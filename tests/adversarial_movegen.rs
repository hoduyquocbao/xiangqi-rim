// Integration test đối kháng chuyên sâu cho module MoveGen & Perft.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use xiangrust::board::{Parser, Position, Square};
use xiangrust::movegen::{legal, perft, List, Move};

thread_local! {
    static TRACK_ALLOC: Cell<bool> = const { Cell::new(false) };
    static ALLOC_COUNT: Cell<usize> = const { Cell::new(0) };
}

#[allow(dead_code)]
struct TrackingAllocator;

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        TRACK_ALLOC.with(|track| {
            if track.get() {
                ALLOC_COUNT.with(|count| {
                    count.set(count.get() + layout.size());
                });
            }
        });
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
    }
}

// #[global_allocator]
// static A: TrackingAllocator = TrackingAllocator;

fn start_tracking() {
    ALLOC_COUNT.with(|c| c.set(0));
    TRACK_ALLOC.with(|t| t.set(true));
}

fn stop_tracking() -> usize {
    TRACK_ALLOC.with(|t| t.set(false));
    ALLOC_COUNT.with(|c| c.get())
}

// 1. Kiểm thử các hằng số và thuộc tính căn lề của Move và List
#[test]
fn test_movegen_types_alignment() {
    use std::mem::{align_of, size_of};
    assert_eq!(size_of::<Move>(), 2, "Move type size MUST be 2 bytes");
    assert_eq!(align_of::<Move>(), 1, "Move type alignment MUST be 1 byte");

    assert_eq!(align_of::<List>(), 64, "List struct alignment MUST be 64 bytes");
    assert_eq!(size_of::<List>(), 320, "List struct size MUST be 320 bytes (256 bytes items + 8 bytes count + 56 bytes padding)");

    // Kiểm tra địa chỉ bộ nhớ stack của List có được căn lề 64-byte không
    let list = List::new();
    let ptr = &list as *const List as usize;
    assert_eq!(ptr % 64, 0, "Stack allocated List address MUST be 64-byte aligned!");
}

// 2. Kiểm thử ZERO HEAP ALLOCATION trong quá trình sinh nước đi và duyệt cây Perft
#[test]
fn test_zero_heap_allocation_during_perft() {
    let mut pos = Parser::parse(Parser::DEFAULT);

    start_tracking();

    // Sinh nước đi pseudo-legal & legal
    let mut list = List::new();
    legal::gen(&mut pos, &mut list);

    // Duyệt Perft Depth 1, 2, 3
    let n1 = perft(&mut pos, 1);
    let n2 = perft(&mut pos, 2);
    let n3 = perft(&mut pos, 3);

    let bytes_allocated = stop_tracking();

    assert_eq!(n1, 44, "Perft Depth 1 MUST equal 44 nodes");
    assert_eq!(n2, 1920, "Perft Depth 2 MUST equal 1,920 nodes");
    assert_eq!(n3, 79666, "Perft Depth 3 MUST equal 79,666 nodes");

    assert_eq!(
        bytes_allocated, 0,
        "Move generation and Perft tree traversal MUST perform ZERO heap allocations! (allocated {} bytes)",
        bytes_allocated
    );
}

// 3. Kiểm thử trường hợp Lộ mặt Tướng (Flying General)
#[test]
fn test_flying_general_prevention() {
    // Đặt Tướng Đỏ ô 4 (file 4, rank 0), Tướng Đen ô 76 (file 4, rank 8).
    // Ở giữa chỉ có 1 quân cản là Tốt Đỏ tại ô 13 (file 4, rank 1).
    let mut pos = Position::empty();
    pos.put(0, 4);  // Red King tại ô 4
    pos.put(7, 76); // Black King tại ô 76
    pos.put(6, 13); // Red Pawn tại ô 13 (quân cản duy nhất)

    // Nếu Tốt Đỏ ở ô 13 di chuyển sang trái (ô 12) hoặc sang phải (ô 14), hai Tướng sẽ nhìn mặt nhau!
    // Do đó nước đi của Tốt Đỏ ra khỏi cột 4 PHẢI BỊ BỎ trong legal moves!
    pos.side = 0; // Red to move
    let mut moves = List::new();
    legal::gen(&mut pos, &mut moves);

    let mut i = 0;
    while i < moves.count {
        let mv = moves.items[i];
        if mv.from == 13 {
            let to_sq = Square(mv.to);
            assert_eq!(to_sq.file(), 4, "Nước đi của quân cản làm lộ mặt Tướng không được xuất hiện trong legal moves!");
        }
        i += 1;
    }
}

// 4. Kiểm thử trường hợp thoát chiếu Tướng (Check Evasion)
#[test]
fn test_check_evasion() {
    // Vị trí Tướng Đỏ bị Xe Đen chiếu trực tiếp:
    // Red King tại ô 4. Black Rook tại ô 13 (file 4, rank 1) chiếu Tướng Đỏ.
    // Red Advisor tại ô 3 (file 3, rank 0) có thể đi vào ô 13 để ăn Xe Đen thoát chiếu.
    let mut pos = Position::empty();
    pos.put(0, 4);   // Red King tại ô 4
    pos.put(11, 13); // Black Rook tại ô 13
    pos.put(1, 3);   // Red Advisor tại ô 3
    pos.side = 0;    // Red to move

    assert!(legal::check(&pos, 0), "Tướng Đỏ phải được ghi nhận là đang bị chiếu!");

    let mut moves = List::new();
    legal::gen(&mut pos, &mut moves);

    // Mọi nước đi hợp lệ thu được phải làm cho Tướng Đỏ không còn bị chiếu
    let mut i = 0;
    while i < moves.count {
        let mv = moves.items[i];
        let state = pos.apply(mv.from, mv.to);
        assert!(!legal::check(&pos, 0), "Nước đi hợp lệ phải giải phóng Tướng khỏi thế bị chiếu!");
        pos.revert(mv.from, mv.to, &state);
        i += 1;
    }
}

// 5. Kiểm thử băm trôi (drift) và khôi phục trạng thái tuyệt đối qua Perft tree traversal
#[test]
fn test_perft_state_invariants() {
    let mut pos = Parser::parse(Parser::DEFAULT);
    let orig = pos;

    let nodes = perft(&mut pos, 2);
    assert_eq!(nodes, 1920);

    // Sau khi duyệt xong cây Perft, trạng thái Position và Hash Zobrist phải trùng khớp 100% với ban đầu
    assert_eq!(pos, orig, "Bàn cờ bị lệch trạng thái sau khi duyệt cây Perft!");
    assert_eq!(pos.hash, orig.hash, "Khóa băm Zobrist bị trôi sau khi duyệt cây Perft!");
}

// 6. Kiểm thử Perft Depth 4 để đánh giá độ chính xác và hiệu năng ở quy mô lớn hơn (Chuẩn Xiangqi: 3,290,240 nodes)
#[test]
fn test_perft_depth_4() {
    let mut pos = Parser::parse(Parser::DEFAULT);
    let n4 = perft(&mut pos, 4);
    assert_eq!(n4, 3290240, "Perft Depth 4 cho vị trí ban đầu phải là 3,290,240 nodes");
}
