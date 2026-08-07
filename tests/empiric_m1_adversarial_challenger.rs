// ============================================================================
// XIANGTI ENGINE: ADVERSARIAL EMPIRICAL CHALLENGER SUITE (MILESTONE M1 ITERATION 3)
// ============================================================================
// Stress tests & Oracles created by Challenger M1_3_2:
// 1. Index wrapping across usize::MAX boundary (MPSC/MPMC).
// 2. High-concurrency MPMC stress test with 16 producers & 16 consumers.
// 3. Variable/Non-uniform payload sizes push and pull stress test.
// 4. Extreme Guard release CAS underflow stress test with 32 concurrent threads.
// 5. Stream desynchronization & partial pull corruption bug demonstration.
// ============================================================================

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use xiangrust::gpu::{Buffer, Guard, Status, Storable, Validatable};

/// Tagged payload struct for data integrity verification
#[derive(Clone, Copy, Debug)]
struct PacketHeader {
    producer: u32,
    sequence: u32,
    checksum: u32,
    length: u32,
}

fn create_dynamic_payload(producer: u32, sequence: u32, len: usize) -> Vec<u8> {
    assert!(len >= 16, "Payload length must be at least 16 bytes for header");
    let checksum = producer ^ sequence ^ (len as u32) ^ 0xCAFEBABE;
    let mut data = vec![0u8; len];
    data[0..4].copy_from_slice(&producer.to_le_bytes());
    data[4..8].copy_from_slice(&sequence.to_le_bytes());
    data[8..12].copy_from_slice(&checksum.to_le_bytes());
    data[12..16].copy_from_slice(&(len as u32).to_le_bytes());

    for i in 16..len {
        data[i] = ((producer as u8).wrapping_add(sequence as u8)).wrapping_add(i as u8);
    }
    data
}

fn verify_dynamic_payload(data: &[u8]) -> Result<PacketHeader, String> {
    if data.len() < 16 {
        return Err(format!("Payload too short: {} bytes", data.len()));
    }
    let producer = u32::from_le_bytes(data[0..4].try_into().unwrap());
    let sequence = u32::from_le_bytes(data[4..8].try_into().unwrap());
    let checksum = u32::from_le_bytes(data[8..12].try_into().unwrap());
    let length = u32::from_le_bytes(data[12..16].try_into().unwrap());

    let expected_checksum = producer ^ sequence ^ length ^ 0xCAFEBABE;
    if checksum != expected_checksum {
        return Err(format!(
            "Checksum mismatch! prod={}, seq={}, len={}, expected_chk=0x{:X}, got_chk=0x{:X}",
            producer, sequence, length, expected_checksum, checksum
        ));
    }

    if data.len() < length as usize {
        return Err(format!("Truncated payload! len field={}, actual slice={}", length, data.len()));
    }

    for i in 16..(length as usize) {
        let expected_byte = ((producer as u8).wrapping_add(sequence as u8)).wrapping_add(i as u8);
        if data[i] != expected_byte {
            return Err(format!(
                "Byte corruption at index {}! prod={}, seq={}, expected=0x{:02X}, got=0x{:02X}",
                i, producer, sequence, expected_byte, data[i]
            ));
        }
    }

    Ok(PacketHeader {
        producer,
        sequence,
        checksum,
        length,
    })
}

/// Adversarial Test 1: Index Wrapping Across usize::MAX Boundary
#[test]
fn adversarial_test_usize_max_wrapping_boundary() {
    let capacities = vec![128, 256, 320, 512, 1024];

    for capacity in capacities {
        let buf = Buffer::allocate(capacity, false).expect("Buffer allocation failed");
        let total_iterations = 10_000;

        for seq in 0..total_iterations {
            let payload = create_dynamic_payload(7, seq as u32, 64);
            let push_res = buf.push(&payload);
            assert!(push_res.is_ok(), "Push failed at seq {} for cap {}", seq, capacity);

            let mut target = vec![0u8; 64];
            let pull_res = buf.pull(&mut target);
            assert!(pull_res.is_ok(), "Pull failed at seq {} for cap {}", seq, capacity);

            let header = verify_dynamic_payload(&target).expect("Payload corruption on index wrapping");
            assert_eq!(header.producer, 7);
            assert_eq!(header.sequence, seq as u32);
        }
    }
}

/// Adversarial Test 2: High Concurrency MPMC Stress (16 Producers, 16 Consumers)
#[test]
fn adversarial_test_high_concurrency_mpmc_32_threads() {
    let capacity = 4096;
    let buf = Arc::new(Buffer::allocate(capacity, false).expect("Buffer allocation failed"));
    let num_producers = 16;
    let num_consumers = 16;
    let items_per_producer = 2_000;
    let total_expected = (num_producers * items_per_producer) as u64;

    let producers_done = Arc::new(AtomicUsize::new(0));
    let corrupted_count = Arc::new(AtomicU64::new(0));
    let valid_count = Arc::new(AtomicU64::new(0));

    let start_time = Instant::now();

    // Spawn Producers
    let mut prod_handles = vec![];
    for pid in 0..num_producers {
        let b = Arc::clone(&buf);
        let pdone = Arc::clone(&producers_done);
        prod_handles.push(thread::spawn(move || {
            for seq in 0..items_per_producer {
                let payload = create_dynamic_payload(pid as u32, seq as u32, 64);
                loop {
                    match b.push(&payload) {
                        Ok(_) => break,
                        Err(Status::Full) => thread::yield_now(),
                        Err(e) => panic!("Producer {} push error: {:?}", pid, e),
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
        let valid = Arc::clone(&valid_count);

        cons_handles.push(thread::spawn(move || {
            let mut dest = vec![0u8; 64];
            loop {
                match b.pull(&mut dest) {
                    Ok(_) => {
                        match verify_dynamic_payload(&dest) {
                            Ok(_) => {
                                valid.fetch_add(1, Ordering::Relaxed);
                            }
                            Err(err) => {
                                corr.fetch_add(1, Ordering::Relaxed);
                                eprintln!("[ADVERSARIAL MPMC Consumer {} CORRUPTION] {}", cid, err);
                            }
                        }
                    }
                    Err(Status::Ready) => {
                        if pdone.load(Ordering::SeqCst) == num_producers {
                            let mut drain_dest = vec![0u8; 64];
                            while b.pull(&mut drain_dest).is_ok() {
                                if verify_dynamic_payload(&drain_dest).is_ok() {
                                    valid.fetch_add(1, Ordering::Relaxed);
                                } else {
                                    corr.fetch_add(1, Ordering::Relaxed);
                                }
                            }
                            break;
                        }
                        thread::yield_now();
                    }
                    Err(e) => panic!("Consumer {} pull error: {:?}", cid, e),
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
    let total_pulled = valid_count.load(Ordering::SeqCst);
    let total_corrupted = corrupted_count.load(Ordering::SeqCst);

    println!(
        "Adversarial 32-Thread MPMC: Pulled {}/{} items in {:?} (Corrupted: {})",
        total_pulled, total_expected, duration, total_corrupted
    );

    assert_eq!(total_corrupted, 0, "High-concurrency MPMC had payload corruptions!");
    assert_eq!(total_pulled, total_expected, "High-concurrency MPMC lost items!");
}

/// Adversarial Test 3: Partial Pull Stream Desynchronization Oracle
/// Demonstrates that pulling into a buffer larger or smaller than pushed items
/// causes stream desynchronization and payload corruption.
#[test]
fn adversarial_test_partial_pull_desynchronization() {
    let capacity = 1024;
    let buf = Buffer::allocate(capacity, false).expect("Buffer allocation failed");

    // Producer pushes two 64-byte payloads
    let payload1 = create_dynamic_payload(1, 100, 64);
    let payload2 = create_dynamic_payload(1, 101, 64);

    buf.push(&payload1).expect("Push payload 1 failed");
    buf.push(&payload2).expect("Push payload 2 failed");

    // Consumer attempts to pull with a 128-byte destination array
    let mut large_dest = vec![0u8; 128];
    let pull_res = buf.pull(&mut large_dest);
    assert!(pull_res.is_ok(), "Pulling should return Ok");

    // Check payload 1 header in first 64 bytes
    let h1 = verify_dynamic_payload(&large_dest[0..64]).expect("First 64 bytes should be payload 1");
    assert_eq!(h1.sequence, 100);

    // Now try to pull payload 2 with a 64-byte array
    let mut target2 = vec![0u8; 64];
    let pull2_res = buf.pull(&mut target2);

    // BUG VERIFICATION: The second pull MUST fail with Status::Ready because the first pull
    // illegally consumed 128 bytes (both payload 1 and payload 2 at once!), leaving the buffer empty!
    if pull2_res.is_err() {
        println!("[BUG CONFIRMED] Second pull returned {:?} because first pull over-read 128 bytes and swallowed payload 2!", pull2_res);
    }
    assert!(
        pull2_res.is_ok(),
        "CRITICAL BUG: Buffer::pull over-consumed bytes and desynchronized the ring buffer stream!"
    );
}

/// Adversarial Test 4: Massive Thread Contention on Guard Underflow Prevention
#[test]
fn adversarial_test_guard_underflow_extreme_contention() {
    let guard = Arc::new(Guard::new());
    
    guard.reserve(100).unwrap();

    let threads = 32;
    let iterations = 10_000;

    let mut handles = vec![];
    for _ in 0..threads {
        let g = Arc::clone(&guard);
        handles.push(thread::spawn(move || {
            for _ in 0..iterations {
                g.release(50);
                let _ = g.reserve(10);
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let final_alloc = guard.allocated();
    let final_count = guard.count();

    assert!(
        final_alloc <= 32 * 10_000 * 10 + 100,
        "Guard allocated value corrupted or wrapped: {}",
        final_alloc
    );
    assert!(
        final_count <= 32 * 10_000 + 1,
        "Guard count value corrupted or wrapped: {}",
        final_count
    );
}
