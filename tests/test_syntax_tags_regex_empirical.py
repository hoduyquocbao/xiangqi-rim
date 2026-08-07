# tests/test_syntax_tags_regex_empirical.py
# Empirical challenge test harness for syntax tags, regex backreference matching, FEN regex, and train.ipynb sync.
# Identifiers in English, comments in Vietnamese.

import os
import sys
import re
import json

sys.path.insert(0, os.path.abspath("scripts"))
from train import FORMAT, FEN, MOVE, syntax, rule, quality, parse, valid

def test_regex_backreference_matching():
    """Kiểm thử chuyên sâu tính năng nhóm tham chiếu ngược (Regex Backreference) trong FORMAT."""
    print("--- TEST 1: Regex Backreference Matching in FORMAT ---")
    
    # 1. Matching valid tags with backreference
    good_thought = "<thought>\nReasoning step 1...\n</thought>\nb2e2"
    good_think = "<think>\nReasoning step 1...\n</think>\nb2e2"
    good_thought_spaces = "  <thought>\nReasoning step 1...\n</thought>\n  b2e2  "
    
    assert FORMAT.match(good_thought) is not None, "FORMAT phải match <thought>...</thought>"
    assert FORMAT.match(good_think) is not None, "FORMAT phải match <think>...</think>"
    assert FORMAT.match(good_thought_spaces) is not None, "FORMAT phải match khi có khoảng trắng xung quanh"
    
    # Check captured groups in backreference match
    m1 = FORMAT.match(good_thought)
    assert m1.group(1) == "thought", f"Group 1 phải là 'thought', nhận {m1.group(1)}"
    assert m1.group(2) == "thought", f"Group 2 (backreference) phải là 'thought', nhận {m1.group(2)}"
    assert m1.group(3) == "b2e2", f"Group 3 phải là 'b2e2', nhận {m1.group(3)}"
    
    m2 = FORMAT.match(good_think)
    assert m2.group(1) == "think", f"Group 1 phải là 'think', nhận {m2.group(1)}"
    assert m2.group(2) == "think", f"Group 2 (backreference) phải là 'think', nhận {m2.group(2)}"
    assert m2.group(3) == "b2e2", f"Group 3 phải là 'b2e2', nhận {m2.group(3)}"
    
    # 2. Mismatched tag combinations (MUST NOT MATCH)
    bad_mix_1 = "<thought>\nReasoning...\n</think>\nb2e2"
    bad_mix_2 = "<think>\nReasoning...\n</thought>\nb2e2"
    
    assert FORMAT.match(bad_mix_1) is None, "FORMAT KHÔNG ĐƯỢC match <thought>...</think>"
    assert FORMAT.match(bad_mix_2) is None, "FORMAT KHÔNG ĐƯỢC match <think>...</thought>"
    
    # 3. Check reward scoring for mismatched tags
    r_bad1 = syntax([None], [bad_mix_1])
    r_bad2 = syntax([None], [bad_mix_2])
    assert r_bad1[0] == -1.0, f"Thẻ lệch <thought>...</think> phải bị phạt -1.0, nhận {r_bad1[0]}"
    assert r_bad2[0] == -1.0, f"Thẻ lệch <think>...</thought> phải bị phạt -1.0, nhận {r_bad2[0]}"
    
    print("✅ TEST 1 PASSED: Backreference regex matching chuẩn xác 100%!")

def test_fen_regex_and_uppercase_matching():
    """Kiểm thử regex FEN nhận diện đúng cả chữ in hoa (Red) và chữ in thường (Black)."""
    print("\n--- TEST 2: FEN Regex & Uppercase Matching ---")
    
    prompt_full = (
        "Trạng thái bàn cờ tướng hiện tại (Biểu diễn đa chiều: Ma trận 2D, Chuỗi FEN chuẩn, và Lịch sử nước đi PGN):\n\n"
        "1. Ma Trận Bàn Cờ 2D (9x10):\n"
        "r n b a k a b n r\n"
        ". . . . . . . . .\n"
        "2. Chuỗi Chuẩn FEN (Forsyth-Edwards Notation):\n"
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1\n\n"
        "3. Lịch Sử Nước Đi PGN (Move History):\n"
        "Ván cờ mới bắt đầu (Chưa có nước đi)\n\n"
        "Đến lượt Đỏ đi. Hãy suy nghĩ sâu sắc trong thẻ <thought> và đưa ra nước đi UCI hợp lệ:"
    )
    
    matched = FEN.search(prompt_full)
    assert matched is not None, "FEN regex phải tìm thấy chuỗi FEN trong prompt"
    extracted_fen = matched.group(1)
    expected_fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1"
    assert extracted_fen == expected_fen, f"FEN trích xuất phải khớp chính xác: nhận '{extracted_fen}'"
    
    # Test valid rule reward with extracted FEN
    completion = "<thought>\nSuy nghĩ\n</thought>\nb2e2"
    r = rule([prompt_full], [completion])
    assert r[0] == 2.0, f"Rule reward phải trả về 2.0 cho FEN in hoa chuẩn, nhận {r[0]}"
    
    print("✅ TEST 2 PASSED: FEN regex hỗ trợ đầy đủ quân cờ in hoa và in thường!")

def test_notebook_sync():
    """Kiểm thử sự đồng bộ giữa train.ipynb và scripts/train.py."""
    print("\n--- TEST 3: train.ipynb Sync Verification ---")
    
    with open("train.ipynb", "r", encoding="utf-8") as f:
        nb = json.load(f)
        
    code_cells = [cell["source"] for cell in nb["cells"] if cell["cell_type"] == "code"]
    cell_5_text = "".join(code_cells[4]) # Cell 5 (0-indexed 4) contains reward functions
    
    # Verify FORMAT regex in train.ipynb
    assert r"<\/(\1)>" in cell_5_text or r"<\/(\1)>" in cell_5_text.replace("\\\\", "\\"), "train.ipynb Cell 5 phải chứa backreference <\\/(\\1)>"
    # Verify FEN regex in train.ipynb
    assert "[a-zA-Z0-9/]" in cell_5_text, "train.ipynb Cell 5 phải chứa FEN regex [a-zA-Z0-9/]"
    # Verify Single-Word Identifiers matched and grounds in train.ipynb
    assert "matched = FEN.search" in cell_5_text, "train.ipynb Cell 5 phải dùng 'matched' (từ đơn)"
    assert "grounds = kwargs.get" in cell_5_text, "train.ipynb Cell 5 phải dùng 'grounds' (từ đơn)"
    
    print("✅ TEST 3 PASSED: train.ipynb hoàn toàn đồng bộ và tuân thủ quy chuẩn!")

if __name__ == "__main__":
    print("============================================================")
    print(" EMPIRICAL TEST SUITE: SYNTAX TAGS & REGEX BACKREFERENCE")
    print("============================================================")
    test_regex_backreference_matching()
    test_fen_regex_and_uppercase_matching()
    test_notebook_sync()
    print("============================================================")
    print(" 🎉 ALL SYNTAX & REGEX EMPIRICAL TESTS PASSED SUCCESSFULLY!")
    print("============================================================")
