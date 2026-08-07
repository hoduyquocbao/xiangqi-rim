// ============================================================================
// EMPIRICAL STRESS TEST SUITE FOR MILESTONE 2: DYNAMIC SYNC & PERSISTENCE
// ============================================================================
// File: tests/empiric_m2_sync_concurrency_stress.rs
// Purpose: Stress-test Book::sync, Endgame::sync, Store::sync_book, and binary
//          persistence under heavy concurrent read/write probes (RwLock).
// ============================================================================

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use xiangrust::board::Parser;
use xiangrust::book::endgame::{Endgame, WIN};
use xiangrust::book::opening::{Book, ENTRIES};
use xiangrust::learn::replay::{Replay, Sample};
use xiangrust::learn::store::Store;
use xiangrust::movegen::Move;

static TEST_MUTEX: Mutex<()> = Mutex::new(());

/// Test 1: Concurrency & Lock Stress test for Book::sync and Endgame::sync under heavy multi-threading.
#[test]
fn test_concurrent_book_and_endgame_sync_and_probe() {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    Book::clear();
    Endgame::clear();

    let threads = 16;
    let iterations = 10_000;
    let running = Arc::new(AtomicBool::new(true));

    let mut handles = Vec::with_capacity(threads);

    for id in 0..threads {
        let flag = Arc::clone(&running);
        let handle = thread::spawn(move || {
            let mut count = 0;
            while flag.load(Ordering::Relaxed) && count < iterations {
                let hash = (id as u64 * 100_000 + count as u64) ^ 0xDEADBEEFCAFEBABE;
                let mv = ((id as u16) << 8) | (count as u16 & 0xFF);
                let weight = (500 + (count % 500)) as u16;
                let score = (count % 15_000) as i32;

                if id % 2 == 0 {
                    // Writer thread
                    let synced_b = Book::sync(hash, mv, weight);
                    let synced_e = Endgame::sync(hash, score);
                    assert!(synced_b, "Book::sync failed to acquire lock!");
                    assert!(synced_e, "Endgame::sync failed to acquire lock!");
                } else {
                    // Reader thread
                    let book = Book::default();
                    let _ = book.find(hash);
                    let _ = Endgame::probe(hash);
                    let _ = Book::count();
                    let _ = Endgame::count();
                }

                count += 1;
            }
        });
        handles.push(handle);
    }

    // Allow threads to run for a bit
    thread::sleep(Duration::from_millis(100));
    running.store(false, Ordering::Relaxed);

    for handle in handles {
        handle.join().expect("Thread panicked during dynamic sync stress test!");
    }

    assert!(Book::count() > 0, "DYNAMIC_BOOK count should be non-zero after concurrent sync");
    assert!(Endgame::count() > 0, "DYNAMIC_ENDGAME count should be non-zero after concurrent sync");

    Book::clear();
    Endgame::clear();
}

/// Test 2: Dynamic Opening Book entries override static entries & weight updates work correctly.
#[test]
fn test_dynamic_book_override_and_supplement_static() {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    Book::clear();
    assert_eq!(Book::count(), 0);

    // Pick a static entry hash from ENTRIES
    let static_entry = ENTRIES[0];
    let static_hash = static_entry.hash;
    let book = Book::default();

    // Verify initial probe matches static move
    let initial_mv = book.find(static_hash);
    assert!(initial_mv.is_some());
    let raw_initial = initial_mv.unwrap().raw();
    assert_eq!(raw_initial, static_entry.mv);

    // Override static entry with dynamic entry having a DIFFERENT move (0x9999)
    let override_mv = 0x9999u16;
    let synced = Book::sync(static_hash, override_mv, 999);
    assert!(synced);
    assert_eq!(Book::count(), 1);

    // Verify probe now returns override_mv instead of static_mv (Dynamic Overrides Static!)
    let probed_override = book.find(static_hash);
    assert!(probed_override.is_some());
    assert_eq!(
        probed_override.unwrap().raw(),
        override_mv,
        "DYNAMIC_BOOK entry MUST override static BOOK_ENTRIES!"
    );

    // Test weight update rules: lower weight should NOT overwrite higher weight
    let lower_weight_mv = 0x1111u16;
    Book::sync(static_hash, lower_weight_mv, 500); // lower than 999
    assert_eq!(
        book.find(static_hash).unwrap().raw(),
        override_mv,
        "Book::sync MUST NOT overwrite move if weight is lower!"
    );

    // Higher weight SHOULD overwrite
    let higher_weight_mv = 0x8888u16;
    Book::sync(static_hash, higher_weight_mv, 1000); // higher than 999
    assert_eq!(
        book.find(static_hash).unwrap().raw(),
        higher_weight_mv,
        "Book::sync MUST overwrite move if weight is higher!"
    );

    // Test supplement: dynamic entry for a non-static hash
    let brand_new_hash = 0x123456789ABCDEF0u64;
    let brand_new_mv = 0x1316u16;
    Book::sync(brand_new_hash, brand_new_mv, 750);
    assert_eq!(Book::count(), 2);
    assert_eq!(book.find(brand_new_hash), Some(Move::new(0x13, 0x16)));

    Book::clear();
    assert_eq!(Book::count(), 0);
}

/// Test 3: Dynamic Endgame Memory entries override/supplement theoretical endgame eval.
#[test]
fn test_dynamic_endgame_override_and_eval() {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    Endgame::clear();
    assert_eq!(Endgame::count(), 0);

    // Initial position has default eval = None in Endgame
    let pos = Parser::parse(Parser::DEFAULT);
    assert_eq!(Endgame::eval(&pos), None);

    // Dynamic sync a custom score (9800 cp) for initial position hash
    let custom_score = 9800i32;
    let synced = Endgame::sync(pos.hash, custom_score);
    assert!(synced);
    assert_eq!(Endgame::count(), 1);

    // Verify probe & eval return custom_score
    assert_eq!(Endgame::probe(pos.hash), Some(custom_score));
    assert_eq!(
        Endgame::eval(&pos),
        Some(custom_score),
        "Endgame::eval MUST return dynamic score when present!"
    );

    // Override score for existing position
    let new_score = WIN;
    Endgame::sync(pos.hash, new_score);
    assert_eq!(Endgame::count(), 1); // count remains 1
    assert_eq!(Endgame::eval(&pos), Some(WIN));

    Endgame::clear();
    assert_eq!(Endgame::count(), 0);
}

/// Test 4: Store::sync_book auto-sync threshold (win rate >= 65%, reward >= 0.65).
#[test]
fn test_store_sync_book_threshold_and_scoring() {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    Book::clear();
    Endgame::clear();

    let mut replay = Replay::capacity(10);

    // Sample 1: reward = 0.50 (< 0.65) -> MUST NOT SYNC
    replay.push(Sample::new(1001, 0x1111, 0.50, 2001, 0));

    // Sample 2: reward = 0.649 (< 0.65) -> MUST NOT SYNC
    replay.push(Sample::new(1002, 0x2222, 0.649, 2002, 0));

    // Sample 3: reward = 0.65 (>= 0.65) -> MUST SYNC (weight = 650, score = 0.65 * 15000 = 9750)
    replay.push(Sample::new(1003, 0x3333, 0.65, 2003, 0));

    // Sample 4: reward = 0.80 (>= 0.65) -> MUST SYNC (weight = 800, score = 0.80 * 15000 = 12000)
    replay.push(Sample::new(1004, 0x4444, 0.80, 2004, 0));

    // Sample 5: reward = 0.95 (>= 0.90) -> MUST SYNC (weight = 950, score = WIN = 15000)
    replay.push(Sample::new(1005, 0x5555, 0.95, 2005, 1));

    // Execute Store::sync
    let count = Store::sync(&replay);
    assert_eq!(count, 3, "Store::sync MUST sync exactly 3 samples with reward >= 0.65!");

    assert_eq!(Book::count(), 3);
    assert_eq!(Endgame::count(), 3);

    // Verify un-synced hashes
    let book = Book::default();
    assert_eq!(book.find(1001), None);
    assert_eq!(book.find(1002), None);

    // Verify synced hashes & details
    assert_eq!(book.find(1003), Some(Move::new(0x33, 0x33)));
    assert_eq!(book.find(1004), Some(Move::new(0x44, 0x44)));
    assert_eq!(book.find(1005), Some(Move::new(0x55, 0x55)));

    assert_eq!(Endgame::probe(1003), Some(9750));
    assert_eq!(Endgame::probe(1004), Some(12000));
    assert_eq!(Endgame::probe(1005), Some(WIN));

    Book::clear();
    Endgame::clear();
}

/// Test 5: Persistent Binary Store Save/Load Roundtrip & Integrity.
#[test]
fn test_store_persistence_binary_roundtrip() {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let path = "/tmp/test_m2_persistence_stress.bin";

    // Clean up if leftover
    let _ = std::fs::remove_file(path);

    let mut replay = Replay::capacity(500);

    for i in 0..500 {
        let hash = (i as u64 + 1) * 0x0123456789ABCDEF;
        let mv = (i as u16) & 0xFFFF;
        let reward = (i as f32) / 500.0;
        let next = hash ^ 0xFFFFFFFFFFFFFFFF;
        let done = if i % 2 == 0 { 1 } else { 0 };

        replay.push(Sample::new(hash, mv, reward, next, done));
    }

    assert_eq!(replay.len(), 500);

    // Save to disk
    let save_res = Store::save(&replay, path);
    assert!(save_res.is_ok(), "Store::save failed: {:?}", save_res.err());

    // Check exact binary file size (Header 64B + 500 * Record 32B = 16064B)
    let meta = std::fs::metadata(path).expect("Failed to read metadata of saved file!");
    assert_eq!(
        meta.len(),
        64 + 500 * 32,
        "Binary file size MUST match Header (64B) + Record count * 32B!"
    );

    // Load back into a fresh Replay buffer
    let mut loaded_replay = Replay::capacity(500);
    let load_res = Store::load(&mut loaded_replay, path);
    assert!(load_res.is_ok(), "Store::load failed: {:?}", load_res.err());
    assert_eq!(load_res.unwrap(), 500);
    assert_eq!(loaded_replay.len(), 500);

    // Verify content matching byte for byte
    for i in 0..500 {
        let orig = replay.get(i).unwrap();
        let loaded = loaded_replay.get(i).unwrap();

        assert_eq!(loaded.hash, orig.hash);
        assert_eq!(loaded.mv, orig.mv);
        assert_eq!(loaded.reward, orig.reward);
        assert_eq!(loaded.next, orig.next);
        assert_eq!(loaded.done, orig.done);
    }

    // Clean up file
    let _ = std::fs::remove_file(path);
}

/// Test 6: Concurrent Clear and Probes resilience.
#[test]
fn test_rwlock_clear_during_concurrent_probes() {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    Book::clear();
    Endgame::clear();

    let running = Arc::new(AtomicBool::new(true));
    let flag = Arc::clone(&running);

    let handle = thread::spawn(move || {
        let book = Book::default();
        while flag.load(Ordering::Relaxed) {
            let _ = book.find(0x12345678);
            let _ = Endgame::probe(0x12345678);
        }
    });

    for i in 0..100 {
        Book::sync(i as u64, 0x1316, 500);
        Endgame::sync(i as u64, 1000);
        if i % 10 == 0 {
            Book::clear();
            Endgame::clear();
        }
    }

    running.store(false, Ordering::Relaxed);
    handle.join().expect("Reader thread panicked during concurrent clear!");

    Book::clear();
    Endgame::clear();
}
