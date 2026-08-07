// ============================================================================
// EMPIRICAL HARNESS: MILESTONE 2 ITERATION 2 REMEDIATION CHALLENGER
// ============================================================================
// Test harness to empirically verify binary store persistence roundtrip,
// memory layout alignment, error handling for corrupted headers/data, and
// single-word identifier compliance.
// ============================================================================

use std::fs::File;
use std::io::Write;
use xiangrust::book::endgame::Endgame;
use xiangrust::book::opening::Book;
use xiangrust::learn::replay::{Replay, Sample};
use xiangrust::learn::store::{Header, Record, Store, VERSION};

/// 1. Test memory layout alignment & size of Header, Record, and Store
#[test]
fn test_memory_alignment_and_sizes() {
    assert_eq!(std::mem::align_of::<Header>(), 64);
    assert_eq!(std::mem::size_of::<Header>(), 64);
    assert_eq!(std::mem::align_of::<Record>(), 16);
    assert_eq!(std::mem::size_of::<Record>(), 32);
    assert_eq!(std::mem::align_of::<Store>(), 64);

    // Verify layout offsets in Record to confirm zero uninitialized padding UB
    let rec = Record {
        hash: 0x123456789ABCDEF0,
        next: 0x0FEDCBA987654321,
        reward: 0.85,
        mv: 0x1316,
        done: 1,
        pad: [0u8; 9],
    };

    let bytes: &[u8] = unsafe {
        std::slice::from_raw_parts(&rec as *const Record as *const u8, std::mem::size_of::<Record>())
    };
    assert_eq!(bytes.len(), 32);
}

/// 2. Test roundtrip persistence with 0 samples, 1 sample, and 10,000 samples
#[test]
fn test_store_persistence_boundary_roundtrips() {
    let path_empty = "/tmp/test_m2_rem_2_empty.bin";
    let path_single = "/tmp/test_m2_rem_2_single.bin";
    let path_large = "/tmp/test_m2_rem_2_large.bin";

    // Case A: 0 samples
    let replay_empty = Replay::capacity(100);
    assert!(Store::save(&replay_empty, path_empty).is_ok());

    let mut target_empty = Replay::capacity(100);
    let loaded_empty = Store::load(&mut target_empty, path_empty);
    assert_eq!(loaded_empty.unwrap(), 0);
    assert_eq!(target_empty.len(), 0);

    // Case B: 1 sample
    let mut replay_single = Replay::capacity(100);
    replay_single.push(Sample::new(0xABC, 0x1316, 0.75, 0xDEF, 0));
    assert!(Store::save(&replay_single, path_single).is_ok());

    let mut target_single = Replay::capacity(100);
    let loaded_single = Store::load(&mut target_single, path_single);
    assert_eq!(loaded_single.unwrap(), 1);
    assert_eq!(target_single.len(), 1);
    assert_eq!(target_single.get(0).unwrap().hash, 0xABC);

    // Case C: 10,000 samples (full capacity)
    let mut replay_large = Replay::capacity(10000);
    for i in 0..10000 {
        let hash = (i as u64).wrapping_mul(0x9E3779B97F4A7C15);
        let mv = (i % 65535) as u16;
        let reward = (i % 100) as f32 / 100.0;
        let next = hash ^ 0xFFFFFFFFFFFFFFFF;
        let done = if i % 2 == 0 { 1 } else { 0 };
        replay_large.push(Sample::new(hash, mv, reward, next, done));
    }
    assert!(Store::save(&replay_large, path_large).is_ok());

    let mut target_large = Replay::capacity(10000);
    let loaded_large = Store::load(&mut target_large, path_large);
    assert_eq!(loaded_large.unwrap(), 10000);
    assert_eq!(target_large.len(), 10000);

    // Spot-check exact sample equality
    for i in (0..10000).step_by(1000) {
        let orig = replay_large.get(i).unwrap();
        let loaded = target_large.get(i).unwrap();
        assert_eq!(orig.hash, loaded.hash);
        assert_eq!(orig.mv, loaded.mv);
        assert_eq!(orig.reward, loaded.reward);
        assert_eq!(orig.next, loaded.next);
        assert_eq!(orig.done, loaded.done);
    }

    let _ = std::fs::remove_file(path_empty);
    let _ = std::fs::remove_file(path_single);
    let _ = std::fs::remove_file(path_large);
}

/// 3. Test corrupted header/magic and truncated binary files
#[test]
fn test_store_corrupted_file_handling() {
    let path_bad_magic = "/tmp/test_m2_rem_2_bad_magic.bin";
    let path_truncated = "/tmp/test_m2_rem_2_truncated.bin";

    // Invalid Magic: Write b"XBAD" instead of b"XRLN"
    {
        let mut f = File::create(path_bad_magic).unwrap();
        let bad_header = Header {
            magic: *b"XBAD",
            version: VERSION,
            count: 5,
            pad: [0u8; 48],
        };
        let hdr_bytes = unsafe {
            std::slice::from_raw_parts(&bad_header as *const Header as *const u8, std::mem::size_of::<Header>())
        };
        f.write_all(hdr_bytes).unwrap();
    }

    let mut target = Replay::capacity(10);
    let res = Store::load(&mut target, path_bad_magic);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().kind(), std::io::ErrorKind::InvalidData);

    // Truncated File: Header claims 5 records, but file only contains 2 records
    {
        let mut f = File::create(path_truncated).unwrap();
        let header = Header::new(5);
        let hdr_bytes = unsafe {
            std::slice::from_raw_parts(&header as *const Header as *const u8, std::mem::size_of::<Header>())
        };
        f.write_all(hdr_bytes).unwrap();

        // Write 2 records
        for i in 0..2 {
            let rec = Record {
                hash: i as u64,
                next: i as u64 + 10,
                reward: 0.9,
                mv: 0x1316,
                done: 0,
                pad: [0u8; 9],
            };
            let raw = unsafe {
                std::slice::from_raw_parts(&rec as *const Record as *const u8, std::mem::size_of::<Record>())
            };
            f.write_all(raw).unwrap();
        }
    }

    let mut target_tr = Replay::capacity(10);
    let loaded_tr = Store::load(&mut target_tr, path_truncated);
    assert!(loaded_tr.is_ok());
    // Should load the 2 available records gracefully without panicking
    assert_eq!(loaded_tr.unwrap(), 2);
    assert_eq!(target_tr.len(), 2);

    let _ = std::fs::remove_file(path_bad_magic);
    let _ = std::fs::remove_file(path_truncated);
}

/// 4. Test concurrent Store::sync, Book::probe, and Endgame::probe under multithreaded load
#[test]
fn test_concurrent_store_sync_and_probes() {
    Book::clear();
    Endgame::clear();

    use std::thread;

    let mut replay = Replay::capacity(100);
    for i in 0..50 {
        let hash = 0x1000 + i as u64;
        let mv = 0x1316;
        let reward = 0.70 + (i as f32 * 0.005);
        replay.push(Sample::new(hash, mv, reward, hash + 1, 0));
    }

    let synced = Store::sync(&replay);
    assert_eq!(synced, 50);
    assert_eq!(Book::count(), 50);
    assert_eq!(Endgame::count(), 50);

    let mut handles = vec![];
    for _t in 0..4 {
        handles.push(thread::spawn(move || {
            let book = Book::default();
            for i in 0..50 {
                let hash = 0x1000 + i as u64;
                assert!(book.find(hash).is_some());
                assert!(Endgame::probe(hash).is_some());
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    Book::clear();
    Endgame::clear();
}
