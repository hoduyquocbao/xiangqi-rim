// ============================================================================
// XIANGTI ENGINE: MPSC & MPMC STRESS HARNESS FOR LOCK-FREE RING BUFFER QUEUE
// ============================================================================
// Empirical stress testing for Buffer lock-free queue in src/gpu/buffer.rs:
// 1. MPSC (Multi-Producer Single-Consumer) concurrent batch pushing & pulling.
// 2. MPMC (Multi-Producer Multi-Consumer) concurrent batch pushing & pulling.
// 3. Boundary modulo wrapping past capacity (push/pull > 100,000 batches).
// 4. Data payload integrity & corruption detection (checksum, pattern, alignment).
// ============================================================================

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use xiangrust::gpu::{Buffer, Status, Storable};

/// Helper: Build a tagged 64-byte position payload with embedded checksum
fn create_payload(producer_id: u32, seq: u32) -> [u8; 64] {
    let mut data = [0u8; 64];
    let pid_bytes = producer_id.to_le_bytes();
    let seq_bytes = seq.to_le_bytes();
    let checksum = producer_id ^ seq ^ 0xDEADBEEF;
    let chk_bytes = checksum.to_le_bytes();

    data[0..4].copy_from_slice(&pid_bytes);
    data[4..8].copy_from_slice(&seq_bytes);
    data[8..12].copy_from_slice(&chk_bytes);

    // Fill remaining bytes with deterministic pattern
    for i in 12..64 {
        data[i] = ((producer_id as u8).wrapping_add(seq as u8)).wrapping_add(i as u8);
    }
    data
}

/// Helper: Verify payload integrity (returns true if valid, false if corrupted)
fn verify_payload(data: &[u8]) -> Result<(u32, u32), String> {
    if data.len() < 64 {
        return Err(format!("Invalid length: {}", data.len()));
    }
    let pid = u32::from_le_bytes(data[0..4].try_into().unwrap());
    let seq = u32::from_le_bytes(data[4..8].try_into().unwrap());
    let chk = u32::from_le_bytes(data[8..12].try_into().unwrap());

    let expected_chk = pid ^ seq ^ 0xDEADBEEF;
    if chk != expected_chk {
        return Err(format!(
            "Checksum failure! pid={}, seq={}, expected_chk=0x{:X}, got_chk=0x{:X}",
            pid, seq, expected_chk, chk
        ));
    }

    for i in 12..64 {
        let expected_byte = ((pid as u8).wrapping_add(seq as u8)).wrapping_add(i as u8);
        if data[i] != expected_byte {
            return Err(format!(
                "Byte corruption at offset {}! pid={}, seq={}, expected=0x{:02X}, got=0x{:02X}",
                i, pid, seq, expected_byte, data[i]
            ));
        }
    }

    Ok((pid, seq))
}

/// Test 1: Single-Threaded Wrap-Around Boundary Test
/// Pushes and pulls 50,000 batches through a 256-byte buffer (capacity = 256).
/// Forces tail & head to wrap modulo 256 over 12,000 times!
#[test]
fn stress_test_single_thread_boundary_wrapping() {
    let capacity = 256; // Holds 4 batches of 64 bytes
    let buf = Buffer::allocate(capacity, false).expect("Buffer allocation failed");
    let total_batches = 50_000;

    let mut output = [0u8; 64];

    for seq in 0..total_batches {
        let payload = create_payload(1, seq);
        let push_res = buf.push(&payload);
        assert!(
            push_res.is_ok(),
            "Push failed at seq {} with status {:?}",
            seq,
            push_res
        );

        let pull_res = buf.pull(&mut output);
        assert!(
            pull_res.is_ok(),
            "Pull failed at seq {} with status {:?}",
            seq,
            pull_res
        );

        let (pid, pulled_seq) = verify_payload(&output).expect("Payload corruption detected");
        assert_eq!(pid, 1);
        assert_eq!(pulled_seq, seq);
    }
}

/// Test 2: MPSC (Multi-Producer Single-Consumer) Stress Test
/// 4 Producers, 1 Consumer, Buffer Capacity = 1024 bytes (16 slots of 64 bytes).
/// Total items pushed = 20,000 (5,000 per producer).
#[test]
fn stress_test_mpsc_concurrent_ring_buffer() {
    let capacity = 1024;
    let buf = Arc::new(Buffer::allocate(capacity, false).expect("Buffer allocation failed"));
    let num_producers = 4;
    let items_per_producer = 5_000;
    let total_expected_items = num_producers * items_per_producer;

    let producers_done = Arc::new(AtomicUsize::new(0));
    let corrupted_count = Arc::new(AtomicU64::new(0));
    let valid_pulled_count = Arc::new(AtomicU64::new(0));

    let start_time = Instant::now();

    // Spawn 4 Producer threads
    let mut handles = vec![];
    for pid in 0..num_producers {
        let b = Arc::clone(&buf);
        let pdone = Arc::clone(&producers_done);
        handles.push(thread::spawn(move || {
            for seq in 0..items_per_producer {
                let payload = create_payload(pid as u32, seq as u32);
                loop {
                    match b.push(&payload) {
                        Ok(_) => break,
                        Err(Status::Full) => thread::yield_now(),
                        Err(e) => panic!("Producer {} push returned unexpected error: {:?}", pid, e),
                    }
                }
            }
            pdone.fetch_add(1, Ordering::SeqCst);
        }));
    }

    // Spawn 1 Consumer thread
    let b_cons = Arc::clone(&buf);
    let pdone_cons = Arc::clone(&producers_done);
    let corr_cons = Arc::clone(&corrupted_count);
    let valid_cons = Arc::clone(&valid_pulled_count);

    let consumer_handle = thread::spawn(move || {
        let mut dest = [0u8; 64];
        loop {
            match b_cons.pull(&mut dest) {
                Ok(_) => {
                    match verify_payload(&dest) {
                        Ok(_) => {
                            valid_cons.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(err) => {
                            corr_cons.fetch_add(1, Ordering::Relaxed);
                            eprintln!("[MPSC CORRUPTION] {}", err);
                        }
                    }
                }
                Err(Status::Ready) => {
                    if pdone_cons.load(Ordering::SeqCst) == num_producers {
                        // Double check if any remaining items exist
                        let mut drain_dest = [0u8; 64];
                        while b_cons.pull(&mut drain_dest).is_ok() {
                            if verify_payload(&drain_dest).is_ok() {
                                valid_cons.fetch_add(1, Ordering::Relaxed);
                            } else {
                                corr_cons.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                        break;
                    }
                    thread::yield_now();
                }
                Err(e) => panic!("Consumer pull returned unexpected error: {:?}", e),
            }
        }
    });

    for h in handles {
        h.join().unwrap();
    }
    consumer_handle.join().unwrap();

    let duration = start_time.elapsed();
    let total_pulled = valid_pulled_count.load(Ordering::SeqCst);
    let total_corrupted = corrupted_count.load(Ordering::SeqCst);

    println!(
        "MPSC Benchmark: Pulled {}/{} items in {:?} (Corrupted: {})",
        total_pulled, total_expected_items, duration, total_corrupted
    );

    assert_eq!(
        total_corrupted, 0,
        "MPSC Ring Buffer suffered {} data payload corruptions!",
        total_corrupted
    );
    assert_eq!(
        total_pulled, total_expected_items as u64,
        "MPSC Ring Buffer lost items! Expected {}, got {}",
        total_expected_items, total_pulled
    );
}

/// Test 3: MPMC (Multi-Producer Multi-Consumer) Stress Test
/// 4 Producers, 4 Consumers, Buffer Capacity = 2048 bytes (32 slots of 64 bytes).
/// Total items pushed = 20,000 (5,000 per producer).
#[test]
fn stress_test_mpmc_concurrent_ring_buffer() {
    let capacity = 2048;
    let buf = Arc::new(Buffer::allocate(capacity, false).expect("Buffer allocation failed"));
    let num_producers = 4;
    let num_consumers = 4;
    let items_per_producer = 5_000;
    let total_expected_items = (num_producers * items_per_producer) as u64;

    let producers_done = Arc::new(AtomicUsize::new(0));
    let corrupted_count = Arc::new(AtomicU64::new(0));
    let valid_pulled_count = Arc::new(AtomicU64::new(0));

    let start_time = Instant::now();

    // Spawn Producers
    let mut prod_handles = vec![];
    for pid in 0..num_producers {
        let b = Arc::clone(&buf);
        let pdone = Arc::clone(&producers_done);
        prod_handles.push(thread::spawn(move || {
            for seq in 0..items_per_producer {
                let payload = create_payload(pid as u32, seq as u32);
                loop {
                    match b.push(&payload) {
                        Ok(_) => break,
                        Err(Status::Full) => thread::yield_now(),
                        Err(e) => panic!("Producer {} push failed: {:?}", pid, e),
                    }
                }
            }
            pdone.fetch_add(1, Ordering::SeqCst);
        }));
    }

    // Spawn Consumers
    let mut cons_handles = vec![];
    for cid in 0..num_consumers {
        let b = Arc::clone(&buf);
        let pdone = Arc::clone(&producers_done);
        let corr = Arc::clone(&corrupted_count);
        let valid = Arc::clone(&valid_pulled_count);

        cons_handles.push(thread::spawn(move || {
            let mut dest = [0u8; 64];
            loop {
                match b.pull(&mut dest) {
                    Ok(_) => {
                        match verify_payload(&dest) {
                            Ok(_) => {
                                valid.fetch_add(1, Ordering::Relaxed);
                            }
                            Err(err) => {
                                corr.fetch_add(1, Ordering::Relaxed);
                                eprintln!("[MPMC Consumer {} CORRUPTION] {}", cid, err);
                            }
                        }
                    }
                    Err(Status::Ready) => {
                        if pdone.load(Ordering::SeqCst) == num_producers {
                            // Drain remaining
                            let mut drain_dest = [0u8; 64];
                            while b.pull(&mut drain_dest).is_ok() {
                                if verify_payload(&drain_dest).is_ok() {
                                    valid.fetch_add(1, Ordering::Relaxed);
                                } else {
                                    corr.fetch_add(1, Ordering::Relaxed);
                                }
                            }
                            break;
                        }
                        thread::yield_now();
                    }
                    Err(e) => panic!("Consumer {} pull failed: {:?}", cid, e),
                }
            }
        }));
    }

    for h in prod_handles {
        h.join().unwrap();
    }
    for h in cons_handles {
        h.join().unwrap();
    }

    let duration = start_time.elapsed();
    let total_pulled = valid_pulled_count.load(Ordering::SeqCst);
    let total_corrupted = corrupted_count.load(Ordering::SeqCst);

    println!(
        "MPMC Benchmark: Pulled {}/{} items in {:?} (Corrupted: {})",
        total_pulled, total_expected_items, duration, total_corrupted
    );

    assert_eq!(
        total_corrupted, 0,
        "MPMC Ring Buffer suffered {} data payload corruptions!",
        total_corrupted
    );
    assert_eq!(
        total_pulled, total_expected_items,
        "MPMC Ring Buffer lost items! Expected {}, got {}",
        total_expected_items, total_pulled
    );
}

/// Test 4: Queue Lockup & Boundary Capacity Overflows
/// Rapidly push & pull batches when boundary exceeds `capacity`.
/// Tests 100,000 pushes into a tiny 128-byte buffer (capacity = 128).
#[test]
fn stress_test_tiny_capacity_overflow_boundary() {
    let capacity = 128; // Fits exactly 2 batches of 64 bytes
    let buf = Arc::new(Buffer::allocate(capacity, false).expect("Buffer allocation failed"));
    let stop = Arc::new(AtomicBool::new(false));
    let total_pushes = 50_000;

    let b_prod = Arc::clone(&buf);
    let stop_prod = Arc::clone(&stop);

    let producer = thread::spawn(move || {
        for seq in 0..total_pushes {
            let payload = create_payload(99, seq);
            while b_prod.push(&payload).is_err() {
                thread::yield_now();
            }
        }
        stop_prod.store(true, Ordering::SeqCst);
    });

    let b_cons = Arc::clone(&buf);
    let stop_cons = Arc::clone(&stop);
    let mut pulled_count = 0u64;
    let mut corrupted = 0u64;

    let consumer = thread::spawn(move || {
        let mut dest = [0u8; 64];
        loop {
            match b_cons.pull(&mut dest) {
                Ok(_) => {
                    pulled_count += 1;
                    if let Err(e) = verify_payload(&dest) {
                        corrupted += 1;
                        eprintln!("[TINY BUFFER CORRUPTION] {}", e);
                    }
                }
                Err(_) => {
                    if stop_cons.load(Ordering::SeqCst) {
                        let mut drain = [0u8; 64];
                        while b_cons.pull(&mut drain).is_ok() {
                            pulled_count += 1;
                            if let Err(e) = verify_payload(&drain) {
                                corrupted += 1;
                                eprintln!("[TINY BUFFER DRAIN CORRUPTION] {}", e);
                            }
                        }
                        break;
                    }
                    thread::yield_now();
                }
            }
        }
        (pulled_count, corrupted)
    });

    producer.join().unwrap();
    let (pulled, corr) = consumer.join().unwrap();

    assert_eq!(corr, 0, "Tiny buffer wrapping caused {} corruptions", corr);
    assert_eq!(
        pulled, total_pushes as u64,
        "Tiny buffer wrapping lost items! Pushed {}, Pulled {}",
        total_pushes, pulled
    );
}
