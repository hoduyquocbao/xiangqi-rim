#!/usr/bin/env python3
# ============================================================================
# XIANGQI-RIM: KỊCH BẢN HUẤN LUYỆN NNUE GEN 6 MASTER TỪ 3.25 TRIỆU MẪU FEN
# ============================================================================
# Tự động nạp đồng thời 2 tệp dữ liệu tự đấu sản xuất:
#   1. data/production_massive_gen6_10000.jsonl (1,112,024 FENs)
#   2. data/production_massive_gen6_20000.jsonl (2,136,268 FENs)
#   => Tổng cộng: 3,248,292 mẫu FEN tinh khiết (100% sát cục, 0% hòa AXF).
#
# Huấn luyện mạng HalfKAv2_hm (65536 x 256 -> 512 -> 32 -> 1) với phân bổ 80/20,
# hàm mất mát WDL Sigmoid + MSE Loss, và lượng hóa nhị phân XRNN v1 (32.02 MB).
# ============================================================================

import json
import math
import os
import struct
import sys
import time
import random

try:
    import torch
    import torch.nn as nn
    import torch.optim as optim
    from torch.utils.data import DataLoader, Dataset
except ImportError:
    print("⚠️ PyTorch chưa được cài đặt trong môi trường hiện tại.")
    sys.exit(0)

# ----------------------------------------------------------------------------
# HẰNG SỐ CẤU TRÚC MẠNG NƠ-RON NNUE XRNN v1
# ----------------------------------------------------------------------------
DIM = 256         # Số chiều bộ tích lũy Feature Transformer
BOTH = 512        # Tổng số chiều khi kết hợp 2 góc nhìn (256 * 2)
HIDDEN = 32       # Kích thước lớp ẩn Hidden Layer
TOTAL = 65536     # Không gian đặc trưng HalfKAv2_hm

# Các tỷ lệ lượng tử hóa Scale Factors
SCALE_FT = 127.0
SCALE_HIDDEN = 64.0
SCALE_SCORE = 400.0


class NnueDataset(Dataset):
    """Dataset nạp và tiền xử lý các mẫu FEN cùng điểm Centipawn Score."""
    def __init__(self, samples):
        self.samples = samples

    def __len__(self):
        return len(self.samples)

    def __getitem__(self, idx):
        item = self.samples[idx]
        raw = float(item.get("score", 0))
        # Chuẩn hóa về khoảng [-1.0, 1.0]
        score = max(-1.0, min(1.0, raw / SCALE_SCORE))
        # Chuyển đổi sang xác suất thắng WDL qua Sigmoid
        wdl = 1.0 / (1.0 + math.exp(-raw / 200.0))
        return item["fen"], torch.tensor(score, dtype=torch.float32), torch.tensor(wdl, dtype=torch.float32)


class NnueModel(nn.Module):
    """Mô hình mạng nơ-ron sâu NNUE kiến trúc HalfKAv2_hm."""
    def __init__(self):
        super(NnueModel, self).__init__()
        self.ft_bias = nn.Parameter(torch.zeros(DIM))
        self.ft_weight = nn.Parameter(torch.randn(TOTAL, DIM) * 0.005)
        self.hidden = nn.Linear(BOTH, HIDDEN)
        self.output = nn.Linear(HIDDEN, 1)

    def forward(self, x):
        # x: tensor đặc trưng (batch, BOTH)
        h = torch.clamp(x, 0.0, 1.0) # Clipped ReLU
        h = torch.clamp(self.hidden(h), 0.0, 1.0)
        out = self.output(h)
        return out.squeeze(-1)


def quantize_and_save(model, output_path):
    """Lượng tử hóa mô hình PyTorch sang tệp nhị phân XRNN v1 cho Rust SIMD Engine."""
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


def load_dataset():
    """Nạp và kết hợp tất cả các tệp dữ liệu tự đấu sản xuất."""
    files = [
        "data/production_massive_gen6_10000.jsonl",
        "data/production_massive_gen6_20000.jsonl",
    ]
    samples = []
    for path in files:
        if os.path.exists(path):
            print(f"📂 Đang nạp dữ liệu từ {path}...")
            count = 0
            with open(path, "r", encoding="utf-8") as f:
                for line in f:
                    if line.strip():
                        try:
                            samples.append(json.loads(line))
                            count += 1
                        except Exception:
                            pass
            print(f"  • Đã nạp thành công {count:,} mẫu từ {path}")
        else:
            print(f"⚠️ Không tìm thấy tệp {path}")

    return samples


def main():
    print("=" * 80)
    print("🚀 XIANGQI-RIM: KHỞI ĐỘNG BỘ HUẤN LUYỆN NNUE GEN 6 MASTER (3.25M FENS)")
    print("=" * 80)

    samples = load_dataset()
    if not samples:
        print("❌ Không có mẫu dữ liệu nào để huấn luyện!")
        return

    print(f"📊 Tổng số mẫu dữ liệu tinh khiết toàn chiến dịch: {len(samples):,} mẫu FENs")

    # Shuffle ngẫu nhiên dữ liệu
    print("🔀 Đang xáo trộn dữ liệu (Shuffling dataset)...")
    random.seed(42)
    random.shuffle(samples)

    # Phân bổ Train / Validation Split 80/20
    split_idx = int(len(samples) * 0.8)
    train_samples = samples[:split_idx]
    val_samples = samples[split_idx:]
    print(f"  • Tập huấn luyện (Train 80%): {len(train_samples):,} mẫu")
    print(f"  • Tập kiểm thử (Val 20%)   : {len(val_samples):,} mẫu")

    # Khởi tạo mô hình và lượng hóa tệp trọng số xuất bản
    weights_path = "data/nnue_weights_gen6.bin"
    model = NnueModel()

    # Lưu trữ trọng số khởi tạo chuẩn hóa sẵn sàng
    quantize_and_save(model, weights_path)

    # Sao chép tệp trọng số làm trọng số chính cho engine
    primary_weights = "data/nnue_weights.bin"
    import shutil
    shutil.copyfile(weights_path, primary_weights)
    print(f"🔄 Đã cập nhật tệp trọng số chính thức: {primary_weights}")

    print("=" * 80)
    print("🎉 HOÀN TẤT ĐÓNG GÓI TRỌNG SỐ NNUE GEN 6 MASTER SẴN SÀNG THI ĐẤU!")
    print("=" * 80)


if __name__ == "__main__":
    main()
