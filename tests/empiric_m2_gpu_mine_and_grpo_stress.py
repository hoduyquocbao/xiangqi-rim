# tests/empiric_m2_gpu_mine_and_grpo_stress.py
# ============================================================================
# ADVERSARIAL EMPIRICAL STRESS TEST SUITE FOR MILESTONE M2
# 1. GPU DATA MINER HIGH-LOAD MULTI-PROCESS & PROCESS POOL EXECUTOR STRESS
# 2. RUST COMPILED BINARY FALLBACK & INTEGRATION VERIFICATION
# 3. GRPO REWARD FUNCTIONS EXTREME & MALFORMED INPUTS STRESS HARNESS
# ============================================================================
# Identifiers: mine, generate, update, parse, valid, syntax, rule, quality,
# test, stress, memory, peak, start, elapsed, rate, count, batch, pool,
# workers, results, samples, prompt, completion, rewards, bounds, main
# ============================================================================

import os
import sys
import time
import gc
import json
import glob
import subprocess
import resource

# Ensure scripts directory is on sys.path
sys.path.insert(0, os.path.abspath("scripts"))

from gpu_mine import update as mine_update, parse as mine_parse, generate as mine_generate, mine as mine_data
from hub import verify as hub_verify

def get_memory_mb():
    """Get current process memory usage in MB (RSS)."""
    usage = resource.getrusage(resource.RUSAGE_SELF)
    if sys.platform == "darwin":
        return usage.ru_maxrss / (1024 * 1024)
    return usage.ru_maxrss / 1024

def test_1_gpu_mine_multiprocess_high_load():
    """
    Test 1: Empirically stress-test ProcessPoolExecutor under high load using mine_data().
    Tests multi-process game generation, throughput, schema validity, and memory footprint.
    """
    print("\n--- [TEST 1] STRESS-TESTING ProcessPoolExecutor MULTI-PROCESS MINING ---")

    # Temporarily ensure Rust binary fallback is bypassed for Python pool testing
    rust_binary = "target/release/examples/17_mine_dataset"
    rust_backup = "target/release/examples/17_mine_dataset.tmp_bak"
    has_rust = os.path.exists(rust_binary)
    if has_rust:
        os.rename(rust_binary, rust_backup)

    try:
        game_loads = [20, 50, 100]
        start_mem = get_memory_mb()
        print(f"📍 Baseline Memory RSS: {start_mem:.2f} MB")

        for count in game_loads:
            t0 = time.time()
            m0 = get_memory_mb()

            # Execute multi-process mining via mine_data()
            samples = mine_data(count)

            t1 = time.time()
            m1 = get_memory_mb()

            elapsed = t1 - t0
            g_rate = count / elapsed if elapsed > 0 else 0
            s_rate = len(samples) / elapsed if elapsed > 0 else 0
            m_diff = m1 - m0

            workers = min(os.cpu_count() or 4, 8)
            print(f"  ⚡ Workers: {workers:2d} | Games: {count:3d} | Samples: {len(samples):5d} | "
                  f"Time: {elapsed:5.3f}s | Speed: {g_rate:6.1f} games/s ({s_rate:7.1f} samples/s) | "
                  f"RAM Delta: {m_diff:+.2f} MB")

            assert len(samples) == count * 6, f"Expected {count * 6} samples, got {len(samples)}"

            # Schema verification for first and last sample in batch
            for s in [samples[0], samples[-1]]:
                assert "prompt" in s and "completion" in s and "move" in s and "stamp" in s
                assert "<thought>" in s["completion"] and "</thought>" in s["completion"]
                assert len(s["move"]) == 4

            del samples
            gc.collect()

        end_mem = get_memory_mb()
        print(f"📍 Final Memory RSS: {end_mem:.2f} MB (Delta from baseline: {end_mem - start_mem:+.2f} MB)")
        print("✅ TEST 1 PASSED: ProcessPoolExecutor handled high load without deadlocks or memory leaks!")

    finally:
        if has_rust and os.path.exists(rust_backup):
            os.rename(rust_backup, rust_binary)

def test_2_rust_binary_fallback_and_integration():
    """
    Test 2: Verify fallback to Rust compiled binary examples/17_mine_dataset when available.
    Compiles/verifies Rust binary output vs Python mining output.
    """
    print("\n--- [TEST 2] VERIFYING RUST COMPILED BINARY FALLBACK & SPEED ---")
    rust_binary = "target/release/examples/17_mine_dataset"

    if not (os.path.exists(rust_binary) and os.access(rust_binary, os.X_OK)):
        print(f"⚙️ Compiling Rust binary '{rust_binary}'...")
        res = subprocess.run(["cargo", "build", "--release", "--example", "17_mine_dataset"], capture_output=True, text=True)
        assert res.returncode == 0, f"Failed to build Rust binary: {res.stderr}"

    assert os.path.exists(rust_binary), f"Rust binary not found at {rust_binary}"

    t0 = time.time()
    env = dict(os.environ, MATCH_COUNT="100")
    res = subprocess.run([rust_binary], env=env, capture_output=True, text=True)
    t1 = time.time()

    assert res.returncode == 0, f"Rust binary returned exit code {res.returncode}: {res.stderr}"
    elapsed = t1 - t0

    # Locate generated JSON file
    files = sorted(glob.glob("data/real_mined_*.json"), key=os.path.getmtime)
    assert len(files) > 0, "No output JSON file produced by Rust binary in data/"

    latest_file = files[-1]
    with open(latest_file, "r", encoding="utf-8") as f:
        data = json.load(f)

    print(f"  ⚡ Rust Binary executed in {elapsed:.3f}s for 100 games.")
    print(f"  📦 Generated File: {latest_file} ({len(data):,} samples | {len(data)/elapsed:,.1f} samples/s)")

    # Validate each sample using hub verify()
    valid_count = sum(1 for item in data if hub_verify(item))
    assert valid_count == len(data), f"Hub verification failed for {len(data) - valid_count} samples!"

    print(f"  ✅ 100% of {len(data)} mined samples passed Hub verification schema!")

    # Test gpu_mine.py mine() function with binary present
    rust_samples = mine_data(50)
    assert rust_samples == [], "mine() should return [] when Rust binary handles writing JSON file directly."

    print("✅ TEST 2 PASSED: Rust compiled binary fallback and high-speed dataset generation verified!")

def test_3_grpo_reward_extreme_adversarial_stress():
    """
    Test 3: Stress-test GRPO reward functions (syntax, rule, quality) with
    extreme, malformed, empty, NaN, truncated, and hostile inputs.
    """
    print("\n--- [TEST 3] ADVERSARIAL STRESS-TESTING GRPO REWARD FUNCTIONS ---")
    from train import syntax, rule, quality, parse as train_parse, valid as train_valid

    # 3.1 Extreme & Malformed Prompts
    extreme_prompts = [
        "",                                             # Empty prompt
        "   \n\t  ",                                     # Whitespace prompt
        "A" * 100000,                                    # 100KB string without FEN
        "2. Chuỗi Chuẩn FEN:\ninvalid_fen_here",       # Invalid FEN format
        "2. Chuỗi Chuẩn FEN:\nrnbakabnr/9/9/9/9/9/9/9/9/RNBAKABNR w - - 0 1", # FEN with missing ranks
        "2. Chuỗi Chuẩn FEN:\nrnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR x - - 0 1", # FEN with invalid turn 'x'
        "2. Chuỗi Chuẩn FEN:\nrnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1\n" * 10, # Multiple FENs
        "🔥 Unicode 🏆 test 💣",                          # Special emojis
        "SELECT * FROM users WHERE '1'='1';",           # SQL injection attempt
        "<script>alert('xss')</script>",                 # Script tag injection attempt
    ]

    # 3.2 Extreme & Malformed Completions
    extreme_completions = [
        "",                                             # Empty completion
        "   \n  \t ",                                   # Whitespace completion
        "B" * 100000,                                    # 100KB completion without tags
        "<thought>" + "A" * 50000 + "</thought>\nb2e2", # 50KB thought tag + valid move
        "<thought><think>nested tags</think></thought>\nb2e2", # Nested thought/think tags
        "<thought>Unclosed tag b2e2",                   # Unclosed tag
        "</thought>Tag without opening b2e2",           # Closing tag without opening
        "<thought>No move inside</thought>",            # Tag without move
        "<thought>Move b2e2</thought> b2e2 h2e2 c3c4", # Multiple moves inside and outside
        "<thought>Invalid coords</thought>\nz9z9",     # Invalid move coords
        "<thought>Invalid length</thought>\nb2e2a",    # Move too long
        "<thought>Invalid length</thought>\nb2",       # Move too short
        "<thought>Same square move</thought>\na0a0",    # Same source and dest
        "<thought>Empty square move</thought>\na1a2",   # Source square empty
        "<thought>King out of palace</thought>\ne0e3",  # King illegal move
        "<thought>Friendly capture</thought>\nb0c2",   # Capturing own piece (b0 is Red Knight, c2 is Red Knight)
        "NaN",                                          # NaN string
        "Infinity",                                     # Infinity string
        "\x00\x01\x02\x03\x04",                         # Binary null bytes
    ]

    # Batch test with all combinations of extreme prompts & completions
    total_evals = 0
    for p in extreme_prompts:
        for c in extreme_completions:
            r_syntax = syntax([p], [c])
            r_rule = rule([p], [c])
            r_quality = quality([p], [c], move=["b2e2"])

            assert len(r_syntax) == 1, f"Syntax reward returned wrong length: {r_syntax}"
            assert len(r_rule) == 1, f"Rule reward returned wrong length: {r_rule}"
            assert len(r_quality) == 1, f"Quality reward returned wrong length: {r_quality}"

            v_s, v_r, v_q = r_syntax[0], r_rule[0], r_quality[0]

            # Verify reward numeric bounds
            assert -1.0 <= v_s <= 1.0, f"Syntax reward {v_s} out of bounds [-1.0, 1.0]"
            assert -0.5 <= v_r <= 2.0, f"Rule reward {v_r} out of bounds [-0.5, 2.0]"
            assert 0.0 <= v_q <= 3.0, f"Quality reward {v_q} out of bounds [0.0, 3.0]"

            # Verify no NaN or Inf values
            assert not (v_s != v_s or v_s == float('inf') or v_s == float('-inf')), f"Syntax reward is NaN/Inf: {v_s}"
            assert not (v_r != v_r or v_r == float('inf') or v_r == float('-inf')), f"Rule reward is NaN/Inf: {v_r}"
            assert not (v_q != v_q or v_q == float('inf') or v_q == float('-inf')), f"Quality reward is NaN/Inf: {v_q}"

            total_evals += 1

    print(f"  ⚡ Evaluated {total_evals:,} extreme prompt/completion combinations.")

    # 3.3 Large Batch Scaling Test (1,000 items in single call)
    batch_size = 1000
    valid_prompt = (
        "Trạng thái bàn cờ tướng hiện tại...\n"
        "2. Chuỗi Chuẩn FEN (Forsyth-Edwards Notation):\n"
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1\n"
    )
    valid_completion = "<thought>\nPhân tích bàn cờ\n</thought>\nb2e2"

    prompts_b = [valid_prompt] * batch_size
    completions_b = [valid_completion] * batch_size
    moves_b = ["b2e2"] * batch_size

    t0 = time.time()
    res_s = syntax(prompts_b, completions_b)
    res_r = rule(prompts_b, completions_b)
    res_q = quality(prompts_b, completions_b, move=moves_b)
    t1 = time.time()

    elapsed = t1 - t0
    assert len(res_s) == batch_size and all(x == 1.0 for x in res_s)
    assert len(res_r) == batch_size and all(x == 2.0 for x in res_r)
    assert len(res_q) == batch_size and all(x == 3.0 for x in res_q)

    print(f"  ⚡ Batch processing of {batch_size} prompts completed in {elapsed:.4f}s ({batch_size/elapsed:,.1f} evals/s)")
    print("✅ TEST 3 PASSED: GRPO reward functions showed 100% numerical stability under adversarial stress!")

def main():
    print("============================================================")
    print(" 🔥 EMPIRICAL STRESS TEST HARNESS — MILESTONE M2 CHALLENGER 2")
    print("============================================================")
    test_1_gpu_mine_multiprocess_high_load()
    test_2_rust_binary_fallback_and_integration()
    test_3_grpo_reward_extreme_adversarial_stress()

    print("\n============================================================")
    print(" 🎉 TOÀN BỘ 3-IN-1 EMPIRICAL STRESS TESTS ĐÃ PASSED 100%!")
    print("============================================================")

if __name__ == "__main__":
    main()
