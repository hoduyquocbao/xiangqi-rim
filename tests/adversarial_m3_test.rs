// ============================================================================
// XIANGTI ENGINE: ADVERSARIAL STRESS TEST SUITE FOR M3 ITERATION 2.1
// ============================================================================
// File: tests/adversarial_m3_test.rs
// Empirical Challenger verification suite for gpu::gym and learn::gym.
// ============================================================================

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use xiangrust::gpu::gym::Gym as GpuGym;
use xiangrust::gpu::sample::Sample;
use xiangrust::gpu::status::Status as GpuStatus;
use xiangrust::learn::gym::{Gym as LearnGym, Status as LearnStatus};

/// 1. EMPIRICAL TEST: GPU Gym layout and alignment
#[test]
fn test_adversarial_gpu_gym_layout() {
    assert_eq!(std::mem::size_of::<GpuGym>() % 64, 0, "GpuGym size must be aligned to 64 bytes");
    assert_eq!(std::mem::align_of::<GpuGym>(), 64, "GpuGym alignment must be 64 bytes");
}

/// 2. EMPIRICAL TEST: Learn Gym layout and alignment
#[test]
fn test_adversarial_learn_gym_layout() {
    assert_eq!(std::mem::size_of::<LearnStatus>() % 64, 0, "LearnStatus size must be aligned to 64 bytes");
    assert_eq!(std::mem::align_of::<LearnStatus>(), 64, "LearnStatus alignment must be 64 bytes");
    assert_eq!(std::mem::size_of::<LearnGym>() % 64, 0, "LearnGym size must be aligned to 64 bytes");
    assert_eq!(std::mem::align_of::<LearnGym>(), 64, "LearnGym alignment must be 64 bytes");
}

/// 3. EMPIRICAL TEST: GPU Gym boundary conditions on `evaluate`
#[test]
fn test_adversarial_gpu_gym_boundary_evaluate() {
    let mut gym = GpuGym::init().expect("GPU Gym init failed");
    
    // Empty position must return Err(GpuStatus::Fault)
    let empty: [u8; 0] = [];
    assert_eq!(gym.evaluate(&empty), Err(GpuStatus::Fault), "Empty slice must yield Fault error");
    
    // 1-byte position boundary condition
    let slice1 = [14u8; 1];
    assert!(gym.evaluate(&slice1).is_ok(), "1-byte position slice must be safely truncated without panic");
    
    // 89-byte position boundary condition
    let slice89 = [14u8; 89];
    assert!(gym.evaluate(&slice89).is_ok(), "89-byte position slice must be safely handled");
    
    // 90-byte position boundary condition (standard empty board)
    let slice90 = [14u8; 90];
    let score = gym.evaluate(&slice90).expect("90-byte position evaluate failed");
    assert_eq!(score, 0, "Empty board score should be 0");
    
    // Oversized position (1000 bytes boundary condition)
    let slice1000 = [14u8; 1000];
    assert!(gym.evaluate(&slice1000).is_ok(), "1000-byte position slice must be safely clamped to 90 bytes without buffer overflow");
}

/// 4. EMPIRICAL TEST: GPU Gym buffer queuing & overflow past batch limit
#[test]
fn test_adversarial_gpu_gym_buffer_queuing_overflow() {
    let mut gym = GpuGym::init().expect("GPU Gym init failed");
    let sample = Sample::new();
    
    // Submit up to limit (4096) + 10 items to trigger auto-process
    let limit = gym.limit();
    assert_eq!(limit, 4096);
    
    for i in 0..(limit + 10) {
        assert!(gym.submit(&sample).is_ok(), "Submit failed at index {}", i);
    }
    
    // After 4106 submits: limit 4096 was reached, triggering 1 auto-process. Remaining in batch should be 10.
    assert_eq!(gym.batch().count(), 10, "Batch count after overflow auto-process should be 10");
    assert_eq!(gym.count(), 4096, "Processed count should be 4096");
    
    // Manual flush should clear remaining 10
    let flushed = gym.flush().expect("Flush failed");
    assert_eq!(flushed, 10);
    assert_eq!(gym.count(), 4106);
    assert_eq!(gym.batch().count(), 0);
}

/// 5. EMPIRICAL TEST: Learn Gym concurrency & race condition stress test
#[test]
fn test_adversarial_learn_gym_concurrency() {
    let gym = Arc::new(LearnGym::new());
    
    // Test double spawn
    let spawned_first = gym.spawn();
    assert!(spawned_first, "First spawn should succeed");
    
    let spawned_second = gym.spawn();
    assert!(!spawned_second, "Second spawn must return false (already active)");
    
    // Launch 8 concurrent threads hammering tune(), status(), live(), matches()
    let mut handles = Vec::new();
    for _thread_idx in 0..8 {
        let gym_clone = gym.clone();
        let handle = thread::spawn(move || {
            for step in 0..100 {
                let d = (step % 13) as u8 + 4;
                gym_clone.tune(d);
                let st = gym_clone.status();
                assert!(st.depth >= 4 && st.depth <= 16);
                let _live = gym_clone.live();
                let _matches = gym_clone.matches();
                thread::sleep(Duration::from_micros(50));
            }
        });
        handles.push(handle);
    }
    
    // Wait for all hammering threads to complete
    for handle in handles {
        handle.join().expect("Hammering thread panicked");
    }
    
    // Stop the gym thread loop cleanly
    gym.stop();
    
    // Wait briefly for background thread to exit cleanly
    thread::sleep(Duration::from_millis(200));
    assert_eq!(gym.status().active, 0, "Gym should be inactive after stop()");
}

/// 6. EMPIRICAL TEST: Learn Gym depth curriculum clamping & tuning
#[test]
fn test_adversarial_learn_gym_depth_clamping() {
    let gym = LearnGym::new();
    
    // Underflow depth (3 -> clamped to 4)
    gym.tune(3);
    assert_eq!(gym.custom.load(std::sync::atomic::Ordering::Relaxed), 4);
    
    // Overflow depth (20 -> clamped to 16)
    gym.tune(20);
    assert_eq!(gym.custom.load(std::sync::atomic::Ordering::Relaxed), 16);
    
    // Valid depth (12 -> set to 12)
    gym.tune(12);
    assert_eq!(gym.custom.load(std::sync::atomic::Ordering::Relaxed), 12);
}

/// 7. EMPIRICAL TEST: Continuous heavy stress test submitting 100,000 samples (>4096) without panics or sample drops
#[test]
fn test_adversarial_gpu_gym_continuous_heavy_samples() {
    let mut gym = GpuGym::init().expect("GPU Gym init failed");
    let sample = Sample::new();
    let total_samples = 100_000usize;
    
    for i in 0..total_samples {
        assert!(gym.submit(&sample).is_ok(), "Submit failed at index {}", i);
    }
    
    // Remaining in batch should be 100_000 % 4096 = 1696
    let remaining = gym.batch().count();
    assert_eq!(remaining, total_samples % 4096, "Batch count must match modulo of limit");
    
    // Flush remaining samples
    let flushed = gym.flush().expect("Flush failed");
    assert_eq!(flushed, remaining);
    
    // Total count processed must equal 100,000 exactly
    assert_eq!(gym.count(), total_samples, "Total processed samples must equal exactly total_samples submitted");
    assert_eq!(gym.batch().count(), 0, "Batch must be empty after flush");
}

