// ============================================================================
// XIANGTI ENGINE: EMPIRICAL STRESS TEST FOR VRAM GUARD & ALIGNMENT (M1)
// ============================================================================
// Stress tests for Milestone M1:
// 1. Boundary allocations near 409.6MB ceiling (429,496,729 bytes) & Status::Exhausted
// 2. Zero-byte allocations and releases
// 3. Multi-threaded concurrent reserve and release calls (testing atomic safety)
// 4. 64-byte physical pointer alignment verification across multiple buffer sizes
// 5. Concurrent Buffer push data race verification
// ============================================================================

use std::sync::Arc;
use std::thread;
use xiangrust::gpu::{Buffer, Guard, Status, Storable, Validatable};

/// Test 1: Boundary allocations near 409.6MB (429,496,729 bytes)
#[test]
fn stress_test_boundary_allocations_near_ceiling() {
    let guard = Guard::new();
    let ceiling = guard.ceiling(); // 429,496,729 bytes (409.6 MB)

    // 1a. Reserve exact ceiling byte count
    let res = guard.reserve(ceiling);
    assert!(res.is_ok(), "Reserving exact ceiling (409.6MB) should succeed");
    assert_eq!(guard.allocated(), ceiling);
    assert_eq!(guard.status(), Status::Exhausted);

    // 1b. Attempting even 1 more byte must fail with Status::Exhausted
    let err = guard.reserve(1);
    assert_eq!(err.err(), Some(Status::Exhausted), "Over-allocation by 1 byte must return Status::Exhausted");
    assert_eq!(guard.allocated(), ceiling, "Allocation counter must not change on failure");

    // 1c. Release and verify recovery
    guard.release(ceiling);
    assert_eq!(guard.allocated(), 0);
    assert_eq!(guard.status(), Status::Ready);

    // 1d. Stepwise allocation reaching boundary
    let chunk = 100 * 1024 * 1024; // 100 MB
    assert!(guard.reserve(chunk).is_ok()); // 100 MB
    assert!(guard.reserve(chunk).is_ok()); // 200 MB
    assert!(guard.reserve(chunk).is_ok()); // 300 MB
    assert!(guard.reserve(chunk).is_ok()); // 400 MB

    let remaining = ceiling - (400 * 1024 * 1024); // 9,496,729 bytes
    assert!(guard.reserve(remaining).is_ok(), "Reserving remaining bytes up to ceiling should succeed");
    assert_eq!(guard.allocated(), ceiling);

    // Any further allocation must be rejected
    assert_eq!(guard.reserve(1).err(), Some(Status::Exhausted));
    assert_eq!(guard.reserve(1024).err(), Some(Status::Exhausted));

    guard.wipe();
    assert_eq!(guard.allocated(), 0);
}

/// Test 2: Zero-byte allocations and releases
#[test]
fn stress_test_zero_byte_allocations() {
    let guard = Guard::new();

    // 2a. Guard reserve zero bytes -> must return Status::Fault
    let res = guard.reserve(0);
    assert_eq!(res.err(), Some(Status::Fault), "Reserving 0 bytes must return Status::Fault");
    assert_eq!(guard.allocated(), 0);

    // 2b. Guard release zero bytes -> should be a safe no-op
    guard.release(0);
    assert_eq!(guard.allocated(), 0);
    assert_eq!(guard.count(), 0);

    // 2c. Buffer allocate zero bytes -> must return Status::Fault
    let buf_res = Buffer::allocate(0, true);
    assert_eq!(buf_res.err(), Some(Status::Fault), "Allocating 0-byte buffer must return Status::Fault");

    // 2d. Guard chunks for 0 bytes -> 0
    assert_eq!(guard.chunks(0), 0);
}

/// Test 3: Multi-threaded concurrent reserve and release calls
#[test]
fn stress_test_multithreaded_concurrency() {
    let guard = Arc::new(Guard::new());
    let threads = 16;
    let iterations = 1_000;
    let alloc_size = 10 * 1024 * 1024; // 10 MB per reserve

    let mut handles = vec![];

    for _ in 0..threads {
        let guard_clone = Arc::clone(&guard);
        let handle = thread::spawn(move || {
            for _ in 0..iterations {
                if let Ok(_) = guard_clone.reserve(alloc_size) {
                    thread::yield_now();
                    guard_clone.release(alloc_size);
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // After all threads finish reserve + release pairs, total allocated should be 0
    let final_allocated = guard.allocated();
    let final_count = guard.count();
    
    assert_eq!(
        final_allocated, 0,
        "Concurrent reserve/release leak/underflow! Final allocated bytes = {}",
        final_allocated
    );
    assert_eq!(
        final_count, 0,
        "Concurrent reserve/release count underflow! Final count = {}",
        final_count
    );
}

/// Test 4: 64-byte physical pointer alignment verification
#[test]
fn stress_test_raw_pointer_64byte_alignment() {
    let test_sizes = vec![1, 15, 16, 31, 32, 63, 64, 65, 127, 128, 1023, 1024, 1_000_000];

    for size in test_sizes {
        let buffer = Buffer::allocate(size, true).expect(&format!("Allocation failed for size {}", size));
        let ptr_addr = buffer.pointer() as usize;

        assert_eq!(
            ptr_addr % 64, 0,
            "Buffer pointer {:p} for size {} is NOT 64-byte aligned (offset = {})",
            buffer.pointer(), size, ptr_addr % 64
        );
        assert!(buffer.aligned(), "Buffer aligned flag must be true");
        assert!(buffer.capacity() >= size, "Buffer capacity must be >= size");
        assert_eq!(buffer.capacity() % 64, 0, "Buffer capacity must be a multiple of 64 bytes");
    }
}

/// Test 5: Validate Status::Exhausted when exceeding ceiling (409.6MB)
#[test]
fn stress_test_status_exhausted_on_overflow() {
    let guard = Guard::new();

    // Allocate almost up to ceiling (400MB)
    let safe_alloc = 400 * 1024 * 1024;
    assert!(guard.reserve(safe_alloc).is_ok());

    // Requesting another 10MB will push total to 410MB (> 409.6MB ceiling)
    let overflow_alloc = 10 * 1024 * 1024;
    let res = guard.reserve(overflow_alloc);
    assert_eq!(res.err(), Some(Status::Exhausted), "Must return Status::Exhausted when exceeding ceiling");

    // Guard allocated must remain 400MB (rolled back)
    assert_eq!(guard.allocated(), safe_alloc);

    // Status check
    assert_eq!(guard.status(), Status::Active); // still < 409.6MB overall
}

/// Test 6: Validate Trait implementation for Guard (Validatable)
#[test]
fn stress_test_guard_validatable_trait() {
    let guard = Guard::new();
    assert_eq!(guard.validate(100 * 1024 * 1024), Status::Ready);
    assert_eq!(guard.validate(409 * 1024 * 1024), Status::Ready); // <= 409.6MB
    assert_eq!(guard.validate(410 * 1024 * 1024), Status::Full);  // 409.6MB < target <= 512MB
    assert_eq!(guard.validate(512 * 1024 * 1024), Status::Full);
    assert_eq!(guard.validate(513 * 1024 * 1024), Status::Fail);  // > 512MB
}

/// Test 7: Race condition check on concurrent Guard over-release
#[test]
fn stress_test_guard_concurrent_release_safety() {
    let guard = Arc::new(Guard::new());
    guard.reserve(100).expect("Initial reserve failed");
    assert_eq!(guard.allocated(), 100);
    assert_eq!(guard.count(), 1);

    let g1 = Arc::clone(&guard);
    let g2 = Arc::clone(&guard);

    let h1 = thread::spawn(move || g1.release(100));
    let h2 = thread::spawn(move || g2.release(100));

    h1.join().unwrap();
    h2.join().unwrap();

    // Guard must not wrap allocated or count to usize::MAX
    assert!(
        guard.allocated() <= 100,
        "Guard allocated underflowed! Value: {}", guard.allocated()
    );
    assert!(
        guard.count() <= 1,
        "Guard count underflowed! Value: {}", guard.count()
    );
}

/// Test 8: Concurrent Buffer push stress test
#[test]
fn stress_test_concurrent_buffer_push() {
    let buf = Arc::new(Buffer::allocate(1024 * 1024, true).unwrap());
    let threads = 4;
    let push_per_thread = 100;
    let payload = vec![0xABu8; 64];

    let mut handles = vec![];
    for _ in 0..threads {
        let b = Arc::clone(&buf);
        let p = payload.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..push_per_thread {
                let _ = b.push(&p);
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    // Verify buffer head and tail state
    let mut target = vec![0u8; 64];
    let pull_res = buf.pull(&mut target);
    assert!(pull_res.is_ok(), "Pulling from buffer after concurrent pushes must succeed");
}
