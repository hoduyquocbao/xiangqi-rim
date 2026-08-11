// ============================================================================
// XIANGTI ENGINE: BỘ KIỂM THỬ THỰC NGHIỆM STRESS TEST GPU BATCH & EVALUATOR (M2.1)
// ============================================================================
// File: tests/empiric_m2_1_1_challenger_gpu_stress.rs
// Mục đích: Stress test thực nghiệm 1k-16k samples, push/pull boundary,
// VRAM Guard capacity limit, zero-copy shared memory alignment, và score write-back.
// Tuân thủ 100% chú thích tiếng Việt và định danh tiếng Anh.
// ============================================================================

use xiangrust::gpu::{Batch, Buffer, Device, Evaluable, Evaluator, Guard, Sample, Sampleable, Status};
use xiangrust::board::Position;

#[test]
fn test_batch_high_volume_submission_1k_to_16k() {
    // Thử nghiệm đẩy 1k, 4k, 8k, 16k samples vào Batch
    let device = Device::init();
    let sizes = [1000usize, 4096, 8192, 16384];

    for &cap in &sizes {
        let mut batch = Batch::allocate(&device, cap).expect("Allocate batch failed");
        assert_eq!(batch.capacity(), cap);
        assert_eq!(batch.count(), 0);
        assert!(batch.empty());

        let pos = Position::default();
        for i in 0..cap {
            let sample = Sample::pack(&pos, i as u32);
            assert!(batch.push(&sample).is_ok(), "Push failed at index {}", i);
        }

        assert_eq!(batch.count(), cap);
        assert!(batch.full());

        // Kiểm tra tính toàn vẹn dữ liệu cho từng sample
        for i in 0..cap {
            let sample = batch.pull(i).expect("Pull failed");
            assert_eq!(sample.index(), i as u32);
            assert_eq!(sample.side(), pos.side);
            assert_eq!(sample.hash(), pos.hash);
        }
    }
}

#[test]
fn test_batch_boundary_handling_full_and_fault() {
    let device = Device::init();

    // 1. Kiểm tra giới hạn sức chứa không hợp lệ (0 và > 16384)
    assert_eq!(Batch::allocate(&device, 0).err(), Some(Status::Fault));
    assert_eq!(Batch::allocate(&device, 16385).err(), Some(Status::Fault));

    // 2. Kiểm tra push vượt quá capacity (Status::Full)
    let mut batch = Batch::allocate(&device, 10).expect("Allocate batch failed");
    let sample = Sample::new();
    for _ in 0..10 {
        assert!(batch.push(&sample).is_ok());
    }
    assert!(batch.full());
    assert_eq!(batch.push(&sample).err(), Some(Status::Full));

    // 3. Kiểm tra pull chỉ số vượt giới hạn count (Status::Fault)
    assert_eq!(batch.pull(10).err(), Some(Status::Fault));
    assert_eq!(batch.pull(999).err(), Some(Status::Fault));

    // 4. Kiểm tra pull sau khi clear()
    batch.clear();
    assert!(batch.empty());
    assert_eq!(batch.count(), 0);
    assert_eq!(batch.pull(0).err(), Some(Status::Fault));
}

#[test]
fn test_vram_guard_capacity_limits() {
    let device = Device::init();
    let guard = device.guard();

    let _initial_allocated = guard.allocated();

    // Cấp phát liên tục các batch lớn cho đến khi chạm trần VRAM (409.6MB = 429,496,729 bytes)
    let mut batches = Vec::new();
    let mut exhausted_hit = false;

    for _ in 0..300 {
        match Batch::allocate(&device, 16384) {
            Ok(b) => batches.push(b),
            Err(Status::Exhausted) => {
                exhausted_hit = true;
                break;
            }
            Err(err) => panic!("Unexpected error: {:?}", err),
        }
    }

    assert!(exhausted_hit, "Expected Status::Exhausted when reaching VRAM ceiling 409.6MB");
    assert!(guard.allocated() <= guard.ceiling(), "Allocated VRAM should not exceed ceiling");

    // Giải phóng bộ nhớ
    drop(batches);
    device.guard().wipe();
    assert_eq!(device.guard().allocated(), 0);
}

#[test]
fn test_zero_copy_shared_memory_alignment_and_integrity() {
    // Kiểm tra căn lề bộ nhớ 64-byte và kích thước vật lý của các struct
    assert_eq!(std::mem::align_of::<Sample>(), 64);
    assert_eq!(std::mem::size_of::<Sample>(), 128);

    assert_eq!(std::mem::align_of::<Batch>(), 64);
    assert_eq!(std::mem::size_of::<Batch>(), 128);

    assert_eq!(std::mem::align_of::<Evaluator>(), 64);
    assert_eq!(std::mem::size_of::<Evaluator>(), 256);

    assert_eq!(std::mem::align_of::<Buffer>(), 64);
    assert_eq!(std::mem::size_of::<Buffer>(), 64);

    assert_eq!(std::mem::align_of::<Guard>(), 64);
    assert_eq!(std::mem::size_of::<Guard>(), 64);

    // Kiểm tra tính toàn vẹn 0-copy byte-level
    let device = Device::init();
    let mut batch = Batch::allocate(&device, 100).unwrap();
    let pos = Position::default();

    for i in 0..100 {
        let mut sample = Sample::pack(&pos, i as u32);
        sample.store((i as i32) * 10 - 500);
        batch.push(&sample).unwrap();
    }

    for i in 0..100 {
        let sample = batch.pull(i).unwrap();
        assert_eq!(sample.score(), (i as i32) * 10 - 500);
    }
}

#[test]
fn test_evaluator_score_writeback_integrity() {
    // Kiểm tra xem Evaluator::flush có cập nhật điểm số trở lại Batch hay không
    let mut evaluator = Evaluator::auto().expect("Init evaluator failed");
    let device = Device::init();
    let mut batch = Batch::allocate(&device, 10).expect("Allocate batch failed");

    let mut grid = [14u8; 90]; // Mảng 90 ô cờ với 14 là ô trống
    grid[0] = 6; // Đỏ có 1 Xe (Rook = index 6, weight = 90)
    grid[1] = 5; // Đỏ có 1 Pháo (Cannon = index 5, weight = 45)

    let mut sample = Sample::new();
    let encode_res = sample.encode(&grid, 0);
    assert_eq!(encode_res, Status::Ready);
    assert_eq!(sample.score(), 0);

    let expected_score = evaluator.compute(&sample).expect("Compute score failed");
    assert_ne!(expected_score, 0, "Expected non-zero computed score for test position");

    batch.push(&sample).expect("Push sample failed");

    let count = evaluator.flush(&mut batch).expect("Flush evaluator failed");
    assert_eq!(count, 1);

    // Rút sample ra sau khi flush để kiểm tra xem score trong Batch có được ghi nhận hay không
    let evaluated_sample = batch.pull(0).expect("Pull sample after flush failed");

    println!("Original sample score before flush: 0");
    println!("Evaluated expected score: {}", expected_score);
    println!("Actual score in batch after flush: {}", evaluated_sample.score());

    assert_eq!(
        evaluated_sample.score(),
        expected_score,
        "CRITICAL BUG DETECTED: Evaluator::flush failed to write evaluated scores back to Batch!"
    );
}
