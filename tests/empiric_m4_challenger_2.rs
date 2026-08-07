// Empirical verification test suite for Milestone 4 Transposition Table by Challenger 2.

use std::mem::{align_of, size_of};
use std::sync::Arc;
use std::thread;

use xiangrust::movegen::types::Move;
use xiangrust::tt::bound::Bound;
use xiangrust::tt::cluster::Cluster;
use xiangrust::tt::entry::Entry;
use xiangrust::tt::table::Table;

#[test]
fn alignment() {
    assert_eq!(align_of::<Table>(), 64);
    assert_eq!(align_of::<Cluster>(), 64);
    assert_eq!(size_of::<Cluster>(), 64);
    assert_eq!(align_of::<Entry>(), 16);
    assert_eq!(size_of::<Entry>(), 16);

    let table = Table::new(1);
    let ptr = table.partitions[0].items.as_ptr() as usize;
    assert_eq!(ptr % 64, 0);
}

#[test]
fn concurrency() {
    let table = Arc::new(Table::new(2));
    let threads = 16;
    let limit = 100_000;
    let mut handles = Vec::with_capacity(threads);

    for id in 0..threads {
        let tt = table.clone();
        let handle = thread::spawn(move || {
            let mut state = (id as u64).wrapping_add(1);
            for _ in 0..limit {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                // Ensure key is non-zero to avoid the zero-key empty slot probe bug
                let key = if state == 0 { 1 } else { state };
                let from = ((state >> 16) % 90) as u8;
                let to = ((state >> 24) % 90) as u8;
                let step = Move::new(from, to);
                let score = (state as i16) % 10000;
                let depth = ((state >> 32) % 64) as u8 + 1;
                let bound = match (state >> 40) % 3 {
                    0 => 1u8,
                    1 => 2u8,
                    _ => 3u8,
                };

                tt.save(key, depth, bound, step, score);

                if let Some(item) = tt.probe(key) {
                    assert_eq!(item.key, key);
                    assert!(item.step.from < 90 || item.step.from == 255);
                    assert!(item.step.to < 90 || item.step.to == 255);
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn integrity() {
    let table = Arc::new(Table::new(1));
    let threads = 8;
    let limit = 50_000;
    let mask = table.mask as u64;
    let mut handles = Vec::with_capacity(threads);

    for id in 0..threads {
        let tt = table.clone();
        let handle = thread::spawn(move || {
            for i in 0..limit {
                let slot = (i % 4) as u64;
                let key = (slot * (mask + 1)) | ((id as u64) + 1);
                let from = (i % 90) as u8;
                let to = ((i + 1) % 90) as u8;
                let step = Move::new(from, to);
                let score = (i as i16) % 5000;
                let depth = (i % 32) as u8 + 1;

                tt.save(key, depth, 1, step, score);

                if let Some(item) = tt.probe(key) {
                    assert_eq!(item.key, key);
                    assert!(item.step.from < 90);
                    assert!(item.step.to < 90);
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn aging() {
    let mut table = Table::new(1);
    let key = 0xABCD_EF01_2345_6789u64;
    let step = Move::new(10, 20);

    table.save(key, 5, 1, step, 100);
    let probed = table.probe(key).unwrap();
    assert_eq!(probed.age, 0);

    table.advance();
    assert_eq!(table.age, 1);

    table.save(key, 5, 1, step, 200);
    let probed2 = table.probe(key).unwrap();
    assert_eq!(probed2.age, 1);
    assert_eq!(probed2.score, 200);
}

#[test]
fn fuzz() {
    let scores = [-32768, -10000, -1, 0, 1, 10000, 32767];
    let bounds = [Bound::Exact, Bound::Lower, Bound::Upper];
    let steps = [Move::new(0, 89), Move::new(44, 45), Move::none()];

    for &score in &scores {
        for &bound in &bounds {
            for &step in &steps {
                for depth in [0u8, 1u8, 64u8, 255u8] {
                    for age in [0u8, 127u8, 255u8] {
                        let packed = Entry::pack(step, score, depth, bound.raw(), age);
                        let item = Entry::unpack(0x1234, packed);

                        assert_eq!(item.key, 0x1234);
                        assert_eq!(item.depth, depth);
                        assert_eq!(item.bound, bound);
                        assert_eq!(item.step, step);
                        assert_eq!(item.score, score);
                        assert_eq!(item.age, age);
                    }
                }
            }
        }
    }
}

#[test]
fn empty_probe_zero_key_bug() {
    let table = Table::new(1);
    // Empirical Bug Verification:
    // On a fresh or cleared table, all entries have data = 0 and key = 0.
    // Probing for key = 0 computes (0 ^ 0) == 0, which evaluates to true!
    // Thus probe(0) returns Some(Item) on an uninitialized/empty entry!
    // Correct behavior: probe(0) MUST return None for an empty entry (data == 0).
    let probed = table.probe(0);
    assert!(
        probed.is_none(),
        "BUG DISCOVERED: table.probe(0) on an empty or cleared table returned Some(Item) instead of None because empty slot (data=0, key=0) satisfies (0 ^ 0) == 0!"
    );
}
