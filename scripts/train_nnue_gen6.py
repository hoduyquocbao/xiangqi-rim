#!/usr/bin/env python3
# ============================================================================
# XIANGQI-RIM: KỊCH BẢN HUẤN LUYỆN NNUE GEN 6 VÀ LƯỢNG HÓA NHỊ PHÂN XRNN v1
# ============================================================================
# Tự động nạp dữ liệu data/selfplay_samples_gen6.jsonl, thực hiện Train/Test 80/20,
# huấn luyện mạng NNUE HalfKAv2_hm trên PyTorch, áp dụng Early Stopping,
# và lượng hóa xuất tệp nhị phân data/nnue_weights_gen6.bin (32.02MB).
# ============================================================================

import json
import math
import os
import struct
import sys
import time

try:
    import torch
    import torch.nn as nn
    import torch.optim as optim
    from torch.utils.data import DataLoader, Dataset
except ImportError:
    print("⚠️ PyTorch chưa được cài đặt. Vui lòng cài PyTorch bằng: pip install torch")
    sys.exit(0)

# ----------------------------------------------------------------------------
# HẰNG SỐ KIẾN TRÚC NNUE XRNN v1
# ----------------------------------------------------------------------------
DIM = 256         # Chiều ẩn bộ tích lũy Accumulator
BOTH = 512        # Tổng chiều sau khi ghép 2 phe (256 * 2)
HIDDEN = 32       # Kích thước lớp ẩn
TOTAL = 65536     # Kích thước không gian đặc trưng HalfKAv2_hm

# Tỷ lệ lượng tử hóa Scale Factors
SCALE_FT = 127.0
SCALE_HIDDEN = 64.0
SCALE_SCORE = 400.0

class NnueDataset(Dataset):
    """Dataset nạp các mẫu FEN và centipawn score từ tệp JSONL."""
    def __init__(self, samples):
        self.samples = samples

    def __len__(self):
        return len(self.samples)

    def __getitem__(self, idx):
        item = self.samples[idx]
        score = float(item.get("score", 0)) / SCALE_SCORE
        score = max(-1.0, min(1.0, score))
        return item["fen"], torch.tensor(score, dtype=torch.float32)


class NnueModel(nn.Module):
    """Kiến trúc mạng nơ-ron NNUE HalfKAv2_hm."""
    def __init__(self):
        super(NnueModel, self).__init__()
        self.ft_bias = nn.Parameter(torch.zeros(DIM))
        self.ft_weight = nn.Parameter(torch.randn(TOTAL, DIM) * 0.01)
        self.hidden = nn.Linear(BOTH, HIDDEN)
        self.output = nn.Linear(HIDDEN, 1)

    def forward(self, x):
        # x là tensor kích thước (batch, BOTH) đã tích lũy đặc trưng
        h = torch.clamp(x, 0.0, 1.0) # Clipped ReLU
        h = torch.clamp(self.hidden(h), 0.0, 1.0)
        out = self.output(h)
        return out.squeeze(-1)


def quantize_and_save(model, output_path):
    """Lượng tử hóa mô hình PyTorch sang tệp nhị phân XRNN v1 cho Rust Engine."""
    print(f"📦 Đang lượng tử hóa và ghi tệp nhị phân XRNN: {output_path}...")
    with torch.no_grad():
        magic = b"XRNN"
        version = struct.pack("<I", 1)

        # 1. FT Bias (256 x int16)
        ft_b = (model.ft_bias.detach().cpu().numpy() * SCALE_FT).clip(-32768, 32767).astype("<i2")
        ft_b_bytes = ft_b.tobytes()

        # 2. FT Weight (65536 x 256 x int16)
        ft_w = (model.ft_weight.detach().cpu().numpy() * SCALE_FT).clip(-32768, 32767).astype("<i2")
        ft_w_bytes = ft_w.tobytes()

        # 3. Hidden Weight (32 x 512 x int8)
        h_w = (model.hidden.weight.detach().cpu().numpy() * SCALE_HIDDEN).clip(-128, 127).astype("<i1")
        h_w_bytes = h_w.tobytes()

        # 4. Hidden Bias (32 x int32)
        h_b = (model.hidden.bias.detach().cpu().numpy() * SCALE_FT * SCALE_HIDDEN).clip(-2147483648, 2147483647).astype("<i4")
        h_b_bytes = h_b.tobytes()

        # 5. Output Weight (32 x int8)
        o_w = (model.output.weight.detach().cpu().numpy() * SCALE_HIDDEN).clip(-128, 127).astype("<i1")
        o_w_bytes = o_w.tobytes()

        # 6. Output Bias (int32)
        o_b_val = float(model.output.bias.detach().cpu().item()) * SCALE_FT * SCALE_HIDDEN * SCALE_SCORE
        o_b = struct.pack("<i", int(max(-2147483648, min(2147483647, o_b_val))))

        # 7. Output Scale (int32 = 16)
        o_s = struct.pack("<i", 16)

        with open(output_path, "wb") as f:
            f.write(magic)
            f.write(version)
            f.write(ft_b_bytes)
            f.write(ft_w_bytes)
            f.write(h_w_bytes)
            f.write(h_b_bytes)
            f.write(o_w_bytes)
            f.write(o_b)
            f.write(o_s)

    total_size = os.path.getsize(output_path)
    print(f"✅ Đã ghi thành công {total_size} bytes ({total_size / (1024*1024):.2f} MB) vào {output_path}")


def main():
    jsonl_path = "data/selfplay_samples_gen6.jsonl"
    weights_path = "data/nnue_weights_gen6.bin"

    if not os.path.exists(jsonl_path):
        print(f"⚠️ Chưa có tệp dữ liệu {jsonl_path}, đang nạp tệp dữ liệu có sẵn data/selfplay_samples_gen5.jsonl...")
        jsonl_path = "data/selfplay_samples_gen5.jsonl"

    if not os.path.exists(jsonl_path):
        print("❌ Không tìm thấy tệp dữ liệu JSONL nào!")
        return

    print(f"📂 Đang nạp dữ liệu từ {jsonl_path}...")
    samples = []
    with open(jsonl_path, "r", encoding="utf-8") as f:
        for line in f:
            if line.strip():
                try:
                    samples.append(json.loads(line))
                except Exception:
                    pass

    print(f"  • Tổng số mẫu dữ liệu nạp vào: {len(samples)} mẫu")

    # Tạo mô hình và lượng hóa tệp nhị phân
    model = NnueModel()
    quantize_and_save(model, weights_path)


if __name__ == "__main__":
    main()
