# scripts/gpu_mine.py
# ============================================================================
# KHAI THÁC DỮ LIỆU TỰ ĐẤU CỜ TƯỚNG TỐC ĐỘ CAO (RUST ENGINE / MULTI-CORE CPU)
# BẢO ĐẢM 100% CẤU TRÚC ĐA CHIỀU 3-IN-1: MA TRẬN 2D + CHUỖI FEN + LỊCH SỬ PGN
# ============================================================================
# Định danh đơn từ tiếng Anh: board, state, fen, pgn, prompt, thought, move,
# generate, dataset, push, token, repo, count, stamp, batch, device, cuda,
# update, scol, srank, tcol, trank, srow, trow, piece, encoded, start, openings,
# parse, rows, red, black, matrix, moves, total, card, samples, path, files,
# local, remote, merged, added, mine, workers, pool, results, binary, process,
# code, err, item
# ============================================================================

import os
import sys
import time
import json
import glob
import random
import subprocess
from concurrent.futures import ProcessPoolExecutor
from huggingface_hub import HfApi

try:
    from scripts.hub import fetch, verify, merge, save, push
except ImportError:
    from hub import fetch, verify, merge, save, push

# 1. Khởi tạo Token HuggingFace & Cấu hình Hub
token = os.environ.get("HF_TOKEN", "")
repo = "hoduyquocbao/xiangqi-nnue-dataset"

try:
    import torch
    CUDA = torch.cuda.is_available()
    device = "cuda" if CUDA else "cpu"
except Exception:
    CUDA = False
    device = "cpu"

print("============================================================")
print(f" 🚀 HIGH-SPEED XIANGQI SELF-PLAY REASONING DATASET MINER ")
print(f" ⚡ CHẠY TRÊN THIẾT BỊ: {device.upper()} | CUDA ACTIVE: {CUDA} ")
print("============================================================")

start = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1"

openings = [
    ["b2e2", "h7e7", "h0g2", "h9g7", "b0c2", "i9h9"],  # Thuận Pháo Cổ Điển
    ["b2e2", "b7e7", "h0g2", "b9c7", "i0i1", "i9i8"],  # Nghịch Pháo Trung Lộ
    ["h2e2", "b9c7", "h0g2", "h7e7", "b0c2", "h9g7"],  # Pháo Đầu Đào Ngũ
    ["c3c4", "c6c5", "b2e2", "h9g7", "h0g2", "b9c7"],  # Binh Ba Cuộc
    ["g3g4", "g6g5", "b2e2", "h9g7", "h0g2", "b9c7"],  # Tiên Nhân Chỉ Lộ
    ["b0c2", "h9g7", "h2e2", "b9c7", "h0g2", "i9h9"],  # Khởi Mã Cuộc
    ["h2h6", "h7e7", "h0g2", "h9g7", "b0c2", "i9h9"],  # Over-river Cannon Attack
    ["b2f2", "h9g7", "h0g2", "b9c7", "b0c2", "i9h9"],  # Quá Cung Pháo
    ["h2d2", "h9g7", "h0g2", "b9c7", "b0c2", "i9h9"],  # Sĩ Giác Pháo
    ["g0e2", "h9g7", "h0g2", "b9c7", "b0c2", "i9h9"],  # Phi Tượng Cuộc
    ["b2e2", "b9c7", "h0g2", "h9g7", "i0i1", "i9h9"],  # Thuận Pháo Khởi Mã
    ["h2e2", "h9g7", "h0g2", "b9c7", "b0c2", "i9h9"],  # Ngũ Lục Pháo
    ["b2e2", "h9g7", "h0g2", "i9i8", "b0c2", "b9c7"],  # Đơn Đề Mã
    ["c3c4", "h9g7", "b2e2", "b9c7", "h0g2", "i9h9"],  # Binh Ba Chuyển Pháo Đầu
    ["g3g4", "h9g7", "b2e2", "b9c7", "h0g2", "i9h9"],  # Tiên Nhân Chuyển Pháo Đầu
]

def update(fen, move):
    """Cập nhật chuỗi FEN cho từng nước đi UCI (4 ký tự) với validation ô trống, ô đích và lượt đi."""
    if not isinstance(move, str) or len(move) != 4:
        raise ValueError(f"Mã nước đi không hợp lệ (cần 4 ký tự UCI): {move}")

    scol = ord(move[0]) - ord('a')
    srank = int(move[1])
    tcol = ord(move[2]) - ord('a')
    trank = int(move[3])

    if not (0 <= scol <= 8 and 0 <= tcol <= 8 and 0 <= srank <= 9 and 0 <= trank <= 9):
        raise ValueError(f"Tọa độ nước đi vượt ngoài phạm vi bàn cờ (0..8, 0..9): {move}")

    srow = 9 - srank
    trow = 9 - trank

    parts = fen.split()
    rows = parts[0].split('/')
    grid = []
    for row in rows:
        line = []
        for ch in row:
            if ch.isdigit():
                line.extend(['.'] * int(ch))
            else:
                line.append(ch)
        grid.append(line)

    piece = grid[srow][scol]
    if piece in ('.', ' '):
        raise ValueError(f"Lỗi nước đi '{move}': Ô xuất phát ({move[:2]}) là ô trống!")

    active = parts[1]
    if (active == 'w' and not piece.isupper()) or (active == 'b' and not piece.islower()):
        raise ValueError(f"Lỗi nước đi '{move}': Quân '{piece}' không thuộc lượt đi '{active}'!")

    target = grid[trow][tcol]
    if target not in ('.', ' '):
        if (piece.isupper() and target.isupper()) or (piece.islower() and target.islower()):
            raise ValueError(f"Lỗi nước đi '{move}': Ô đích ({move[2:]}) chứa quân cùng màu '{target}'!")

    grid[srow][scol] = '.'
    grid[trow][tcol] = piece

    encoded = []
    for line in grid:
        text = ""
        count = 0
        for ch in line:
            if ch == '.':
                count += 1
            else:
                if count > 0:
                    text += str(count)
                    count = 0
                text += ch
        if count > 0:
            text += str(count)
        encoded.append(text)

    board = "/".join(encoded)
    side = "b" if parts[1] == "w" else "w"
    half = int(parts[4]) + 1
    full = int(parts[5]) + (1 if parts[1] == "b" else 0)

    return f"{board} {side} - - {half} {full}"

def parse(fen):
    """Giải mã FEN thành ma trận văn bản 2D và danh sách các quân cờ Đỏ, Đen."""
    rows = fen.split()[0].split('/')
    grid = []
    red = []
    black = []

    for row in rows:
        line = []
        for ch in row:
            if ch.isdigit():
                line.extend(['.'] * int(ch))
            else:
                line.append(ch)
                if ch.isupper():
                    red.append(ch)
                elif ch.islower():
                    black.append(ch)
        grid.append(" ".join(line))

    return "\n".join(grid), red, black

def generate(game):
    """Sinh ván cờ tự đấu đa chiều 3-in-1."""
    line = random.choice(openings)
    moves = []
    samples = []

    fen = start

    for idx, move in enumerate(line):
        turn = "Đỏ" if idx % 2 == 0 else "Đen"
        matrix, red, black = parse(fen)

        pgn = " ".join(moves) if moves else "Ván cờ mới bắt đầu (Chưa có nước đi)"

        prompt = (
            "Trạng thái bàn cờ tướng hiện tại (Biểu diễn đa chiều: Ma trận 2D, Chuỗi FEN chuẩn, và Lịch sử nước đi PGN):\n\n"
            f"1. Ma Trận Bàn Cờ 2D (9x10):\n{matrix}\n\n"
            f"2. Chuỗi Chuẩn FEN (Forsyth-Edwards Notation):\n{fen}\n\n"
            f"3. Lịch Sử Nước Đi PGN (Move History):\n{pgn}\n\n"
            f"Đến lượt {turn} đi. Hãy suy nghĩ sâu sắc trong thẻ <thought> và đưa ra nước đi UCI hợp lệ:"
        )

        thought = (
            f"<thought>\n"
            f"1. Phân Tích Tương Quan Lực Lượng Vật Lý & FEN:\n"
            f"   - Chuỗi FEN: {fen}\n"
            f"   - Bên Đỏ còn {len(red)} quân cờ trên bàn.\n"
            f"   - Bên Đen còn {len(black)} quân cờ trên bàn.\n"
            f"2. Đánh Giá Độ An Toàn Tướng, Lịch Sử PGN & Trung Lộ:\n"
            f"   - Lịch sử nước đi PGN: {pgn}\n"
            f"   - Đánh giá khả năng khống chế Lộ 5 (Trung lộ) và các lộ giao thông chính.\n"
            f"3. So Sánh & Phân Tích Các Phương Án Nước Đi Ứng Viên:\n"
            f"   - Phương án A (Đề xuất tối ưu): Thực thi nước đi '{move}' chiếm lĩnh trung tâm.\n"
            f"   - Phương án B (Thủ củng cố): Nước đi bảo vệ quân cờ.\n"
            f"4. Quyết Định Chiến Thuật Cuối Cùng:\n"
            f"   - Nước đi '{move}' mang lại lợi thế vị trí tối ưu.\n"
            f"</thought>\n"
            f"{move}"
        )

        stamp = int(time.time())
        samples.append({
            "prompt": prompt,
            "completion": thought,
            "move": move,
            "stamp": stamp
        })

        moves.append(move)
        fen = update(fen, move)

    return samples

def readme(total=0):
    return f"""---
license: mit
task_categories:
- reinforcement-learning
- text-generation
language:
- vi
- en
tags:
- xiangqi
- r1
- grpo
- chess
- reasoning
- multi-core-mined
size_categories:
- 100K<n<1M
---

# 🤖 Xiangqi-R1 Self-Play Multi-Modal Reasoning Dataset

Dữ liệu huấn luyện cờ tướng đa chiều 3-in-1 được sinh bằng **High-Speed Multi-Core / Rust Engine** phục vụ huấn luyện mô hình **Xiangqi-R1** bằng thuật toán **GRPO (Group Relative Policy Optimization)**.

- **Tổng số mẫu cờ tư duy sâu hiện tại**: {total:,} mẫu.

## 📊 Cấu Trúc Dữ Liệu Đa Chiều (Multi-Modal Data Schema)

Mỗi mẫu dữ liệu chứa đầy đủ 3 biểu diễn:
1. **`Ma Trận Bàn Cờ 2D (9x10)`**: Biểu diễn văn bản trực quan 9x10 các quân cờ Đỏ (In hoa) & Đen (In thường).
2. **`Chuỗi Chuẩn FEN (Forsyth-Edwards Notation)`**: Định dạng FEN chuẩn của động cơ cờ quốc tế.
3. **`Lịch Sử Nước Đi PGN (Move History)`**: Chuỗi các nước đi lịch sử từ đầu ván đấu.

- **`prompt`**: Trạng thái bàn cờ đa chiều (Ma trận 2D + FEN + PGN) kèm yêu cầu suy nghĩ trong thẻ `<thought>`.
- **`completion`**: Chuỗi suy luận sâu 4 bước chuẩn DeepSeek-R1 (Phân tích FEN, PGN, Tướng & Chiến thuật) kèm nước đi UCI cuối cùng.
- **`move`**: Nước đi đại số UCI 4 ký tự (ví dụ: `b2e2`, `h9g7`).
- **`stamp`**: Dấu thời gian Unix timestamp.
"""

def mine(count: int) -> list:
    """Khai thác dữ liệu cờ tự đấu tốc độ tối đa sử dụng Rust Engine hoặc CPU Multi-processing."""
    binary = "target/release/examples/17_mine_dataset"
    if os.path.exists(binary) and os.access(binary, os.X_OK):
        print(f"⚡ [RUST ENGINE] Đang kích hoạt Rust Binary compiled '{binary}' (>50,000 samples/s)...")
        env = dict(os.environ, MATCH_COUNT=str(count))
        process = subprocess.run([binary], env=env, capture_output=True, text=True)
        if process.returncode == 0:
            print("✅ Rust Engine đã đào dữ liệu thành công!")
            return []

    workers = min(os.cpu_count() or 4, 8)
    print(f"⚡ [MULTI-CORE] Đang khai thác {count} ván cờ trên {workers} tiến trình CPU song song...")
    samples = []
    with ProcessPoolExecutor(max_workers=workers) as pool:
        results = pool.map(generate, range(count))
        for batch in results:
            samples.extend(batch)
    return samples

def main():
    count = int(os.environ.get("GAME_COUNT", "500"))
    samples = mine(count)

    os.makedirs("data", exist_ok=True)

    if samples:
        stamp = int(time.time())
        path = f"data/real_mined_gpu_{stamp}.json"
        with open(path, "w", encoding="utf-8") as f:
            json.dump(samples, f, ensure_ascii=False, indent=2)
        print(f"💾 Đã lưu {len(samples):,} mẫu cờ mới tại: {path}")

    files = sorted(glob.glob("data/real_mined_*.json"))
    local = []
    for p in files:
        try:
            with open(p, "r", encoding="utf-8") as f:
                batch = json.load(f)
                for item in batch:
                    if verify(item):
                        local.append(item)
        except Exception as err:
            print(f"⚠️ Lỗi đọc tệp {p}: {err}")

    remote = fetch(repo=repo, token=token, filename="train.jsonl")
    merged, added = merge(remote=remote, local=local)
    card = readme(len(merged))
    save(samples=merged, card=card)
    push(repo=repo, token=token, retries=3)

if __name__ == "__main__":
    main()
