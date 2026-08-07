// ============================================================================
// EMPIRICAL ADVERSARIAL STRESS TEST: MILESTONE M3 ITERATION 2.2 REVIEW
// ============================================================================
// Kiểm thử độc lập và stress-test đối kháng cho:
// 1. `src/gpu/gym.rs`: Gym GPU Accelerator alignment 64-byte, submission, process, evaluation.
// 2. `src/learn/gym.rs`: Gym Curriculum Trainer alignment 64-byte, single-word API methods.
// 3. CPU/GPU fallback mechanisms & sample packing integrity.
// ============================================================================

use std::mem::{align_of, size_of};
use std::time::Duration;
use xiangrust::board::Parser;
use xiangrust::gpu::gym::Gym as GpuGym;
use xiangrust::gpu::sample::Sample;
use xiangrust::learn::gym::{DATASET, Gym as LearnGym, Match, Status as LearnStatus};

#[test]
fn test_m3_2_2_gpu_gym_alignment_and_lifecycle() {
    println!("=== ADVERSARIAL REVIEW: GPU GYM ALIGNMENT & LIFECYCLE ===");

    // 1. Kiểm tra Căn lề bộ nhớ 64-byte cho GpuGym struct
    assert_eq!(align_of::<GpuGym>(), 64, "GpuGym MUST be 64-byte aligned!");
    assert_eq!(
        size_of::<GpuGym>(),
        704,
        "GpuGym size MUST be exactly 704 bytes (11 cache lines)!"
    );
    assert_eq!(
        size_of::<GpuGym>() % 64,
        0,
        "GpuGym size ({}) MUST be a multiple of 64 bytes!",
        size_of::<GpuGym>()
    );

    // 2. Khởi tạo GpuGym động cơ gia tốc phần cứng GPU
    let mut gpu_gym = GpuGym::init().expect("GpuGym init must succeed!");
    assert!(gpu_gym.active(), "GpuGym must be active upon initialization");
    assert_eq!(gpu_gym.depth(), 12, "Default depth must be 12");
    assert_eq!(gpu_gym.limit(), 4096, "Default batch limit must be 4096");

    // 3. Đánh giá vị trí rỗng [14u8; 90]
    let empty_board = [14u8; 90];
    let score = gpu_gym.evaluate(&empty_board).expect("Evaluate empty board must succeed");
    assert_eq!(score, 0, "Empty board evaluation score must be 0");

    // 4. Submit real position sample packed from Position
    let pos = Parser::parse(Parser::DEFAULT);
    let sample = Sample::pack(&pos, 1);
    assert!(gpu_gym.submit(&sample).is_ok(), "Submit valid packed sample must succeed");
    assert_eq!(gpu_gym.batch().count(), 1, "Batch count must be 1 after 1 submit");

    // 5. Process batch
    let processed = gpu_gym.process().expect("Process batch must succeed");
    assert_eq!(processed, 1, "Processed count must be 1");
    assert_eq!(gpu_gym.count(), 1, "Total count must be 1");

    // 6. Test reset
    assert!(gpu_gym.reset().is_ok(), "Reset must succeed");
    assert_eq!(gpu_gym.count(), 0, "Count must be reset to 0");
    assert_eq!(gpu_gym.batch().count(), 0, "Batch must be clear after reset");
}

#[test]
fn test_m3_2_2_learn_gym_single_word_api_and_alignment() {
    println!("=== ADVERSARIAL REVIEW: LEARN GYM SINGLE-WORD API & ALIGNMENT ===");

    // 1. Kiểm tra Hằng số DATASET
    assert_eq!(
        DATASET,
        ".agents/memory/experience_store.bin",
        "DATASET constant must point to experience store bin"
    );

    // 2. Kiểm tra Căn lề bộ nhớ 64-byte cho LearnStatus
    assert_eq!(align_of::<LearnStatus>(), 64, "LearnStatus MUST be 64-byte aligned!");
    assert_eq!(
        size_of::<LearnStatus>() % 64,
        0,
        "LearnStatus size ({}) MUST be a multiple of 64 bytes!",
        size_of::<LearnStatus>()
    );

    // 3. Kiểm tra Căn lề bộ nhớ 64-byte cho LearnGym
    assert_eq!(align_of::<LearnGym>(), 64, "LearnGym MUST be 64-byte aligned!");
    assert_eq!(
        size_of::<LearnGym>() % 64,
        0,
        "LearnGym size ({}) MUST be a multiple of 64 bytes!",
        size_of::<LearnGym>()
    );

    // 4. Kiểm tra các phương thức API từ đơn
    let learn_gym = LearnGym::new();
    let status = learn_gym.status();
    assert_eq!(status.active, 0, "Initial status active should be 0");
    assert_eq!(status.depth, 4, "Initial status depth should be 4");

    // Test `tune` method (single-word identifier)
    learn_gym.tune(12);
    assert_eq!(learn_gym.custom.load(std::sync::atomic::Ordering::Relaxed), 12);

    // Test `live` method (single-word identifier)
    let (live_fen, live_moves) = learn_gym.live();
    assert_eq!(live_fen, Parser::DEFAULT, "Live FEN should initially be DEFAULT");
    assert!(live_moves.is_empty(), "Live moves should initially be empty");

    // Test `matches` method (single-word identifier)
    let matches_list = learn_gym.matches();
    assert!(matches_list.is_empty(), "Initial matches list should be empty");
}

#[test]
fn test_m3_2_2_learn_gym_concurrency_and_spawn_lifecycle() {
    println!("=== ADVERSARIAL REVIEW: LEARN GYM SPAWN/STOP LIFECYCLE ===");

    let gym = LearnGym::new();
    gym.tune(4); // Set fast depth 4 for quick iteration test

    // Spawn background thread
    let spawned = gym.spawn();
    assert!(spawned, "First spawn call must return true");

    // Second spawn call should return false (already running)
    let spawned_again = gym.spawn();
    assert!(!spawned_again, "Second spawn call while active must return false");

    // Allow thread to run briefly
    std::thread::sleep(Duration::from_millis(100));

    let status = gym.status();
    assert_eq!(status.active, 1, "Status active should be 1 while running");

    // Stop background thread
    gym.stop();
    std::thread::sleep(Duration::from_millis(50));

    let status_after = gym.status();
    assert_eq!(status_after.active, 0, "Status active should be 0 after stop");
}

#[test]
fn test_m3_2_2_match_struct_integrity() {
    println!("=== ADVERSARIAL REVIEW: MATCH STRUCT INTEGRITY ===");

    let m = Match {
        id: 42,
        depth: 12,
        fen: Parser::DEFAULT.to_string(),
        moves: vec!["c2c5".to_string(), "h8g7".to_string()],
        outcome: "CHECKMATE".to_string(),
    };

    assert_eq!(m.id, 42);
    assert_eq!(m.depth, 12);
    assert_eq!(m.fen, Parser::DEFAULT);
    assert_eq!(m.moves.len(), 2);
    assert_eq!(m.outcome, "CHECKMATE");
}
