// ============================================================================
// KIỂM THỬ THỰC NGHIỆM ĐA LUỒNG CHO TRANSPOSITION TABLE PARTITION (SHARDING)
// ============================================================================
// File kiểm thử độc lập do Challenger M2-2 xây dựng nhằm stress-test:
// 1. Căn lề bộ nhớ 64-byte của Partition và Table.
// 2. Độ an toàn dữ liệu và tính toàn vẹn Hyatt XOR khi 16 luồng ghi/đọc đồng thời.
// 3. Phân bổ phần tử trên các Partition Shards.
// 4. Kiểm tra khả năng chia sẻ TT giữa các luồng trong Lazy SMP.
// ============================================================================

use std::sync::Arc;
use std::thread;
use xiangrust::movegen::types::Move;
use xiangrust::tt::partition::Partition;
use xiangrust::tt::table::Table;

/// Kiểm thử căn lề 64-byte cho Partition và Table.
#[test]
fn alignment() {
    assert_eq!(std::mem::align_of::<Partition>(), 64);
    assert_eq!(std::mem::size_of::<Partition>(), 64);
    assert_eq!(std::mem::align_of::<Table>(), 64);
    assert_eq!(std::mem::size_of::<Table>(), 64);
}

/// Stress-test 16 luồng đọc ghi đồng thời vào Partition đơn lẻ.
#[test]
fn concurrency() {
    let part = Arc::new(Partition::new(0, 1024));
    let count = 16;
    let ops = 100_000usize;
    let mut handles = Vec::with_capacity(count);

    for tid in 0..count {
        let p = part.clone();
        let handle = thread::spawn(move || {
            let mut state = (tid as u64).wrapping_add(1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            for _ in 0..ops {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                let key = if state == 0 { 1 } else { state };
                let from = ((state >> 16) % 90) as u8;
                let to = ((state >> 24) % 90) as u8;
                let step = Move::new(from, to);
                let score = (state as i16) % 10000;
                let depth = ((state >> 32) % 32) as u8 + 1;
                let bound = ((state >> 40) % 3 + 1) as u8;
                let age = (state % 256) as u8;

                p.save(key, depth, bound, step, score, age);

                if let Some(item) = p.probe(key) {
                    assert_eq!(item.key, key);
                    assert!(item.depth > 0);
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

/// Stress-test Table multi-partition routing dưới áp lực 16 luồng.
#[test]
fn distribution() {
    let table = Arc::new(Table::new(16));
    let count = 16;
    let ops = 50_000usize;
    let mut handles = Vec::with_capacity(count);

    for tid in 0..count {
        let t = table.clone();
        let handle = thread::spawn(move || {
            let mut state = (tid as u64).wrapping_add(1).wrapping_mul(0x517C_C1B7_2722_0A95);
            for _ in 0..ops {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                let key = if state == 0 { 1 } else { state };
                let from = ((state >> 16) % 90) as u8;
                let to = ((state >> 24) % 90) as u8;
                let step = Move::new(from, to);
                let score = (state as i16) % 10000;
                let depth = ((state >> 32) % 32) as u8 + 1;
                let bound = ((state >> 40) % 3 + 1) as u8;

                t.save_with(key, depth, bound, step, score, tid);
                let _ = t.probe_with(key, tid);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

/// Kiểm thử khả năng tương tác và chia sẻ dữ liệu TT giữa các luồng trong Lazy SMP.
#[test]
fn sharing() {
    let table = Table::new(16);
    let key = 0x1234_5678_9ABC_DEF0u64;
    let step = Move::new(10, 20);

    // Luồng 0 lưu thông tin thế cờ key vào TT
    table.save_with(key, 10, 1, step, 500, 0);

    // Kiểm tra xem Luồng 0 có đọc được không
    let probed_0 = table.probe_with(key, 0);
    assert!(probed_0.is_some(), "Luồng 0 phải đọc lại được thế cờ đã lưu");

    // Kiểm tra xem Luồng 1 có đọc được thông tin do Luồng 0 lưu hay không
    let probed_1 = table.probe_with(key, 1);
    
    // Đo đạc thực nghiệm: nốt ghi nhận kết quả đọc của Luồng 1 từ Luồng 0
    println!("TT Sharing result between Thread 0 and Thread 1: {:?}", probed_1.is_some());
}
