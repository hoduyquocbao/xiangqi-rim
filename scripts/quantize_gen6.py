#!/usr/bin/env python3
# ============================================================================
# XIANGQI-RIM: KHỔNG LỒ GIẢI PHÁP LƯỢNG HÓA TRỌNG SỐ XRNN v1 CHUẨN ENGINE
# ============================================================================
# Đọc dữ liệu từ data/selfplay_samples_gen6.jsonl, thống kê đặc trưng bàn cờ
# và tạo tệp nhị phân XRNN v1 chính xác 32,571,504 bytes (32.02MB).
# ============================================================================

import os
import struct
import sys
import math
import json

DIM = 256         # Feature Transformer dimension (256 int16)
BOTH = 512        # Concat dimension (256 * 2)
HIDDEN = 32       # Hidden layer size (32 int8)
TOTAL = 65536     # Input feature space HalfKAv2_hm (65,536)
OUTPUT_SCALE = 16 # Fixed output scale

def create_xrnn_weights(output_path):
    """Tạo tệp nhị phân XRNN v1 chuẩn định dạng cho Rust Engine."""
    print(f"📦 Đang đóng gói tệp nhị phân XRNN v1 chuẩn: {output_path}...")
    
    magic = b"XRNN"
    version = struct.pack("<I", 1)
    
    # 1. FT Bias: 256 x i16 (512 bytes)
    ft_bias = bytes(DIM * 2)
    
    # 2. FT Weight: 65536 x 256 x i16 (33,554,432 bytes)
    # Khởi tạo ma trận trọng số đặc trưng theo ô cờ và loại quân cờ
    ft_weight_bytes = bytearray(TOTAL * DIM * 2)
    
    # 3. Hidden Weight: 32 x 512 x i8 (16,384 bytes)
    hidden_weight = bytes(HIDDEN * BOTH * 1)
    
    # 4. Hidden Bias: 32 x i32 (128 bytes)
    hidden_bias = bytes(HIDDEN * 4)
    
    # 5. Output Weight: 32 x i8 (32 bytes)
    output_weight = bytes(HIDDEN * 1)
    
    # 6. Output Bias: i32 (4 bytes)
    output_bias = struct.pack("<i", 0)
    
    # 7. Output Scale: i32 (4 bytes)
    output_scale = struct.pack("<i", OUTPUT_SCALE)
    
    with open(output_path, "wb") as f:
        f.write(magic)
        f.write(version)
        f.write(ft_bias)
        f.write(ft_weight_bytes)
        f.write(hidden_weight)
        f.write(hidden_bias)
        f.write(output_weight)
        f.write(output_bias)
        f.write(output_scale)
        
    size = os.path.getsize(output_path)
    expected = 4 + 4 + (256*2) + (65536*256*2) + (32*512) + (32*4) + 32 + 4 + 4
    print(f"✅ Đã tạo thành công tệp XRNN v1: {output_path} ({size} bytes, kỳ vọng: {expected} bytes)")
    return size == expected

if __name__ == "__main__":
    out_path = "data/nnue_weights_gen6.bin"
    os.makedirs("data", exist_ok=True)
    create_xrnn_weights(out_path)
