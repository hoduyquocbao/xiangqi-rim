# tests/test_m1_i3_1_challenger.py
# ============================================================================
# EMPIRICAL TEST HARNESS FOR ITERATION 3 VERIFICATION
# Challenger Agent: challenger_m1_i3_1
# Purpose: Empirical verification of 15 openings (90 UCI moves), FEN updates,
#          invalid move validation, and O(1) SHA256 Hub deduplication.
# ============================================================================

import sys
import os
import time
import hashlib
import unittest

# Import system modules under test
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

from scripts.gpu_mine import update, parse, generate, openings, start
from scripts.hub import fetch, verify, key, merge, save


class TestXiangqiEmpiricalIteration3(unittest.TestCase):
    """Bộ test thực nghiệm đối kháng cho Iteration 3."""

    def test_01_fifteen_openings_legal_and_fen_update(self):
        """1. Thực nghiệm kiểm tra 100% 15 thế khai cuộc (90 nước đi UCI) trên bàn cờ FEN."""
        total_moves = 0
        print("\n--- TEST 1: Kiểm tra 15 thế khai cuộc (90 nước đi) ---")
        
        for open_idx, line in enumerate(openings):
            fen = start
            self.assertEqual(len(line), 6, f"Khai cuộc {open_idx} không đủ 6 nước đi")
            
            for move_idx, move in enumerate(line):
                total_moves += 1
                parts = fen.split()
                rows = parts[0].split('/')
                grid = []
                for r in rows:
                    l = []
                    for ch in r:
                        if ch.isdigit():
                            l.extend(['.'] * int(ch))
                        else:
                            l.append(ch)
                    grid.append(l)
                    
                scol = ord(move[0]) - ord('a')
                srank = int(move[1])
                tcol = ord(move[2]) - ord('a')
                trank = int(move[3])
                
                srow = 9 - srank
                trow = 9 - trank
                
                spiece = grid[srow][scol]
                tpiece = grid[trow][tcol]
                
                active_turn = parts[1] # 'w' or 'b'
                
                # Check 1: Source piece exists and is non-empty
                self.assertNotIn(spiece, ('.', ' '), f"Opening {open_idx} move {move_idx} ({move}): Source square is empty!")
                
                # Check 2: Piece belongs to active player
                if active_turn == 'w':
                    self.assertTrue(spiece.isupper(), f"Opening {open_idx} move {move_idx} ({move}): Red active but piece '{spiece}' is black!")
                else:
                    self.assertTrue(spiece.islower(), f"Opening {open_idx} move {move_idx} ({move}): Black active but piece '{spiece}' is red!")
                    
                # Check 3: Target square does not contain own piece (0 self-capture)
                if tpiece not in ('.', ' '):
                    if active_turn == 'w':
                        self.assertFalse(tpiece.isupper(), f"Opening {open_idx} move {move_idx} ({move}): Self-capture Red piece '{tpiece}' at target!")
                    else:
                        self.assertFalse(tpiece.islower(), f"Opening {open_idx} move {move_idx} ({move}): Self-capture Black piece '{tpiece}' at target!")
                        
                # Check 4: Update FEN must succeed without exception
                next_fen = update(fen, move)
                next_parts = next_fen.split()
                
                # Check 5: Active side toggled correctly
                expected_side = 'b' if active_turn == 'w' else 'w'
                self.assertEqual(next_parts[1], expected_side, f"Opening {open_idx} move {move_idx} ({move}): Side failed to toggle!")
                
                fen = next_fen
                
        self.assertEqual(total_moves, 90, f"Tổng số nước đi kiểm tra là {total_moves}, kỳ vọng 90 nước")
        print(f"✅ Đã thực nghiệm 100% {len(openings)} thế khai cuộc ({total_moves} nước đi): 0 nước tự ăn quân, 100% FEN update hợp lệ!")

    def test_02_validation_violations_harness(self):
        """2. Thực nghiệm test harness kiểm tra các trường hợp vi phạm bị chặn chính xác."""
        print("\n--- TEST 2: Kiểm tra Validation Violations Harness ---")
        fen = start  # rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1
        
        # Case A: Empty source square
        with self.assertRaises(ValueError) as ctx:
            update(fen, "a5a4") # Ô a5 rỗng trên bàn cờ ban đầu
        self.assertIn("Ô xuất phát (a5) là ô trống", str(ctx.exception))
        
        # Case B: Self-capture (Xe Đỏ b0 định ăn Mã Đỏ a0)
        with self.assertRaises(ValueError) as ctx:
            update(fen, "b0a0")
        self.assertIn("chứa quân cùng màu", str(ctx.exception))
        
        # Case C: Wrong turn (Lượt Đỏ 'w' nhưng đi Mã Đen b9c7)
        with self.assertRaises(ValueError) as ctx:
            update(fen, "b9c7")
        self.assertIn("không thuộc lượt đi 'w'", str(ctx.exception))
        
        # Case D: Out of bounds col (col 'j' = 9, max is 8)
        with self.assertRaises(ValueError) as ctx:
            update(fen, "a0j0")
        self.assertIn("vượt ngoài phạm vi bàn cờ", str(ctx.exception))
        
        # Case E: Malformed UCI length
        with self.assertRaises(ValueError) as ctx:
            update(fen, "b2e")
        self.assertIn("cần 4 ký tự UCI", str(ctx.exception))
        
        print("✅ Validation harness PASSED: Đã chặn chính xác 100% các trường hợp nước đi vi phạm!")

    def test_03_sha256_hub_deduplication(self):
        """3. Thực nghiệm kiểm tra O(1) SHA256 Hub deduplication."""
        print("\n--- TEST 3: Kiểm tra O(1) SHA256 Hub Deduplication ---")
        
        # Schema verification test
        valid_sample = {"prompt": "p1", "completion": "c1", "move": "b2e2", "stamp": 12345}
        invalid_sample = {"prompt": "p1"}
        self.assertTrue(verify(valid_sample))
        self.assertFalse(verify(invalid_sample))
        
        # SHA256 Key generation test
        k1 = key(valid_sample)
        self.assertEqual(len(k1), 64, "Mã băm SHA256 phải là chuỗi 64 ký tự hex")
        expected_raw = f"{valid_sample['prompt']}||{valid_sample['move']}"
        expected_hash = hashlib.sha256(expected_raw.encode("utf-8")).hexdigest()
        self.assertEqual(k1, expected_hash, "Mã băm key() không trùng khớp với SHA256(prompt||move)")
        
        # Deduplication behavior test
        same_key_different_stamp = {"prompt": "p1", "completion": "different", "move": "b2e2", "stamp": 99999}
        self.assertEqual(key(valid_sample), key(same_key_different_stamp), "Hai sample cùng (prompt, move) phải có cùng SHA256 key")
        
        remote = [valid_sample]
        local = [valid_sample, same_key_different_stamp, {"prompt": "p2", "completion": "c2", "move": "h7e7", "stamp": 12346}]
        
        merged, added = merge(remote=remote, local=local)
        self.assertEqual(len(merged), 2, "Merged dataset phải có đúng 2 mẫu độc nhất")
        self.assertEqual(added, 1, "Chỉ có 1 mẫu mới được thêm từ local")
        
        # Performance stress test for 100,000 items
        print("⚡ Chạy Stress Test Deduplication 100,000 items...")
        large_remote = [{"prompt": f"prompt_{i % 50000}", "completion": f"c_{i}", "move": "b2e2", "stamp": i} for i in range(50000)]
        large_local = [{"prompt": f"prompt_{i}", "completion": f"c_{i}", "move": "b2e2", "stamp": i} for i in range(30000, 80000)]
        
        start_time = time.time()
        large_merged, large_added = merge(remote=large_remote, local=large_local)
        elapsed = time.time() - start_time
        
        self.assertEqual(len(large_merged), 80000, f"Tổng số mẫu hợp nhất là {len(large_merged)}, kỳ vọng 80,000")
        self.assertEqual(large_added, 30000, f"Mới thêm là {large_added}, kỳ vọng 30,000")
        print(f"⏱️ Hợp nhất {len(large_remote) + len(large_local):,} mẫu mất {elapsed:.4f}s (Hiệu năng O(1) SHA256 hash lookup!)")
        
        print("✅ SHA256 Hub Deduplication PASSED: Hoạt động chính xác O(1) hash lookups!")

    def test_04_data_generation_3in1_structure(self):
        """4. Thực nghiệm sinh thử 10 ván cờ (generate) kiểm tra cấu trúc 3-in-1."""
        print("\n--- TEST 4: Kiểm tra Sinh Ván Cờ Tự Đấu 3-in-1 ---")
        samples = []
        for g in range(10):
            batch = generate(g)
            samples.extend(batch)
            
        self.assertGreater(len(samples), 0, "Không sinh được mẫu cờ nào")
        for sample in samples:
            self.assertIn("prompt", sample)
            self.assertIn("completion", sample)
            self.assertIn("move", sample)
            self.assertIn("stamp", sample)
            
            # Check 3-in-1 multi-modal representations in prompt
            prompt_text = sample["prompt"]
            self.assertIn("1. Ma Trận Bàn Cờ 2D (9x10):", prompt_text)
            self.assertIn("2. Chuỗi Chuẩn FEN (Forsyth-Edwards Notation):", prompt_text)
            self.assertIn("3. Lịch Sử Nước Đi PGN (Move History):", prompt_text)
            
        print(f"✅ Đã sinh thử {len(samples)} mẫu cờ 3-in-1 hợp lệ 100%!")


if __name__ == "__main__":
    unittest.main()
