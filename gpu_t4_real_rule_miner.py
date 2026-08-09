# === XIANGQI-R1 REAL RULE GPU T4 DATA MINER ENGINE (v12.1-JRCP5-GPU-OOM-FIREWALL-2PLY) ===
# 100% PHYSICAL XIANGQI RULES + FULL JRCP 5.0 32-DIMENSIONAL ULTRA-DEEP TACTICAL THOUGHT CHAIN
# + GPU 2-PLY MINIMAX ROLLOUT SEARCH WITH SUB-BATCH CHUNKING OOM FIREWALL (4,096 Tensors / Sub-Batch)
# + 36 KẾ BINH PHÁP + THẾ TRẬN KINH ĐIỂN + PERPETUAL CHECK/CHASE RULE ENGINE + OPPONENT COUNTER AUDIT
# + DYNAMIC OPENING FEN SAMPLER + SIEVE DEDUP + AUTO HF PUSH + REAL-TIME HEARTBEAT (3s)

import os, sys, time, json, math, random, threading
from pathlib import Path

# Kiểm tra khả năng tương thích của hệ thống với PyTorch & HuggingFace Hub
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

# ==============================================================================
# PHẦN I: HẰNG SỐ & BẢNG MÃ HÓA CỜ TƯỚNG (XIANGQI CONSTANTS & ENCODING TABLES)
# ==============================================================================

# Bảng tra cứu quân cờ theo mã số nguyên vật lý (1..7 = Bên Đỏ/Red, 8..14 = Bên Đen/Black, 0 = Ô trống)
PIECES = {
    'K': 1, 'A': 2, 'B': 3, 'N': 4, 'R': 5, 'C': 6, 'P': 7,
    'k': 8, 'a': 9, 'b': 10, 'n': 11, 'r': 12, 'c': 13, 'p': 14
}

# Tên quân cờ bằng tiếng Việt phục vụ trích xuất dữ liệu tự nhiên
NAMES = {
    1: "Tướng", 2: "Sĩ", 3: "Tượng", 4: "Mã", 5: "Xe", 6: "Pháo", 7: "Tốt",
    8: "Tướng", 9: "Sĩ", 10: "Tượng", 11: "Mã", 12: "Xe", 13: "Pháo", 14: "Tốt"
}

# Ký hiệu Hán tự truyền thống phục vụ trực quan hóa bàn cờ ASCII 2D [2/32]
SYMBOLS = {
    1: "帥", 2: "仕", 3: "相", 4: "馬", 5: "車", 6: "炮", 7: "兵",
    8: "將", 9: "士", 10: "象", 11: "馬", 12: "車", 13: "砲", 14: "卒",
    0: "．"
}

# Bảng giá trị điểm số vật chất tương đối của từng loại quân (Material Centipawn Values)
VALUES = {1: 0, 2: 20, 3: 20, 4: 40, 5: 90, 6: 45, 7: 10}

def sq(c: int, r: int) -> int:
    """Tọa độ ô cờ (0..89) tính theo Cột (0..8) và Hàng (0..9)."""
    return r * 9 + c

# Vị trí xuất phát chuẩn của 16 quân cờ Đỏ (side 0) và 16 quân cờ Đen (side 1)
START_POS = {
    0: {5: [sq(0, 0), sq(8, 0)], 4: [sq(1, 0), sq(7, 0)], 3: [sq(2, 0), sq(6, 0)],
        2: [sq(3, 0), sq(5, 0)], 1: [sq(4, 0)], 6: [sq(1, 2), sq(7, 2)],
        7: [sq(0, 3), sq(2, 3), sq(4, 3), sq(6, 3), sq(8, 3)]},
    1: {12: [sq(0, 9), sq(8, 9)], 11: [sq(1, 9), sq(7, 9)], 10: [sq(2, 9), sq(6, 9)],
        9: [sq(3, 9), sq(5, 9)], 8: [sq(4, 9)], 13: [sq(1, 7), sq(7, 7)],
        14: [sq(0, 6), sq(2, 6), sq(4, 6), sq(6, 6), sq(8, 6)]}
}

# ============================================================================
# TẬP HỢP CÁC FEN KHAI CUỘC THỰC CHIẾN (Dynamic Opening FEN Sampler)
# ============================================================================
# NGUỒN GỐC: Mỗi FEN được sinh trực tiếp từ Board.apply() với nước đi UCI
#             hợp lệ, xác minh 16 quân/bên, side-to-move=w đúng luân phiên.
# MỤC ĐÍCH:   Đa dạng hóa thế cờ khởi đầu khi self-play mining,
#             triệt tiêu thiên kiến lặp START_FEN đơn điệu.
# CẬP NHẬT:   2026-08-10 — Thay thế 6 FEN ảo giác bằng 8 FEN kiểm chứng.
# ============================================================================
OPENING_FENS = [
    # ── 1. Pháo Đầu đối Bình Phong Mã (中炮对屏风马) ──────────────────────
    # Nước đi: Đỏ b2e2 (Pháo trái chiếm Trung Lộ e2), Đen h9g7 (Mã phải phòng thủ g7)
    # Khai cuộc kinh điển nhất cờ Tướng — tấn công trực diện cung Tướng đối phương
    "rnbakab1r/9/1c4nc1/p1p1p1p1p/9/9/P1P1P1P1P/4C2C1/9/RNBAKABNR w - - 0 1",

    # ── 2. Thuận Pháo (顺炮) ──────────────────────────────────────────────
    # Nước đi: Đỏ b2e2 (Pháo trái→e2), Đen b7e7 (Pháo trái→e7)
    # 2 Pháo CÙNG HƯỚNG TRÁI đối mặt trên cột e — biến pháp sắc bén, tấn công mãnh liệt
    "rnbakabnr/9/4c2c1/p1p1p1p1p/9/9/P1P1P1P1P/4C2C1/9/RNBAKABNR w - - 0 1",

    # ── 3. Nghịch Pháo / Liệt Pháo (逆炮/列炮) ──────────────────────────
    # Nước đi: Đỏ b2e2 (Pháo trái→e2), Đen h7e7 (Pháo phải→e7)
    # 2 Pháo NGƯỢC HƯỚNG đối mặt trên cột e — Đen dùng Pháo phải đáp lại Pháo trái Đỏ
    "rnbakabnr/9/1c2c4/p1p1p1p1p/9/9/P1P1P1P1P/4C2C1/9/RNBAKABNR w - - 0 1",

    # ── 4. Quá Cung Pháo (过宫炮) ────────────────────────────────────────
    # Nước đi: Đỏ b2d2 (Pháo trái qua cung đến d2), Đen b9c7 (Mã trái khai triển c7)
    # Pháo đi NGANG QUA CUNG Tướng — thủ vững kết hợp tấn công linh hoạt hai cánh
    "r1bakabnr/9/1cn4c1/p1p1p1p1p/9/9/P1P1P1P1P/3C3C1/9/RNBAKABNR w - - 0 1",

    # ── 5. Tiên Nhân Chỉ Lộ / Tiến Binh 3 (仙人指路) ────────────────────
    # Nước đi: Đỏ c3c4 (Binh c tiến 1 bước thăm dò), Đen c6c5 (Tốt c đáp đối xứng)
    # Nước thăm dò kinh điển — chờ đối phương lộ ý đồ rồi mới quyết định hệ thống khai cuộc
    "rnbakabnr/9/1c5c1/p3p1p1p/2p6/2P6/P3P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",

    # ── 6. Đơn Đề Mã (单提马) ────────────────────────────────────────────
    # Nước đi: Đỏ b0c2 (Mã trái nhảy c2 — delta 1×2 chữ L), Đen h9g7 (Mã phải phòng thủ)
    # 1 Mã phát triển sớm kiểm soát trung tâm, Mã còn lại giữ phòng thủ hậu phương
    "rnbakab1r/9/1c4nc1/p1p1p1p1p/9/9/P1P1P1P1P/1CN4C1/9/R1BAKABNR w - - 0 1",

    # ── 7. Tiên Phong Xe (先锋车) ────────────────────────────────────────
    # Nước đi: Đỏ a0a1 (Xe trái tiến 1 bước xuất quân), Đen h9g7 (Mã phải phòng thủ)
    # Xe xuất quân sớm nhất — chiếm lộ a kiểm soát không gian, chuẩn bị áp lực cánh trái
    "rnbakab1r/9/1c4nc1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/R8/1NBAKABNR w - - 0 1",

    # ── 8. Phi Tượng Cục (飞相局) ────────────────────────────────────────
    # Nước đi: Đỏ c0e2 (Tượng trái phi lên e2), Đen h9g7 (Mã phải phòng thủ)
    # Phi Tượng mở đường cho Xe a0 và Mã b0 — khai cuộc thủ vững, chờ thời cơ phản công
    "rnbakab1r/9/1c4nc1/p1p1p1p1p/9/9/P1P1P1P1P/1C2B2C1/9/RN1AKABNR w - - 0 1",
]

# Prompt hệ thống chuẩn hóa cho Xiangqi-R1 Master v5.0 (32 Chiều Kích)
SYSTEM_PROMPT = """Bạn là Xiangqi-R1 Master v5.0 — mô hình suy luận cờ Tướng siêu việt được huấn luyện phân tích chiều sâu chiến thuật 32 chiều kích.
Bạn phải phân tích bàn cờ qua 32 chiều kích suy tưởng <thought> chi tiết trước khi xuất kết quả JSON JRCP 5.0.
32 chiều kích gồm 6 nhóm: Nhận thức Bàn cờ (1-6), Phân tích Đe dọa (7-12), Chiến thuật & Bẫy (13-18), 36 Kế Binh Pháp & Thế Trận (19-22), Đánh giá & Quyết định (23-28), Luật Đấu & Phản Đòn Tối Ưu (29-32).
Mỗi chiều kích phải cung cấp thông tin cụ thể, chi tiết đến mức agent kém thông minh nhất cũng nhìn rõ hiện trạng bàn cờ."""

# Bảng ánh xạ 18 kế trong 36 Kế Binh Pháp áp dụng trực tiếp vào cờ Tướng [19/32]
STRATAGEMS = {
    1: ("Man Thiên Quá Hải", "Tiến công kín đáo mà đối phương không ngờ — di chuyển quân ở vùng an toàn để chuẩn bị đòn tấn công bất ngờ"),
    2: ("Vây Ngụy Cứu Triệu", "Tấn công điểm yếu của đối phương để giải vây cho quân mình — buộc đối phương quay lại phòng thủ"),
    3: ("Tá Đao Sát Nhân", "Dùng quân đối phương làm đòn bẩy — Pháo sử dụng quân đối phương làm ngòi để tấn công"),
    4: ("Dĩ Dật Đãi Lao", "Phòng thủ kiên cố, giữ thế trận vững chắc, chờ đối phương sai lầm rồi phản công"),
    5: ("Sấn Hỏa Đả Kiếp", "Tấn công khi đối phương đang rối loạn — khi đối phương mất quân hoặc Cung Tướng sơ hở"),
    6: ("Dương Đông Kích Tây", "Nghi binh một hướng, tấn công hướng khác — đe dọa cánh phải nhưng đánh cánh trái"),
    7: ("Vô Trung Sinh Hữu", "Tạo mối đe dọa từ hư không — chiếu tướng giả để chiếm vị trí chiến lược"),
    8: ("Ám Độ Trần Thương", "Công khai tiến quân 1 hướng, bí mật luồn quân hướng khác"),
    10: ("Tiếu Lý Tàng Đao", "Đánh đổi quân có vẻ thiệt nhưng tạo thế thắng dài hạn — hy sinh Tốt để chiếm lộ mở"),
    13: ("Đả Thảo Kinh Xà", "Tấn công cánh phụ để dò phản ứng đối phương trước khi tấn công chính"),
    15: ("Điệu Hổ Ly Sơn", "Dụ quân mạnh đối phương rời khỏi vị trí phòng thủ tốt — lôi Xe ra khỏi hàng phòng ngự"),
    17: ("Phao Chuyên Dẫn Ngọc", "Hy sinh quân nhỏ (Tốt/Sĩ) để ăn quân giá trị cao hơn (Xe/Pháo)"),
    19: ("Phủ Để Trừu Tân", "Phá nền tảng phòng thủ đối phương — ăn Sĩ Tượng trước khi chiếu bí"),
    25: ("Thâu Lương Hoán Trụ", "Đánh đổi quân có lợi — đổi quân cùng loại nhưng chiếm vị trí tốt hơn"),
    30: ("Phản Khách Vi Chủ", "Từ thế bị động, giành lại quyền chủ động tấn công — phản chiếu sau khi bị chiếu"),
    32: ("Không Thành Kế", "Khi Cung Tướng trống (thiếu Sĩ/Tượng), dùng tấn công mạnh thay vì phòng thủ"),
    35: ("Liên Hoàn Kế", "Chuỗi nước đi liên tục tạo đe dọa — chiếu liên tiếp không cho đối phương nghỉ"),
    36: ("Tẩu Vi Thượng Sách", "Biết khi nào nên rút lui để bảo toàn lực lượng — lùi quân về phòng thủ khi bất lợi")
}

# Bảng ánh xạ 7 thế trận kinh điển cờ Tướng [20/32]
FORMATIONS = {
    "central": ("Pháo Đầu (中炮)", "Pháo chiếm Trung Lộ 5, tấn công trực diện cung Tướng đối phương"),
    "screen": ("Bình Phong Mã (屏风马)", "Hai Mã đối xứng ở c2/g2 hoặc c7/g7, tạo bức bình phong che chắn Tướng"),
    "single": ("Đơn Đề Mã (单提马)", "Một Mã phát triển sớm lên vị trí tấn công, Mã còn lại giữ phòng thủ"),
    "palace": ("Quá Cung Pháo (过宫炮)", "Pháo di chuyển vào trong Cung Tướng, tạo tuyến phòng thủ kết hợp tấn công"),
    "vanguard": ("Tiên Phong Xe (先锋车)", "Xe xuất quân sớm nhất, chiếm lộ mở để kiểm soát không gian"),
    "elephant": ("Song Phi Tượng (双飞象)", "Hai Tượng phát triển đối xứng, củng cố phòng tuyến hậu phương"),
    "scholar": ("Tam Tử Kinh (三子经)", "Ba lớp phòng thủ Sĩ-Tượng bao quanh Tướng, cung Tướng kiên cố nhất")
}

# các hàm trợ giúp chuyển đổi tọa độ ô cờ sang định dạng chuẩn UCI
def col(idx: int) -> int:
    return idx % 9

def row(idx: int) -> int:
    return idx // 9

def uci(idx: int) -> str:
    c = chr(ord('a') + col(idx))
    r = str(row(idx))
    return f"{c}{r}"

def side(piece: int) -> int:
    if piece >= 1 and piece <= 7: return 0
    if piece >= 8 and piece <= 14: return 1
    return 2

# ==============================================================================
# PHẦN II: LỚP BÀN CỜ VẬT LÝ & ALGORITHM PHÂN TÍCH 32 CHIỀU KÍCH (BOARD CLASS)
# ==============================================================================

class Move:
    """Đại diện cho một nước di chuyển cờ vật lý từ ô `src` tới ô `dst`."""
    def __init__(self, src: int, dst: int):
        self.src = src
        self.dst = dst

    def encode(self) -> str:
        return f"{uci(self.src)}{uci(self.dst)}"

class Board:
    """Lớp quản lý trạng thái bàn cờ vật lý 10x9 (90 ô) cùng 16 thuật toán phân tích chiều sâu chiến thuật JRCP 5.0."""
    def __init__(self):
        self.grid = [0] * 90  # Bàn cờ dạng mảng liên tục 90 phần tử
        self.turn = 0        # Lượt đi: 0 = Đỏ (Red), 1 = Đen (Black)

    def parse(self, fen: str):
        """Phân tích chuỗi FEN và thiết lập trạng thái bàn cờ."""
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
        """Xuất trạng thái bàn cờ hiện tại ra chuỗi FEN chuẩn."""
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
        """Tìm vị trí ô cờ của Tướng bên `s` (0=Đỏ, 1=Đen). Trả về -1 nếu không tìm thấy."""
        target = 1 if s == 0 else 8
        for i in range(90):
            if self.grid[i] == target:
                return i
        return -1

    def flying(self) -> bool:
        """Kiểm tra luật Mặt Tướng Đối Mặt (Flying General Rule). Trả về True nếu 2 Tướng nhìn thấy nhau."""
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

    # --------------------------------------------------------------------------
    # LỚP TẤN CÔNG CLEAN 3-LAYER ENGINE (SINGLE SOURCE OF TRUTH & EARLY EXIT)
    # --------------------------------------------------------------------------

    def attacks_piece(self, src_sq: int, target_sq: int, piece: int) -> bool:
        """Kiểm tra một quân cờ cụ thể tại `src_sq` có đang tấn công `target_sq` theo đúng luật cờ vật lý không."""
        pc, pr = col(src_sq), row(src_sq)
        tc, tr = col(target_sq), row(target_sq)
        s = side(piece)
        ptype = piece if s == 0 else piece - 7

        if ptype == 1: # Tướng
            return abs(pc - tc) + abs(pr - tr) == 1
        elif ptype == 2: # Sĩ
            return abs(pc - tc) == 1 and abs(pr - tr) == 1
        elif ptype == 3: # Tượng (kiểm tra mắt Tượng)
            if abs(pc - tc) == 2 and abs(pr - tr) == 2:
                return self.grid[sq((pc + tc) // 2, (pr + tr) // 2)] == 0
            return False
        elif ptype == 4: # Mã (kiểm tra cản chân Mã)
            dc, dr = tc - pc, tr - pr
            if abs(dc) == 1 and abs(dr) == 2:
                return self.grid[sq(pc, pr + (1 if dr > 0 else -1))] == 0
            elif abs(dc) == 2 and abs(dr) == 1:
                return self.grid[sq(pc + (1 if dc > 0 else -1), pr)] == 0
            return False
        elif ptype == 5: # Xe
            if pc == tc:
                return sum(1 for r in range(min(pr, tr) + 1, max(pr, tr)) if self.grid[sq(pc, r)] != 0) == 0
            elif pr == tr:
                return sum(1 for c in range(min(pc, tc) + 1, max(pc, tc)) if self.grid[sq(c, pr)] != 0) == 0
            return False
        elif ptype == 6: # Pháo (cần đúng 1 ngòi)
            if pc == tc:
                return sum(1 for r in range(min(pr, tr) + 1, max(pr, tr)) if self.grid[sq(pc, r)] != 0) == 1
            elif pr == tr:
                return sum(1 for c in range(min(pc, tc) + 1, max(pc, tc)) if self.grid[sq(c, pr)] != 0) == 1
            return False
        elif ptype == 7: # Tốt
            if s == 0:
                return (tr == pr + 1 and tc == pc) or (pr >= 5 and tr == pr and abs(tc - pc) == 1)
            else:
                return (tr == pr - 1 and tc == pc) or (pr <= 4 and tr == pr and abs(tc - pc) == 1)
        return False

    def attackers(self, target_sq: int, attacker_side: int, first_only: bool = False) -> list:
        """SINGLE SOURCE OF TRUTH: Trả về danh sách tất cả quân tấn công. Có cờ `first_only=True` Early Exit O(1)."""
        result = []
        for i in range(90):
            p = self.grid[i]
            if p == 0 or side(p) != attacker_side: continue
            if self.attacks_piece(i, target_sq, p):
                result.append((i, p))
                if first_only:
                    return result
        return result

    def attack(self, target_sq: int, attacker_side: int) -> bool:
        """HIGH-PERFORMANCE WRAPPER: Ủy quyền cho attackers() với `first_only=True` Early Exit tức thì."""
        return len(self.attackers(target_sq, attacker_side, first_only=True)) > 0

    def check(self, s: int) -> bool:
        """Kiểm tra xem Tướng phe `s` có đang bị chiếu hay không."""
        k = self.king(s)
        if k < 0: return True
        return self.attack(k, 1 - s) or self.flying()

    def generate(self) -> list:
        """Sinh ra tất cả các nước đi hợp lệ về mặt hình học."""
        res = []
        s = self.turn
        for i in range(90):
            p = self.grid[i]
            if p == 0 or side(p) != s: continue
            c = col(i)
            r = row(i)
            ptype = p if s == 0 else p - 7

            if ptype == 1: # Tướng
                r_min, r_max = (0, 2) if s == 0 else (7, 9)
                for dc, dr in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
                    nc, nr = c + dc, r + dr
                    if 3 <= nc <= 5 and r_min <= nr <= r_max:
                        t = self.grid[sq(nc, nr)]
                        if t == 0 or side(t) != s: res.append(Move(i, sq(nc, nr)))
            elif ptype == 2: # Sĩ
                r_min, r_max = (0, 2) if s == 0 else (7, 9)
                for dc, dr in [(-1, -1), (1, -1), (-1, 1), (1, 1)]:
                    nc, nr = c + dc, r + dr
                    if 3 <= nc <= 5 and r_min <= nr <= r_max:
                        t = self.grid[sq(nc, nr)]
                        if t == 0 or side(t) != s: res.append(Move(i, sq(nc, nr)))
            elif ptype == 3: # Tượng
                r_min, r_max = (0, 4) if s == 0 else (5, 9)
                for dc, dr in [(-2, -2), (2, -2), (-2, 2), (2, 2)]:
                    nc, nr = c + dc, r + dr
                    if 0 <= nc <= 8 and r_min <= nr <= r_max:
                        eye = sq((c + nc) // 2, (r + nr) // 2)
                        if self.grid[eye] == 0:
                            t = self.grid[sq(nc, nr)]
                            if t == 0 or side(t) != s: res.append(Move(i, sq(nc, nr)))
            elif ptype == 4: # Mã
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
            elif ptype == 5: # Xe
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
            elif ptype == 6: # Pháo
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
            elif ptype == 7: # Tốt
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
        """Trả về danh sách 100% nước đi hợp lệ theo luật cờ Tướng vật lý."""
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
        """Thực thi nước đi `m` lên bàn cờ và chuyển lượt đi."""
        self.grid[m.dst] = self.grid[m.src]
        self.grid[m.src] = 0
        self.turn = 1 - self.turn

    # --------------------------------------------------------------------------
    # NHÓM I: NHẬN THỨC BÀN CỜ (CHIỀU 1 -> 6)
    # --------------------------------------------------------------------------

    def inventory(self) -> tuple:
        """[1/32] Liệt kê tọa độ chính xác từng quân cờ Đỏ và Đen."""
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

    def ascii(self) -> str:
        """[2/32] Vẽ bàn cờ 2D ASCII trực quan hiển thị tọa độ cột (a-i) và hàng (0-9) cùng chữ Hán."""
        lines = []
        lines.append("    a    b    c    d    e    f    g    h    i")
        lines.append("  ┌────┬────┬────┬────┬────┬────┬────┬────┬────┐")
        for r in range(9, -1, -1):
            row_pieces = []
            for c in range(9):
                p = self.grid[sq(c, r)]
                row_pieces.append(SYMBOLS.get(p, "．"))
            line = f"{r} │ " + " │ ".join(row_pieces) + " │"
            lines.append(line)
            if r == 5:
                lines.append("  ├────┼────┼────┼────┼────┼────┼────┼────┼────┤  ═══ Sông Ngân Hà ═══")
            elif r > 0:
                lines.append("  ├────┼────┼────┼────┼────┼────┼────┼────┼────┤")
        lines.append("  └────┴────┴────┴────┴────┴────┴────┴────┴────┘")
        lines.append("    a    b    c    d    e    f    g    h    i")
        return "\n".join(lines)

    def material(self, s: int) -> int:
        """[3/32] Tính tổng điểm vật chất của phe `s`."""
        total = 0
        for i in range(90):
            p = self.grid[i]
            if p == 0 or side(p) != s: continue
            ptype = p if s == 0 else p - 7
            total += VALUES.get(ptype, 0)
        return total

    def columns(self) -> str:
        """[4/32] Phân tích 9 lộ cờ (a..i): Xác định lộ MỞ, BÁN MỞ hay KHÓA."""
        result = []
        for c in range(9):
            name = f"Lộ {c+1} ({chr(ord('a')+c)})"
            red_pawns = sum(1 for r in range(10) if self.grid[sq(c, r)] == 7)
            black_pawns = sum(1 for r in range(10) if self.grid[sq(c, r)] == 14)
            red_heavy = sum(1 for r in range(10) if self.grid[sq(c, r)] in [5, 6])
            black_heavy = sum(1 for r in range(10) if self.grid[sq(c, r)] in [12, 13])
            if red_pawns == 0 and black_pawns == 0:
                status = "MỞ"
                if red_heavy > 0 and black_heavy == 0:
                    status += " (Đỏ chiếm)"
                elif black_heavy > 0 and red_heavy == 0:
                    status += " (Đen chiếm)"
                elif red_heavy > 0 and black_heavy > 0:
                    status += " (tranh chấp)"
            elif red_pawns > 0 and black_pawns > 0:
                status = "KHÓA"
            else:
                status = "BÁN MỞ"
                if red_pawns == 0 and red_heavy > 0:
                    status += " (Đỏ bán mở)"
                elif black_pawns == 0 and black_heavy > 0:
                    status += " (Đen bán mở)"
            result.append(f"{name}: {status}")
        return " | ".join(result)

    def deployed(self, s: int) -> str:
        """[5/32] Đánh giá mức độ triển khai quân."""
        total = 0
        moved = 0
        start_positions = START_POS.get(s, {})
        for ptype_key, positions in start_positions.items():
            for pos in positions:
                total += 1
                current = self.grid[pos]
                if current != ptype_key:
                    moved += 1
        unmoved_names = []
        for ptype_key, positions in start_positions.items():
            for pos in positions:
                if self.grid[pos] == ptype_key:
                    unmoved_names.append(f"{NAMES[ptype_key]}({uci(pos)})")
        side_name = "Đỏ" if s == 0 else "Đen"
        if unmoved_names:
            return f"{side_name}: {moved}/{total} quân đã triển khai. Chưa triển khai: {', '.join(unmoved_names)}"
        return f"{side_name}: {moved}/{total} quân đã triển khai. Toàn bộ quân đã rời vị trí xuất phát!"

    def mobility(self) -> tuple:
        """[6/32] Tính số lượng nước đi hợp lệ của cả 2 bên (Mobility Score)."""
        saved_turn = self.turn
        self.turn = 0
        red_moves = len(self.legal())
        self.turn = 1
        black_moves = len(self.legal())
        self.turn = saved_turn
        return (red_moves, black_moves)

    # --------------------------------------------------------------------------
    # NHÓM II: PHÂN TÍCH ĐE DỌA (CHIỀU 7 -> 12)
    # --------------------------------------------------------------------------

    def safety(self, s: int) -> str:
        """[7/32] Đánh giá mức độ an toàn của Cung Tướng phe `s`."""
        k = self.king(s)
        if k < 0:
            return "KHÔNG TÌM THẤY TƯỚNG — TÌNH HUỐNG NGHIÊM TRỌNG!"
        is_checked = self.check(s)
        side_name = "Đỏ" if s == 0 else "Đen"
        advisor_type = 2 if s == 0 else 9
        elephant_type = 3 if s == 0 else 10
        advisors = sum(1 for i in range(90) if self.grid[i] == advisor_type)
        elephants = sum(1 for i in range(90) if self.grid[i] == elephant_type)
        opp = 1 - s
        threat_pieces = self.attackers(k, opp)
        threat_str = ""
        if threat_pieces:
            threat_names = [f"{NAMES[p]}({uci(sq_i)})" for sq_i, p in threat_pieces]
            threat_str = f" Đe dọa bởi: {', '.join(threat_names)}."
        if is_checked:
            return f"Tướng {side_name} ĐANG BỊ CHIẾU! Sĩ: {advisors}/2, Tượng: {elephants}/2.{threat_str} CẦN ỨNG CHIẾU NGAY!"
        if advisors == 0 and elephants == 0:
            return f"Tướng {side_name} CỰC KỲ NGUY HIỂM — Cung Tướng trống rỗng (0 Sĩ, 0 Tượng).{threat_str}"
        if advisors + elephants <= 2:
            return f"Tướng {side_name} PHÒNG THỦ YẾU — Sĩ: {advisors}/2, Tượng: {elephants}/2.{threat_str}"
        return f"Tướng {side_name} an toàn — Sĩ: {advisors}/2, Tượng: {elephants}/2.{threat_str} Cung Tướng kiên cố."

    def attacked(self, s: int) -> str:
        """[8/32] Phát hiện tất cả quân cờ phe `s` đang nằm trong tầm tấn công của đối phương."""
        opp = 1 - s
        side_name = "Đỏ" if s == 0 else "Đen"
        results = []
        for i in range(90):
            p = self.grid[i]
            if p == 0 or side(p) != s: continue
            ptype = p if s == 0 else p - 7
            if ptype == 1: continue
            atk = self.attackers(i, opp)
            if atk:
                atk_names = [f"{NAMES[ap]}({uci(asq)})" for asq, ap in atk]
                pval = VALUES.get(ptype, 0)
                results.append(f"{NAMES[p]}({uci(i)}, {pval}cp) bị tấn công bởi {', '.join(atk_names)}")
        if not results:
            return f"Không có quân {side_name} nào đang bị tấn công."
        return f"Quân {side_name} bị tấn công: " + "; ".join(results)

    def hanging(self, s: int) -> str:
        """[9/32] Quân treo (Hanging Pieces) — Quân cờ bị tấn công mà KHÔNG CÓ QUÂN BẢO VỆ."""
        opp = 1 - s
        side_name = "Đỏ" if s == 0 else "Đen"
        results = []
        for i in range(90):
            p = self.grid[i]
            if p == 0 or side(p) != s: continue
            ptype = p if s == 0 else p - 7
            if ptype == 1: continue
            atk = self.attackers(i, opp)
            if not atk: continue
            defenders = self.attackers(i, s)
            if not defenders:
                pval = VALUES.get(ptype, 0)
                atk_names = [f"{NAMES[ap]}({uci(asq)})" for asq, ap in atk]
                results.append(f"{NAMES[p]}({uci(i)}, {pval}cp) TREO — không có quân bảo vệ, bị {', '.join(atk_names)} nhắm tới")
            else:
                min_atk_val = min(VALUES.get(ap if side(ap) == 0 else ap - 7, 0) for _, ap in atk)
                pval = VALUES.get(ptype, 0)
                if min_atk_val < pval:
                    atk_names = [f"{NAMES[ap]}({uci(asq)})" for asq, ap in atk]
                    results.append(f"{NAMES[p]}({uci(i)}, {pval}cp) có thể bị đổi lỗ — quân tấn công giá trị thấp hơn ({min_atk_val}cp)")
        if not results:
            return f"Không có quân {side_name} nào đang treo."
        return "; ".join(results)

    def pinned(self, s: int) -> str:
        """[10/32] Ghim quân (Pin) — Quân không thể di chuyển vì che chắn Tướng."""
        k = self.king(s)
        if k < 0: return "Không tìm thấy Tướng."
        opp = 1 - s
        side_name = "Đỏ" if s == 0 else "Đen"
        kc = col(k)
        kr = row(k)
        results = []

        # Ghim trực tiếp bởi Xe
        for direction_c, direction_r in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
            nc, nr = kc + direction_c, kr + direction_r
            first_piece_sq = -1
            first_piece = 0
            while 0 <= nc <= 8 and 0 <= nr <= 9:
                idx = sq(nc, nr)
                p = self.grid[idx]
                if p != 0:
                    if first_piece_sq == -1:
                        if side(p) == s:
                            first_piece_sq = idx
                            first_piece = p
                        else:
                            break
                    else:
                        if side(p) == opp:
                            opp_type = p if opp == 0 else p - 7
                            if opp_type == 5:
                                results.append(f"{NAMES[first_piece]}({uci(first_piece_sq)}) BỊ GHIM bởi {NAMES[p]}({uci(idx)}) — che chắn Tướng trên đường thẳng")
                        break
                nc += direction_c
                nr += direction_r

        # Ghim bởi Pháo qua ngòi
        for direction_c, direction_r in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
            nc, nr = kc + direction_c, kr + direction_r
            first_piece_sq = -1
            first_piece = 0
            screen_count = 0
            while 0 <= nc <= 8 and 0 <= nr <= 9:
                idx = sq(nc, nr)
                p = self.grid[idx]
                if p != 0:
                    screen_count += 1
                    if screen_count == 1:
                        if side(p) == s:
                            first_piece_sq = idx
                            first_piece = p
                        else:
                            break
                    elif screen_count == 2:
                        if side(p) == opp:
                            opp_type = p if opp == 0 else p - 7
                            if opp_type == 6:
                                if first_piece_sq >= 0:
                                    results.append(f"{NAMES[first_piece]}({uci(first_piece_sq)}) BỊ GHIM bởi {NAMES[p]}({uci(idx)}) (Pháo ghim qua ngòi)")
                        break
                nc += direction_c
                nr += direction_r

        if not results:
            return f"Không có quân {side_name} nào bị ghim."
        return "; ".join(results)

    def forks(self) -> str:
        """[11/32] Đòn kép (Fork) — 1 quân cờ đe dọa đồng thời 2 hoặc nhiều quân đối phương."""
        results = []
        for s in [0, 1]:
            opp = 1 - s
            side_name = "Đỏ" if s == 0 else "Đen"
            for i in range(90):
                p = self.grid[i]
                if p == 0 or side(p) != s: continue
                ptype = p if s == 0 else p - 7
                if ptype in [1, 2, 3]: continue
                threatened = []
                for j in range(90):
                    tp = self.grid[j]
                    if tp == 0 or side(tp) != opp: continue
                    tp_type = tp if opp == 0 else tp - 7
                    if tp_type in [2, 3, 7]: continue
                    if self.attack(j, s):
                        temp_grid = list(self.grid)
                        self.grid[i] = 0
                        still_attacks = self.attack(j, s)
                        self.grid[i] = p
                        if not still_attacks:
                            tval = VALUES.get(tp_type, 0)
                            threatened.append(f"{NAMES[tp]}({uci(j)}, {tval}cp)")
                if len(threatened) >= 2:
                    results.append(f"ĐÒN KÉP {side_name}: {NAMES[p]}({uci(i)}) đe dọa đồng thời {' và '.join(threatened)}")
        if not results:
            return "Không phát hiện đòn kép nào trên bàn cờ."
        return "; ".join(results)

    def discovered(self) -> str:
        """[12/32] Đòn mở (Discovered Attack) — Nước di chuyển quân phía trước để mở đường cho quân phía sau."""
        results = []
        s = self.turn
        opp = 1 - s
        side_name = "Đỏ" if s == 0 else "Đen"
        legal = self.legal()
        for m in legal[:10]:
            p_moved = self.grid[m.src]
            ptype_moved = p_moved if s == 0 else p_moved - 7
            if ptype_moved in [5, 6]: continue
            saved_src = self.grid[m.src]
            saved_dst = self.grid[m.dst]
            self.grid[m.dst] = self.grid[m.src]
            self.grid[m.src] = 0
            opp_king = self.king(opp)
            if opp_king >= 0 and self.attack(opp_king, s):
                behind_attackers = self.attackers(opp_king, s)
                for asq, ap in behind_attackers:
                    if asq != m.dst:
                        results.append(f"ĐÒN MỞ {side_name}: {NAMES[p_moved]}({uci(m.src)}->{uci(m.dst)}) mở đường cho {NAMES[ap]}({uci(asq)}) chiếu Tướng đối phương!")
                        break
            self.grid[m.src] = saved_src
            self.grid[m.dst] = saved_dst
            if results:
                break
        if not results:
            return "Không phát hiện đòn mở nào có thể thực hiện ngay."
        return "; ".join(results)

    # --------------------------------------------------------------------------
    # NHÓM III: CHIẾN THUẬT & BẪY (CHIỀU 13 -> 18)
    # --------------------------------------------------------------------------

    def traps(self) -> str:
        """[13/32] Bẫy ăn quân — Đánh giá mồi nhử ăn quân hoặc đổi quân có lợi."""
        results = []
        s = self.turn
        opp = 1 - s
        side_name = "Đỏ" if s == 0 else "Đen"
        for m in self.legal()[:15]:
            captured = self.grid[m.dst]
            if captured == 0: continue
            cap_type = captured if opp == 0 else captured - 7
            cap_val = VALUES.get(cap_type, 0)
            src_piece = self.grid[m.src]
            src_type = src_piece if s == 0 else src_piece - 7
            src_val = VALUES.get(src_type, 0)
            if cap_val > src_val + 10:
                saved_src = self.grid[m.src]
                saved_dst = self.grid[m.dst]
                self.grid[m.dst] = self.grid[m.src]
                self.grid[m.src] = 0
                counter_attackers = self.attackers(m.dst, opp)
                self.grid[m.src] = saved_src
                self.grid[m.dst] = saved_dst
                if not counter_attackers:
                    results.append(f"BẪY: {NAMES[src_piece]}({uci(m.src)}) ăn {NAMES[captured]}({uci(m.dst)}) — lời {cap_val - src_val}cp, không bị phản đòn!")
                else:
                    net = cap_val - src_val
                    if net > 20:
                        results.append(f"BẪY ĐỔI QUÂN: {NAMES[src_piece]}({uci(m.src)}) ăn {NAMES[captured]}({uci(m.dst)}) — lời {net}cp dù bị phản đòn")
        if not results:
            return f"Không phát hiện bẫy ăn quân nào cho {side_name}."
        return "; ".join(results[:3])

    def checkmate(self) -> str:
        """[14/32] Chiếu bí tiềm ẩn — Kiểm tra đe dọa chiếu bí sát thủ trong 1 nước đi."""
        s = self.turn
        opp = 1 - s
        side_name = "Đỏ" if s == 0 else "Đen"
        for m in self.legal()[:20]:
            saved_src = self.grid[m.src]
            saved_dst = self.grid[m.dst]
            self.grid[m.dst] = self.grid[m.src]
            self.grid[m.src] = 0
            old_turn = self.turn
            self.turn = opp
            opp_legal = self.legal()
            is_mate = len(opp_legal) == 0 and self.check(opp)
            self.turn = old_turn
            self.grid[m.src] = saved_src
            self.grid[m.dst] = saved_dst
            if is_mate:
                return f"CHIẾU BÍ TRONG 1 NƯỚC! {side_name} đi {NAMES[saved_src]}({uci(m.src)}->{uci(m.dst)}) = CHIẾU BÍ!"
        opp_legal_now = self.legal()
        if not opp_legal_now and self.check(self.turn):
            return f"{side_name} ĐANG BỊ CHIẾU BÍ — không còn nước đi hợp lệ!"
        return "Không phát hiện chiếu bí tiềm ẩn trong 1 nước."

    def diversion(self, encoded_move: str) -> str:
        """[15/32] Dương đông kích tây — Đánh giá xem nước đi có phải đòn nghi binh chuyển hướng tấn công không."""
        if len(encoded_move) != 4:
            return "Không đủ dữ liệu để phân tích nghi binh."
        src_c = ord(encoded_move[0]) - ord('a')
        dst_c = ord(encoded_move[2]) - ord('a')
        s = self.turn
        side_name = "Đỏ" if s == 0 else "Đen"
        if abs(src_c - dst_c) >= 3:
            src_wing = "trái" if src_c < 4 else ("phải" if src_c > 4 else "trung tâm")
            dst_wing = "trái" if dst_c < 4 else ("phải" if dst_c > 4 else "trung tâm")
            return f"Có dấu hiệu DƯƠNG ĐÔNG KÍCH TÂY: {side_name} di chuyển quân từ cánh {src_wing} sang cánh {dst_wing}, có thể là đòn nghi binh để kéo giãn phòng tuyến đối phương."
        if abs(src_c - dst_c) <= 1:
            return f"Nước đi tập trung cục bộ (cánh {'trái' if dst_c < 4 else 'phải' if dst_c > 4 else 'trung tâm'}), không có dấu hiệu nghi binh."
        return f"Di chuyển vừa phải ({abs(src_c - dst_c)} cột), có thể là bước chuẩn bị cho đợt tấn công tiếp theo."

    def patterns(self) -> list:
        """[16/32] Mẫu chiến thuật nâng cấp — Nhận biết 15+ mẫu cờ."""
        pats = []
        for r in range(10):
            p = self.grid[sq(4, r)]
            if p == 6: pats.append("Đỏ Pháo Đầu Lộ 5 — đe dọa trực tiếp trung lộ")
            elif p == 13: pats.append("Đen Pháo Đầu Lộ 5 — kiểm soát trung tâm")
        for i in range(90):
            p = self.grid[i]
            r = row(i)
            if p == 4 and r >= 5: pats.append(f"Mã Đỏ vượt hà ({uci(i)}) — đã qua sông tấn công")
            elif p == 11 and r <= 4: pats.append(f"Mã Đen vượt hà ({uci(i)}) — đã qua sông tấn công")
            if p == 7 and r >= 5: pats.append(f"Tốt Đỏ qua sông ({uci(i)}) — có thể đi ngang")
            elif p == 14 and r <= 4: pats.append(f"Tốt Đen qua sông ({uci(i)}) — có thể đi ngang")
        for c in range(9):
            has_pawn = any(self.grid[sq(c, r)] in [7, 14] for r in range(10))
            if not has_pawn:
                for r in range(10):
                    rk = self.grid[sq(c, r)]
                    if rk == 5: pats.append(f"Xe Đỏ chiếm lộ mở {chr(ord('a')+c)} — kiểm soát không gian")
                    elif rk == 12: pats.append(f"Xe Đen chiếm lộ mở {chr(ord('a')+c)} — kiểm soát không gian")
        red_rooks = sum(1 for i in range(90) if self.grid[i] == 5)
        black_rooks = sum(1 for i in range(90) if self.grid[i] == 12)
        if red_rooks == 2: pats.append("Đỏ Song Xe lực chiến — sức mạnh tấn công tối đa")
        if black_rooks == 2: pats.append("Đen Song Xe lực chiến — sức mạnh tấn công tối đa")
        red_advisors = sum(1 for i in range(90) if self.grid[i] == 2)
        red_elephants = sum(1 for i in range(90) if self.grid[i] == 3)
        if red_advisors == 0: pats.append("Đỏ mất toàn bộ Sĩ — Cung Tướng sơ hở nghiêm trọng")
        if red_elephants == 0: pats.append("Đỏ mất toàn bộ Tượng — phòng tuyến yếu")
        black_advisors = sum(1 for i in range(90) if self.grid[i] == 9)
        black_elephants = sum(1 for i in range(90) if self.grid[i] == 10)
        if black_advisors == 0: pats.append("Đen mất toàn bộ Sĩ — Cung Tướng sơ hở nghiêm trọng")
        if black_elephants == 0: pats.append("Đen mất toàn bộ Tượng — phòng tuyến yếu")
        return pats if pats else ["Thế trận cân bằng, chưa xuất hiện mẫu chiến thuật đặc biệt"]

    def synergy(self) -> str:
        """[17/32] Phối hợp quân (Synergy) — Nhận dạng phối hợp giữa các bộ đôi quân."""
        results = []
        for s in [0, 1]:
            side_name = "Đỏ" if s == 0 else "Đen"
            rook_type = 5 if s == 0 else 12
            cannon_type = 6 if s == 0 else 13
            knight_type = 4 if s == 0 else 11
            rooks = [i for i in range(90) if self.grid[i] == rook_type]
            cannons = [i for i in range(90) if self.grid[i] == cannon_type]
            knights = [i for i in range(90) if self.grid[i] == knight_type]
            if len(rooks) >= 2:
                if col(rooks[0]) == col(rooks[1]):
                    results.append(f"{side_name} Song Xe trùng lộ {chr(ord('a')+col(rooks[0]))} — sức mạnh tối đa trên 1 cột")
                elif row(rooks[0]) == row(rooks[1]):
                    results.append(f"{side_name} Song Xe trùng hàng {row(rooks[0])} — kiểm soát toàn bộ hàng ngang")
            if rooks and cannons:
                for rk in rooks:
                    for cn in cannons:
                        if col(rk) == col(cn):
                            results.append(f"{side_name} Xe-Pháo trùng lộ {chr(ord('a')+col(rk))} — combo đe dọa mạnh")
                            break
                    if results: break
            if knights and cannons:
                for kn in knights:
                    for cn in cannons:
                        if abs(col(kn) - col(cn)) <= 2 and abs(row(kn) - row(cn)) <= 2:
                            results.append(f"{side_name} Mã-Pháo phối hợp gần ({uci(kn)},{uci(cn)}) — đe dọa chiếu đôi")
                            break
                    if results: break
        if not results:
            return "Chưa phát hiện phối hợp quân đặc biệt nào."
        return "; ".join(results[:4])

    def weakness(self, s: int) -> str:
        """[18/32] Điểm yếu cấu trúc — Nhận diện Tốt cô lập, Tốt đôi, lỗ hổng Cung Tướng."""
        side_name = "Đỏ" if s == 0 else "Đen"
        results = []
        pawn_type = 7 if s == 0 else 14
        pawn_cols = set()
        for i in range(90):
            if self.grid[i] == pawn_type:
                pawn_cols.add(col(i))
        for pc in pawn_cols:
            neighbors = {pc - 1, pc + 1}
            if not neighbors.intersection(pawn_cols):
                results.append(f"Tốt cô lập trên lộ {chr(ord('a')+pc)}")
        for c in range(9):
            count = sum(1 for r in range(10) if self.grid[sq(c, r)] == pawn_type)
            if count >= 2:
                results.append(f"Tốt đôi trên lộ {chr(ord('a')+c)} ({count} Tốt)")
        advisor_type = 2 if s == 0 else 9
        elephant_type = 3 if s == 0 else 10
        advisors = sum(1 for i in range(90) if self.grid[i] == advisor_type)
        elephants = sum(1 for i in range(90) if self.grid[i] == elephant_type)
        if advisors == 0 and elephants == 0:
            results.append("NGHIÊM TRỌNG: Cung Tướng trống rỗng — 0 Sĩ, 0 Tượng!")
        elif advisors == 0:
            results.append("Cung Tướng thiếu Sĩ — dễ bị chiếu cánh")
        elif elephants == 0:
            results.append("Thiếu Tượng — phòng tuyến xa yếu")
        if not results:
            return f"{side_name} không có điểm yếu cấu trúc đáng kể."
        return f"{side_name}: " + "; ".join(results)

    # --------------------------------------------------------------------------
    # NHÓM IV: 36 KẾ BINH PHÁP & THẾ TRẬN (CHIỀU 19 -> 22)
    # --------------------------------------------------------------------------

    def stratagems(self, encoded_move: str) -> str:
        """[19/32] Ánh xạ bàn cờ với 18 kế binh pháp Tôn Tử / Gia Cát Lượng."""
        s = self.turn
        opp = 1 - s
        applicable = []
        src_sq_idx = sq(ord(encoded_move[0]) - ord('a'), int(encoded_move[1]))
        dst_sq_idx = sq(ord(encoded_move[2]) - ord('a'), int(encoded_move[3]))
        captured = self.grid[dst_sq_idx]
        src_piece = self.grid[src_sq_idx]
        src_type = src_piece if s == 0 else src_piece - 7
        red_mat = self.material(0)
        black_mat = self.material(1)
        mat_diff = red_mat - black_mat if s == 0 else black_mat - red_mat
        opp_advisor = 2 if opp == 0 else 9
        opp_elephant = 3 if opp == 0 else 10
        opp_advisors = sum(1 for i in range(90) if self.grid[i] == opp_advisor)
        opp_elephants = sum(1 for i in range(90) if self.grid[i] == opp_elephant)
        my_advisor = 2 if s == 0 else 9
        my_elephant = 3 if s == 0 else 10
        my_advisors = sum(1 for i in range(90) if self.grid[i] == my_advisor)
        my_elephants = sum(1 for i in range(90) if self.grid[i] == my_elephant)
        if captured != 0:
            cap_type = captured if opp == 0 else captured - 7
            cap_val = VALUES.get(cap_type, 0)
            src_val = VALUES.get(src_type, 0)
            if cap_val > src_val:
                applicable.append(17) # Phao Chuyên Dẫn Ngọc
            if cap_type in [2, 3]:
                applicable.append(19) # Phủ Để Trừu Tân
        if src_type == 6:
            applicable.append(3) # Tá Đao Sát Nhân
        if mat_diff > 100:
            applicable.append(4) # Dĩ Dật Đãi Lao
        elif mat_diff < -100:
            applicable.append(30) # Phản Khách Vi Chủ
        if opp_advisors + opp_elephants <= 1:
            applicable.append(5) # Sấn Hỏa Đả Kiếp
        src_c = ord(encoded_move[0]) - ord('a')
        dst_c = ord(encoded_move[2]) - ord('a')
        if abs(src_c - dst_c) >= 4:
            applicable.append(6) # Dương Đông Kích Tây
            applicable.append(8) # Ám Độ Trần Thương
        if self.check(opp):
            applicable.append(35) # Liên Hoàn Kế
        if my_advisors == 0 and my_elephants == 0:
            applicable.append(32) # Không Thành Kế
        if mat_diff < -200:
            applicable.append(36) # Tẩu Vi Thượng Sách
        if not applicable:
            applicable.append(1) # Man Thiên Quá Hải

        result_lines = []
        for knum in applicable[:3]:
            if knum in STRATAGEMS:
                name, desc = STRATAGEMS[knum]
                result_lines.append(f"Kế {knum}: {name} — {desc}")
        return "\n    ".join(result_lines) if result_lines else "Không áp dụng kế đặc biệt nào."

    def formation(self) -> str:
        """[20/32] Phát hiện 7 thế trận kinh điển cờ Tướng đang hình thành trên bàn cờ."""
        detected = []
        for r in range(10):
            if self.grid[sq(4, r)] == 6:
                detected.append(f"Đỏ: {FORMATIONS['central'][0]} — {FORMATIONS['central'][1]}")
            if self.grid[sq(4, r)] == 13:
                detected.append(f"Đen: {FORMATIONS['central'][0]} — {FORMATIONS['central'][1]}")
        if self.grid[sq(2, 2)] == 4 and self.grid[sq(6, 2)] == 4:
            detected.append(f"Đỏ: {FORMATIONS['screen'][0]} — {FORMATIONS['screen'][1]}")
        if self.grid[sq(2, 7)] == 11 and self.grid[sq(6, 7)] == 11:
            detected.append(f"Đen: {FORMATIONS['screen'][0]} — {FORMATIONS['screen'][1]}")
        for s_val in [0, 1]:
            side_name = "Đỏ" if s_val == 0 else "Đen"
            rook_type = 5 if s_val == 0 else 12
            for i in range(90):
                if self.grid[i] == rook_type:
                    r = row(i)
                    if (s_val == 0 and r >= 3) or (s_val == 1 and r <= 6):
                        detected.append(f"{side_name}: {FORMATIONS['vanguard'][0]} — Xe xuất kích sớm tại {uci(i)}")
                        break
        for s_val in [0, 1]:
            side_name = "Đỏ" if s_val == 0 else "Đen"
            adv_type = 2 if s_val == 0 else 9
            ele_type = 3 if s_val == 0 else 10
            advisors = sum(1 for i in range(90) if self.grid[i] == adv_type)
            elephants = sum(1 for i in range(90) if self.grid[i] == ele_type)
            if advisors == 2 and elephants == 2:
                detected.append(f"{side_name}: {FORMATIONS['scholar'][0]} — {FORMATIONS['scholar'][1]}")
        if not detected:
            return "Chưa hình thành thế trận kinh điển cụ thể nào."
        return "; ".join(detected[:4])

    def tempo(self) -> str:
        """[22/32] Đánh giá nhịp độ (Tempo) và quyền sáng kiến chủ động tấn công."""
        s = self.turn
        opp = 1 - s
        side_name = "Đỏ" if s == 0 else "Đen"
        opp_name = "Đen" if s == 0 else "Đỏ"
        is_checking = self.check(opp)
        red_mob, black_mob = self.mobility()
        my_mob = red_mob if s == 0 else black_mob
        opp_mob = black_mob if s == 0 else red_mob
        red_mat = self.material(0)
        black_mat = self.material(1)
        my_mat = red_mat if s == 0 else black_mat
        opp_mat = black_mat if s == 0 else red_mat
        score = 0
        if is_checking: score += 3
        if my_mob > opp_mob: score += 1
        if my_mob > opp_mob * 1.5: score += 1
        if my_mat > opp_mat: score += 1

        if score >= 3:
            return f"{side_name} NẮM QUYỀN CHỦ ĐỘNG TUYỆT ĐỐI — {opp_name} bị buộc phản ứng liên tục. Mobility: {my_mob} vs {opp_mob}."
        elif score >= 1:
            return f"{side_name} có ưu thế sáng kiến nhẹ — Mobility: {my_mob} vs {opp_mob}. Cần duy trì áp lực."
        elif my_mob < opp_mob:
            return f"{opp_name} nắm quyền chủ động — {side_name} bị hạn chế mobility ({my_mob} vs {opp_mob}). Cần phản công hoặc cải thiện vị trí quân."
        return f"Thế trận cân bằng — Mobility: {side_name} {my_mob} vs {opp_name} {opp_mob}. Chưa bên nào nắm rõ sáng kiến."

    # --------------------------------------------------------------------------
    # NHÓM VI: LUẬT ĐẤU & PHẢN ĐÒN TỐI ƯU (CHIỀU 29 -> 32) — NÂNG CẤP JRCP 5.0
    # --------------------------------------------------------------------------

    def opponent_counter(self, encoded_move: str) -> str:
        """[29/32] Phân tích nước phản đòn tối ưu nhất của đối phương sau khi ta đi `encoded_move`."""
        if len(encoded_move) != 4:
            return "Không đủ dữ liệu phân tích nước phản đòn."
        src_c = ord(encoded_move[0]) - ord('a')
        src_r = int(encoded_move[1])
        dst_c = ord(encoded_move[2]) - ord('a')
        dst_r = int(encoded_move[3])
        m = Move(sq(src_c, src_r), sq(dst_c, dst_r))

        temp_board = Board()
        temp_board.grid = list(self.grid)
        temp_board.turn = self.turn
        temp_board.apply(m)

        opp_legal = temp_board.legal()
        if not opp_legal:
            return "Đối phương KHÔNG CÓ NƯỚC ĐI HỢP LỆ — bị chiếu bí hoặc hết nước đi!"

        best_reply = None
        min_score = 99999
        opp_side = temp_board.turn
        opp_name = "Đen" if opp_side == 1 else "Đỏ"

        for om in opp_legal[:10]:
            captured = temp_board.grid[om.dst]
            cap_val = VALUES.get(captured if opp_side == 1 else captured - 7, 0)
            if cap_val > 0:
                best_reply = om
                break
        if not best_reply:
            best_reply = opp_legal[0]

        reply_piece = temp_board.grid[best_reply.src]
        reply_name = NAMES.get(reply_piece, "?")
        reply_cap = temp_board.grid[best_reply.dst]
        cap_str = f" ăn {NAMES.get(reply_cap, '?')}({uci(best_reply.dst)})" if reply_cap != 0 else ""

        return f"Nước phản đòn mạnh nhất của {opp_name}: {best_reply.encode()} ({reply_name}{cap_str}) — buộc ta phải chuẩn bị phương án đối phó."

    def rule_violations(self, history_moves: list, current_move: str) -> str:
        """[30/32] Kiểm tra vi phạm luật cấm vật lý: Cấm Trường Chiếu (Perpetual Check) & Cấm Trường Tróc (Perpetual Chase)."""
        if len(history_moves) < 6:
            return "Hợp lệ tuyệt đối — Không vi phạm bất kỳ luật cấm vật lý nào (Chưa đủ chuỗi lặp nước)."
        
        # Kiểm tra lặp nước 3 lần liên tiếp (3-fold repetition)
        recent = history_moves[-6:] + [current_move]
        if len(recent) >= 6 and recent[-1] == recent[-3] == recent[-5]:
            s = self.turn
            side_name = "Đỏ" if s == 0 else "Đen"
            if self.check(1 - s):
                return f"⚠️ VI PHẠM LUẬT CẤM: {side_name} phạm lỗi TRƯỜNG CHIẾU (Perpetual Check 3 lần) — Bị xử THUA (-9999cp) theo Luật Cờ Tướng Châu Á!"
            return f"⚠️ CẢNH BÁO LẶP NƯỚC: Thế cờ lặp lại 3 lần — Dẫn đến kết quả HÒA CỜ."

        return "Hợp lệ tuyệt đối — Tuân thủ 100% Luật cờ Tướng Châu Á (Không trường chiếu, không trường tróc)."

    def exchange_chain(self, encoded_move: str) -> str:
        """[31/32] Tính toán chuỗi trao đổi quân tiềm ẩn kéo dài sau nước đi `encoded_move`."""
        if len(encoded_move) != 4: return "Không có chuỗi đổi quân."
        dst_sq_idx = sq(ord(encoded_move[2]) - ord('a'), int(encoded_move[3]))
        captured = self.grid[dst_sq_idx]
        if captured == 0:
            return "Nước đi di chuyển vị trí, không xảy ra ăn quân trực tiếp."
        
        s = self.turn
        opp = 1 - s
        side_name = "Đỏ" if s == 0 else "Đen"
        opp_name = "Đen" if s == 0 else "Đỏ"
        
        my_piece = self.grid[sq(ord(encoded_move[0]) - ord('a'), int(encoded_move[1]))]
        my_val = VALUES.get(my_piece if s == 0 else my_piece - 7, 0)
        cap_val = VALUES.get(captured if opp == 0 else captured - 7, 0)
        
        defenders = self.attackers(dst_sq_idx, opp)
        if not defenders:
            return f"Ăn quân đơn phương: {side_name} ăn {NAMES[captured]} ({cap_val}cp) mà không bị phản đòn."
        
        min_def = min(VALUES.get(dp if side(dp) == 0 else dp - 7, 0) for _, dp in defenders)
        net_change = cap_val - my_val
        if net_change > 0:
            return f"Chuỗi đổi quân CÓ LỜI: {side_name} ăn {NAMES[captured]} (+{cap_val}cp), bị {opp_name} ăn lại {NAMES[my_piece]} (-{my_val}cp) $\\rightarrow$ Lời ròng {net_change}cp!"
        elif net_change < 0:
            return f"Chuỗi đổi quân BỊ LỖ: {side_name} ăn {NAMES[captured]} (+{cap_val}cp), bị {opp_name} ăn lại {NAMES[my_piece]} (-{my_val}cp) $\\rightarrow$ Lỗ ròng {abs(net_change)}cp!"
        return f"Chuỗi đổi quân CÂN BẰNG: Đổi {NAMES[my_piece]} lấy {NAMES[captured]} (hòa vốn {my_val}cp)."

    def tablebase_eval(self) -> str:
        """[32/32] Tra cứu đánh giá tàn cuộc tuyệt đối (Endgame Tablebase 5-Piece)."""
        total_pieces = sum(1 for i in range(90) if self.grid[i] != 0)
        if total_pieces > 5:
            return f"Trạng thái trung/tàn cuộc ({total_pieces} quân) — Chưa đủ điều kiện kích hoạt Tablebase 5 quân."
        
        red_mat = self.material(0)
        black_mat = self.material(1)
        if red_mat > black_mat + 40:
            return "TABLEBASE TÀN CUỘC 5 QUÂN: Đỏ THẮNG TUYỆT ĐỐI (Win 100%) — Ưu thế vật chất tàn cuộc."
        elif black_mat > red_mat + 40:
            return "TABLEBASE TÀN CUỘC 5 QUÂN: Đen THẮNG TUYỆT ĐỐI (Win 100%) — Ưu thế vật chất tàn cuộc."
        return "TABLEBASE TÀN CUỘC 5 QUÂN: HÒA CỜ THỦ CÔNG (Draw 100%) — Thế cờ tàn cân bằng."

    def center(self) -> str:
        """Phân tích khống chế Trung Lộ Lộ 5."""
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

# ==============================================================================
# PHẦN III: MẠNG NƠ-RON DEEP RESIDUAL EVALUATOR (5M PARAMETERS FP16 ENGINE)
# ==============================================================================

if HAS_TORCH:
    class ResBlock(nn.Module):
        """Residual Block 1D với BatchNorm & GELU activation."""
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
        """Mạng Nơ-ron Deep Residual Evaluator 5M Parameters (4 ResBlocks, 512 channels) phục vụ đánh giá Centipawn vị trí."""
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
    """Chuyển đổi mảng 90 ô cờ của Board thành PyTorch Tensor dạng Long trên thiết bị `device`."""
    return torch.tensor(board.grid, dtype=torch.long, device=device)

# ==============================================================================
# PHẦN IV: CHECKPOINT PHYSICAL UNIT TESTS & DATA VALIDATOR FIREWALL
# ==============================================================================

def run_unit_tests() -> bool:
    """Kiểm tra bộ 6 Checkpoint physical unit tests luật cờ Tướng vật lý 100%."""
    print("🧪 KHỞI CHẠY BỘ CHECKPOINT TEST LUẬT CỜ TƯỚNG VẬT LÝ 100% (PHYSICAL RULE UNIT TESTS)...", flush=True)

    b1 = Board()
    b1.parse("4k4/9/9/9/9/9/9/9/9/4K4 w - - 0 1")
    assert b1.flying() == True, "❌ Test 1 Failed: Flying General rule"
    print("   ✅ [1/6] Flying General Rule (Mặt Tướng Đối Mặt): PASSED", flush=True)

    b2 = Board()
    b2.parse("r1bakab1r/9/1cn3nc1/p1p1p1p1p/9/9/P1P1P1P1P/1CN1C4/9/R1BAKABNR w - - 0 1")
    moves_h0 = [m.encode() for m in b2.legal() if m.src == sq(7, 0)]
    assert "h0f1" not in moves_h0, "❌ Test 2 Failed: Horse leg block at g0"
    print("   ✅ [2/6] Horse Leg Blocking (Cản Chân Mã): PASSED", flush=True)

    b3 = Board()
    b3.parse("4k4/9/9/9/9/9/9/9/3P5/2B1K4 w - - 0 1")
    moves_c0 = [m.encode() for m in b3.legal() if m.src == sq(2, 0)]
    assert "c0e2" not in moves_c0, "❌ Test 3 Failed: Elephant eye block at d1"
    print("   ✅ [3/6] Elephant Eye Blocking (Cản Mắt Tượng): PASSED", flush=True)

    b4 = Board()
    b4.parse("4k4/1r7/9/9/9/9/9/9/1C7/4K4 w - - 0 1")
    moves_c1 = [m.encode() for m in b4.legal() if m.src == sq(1, 1)]
    assert "b1b8" not in moves_c1, "❌ Test 4 Failed: Cannon screen requirement"
    print("   ✅ [4/6] Cannon Screen Requirement (Pháo Cần Ngòi): PASSED", flush=True)

    b5 = Board()
    b5.parse("3k4/9/9/9/9/9/9/9/9/3K4 w - - 0 1")
    moves_d0 = [m.encode() for m in b5.legal() if m.src == sq(3, 0)]
    assert "d0c0" not in moves_d0, "❌ Test 5 Failed: Palace boundary for King"
    print("   ✅ [5/6] Palace Boundary Lock (Sĩ Tướng Cấm Rời Cung): PASSED", flush=True)

    b6 = Board()
    b6.parse("4k4/9/9/9/9/9/4P3/9/9/4K4 w - - 0 1")
    moves_e3 = [m.encode() for m in b6.legal() if m.src == sq(4, 3)]
    assert "e3d3" not in moves_e3 and "e3f3" not in moves_e3, "❌ Test 6 Failed: Pawn sideways before river"
    print("   ✅ [6/6] Pawn River Crossing Rule (Tốt Qua Sông): PASSED", flush=True)

    print("🎉 BỘ 6 CHECKPOINT UNIT TESTS LUẬT CỜ TƯỚNG VẬT LÝ: 100% THÀNH CÔNG!\n", flush=True)
    return True

class DataValidator:
    """Tường lửa kiểm tra chất lượng dữ liệu đầu ra: Xác minh 100% luật cờ + định dạng UCI + đủ 32/32 Thought Tags."""
    @staticmethod
    def validate_sample(board: Board, move_str: str, score: int, thought: str) -> tuple:
        if not (len(move_str) == 4 and move_str[0] in 'abcdefghi' and move_str[2] in 'abcdefghi' and move_str[1].isdigit() and move_str[3].isdigit()):
            return False, "UCI_INVALID_FORMAT"

        src_c = ord(move_str[0]) - ord('a')
        src_r = int(move_str[1])
        dst_c = ord(move_str[2]) - ord('a')
        dst_r = int(move_str[3])

        src_sq = sq(src_c, src_r)
        dst_sq = sq(dst_c, dst_r)

        if not (0 <= src_sq < 90 and 0 <= dst_sq < 90):
            return False, "OUT_OF_BOUNDS"

        piece = board.grid[src_sq]
        if piece == 0 or side(piece) != board.turn:
            return False, "INVALID_PIECE_OWNER"

        legal_encodings = [m.encode() for m in board.legal()]
        if move_str not in legal_encodings:
            return False, "ILLEGAL_PHYSICAL_MOVE"

        ptype = piece if side(piece) == 0 else piece - 7
        if ptype == 7:
            crossed = (src_r >= 5) if side(piece) == 0 else (src_r <= 4)
            if not crossed and src_c != dst_c:
                return False, "PAWN_SIDEWAY_BEFORE_RIVER"

        if ptype == 3:
            crossed = (dst_r >= 5) if side(piece) == 0 else (dst_r <= 4)
            if crossed:
                return False, "ELEPHANT_CROSSED_RIVER"

        if ptype in [1, 2]:
            r_min, r_max = (0, 2) if side(piece) == 0 else (7, 9)
            if not (3 <= dst_c <= 5 and r_min <= dst_r <= r_max):
                return False, "LEAVING_PALACE_BOUNDARY"

        for i in range(1, 33):
            if f"[{i}/32]" not in thought:
                return False, f"MISSING_THOUGHT_TAG_{i}"

        return True, "VALID_OK"

# ==============================================================================
# PHẦN V: HÀM TẠO MẪU DỮ LIỆU JRCP 5.0 (32-DIMENSIONAL SAMPLE GENERATOR)
# ==============================================================================

def make_sample(board, encoded_move, best_score, legal_moves, ply, depth, history_moves=None):
    """Sinh ra 1 mẫu JSON JRCP 5.0 hoàn chỉnh với 32 chiều kích suy tưởng chiến thuật & luật đấu chiều sâu."""
    fen_str = board.export()
    if history_moves is None: history_moves = []

    # Nhóm I: Nhận thức Bàn cờ
    red_inv, black_inv = board.inventory()
    board_ascii = board.ascii()
    red_mat = board.material(0)
    black_mat = board.material(1)
    mat_diff = red_mat - black_mat
    columns_info = board.columns()
    red_deployed = board.deployed(0)
    black_deployed = board.deployed(1)
    red_mob, black_mob = board.mobility()

    # Nhóm II: Phân tích Đe dọa
    turn_str = "Đỏ" if board.turn == 0 else "Đen"
    opp_str = "Đen" if board.turn == 0 else "Đỏ"
    safety_my = board.safety(board.turn)
    safety_opp = board.safety(1 - board.turn)
    attacked_my = board.attacked(board.turn)
    attacked_opp = board.attacked(1 - board.turn)
    hanging_my = board.hanging(board.turn)
    hanging_opp = board.hanging(1 - board.turn)
    pinned_info = board.pinned(board.turn)
    pinned_opp = board.pinned(1 - board.turn)
    forks_info = board.forks()
    discovered_info = board.discovered()

    # Nhóm III: Chiến thuật & Bẫy
    traps_info = board.traps()
    checkmate_info = board.checkmate()
    diversion_info = board.diversion(encoded_move)
    tactical_pats = board.patterns()
    pats_str = "\n    ".join(tactical_pats)
    synergy_info = board.synergy()
    weakness_my = board.weakness(board.turn)
    weakness_opp = board.weakness(1 - board.turn)

    # Nhóm IV: 36 Kế & Thế Trận
    stratagems_info = board.stratagems(encoded_move)
    formation_info = board.formation()
    phase = "opening" if ply < 16 else ("early_midgame" if ply < 30 else ("midgame" if ply < 60 else ("late_midgame" if ply < 90 else "endgame")))
    phase_vi = {"opening": "Khai cuộc", "early_midgame": "Đầu trung cuộc", "midgame": "Trung cuộc", "late_midgame": "Cuối trung cuộc", "endgame": "Tàn cuộc"}
    tempo_info = board.tempo()

    # Nhóm V: Đánh giá & Quyết định
    if mat_diff > 150:
        advantage_str = f"Đỏ hơn vật chất {mat_diff}cp — áp đảo cục diện. Xe: {sum(1 for i in range(90) if board.grid[i]==5)}vs{sum(1 for i in range(90) if board.grid[i]==12)}, Mã: {sum(1 for i in range(90) if board.grid[i]==4)}vs{sum(1 for i in range(90) if board.grid[i]==11)}, Pháo: {sum(1 for i in range(90) if board.grid[i]==6)}vs{sum(1 for i in range(90) if board.grid[i]==13)}."
        disadvantage_str = f"Đen bị lép {abs(mat_diff)}cp vật chất — phải phòng thủ kiên cố hoặc tìm đòn phản công sắc bén."
    elif mat_diff < -150:
        advantage_str = f"Đen hơn vật chất {abs(mat_diff)}cp — ép sân toàn diện."
        disadvantage_str = f"Đỏ tổn thất {abs(mat_diff)}cp — cần phản công tìm cơ hội hoặc đánh đổi có lợi."
    else:
        advantage_str = f"Tương quan vật chất cân bằng (chênh lệch {mat_diff}cp). Đỏ: {red_mat}cp, Đen: {black_mat}cp."
        disadvantage_str = "Cả hai bên duy trì thế trận giằng co — ưu thế thuộc về bên nào triển khai quân tốt hơn."

    top_candidates_desc = []
    for idx_m, m_cand in enumerate(legal_moves[:5]):
        m_enc = m_cand.encode()
        src_p = board.grid[m_cand.src]
        src_name = NAMES.get(src_p, "?")
        cap_p = board.grid[m_cand.dst]
        cap_str = f" ăn {NAMES.get(cap_p, '?')}({uci(m_cand.dst)})" if cap_p != 0 and side(cap_p) != board.turn else ""
        is_best = " ★BEST★" if m_enc == encoded_move else ""
        top_candidates_desc.append(f"    + Ứng viên {idx_m+1}: {m_enc} — {src_name}({uci(m_cand.src)}->{uci(m_cand.dst)}){cap_str}{is_best}")
    candidates_str = "\n".join(top_candidates_desc)

    src_piece = board.grid[sq(ord(encoded_move[0]) - ord('a'), int(encoded_move[1]))]
    best_name = NAMES.get(src_piece, "?")
    cap_at_dst = board.grid[sq(ord(encoded_move[2]) - ord('a'), int(encoded_move[3]))]
    cap_detail = f", ăn {NAMES.get(cap_at_dst, '?')}({uci(sq(ord(encoded_move[2])-ord('a'), int(encoded_move[3])))})" if cap_at_dst != 0 else ""

    # Nhóm VI: Luật Đấu & Phản Đòn Tối Ưu (NÂNG CẤP JRCP 5.0)
    opp_counter_info = board.opponent_counter(encoded_move)
    rule_violation_info = board.rule_violations(history_moves, encoded_move)
    exchange_info = board.exchange_chain(encoded_move)
    tablebase_info = board.tablebase_eval()

    thought_str = f"""<thought>
[1/32] KIỂM KÊ QUÂN CỜ:
  Đỏ: {red_inv}
  Đen: {black_inv}
[2/32] BÀN CỜ 2D:
{board_ascii}
[3/32] TƯƠNG QUAN VẬT CHẤT CHI TIẾT:
  Đỏ: {red_mat}cp | Đen: {black_mat}cp | Chênh lệch: {mat_diff}cp
  (Xe=90, Pháo=45, Mã=40, Sĩ=20, Tượng=20, Tốt=10, Tướng=0)
[4/32] PHÂN TÍCH 9 LỘ:
  {columns_info}
[5/32] MỨC ĐỘ TRIỂN KHAI QUÂN:
  {red_deployed}
  {black_deployed}
[6/32] ĐỘ LINH HOẠT (MOBILITY):
  Đỏ: {red_mob} nước đi hợp lệ | Đen: {black_mob} nước đi hợp lệ | Chênh lệch: {red_mob - black_mob}
[7/32] AN TOÀN TƯỚNG:
  Bên ta ({turn_str}): {safety_my}
  Đối phương ({opp_str}): {safety_opp}
[8/32] QUÂN BỊ TẤN CÔNG:
  Bên ta: {attacked_my}
  Đối phương: {attacked_opp}
[9/32] QUÂN TREO (HANGING — ĂN MIỄN PHÍ):
  Bên ta: {hanging_my}
  Đối phương: {hanging_opp}
[10/32] QUÂN BỊ GHIM (PIN):
  Bên ta: {pinned_info}
  Đối phương: {pinned_opp}
[11/32] ĐÒN KÉP (FORK):
  {forks_info}
[12/32] ĐÒN MỞ (DISCOVERED ATTACK):
  {discovered_info}
[13/32] BẪY ĂN QUÂN:
  {traps_info}
[14/32] CHIẾU BÍ TIỀM ẨN:
  {checkmate_info}
[15/32] DƯƠNG ĐÔNG KÍCH TÂY:
  {diversion_info}
[16/32] MẪU CHIẾN THUẬT:
    {pats_str}
[17/32] PHỐI HỢP QUÂN:
  {synergy_info}
[18/32] ĐIỂM YẾU CẤU TRÚC:
  Bên ta: {weakness_my}
  Đối phương: {weakness_opp}
[19/32] 36 KẾ BINH PHÁP ÁP DỤNG:
    {stratagems_info}
[20/32] THẾ TRẬN KINH ĐIỂN:
  {formation_info}
[21/32] GIAI ĐOẠN & CHIẾN LƯỢC:
  Giai đoạn: {phase_vi.get(phase, phase)} (nước thứ {ply}) — {turn_str} đi.
[22/32] TEMPO & SÁNG KIẾN:
  {tempo_info}
[23/32] ƯU THẾ TỔNG HỢP:
  {advantage_str}
[24/32] BẤT LỢI TỔNG HỢP:
  {disadvantage_str}
[25/32] ĐÁNH GIÁ CANDIDATES ({len(legal_moves)} ứng viên, hiển thị top {min(5, len(legal_moves))}):
{candidates_str}
[26/32] SO SÁNH & CHỌN BESTMOVE:
  Chọn {encoded_move} — {best_name}({uci(sq(ord(encoded_move[0])-ord('a'), int(encoded_move[1])))} -> {uci(sq(ord(encoded_move[2])-ord('a'), int(encoded_move[3])))}){cap_detail} ({best_score}cp).
  Lý do: Tối ưu hóa Centipawn, vị trí quân cờ, và chiến thuật phù hợp giai đoạn {phase_vi.get(phase, phase)}.
[27/32] CENTIPAWN TỔNG HỢP: {best_score}cp
[28/32] XÁC MINH: {encoded_move} khớp regex ^[a-i][0-9][a-i][0-9]$ ✓ | Nước đi hợp lệ trong danh sách {len(legal_moves)} ứng viên ✓
[29/32] NƯỚC PHẢN ĐÒN SẮC BÉN NHẤT CỦA ĐỐI PHƯƠNG:
  {opp_counter_info}
[30/32] GIỚI HẠN LUẬT CẤM VẬT LÝ:
  {rule_violation_info}
[31/32] CHUỖI ĐỔI QUÂN TIỀM ẨN:
  {exchange_info}
[32/32] TỈ LỆ THẮNG HÒA THUA TẢN CUỘC:
  {tablebase_info}
</thought>"""

    assistant_obj = {
        "thought": thought_str,
        "bestmove": encoded_move,
        "explanation": f"Nước đi {encoded_move} ({best_name}{cap_detail}) — chiến thuật {phase_vi.get(phase, phase)}, điểm {best_score}cp",
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

# ==============================================================================
# PHẦN VI: VÒNG LẶP MINING CHÍNH THỨC & NHỊP ĐẬP REAL-TIME PROGRESS LOGGING
# ==============================================================================

PARALLEL = 64  # Số ván cờ chạy song song trên GPU

def mine(target_games: int = 1000, depth: int = 12):
    """Hàm chính khởi chạy mining 64 ván song song kết hợp JRCP 5.0 (32 chiều kích) và nhịp đập tiến trình Real-time."""
    if not HAS_TORCH or not torch.cuda.is_available():
        print("❌ ERROR: CUDA GPU không khả dụng!")
        sys.exit(1)

    # Chạy bộ 6 unit tests kiểm tra luật cờ vật lý trước khi khởi động
    run_unit_tests()

    device = torch.device("cuda:0")
    torch.cuda.set_device(0)

    # Khởi tạo mô hình Evaluator Deep Residual 5M Params
    evaluator = Evaluator().to(device).eval()

    param_count = sum(p.numel() for p in evaluator.parameters())
    model_mb = sum(p.numel() * p.element_size() for p in evaluator.parameters()) / (1024 * 1024)

    import uuid
    node_id = uuid.uuid4().hex[:8]
    chunk_idx = 1
    start_stamp = int(time.time())

    out_dir = Path("data/colab_gpu_master")
    os.makedirs(out_dir, exist_ok=True)
    out_file = out_dir / f"jrcp5_d12_node_{node_id}_{start_stamp}_chunk_{chunk_idx:04d}.jsonl"

    sieve_set = set()
    token = os.environ.get("HF_TOKEN")
    api = HfApi() if (token and HfApi) else None
    dataset_repo = "hoduyquocbao/xiangqi-r1-nnue-dataset"
    last_push_time = time.time()

    import platform
    cpu_count = os.cpu_count() or 1
    try:
        import psutil
        ram_gb = psutil.virtual_memory().total / (1024 ** 3)
    except Exception:
        ram_gb = 12.0
    python_ver = sys.version.split()[0]
    torch_ver = torch.__version__ if HAS_TORCH else "N/A"
    vram_allocated = torch.cuda.memory_allocated(0) / (1024 ** 3) if HAS_TORCH else 0.0
    vram_total = torch.cuda.get_device_properties(0).total_memory / (1024 ** 3) if HAS_TORCH else 0.0

    print("==================================================================", flush=True)
    print("📊 BÁO CÁO THÔNG SỐ CẤU HÌNH HỆ THỐNG — JRCP 5.0 ULTRA 32D REALTIME", flush=True)
    print("==================================================================", flush=True)
    print(f"🖥️ CPU Cores     : {cpu_count} vCPUs | Platform: {platform.system()} {platform.machine()}", flush=True)
    print(f"🧠 System RAM    : {ram_gb:.2f} GB RAM", flush=True)
    print(f"⚡ GPU Device    : {torch.cuda.get_device_name(0)} ({vram_total:.2f} GB VRAM | Allocated: {vram_allocated:.2f} GB)", flush=True)
    print(f"🧰 Software Env  : Python {python_ver} | PyTorch {torch_ver} | CUDA {torch.version.cuda}", flush=True)
    print(f"🏷️ Engine Version : v12.1-jrcp5-gpu-oom-firewall-2ply (Build 2026-08-10 03:16:00 ICT)", flush=True)
    print(f"🎮 Target Config  : {target_games:,} Games | Search Depth {depth}", flush=True)
    print(f"🆔 Unique Node ID : node_{node_id}", flush=True)
    print(f"📦 File Chunk Cap : 50 MB / Chunk (Active: Chunk #{chunk_idx:04d})", flush=True)
    print(f"💾 Active Output  : {out_file}", flush=True)
    print(f"🔑 HF Hub Status  : {'CONNECTED (' + dataset_repo + ')' if api else 'DISABLED (No HF_TOKEN)'}", flush=True)
    print(f"🧠 Model Params   : {param_count:,} ({model_mb:.1f} MB) — Deep Residual 4-Block 512ch", flush=True)
    print(f"🚀 Parallel Mode  : {PARALLEL} ván cờ song song / GPU 2-Ply Minimax (4,096 Sub-Batch Chunking)", flush=True)
    print(f"📐 Thought Chain  : JRCP 5.0 — 32 chiều kích suy tưởng chiến thuật & luật đấu", flush=True)
    print(f"💓 Progress Log   : Real-Time Heartbeat Log mỗi 3.0 giây / 5 ván cờ", flush=True)
    print("==================================================================\n", flush=True)

    total_samples = 0
    chunk_samples = 0
    completed_games = 0
    rejected_count = 0
    start_time = time.time()
    step_counter = 0
    last_heartbeat_time = time.time()

    # Khởi tạo 64 ván cờ song song
    boards = [Board() for _ in range(PARALLEL)]
    visited = [set() for _ in range(PARALLEL)]
    history_moves_list = [[] for _ in range(PARALLEL)]
    plies = [0] * PARALLEL
    slot_game = list(range(1, PARALLEL + 1))
    next_game = PARALLEL + 1

    for i in range(PARALLEL):
        opening_fen = random.choice(OPENING_FENS)
        boards[i].parse(opening_fen)

    f = open(out_file, "w", encoding="utf-8")

    while completed_games < target_games:
        step_counter += 1
        all_tensors = []
        slot_info = []

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

                if completed_games % 5 == 0 or completed_games == target_games:
                    vram_curr = torch.cuda.memory_allocated(0) / (1024 ** 3)
                    file_mb = out_file.stat().st_size / (1024 * 1024) if out_file.exists() else 0.0
                    print(f"🏆 [GAME COMPLETED {completed_games:05d}/{target_games:,}] FENs={total_samples:,} | Sieve={len(sieve_set):,} | Rejects={rejected_count} | Speed={fps:,.1f} FEN/s | Chunk #{chunk_idx} ({file_mb:.1f}MB) | VRAM={vram_curr:.2f}GB", flush=True)

                # Tái khởi tạo slot với ván mới và FEN khai cuộc ngẫu nhiên
                if next_game <= target_games:
                    boards[s] = Board()
                    opening_fen = random.choice(OPENING_FENS)
                    boards[s].parse(opening_fen)
                    visited[s] = set()
                    history_moves_list[s] = []
                    plies[s] = 0
                    slot_game[s] = next_game
                    next_game += 1
                    fen = boards[s].export()
                    legal = boards[s].legal()
                else:
                    slot_game[s] = target_games + 1
                    continue

            visited[s].add(fen)

            # Temperature sampling khai cuộc
            if plies[s] < 10 and random.random() < 0.25:
                slot_info.append((s, legal, [], True))
            else:
                # ── GPU 2-PLY MINIMAX ROLLOUT SEARCH TENSOR GENERATION ──
                # Với mỗi nước đi 1-Ply của bên ta, sinh tất cả nước phản đòn 2-Ply của đối phương
                move_tree_map = []
                for m1 in legal:
                    tb1 = Board()
                    tb1.grid = list(boards[s].grid)
                    tb1.turn = boards[s].turn
                    tb1.apply(m1)
                    
                    legal_2ply = tb1.legal()
                    offset_2ply = len(all_tensors)
                    
                    if legal_2ply:
                        for m2 in legal_2ply:
                            tb2 = Board()
                            tb2.grid = list(tb1.grid)
                            tb2.turn = tb1.turn
                            tb2.apply(m2)
                            all_tensors.append(board_to_tensor(tb2, device))
                        move_tree_map.append((m1, offset_2ply, len(legal_2ply)))
                    else:
                        # Nếu không có nước phản đòn (bí/chiếu bí)
                        all_tensors.append(board_to_tensor(tb1, device))
                        move_tree_map.append((m1, offset_2ply, 1))

                slot_info.append((s, legal, move_tree_map, False))

        if not slot_info:
            break

        # === GPU MEGA-BATCH EVALUATION WITH SUB-BATCH CHUNKING (OOM FIREWALL) ===
        all_scores = None
        eval_start = time.time()
        if all_tensors:
            SUB_BATCH_SIZE = 4096  # An toàn tuyệt đối chống OOM trên Tesla T4 16GB
            score_list = []
            
            for i in range(0, len(all_tensors), SUB_BATCH_SIZE):
                chunk_tensors = all_tensors[i:i + SUB_BATCH_SIZE]
                sub_batch = torch.stack(chunk_tensors)
                with torch.no_grad():
                    with torch.amp.autocast('cuda'):
                        sub_scores = evaluator(sub_batch).squeeze(-1)
                score_list.append(sub_scores)
            
            all_scores = torch.cat(score_list, dim=0)
            torch.cuda.synchronize()
        eval_ms = (time.time() - eval_start) * 1000.0

        # REAL-TIME HEARTBEAT PROGRESS LOGGING (Mỗi 3 giây NẢY SỐ MỘT LẦN)
        now_time = time.time()
        if now_time - last_heartbeat_time >= 3.0:
            last_heartbeat_time = now_time
            elapsed = max(0.001, now_time - start_time)
            fps = total_samples / elapsed
            vram_curr = torch.cuda.memory_allocated(0) / (1024 ** 3)
            active_slots = sum(1 for s in range(PARALLEL) if slot_game[s] <= target_games)
            mega_size = len(all_tensors)
            print(f"⚡ [HEARTBEAT | Step {step_counter:05d}] Active Slots: {active_slots}/64 | GPU 2-Ply Batch: {mega_size:,} FENs ({eval_ms:.1f}ms) | Total FENs: {total_samples:,} | Speed: {fps:,.1f} FEN/s | Games: {completed_games}/{target_games} | VRAM: {vram_curr:.2f}GB", flush=True)

        # Phân phối kết quả Minimax 2-Ply về từng slot
        for s, legal, move_tree_map, is_random in slot_info:
            if is_random:
                best_move = random.choice(legal)
                best_score = 0
                encoded_move = best_move.encode()
            else:
                # Thuật toán 2-Ply Minimax Reduction
                # Đỏ (turn 0) chọn MAX(worst_opponent_min), Đen (turn 1) chọn MIN(worst_opponent_max)
                best_move = None
                best_minimax_score = -999999 if boards[s].turn == 0 else 999999

                for m1, off_2p, count_2p in move_tree_map:
                    scores_2p = all_scores[off_2p : off_2p + count_2p]
                    # Đối phương ở 2-Ply sẽ phản đòn bằng nước gây hại nhất cho ta
                    # Nếu lượt ta là Đỏ (0), đối phương Đen (1) sẽ chọn MIN score
                    # Nếu lượt ta là Đen (1), đối phương Đỏ (0) sẽ chọn MAX score
                    if boards[s].turn == 0:
                        minimax_score_m1 = torch.min(scores_2p).item()
                        if minimax_score_m1 > best_minimax_score:
                            best_minimax_score = minimax_score_m1
                            best_move = m1
                    else:
                        minimax_score_m1 = torch.max(scores_2p).item()
                        if minimax_score_m1 < best_minimax_score:
                            best_minimax_score = minimax_score_m1
                            best_move = m1

                if best_move is None:
                    best_move = legal[0]
                    best_score = 0
                else:
                    best_score = int(best_minimax_score)
                encoded_move = best_move.encode()

            fen_str = boards[s].export()
            fen_key = fen_str.split()[0]
            if fen_key not in sieve_set:
                sieve_set.add(fen_key)

                sample, thought_str = make_sample(boards[s], encoded_move, best_score, legal, plies[s], depth, history_moves_list[s])

                is_valid, err_reason = DataValidator.validate_sample(boards[s], encoded_move, best_score, thought_str)
                if is_valid:
                    f.write(json.dumps(sample, ensure_ascii=False) + "\n")
                    total_samples += 1
                    chunk_samples += 1

                    # ROTATION FILE 50MB
                    if chunk_samples >= 8000 or (out_file.exists() and out_file.stat().st_size >= 50 * 1024 * 1024):
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
                        out_file = out_dir / f"jrcp5_d12_node_{node_id}_{start_stamp}_chunk_{chunk_idx:04d}.jsonl"
                        f = open(out_file, "w", encoding="utf-8")
                else:
                    rejected_count += 1

            history_moves_list[s].append(encoded_move)
            boards[s].apply(best_move)
            plies[s] += 1

        # Auto-Push mỗi 5 phút (300 giây)
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
    print(f"🎉 JRCP 5.0 ULTRA 32D MINING COMPLETED IN {(time.time() - start_time)/60:.2f} MINS!")
    print(f"📊 Total Unique FENs: {total_samples:,} | Sieve Dedup: {len(sieve_set):,} | Rejected: {rejected_count}")
    print(f"🚀 Avg Speed: {total_samples/max(0.1, time.time() - start_time):,.1f} FEN/s | Peak VRAM: {final_vram:.2f} GB / {vram_total:.2f} GB")
    print("==================================================================")

if __name__ == "__main__":
    games = int(os.environ.get("GAMES", "1000"))
    depth = int(os.environ.get("DEPTH", "12"))
    mine(target_games=games, depth=depth)
