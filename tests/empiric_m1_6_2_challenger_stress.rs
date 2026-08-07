// ============================================================================
// XIANGTI ENGINE: EMPIRICAL CHALLENGER M1_6_2 STRESS TEST HARNESS
// ============================================================================
// Empirical stress test harness created by Challenger M1_6_2 for src/gpu/buffer.rs:
// 1. Ring buffer framing alignment & boundary wraparound (1/3, 2/2, 3/1 header split).
// 2. Variable payload length framing (1 byte to 4096 bytes testing stack/heap allocs).
// 3. Hardware alignment (64-byte cache line alignment, size_of, align_of, pointer % 64).
// 4. Shared memory zero-copy semantics (macOS MTLResourceStorageModeShared detection).
// 5. Framing desynchronization resilience & target buffer sizing protection.
// 6. High-concurrency MPSC and MPMC variable-length packet queue stress.
// ============================================================================

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use xiangrust::gpu::{Buffer, Status, Storable};

/// Helper: Generate a variable length payload with embedded checksum and metadata
fn make_payload(pid: u32, seq: u32, len: usize) -> Vec<u8> {
    assert!(len >= 12, "Payload length must be at least 12 bytes");
    let mut payload = vec![0u8; len];
    let pid_bytes = pid.to_le_bytes();
    let seq_bytes = seq.to_le_bytes();
    let checksum = pid ^ seq ^ (len as u32) ^ 0xCAFEBABE;
    let chk_bytes = checksum.to_le_bytes();

    payload[0..4].copy_from_slice(&pid_bytes);
    payload[4..8].copy_from_slice(&seq_bytes);
    payload[8..12].copy_from_slice(&chk_bytes);

    for i in 12..len {
        payload[i] = ((pid as u8).wrapping_add(seq as u8)).wrapping_add(i as u8);
    }
    payload
}

/// Helper: Verify payload integrity and return (pid, seq, len)
fn check_payload(data: &[u8]) -> Result<(u32, u32, usize), String> {
    if data.len() < 12 {
        return Err(format!("Data length too small: {}", data.len()));
    }
    let pid = u32::from_le_bytes(data[0..4].try_into().unwrap());
    let seq = u32::from_le_bytes(data[4..8].try_into().unwrap());
    let chk = u32::from_le_bytes(data[8..12].try_into().unwrap());
    let len = (chk ^ pid ^ seq ^ 0xCAFEBABE) as usize;

    if data.len() < len {
        return Err(format!("Data length {} smaller than payload len {}", data.len(), len));
    }

    for i in 12..len {
        let expected_byte = ((pid as u8).wrapping_add(seq as u8)).wrapping_add(i as u8);
        if data[i] != expected_byte {
            return Err(format!(
                "Byte corruption at index {}! pid={}, seq={}, expected=0x{:02X}, got=0x{:02X}",
                i, pid, seq, expected_byte, data[i]
            ));
        }
    }

    Ok((pid, seq, len))
}

/// Test 1: Hardware 64-byte Cache Line Alignment & Struct Layout
#[test]
fn test_m1_6_2_hardware_alignment_and_layout() {
    assert_eq!(
        std::mem::size_of::<Buffer>(),
        64,
        "Buffer struct size must be exactly 64 bytes"
    );
    assert_eq!(
        std::mem::align_of::<Buffer>(),
        64,
        "Buffer struct alignment must be 64 bytes"
    );

    // Test heap allocation alignment for host (device=false) and device (device=true)
    let host_buf = Buffer::allocate(256, false).expect("Host buffer allocation failed");
    assert!(host_buf.aligned(), "Host buffer pointer must be 64-byte aligned");
    assert_eq!(
        (host_buf.pointer() as usize) % 64,
        0,
        "Host buffer address must be divisible by 64"
    );
    assert!(!host_buf.shared(), "Host buffer should have shared=false");

    let dev_buf = Buffer::allocate(256, true).expect("Device buffer allocation failed");
    assert!(dev_buf.aligned(), "Device buffer pointer must be 64-byte aligned");
    assert_eq!(
        (dev_buf.pointer() as usize) % 64,
        0,
        "Device buffer address must be divisible by 64"
    );
    assert_eq!(
        dev_buf.shared(),
        cfg!(target_os = "macos"),
        "Device buffer shared flag must match macOS unified memory policy"
    );
}

/// Test 2: Boundary Header & Payload Wrapping (1/3, 2/2, 3/1 Header Split across capacity)
#[test]
fn test_m1_6_2_boundary_header_split_wraparound() {
    let capacity = 64; // Buffer capacity is 64 bytes
    let buf = Buffer::allocate(capacity, false).expect("Buffer allocation");

    // Push & pull 20,000 variable length packets to force continuous wraparounds
    for iteration in 0..20_000 {
        // Vary lengths 12..36 to hit all header split boundaries
        let len = 12 + (iteration % 25);
        let payload = make_payload(7, iteration as u32, len);

        let push_res = buf.push(&payload);
        assert!(
            push_res.is_ok(),
            "Push failed at iteration {} (len={}): {:?}",
            iteration,
            len,
            push_res
        );

        let mut target = vec![0u8; 64];
        let pull_res = buf.pull(&mut target);
        assert!(
            pull_res.is_ok(),
            "Pull failed at iteration {}: {:?}",
            iteration,
            pull_res
        );

        let (pid, seq, pulled_len) = check_payload(&target[..len]).expect("Payload verification failed");
        assert_eq!(pid, 7);
        assert_eq!(seq, iteration as u32);
        assert_eq!(pulled_len, len);
    }
}

/// Test 3: Variable Payload Sizes (Stack <= 1024 bytes vs Heap > 1024 bytes)
#[test]
fn test_m1_6_2_stack_and_heap_payload_tiering() {
    let buf = Buffer::allocate(16384, false).expect("Buffer allocation");

    // Test small stack payload (16 bytes)
    let p_small = make_payload(1, 100, 16);
    buf.push(&p_small).unwrap();
    let mut t_small = vec![0u8; 64];
    buf.pull(&mut t_small).unwrap();
    assert_eq!(check_payload(&t_small[..16]).unwrap(), (1, 100, 16));

    // Test medium stack payload (1024 bytes)
    let p_med = make_payload(2, 200, 1024);
    buf.push(&p_med).unwrap();
    let mut t_med = vec![0u8; 2048];
    buf.pull(&mut t_med).unwrap();
    assert_eq!(check_payload(&t_med[..1024]).unwrap(), (2, 200, 1024));

    // Test large heap payload (4096 bytes)
    let p_large = make_payload(3, 300, 4096);
    buf.push(&p_large).unwrap();
    let mut t_large = vec![0u8; 8192];
    buf.pull(&mut t_large).unwrap();
    assert_eq!(check_payload(&t_large[..4096]).unwrap(), (3, 300, 4096));
}

/// Test 4: Framing Protection, Fault Ingestion & Target Sizing
#[test]
fn test_m1_6_2_framing_protection_and_fault_handling() {
    let buf = Buffer::allocate(256, false).expect("Buffer allocation");

    // 1. Empty payload push -> Status::Fault
    assert_eq!(buf.push(&[]), Err(Status::Fault));

    // 2. Empty buffer pull -> Status::Ready
    let mut target = [0u8; 64];
    assert_eq!(buf.pull(&mut target), Err(Status::Ready));

    // 3. Payload larger than buffer capacity -> Status::Full
    let huge_payload = vec![0u8; 512];
    assert_eq!(buf.push(&huge_payload), Err(Status::Full));

    // 4. Push valid packet of 32 bytes
    let payload = make_payload(99, 1, 32);
    buf.push(&payload).unwrap();

    // 5. Target buffer smaller than packet payload -> Status::Fault, head left intact
    let mut small_target = [0u8; 16];
    assert_eq!(buf.pull(&mut small_target), Err(Status::Fault));

    // 6. Pull again with correct size target -> Should succeed with full payload intact!
    let mut valid_target = [0u8; 64];
    assert!(buf.pull(&mut valid_target).is_ok());
    assert_eq!(check_payload(&valid_target[..32]).unwrap(), (99, 1, 32));
}

/// Test 5: High Concurrency MPSC (16 Producers, 1 Consumer) Variable Length Stress
#[test]
fn test_m1_6_2_mpsc_variable_length_high_concurrency() {
    let capacity = 8192;
    let buf = Arc::new(Buffer::allocate(capacity, false).expect("Buffer allocation"));
    let num_producers = 16;
    let items_per_prod = 2_000;
    let total_expected = (num_producers * items_per_prod) as u64;

    let producers_done = Arc::new(AtomicUsize::new(0));
    let valid_count = Arc::new(AtomicU64::new(0));
    let corrupt_count = Arc::new(AtomicU64::new(0));

    let start = Instant::now();

    // Producers
    let mut prod_handles = vec![];
    for pid in 0..num_producers {
        let b = Arc::clone(&buf);
        let pd = Arc::clone(&producers_done);
        prod_handles.push(thread::spawn(move || {
            for seq in 0..items_per_prod {
                let len = 12 + ((pid * 17 + seq * 3) % 128); // Variable lengths 12..140
                let payload = make_payload(pid as u32, seq as u32, len);
                loop {
                    match b.push(&payload) {
                        Ok(_) => break,
                        Err(Status::Full) => thread::yield_now(),
                        Err(e) => panic!("Producer {} push error: {:?}", pid, e),
                    }
                }
            }
            pd.fetch_add(1, Ordering::SeqCst);
        }));
    }

    // Consumer
    let b_cons = Arc::clone(&buf);
    let pd_cons = Arc::clone(&producers_done);
    let vc_cons = Arc::clone(&valid_count);
    let cc_cons = Arc::clone(&corrupt_count);

    let cons_handle = thread::spawn(move || {
        let mut target = vec![0u8; 512];
        loop {
            match b_cons.pull(&mut target) {
                Ok(_) => match check_payload(&target) {
                    Ok(_) => {
                        vc_cons.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(e) => {
                        cc_cons.fetch_add(1, Ordering::Relaxed);
                        eprintln!("[MPSC CORRUPTION] {}", e);
                    }
                },
                Err(Status::Ready) => {
                    if pd_cons.load(Ordering::SeqCst) == num_producers {
                        let mut drain = vec![0u8; 512];
                        while b_cons.pull(&mut drain).is_ok() {
                            if check_payload(&drain).is_ok() {
                                vc_cons.fetch_add(1, Ordering::Relaxed);
                            } else {
                                cc_cons.fetch_add(1, Ordering::Relaxed);
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
    let total_valid = valid_count.load(Ordering::SeqCst);
    let total_corrupt = corrupt_count.load(Ordering::SeqCst);

    println!(
        "MPSC 16-Producer Stress: Processed {}/{} items in {:?} (Corrupt: {})",
        total_valid, total_expected, duration, total_corrupt
    );

    assert_eq!(total_corrupt, 0, "MPSC suffered data corruptions!");
    assert_eq!(total_valid, total_expected, "MPSC lost packets!");
}

/// Test 6: High Concurrency MPMC (16 Producers, 16 Consumers) Variable Length Stress
#[test]
fn test_m1_6_2_mpmc_variable_length_high_concurrency() {
    let capacity = 8192;
    let buf = Arc::new(Buffer::allocate(capacity, false).expect("Buffer allocation"));
    let num_producers = 16;
    let num_consumers = 16;
    let items_per_prod = 2_000;
    let total_expected = (num_producers * items_per_prod) as u64;

    let producers_done = Arc::new(AtomicUsize::new(0));
    let valid_count = Arc::new(AtomicU64::new(0));
    let corrupt_count = Arc::new(AtomicU64::new(0));

    let start = Instant::now();

    // Producers
    let mut prod_handles = vec![];
    for pid in 0..num_producers {
        let b = Arc::clone(&buf);
        let pd = Arc::clone(&producers_done);
        prod_handles.push(thread::spawn(move || {
            for seq in 0..items_per_prod {
                let len = 12 + ((pid * 31 + seq * 11) % 200); // Variable lengths 12..212
                let payload = make_payload(pid as u32, seq as u32, len);
                loop {
                    match b.push(&payload) {
                        Ok(_) => break,
                        Err(Status::Full) => thread::yield_now(),
                        Err(e) => panic!("Producer {} push error: {:?}", pid, e),
                    }
                }
            }
            pd.fetch_add(1, Ordering::SeqCst);
        }));
    }

    // Consumers
    let mut cons_handles = vec![];
    for cid in 0..num_consumers {
        let b = Arc::clone(&buf);
        let pd = Arc::clone(&producers_done);
        let vc = Arc::clone(&valid_count);
        let cc = Arc::clone(&corrupt_count);

        cons_handles.push(thread::spawn(move || {
            let mut target = vec![0u8; 512];
            loop {
                match b.pull(&mut target) {
                    Ok(_) => match check_payload(&target) {
                        Ok(_) => {
                            vc.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(e) => {
                            cc.fetch_add(1, Ordering::Relaxed);
                            eprintln!("[MPMC Consumer {} CORRUPTION] {}", cid, e);
                        }
                    },
                    Err(Status::Ready) => {
                        if pd.load(Ordering::SeqCst) == num_producers {
                            let mut drain = vec![0u8; 512];
                            while b.pull(&mut drain).is_ok() {
                                if check_payload(&drain).is_ok() {
                                    vc.fetch_add(1, Ordering::Relaxed);
                                } else {
                                    cc.fetch_add(1, Ordering::Relaxed);
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

    let duration = start.elapsed();
    let total_valid = valid_count.load(Ordering::SeqCst);
    let total_corrupt = corrupt_count.load(Ordering::SeqCst);

    println!(
        "MPMC 16x16 Stress: Processed {}/{} items in {:?} (Corrupt: {})",
        total_valid, total_expected, duration, total_corrupt
    );

    assert_eq!(total_corrupt, 0, "MPMC suffered data corruptions!");
    assert_eq!(total_valid, total_expected, "MPMC lost packets!");
}
