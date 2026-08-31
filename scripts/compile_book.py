#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
===============================================================================
KỊCH BẢN BIÊN DỊCH SÁCH KHAI CUỘC ĐỘNG CHUẨN XRBK v1 (XIANGQI-RIM BOOK COMPILER)
===============================================================================
Kịch bản này chuyển đổi các tệp JSONL thế cờ Grandmaster thành tệp nhị phân
chuẩn `XRBK v1` siêu nhẹ (16 bytes/record) cho Xiangqi-RIM Engine nạp tức thì < 5ms.

Cú pháp:
    python3 scripts/compile_book.py --input data/pikafish_in_memory_distill.jsonl --output data/master_book.xrbk --limit 50000

Quy tắc:
    - 100% chú thích tiếng Việt từng dòng.
    - 100% định danh biến tiếng Anh từ đơn.
===============================================================================
"""

import sys
import os
import json
import struct
import argparse
from typing import List, Tuple, Dict

# Ma trận khóa Zobrist Hash 64-bit chuẩn tương thích Xiangqi-RIM Rust Engine
# 14 loại quân (0..13) x 90 ô cờ (0..89)
PIECES = 14
SQUARES = 90

def build_keys() -> List[int]:
    """Sinh mảng khóa Zobrist Hash giả định phân bố ngẫu nhiên đồng đều theo LCG."""
    keys = []
    seed = 0x9E3779B97F4A7C15
    for i in range(PIECES * SQUARES):
        seed = (seed ^ (seed >> 30)) * 0xBF58476D1CE4E5B9 & 0xFFFFFFFFFFFFFFFF
        seed = (seed ^ (seed >> 27)) * 0x94D049BB133111EB & 0xFFFFFFFFFFFFFFFF
        seed = seed ^ (seed >> 31) & 0xFFFFFFFFFFFFFFFF
        keys.append(seed)
    return keys

ZOBRIST_TABLE = build_keys()

# Bảng ánh xạ ký tự FEN sang chỉ số quân cờ (0..13)
CHAR_TO_PIECE = {
    'R': 0, 'N': 1, 'M': 1, 'B': 2, 'E': 2, 'A': 3, 'K': 4, 'C': 5, 'P': 6,
    'r': 7, 'n': 8, 'm': 8, 'b': 9, 'e': 9, 'a': 10, 'k': 11, 'c': 12, 'p': 13
}

def parse_fen_to_grid(fen: str) -> Tuple[List[int], int]:
    """Chuyển đổi chuỗi FEN sang mảng 90 ô cờ (0..89) và lượt đi side (0: Red, 1: Black)."""
    parts = fen.strip().split()
    board_part = parts[0]
    side = 0 if len(parts) < 2 or parts[1] == 'w' or parts[1] == 'r' else 1
    
    grid = [14] * 90  # 14 là ô trống (EMPTY)
    ranks = board_part.split('/')
    
    for row_idx, rank in enumerate(ranks):
        col_idx = 0
        for char in rank:
            if char.isdigit():
                col_idx += int(char)
            elif char in CHAR_TO_PIECE:
                # Hệ tọa độ: row 0 (Đỏ đáy), row 9 (Đen đáy)
                # FEN rank 0 là hàng trên cùng của Đen (row 9), rank 9 là hàng dưới cùng của Đỏ (row 0)
                actual_row = 9 - row_idx
                sq = actual_row * 9 + col_idx
                grid[sq] = CHAR_TO_PIECE[char]
                col_idx += 1
                
    return grid, side

def calculate_hash(grid: List[int], side: int) -> int:
    """Tính toán Zobrist Hash 64-bit của vị trí bàn cờ."""
    h = 0
    for sq, piece in enumerate(grid):
        if piece < 14:
            idx = piece * SQUARES + sq
            h ^= ZOBRIST_TABLE[idx]
    if side == 1:
        h ^= 0xF00DCAFE12345678  # Turn key
    return h & 0xFFFFFFFFFFFFFFFF

def uci_to_move(move_str: str) -> int:
    """Chuyển đổi nước đi UCI (vd: 'b2e2', 'h9g7') sang số nguyên 16-bit (from << 8 | to)."""
    if len(move_str) != 4:
        return 0
    
    col_from = ord(move_str[0]) - ord('a')
    row_from = int(move_str[1])
    col_to = ord(move_str[2]) - ord('a')
    row_to = int(move_str[3])
    
    from_sq = row_from * 9 + col_from
    to_sq = row_to * 9 + col_to
    
    return ((from_sq & 0xFF) << 8) | (to_sq & 0xFF)

def compile_book(input_path: str, output_path: str, limit: int = 200000, min_pieces: int = 24):
    """Đọc tệp JSONL, trích xuất FEN và nước đi, sau đó xuất ra tệp nhị phân XRBK v1."""
    print(f"📖 Đang đọc dữ liệu từ: {input_path}")
    
    if not os.path.exists(input_path):
        print(f"❌ Lỗi: Tệp nguồn {input_path} không tồn tại!")
        return
        
    entries: Dict[int, Tuple[int, int, int]] = {}  # hash -> (mv, weight, score)
    count = 0
    
    with open(input_path, 'r', encoding='utf-8') as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
                
            try:
                data = json.loads(line)
                fen = data.get("fen", "")
                best_move = data.get("best_move", "")
                score = int(data.get("score", 0))
                
                if not fen or not best_move:
                    continue
                    
                grid, side = parse_fen_to_grid(fen)
                # Đếm số quân trên bàn cờ để lọc lấy giai đoạn Khai Cuộc và Tiền Trung Cuộc (>= min_pieces quân)
                piece_count = sum(1 for p in grid if p < 14)
                if piece_count < min_pieces:
                    continue
                    
                h = calculate_hash(grid, side)
                mv = uci_to_move(best_move)
                
                if mv == 0:
                    continue
                    
                if h in entries:
                    old_mv, old_weight, old_score = entries[h]
                    if mv == old_mv:
                        entries[h] = (mv, min(65535, old_weight + 10), score)
                    elif (side == 0 and score > old_score) or (side == 1 and score > old_score):
                        entries[h] = (mv, 100, score)
                else:
                    entries[h] = (mv, 100, score)
                    
                count += 1
                if len(entries) >= limit:
                    break
                    
                if count % 100000 == 0:
                    print(f"  -> Đã quét {count} mẫu, thu thập được {len(entries)} vị trí khai cuộc duy nhất...")
            except Exception as e:
                continue
                
    print(f"✅ Thu thập hoàn tất: {len(entries)} bản ghi khai cuộc chất lượng cao!")
    
    # Sắp xếp các bản ghi tăng dần theo Zobrist Hash để tối ưu hóa Binary Search O(log N)
    sorted_hashes = sorted(entries.keys())
    
    print(f"💾 Đang ghi tệp nhị phân XRBK v1: {output_path}")
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    
    with open(output_path, 'wb') as f:
        # Header (16 bytes): Magic(4B) + Version(4B) + Count(4B) + Flags(4B)
        magic = b"XRBK"
        version = 1
        entry_count = len(sorted_hashes)
        flags = 0
        
        f.write(struct.pack("<4sIII", magic, version, entry_count, flags))
        
        # Records (16 bytes each): hash(8B) + mv(2B) + weight(2B) + score(2B) + pad(2B)
        for h in sorted_hashes:
            mv, weight, score = entries[h]
            f.write(struct.pack("<QHHh2s", h, mv, weight, score, b"\x00\x00"))
            
    file_size = os.path.getsize(output_path)
    print(f"🎉 Xuất bản thành công tệp {output_path} ({file_size / 1024:.2f} KB, {len(sorted_hashes)} bản ghi)!")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Xiangqi-RIM Grandmaster Opening Book Compiler")
    parser.add_argument("--input", default="data/pikafish_in_memory_distill.jsonl", help="Đường dẫn tệp JSONL nguồn")
    parser.add_argument("--output", default="data/master_book.xrbk", help="Đường dẫn tệp nhị phân XRBK đầu ra")
    parser.add_argument("--limit", type=int, default=200000, help="Số lượng thế cờ khai cuộc tối đa")
    parser.add_argument("--min-pieces", type=int, default=24, help="Số quân cờ tối thiểu trên bàn cờ")
    
    args = parser.parse_args()
    compile_book(args.input, args.output, args.limit, args.min_pieces)
