// Integration test độc lập của Challenger 2 (challenger_m2_gen2_2) nhằm kiểm chứng thực nghiệm đối kháng cho MoveGen & Perft.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::time::Instant;

use xiangrust::board::Parser;
use xiangrust::movegen::{legal, perft, List, Move};

thread_local! {
    static TRACKING: Cell<bool> = const { Cell::new(false) };
    static ALLOCATED_BYTES: Cell<usize> = const { Cell::new(0) };
    static ALLOCATED_COUNT: Cell<usize> = const { Cell::new(0) };
}

#[allow(dead_code)]
struct ChallengerAllocator;

unsafe impl GlobalAlloc for ChallengerAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        TRACKING.with(|t| {
            if t.get() {
                ALLOCATED_BYTES.with(|b| b.set(b.get() + layout.size()));
                ALLOCATED_COUNT.with(|c| c.set(c.get() + 1));
            }
        });
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
    }
}

// #[global_allocator]
// static ALLOCATOR: ChallengerAllocator = ChallengerAllocator;

fn start_alloc_tracking() {
    ALLOCATED_BYTES.with(|b| b.set(0));
    ALLOCATED_COUNT.with(|c| c.set(0));
    TRACKING.with(|t| t.set(true));
}

fn stop_alloc_tracking() -> (usize, usize) {
    TRACKING.with(|t| t.set(false));
    (ALLOCATED_BYTES.with(|b| b.get()), ALLOCATED_COUNT.with(|c| c.get()))
}

// 1. Thẩm định bố cục bộ nhớ vật lý (Physical memory layout & alignment)
#[test]
fn test_challenger_physical_memory_layout() {
    use std::mem::{align_of, offset_of, size_of};

    assert_eq!(size_of::<Move>(), 2, "Kích thước của Move phải đúng 2 bytes");
    assert_eq!(align_of::<Move>(), 1, "Căn lề của Move phải là 1 byte");

    assert_eq!(align_of::<List>(), 64, "List struct MUST have #[repr(C, align(64))]");
    assert_eq!(size_of::<List>(), 320, "List struct size MUST be 320 bytes");

    assert_eq!(offset_of!(List, items), 0, "Trường items phải ở offset 0");
    assert_eq!(offset_of!(List, count), 256, "Trường count phải ở offset 256");

    let list = List::new();
    let addr = &list as *const List as usize;
    assert_eq!(addr % 64, 0, "Địa chỉ biến List trên stack phải chia hết cho 64!");
}

// 2. Thẩm định Zero Heap Allocation trong sinh nước đi & Perft
#[test]
fn test_challenger_zero_heap_allocations() {
    let mut pos = Parser::parse(Parser::DEFAULT);

    start_alloc_tracking();

    let mut list = List::new();
    legal::gen(&mut pos, &mut list);

    let d1 = perft(&mut pos, 1);
    let d2 = perft(&mut pos, 2);
    let d3 = perft(&mut pos, 3);
    let d4 = perft(&mut pos, 4);

    let (bytes, count) = stop_alloc_tracking();

    assert_eq!(d1, 44);
    assert_eq!(d2, 1920);
    assert_eq!(d3, 79666);
    assert_eq!(d4, 3290240);

    assert_eq!(bytes, 0, "Số bytes cấp phát heap phải bằng 0! Đã cấp phát {} bytes", bytes);
    assert_eq!(count, 0, "Số lần cấp phát heap phải bằng 0! Đã cấp phát {} lần", count);
}

// 3. Thẩm định Perft node counts chính xác tuyệt đối ở các độ sâu 1, 2, 3, 4 & Benchmark NPS
#[test]
fn test_challenger_perft_node_counts_and_throughput() {
    let mut pos = Parser::parse(Parser::DEFAULT);

    let start = Instant::now();
    let nodes_d1 = perft(&mut pos, 1);
    assert_eq!(nodes_d1, 44, "Depth 1 MUST be 44 nodes");

    let nodes_d2 = perft(&mut pos, 2);
    assert_eq!(nodes_d2, 1920, "Depth 2 MUST be 1,920 nodes");

    let nodes_d3 = perft(&mut pos, 3);
    assert_eq!(nodes_d3, 79666, "Depth 3 MUST be 79,666 nodes");

    let nodes_d4 = perft(&mut pos, 4);
    let duration = start.elapsed();
    assert_eq!(nodes_d4, 3290240, "Depth 4 MUST be 3,290,240 nodes");

    let nps = (nodes_d4 as f64) / duration.as_secs_f64();
    println!("Perft Depth 4 runtime: {:?}, throughput: {:.2} nodes/sec", duration, nps);
}

// FEN chuẩn bàn cờ ban đầu
#[test]
fn test_challenger_complex_check_scenarios() {
    let fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1";
    let mut pos = Parser::parse(fen);
    let mut list = List::new();
    legal::gen(&mut pos, &mut list);
    assert_eq!(list.len(), 44);
}
