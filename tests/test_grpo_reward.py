# tests/test_grpo_reward.py
# ============================================================================
# KIỂM THỬ ĐƠN VỊ CÁC HÀM THƯỞNG GRPO VÀ BỘ KIỂM TRA HỢP LỆ FEN (TEST SUITE)
# ============================================================================
# Định danh đơn từ tiếng Anh: syntax, rule, quality, parse, valid, test,
# prompt, completion, rewards, fen, move, ground, assert, equal, main
# ============================================================================

import sys
import os

# Nạp module scripts/train.py
sys.path.insert(0, os.path.abspath("scripts"))
from train import syntax, rule, quality, parse, valid

def test_syntax():
    """Kiểm thử hàm thưởng cú pháp syntax."""
    # Exact match <thought> tag
    c1 = "<thought>\n1. Phân tích FEN\n</thought>\nb2e2"
    r1 = syntax([None], [c1])
    assert r1[0] == 1.0, f"Mong đợi 1.0, nhận {r1[0]}"

    # Exact match <think> tag
    c2 = "<think>\n1. Phân tích FEN\n</think>\nh2e2"
    r2 = syntax([None], [c2])
    assert r2[0] == 1.0, f"Mong đợi 1.0, nhận {r2[0]}"

    # Thẻ mở đóng không khớp (<thought> ... </think>) không đạt điểm tối đa 1.0
    c_bad = "<thought>\nSuy nghĩ\n</think>\nb2e2"
    r_bad = syntax([None], [c_bad])
    assert r_bad[0] != 1.0, f"Thẻ không khớp không được nhận điểm 1.0, nhận {r_bad[0]}"

    # Partial match (có thẻ thought/think và có nước đi nhưng chưa đạt exact pattern)
    c3 = "Lời đáp: <thought>Phân tích</thought> Tôi đề xuất nước đi b2e2 cho trận đấu."
    r3 = syntax([None], [c3])
    assert r3[0] == 0.5, f"Mong đợi 0.5, nhận {r3[0]}"

    # Invalid format (không chứa thẻ thought/think)
    c4 = "Không có thẻ suy nghĩ b2e2"
    r4 = syntax([None], [c4])
    assert r4[0] == -1.0, f"Mong đợi -1.0, nhận {r4[0]}"

    print("✅ test_syntax PASSED!")

def test_rule():
    """Kiểm thử hàm thưởng luật cờ rule dựa trên bàn cờ FEN và các quy tắc biên."""
    prompt = (
        "Trạng thái bàn cờ tướng hiện tại...\n"
        "2. Chuỗi Chuẩn FEN (Forsyth-Edwards Notation):\n"
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1\n"
    )

    # Nước đi Pháo Đầu Đỏ b2e2 (hợp lệ FEN)
    c1 = "<thought>\nSuy nghĩ\n</thought>\nb2e2"
    r1 = rule([prompt], [c1])
    assert r1[0] == 2.0, f"Mong đợi 2.0 cho nước đi hợp lệ FEN, nhận {r1[0]}"

    # Nước đi đi từ ô trống a1a2 (bàn cờ ban đầu a1 là ô trống)
    c2 = "<thought>\nSuy nghĩ\n</thought>\na1a2"
    r2 = rule([prompt], [c2])
    assert r2[0] == -0.5, f"Mong đợi -0.5 cho nước đi ô trống, nhận {r2[0]}"

    # Nước đi tự di chuyển cùng ô a0a0
    c3 = "<thought>\nSuy nghĩ\n</thought>\na0a0"
    r3 = rule([prompt], [c3])
    assert r3[0] == -0.5, f"Mong đợi -0.5 cho nước đi a0a0, nhận {r3[0]}"

    # Kiểm tra guard chữ số trong move
    fen_start = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1"
    assert valid(fen_start, "abcd") == False

    # Kiểm tra FEN lỗi kích thước cột (< 9)
    bad_fen = "rnbakabnr/8/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1"
    assert valid(bad_fen, "b2e2") == False

    # Kiểm tra Tốt Đỏ ('P') đi lùi và đi ngang trước khi qua sông
    assert valid(fen_start, "c3c2") == False  # Tốt Đỏ đi lùi
    assert valid(fen_start, "c3d3") == False  # Tốt Đỏ chưa qua sông đi ngang
    assert valid(fen_start, "c3c4") == True   # Tốt Đỏ tiến 1 bước hợp lệ

    # Tốt Đỏ đã qua sông ('P' tại c5)
    fen_crossed = "rnbakabnr/9/1c5c1/p1p1p1p1p/2P6/9/P3P1P1P/1C5C1/9/RNBAKABNR w - - 0 1"
    assert valid(fen_crossed, "c5d5") == True   # Tốt Đỏ qua sông được đi ngang
    assert valid(fen_crossed, "c5c4") == False  # Tốt Đỏ không bao giờ được đi lùi

    # Tượng Đỏ ('B') qua sông
    fen_elephant = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/2B6/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1"
    assert valid(fen_elephant, "c4e6") == False # Tượng Đỏ không được qua sông (trank > 4)
    assert valid(fen_elephant, "c4e2") == True  # Tượng Đỏ ở sân nhà

    # Tượng Đen ('b') qua sông
    fen_black_b = "rnbakabnr/9/1c5b1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR b - - 0 1"
    assert valid(fen_black_b, "h7f3") == False  # Tượng Đen không được sang sân Đỏ (trank < 5)

    print("✅ test_rule PASSED!")

def test_quality():
    """Kiểm thử hàm thưởng chất lượng quality."""
    prompt = "Prompt test"

    # Trùng khớp Ground Truth
    c1 = "<thought>\nSuy nghĩ\n</thought>\nb2e2"
    r1 = quality([prompt], [c1], move=["b2e2"])
    assert r1[0] == 3.0, f"Mong đợi 3.0 cho ground truth, nhận {r1[0]}"

    # Nước đi khai cuộc chuẩn không trùng ground truth
    c2 = "<thought>\nSuy nghĩ\n</thought>\nh2e2"
    r2 = quality([prompt], [c2], move=["b2e2"])
    assert r2[0] == 1.5, f"Mong đợi 1.5 cho khai cuộc chuẩn, nhận {r2[0]}"

    # Nước đi hợp lệ khác
    c3 = "<thought>\nSuy nghĩ\n</thought>\na0a1"
    r3 = quality([prompt], [c3], move=["b2e2"])
    assert r3[0] == 0.5, f"Mong đợi 0.5 cho nước đi thường, nhận {r3[0]}"

    print("✅ test_quality PASSED!")

if __name__ == "__main__":
    test_syntax()
    test_rule()
    test_quality()
    print("============================================================")
    print("🎉 TOÀN BỘ 100% UNIT TESTS CHO REWARD FUNCTIONS ĐÃ PASSED!")
    print("============================================================")
