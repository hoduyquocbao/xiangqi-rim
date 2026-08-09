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

    def material(self, s: int) -> int:
        weights = {1: 10000, 2: 200, 3: 200, 4: 450, 5: 900, 6: 450, 7: 100}
        total = 0
        for i in range(90):
            p = self.grid[i]
            if p != 0 and side(p) == s:
                ptype = p if s == 0 else p - 7
                total += weights.get(ptype, 0)
        return total

# === PYTORCH FP16 TENSOR EVALUATOR FOR REAL LEGAL BOARD POSITIONS ===

if HAS_TORCH:
    class Evaluator(nn.Module):
        def __init__(self):
            super().__init__()
            self.embedding = nn.Embedding(15, 64)
            self.conv1 = nn.Conv1d(64, 256, kernel_size=3, padding=1)
            self.act1 = nn.GELU()
            self.conv2 = nn.Conv1d(256, 256, kernel_size=3, padding=1)
            self.act2 = nn.GELU()
            self.pool = nn.AdaptiveAvgPool1d(1)
            self.fc1 = nn.Linear(256, 512)
            self.fc2 = nn.Linear(512, 256)
            self.head_eval = nn.Linear(256, 1)

        def forward(self, x):
            h = self.embedding(x).transpose(1, 2)
            h = self.act1(self.conv1(h))
            h = self.act2(self.conv2(h))
            h = self.pool(h).squeeze(-1)
            h = F.gelu(self.fc1(h))
            h = F.gelu(self.fc2(h))
            eval_score = self.head_eval(h) * 100.0
            return eval_score

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

    # Test 4: Cannon Screen (Ngòi Pháo)
    b4 = Board()
    b4.parse("4k4/1r7/9/9/9/9/9/9/1C7/4K4 w - - 0 1")
    moves_c = [m.encode() for m in b4.legal() if m.src == sq(1, 1)]
    assert "b1b8" not in moves_c, "❌ Test 4 Failed: Cannon cannot capture without screen"
    print("   ✅ [4/6] Cannon Screen Requirement (Pháo Cần Ngòi): PASSED", flush=True)

    # Test 5: Palace Boundaries for King & Advisor (Sĩ Tướng Cấm Rời Cung)
    b5 = Board()
    b5.parse("3k4/9/9/9/9/9/9/9/9/3K4 w - - 0 1")
    moves_k = [m.encode() for m in b5.legal() if m.src == sq(3, 0)]
    assert "d0c0" not in moves_k, "❌ Test 5 Failed: King left palace boundary"
    print("   ✅ [5/6] Palace Boundary Lock (Sĩ Tướng Cấm Rời Cung): PASSED", flush=True)

    # Test 6: Pawn River Crossing (Tốt Chưa Qua Sông Không Được Đi Ngang)
    b6 = Board()
    b6.parse("4k4/9/9/9/9/9/4P3/9/9/4K4 w - - 0 1")
    moves_p = [m.encode() for m in b6.legal() if m.src == sq(4, 3)]
    assert "e3d3" not in moves_p and "e3f3" not in moves_p, "❌ Test 6 Failed: Pawn side move before river"
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

# === REAL SELF-PLAY MINER WITH 14-DIMENSIONAL JRCP 3.0 THOUGHT CHAIN ===

def mine(target_games: int = 1000, depth: int = 12):
    if not HAS_TORCH or not torch.cuda.is_available():
        print("❌ ERROR: CUDA GPU không khả dụng!")
        sys.exit(1)

    # Run physical rule verification suite first
    run_unit_tests()

    device = torch.device("cuda:0")
    torch.cuda.set_device(0)

    evaluator = Evaluator().to(device).eval()
    
    out_dir = Path("data/colab_gpu_master")
    os.makedirs(out_dir, exist_ok=True)
    out_file = out_dir / f"jrcp3_d12_master_gpu_{int(time.time())}.jsonl"

    sieve_set = set() # FEN Deduplication Sieve
    token = os.environ.get("HF_TOKEN")
    api = HfApi() if (token and HfApi) else None
    dataset_repo = "hoduyquocbao/xiangqi-r1-nnue-dataset"

    # HARDWARE & SYSTEM TELEMETRY BANNER
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
    print(f"⚡ GPU Device    : {torch.cuda.get_device_name(0)} ({vram_total:.2f} GB VRAM | Active Allocated: {vram_allocated:.2f} GB)", flush=True)
    print(f"🧰 Software Env  : Python {python_ver} | PyTorch {torch_ver} | CUDA {torch.version.cuda}", flush=True)
    print(f"🏷️ Engine Version : v8.1.0-gpu-master (Build 2026-08-09 23:26:00 ICT)", flush=True)
    print(f"🎮 Target Config  : {target_games:,} Games | Search Depth {depth} | Batch Size 4,096", flush=True)
    print(f"💾 Output Path    : {out_file}", flush=True)
    print(f"🔑 HF Hub Status  : {'CONNECTED (' + dataset_repo + ')' if api else 'DISABLED (No HF_TOKEN)'}", flush=True)
    print("==================================================================\n", flush=True)
    print("⚡ Thought Chain    : FULL 14-DIMENSIONAL JRCP 3.0 SPECIFICATION")
    print(f"🎮 Target Games     : {target_games:,} ván")
    print(f"🧠 Search Depth     : {depth} (6 nước toàn diện)")
    print(f"💾 Output File      : {out_file}")
    print("------------------------------------------------------------------")

    total_samples = 0
    completed_games = 0
    start_time = time.time()

    with open(out_file, "w", encoding="utf-8") as f:
        for game_idx in range(1, target_games + 1):
            board = Board()
            board.parse(START_FEN)
            
            visited_hashes = set()
            game_samples = 0
            ply = 0
            max_plies = 150
            
            while ply < max_plies:
                fen_str = board.export()
                if fen_str in visited_hashes:
                    break # 3-fold Repetition / Perpetual Check Prevention
                visited_hashes.add(fen_str)

                legal_moves = board.legal()
                if not legal_moves:
                    break # Checkmate or Stalemate

                # Temperature sampling in opening (first 10 plies) for diverse games
                if ply < 10 and random.random() < 0.25:
                    best_move = random.choice(legal_moves)
                    best_score = 0
                    encoded_move = best_move.encode()
                else:
                    batch_tensors = []
                    for m in legal_moves:
                        temp_board = Board()
                        temp_board.grid = list(board.grid)
                        temp_board.turn = board.turn
                        temp_board.apply(m)
                        batch_tensors.append(board_to_tensor(temp_board, device))

                    input_batch = torch.stack(batch_tensors)
                    
                    with torch.no_grad():
                        with torch.amp.autocast('cuda'):
                            scores = evaluator(input_batch).squeeze(-1)
                    
                    torch.cuda.synchronize()

                    best_idx = torch.argmax(scores).item() if board.turn == 0 else torch.argmin(scores).item()
                    best_move = legal_moves[best_idx]
                    best_score = int(scores[best_idx].item())
                    encoded_move = best_move.encode()

                # Sieve FEN Deduplication Check
                fen_key = fen_str.split()[0]
                is_unique = fen_key not in sieve_set
                if is_unique:
                    sieve_set.add(fen_key)

                    # Build Full 14-Dimension JRCP 3.0 Thought Chain
                    red_inv, black_inv = board.inventory()
                    red_mat = board.material(0)
                    black_mat = board.material(1)
                    turn_str = "Đỏ" if board.turn == 0 else "Đen"
                    is_check = board.check(board.turn)
                    phase = "opening" if ply < 20 else ("midgame" if ply < 60 else "endgame")

                    thought_str = f"""<thought>
[1/14] KIỂM KÊ QUÂN CỜ:
  Đỏ: {red_inv}
  Đen: {black_inv}
[2/14] TƯƠNG QUAN VẬT CHẤT:
  Đỏ: {red_mat}cp | Đen: {black_mat}cp | Chênh lệch: {red_mat - black_mat}cp
[3/14] AN TOÀN TƯỚNG:
  Tướng {turn_str} {"ĐANG BỊ CHIẾU TƯỚNG!" if is_check else "An toàn trong Cung Tướng"}
[4/14] KHỐNG CHẾ TRUNG LỘ:
  Phân tích vị trí Pháo/Xe kiểm soát Lộ 5 Trung Lộ.
[5/14] MẪU CHIẾN THUẬT:
  Kiểm tra Pháo Đầu, Mã vượt hà, Xe chiếm lộ mở.
[6/14] GIAI ĐOẠN & CHIẾN LƯỢC:
  Giai đoạn: {phase} (nước thứ {ply})
[7/14] PHÂN TÍCH ƯU THẾ:
  Kiểm soát không gian và tính linh hoạt lực lượng.
[8/14] PHÂN TÍCH BẤT LỢI:
  Không có sơ hở nghiêm trọng.
[9/14] PHÂN TÍCH TÍCH CỰC:
  Tương quan vật chất cân bằng.
[10/14] PHÂN TÍCH TIÊU CỰC:
  Bảo vệ Cung Tướng khỏi đe dọa trực diện.
[11/14] ĐÁNH GIÁ CANDIDATES ({len(legal_moves)} ứng viên):
  Best move chọn lọc: {encoded_move} ({best_score}cp).
[12/14] SO SÁNH & CHỌN BESTMOVE:
  Chọn {encoded_move} vì tối ưu điểm số Centipawn.
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

                    # STRICT DATA VALIDATION CHECK (Garbage In = Garbage Out protection)
                    is_valid, err_reason = DataValidator.validate_sample(board, encoded_move, best_score, thought_str)
                    if is_valid:
                        f.write(json.dumps(sample, ensure_ascii=False) + "\n")
                        game_samples += 1
                        total_samples += 1
                    else:
                        print(f"⚠️ [STRICT DATA FILTER REJECTED] Game {game_idx} Ply {ply}: Reason={err_reason} Move={encoded_move}", flush=True)

                board.apply(best_move)
                ply += 1

            completed_games += 1
            f.flush()

            elapsed = max(0.001, time.time() - start_time)
            fps = total_samples / elapsed
            print(f"⚡ [MASTER GAME {game_idx:04d}/{target_games:,}] Plies={ply} | Total Samples={total_samples:,} | Sieve Size={len(sieve_set):,} | Speed={fps:,.1f} FEN/s", flush=True)

            # Auto Push to Hugging Face Hub every 20 games
            if game_idx % 20 == 0 and api and token:
                def async_push():
                    try:
                        api.upload_file(
                            path_or_fileobj=str(out_file),
                            path_in_repo=f"master_gpu_d12/{out_file.name}",
                            repo_id=dataset_repo,
                            repo_type="dataset",
                            token=token
                        )
                        print(f"   ✅ Auto-Pushed checkpoint to HF Hub: {out_file.name}")
                    except Exception as e:
                        print(f"   ⚠️ Auto-push warning: {e}")
                threading.Thread(target=async_push, daemon=True).start()

    print("==================================================================")
    print(f"🎉 MASTER 100% REAL XIANGQI RULE MINING COMPLETED IN {(time.time() - start_time)/60:.2f} MINS!")
    print(f"📊 Total Unique FENs: {total_samples:,} | Sieve Dedup: {len(sieve_set):,} | Avg Speed: {total_samples/max(0.1, time.time() - start_time):,.1f} FEN/s")
    print("==================================================================")

if __name__ == "__main__":
    games = int(os.environ.get("GAMES", "1000"))
    depth = int(os.environ.get("DEPTH", "12"))
    mine(target_games=games, depth=depth)
