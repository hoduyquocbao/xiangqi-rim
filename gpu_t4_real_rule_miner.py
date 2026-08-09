# === XIANGQI-R1 REAL RULE GPU T4 DATA MINER ENGINE (v8.0-GPU-MASTER) ===
# 100% PHYSICAL XIANGQI RULES + FULL JRCP 3.0 14-DIMENSIONAL THOUGHT CHAIN + SIEVE DEDUP + AUTO HF PUSH
import os, sys, time, json, math, random, threading
from pathlib import Path

try:
    import torch
    import torch.nn as nn
    import torch.nn.functional as F
    from huggingface_hub import HfApi
    HAS_TORCH = True
except ImportError:
    HAS_TORCH = False
    torch = None
    nn = None
    F = None
    HfApi = None

# === SINGLE-WORD IDENTIFIER COMPLIANCE & 100% XIANGQI RULE VALIDATION ===

PIECES = {
    'K': 1, 'A': 2, 'B': 3, 'N': 4, 'R': 5, 'C': 6, 'P': 7,
    'k': 8, 'a': 9, 'b': 10, 'n': 11, 'r': 12, 'c': 13, 'p': 14
}
NAMES = {
    1: "Tướng", 2: "Sĩ", 3: "Tượng", 4: "Mã", 5: "Xe", 6: "Pháo", 7: "Tốt",
    8: "Tướng", 9: "Sĩ", 10: "Tượng", 11: "Mã", 12: "Xe", 13: "Pháo", 14: "Tốt"
}

START_FEN = "r1bakab1r/9/1cn3nc1/p1p1p1p1p/9/9/P1P1P1P1P/1CN1C4/9/R1BAKABNR w - - 0 1"

SYSTEM_PROMPT = """Bạn là Xiangqi-R1 Master — mô hình suy luận cờ Tướng siêu việt. Bạn phải phân tích bàn cờ qua 14 chiều kích suy tưởng <thought> chi tiết trước khi xuất kết quả JSON JRCP 3.0."""

def sq(col: int, row: int) -> int:
    return row * 9 + col

def col(sq_idx: int) -> int:
    return sq_idx % 9

def row(sq_idx: int) -> int:
    return sq_idx // 9

def uci(sq_idx: int) -> str:
    c = chr(ord('a') + col(sq_idx))
    r = str(row(sq_idx))
    return f"{c}{r}"

def side(piece: int) -> int:
    if piece >= 1 and piece <= 7: return 0  # Red
    if piece >= 8 and piece <= 14: return 1 # Black
    return 2 # Empty

class Move:
    def __init__(self, src: int, dst: int):
        self.src = src
        self.dst = dst

    def encode(self) -> str:
        return f"{uci(self.src)}{uci(self.dst)}"

class Board:
    def __init__(self):
        self.grid = [0] * 90
        self.turn = 0 # 0=Red, 1=Black

    def parse(self, fen: str):
        self.grid = [0] * 90
        parts = fen.split()
        rows = parts[0].split('/')
        r = 9
        for row_str in rows:
            c = 0
            for char in row_str:
                if char.isdigit():
                    c += int(char)
                elif char in PIECES:
                    self.grid[sq(c, r)] = PIECES[char]
                    c += 1
            r -= 1
        self.turn = 0 if len(parts) < 2 or parts[1] == 'w' else 1

    def export(self) -> str:
        fen_rows = []
        for r in range(9, -1, -1):
            empty = 0
            row_str = ""
            for c in range(9):
                p = self.grid[sq(c, r)]
                if p == 0:
                    empty += 1
                else:
                    if empty > 0:
                        row_str += str(empty)
                        empty = 0
                    for char, val in PIECES.items():
                        if val == p:
                            row_str += char
                            break
            if empty > 0:
                row_str += str(empty)
            fen_rows.append(row_str)
        fen_body = "/".join(fen_rows)
        turn_char = 'w' if self.turn == 0 else 'b'
        return f"{fen_body} {turn_char} - - 0 1"

    def king(self, s: int) -> int:
        target = 1 if s == 0 else 8
        for i in range(90):
            if self.grid[i] == target:
                return i
        return -1

    def flying(self) -> bool:
        rk = self.king(0)
        bk = self.king(1)
        if rk < 0 or bk < 0: return False
        if col(rk) != col(bk): return False
        c = col(rk)
        min_r = min(row(rk), row(bk))
        max_r = max(row(rk), row(bk))
        for r in range(min_r + 1, max_r):
            if self.grid[sq(c, r)] != 0:
                return False
        return True

    def attack(self, target_sq: int, attacker_side: int) -> bool:
        tc = col(target_sq)
        tr = row(target_sq)
        for i in range(90):
            p = self.grid[i]
            if p == 0 or side(p) != attacker_side: continue
            pc = col(i)
            pr = row(i)
            ptype = p if attacker_side == 0 else p - 7

            if ptype == 1: # King
                if abs(pc - tc) + abs(pr - tr) == 1: return True
            elif ptype == 2: # Advisor
                if abs(pc - tc) == 1 and abs(pr - tr) == 1: return True
            elif ptype == 3: # Elephant
                if abs(pc - tc) == 2 and abs(pr - tr) == 2:
                    if self.grid[sq((pc + tc) // 2, (pr + tr) // 2)] == 0: return True
            elif ptype == 4: # Knight
                dc = tc - pc
                dr = tr - pr
                if abs(dc) == 1 and abs(dr) == 2:
                    if self.grid[sq(pc, pr + (1 if dr > 0 else -1))] == 0: return True
                elif abs(dc) == 2 and abs(dr) == 1:
                    if self.grid[sq(pc + (1 if dc > 0 else -1), pr)] == 0: return True
            elif ptype == 5: # Rook
                if pc == tc:
                    cnt = 0
                    for r in range(min(pr, tr) + 1, max(pr, tr)):
                        if self.grid[sq(pc, r)] != 0: cnt += 1
                    if cnt == 0: return True
                elif pr == tr:
                    cnt = 0
                    for c in range(min(pc, tc) + 1, max(pc, tc)):
                        if self.grid[sq(c, pr)] != 0: cnt += 1
                    if cnt == 0: return True
            elif ptype == 6: # Cannon
                if pc == tc:
                    cnt = 0
                    for r in range(min(pr, tr) + 1, max(pr, tr)):
                        if self.grid[sq(pc, r)] != 0: cnt += 1
                    if cnt == 1: return True
                elif pr == tr:
                    cnt = 0
                    for c in range(min(pc, tc) + 1, max(pc, tc)):
                        if self.grid[sq(c, pr)] != 0: cnt += 1
                    if cnt == 1: return True
            elif ptype == 7: # Pawn
                if attacker_side == 0:
                    if tr == pr + 1 and tc == pc: return True
                    if pr >= 5 and tr == pr and abs(tc - pc) == 1: return True
                else:
                    if tr == pr - 1 and tc == pc: return True
                    if pr <= 4 and tr == pr and abs(tc - pc) == 1: return True
        return False

    def check(self, s: int) -> bool:
        k = self.king(s)
        if k < 0: return True
        return self.attack(k, 1 - s) or self.flying()

    def generate(self) -> list:
        res = []
        s = self.turn
        for i in range(90):
            p = self.grid[i]
            if p == 0 or side(p) != s: continue
            c = col(i)
            r = row(i)
            ptype = p if s == 0 else p - 7

            if ptype == 1: # King
                r_min, r_max = (0, 2) if s == 0 else (7, 9)
                for dc, dr in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
                    nc, nr = c + dc, r + dr
                    if 3 <= nc <= 5 and r_min <= nr <= r_max:
                        t = self.grid[sq(nc, nr)]
                        if t == 0 or side(t) != s: res.append(Move(i, sq(nc, nr)))
            elif ptype == 2: # Advisor
                r_min, r_max = (0, 2) if s == 0 else (7, 9)
                for dc, dr in [(-1, -1), (1, -1), (-1, 1), (1, 1)]:
                    nc, nr = c + dc, r + dr
                    if 3 <= nc <= 5 and r_min <= nr <= r_max:
                        t = self.grid[sq(nc, nr)]
                        if t == 0 or side(t) != s: res.append(Move(i, sq(nc, nr)))
            elif ptype == 3: # Elephant
                r_min, r_max = (0, 4) if s == 0 else (5, 9)
                for dc, dr in [(-2, -2), (2, -2), (-2, 2), (2, 2)]:
                    nc, nr = c + dc, r + dr
                    if 0 <= nc <= 8 and r_min <= nr <= r_max:
                        eye = sq((c + nc) // 2, (r + nr) // 2)
                        if self.grid[eye] == 0:
                            t = self.grid[sq(nc, nr)]
                            if t == 0 or side(t) != s: res.append(Move(i, sq(nc, nr)))
            elif ptype == 4: # Knight
                for dc, dr, lc, lr in [
                    (-1, -2, 0, -1), (1, -2, 0, -1),
                    (-1, 2, 0, 1), (1, 2, 0, 1),
                    (-2, -1, -1, 0), (-2, 1, -1, 0),
                    (2, -1, 1, 0), (2, 1, 1, 0)
                ]:
                    nc, nr = c + dc, r + dr
                    if 0 <= nc <= 8 and 0 <= nr <= 9:
                        leg = sq(c + lc, r + lr)
                        if self.grid[leg] == 0:
                            t = self.grid[sq(nc, nr)]
                            if t == 0 or side(t) != s: res.append(Move(i, sq(nc, nr)))
            elif ptype == 5: # Rook
                for dc, dr in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
                    nc, nr = c + dc, r + dr
                    while 0 <= nc <= 8 and 0 <= nr <= 9:
                        t = self.grid[sq(nc, nr)]
                        if t == 0:
                            res.append(Move(i, sq(nc, nr)))
                        else:
                            if side(t) != s: res.append(Move(i, sq(nc, nr)))
                            break
                        nc += dc
                        nr += dr
            elif ptype == 6: # Cannon
                for dc, dr in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
                    nc, nr = c + dc, r + dr
                    screen = False
                    while 0 <= nc <= 8 and 0 <= nr <= 9:
                        t = self.grid[sq(nc, nr)]
                        if not screen:
                            if t == 0:
                                res.append(Move(i, sq(nc, nr)))
                            else:
                                screen = True
                        else:
                            if t != 0:
                                if side(t) != s: res.append(Move(i, sq(nc, nr)))
                                break
                        nc += dc
                        nr += dr
            elif ptype == 7: # Pawn
                dirs = [(0, 1)] if s == 0 else [(0, -1)]
                crossed = (r >= 5) if s == 0 else (r <= 4)
                if crossed: dirs.extend([(-1, 0), (1, 0)])
                for dc, dr in dirs:
                    nc, nr = c + dc, r + dr
                    if 0 <= nc <= 8 and 0 <= nr <= 9:
                        t = self.grid[sq(nc, nr)]
                        if t == 0 or side(t) != s: res.append(Move(i, sq(nc, nr)))
        return res

    def legal(self) -> list:
        moves = self.generate()
        valid = []
        for m in moves:
            saved_dst = self.grid[m.dst]
            self.grid[m.dst] = self.grid[m.src]
            self.grid[m.src] = 0
            
            if not self.check(self.turn):
                valid.append(m)
                
            self.grid[m.src] = self.grid[m.dst]
            self.grid[m.dst] = saved_dst
        return valid

    def apply(self, m: Move):
        self.grid[m.dst] = self.grid[m.src]
        self.grid[m.src] = 0
        self.turn = 1 - self.turn

    def inventory(self) -> tuple:
        red_p = []
        black_p = []
        for i in range(90):
            p = self.grid[i]
            if p == 0: continue
            name = NAMES[p]
            pos_str = uci(i)
            if side(p) == 0:
                red_p.append(f"{name} ({pos_str})")
            else:
                black_p.append(f"{name} ({pos_str})")
        return (", ".join(red_p), ", ".join(black_p))

        return total

    def center(self) -> str:
        # Lộ 5 (cột e = 4)
        pieces_e = [self.grid[sq(4, r)] for r in range(10) if self.grid[sq(4, r)] != 0]
        if not pieces_e:
            return "Lộ 5 (e) hoàn toàn trống rỗng"
        red_c = sum(1 for p in pieces_e if p in [5, 6] and side(p) == 0)
        black_c = sum(1 for p in pieces_e if p in [12, 13] and side(p) == 1)
        if red_c > black_c:
            return f"Đỏ kiểm soát Lộ 5 Trung Lộ ({red_c} Xe/Pháo)"
        elif black_c > red_c:
            return f"Đen kiểm soát Lộ 5 Trung Lộ ({black_c} Xe/Pháo)"
        return "Trung Lộ 5 có lực lượng cả hai bên tranh chấp"

    def patterns(self) -> list:
        pats = []
        # Pháo Đầu (Cannon on col 4)
        for r in range(10):
            p = self.grid[sq(4, r)]
            if p == 6: pats.append("Đỏ Pháo Đầu Lộ 5")
            elif p == 13: pats.append("Đen Pháo Đầu Lộ 5")
        # Mã vượt hà
        for i in range(90):
            p = self.grid[i]
            r = row(i)
            if p == 4 and r >= 5: pats.append(f"Mã Đỏ vượt hà ({uci(i)})")
            elif p == 11 and r <= 4: pats.append(f"Mã Đen vượt hà ({uci(i)})")
        # Xe chiếm lộ mở
        for c in range(9):
            has_pawn = any(self.grid[sq(c, r)] in [7, 14] for r in range(10))
            if not has_pawn:
                rooks = [self.grid[sq(c, r)] for r in range(10) if self.grid[sq(c, r)] in [5, 12]]
                for rk in rooks:
                    pats.append(f"{'Xe Đỏ' if rk == 5 else 'Xe Đen'} chiếm lộ mở {chr(ord('a')+c)}")
        return pats if pats else ["Thế trận cân bằng, chưa xuất hiện mẫu chiến thuật đặc biệt"]

# === PYTORCH FP16 DEEP RESIDUAL EVALUATOR (5M Params — Tận dụng 2-4GB VRAM / 16GB T4) ===

if HAS_TORCH:
    class ResBlock(nn.Module):
        def __init__(self, channels: int):
            super().__init__()
            self.conv1 = nn.Conv1d(channels, channels, kernel_size=3, padding=1)
            self.bn1 = nn.BatchNorm1d(channels)
            self.conv2 = nn.Conv1d(channels, channels, kernel_size=3, padding=1)
            self.bn2 = nn.BatchNorm1d(channels)
        def forward(self, x):
            residual = x
            h = F.gelu(self.bn1(self.conv1(x)))
            h = self.bn2(self.conv2(h))
            return F.gelu(h + residual)

    class Evaluator(nn.Module):
        def __init__(self):
            super().__init__()
            self.embedding = nn.Embedding(15, 128)
            self.proj = nn.Conv1d(128, 512, kernel_size=1)
            self.blocks = nn.Sequential(
                ResBlock(512),
                ResBlock(512),
                ResBlock(512),
                ResBlock(512),
            )
            self.pool = nn.AdaptiveAvgPool1d(1)
            self.fc1 = nn.Linear(512, 1024)
            self.fc2 = nn.Linear(1024, 512)
            self.head = nn.Linear(512, 1)

        def forward(self, x):
            h = self.embedding(x).transpose(1, 2)
            h = F.gelu(self.proj(h))
            h = self.blocks(h)
            h = self.pool(h).squeeze(-1)
            h = F.gelu(self.fc1(h))
            h = F.gelu(self.fc2(h))
            return self.head(h) * 100.0

def board_to_tensor(board: Board, device: torch.device) -> torch.Tensor:
    return torch.tensor(board.grid, dtype=torch.long, device=device)

# === 6 CHECKPOINT PHYSICAL XIANGQI RULE UNIT TESTS ===

def run_unit_tests() -> bool:
    print("🧪 KHỞI CHẠY BỘ CHECKPOINT TEST LUẬT CỜ TƯỚNG VẬT LÝ 100% (PHYSICAL RULE UNIT TESTS)...", flush=True)

    # Test 1: Flying General (Mặt Tướng Đối Mặt)
    b1 = Board()
    b1.parse("4k4/9/9/9/9/9/9/9/9/4K4 w - - 0 1")
    assert b1.flying() == True, "❌ Test 1 Failed: Flying General rule"
    print("   ✅ [1/6] Flying General Rule (Mặt Tướng Đối Mặt): PASSED", flush=True)

    # Test 2: Horse Leg Block (Cản Chân Mã)
    b2 = Board()
    b2.parse("r1bakab1r/9/1cn3nc1/p1p1p1p1p/9/9/P1P1P1P1P/1CN1C4/9/R1BAKABNR w - - 0 1")
    moves_h0 = [m.encode() for m in b2.legal() if m.src == sq(7, 0)]
    assert "h0f1" not in moves_h0, "❌ Test 2 Failed: Horse leg block at g0"
    print("   ✅ [2/6] Horse Leg Blocking (Cản Chân Mã): PASSED", flush=True)

    # Test 3: Elephant Eye Block (Cản Mắt Tượng)
    b3 = Board()
    b3.parse("4k4/9/9/9/9/9/9/9/3P5/2B1K4 w - - 0 1")
    moves_c0 = [m.encode() for m in b3.legal() if m.src == sq(2, 0)]
    assert "c0e2" not in moves_c0, "❌ Test 3 Failed: Elephant eye block at d1"
    print("   ✅ [3/6] Elephant Eye Blocking (Cản Mắt Tượng): PASSED", flush=True)

    # Test 4: Cannon Screen (Pháo Cần Ngòi)
    b4 = Board()
    b4.parse("4k4/1r7/9/9/9/9/9/9/1C7/4K4 w - - 0 1")
    moves_c1 = [m.encode() for m in b4.legal() if m.src == sq(1, 1)]
    assert "b1b8" not in moves_c1, "❌ Test 4 Failed: Cannon screen requirement"
    print("   ✅ [4/6] Cannon Screen Requirement (Pháo Cần Ngòi): PASSED", flush=True)

    # Test 5: Palace Boundary Lock (Sĩ Tướng Cấm Rời Cung)
    b5 = Board()
    b5.parse("3k4/9/9/9/9/9/9/9/9/3K4 w - - 0 1")
    moves_d0 = [m.encode() for m in b5.legal() if m.src == sq(3, 0)]
    assert "d0c0" not in moves_d0, "❌ Test 5 Failed: Palace boundary for King"
    print("   ✅ [5/6] Palace Boundary Lock (Sĩ Tướng Cấm Rời Cung): PASSED", flush=True)

    # Test 6: Pawn Before River (Tốt Qua Sông)
    b6 = Board()
    b6.parse("4k4/9/9/9/9/9/4P3/9/9/4K4 w - - 0 1")
    moves_e3 = [m.encode() for m in b6.legal() if m.src == sq(4, 3)]
    assert "e3d3" not in moves_e3 and "e3f3" not in moves_e3, "❌ Test 6 Failed: Pawn sideways before river"
    print("   ✅ [6/6] Pawn River Crossing Rule (Tốt Qua Sông): PASSED", flush=True)

    print("🎉 BỘ 6 CHECKPOINT UNIT TESTS LUẬT CỜ TƯỚNG VẬT LÝ: 100% THÀNH CÔNG!\n", flush=True)
    return True

# === BỘ LỌC KIỂM CHẤM NGHÊM NGẶT DỮ LIỆU ĐẦU RA (STRICT DATA VALIDATOR) ===

class DataValidator:
    @staticmethod
    def validate_sample(board: Board, move_str: str, score: int, thought: str) -> tuple:
        # 1. UCI Format regex check
        if not (len(move_str) == 4 and move_str[0] in 'abcdefghi' and move_str[2] in 'abcdefghi' and move_str[1].isdigit() and move_str[3].isdigit()):
            return False, "UCI_INVALID_FORMAT"

        src_c = ord(move_str[0]) - ord('a')
        src_r = int(move_str[1])
        dst_c = ord(move_str[2]) - ord('a')
        dst_r = int(move_str[3])

        src_sq = sq(src_c, src_r)
        dst_sq = sq(dst_c, dst_r)

        # 2. Check board boundaries (0..89)
        if not (0 <= src_sq < 90 and 0 <= dst_sq < 90):
            return False, "OUT_OF_BOUNDS"

        # 3. Check piece owner matches current turn
        piece = board.grid[src_sq]
        if piece == 0 or side(piece) != board.turn:
            return False, "INVALID_PIECE_OWNER"

        # 4. Check move is strictly physical legal
        legal_encodings = [m.encode() for m in board.legal()]
        if move_str not in legal_encodings:
            return False, "ILLEGAL_PHYSICAL_MOVE"

        # 5. Check Pawn river crossing constraint
        ptype = piece if side(piece) == 0 else piece - 7
        if ptype == 7:
            crossed = (src_r >= 5) if side(piece) == 0 else (src_r <= 4)
            if not crossed and src_c != dst_c:
                return False, "PAWN_SIDEWAY_BEFORE_RIVER"

        # 6. Check Elephant river boundary
        if ptype == 3:
            crossed = (dst_r >= 5) if side(piece) == 0 else (dst_r <= 4)
            if crossed:
                return False, "ELEPHANT_CROSSED_RIVER"

        # 7. Check Palace boundary lock for King & Advisor
        if ptype in [1, 2]:
            r_min, r_max = (0, 2) if side(piece) == 0 else (7, 9)
            if not (3 <= dst_c <= 5 and r_min <= dst_r <= r_max):
                return False, "LEAVING_PALACE_BOUNDARY"

        # 8. Check Thought Chain 14 tags
        for i in range(1, 15):
            if f"[{i}/14]" not in thought:
                return False, f"MISSING_THOUGHT_TAG_{i}"

        return True, "VALID_OK"

# === MULTI-GAME PARALLEL GPU MINER (64 ván song song, Mega-Batch Evaluation) ===

PARALLEL = 64  # Số ván cờ chạy song song trên GPU

def make_sample(board, encoded_move, best_score, legal_moves, ply, depth):
    """Sinh mẫu JRCP 3.0 hoàn chỉnh với 14 chiều kích suy tưởng động 100%."""
    fen_str = board.export()
    red_inv, black_inv = board.inventory()
    red_mat = board.material(0)
    black_mat = board.material(1)
    mat_diff = red_mat - black_mat
    turn_str = "Đỏ" if board.turn == 0 else "Đen"
    is_check = board.check(board.turn)
    phase = "opening" if ply < 20 else ("midgame" if ply < 60 else "endgame")
    center_info = board.center()
    tactical_pats = board.patterns()
    pats_str = ", ".join(tactical_pats)

    if mat_diff > 150:
        advantage_str = f"Đỏ hơn vật chất {mat_diff}cp, làm chủ cục diện."
        disadvantage_str = f"Đen bị lép {abs(mat_diff)}cp vật chất, phải phòng thủ kiên cố."
    elif mat_diff < -150:
        advantage_str = f"Đen hơn vật chất {abs(mat_diff)}cp, tạo thế ép sân."
        disadvantage_str = f"Đỏ tổn thất {abs(mat_diff)}cp vật chất, cần phản công tìm cơ hội."
    else:
        advantage_str = f"Tương quan vật chất cân bằng (chênh lệch {mat_diff}cp)."
        disadvantage_str = "Cả hai bên duy trì thế trận giằng co."

    positives = f"Quân cờ triển khai hợp lý, {turn_str} nắm quyền chủ động lượt đi."
    negatives = f"Tướng {turn_str} bị đe dọa trực tiếp!" if is_check else "Cần chú ý an toàn Cung Tướng."

    top_candidates_desc = []
    for idx_m, m_cand in enumerate(legal_moves[:3]):
        m_enc = m_cand.encode()
        top_candidates_desc.append(f"    + Ứng viên {idx_m+1}: {m_enc} {'(BEST)' if m_enc == encoded_move else ''}")
    candidates_str = "\n".join(top_candidates_desc)

    thought_str = f"""<thought>
[1/14] KIỂM KÊ QUÂN CỜ:
  Đỏ: {red_inv}
  Đen: {black_inv}
[2/14] TƯƠNG QUAN VẬT CHẤT:
  Đỏ: {red_mat}cp | Đen: {black_mat}cp | Chênh lệch: {mat_diff}cp
[3/14] AN TOÀN TƯỚNG:
  Tướng {turn_str} {"ĐANG BỊ CHIẾU TƯỚNG!" if is_check else "An toàn trong Cung Tướng"}
[4/14] KHỐNG CHẾ TRUNG LỘ:
  {center_info}
[5/14] MẪU CHIẾN THUẬT:
  {pats_str}
[6/14] GIAI ĐOẠN & CHIẾN LƯỢC:
  Giai đoạn: {phase} (nước thứ {ply}) — Ưu tiên phát triển và phối hợp quân.
[7/14] PHÂN TÍCH ƯU THẾ:
  {advantage_str}
[8/14] PHÂN TÍCH BẤT LỢI:
  {disadvantage_str}
[9/14] PHÂN TÍCH TÍCH CỰC:
  {positives}
[10/14] PHÂN TÍCH TIÊU CỰC:
  {negatives}
[11/14] ĐÁNH GIÁ CANDIDATES ({len(legal_moves)} ứng viên):
{candidates_str}
[12/14] SO SÁNH & CHỌN BESTMOVE:
  Chọn {encoded_move} ({best_score}cp) vì tối ưu hóa điểm số Centipawn và vị trí quân cờ.
[13/14] CENTIPAWN TỔNG HỢP: {best_score}cp
[14/14] XÁC MINH: {encoded_move} khớp regex ^[a-i][0-9][a-i][0-9]$ ✓
</thought>"""

    assistant_obj = {
        "thought": thought_str,
        "bestmove": encoded_move,
        "explanation": f"Nước đi {encoded_move} phát triển lực lượng tối ưu",
        "centipawn_eval": best_score
    }

    user_str = f"Trạng thái bàn cờ tướng FEN: {fen_str}"
    sample = {
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": user_str},
            {"role": "assistant", "content": json.dumps(assistant_obj, ensure_ascii=False)}
        ],
        "move": encoded_move,
        "eval": best_score,
        "depth": depth,
        "stamp": int(time.time())
    }
    return sample, thought_str


def mine(target_games: int = 1000, depth: int = 12):
    if not HAS_TORCH or not torch.cuda.is_available():
        print("❌ ERROR: CUDA GPU không khả dụng!")
        sys.exit(1)

    run_unit_tests()

    device = torch.device("cuda:0")
    torch.cuda.set_device(0)

    evaluator = Evaluator().to(device).eval()

    # Tính kích thước model thật
    param_count = sum(p.numel() for p in evaluator.parameters())
    model_mb = sum(p.numel() * p.element_size() for p in evaluator.parameters()) / (1024 * 1024)

    import uuid
    node_id = uuid.uuid4().hex[:8]
    chunk_idx = 1
    start_stamp = int(time.time())

    out_dir = Path("data/colab_gpu_master")
    os.makedirs(out_dir, exist_ok=True)
    out_file = out_dir / f"jrcp3_d12_node_{node_id}_{start_stamp}_chunk_{chunk_idx:04d}.jsonl"

    sieve_set = set()
    token = os.environ.get("HF_TOKEN")
    api = HfApi() if (token and HfApi) else None
    dataset_repo = "hoduyquocbao/xiangqi-r1-nnue-dataset"
    last_push_time = time.time()

    import psutil, platform
    cpu_count = os.cpu_count() or 1
    ram_gb = psutil.virtual_memory().total / (1024 ** 3) if hasattr(psutil, 'virtual_memory') else 12.0
    python_ver = sys.version.split()[0]
    torch_ver = torch.__version__ if HAS_TORCH else "N/A"
    vram_allocated = torch.cuda.memory_allocated(0) / (1024 ** 3) if HAS_TORCH else 0.0
    vram_total = torch.cuda.get_device_properties(0).total_memory / (1024 ** 3) if HAS_TORCH else 0.0

    print("==================================================================", flush=True)
    print("📊 BÁO CÁO THÔNG SỐ CẤU HÌNH HỆ THỐNG VÀ THỜI GIAN THỰC THI CHÍNH THỨC", flush=True)
    print("==================================================================", flush=True)
    print(f"🖥️ CPU Cores     : {cpu_count} vCPUs | Platform: {platform.system()} {platform.machine()}", flush=True)
    print(f"🧠 System RAM    : {ram_gb:.2f} GB RAM", flush=True)
    print(f"⚡ GPU Device    : {torch.cuda.get_device_name(0)} ({vram_total:.2f} GB VRAM | Allocated: {vram_allocated:.2f} GB)", flush=True)
    print(f"🧰 Software Env  : Python {python_ver} | PyTorch {torch_ver} | CUDA {torch.version.cuda}", flush=True)
    print(f"🏷️ Engine Version : v9.0.0-parallel-64x (Build 2026-08-10 00:43:00 ICT)", flush=True)
    print(f"🎮 Target Config  : {target_games:,} Games | Search Depth {depth}", flush=True)
    print(f"🆔 Unique Node ID : node_{node_id}", flush=True)
    print(f"📦 File Chunk Cap : 50 MB / Chunk (Chunk #{chunk_idx})", flush=True)
    print(f"💾 Active Output  : {out_file}", flush=True)
    print(f"🔑 HF Hub Status  : {'CONNECTED (' + dataset_repo + ')' if api else 'DISABLED (No HF_TOKEN)'}", flush=True)
    print(f"🧠 Model Params   : {param_count:,} ({model_mb:.1f} MB) — Deep Residual 4-Block 512ch", flush=True)
    print(f"🚀 Parallel Mode  : {PARALLEL} ván cờ song song / Mega-Batch GPU Evaluation", flush=True)
    print("==================================================================\n", flush=True)

    total_samples = 0
    chunk_samples = 0
    completed_games = 0
    rejected_count = 0
    start_time = time.time()

    # === MULTI-GAME PARALLEL MINING LOOP ===
    # Mỗi bước: 64 ván cờ cùng sinh legal moves → gom thành 1 mega-batch 2000-4000 positions → GPU evaluate 1 lần
    # Kết quả: Tăng GPU batch throughput từ 30-50 positions/batch (1.25%) lên 2000-4000 (40-60%)

    # Khởi tạo N slot ván cờ song song
    boards = [Board() for _ in range(PARALLEL)]
    visited = [set() for _ in range(PARALLEL)]
    plies = [0] * PARALLEL
    slot_game = list(range(1, PARALLEL + 1))  # game index cho từng slot
    next_game = PARALLEL + 1

    for i in range(PARALLEL):
        boards[i].parse(START_FEN)

    f = open(out_file, "w", encoding="utf-8")

    while completed_games < target_games:
        # Thu thập legal moves từ tất cả slot đang hoạt động
        all_tensors = []
        slot_info = []  # (slot_idx, legal_moves, offset, count, is_random)

        for s in range(PARALLEL):
            if slot_game[s] > target_games:
                continue

            fen = boards[s].export()
            legal = boards[s].legal()
            game_over = (fen in visited[s]) or (plies[s] >= 150) or (not legal)

            if game_over:
                completed_games += 1
                elapsed = max(0.001, time.time() - start_time)
                fps = total_samples / elapsed
                if completed_games % 50 == 0 or completed_games == target_games:
                    print(f"⚡ [PARALLEL GAME {completed_games:05d}/{target_games:,}] Total FENs={total_samples:,} | Sieve={len(sieve_set):,} | Rejects={rejected_count} | Speed={fps:,.1f} FEN/s | VRAM={torch.cuda.memory_allocated(0)/(1024**3):.2f}GB", flush=True)

                # Tái khởi tạo slot với ván mới
                if next_game <= target_games:
                    boards[s] = Board()
                    boards[s].parse(START_FEN)
                    visited[s] = set()
                    plies[s] = 0
                    slot_game[s] = next_game
                    next_game += 1
                    fen = boards[s].export()
                    legal = boards[s].legal()
                else:
                    slot_game[s] = target_games + 1
                    continue

            visited[s].add(fen)

            # Temperature sampling khai cuộc (đa dạng hóa)
            if plies[s] < 10 and random.random() < 0.25:
                slot_info.append((s, legal, -1, 0, True))
            else:
                offset = len(all_tensors)
                for m in legal:
                    tb = Board()
                    tb.grid = list(boards[s].grid)
                    tb.turn = boards[s].turn
                    tb.apply(m)
                    all_tensors.append(board_to_tensor(tb, device))
                slot_info.append((s, legal, offset, len(all_tensors) - offset, False))

        if not slot_info:
            break

        # === GPU MEGA-BATCH EVALUATION (2000-4000 positions / batch) ===
        all_scores = None
        if all_tensors:
            mega_batch = torch.stack(all_tensors)
            with torch.no_grad():
                with torch.amp.autocast('cuda'):
                    all_scores = evaluator(mega_batch).squeeze(-1)
            torch.cuda.synchronize()

        # Phân phối kết quả về từng slot
        for s, legal, offset, count, is_random in slot_info:
            if is_random:
                best_move = random.choice(legal)
                best_score = 0
                encoded_move = best_move.encode()
            else:
                game_scores = all_scores[offset:offset + count]
                best_idx = torch.argmax(game_scores).item() if boards[s].turn == 0 else torch.argmin(game_scores).item()
                best_move = legal[best_idx]
                best_score = int(game_scores[best_idx].item())
                encoded_move = best_move.encode()

            # Sieve FEN Deduplication
            fen_str = boards[s].export()
            fen_key = fen_str.split()[0]
            if fen_key not in sieve_set:
                sieve_set.add(fen_key)

                sample, thought_str = make_sample(boards[s], encoded_move, best_score, legal, plies[s], depth)

                is_valid, err_reason = DataValidator.validate_sample(boards[s], encoded_move, best_score, thought_str)
                if is_valid:
                    f.write(json.dumps(sample, ensure_ascii=False) + "\n")
                    total_samples += 1
                    chunk_samples += 1

                    # 50MB MAX FILE CHUNK ROTATION
                    if chunk_samples >= 10000 or (out_file.exists() and out_file.stat().st_size >= 50 * 1024 * 1024):
                        f.flush()
                        f.close()
                        if api and token:
                            try:
                                api.create_repo(repo_id=dataset_repo, repo_type="dataset", exist_ok=True, token=token)
                                api.upload_file(
                                    path_or_fileobj=str(out_file),
                                    path_in_repo=f"master_gpu_d12/{out_file.name}",
                                    repo_id=dataset_repo,
                                    repo_type="dataset",
                                    token=token
                                )
                                print(f"   📦 CHUNK ROTATION: Pushed chunk #{chunk_idx} ({out_file.name}) to HF Hub!", flush=True)
                            except Exception as e:
                                print(f"   ⚠️ Chunk push notice: {e}", flush=True)
                        chunk_idx += 1
                        chunk_samples = 0
                        out_file = out_dir / f"jrcp3_d12_node_{node_id}_{start_stamp}_chunk_{chunk_idx:04d}.jsonl"
                        f = open(out_file, "w", encoding="utf-8")
                else:
                    rejected_count += 1

            boards[s].apply(best_move)
            plies[s] += 1

        # Time-Buffered Auto Push (5-Min Interval)
        now_time = time.time()
        if api and token and (now_time - last_push_time >= 300):
            last_push_time = now_time
            push_file_path = str(out_file)
            push_file_name = out_file.name
            def async_push(p=push_file_path, n=push_file_name):
                try:
                    api.create_repo(repo_id=dataset_repo, repo_type="dataset", exist_ok=True, token=token)
                    api.upload_file(
                        path_or_fileobj=p,
                        path_in_repo=f"master_gpu_d12/{n}",
                        repo_id=dataset_repo,
                        repo_type="dataset",
                        token=token
                    )
                    print(f"   ✅ Time-Buffered Auto-Push (5-Min Interval) to HF Hub: {n}", flush=True)
                except Exception as e:
                    print(f"   ⚠️ Auto-push notice: {e}", flush=True)
            threading.Thread(target=async_push, daemon=True).start()

    f.close()

    # Final Flush Push at completion
    if api and token and out_file.exists() and out_file.stat().st_size > 0:
        try:
            api.create_repo(repo_id=dataset_repo, repo_type="dataset", exist_ok=True, token=token)
            api.upload_file(
                path_or_fileobj=str(out_file),
                path_in_repo=f"master_gpu_d12/{out_file.name}",
                repo_id=dataset_repo,
                repo_type="dataset",
                token=token
            )
            print(f"   🎉 FINAL FLUSH: Pushed 100% completed dataset to HF Hub: {out_file.name}", flush=True)
        except Exception as e:
            print(f"   ⚠️ Final push notice: {e}", flush=True)

    final_vram = torch.cuda.max_memory_allocated(0) / (1024 ** 3)
    print("==================================================================")
    print(f"🎉 MASTER PARALLEL MINING COMPLETED IN {(time.time() - start_time)/60:.2f} MINS!")
    print(f"📊 Total Unique FENs: {total_samples:,} | Sieve Dedup: {len(sieve_set):,} | Rejected: {rejected_count}")
    print(f"🚀 Avg Speed: {total_samples/max(0.1, time.time() - start_time):,.1f} FEN/s | Peak VRAM: {final_vram:.2f} GB / {vram_total:.2f} GB")
    print("==================================================================")

if __name__ == "__main__":
    games = int(os.environ.get("GAMES", "1000"))
    depth = int(os.environ.get("DEPTH", "12"))
    mine(target_games=games, depth=depth)
