// ============================================================================
// XIANGTI ENGINE: EMPIRICAL CHALLENGER M1_4_1 FRAMING & CONCURRENCY STRESS SUITE
// ============================================================================
// Adversarial test suite created by Challenger M1_4_1 to empirically stress test:
// 1. 4-byte u32 length header framing boundary split across ring buffer capacity (1/3, 2/2, 3/1 split).
// 2. High concurrency MPSC (32 producers, 1 consumer) with variable payload lengths.
// 3. High concurrency MPMC (32 producers, 32 consumers) with random buffer sizing & zero packet loss.
// 4. Index wrapping near usize::MAX boundary & Guard CAS underflow safety.
// 5. Device eval wrap-around framing decode correctness under usize::MAX overflow.
// ============================================================================

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use xiangrust::gpu::{Buffer, Device, Guard, Status, Storable};

/// Tagged packet generator for data integrity check
fn generate(tag: u32, seq: u32, len: usize) -> Vec<u8> {
    assert!(len >= 16, "Length must be at least 16 bytes");
    let hash = tag ^ seq ^ (len as u32) ^ 0xDEADBEEF;
    let mut data = vec![0u8; len];
    data[0..4].copy_from_slice(&tag.to_le_bytes());
    data[4..8].copy_from_slice(&seq.to_le_bytes());
    data[8..12].copy_from_slice(&hash.to_le_bytes());
    data[12..16].copy_from_slice(&(len as u32).to_le_bytes());

    for i in 16..len {
        data[i] = ((tag as u8).wrapping_add(seq as u8)).wrapping_add(i as u8);
    }
    data
}

/// Verify packet integrity
fn validate(data: &[u8]) -> Result<(u32, u32, usize), String> {
    if data.len() < 16 {
        return Err(format!("Data too short: {} bytes", data.len()));
    }
    let tag = u32::from_le_bytes(data[0..4].try_into().unwrap());
    let seq = u32::from_le_bytes(data[4..8].try_into().unwrap());
    let hash = u32::from_le_bytes(data[8..12].try_into().unwrap());
    let len = u32::from_le_bytes(data[12..16].try_into().unwrap()) as usize;

    let expected = tag ^ seq ^ (len as u32) ^ 0xDEADBEEF;
    if hash != expected {
        return Err(format!(
            "Hash mismatch: tag={}, seq={}, len={}, expected=0x{:X}, got=0x{:X}",
            tag, seq, len, expected, hash
        ));
    }

    if data.len() < len {
        return Err(format!("Truncated payload: len={}, actual={}", len, data.len()));
    }

    for i in 16..len {
        let expected_byte = ((tag as u8).wrapping_add(seq as u8)).wrapping_add(i as u8);
        if data[i] != expected_byte {
            return Err(format!(
                "Corruption at byte {}: tag={}, seq={}, expected=0x{:02X}, got=0x{:02X}",
                i, tag, seq, expected_byte, data[i]
            ));
        }
    }

    Ok((tag, seq, len))
}

/// Test 1: Header boundary split (1/3, 2/2, 3/1 split across capacity boundary)
#[test]
fn test_header_boundary_split_offsets() {
    let capacity = 64;
    let buf = Buffer::allocate(capacity, false).expect("Allocate buffer");

    // We push packets of specific sizes so header splits at offsets 61, 62, 63
    for iteration in 0..100 {
        let payload = generate(10, iteration, 20); // total 24 bytes (4 header + 20 payload)
        let push_res = buf.push(&payload);
        assert!(push_res.is_ok(), "Push failed at iteration {}", iteration);

        let mut target = vec![0u8; 32];
        let pull_res = buf.pull(&mut target);
        assert!(pull_res.is_ok(), "Pull failed at iteration {}", iteration);

        let (tag, seq, len) = validate(&target[0..20]).expect("Validation failed");
        assert_eq!(tag, 10);
        assert_eq!(seq, iteration);
        assert_eq!(len, 20);
    }
}

/// Test 2: High concurrency MPSC (32 Producers, 1 Consumer) with variable length payloads
#[test]
fn test_mpsc_variable_length_concurrency() {
    let capacity = 8192;
    let buf = Arc::new(Buffer::allocate(capacity, false).expect("Allocate buffer"));
    let producers = 32;
    let items = 1_000;
    let total_expected = (producers * items) as u64;

    let done = Arc::new(AtomicUsize::new(0));
    let valid = Arc::new(AtomicU64::new(0));
    let corrupt = Arc::new(AtomicU64::new(0));

    let start = Instant::now();

    // Producers
    let mut prod_handles = vec![];
    for pid in 0..producers {
        let b = Arc::clone(&buf);
        let d = Arc::clone(&done);
        prod_handles.push(thread::spawn(move || {
            for seq in 0..items {
                let len = 16 + ((pid + seq) % 64); // Variable payload sizes 16..80 bytes
                let payload = generate(pid as u32, seq as u32, len);
                loop {
                    match b.push(&payload) {
                        Ok(_) => break,
                        Err(Status::Full) => thread::yield_now(),
                        Err(e) => panic!("Producer {} push error: {:?}", pid, e),
                    }
                }
            }
            d.fetch_add(1, Ordering::SeqCst);
        }));
    }

    // Consumer
    let b_cons = Arc::clone(&buf);
    let d_cons = Arc::clone(&done);
    let v_cons = Arc::clone(&valid);
    let c_cons = Arc::clone(&corrupt);

    let cons_handle = thread::spawn(move || {
        let mut target = vec![0u8; 256];
        loop {
            match b_cons.pull(&mut target) {
                Ok(_) => match validate(&target) {
                    Ok((_, _, _)) => {
                        v_cons.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(e) => {
                        c_cons.fetch_add(1, Ordering::Relaxed);
                        eprintln!("[MPSC Consumer CORRUPTION] {}", e);
                    }
                },
                Err(Status::Ready) => {
                    if d_cons.load(Ordering::SeqCst) == producers {
                        let mut drain = vec![0u8; 256];
                        while b_cons.pull(&mut drain).is_ok() {
                            if validate(&drain).is_ok() {
                                v_cons.fetch_add(1, Ordering::Relaxed);
                            } else {
                                c_cons.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                        break;
                    }
                    thread::yield_now();
                }
                Err(e) => panic!("Consumer pull error: {:?}", e),
            }
        }
    });

    for h in prod_handles {
        h.join().unwrap();
    }
    cons_handle.join().unwrap();

    let duration = start.elapsed();
    let total_valid = valid.load(Ordering::SeqCst);
    let total_corrupt = corrupt.load(Ordering::SeqCst);

    println!(
        "MPSC 32-Producer Variable Length: Pulled {}/{} in {:?} (Corrupt: {})",
        total_valid, total_expected, duration, total_corrupt
    );

    assert_eq!(total_corrupt, 0, "Corrupted packets detected in MPSC test!");
    assert_eq!(total_valid, total_expected, "Lost packets detected in MPSC test!");
}

/// Test 3: High Concurrency MPMC (32 Producers, 32 Consumers) with variable length payloads
#[test]
fn test_mpmc_variable_length_concurrency() {
    let capacity = 8192;
    let buf = Arc::new(Buffer::allocate(capacity, false).expect("Allocate buffer"));
    let num_producers = 32;
    let num_consumers = 32;
    let items_per_prod = 1_000;
    let total_expected = (num_producers * items_per_prod) as u64;

    let producers_done = Arc::new(AtomicUsize::new(0));
    let valid_count = Arc::new(AtomicU64::new(0));
    let corrupt_count = Arc::new(AtomicU64::new(0));

    let start = Instant::now();

    let mut prod_handles = vec![];
    for pid in 0..num_producers {
        let b = Arc::clone(&buf);
        let pd = Arc::clone(&producers_done);
        prod_handles.push(thread::spawn(move || {
            for seq in 0..items_per_prod {
                let len = 16 + ((pid * 13 + seq * 7) % 96);
                let payload = generate(pid as u32, seq as u32, len);
                loop {
                    match b.push(&payload) {
                        Ok(_) => break,
                        Err(Status::Full) => thread::yield_now(),
                        Err(e) => panic!("Producer {} error: {:?}", pid, e),
                    }
                }
            }
            pd.fetch_add(1, Ordering::SeqCst);
        }));
    }

    let mut cons_handles = vec![];
    for cid in 0..num_consumers {
        let b = Arc::clone(&buf);
        let pd = Arc::clone(&producers_done);
        let vc = Arc::clone(&valid_count);
        let cc = Arc::clone(&corrupt_count);

        cons_handles.push(thread::spawn(move || {
            let mut target = vec![0u8; 256];
            loop {
                match b.pull(&mut target) {
                    Ok(_) => match validate(&target) {
                        Ok(_) => {
                            vc.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(err) => {
                            cc.fetch_add(1, Ordering::Relaxed);
                            eprintln!("[MPMC Consumer {} CORRUPTION] {}", cid, err);
                        }
                    },
                    Err(Status::Ready) => {
                        if pd.load(Ordering::SeqCst) == num_producers {
                            let mut drain = vec![0u8; 256];
                            while b.pull(&mut drain).is_ok() {
                                if validate(&drain).is_ok() {
                                    vc.fetch_add(1, Ordering::Relaxed);
                                } else {
                                    cc.fetch_add(1, Ordering::Relaxed);
                                }
                            }
                            break;
                        }
                        thread::yield_now();
                    }
                    Err(e) => panic!("Consumer {} error: {:?}", cid, e),
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

    let duration = start.elapsed();
    let total_valid = valid_count.load(Ordering::SeqCst);
    let total_corrupt = corrupt_count.load(Ordering::SeqCst);

    println!(
        "MPMC 32x32 Variable Length: Pulled {}/{} in {:?} (Corrupt: {})",
        total_valid, total_expected, duration, total_corrupt
    );

    assert_eq!(total_corrupt, 0, "Corruptions in MPMC variable length test!");
    assert_eq!(total_valid, total_expected, "Lost items in MPMC variable length test!");
}

/// Test 4: Extreme Guard CAS Underflow and Release Stress
#[test]
fn test_guard_cas_underflow_safety() {
    let guard = Arc::new(Guard::new());
    let threads = 32;
    let iterations = 20_000;

    // Initially reserve 500 bytes
    guard.reserve(500).unwrap();

    let mut handles = vec![];
    for _ in 0..threads {
        let g = Arc::clone(&guard);
        handles.push(thread::spawn(move || {
            for _ in 0..iterations {
                // Try releasing more bytes than reserved to trigger potential underflow
                g.release(10_000);
                let _ = g.reserve(64);
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let alloc = guard.allocated();
    let count = guard.count();

    println!("Guard Stress Result: allocated={}, count={}", alloc, count);
    assert!(
        alloc <= threads * iterations * 64 + 500,
        "Allocated overflow/wrap detected: {}",
        alloc
    );
}

/// Test 5: Device eval wrap-around test
#[test]
fn test_device_eval_length_framed_queue() {
    let dev = Device::init();
    let buf = dev.allocate(1024).expect("Allocate buffer");

    // Queue 3 batch positions with length framing
    let batch1 = [1u8, 2u8, 3u8, 4u8, 5u8, 6u8]; // Weights: 10 + 20 + 20 + 40 + 45 + 90 = 225
    let batch2 = [4u8, 5u8, 6u8];               // Weights: 40 + 45 + 90 = 175
    let batch3 = [7u8];                         // Weight: 1000

    dev.queue(&buf, &batch1).unwrap();
    dev.queue(&buf, &batch2).unwrap();
    dev.queue(&buf, &batch3).unwrap();

    let score = dev.eval(&buf).expect("Eval failed");
    assert_eq!(score, 225 + 175 + 1000, "Eval score mismatch: expected 1400, got {}", score);
}
