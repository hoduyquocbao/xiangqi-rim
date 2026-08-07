# tests/test_gpu_mine_empirical.py
# Empirical test harness for scripts/gpu_mine.py multi-processing and Rust binary fallback.
# Identifiers in English, comments in Vietnamese.

import os
import sys
import time
import json
import subprocess
import shutil

# Make sure project root is in path
sys.path.insert(0, os.path.abspath("."))

def test_multiprocessing_batching():
    """Kiểm thử cơ chế đa tiến trình (ProcessPoolExecutor) khi không có Rust binary."""
    print("--- TEST 1: Python Multi-Processing Batching ---")
    from scripts.gpu_mine import mine, verify
    
    # Đảm bảo binary Rust tạm thời không can thiệp
    binary_path = "target/release/examples/17_mine_dataset"
    backup_path = "target/release/examples/17_mine_dataset.bak"
    has_binary = os.path.exists(binary_path)
    if has_binary:
        shutil.move(binary_path, backup_path)
        
    try:
        start_time = time.time()
        count = 10
        samples = mine(count)
        elapsed = time.time() - start_time
        
        print(f"  - Số ván cờ yêu cầu: {count}")
        print(f"  - Số mẫu cờ sinh ra: {len(samples)}")
        print(f"  - Thời gian thực thi: {elapsed:.3f} giây")
        
        assert len(samples) > 0, "Lỗi: Không sinh được mẫu dữ liệu nào từ Multi-Processing!"
        
        # Validation schema từng mẫu
        for idx, sample in enumerate(samples):
            valid = verify(sample)
            assert valid, f"Lỗi schema tại mẫu index {idx}: {sample}"
            assert sample["move"] in sample["completion"], f"Lỗi completion không chứa nước đi tại index {idx}"
            
        print("✅ TEST 1 PASSED: Multi-processing batching hoạt động chính xác!")
        return True, len(samples), elapsed
    finally:
        if has_binary and os.path.exists(backup_path):
            shutil.move(backup_path, binary_path)

def test_rust_binary_execution():
    """Kiểm thử việc gọi Rust binary compiled khi tệp tồn tại và chạy thành công."""
    print("\n--- TEST 2: Rust Binary Execution ---")
    binary_path = "target/release/examples/17_mine_dataset"
    
    # Biên dịch binary nếu chưa có
    if not os.path.exists(binary_path):
        print("  - Đang biên dịch target/release/examples/17_mine_dataset...")
        res = subprocess.run(["cargo", "build", "--release", "--example", "17_mine_dataset"], capture_output=True, text=True)
        assert res.returncode == 0, f"Lỗi biên dịch Rust binary: {res.stderr}"
        
    from scripts.gpu_mine import mine
    
    start_time = time.time()
    count = 5
    # Chạy mine với Rust binary
    samples = mine(count)
    elapsed = time.time() - start_time
    
    # Khi Rust binary chạy thành công, mine() trả về [] và tạo file trong data/real_mined_*.json
    assert samples == [], "Lỗi: mine() phải trả về list rỗng khi Rust binary thực thi thành công!"
    
    print(f"  - Thời gian thực thi Rust Engine: {elapsed:.3f} giây")
    print("✅ TEST 2 PASSED: Rust binary execution thành công!")
    return True, elapsed

def test_rust_binary_fallback_on_failure():
    """Kiểm thử cơ chế fallback sang Python multi-processing khi Rust binary gặp lỗi (non-zero exit code)."""
    print("\n--- TEST 3: Rust Binary Failure Fallback ---")
    binary_path = "target/release/examples/17_mine_dataset"
    backup_path = "target/release/examples/17_mine_dataset.real_bak"
    dummy_path = binary_path
    
    has_binary = os.path.exists(binary_path)
    if has_binary:
        shutil.move(binary_path, backup_path)
        
    os.makedirs(os.path.dirname(dummy_path), exist_ok=True)
    # Tạo một script giả mạo trả về exit code 1 (thất bại)
    with open(dummy_path, "w") as f:
        f.write("#!/bin/sh\nexit 1\n")
    os.chmod(dummy_path, 0o755)
    
    try:
        from scripts.gpu_mine import mine
        samples = mine(5)
        
        # Bắt buộc phải fallback thành công về Python multi-processing và trả về mẫu
        assert len(samples) > 0, "Lỗi: Fallback thất bại, không nhận được samples từ Python multi-processing!"
        print("✅ TEST 3 PASSED: Fallback sang Python multi-processing hoạt động chuẩn xác khi Rust binary thất bại!")
        return True
    finally:
        if os.path.exists(dummy_path):
            os.remove(dummy_path)
        if has_binary and os.path.exists(backup_path):
            shutil.move(backup_path, binary_path)

if __name__ == "__main__":
    print("============================================================")
    print(" EMPIRICAL TEST SUITE: DATA MINER & RUST FALLBACK")
    print("============================================================")
    test_multiprocessing_batching()
    test_rust_binary_execution()
    test_rust_binary_fallback_on_failure()
    print("============================================================")
    print(" 🎉 ALL MINER EMPIRICAL TESTS PASSED SUCCESSFULLY!")
    print("============================================================")
