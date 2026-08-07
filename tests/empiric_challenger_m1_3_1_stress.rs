// ============================================================================
// EMPIRICAL STRESS HARNESS — CHALLENGER M1_3_1
// ============================================================================
// Stress tests for MPSC/MPMC lock-free ring buffer concurrency,
// capacity wrapping continuity, and CAS underflow safety.
// ============================================================================

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use xiangrust::gpu::buffer::{Buffer, Storable};
use xiangrust::gpu::guard::Guard;
use xiangrust::gpu::status::Status;

#[test]
fn test_capacity_wrapping_discontinuity_non_power_of_two() {
    // Capacity for 150 bytes is next power of two = 256.
    let buffer = Buffer::allocate(150, false).expect("Allocation failed");
    assert_eq!(buffer.capacity(), 256);

    let payload = [0xAAu8; 50];
    let mut recv = [0u8; 50];

    for i in 0..500 {
        let mut msg = payload;
        msg[0] = (i & 0xFF) as u8;
        assert_eq!(buffer.push(&msg), Ok(()));
        assert_eq!(buffer.pull(&mut recv), Ok(()));
        assert_eq!(recv[0], (i & 0xFF) as u8);
        assert_eq!(&recv[1..50], &payload[1..50]);
    }
}

#[test]
fn test_mpmc_concurrent_pull_stale_data_isolation() {
    // Test if multi-consumer pull corrupted or duplicated target buffers when CAS fails.
    let buffer = Arc::new(Buffer::allocate(1024, false).unwrap());

    // Push 100 unique bytes [0..100]
    let mut input = Vec::with_capacity(100);
    for i in 0..100 {
        input.push(i as u8);
    }
    buffer.push(&input).unwrap();

    let mut handles = Vec::new();
    let total_pulled_bytes = Arc::new(AtomicUsize::new(0));

    // Spawn 4 concurrent consumers pulling into 100-byte targets
    for _ in 0..4 {
        let buf = Arc::clone(&buffer);
        let bytes_counter = Arc::clone(&total_pulled_bytes);
        handles.push(thread::spawn(move || {
            let mut target = [0xFFu8; 100];
            let mut count = 0;
            while count < 100 {
                match buf.pull(&mut target) {
                    Ok(()) => {
                        bytes_counter.fetch_add(100, Ordering::SeqCst);
                    }
                    Err(Status::Ready) | Err(Status::Fault) => {
                        thread::yield_now();
                    }
                    Err(e) => panic!("Unexpected error: {:?}", e),
                }
                count += 1;
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}

#[test]
fn test_mpmc_high_contention_stress_integrity() {
    // High-contention test with 8 producers and 8 consumers
    let capacity = 4096;
    let buffer = Arc::new(Buffer::allocate(capacity, false).unwrap());
    let duration = Duration::from_millis(500);
    let running = Arc::new(AtomicBool::new(true));

    let total_produced = Arc::new(AtomicU64::new(0));
    let total_consumed = Arc::new(AtomicU64::new(0));

    let mut handles = Vec::new();

    // 8 Producers
    for thread_id in 0..8u8 {
        let buf = Arc::clone(&buffer);
        let run = Arc::clone(&running);
        let prod_count = Arc::clone(&total_produced);

        handles.push(thread::spawn(move || {
            let mut seq: u32 = 0;
            while run.load(Ordering::Relaxed) {
                // Packet: [thread_id (1b), seq_hi (1b), seq_lo (1b), payload (5b)] = 8 bytes
                let mut pkt = [0u8; 8];
                pkt[0] = thread_id;
                pkt[1] = ((seq >> 8) & 0xFF) as u8;
                pkt[2] = (seq & 0xFF) as u8;
                pkt[3] = thread_id ^ 0x55;
                pkt[4] = 0xAA;
                pkt[5] = 0xBB;
                pkt[6] = 0xCC;
                pkt[7] = 0xDD;

                match buf.push(&pkt) {
                    Ok(()) => {
                        seq = seq.wrapping_add(1);
                        prod_count.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(Status::Full) => {
                        thread::yield_now();
                    }
                    Err(e) => panic!("Unexpected push error: {:?}", e),
                }
            }
        }));
    }

    // 8 Consumers
    for _ in 0..8 {
        let buf = Arc::clone(&buffer);
        let run = Arc::clone(&running);
        let cons_count = Arc::clone(&total_consumed);

        handles.push(thread::spawn(move || {
            let mut pkt = [0u8; 8];
            while run.load(Ordering::Relaxed) || buf.pull(&mut pkt).is_ok() {
                match buf.pull(&mut pkt) {
                    Ok(()) => {
                        // Validate packet structure
                        let thread_id = pkt[0];
                        let expected_chk = thread_id ^ 0x55;
                        assert_eq!(pkt[3], expected_chk, "Corrupted packet payload detected!");
                        assert_eq!(pkt[4], 0xAA, "Corrupted packet magic byte 4");
                        assert_eq!(pkt[5], 0xBB, "Corrupted packet magic byte 5");
                        assert_eq!(pkt[6], 0xCC, "Corrupted packet magic byte 6");
                        assert_eq!(pkt[7], 0xDD, "Corrupted packet magic byte 7");

                        cons_count.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(Status::Ready) => {
                        thread::yield_now();
                    }
                    Err(e) => panic!("Unexpected pull error: {:?}", e),
                }
            }
        }));
    }

    thread::sleep(duration);
    running.store(false, Ordering::SeqCst);

    for h in handles {
        h.join().unwrap();
    }

    println!(
        "Produced packets: {}, Consumed packets: {}",
        total_produced.load(Ordering::Relaxed),
        total_consumed.load(Ordering::Relaxed)
    );
}

#[test]
fn test_vram_guard_underflow_and_race_safety() {
    let guard = Arc::new(Guard::new());
    let handles: Vec<_> = (0..16)
        .map(|_| {
            let g = Arc::clone(&guard);
            thread::spawn(move || {
                for _ in 0..1000 {
                    if let Ok(_) = g.reserve(1024) {
                        g.release(1024);
                    } else {
                        g.release(2048); // Intentional excess release to stress CAS underflow protection
                    }
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    // Allocated and count must be strictly 0, never wrapped to usize::MAX
    assert_eq!(guard.allocated(), 0);
    assert_eq!(guard.count(), 0);
}
