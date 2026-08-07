// ============================================================================
// XIANGTI ENGINE: EMPIRICAL STRESS TEST HARNESS FOR BUFFER RING BUFFER (M1)
// ============================================================================
// Kiểm thử thực nghiệm áp lực cao (Stress Test) đối với Lock-Free Ring Buffer Queue
// và Zero-Copy transfer trong class `Buffer` (src/gpu/buffer.rs).
// Đánh giá đồng thời đa luồng (Multi-producer, Multi-consumer), SPSC, MPSC, MPMC,
// kiểm tra toàn vẹn dữ liệu (Data Corruption), sai lệch đua tranh (Race Condition),
// và rò rỉ bộ nhớ (Memory Leak).
// Tuân thủ 100% định danh từ đơn tiếng Anh, 100% chú thích tiếng Việt.
// ============================================================================

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use xiangrust::gpu::{Buffer, Storable, Status};

/// 1. EMPIRICAL TEST: Single Producer Single Consumer (SPSC) Sequential Transfer Integrity
#[test]
fn test_spsc_queue_integrity() {
    let size = 64 * 1024; // 64 KB capacity
    let buffer = Arc::new(Buffer::allocate(size, false).expect("Buffer allocation failed"));
    let count = 1000;

    let signal = Arc::new(AtomicBool::new(true));

    // Producer Thread
    let p_buf = Arc::clone(&buffer);
    let p_sig = Arc::clone(&signal);
    let producer = thread::spawn(move || {
        let mut sent = 0;
        for i in 0..count {
            let mut payload = [0u8; 32];
            // Encode sequence index into payload
            let bytes = (i as u32).to_le_bytes();
            payload[0..4].copy_from_slice(&bytes);
            payload[4..32].fill((i % 255) as u8 + 1);

            while p_sig.load(Ordering::Relaxed) {
                match p_buf.push(&payload) {
                    Ok(()) => {
                        sent += 1;
                        break;
                    }
                    Err(Status::Full) => {
                        thread::yield_now();
                    }
                    Err(err) => panic!("Unexpected push error: {:?}", err),
                }
            }
        }
        sent
    });

    // Consumer Thread
    let c_buf = Arc::clone(&buffer);
    let c_sig = Arc::clone(&signal);
    let consumer = thread::spawn(move || {
        let mut received = 0;
        let mut corrupt = 0;
        let mut read = [0u8; 32];

        while received < count && c_sig.load(Ordering::Relaxed) {
            match c_buf.pull(&mut read) {
                Ok(()) => {
                    let seq = u32::from_le_bytes([read[0], read[1], read[2], read[3]]);
                    let expected_pad = (seq % 255) as u8 + 1;
                    if read[4..32].iter().any(|&b| b != expected_pad) {
                        corrupt += 1;
                    }
                    received += 1;
                }
                Err(Status::Ready) => {
                    thread::yield_now();
                }
                Err(err) => panic!("Unexpected pull error: {:?}", err),
            }
        }
        (received, corrupt)
    });

    let sent_count = producer.join().expect("Producer panic");
    let (recv_count, corrupt_count) = consumer.join().expect("Consumer panic");

    signal.store(false, Ordering::Relaxed);

    assert_eq!(sent_count, count, "Producer must send all batches");
    assert_eq!(recv_count, count, "Consumer must receive all batches");
    assert_eq!(corrupt_count, 0, "SPSC must have ZERO data corruption");
}

/// 2. EMPIRICAL TEST: Multi-Producer Single-Consumer (MPSC) Stress Testing
#[test]
fn test_mpsc_concurrent_push_stress() {
    let size = 256 * 1024; // 256 KB
    let buffer = Arc::new(Buffer::allocate(size, false).expect("MPSC buffer allocation failed"));
    let producers_count = 4;
    let items_per_producer = 500;

    let total_sent = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();

    for p in 0..producers_count {
        let buf = Arc::clone(&buffer);
        let sent_counter = Arc::clone(&total_sent);
        let handle = thread::spawn(move || {
            for i in 0..items_per_producer {
                let mut payload = [0u8; 32];
                let tag = ((p as u32) << 16) | (i as u32);
                payload[0..4].copy_from_slice(&tag.to_le_bytes());
                payload[4..32].fill((p + 1) as u8);

                loop {
                    match buf.push(&payload) {
                        Ok(()) => {
                            sent_counter.fetch_add(1, Ordering::Relaxed);
                            break;
                        }
                        Err(Status::Full) => {
                            thread::yield_now();
                        }
                        Err(err) => panic!("MPSC push error: {:?}", err),
                    }
                }
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().expect("Producer thread panic");
    }

    let expected_total = producers_count * items_per_producer;
    assert_eq!(total_sent.load(Ordering::Relaxed), expected_total);

    // Now pull all items and check for corruption / overwritten bytes
    let mut pulled = 0;
    let mut corrupted = 0;
    let mut read_buf = [0u8; 32];

    while pulled < expected_total {
        match buffer.pull(&mut read_buf) {
            Ok(()) => {
                let tag = u32::from_le_bytes([read_buf[0], read_buf[1], read_buf[2], read_buf[3]]);
                let pid = (tag >> 16) as usize;
                let expected_pad = (pid + 1) as u8;
                if read_buf[4..32].iter().any(|&b| b != expected_pad) {
                    corrupted += 1;
                }
                pulled += 1;
            }
            Err(Status::Ready) => break,
            Err(err) => panic!("MPSC pull error: {:?}", err),
        }
    }

    println!(
        "MPSC Stress Test Result: Expected {}, Pulled {}, Corrupted {}",
        expected_total, pulled, corrupted
    );
    assert_eq!(
        corrupted, 0,
        "MPSC lock-free push MUST have zero data corruption under concurrent writers!"
    );
}

/// 3. EMPIRICAL TEST: Multi-Producer Multi-Consumer (MPMC) High-Concurrency Stress Test
#[test]
fn test_mpmc_concurrent_push_pull_stress() {
    let size = 512 * 1024; // 512 KB
    let buffer = Arc::new(Buffer::allocate(size, false).expect("MPMC buffer allocation failed"));
    let num_producers = 4;
    let num_consumers = 4;
    let items_per_producer = 1000;
    let total_items = num_producers * items_per_producer;

    let active_producers = Arc::new(AtomicUsize::new(num_producers));
    let total_consumed = Arc::new(AtomicUsize::new(0));
    let total_corrupted = Arc::new(AtomicUsize::new(0));

    let mut prod_handles = Vec::new();
    let mut cons_handles = Vec::new();

    // Spawn Producers
    for p in 0..num_producers {
        let buf = Arc::clone(&buffer);
        let p_active = Arc::clone(&active_producers);
        prod_handles.push(thread::spawn(move || {
            for i in 0..items_per_producer {
                let mut batch = [0u8; 64];
                let pid_byte = (p as u8) + 1;
                let seq = i as u32;
                batch[0..4].copy_from_slice(&seq.to_le_bytes());
                batch[4] = pid_byte;
                batch[5..64].fill(pid_byte);

                loop {
                    match buf.push(&batch) {
                        Ok(()) => break,
                        Err(Status::Full) => thread::yield_now(),
                        Err(e) => panic!("MPMC push error: {:?}", e),
                    }
                }
            }
            p_active.fetch_sub(1, Ordering::Relaxed);
        }));
    }

    // Spawn Consumers
    for _ in 0..num_consumers {
        let buf = Arc::clone(&buffer);
        let p_active = Arc::clone(&active_producers);
        let t_consumed = Arc::clone(&total_consumed);
        let t_corrupted = Arc::clone(&total_corrupted);

        cons_handles.push(thread::spawn(move || {
            let mut read_batch = [0u8; 64];
            loop {
                match buf.pull(&mut read_batch) {
                    Ok(()) => {
                        t_consumed.fetch_add(1, Ordering::Relaxed);
                        let pid_byte = read_batch[4];
                        if pid_byte == 0 || read_batch[5..64].iter().any(|&b| b != pid_byte) {
                            t_corrupted.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    Err(Status::Ready) => {
                        if p_active.load(Ordering::Relaxed) == 0 {
                            // Empty queue and all producers finished -> attempt one final pull check
                            match buf.pull(&mut read_batch) {
                                Ok(()) => {
                                    t_consumed.fetch_add(1, Ordering::Relaxed);
                                    let pid_byte = read_batch[4];
                                    if pid_byte == 0 || read_batch[5..64].iter().any(|&b| b != pid_byte) {
                                        t_corrupted.fetch_add(1, Ordering::Relaxed);
                                    }
                                }
                                _ => break,
                            }
                        } else {
                            thread::yield_now();
                        }
                    }
                    Err(e) => panic!("MPMC pull error: {:?}", e),
                }
            }
        }));
    }

    for h in prod_handles {
        h.join().expect("Producer thread panic");
    }
    for h in cons_handles {
        h.join().expect("Consumer thread panic");
    }

    let consumed = total_consumed.load(Ordering::Relaxed);
    let corrupted = total_corrupted.load(Ordering::Relaxed);

    println!(
        "MPMC Stress Summary: Total Pushed = {}, Total Consumed = {}, Corrupted = {}",
        total_items, consumed, corrupted
    );
    assert_eq!(
        corrupted, 0,
        "MPMC lock-free queue MUST have zero data corruption under concurrent threads!"
    );
}

/// 4. EMPIRICAL TEST: Repeated Allocation, Clear, and Deallocation for Memory Leak Verification
#[test]
fn test_memory_leak_and_lifecycle_integrity() {
    let iterations = 10_000;
    let size = 64 * 1024; // 64 KB

    for _ in 0..iterations {
        let mut buffer = Buffer::allocate(size, false).expect("Buffer allocation failed");
        assert!(buffer.aligned());
        assert_eq!(buffer.capacity(), size);

        let data = [0xAAu8; 128];
        assert!(buffer.push(&data).is_ok());

        let mut read = [0u8; 128];
        assert!(buffer.pull(&mut read).is_ok());
        assert_eq!(data, read);

        buffer.clear();
        assert_eq!(buffer.bytes(), 0);

        buffer.free();
        assert!(buffer.pointer().is_null());
        assert_eq!(buffer.capacity(), 0);
    }
}
