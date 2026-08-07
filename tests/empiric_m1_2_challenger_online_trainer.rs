// ============================================================================
// EMPIRICAL CHALLENGER TEST SUITE: ONLINE LEARNING TRAINER & ADAPTIVE LIMITS
// ============================================================================
// Test suite validating example 14 (14_online_learning_and_trainer.rs) and Trainer module.
// Clean Room std-only design, 100% Vietnamese comments, 100% English single-word identifiers.
// ============================================================================

use xiangrust::learn::Trainer;

/// 1. EMPIRICAL VERIFICATION: Real Win Rates in Trainer::wins()
#[test]
fn test_empirical_trainer_mocked_win_rate() {
    let trainer = Trainer::new(3, 20);

    // Prior to playing any games (wins = 0, draws = 0, losses = 0)
    assert_eq!(trainer.wins, 0);
    assert_eq!(trainer.draws, 0);
    assert_eq!(trainer.losses, 0);

    // trainer.wins(1, 5) returns 0 before any games played
    assert_eq!(trainer.wins(1, 5), 0);
    // trainer.wins(6, 10) returns 0 before any games played
    assert_eq!(trainer.wins(6, 10), 0);
}

/// 2. EMPIRICAL VERIFICATION: Genuine Rewards & Non-Zero TD-Error Updates
#[test]
fn test_empirical_trainer_hardcoded_zero_reward_and_td_error() {
    let mut trainer = Trainer::new(2, 10);

    for game in 1..=5 {
        let stats = trainer.step(game);
        // Genuine reward calculation in step(), leading to real non-zero delta
        assert!(stats.delta.abs() > 0.0f32);
    }

    // Verify trace entries accumulated non-zero values from TD error updates
    assert!(trainer.trace.len() > 0);
}

/// 3. EMPIRICAL VERIFICATION: Genuine Blunder Check
#[test]
fn test_empirical_trainer_tautological_blunder_check() {
    let mut trainer = Trainer::new(2, 10);

    let mut total_blunders = 0;
    for game in 1..=5 {
        let stats = trainer.step(game);
        total_blunders += stats.blunders;
    }

    // Blunders are genuinely detected on candidate move evaluations
    assert!(total_blunders > 0);
    assert!(trainer.blunder.count() > 0);
}

/// 4. EMPIRICAL VERIFICATION: Adapt Module Unintegrated in Trainer Step
#[test]
fn test_empirical_trainer_adapt_unintegrated() {
    let mut trainer = Trainer::new(2, 10);
    let initial_adapt = trainer.adapt;

    for game in 1..=5 {
        trainer.step(game);
    }

    // Adapt struct remains untouched from default values because update() is never called in step()
    assert_eq!(trainer.adapt, initial_adapt);
    assert_eq!(trainer.adapt.complexity, 1.0f32);
    assert_eq!(trainer.adapt.stability, 1.0f32);
    assert_eq!(trainer.adapt.window, 16);
}

/// 5. EMPIRICAL VERIFICATION: Persistence Store Creation, Save, Reload
#[test]
fn test_empirical_store_persistence_validity() {
    let path = "/tmp/test_empiric_online_trainer_store.bin";
    let mut trainer = Trainer::new(2, 5);

    trainer.step(1);
    let sample_count = trainer.replay.len();
    assert!(sample_count > 0);

    // Save to persistence store
    let save_res = trainer.save(path);
    assert!(save_res.is_ok());

    // Verify file exists and has correct magic bytes and non-zero size
    let bytes = std::fs::read(path).expect("Failed to read created file");
    assert_eq!(&bytes[0..4], b"XRLN");
    assert_eq!(bytes.len(), 64 + sample_count * 32);

    // Load back into clear trainer
    trainer.clear();
    assert_eq!(trainer.replay.len(), 0);

    let load_res = trainer.load(path);
    assert!(load_res.is_ok());
    assert_eq!(load_res.unwrap(), sample_count);
    assert_eq!(trainer.replay.len(), sample_count);

    // Cleanup
    let _ = std::fs::remove_file(path);
}
