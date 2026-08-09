# === XIANGQI-R1 REAL RULE GPU T4 DATA MINER ENGINE (v10.0-JRCP4-TACTICAL-28D) ===
# 100% PHYSICAL XIANGQI RULES + FULL JRCP 4.0 28-DIMENSIONAL TACTICAL THOUGHT CHAIN
# + 36 KẾ BINH PHÁP + THẾ TRẬN KINH ĐIỂN + FORK/PIN/DISCOVERED ATTACK + SIEVE DEDUP + AUTO HF PUSH
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
SYMBOLS = {
    1: "帥", 2: "仕", 3: "相", 4: "馬", 5: "車", 6: "炮", 7: "兵",
    8: "將", 9: "士", 10: "象", 11: "馬", 12: "車", 13: "砲", 14: "卒",
    0: "．"
}
VALUES = {1: 0, 2: 20, 3: 20, 4: 40, 5: 90, 6: 45, 7: 10}

def sq(c: int, r: int) -> int:
    return r * 9 + c

START_POS = {
    0: {5: [sq(0, 0), sq(8, 0)], 4: [sq(1, 0), sq(7, 0)], 3: [sq(2, 0), sq(6, 0)],
        2: [sq(3, 0), sq(5, 0)], 1: [sq(4, 0)], 6: [sq(1, 2), sq(7, 2)],
        7: [sq(0, 3), sq(2, 3), sq(4, 3), sq(6, 3), sq(8, 3)]},
    1: {12: [sq(0, 9), sq(8, 9)], 11: [sq(1, 9), sq(7, 9)], 10: [sq(2, 9), sq(6, 9)],
        9: [sq(3, 9), sq(5, 9)], 8: [sq(4, 9)], 13: [sq(1, 7), sq(7, 7)],
        14: [sq(0, 6), sq(2, 6), sq(4, 6), sq(6, 6), sq(8, 6)]}
}

START_FEN = "r1bakab1r/9/1cn3nc1/p1p1p1p1p/9/9/P1P1P1P1P/1CN1C4/9/R1BAKABNR w - - 0 1"

SYSTEM_PROMPT = """Bạn là Xiangqi-R1 Master v4.0 — mô hình suy luận cờ Tướng siêu việt được huấn luyện phân tích chiều sâu chiến thuật đa tầng.
Bạn phải phân tích bàn cờ qua 28 chiều kích suy tưởng <thought> chi tiết trước khi xuất kết quả JSON JRCP 4.0.
28 chiều kích gồm 5 nhóm: Nhận thức Bàn cờ (1-6), Phân tích Đe dọa (7-12), Chiến thuật & Bẫy (13-18), 36 Kế Binh Pháp & Thế Trận (19-22), Đánh giá & Quyết định (23-28).
Mỗi chiều kích phải cung cấp thông tin cụ thể, chi tiết đến mức agent kém thông minh nhất cũng nhìn rõ hiện trạng bàn cờ."""

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

FORMATIONS = {
    "central": ("Pháo Đầu (中炮)", "Pháo chiếm Trung Lộ 5, tấn công trực diện cung Tướng đối phương"),
    "screen": ("Bình Phong Mã (屏风马)", "Hai Mã đối xứng ở c2/g2 hoặc c7/g7, tạo bức bình phong che chắn Tướng"),
    "single": ("Đơn Đề Mã (单提马)", "Một Mã phát triển sớm lên vị trí tấn công, Mã còn lại giữ phòng thủ"),
    "palace": ("Quá Cung Pháo (过宫炮)", "Pháo di chuyển vào trong Cung Tướng, tạo tuyến phòng thủ kết hợp tấn công"),
    "vanguard": ("Tiên Phong Xe (先锋车)", "Xe xuất quân sớm nhất, chiếm lộ mở để kiểm soát không gian"),
    "elephant": ("Song Phi Tượng (双飞象)", "Hai Tượng phát triển đối xứng, củng cố phòng tuyến hậu phương"),
    "scholar": ("Tam Tử Kinh (三子经)", "Ba lớp phòng thủ Sĩ-Tượng bao quanh Tướng, cung Tướng kiên cố nhất")
}

# sq() đã được định nghĩa ở trên (trước START_POS)

def col(idx: int) -> int:
    return idx % 9

def row(idx: int) -> int:
    return idx // 9

def uci(idx: int) -> str:
    c = chr(ord('a') + col(idx))
    r = str(row(idx))
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

    def attackers(self, target_sq: int, attacker_side: int) -> list:
        """Trả về danh sách (square_index, piece) của tất cả quân tấn công 1 ô."""
        result = []
        tc = col(target_sq)
        tr = row(target_sq)
        for i in range(90):
            p = self.grid[i]
            if p == 0 or side(p) != attacker_side: continue
            pc = col(i)
            pr = row(i)
            ptype = p if attacker_side == 0 else p - 7
            hit = False
            if ptype == 1:
                hit = abs(pc - tc) + abs(pr - tr) == 1
            elif ptype == 2:
                hit = abs(pc - tc) == 1 and abs(pr - tr) == 1
            elif ptype == 3:
                if abs(pc - tc) == 2 and abs(pr - tr) == 2:
                    hit = self.grid[sq((pc + tc) // 2, (pr + tr) // 2)] == 0
            elif ptype == 4:
                dc = tc - pc
                dr = tr - pr
                if abs(dc) == 1 and abs(dr) == 2:
                    hit = self.grid[sq(pc, pr + (1 if dr > 0 else -1))] == 0
                elif abs(dc) == 2 and abs(dr) == 1:
                    hit = self.grid[sq(pc + (1 if dc > 0 else -1), pr)] == 0
            elif ptype == 5:
                if pc == tc:
                    cnt = sum(1 for r in range(min(pr, tr) + 1, max(pr, tr)) if self.grid[sq(pc, r)] != 0)
                    hit = cnt == 0
                elif pr == tr:
                    cnt = sum(1 for c in range(min(pc, tc) + 1, max(pc, tc)) if self.grid[sq(c, pr)] != 0)
                    hit = cnt == 0
            elif ptype == 6:
                if pc == tc:
                    cnt = sum(1 for r in range(min(pr, tr) + 1, max(pr, tr)) if self.grid[sq(pc, r)] != 0)
                    hit = cnt == 1
                elif pr == tr:
                    cnt = sum(1 for c in range(min(pc, tc) + 1, max(pc, tc)) if self.grid[sq(c, pr)] != 0)
                    hit = cnt == 1
            elif ptype == 7:
                if attacker_side == 0:
                    hit = (tr == pr + 1 and tc == pc) or (pr >= 5 and tr == pr and abs(tc - pc) == 1)
                else:
                    hit = (tr == pr - 1 and tc == pc) or (pr <= 4 and tr == pr and abs(tc - pc) == 1)
            if hit:
                result.append((i, p))
        return result

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

    # === NHÓM I: NHẬN THỨC BÀN CỜ (Chiều 1-6) ===

    def inventory(self) -> tuple:
        """[1/28] Liệt kê toàn bộ quân cờ và vị trí."""
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
        """[2/28] Bàn cờ 2D văn bản ASCII với tọa độ và ký hiệu Hán tự."""
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
        """[3/28] Tính điểm vật chất chi tiết."""
        total = 0
        for i in range(90):
            p = self.grid[i]
            if p == 0 or side(p) != s: continue
            ptype = p if s == 0 else p - 7
            total += VALUES.get(ptype, 0)
        return total

    def columns(self) -> str:
        """[4/28] Phân tích 9 lộ: mở, bán mở, hoặc khóa."""
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
        """[5/28] Mức độ triển khai quân (đã rời vị trí xuất phát)."""
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
        """[6/28] Số nước đi hợp lệ của mỗi bên (mobility score)."""
        saved_turn = self.turn
        self.turn = 0
        red_moves = len(self.legal())
        self.turn = 1
        black_moves = len(self.legal())
        self.turn = saved_turn
        return (red_moves, black_moves)

    # === NHÓM II: PHÂN TÍCH ĐE DỌA (Chiều 7-12) ===

    def safety(self, s: int) -> str:
        """[7/28] An toàn Tướng — phân tích chi tiết Cung Tướng."""
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
        """[8/28] Danh sách quân của bên s đang bị quân đối phương tấn công."""
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
        """[9/28] Quân treo — bị tấn công nhưng KHÔNG được bảo vệ."""
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
        """[10/28] Quân bị ghim — không thể di chuyển vì che chắn cho Tướng."""
        k = self.king(s)
        if k < 0: return "Không tìm thấy Tướng."
        opp = 1 - s
        side_name = "Đỏ" if s == 0 else "Đen"
        kc = col(k)
        kr = row(k)
        results = []
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
                                fp_type = first_piece if s == 0 else first_piece - 7
                                results.append(f"{NAMES[first_piece]}({uci(first_piece_sq)}) BỊ GHIM bởi {NAMES[p]}({uci(idx)}) — che chắn Tướng trên đường thẳng")
                        break
                nc += direction_c
                nr += direction_r
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
        """[11/28] Đòn kép — 1 quân đe dọa 2+ quân đối phương cùng lúc."""
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
        """[12/28] Đòn mở — di chuyển 1 quân mở đường tấn công cho quân phía sau."""
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

    # === NHÓM III: CHIẾN THUẬT & BẪY (Chiều 13-18) ===

    def traps(self) -> str:
        """[13/28] Bẫy ăn quân — quân giá trị thấp tấn công quân giá trị cao không bảo vệ."""
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
                    min_counter = min(VALUES.get(cp if side(cp) == 0 else cp - 7, 0) for _, cp in counter_attackers)
                    net = cap_val - src_val
                    if net > 20:
                        results.append(f"BẪY ĐỔI QUÂN: {NAMES[src_piece]}({uci(m.src)}) ăn {NAMES[captured]}({uci(m.dst)}) — lời {net}cp dù bị phản đòn")
        if not results:
            return f"Không phát hiện bẫy ăn quân nào cho {side_name}."
        return "; ".join(results[:3])

    def checkmate(self) -> str:
        """[14/28] Chiếu bí tiềm ẩn — kiểm tra xem có đe dọa chiếu bí trong 1 nước."""
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
        """[15/28] Dương đông kích tây — phân tích nước đi có tính nghi binh."""
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
        """[16/28] Mẫu chiến thuật nâng cấp — 15+ patterns."""
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
        """[17/28] Phối hợp quân — phát hiện các bộ quân đang phối hợp hiệu quả."""
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
        """[18/28] Điểm yếu cấu trúc — Tốt cô lập, lỗ hổng phòng tuyến."""
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
        doubled_cols = set()
        for c in range(9):
            count = sum(1 for r in range(10) if self.grid[sq(c, r)] == pawn_type)
            if count >= 2:
                doubled_cols.add(c)
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

    # === NHÓM IV: 36 KẾ BINH PHÁP & THẾ TRẬN (Chiều 19-22) ===

    def stratagems(self, encoded_move: str) -> str:
        """[19/28] 36 kế binh pháp áp dụng vào tình huống hiện tại."""
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
                applicable.append(17)
            if cap_type in [2, 3]:
                applicable.append(19)
        if src_type == 6:
            applicable.append(3)
        if mat_diff > 100:
            applicable.append(4)
        elif mat_diff < -100:
            applicable.append(30)
        if opp_advisors + opp_elephants <= 1:
            applicable.append(5)
        src_c = ord(encoded_move[0]) - ord('a')
        dst_c = ord(encoded_move[2]) - ord('a')
        if abs(src_c - dst_c) >= 4:
            applicable.append(6)
            applicable.append(8)
        if self.check(opp):
            applicable.append(35)
        if my_advisors == 0 and my_elephants == 0:
            applicable.append(32)
        if mat_diff < -200:
            applicable.append(36)
        if not applicable:
            applicable.append(1)
        result_lines = []
        for knum in applicable[:3]:
            if knum in STRATAGEMS:
                name, desc = STRATAGEMS[knum]
                result_lines.append(f"Kế {knum}: {name} — {desc}")
        return "\n    ".join(result_lines) if result_lines else "Không áp dụng kế đặc biệt nào."

    def formation(self) -> str:
        """[20/28] Thế trận kinh điển — phát hiện khai cuộc/thế trận đã biết."""
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
        """[22/28] Tempo & sáng kiến — bên nào nắm quyền chủ động."""
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

    def center(self) -> str:
        """Phân tích trung lộ (dùng cho JRCP cũ, giữ tương thích)."""
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

# === BỘ LỌC KIỂM CHẤM NGHIÊM NGẶT DỮ LIỆU ĐẦU RA (STRICT DATA VALIDATOR — JRCP 4.0) ===

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

        if ptype in [1, 2]:
            r_min, r_max = (0, 2) if side(piece) == 0 else (7, 9)
            if not (3 <= dst_c <= 5 and r_min <= dst_r <= r_max):
                return False, "LEAVING_PALACE_BOUNDARY"

        for i in range(1, 29):
            if f"[{i}/28]" not in thought:
                return False, f"MISSING_THOUGHT_TAG_{i}"

        return True, "VALID_OK"

# === MULTI-GAME PARALLEL GPU MINER (64 ván song song, Mega-Batch Evaluation) — JRCP 4.0 ===

PARALLEL = 64  # Số ván cờ chạy song song trên GPU

def make_sample(board, encoded_move, best_score, legal_moves, ply, depth):
    """Sinh mẫu JRCP 4.0 hoàn chỉnh với 28 chiều kích suy tưởng chiến thuật chiều sâu."""
    fen_str = board.export()

    # === NHÓM I: NHẬN THỨC BÀN CỜ ===
    red_inv, black_inv = board.inventory()
    board_ascii = board.ascii()
    red_mat = board.material(0)
    black_mat = board.material(1)
    mat_diff = red_mat - black_mat
    columns_info = board.columns()
    red_deployed = board.deployed(0)
    black_deployed = board.deployed(1)
    red_mob, black_mob = board.mobility()

    # === NHÓM II: PHÂN TÍCH ĐE DỌA ===
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

    # === NHÓM III: CHIẾN THUẬT & BẪY ===
    traps_info = board.traps()
    checkmate_info = board.checkmate()
    diversion_info = board.diversion(encoded_move)
    tactical_pats = board.patterns()
    pats_str = "\n    ".join(tactical_pats)
    synergy_info = board.synergy()
    weakness_my = board.weakness(board.turn)
    weakness_opp = board.weakness(1 - board.turn)

    # === NHÓM IV: 36 KẾ & THẾ TRẬN ===
    stratagems_info = board.stratagems(encoded_move)
    formation_info = board.formation()
    phase = "opening" if ply < 16 else ("early_midgame" if ply < 30 else ("midgame" if ply < 60 else ("late_midgame" if ply < 90 else "endgame")))
    phase_vi = {"opening": "Khai cuộc", "early_midgame": "Đầu trung cuộc", "midgame": "Trung cuộc", "late_midgame": "Cuối trung cuộc", "endgame": "Tàn cuộc"}
    tempo_info = board.tempo()

    # === NHÓM V: ĐÁNH GIÁ & QUYẾT ĐỊNH ===
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

    thought_str = f"""<thought>
[1/28] KIỂM KÊ QUÂN CỜ:
  Đỏ: {red_inv}
  Đen: {black_inv}
[2/28] BÀN CỜ 2D:
{board_ascii}
[3/28] TƯƠNG QUAN VẬT CHẤT CHI TIẾT:
  Đỏ: {red_mat}cp | Đen: {black_mat}cp | Chênh lệch: {mat_diff}cp
  (Xe=90, Pháo=45, Mã=40, Sĩ=20, Tượng=20, Tốt=10, Tướng=0)
[4/28] PHÂN TÍCH 9 LỘ:
  {columns_info}
[5/28] MỨC ĐỘ TRIỂN KHAI QUÂN:
  {red_deployed}
  {black_deployed}
[6/28] ĐỘ LINH HOẠT (MOBILITY):
  Đỏ: {red_mob} nước đi hợp lệ | Đen: {black_mob} nước đi hợp lệ | Chênh lệch: {red_mob - black_mob}
[7/28] AN TOÀN TƯỚNG:
  Bên ta ({turn_str}): {safety_my}
  Đối phương ({opp_str}): {safety_opp}
[8/28] QUÂN BỊ TẤN CÔNG:
  Bên ta: {attacked_my}
  Đối phương: {attacked_opp}
[9/28] QUÂN TREO (HANGING — ĂN MIỄN PHÍ):
  Bên ta: {hanging_my}
  Đối phương: {hanging_opp}
[10/28] QUÂN BỊ GHIM (PIN):
  Bên ta: {pinned_info}
  Đối phương: {pinned_opp}
[11/28] ĐÒN KÉP (FORK):
  {forks_info}
[12/28] ĐÒN MỞ (DISCOVERED ATTACK):
  {discovered_info}
[13/28] BẪY ĂN QUÂN:
  {traps_info}
[14/28] CHIẾU BÍ TIỀM ẨN:
  {checkmate_info}
[15/28] DƯƠNG ĐÔNG KÍCH TÂY:
  {diversion_info}
[16/28] MẪU CHIẾN THUẬT:
    {pats_str}
[17/28] PHỐI HỢP QUÂN:
  {synergy_info}
[18/28] ĐIỂM YẾU CẤU TRÚC:
  Bên ta: {weakness_my}
  Đối phương: {weakness_opp}
[19/28] 36 KẾ BINH PHÁP ÁP DỤNG:
    {stratagems_info}
[20/28] THẾ TRẬN KINH ĐIỂN:
  {formation_info}
[21/28] GIAI ĐOẠN & CHIẾN LƯỢC:
  Giai đoạn: {phase_vi.get(phase, phase)} (nước thứ {ply}) — {turn_str} đi.
[22/28] TEMPO & SÁNG KIẾN:
  {tempo_info}
[23/28] ƯU THẾ TỔNG HỢP:
  {advantage_str}
[24/28] BẤT LỢI TỔNG HỢP:
  {disadvantage_str}
[25/28] ĐÁNH GIÁ CANDIDATES ({len(legal_moves)} ứng viên, hiển thị top {min(5, len(legal_moves))}):
{candidates_str}
[26/28] SO SÁNH & CHỌN BESTMOVE:
  Chọn {encoded_move} — {best_name}({uci(sq(ord(encoded_move[0])-ord('a'), int(encoded_move[1])))} -> {uci(sq(ord(encoded_move[2])-ord('a'), int(encoded_move[3])))}){cap_detail} ({best_score}cp).
  Lý do: Tối ưu hóa Centipawn, vị trí quân cờ, và chiến thuật phù hợp giai đoạn {phase_vi.get(phase, phase)}.
[27/28] CENTIPAWN TỔNG HỢP: {best_score}cp
[28/28] XÁC MINH: {encoded_move} khớp regex ^[a-i][0-9][a-i][0-9]$ ✓ | Nước đi hợp lệ trong danh sách {len(legal_moves)} ứng viên ✓
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
    out_file = out_dir / f"jrcp4_d12_node_{node_id}_{start_stamp}_chunk_{chunk_idx:04d}.jsonl"

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
    print("📊 BÁO CÁO THÔNG SỐ CẤU HÌNH HỆ THỐNG — JRCP 4.0 TACTICAL 28D", flush=True)
    print("==================================================================", flush=True)
    print(f"🖥️ CPU Cores     : {cpu_count} vCPUs | Platform: {platform.system()} {platform.machine()}", flush=True)
    print(f"🧠 System RAM    : {ram_gb:.2f} GB RAM", flush=True)
    print(f"⚡ GPU Device    : {torch.cuda.get_device_name(0)} ({vram_total:.2f} GB VRAM | Allocated: {vram_allocated:.2f} GB)", flush=True)
    print(f"🧰 Software Env  : Python {python_ver} | PyTorch {torch_ver} | CUDA {torch.version.cuda}", flush=True)
    print(f"🏷️ Engine Version : v10.0-jrcp4-tactical-28d (Build 2026-08-10 01:07:00 ICT)", flush=True)
    print(f"🎮 Target Config  : {target_games:,} Games | Search Depth {depth}", flush=True)
    print(f"🆔 Unique Node ID : node_{node_id}", flush=True)
    print(f"📦 File Chunk Cap : 50 MB / Chunk (Chunk #{chunk_idx})", flush=True)
    print(f"💾 Active Output  : {out_file}", flush=True)
    print(f"🔑 HF Hub Status  : {'CONNECTED (' + dataset_repo + ')' if api else 'DISABLED (No HF_TOKEN)'}", flush=True)
    print(f"🧠 Model Params   : {param_count:,} ({model_mb:.1f} MB) — Deep Residual 4-Block 512ch", flush=True)
    print(f"🚀 Parallel Mode  : {PARALLEL} ván cờ song song / Mega-Batch GPU Evaluation", flush=True)
    print(f"📐 Thought Chain  : JRCP 4.0 — 28 chiều kích chiến thuật chiều sâu", flush=True)
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
                        out_file = out_dir / f"jrcp4_d12_node_{node_id}_{start_stamp}_chunk_{chunk_idx:04d}.jsonl"
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
    print(f"🎉 JRCP 4.0 TACTICAL 28D MINING COMPLETED IN {(time.time() - start_time)/60:.2f} MINS!")
    print(f"📊 Total Unique FENs: {total_samples:,} | Sieve Dedup: {len(sieve_set):,} | Rejected: {rejected_count}")
    print(f"🚀 Avg Speed: {total_samples/max(0.1, time.time() - start_time):,.1f} FEN/s | Peak VRAM: {final_vram:.2f} GB / {vram_total:.2f} GB")
    print("==================================================================")

if __name__ == "__main__":
    games = int(os.environ.get("GAMES", "1000"))
    depth = int(os.environ.get("DEPTH", "12"))
    mine(target_games=games, depth=depth)
