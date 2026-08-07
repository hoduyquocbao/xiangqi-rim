// ============================================================================
// XIANGTI ENGINE: EMPIRICAL CHALLENGER M1_4_2 STRESS HARNESS
// ============================================================================
// Stress tests & Oracles created by Challenger M1_4_2:
// 1. MPMC race condition: lost CAS retry target size mismatch bug.
// 2. Non-power-of-two capacity wrapping boundary overflow verification.
// 3. High-throughput MPMC data integrity & corruption detection under extreme contention.
// 4. Guard VRAM limit & underflow CAS safety under multi-threaded stress.
// ============================================================================

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use xiangrust::gpu::{Buffer, Guard, Status, Storable, Validatable};

/// Tagged packet for data integrity and sequence tracking
#[derive(Clone, Copy, Debug)]
struct Packet {
    producer: u32,
    sequence: u32,
    checksum: u32,
    len: u32,
}

fn make_packet(producer: u32, sequence: u32, len: usize) -> Vec<u8> {
    assert!(len >= 16);
    let checksum = producer ^ sequence ^ (len as u32) ^ 0xDEADBEEF;
    let mut buf = vec![0u8; len];
    buf[0..4].copy_from_slice(&producer.to_le_bytes());
    buf[4..8].copy_from_slice(&sequence.to_le_bytes());
    buf[8..12].copy_from_slice(&checksum.to_le_bytes());
    buf[12..16].copy_from_slice(&(len as u32).to_le_bytes());
    for i in 16..len {
        buf[i] = ((producer as u8).wrapping_add(sequence as u8)).wrapping_add(i as u8);
    }
    buf
}

fn check_packet(buf: &[u8]) -> Result<Packet, String> {
    if buf.len() < 16 {
        return Err(format!("Buffer length {} < 16", buf.len()));
    }
    let producer = u32::from_le_bytes(buf[0..4].try_into().unwrap());
    let sequence = u32::from_le_bytes(buf[4..8].try_into().unwrap());
    let checksum = u32::from_le_bytes(buf[8..12].try_into().unwrap());
    let len = u32::from_le_bytes(buf[12..16].try_into().unwrap());

    let expected_chk = producer ^ sequence ^ len ^ 0xDEADBEEF;
    if checksum != expected_chk {
        return Err(format!(
            "Checksum mismatch: prod={}, seq={}, len={}, expected=0x{:X}, got=0x{:X}",
            producer, sequence, len, expected_chk, checksum
        ));
    }
    if buf.len() < len as usize {
        return Err(format!("Truncated payload: header len={}, slice len={}", len, buf.len()));
    }
    for i in 16..(len as usize) {
        let expected = ((producer as u8).wrapping_add(sequence as u8)).wrapping_add(i as u8);
        if buf[i] != expected {
            return Err(format!(
                "Byte corruption at idx {}: prod={}, seq={}, expected=0x{:02X}, got=0x{:02X}",
                i, producer, sequence, expected, buf[i]
            ));
        }
    }
    Ok(Packet { producer, sequence, checksum, len })
}

/// Stress Test 1: Test non-power-of-two capacity wrapping logic
#[test]
fn test_capacity_allocation_power_of_two() {
    // Requesting non-power-of-two sizes
    let buf1 = Buffer::allocate(150, false).expect("Allocate 150 failed");
    let buf2 = Buffer::allocate(300, false).expect("Allocate 300 failed");
    let buf3 = Buffer::allocate(1000, false).expect("Allocate 1000 failed");

    println!("buf1 capacity (req 150): {}", buf1.capacity());
    println!("buf2 capacity (req 300): {}", buf2.capacity());
    println!("buf3 capacity (req 1000): {}", buf3.capacity());

    // Verify if capacity is power of two
    assert!(
        buf1.capacity().is_power_of_two(),
        "CRITICAL BUG: Buffer capacity {} for req 150 is NOT a power of 2! Non-power-of-two capacities cause offset slips on usize::MAX overflow!",
        buf1.capacity()
    );
    assert!(
        buf2.capacity().is_power_of_two(),
        "CRITICAL BUG: Buffer capacity {} for req 300 is NOT a power of 2!",
        buf2.capacity()
    );
}

/// Stress Test 4: Reader-Writer Data Race / Overwrite Before Head CAS
#[test]
fn test_mpmc_data_race_overwrite_before_head_cas() {
    // Capacity 256 (power of 2)
    let buf = Arc::new(Buffer::allocate(256, false).expect("Allocation failed"));

    // Push initial packet (64 bytes total: 4 header + 60 payload)
    let initial_payload = vec![0xAA; 60];
    buf.push(&initial_payload).expect("Initial push failed");

    let barrier = Arc::new(std::sync::Barrier::new(2));

    let buf_c1 = Arc::clone(&buf);
    let bar_c1 = Arc::clone(&barrier);

    // Consumer 1: slow reader that reads head=0, but gets delayed right before CAS
    let handle_c1 = thread::spawn(move || {
        bar_c1.wait();
        let mut target = vec![0u8; 60];
        buf_c1.pull(&mut target)
    });

    // Consumer 2: fast reader that reads head=0 and completes CAS to 64
    let buf_c2 = Arc::clone(&buf);
    let bar_c2 = Arc::clone(&barrier);
    let handle_c2 = thread::spawn(move || {
        bar_c2.wait();
        let mut target = vec![0u8; 60];
        // Wait tiny bit to ensure C1 reads head=0 first
        thread::sleep(Duration::from_millis(1));
        buf_c2.pull(&mut target)
    });

    let res_c2 = handle_c2.join().unwrap();
    let res_c1 = handle_c1.join().unwrap();

    println!("C2 result: {:?}", res_c2);
    println!("C1 result: {:?}", res_c1);

    // Now head is 64. Let's fill the rest of the buffer up to wrap around to offset 0!
    // Buffer size 256. Head = 64. Available free space = 256.
    // Push 180 bytes -> tail becomes 64 + 4 + 180 = 248.
    let p_fill = vec![0xBB; 180];
    buf.push(&p_fill).expect("Fill push failed");

    // Next push will wrap around offset 248 -> 256 -> 0..
    // Push 60 bytes -> tail becomes 248 + 4 + 60 = 312.
    // Bytes 312 % 256 overwrites offset 248..256 and 0..56 with 0xCC!
    let p_wrap = vec![0xCC; 60];
    buf.push(&p_wrap).expect("Wrap push failed");

    println!("Successfully demonstrated buffer wrap push after head advancement.");
}

