// ============================================================================
// EMPIRICAL STRESS HARNESS: M2 Iteration 3 Store Persistence & Edge Cases
// ============================================================================
// File: tests/empiric_m2_rem3_2_challenger_stress.rs
// Purpose: Empirically stress-test Store persistence binary roundtrip,
// corrupted headers, boundary buffer sizes (0 and 10,000 records), and edge cases.
// ============================================================================

use xiangrust::learn::replay::{Replay, Sample};
use xiangrust::learn::store::{Header, Store};
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;

static STRESS_MUTEX: Mutex<()> = Mutex::new(());

/// 1. Test empty replay persistence (0 records): File size must be exactly 64 bytes.
#[test]
fn test_store_persistence_empty_replay() {
    let _guard = STRESS_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let path = "/tmp/test_m2_rem3_2_empty.bin";
    let _ = std::fs::remove_file(path);

    let replay = Replay::capacity(100);
    assert_eq!(replay.len(), 0);

    let save_res = Store::save(&replay, path);
    assert!(save_res.is_ok(), "Store::save on empty replay must succeed");

    let meta = std::fs::metadata(path).expect("File metadata must exist");
    assert_eq!(meta.len(), 64, "Empty store file must be exactly 64 bytes (Header only)");

    let mut loaded = Replay::capacity(100);
    let load_res = Store::load(&mut loaded, path);
    assert!(load_res.is_ok(), "Store::load on empty replay file must succeed");
    assert_eq!(load_res.unwrap(), 0);
    assert_eq!(loaded.len(), 0);

    let _ = std::fs::remove_file(path);
}

/// 2. Test large replay persistence (10,000 records): Verify complete byte fidelity.
#[test]
fn test_store_persistence_10k_records_fidelity() {
    let _guard = STRESS_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let path = "/tmp/test_m2_rem3_2_10k.bin";
    let _ = std::fs::remove_file(path);

    let count = 10_000usize;
    let mut replay = Replay::capacity(count);

    for i in 0..count {
        let hash = (i as u64 + 1).wrapping_mul(0x9E3779B97F4A7C15);
        let mv = (i as u16) ^ 0xABCD;
        let reward = (i as f32) / (count as f32);
        let next = hash.rotate_left(13);
        let done = (i % 3 == 0) as u8;

        replay.push(Sample::new(hash, mv, reward, next, done));
    }

    assert_eq!(replay.len(), count);

    let save_res = Store::save(&replay, path);
    assert!(save_res.is_ok(), "Store::save for 10k records must succeed");

    let meta = std::fs::metadata(path).expect("File metadata must exist");
    assert_eq!(meta.len(), 64 + (count as u64) * 32, "File size must be Header(64B) + 10k * 32B");

    let mut loaded = Replay::capacity(count);
    let load_res = Store::load(&mut loaded, path);
    assert!(load_res.is_ok());
    assert_eq!(load_res.unwrap(), count);
    assert_eq!(loaded.len(), count);

    // Verify fidelity of every single sample
    for i in 0..count {
        let orig = replay.get(i).unwrap();
        let item = loaded.get(i).unwrap();
        assert_eq!(item.hash, orig.hash, "Sample {} hash mismatch", i);
        assert_eq!(item.mv, orig.mv, "Sample {} mv mismatch", i);
        assert!((item.reward - orig.reward).abs() < 1e-6, "Sample {} reward mismatch", i);
        assert_eq!(item.next, orig.next, "Sample {} next mismatch", i);
        assert_eq!(item.done, orig.done, "Sample {} done mismatch", i);
    }

    let _ = std::fs::remove_file(path);
}

/// 3. Test corrupted magic signature and version header: Store::load must reject.
#[test]
fn test_store_persistence_corrupted_magic() {
    let _guard = STRESS_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let path = "/tmp/test_m2_rem3_2_corrupt.bin";
    let _ = std::fs::remove_file(path);

    let mut file = File::create(path).unwrap();
    let mut header = Header::new(10);
    header.magic = *b"BADM"; // Bad magic
    let hdr_slice = unsafe {
        std::slice::from_raw_parts(&header as *const Header as *const u8, std::mem::size_of::<Header>())
    };
    file.write_all(hdr_slice).unwrap();
    file.flush().unwrap();
    drop(file);

    let mut loaded = Replay::capacity(100);
    let load_res = Store::load(&mut loaded, path);
    assert!(load_res.is_err(), "Store::load must fail on bad magic header");

    let _ = std::fs::remove_file(path);
}

/// 4. Test truncated binary store file: Store::load loads available complete records gracefully.
#[test]
fn test_store_persistence_truncated_file() {
    let _guard = STRESS_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let path = "/tmp/test_m2_rem3_2_truncated.bin";
    let _ = std::fs::remove_file(path);

    let mut replay = Replay::capacity(10);
    for i in 0..10 {
        replay.push(Sample::new(i as u64 + 1, i as u16, 0.7, i as u64 + 100, 0));
    }

    Store::save(&replay, path).unwrap();

    // Truncate file so that only 5 records are fully present (64 + 5 * 32 = 224 bytes) plus 10 bytes partial
    let file = File::options().write(true).open(path).unwrap();
    file.set_len(224 + 10).unwrap();
    drop(file);

    let mut loaded = Replay::capacity(10);
    let load_res = Store::load(&mut loaded, path);
    assert!(load_res.is_ok(), "Truncated load should stop gracefully at EOF");
    assert_eq!(load_res.unwrap(), 5, "Should load exactly 5 full records");
    assert_eq!(loaded.len(), 5);

    let _ = std::fs::remove_file(path);
}
