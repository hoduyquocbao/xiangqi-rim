// ============================================================================
// EMPIRICAL STRESS HARNESS FOR MILESTONE M2 ITERATION 1: GPU KERNEL
// ============================================================================
// Stress tests `Kernel` dispatching, atomic counters (`dispatch`, `finish`),
// threadgroup size scaling (`threads`), non-blocking asynchronous execution,
// score retrieval in `finish`, and CPU SIMD fallback execution under multi-threaded contention.
// ============================================================================

use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::atomic::{AtomicUsize, Ordering};
use xiangrust::gpu::kernel::{Kernel, Dispatchable};
use xiangrust::gpu::buffer::Buffer;
use xiangrust::gpu::status::Status;

#[test]
fn test_kernel_struct_memory_layout_and_alignment() {
    // Verify 64-byte hardware alignment and exact 64-byte size to prevent false sharing
    assert_eq!(std::mem::size_of::<Kernel>(), 64, "Kernel size must be exactly 64 bytes");
    assert_eq!(std::mem::align_of::<Kernel>(), 64, "Kernel alignment must be 64 bytes");
}

#[test]
fn test_kernel_init_parameter_validation() {
    // 1. Zero limit should fail
    assert_eq!(Kernel::init(0, 420, 256).err(), Some(Status::Fault));
    // 2. Zero stride should fail
    assert_eq!(Kernel::init(100, 0, 256).err(), Some(Status::Fault));
    // 3. Zero threads should fail
    assert_eq!(Kernel::init(100, 420, 0).err(), Some(Status::Fault));

    // 4. Valid init
    let kernel = Kernel::init(1000, 128, 256).expect("Valid Kernel init failed");
    assert_eq!(kernel.status(), Status::Ready);
    assert!(kernel.ready());
    assert!(!kernel.active());
    assert_eq!(kernel.limit(), 1000);
    assert_eq!(kernel.stride(), 128);
    assert_eq!(kernel.threads(), 256);
    assert_eq!(kernel.batch(), 0);
}

#[test]
fn test_kernel_dispatch_boundary_and_auto_execute() {
    let buffer = Buffer::allocate(65536, false).expect("Buffer allocation failed");
    let mut kernel = Kernel::init(100, 128, 64).expect("Kernel init failed");

    // Zero count dispatch should fail with Fault
    assert_eq!(kernel.dispatch(&buffer, 0), Err(Status::Fault));

    // Over-limit single dispatch should fail with Full
    assert_eq!(kernel.dispatch(&buffer, 150), Err(Status::Full));

    // Incremental dispatches
    let res1 = kernel.dispatch(&buffer, 40).unwrap();
    assert_eq!(res1, 40);
    assert_eq!(kernel.batch(), 40);

    let res2 = kernel.dispatch(&buffer, 50).unwrap();
    assert_eq!(res2, 90);
    assert_eq!(kernel.batch(), 90);

    // Dispatching remaining 10 triggers auto-execute (batch reaches limit=100)
    let res3 = kernel.dispatch(&buffer, 10).unwrap();
    // After auto-execute, batch is reset to 0 in execute()
    assert_eq!(res3, 0, "Dispatch auto-execute should reset batch to 0");
    assert_eq!(kernel.batch(), 0);
}

#[test]
fn test_kernel_atomic_counters_accrual_and_reset() {
    let buffer = Buffer::allocate(65536, false).expect("Buffer allocation failed");
    let mut kernel = Kernel::init(500, 128, 128).expect("Kernel init failed");

    // Dispatch and flush multiple batches
    for _ in 0..10 {
        kernel.dispatch(&buffer, 50).unwrap();
        kernel.flush(&buffer).unwrap();
    }

    // Target check on atomic counters
    // Each loop dispatched 50 items and executed -> total 500 items dispatched & finished
    let scores = &mut [0isize; 600];
    let finished_count = kernel.finish(&buffer, scores).unwrap();
    assert_eq!(finished_count, 500);

    // Reset kernel
    kernel.reset();
    assert_eq!(kernel.batch(), 0);
    assert_eq!(kernel.finish(&buffer, scores).unwrap(), 0);
}

#[test]
fn test_kernel_finish_score_buffer_retrieval_empirical_check() {
    // Empirical check on finish() method behaviour:
    // Does finish() populate scores into target array, or does it only check atomic count?
    let buffer = Buffer::allocate(4096, false).expect("Buffer allocation failed");
    let mut kernel = Kernel::init(10, 128, 32).expect("Kernel init failed");

    kernel.dispatch(&buffer, 5).unwrap();
    kernel.flush(&buffer).unwrap();

    let mut target = [777isize; 10];
    let count = kernel.finish(&buffer, &mut target).unwrap();
    assert_eq!(count, 5);

    // Check if target elements were updated by finish()
    let untouched = target.iter().all(|&x| x == 777);
    println!("Kernel finish target buffer untouched state: {}", untouched);
}

#[test]
fn test_kernel_threadgroup_scaling_stress() {
    let thread_sizes = [1, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096];
    let buffer = Buffer::allocate(32768, false).unwrap();

    for &threads in &thread_sizes {
        let mut kernel = Kernel::init(100, 64, threads).expect("Scaling init failed");
        assert_eq!(kernel.threads(), threads);

        kernel.dispatch(&buffer, 50).unwrap();
        let flushed = kernel.flush(&buffer).unwrap();
        assert_eq!(flushed, 50);
    }
}

#[test]
fn test_kernel_async_polling_and_lifecycle() {
    let buffer = Buffer::allocate(8192, false).unwrap();
    let mut kernel = Kernel::init(200, 64, 128).unwrap();

    assert_eq!(kernel.poll(), Status::Ready);
    assert!(kernel.ready());

    kernel.dispatch(&buffer, 100).unwrap();
    assert_eq!(kernel.poll(), Status::Ready);

    kernel.flush(&buffer).unwrap();
    assert_eq!(kernel.poll(), Status::Ready);
}

#[test]
fn test_kernel_multithreaded_contention_stress() {
    // High-contention multi-threaded stress test: 16 worker threads submitting dispatches
    let thread_count = 16;
    let dispatches_per_thread = 5000; // Total 80,000 items
    let batch_size = 25;

    let buffer = Arc::new(Buffer::allocate(131072, false).unwrap());
    let kernel = Arc::new(Mutex::new(Kernel::init(100, 128, 256).unwrap()));
    let total_dispatched = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();

    for _ in 0..thread_count {
        let buf_clone = Arc::clone(&buffer);
        let ker_clone = Arc::clone(&kernel);
        let total_clone = Arc::clone(&total_dispatched);

        let handle = thread::spawn(move || {
            for _ in 0..(dispatches_per_thread / batch_size) {
                let mut k = ker_clone.lock().unwrap();
                match k.dispatch(&buf_clone, batch_size) {
                    Ok(_) => {
                        total_clone.fetch_add(batch_size, Ordering::SeqCst);
                    }
                    Err(Status::Full) => {
                        k.flush(&buf_clone).unwrap();
                        k.dispatch(&buf_clone, batch_size).unwrap();
                        total_clone.fetch_add(batch_size, Ordering::SeqCst);
                    }
                    Err(e) => panic!("Unexpected error during multithreaded dispatch: {:?}", e),
                }
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    // Flush any remaining items in kernel
    let mut k = kernel.lock().unwrap();
    k.flush(&buffer).unwrap();

    let expected_total = thread_count * dispatches_per_thread;
    let actual_dispatched = total_dispatched.load(Ordering::SeqCst);
    assert_eq!(actual_dispatched, expected_total);

    let mut target = vec![0isize; expected_total + 100];
    let finished_count = k.finish(&buffer, &mut target).unwrap();
    assert_eq!(finished_count, expected_total, "Atomic finish count must match total dispatched work");
}
