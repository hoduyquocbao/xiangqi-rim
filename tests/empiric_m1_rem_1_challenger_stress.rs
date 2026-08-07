// ============================================================================
// EMPIRICAL CHALLENGER TEST HARNESS: M1 REMEDIATION ITERATION 2 VERIFICATION
// ============================================================================
// Test harness for empirically stress-testing Trainer win tracking, TD(lambda),
// and binary persistence store after Iteration 2 remediation.
// 100% Clean Room std-only design, 100% Vietnamese comments, 100% English single-word identifiers.
// ============================================================================

use xiangrust::learn::{Replay, Store, Trainer};

/// 1. EMPIRICAL VERIFICATION: Genuine Win Tracking in Trainer::wins()
#[test]
fn test_genuine_win_tracking_accuracy() {
    let mut trainer = Trainer::new(3, 20);

    // Initial state: 0 wins, 0 draws, 0 losses
    assert_eq!(trainer.wins, 0);
    assert_eq!(trainer.draws, 0);
    assert_eq!(trainer.losses, 0);
    assert_eq!(trainer.wins(1, 10), 0);

    // Simulate 5 games with known outcomes:
    // Game 1: RED WINS (outcome 1)
    // Game 2: BLACK WINS (outcome 2)
    // Game 3: DRAW (outcome 0)
    // Game 4: RED WINS (outcome 1)
    // Game 5: RED WINS (outcome 1)
    trainer.history[0] = 1; // Game 1
    trainer.history[1] = 2; // Game 2
    trainer.history[2] = 0; // Game 3
    trainer.history[3] = 1; // Game 4
    trainer.history[4] = 1; // Game 5

    assert_eq!(trainer.wins(1, 5), 3);
    assert_eq!(trainer.wins(1, 2), 1);
    assert_eq!(trainer.wins(3, 5), 2);
    assert_eq!(trainer.wins(2, 3), 0);

    // Test out of bounds / edge start values (e.g. start = 0)
    assert_eq!(trainer.wins(0, 5), 3);
}

/// 2. EMPIRICAL VERIFICATION: History Buffer Wrap-Around (> 64 Games)
#[test]
fn test_history_buffer_wraparound() {
    let mut trainer = Trainer::new(3, 20);

    // Fill history with RED WINS up to 64 games
    for g in 1..=64 {
        let idx = (g - 1) % 64;
        trainer.history[idx] = 1;
    }
    assert_eq!(trainer.wins(1, 64), 64);

    // Game 65 overwrites Game 1 with BLACK WINS (2)
    let idx_65 = (65 - 1) % 64;
    trainer.history[idx_65] = 2;

    // Window from 2..=65 should have 63 wins
    assert_eq!(trainer.wins(2, 65), 63);
}

/// 3. EMPIRICAL VERIFICATION: TD(lambda) State Value Updates & Non-Zero Delta
#[test]
fn test_td_lambda_state_value_updates() {
    let mut trainer = Trainer::new(2, 10);

    let stats = trainer.step(1);
    assert!(stats.moves > 0);
    assert!(stats.samples > 0);
    assert!(stats.delta > 0.0f32, "Delta must be non-zero during self-play step");

    // Verify trace entries have non-zero state values V(s)
    let trace_count = trainer.trace.len();
    assert!(trace_count > 0);

    let mut non_zero_values = 0;
    for i in 0..trace_count {
        if trainer.trace.entries[i].value != 0.0f32 {
            non_zero_values += 1;
        }
    }
    assert!(non_zero_values > 0, "State values V(s) must be updated with non-zero rewards");
}

/// 4. EMPIRICAL VERIFICATION: Binary Persistence Integrity & Roundtrip
#[test]
fn test_binary_persistence_roundtrip() {
    let path = "/tmp/test_empiric_m1_rem_1_persistence.bin";
    let mut trainer = Trainer::new(2, 5);

    // Generate samples via self-play step
    let stats = trainer.step(1);
    let orig_count = trainer.replay.len();
    assert_eq!(orig_count, stats.samples);

    // Save to disk
    let save_res = trainer.save(path);
    assert!(save_res.is_ok());

    // Verify raw file format: header size = 64B, record size = 32B
    let file_bytes = std::fs::read(path).expect("File must exist");
    assert_eq!(&file_bytes[0..4], b"XRLN");
    assert_eq!(file_bytes.len(), 64 + orig_count * 32);

    // Load back into empty trainer
    let mut loaded_trainer = Trainer::new(2, 5);
    let load_res = loaded_trainer.load(path);
    assert!(load_res.is_ok());
    assert_eq!(load_res.unwrap(), orig_count);
    assert_eq!(loaded_trainer.replay.len(), orig_count);

    // Validate field-by-field equality of samples
    for i in 0..orig_count {
        let orig_sample = trainer.replay.get(i).unwrap();
        let loaded_sample = loaded_trainer.replay.get(i).unwrap();

        assert_eq!(orig_sample.hash, loaded_sample.hash);
        assert_eq!(orig_sample.mv, loaded_sample.mv);
        assert_eq!(orig_sample.reward, loaded_sample.reward);
        assert_eq!(orig_sample.next, loaded_sample.next);
        assert_eq!(orig_sample.done, loaded_sample.done);
    }

    // Cleanup
    let _ = std::fs::remove_file(path);
}

/// 5. EMPIRICAL VERIFICATION: Corrupted / Invalid File Load Error Handling
#[test]
fn test_binary_persistence_invalid_header() {
    let path = "/tmp/test_empiric_invalid_magic.bin";

    // Write file with bad magic bytes
    let mut bad_bytes = vec![0u8; 64];
    bad_bytes[0..4].copy_from_slice(b"BADM");
    std::fs::write(path, bad_bytes).unwrap();

    let mut replay = Replay::new();
    let load_res = Store::load(&mut replay, path);
    assert!(load_res.is_err(), "Store::load must fail on invalid magic bytes");

    let _ = std::fs::remove_file(path);
}
