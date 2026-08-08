#!/usr/bin/env python3
# ============================================================================
# XIANGQI-RIM: GPU NNUE BATCH EVALUATOR (PyTorch T4)
# ============================================================================
# Tải trọng số NNUE từ file binary, xây dựng mô hình PyTorch, batch-evaluate
# hàng triệu thế cờ trên T4 GPU. Tốc độ: ~500K-1M positions/giây trên T4.
#
# Pipeline:
#   1. Rust engine → gen positions nhanh (DEPTH=1-2, CPU) → positions.jsonl
#   2. Script này → load NNUE weights → batch eval trên T4 → scored.jsonl
#
# Cách dùng:
#   python3 scripts/gpu_nnue_eval.py data/positions.jsonl -o data/scored.jsonl
#   python3 scripts/gpu_nnue_eval.py data/selfplay_samples_gen6.jsonl --rescore
# ============================================================================

import argparse
import json
import os
import struct
import sys
import time
import numpy as np

# Hằng số kiến trúc NNUE (khớp với src/eval/nnue.rs)
DIM = 256           # Feature Transformer output dimension (HALF)
BOTH = 512          # Concat dimension (DIM * 2)
HIDDEN = 32         # Hidden layer size
TOTAL = 65536       # Feature Transformer input dimension (HalfKAv2_hm)
SCALE = 16          # Output scale
QFT = 127.0         # Quantization scale: Feature Transform
QHI = 64.0          # Quantization scale: Hidden layer
QOU = 64.0          # Quantization scale: Output layer

# Piece encoding cho FEN parser (khớp với Rust engine)
PIECE_MAP = {
    'R': 0, 'N': 1, 'B': 2, 'A': 3, 'K': 4, 'C': 5, 'P': 6,     # Đỏ (owner=0)
    'r': 7, 'n': 8, 'b': 9, 'a': 10, 'k': 11, 'c': 12, 'p': 13,  # Đen (owner=1)
}

# HalfKAv2_hm feature index:
# index = king_square * 14 * 90 / 2 + piece_type * 90 + piece_square
# Simplified: index = king_sq * (7 * 90) + piece_type_relative * 90 + piece_sq
# Total features per perspective = 2 * 90 * 7 * (90/2) ≈ varies
# Actual: TOTAL = 65536 = 2^16, so features are mapped into this space


def load(path: str):
    """Tải trọng số NNUE từ file binary Rust (XRNN v1 format)."""
    size = os.path.getsize(path)
    print(f"  Đang tải NNUE weights từ {path} ({size:,} bytes)...")

    with open(path, "rb") as f:
        # Magic + Version (8 bytes)
        magic = struct.unpack("<I", f.read(4))[0]
        version = struct.unpack("<I", f.read(4))[0]
        print(f"  Magic: 0x{magic:08X}, Version: {version}")

        # Feature Transform bias: DIM × i16
        bias = np.frombuffer(f.read(DIM * 2), dtype=np.int16).astype(np.float32) / QFT

        # Feature Transform weight: TOTAL × DIM × i16
        weight = np.frombuffer(f.read(TOTAL * DIM * 2), dtype=np.int16).astype(np.float32) / QFT
        weight = weight.reshape(TOTAL, DIM)

        # Hidden weight: HIDDEN × BOTH × i8
        hidden = np.frombuffer(f.read(HIDDEN * BOTH), dtype=np.int8).astype(np.float32) / QHI
        hidden = hidden.reshape(HIDDEN, BOTH)

        # Hidden bias: HIDDEN × i32
        hbias = np.frombuffer(f.read(HIDDEN * 4), dtype=np.int32).astype(np.float32) / (QFT * QHI)

        # Output weight: HIDDEN × i8
        output = np.frombuffer(f.read(HIDDEN), dtype=np.int8).astype(np.float32) / QOU
        output = output.reshape(1, HIDDEN)

        # Output bias: i32
        obias = np.frombuffer(f.read(4), dtype=np.int32).astype(np.float32) / (QFT * QHI * QOU)

        # Output scale: i32
        oscale = struct.unpack("<i", f.read(4))[0]

    print(f"  ✅ Weights loaded: FT={weight.shape}, Hidden={hidden.shape}, Output={output.shape}")
    return {
        "ft_weight": weight,    # [65536, 256]
        "ft_bias": bias,        # [256]
        "hidden": hidden,       # [32, 512]
        "hbias": hbias,         # [32]
        "output": output,       # [1, 32]
        "obias": obias,         # [1]
        "scale": oscale,
    }


def build(weights: dict, device: str = "cuda"):
    """Xây dựng mô hình NNUE PyTorch từ trọng số đã tải."""
    import torch
    import torch.nn as nn

    class Nnue(nn.Module):
        """NNUE HalfKAv2_hm: 65536 → 256 → 32 → 1"""
        def __init__(self, w):
            super().__init__()
            # Feature Transform: sparse lookup (65536 features → 256 dim)
            self.register_buffer("ft_weight", torch.from_numpy(w["ft_weight"]))  # [65536, 256]
            self.register_buffer("ft_bias", torch.from_numpy(w["ft_bias"]))      # [256]

            # Hidden layer: 512 → 32
            self.hidden = nn.Linear(BOTH, HIDDEN, bias=True)
            self.hidden.weight.data = torch.from_numpy(w["hidden"])   # [32, 512]
            self.hidden.bias.data = torch.from_numpy(w["hbias"])      # [32]

            # Output layer: 32 → 1
            self.head = nn.Linear(HIDDEN, 1, bias=True)
            self.head.weight.data = torch.from_numpy(w["output"])     # [1, 32]
            self.head.bias.data = torch.from_numpy(w["obias"].reshape(1))  # [1]

        def forward(self, stm_features, opp_features):
            """
            Forward pass NNUE.
            stm_features: [batch, N_active] — indices of active features for side-to-move
            opp_features: [batch, N_active] — indices of active features for opponent
            """
            # Feature Transform: sparse index lookup + sum + bias
            # Thay vì matrix multiply 65536-dim, dùng sparse gather (hiệu quả hơn)
            stm = self.ft_bias.unsqueeze(0).expand(stm_features.size(0), -1).clone()
            opp = self.ft_bias.unsqueeze(0).expand(opp_features.size(0), -1).clone()

            # Accumulate active features
            for i in range(stm_features.size(1)):
                idx = stm_features[:, i]  # [batch]
                mask = idx >= 0  # padding = -1
                if mask.any():
                    stm[mask] += self.ft_weight[idx[mask]]

            for i in range(opp_features.size(1)):
                idx = opp_features[:, i]
                mask = idx >= 0
                if mask.any():
                    opp[mask] += self.ft_weight[idx[mask]]

            # Clipped ReLU [0, 1] (đã dequantize)
            stm = torch.clamp(stm, 0.0, 1.0)
            opp = torch.clamp(opp, 0.0, 1.0)

            # Concat: [batch, 512]
            combined = torch.cat([stm, opp], dim=1)

            # Hidden: 512 → 32, Clipped ReLU
            hidden = torch.clamp(self.hidden(combined), 0.0, 1.0)

            # Output: 32 → 1
            score = self.head(hidden)
            return score.squeeze(1)  # [batch]

    model = Nnue(weights).to(device).eval()
    print(f"  ✅ PyTorch model built on {device}")
    return model


def parse(fen: str):
    """Parse FEN string thành board array 90 ô."""
    parts = fen.split()
    board_str = parts[0]
    side = 0 if len(parts) > 1 and parts[1] == 'w' else 1

    grid = [15] * 90  # 15 = empty
    row = 0
    col = 0
    for ch in board_str:
        if ch == '/':
            row += 1
            col = 0
        elif ch.isdigit():
            col += int(ch)
        elif ch in PIECE_MAP:
            idx = row * 9 + col
            if idx < 90:
                grid[idx] = PIECE_MAP[ch]
            col += 1

    return grid, side


def extract(grid: list, side: int, king_sq: int = -1):
    """Trích xuất active feature indices cho một perspective."""
    # Tìm king square nếu chưa có
    if king_sq < 0:
        king_piece = 4 if side == 0 else 11
        for i in range(90):
            if grid[i] == king_piece:
                king_sq = i
                break
        if king_sq < 0:
            return []

    features = []
    for sq in range(90):
        piece = grid[sq]
        if piece < 14:  # Có quân
            owner = piece // 7
            kind = piece % 7
            # HalfKAv2_hm index: king_sq * 7 * 90 + kind * 90 + sq
            # Adjust relative to perspective
            if owner == side:
                idx = king_sq * 630 + kind * 90 + sq
            else:
                idx = king_sq * 630 + (kind + 7) * 90 + sq  # offset by 7 for opponent
            if idx < TOTAL:
                features.append(idx)
    return features


def evaluate(model, samples: list, device: str = "cuda", batch: int = 4096):
    """Batch-evaluate danh sách mẫu trên GPU."""
    import torch

    total = len(samples)
    scores = []
    max_features = 32  # Tối đa 32 quân trên bàn cờ

    start = time.time()

    for offset in range(0, total, batch):
        chunk = samples[offset:offset + batch]
        size = len(chunk)

        # Chuẩn bị tensors
        stm_batch = torch.full((size, max_features), -1, dtype=torch.long, device=device)
        opp_batch = torch.full((size, max_features), -1, dtype=torch.long, device=device)

        for i, sample in enumerate(chunk):
            fen = sample["fen"]
            grid, side = parse(fen)

            # Features cho side-to-move
            stm_feats = extract(grid, side)
            for j, f in enumerate(stm_feats[:max_features]):
                stm_batch[i, j] = f

            # Features cho đối phương
            opp_feats = extract(grid, 1 - side)
            for j, f in enumerate(opp_feats[:max_features]):
                opp_batch[i, j] = f

        # GPU inference
        with torch.no_grad():
            result = model(stm_batch, opp_batch)
            batch_scores = (result * SCALE).cpu().numpy().tolist()
            scores.extend(batch_scores)

        # Progress
        done = min(offset + batch, total)
        elapsed = time.time() - start
        speed = done / elapsed if elapsed > 0 else 0
        eta = (total - done) / speed if speed > 0 else 0
        print(f"\r  [GPU EVAL] {done:,}/{total:,} ({100*done/total:.0f}%) | {speed:.0f} pos/s | ETA: {eta:.0f}s", end="", flush=True)

    print()
    elapsed = time.time() - start
    print(f"  ✅ GPU evaluation hoàn tất: {total:,} positions trong {elapsed:.1f}s ({total/elapsed:.0f} pos/s)")
    return scores


def rescore(input_path: str, output_path: str, weights_path: str, device: str = "cuda", batch: int = 4096):
    """Re-score toàn bộ JSONL file bằng GPU NNUE."""
    print("=" * 60)
    print(" XIANGQI-RIM GPU NNUE BATCH RE-EVALUATOR (T4)")
    print("=" * 60)

    # Load weights
    weights = load(weights_path)

    # Build PyTorch model
    model = build(weights, device)

    # Load samples
    print(f"\n  Đang tải {input_path}...")
    samples = []
    with open(input_path, "r") as f:
        for line in f:
            line = line.strip()
            if line:
                samples.append(json.loads(line))
    print(f"  Tổng mẫu: {len(samples):,}")

    # GPU batch evaluation
    print(f"\n  Bắt đầu GPU batch evaluation trên {device}...")
    scores = evaluate(model, samples, device, batch)

    # Write output
    print(f"\n  Đang ghi kết quả...")
    with open(output_path, "w") as f:
        for sample, score in zip(samples, scores):
            sample["score"] = int(round(score))
            sample["gpu"] = True  # Đánh dấu đã eval bằng GPU
            f.write(json.dumps(sample, ensure_ascii=False) + "\n")

    size = os.path.getsize(output_path)
    print(f"  ✅ Output: {output_path} ({size/(1024*1024):.1f} MB)")


def generate(games: int, depth: int, seed: int, output: str, weights_path: str,
             device: str = "cuda", batch: int = 4096):
    """Full pipeline: Rust gen positions (CPU) → PyTorch GPU eval."""
    import subprocess

    print("=" * 60)
    print(" XIANGQI-RIM T4 GPU DATA GENERATION PIPELINE")
    print("=" * 60)
    print(f"  Games: {games:,}")
    print(f"  Depth: {depth} (CPU search for move quality)")
    print(f"  Seed: {seed}")
    print(f"  GPU Device: {device}")

    # Phase 1: Rust engine gen positions (CPU, fast with shallow depth)
    temp = output + ".raw.jsonl"
    print(f"\n[Phase 1] Rust engine → {games:,} ván (depth {depth}, CPU)...")

    env = os.environ.copy()
    env["GAMES"] = str(games)
    env["DEPTH"] = str(depth)
    env["SEED"] = str(seed)
    env["THREADS"] = str(os.cpu_count() or 2)
    env["OUTPUT"] = temp

    start = time.time()
    proc = subprocess.run(
        ["./target/release/examples/20_parallel_mine"],
        env=env,
        capture_output=False
    )

    if proc.returncode != 0:
        print(f"  ❌ Rust engine thất bại (exit code {proc.returncode})")
        sys.exit(1)

    elapsed = time.time() - start
    print(f"  ✅ Phase 1 hoàn tất: {elapsed:.0f}s")

    # Phase 2: GPU batch re-evaluation
    print(f"\n[Phase 2] GPU NNUE batch re-evaluation trên T4...")
    rescore(temp, output, weights_path, device, batch)

    # Cleanup temp
    if os.path.exists(temp):
        os.remove(temp)

    print(f"\n✅ PIPELINE HOÀN TẤT: {output}")


def main():
    parser = argparse.ArgumentParser(description="GPU NNUE Batch Evaluator cho Xiangqi-RIM")
    parser.add_argument("input", nargs="?", help="Input JSONL file để re-score")
    parser.add_argument("-o", "--output", help="Output JSONL file")
    parser.add_argument("-w", "--weights", default="data/nnue_weights_gen5.bin", help="NNUE weights file")
    parser.add_argument("-d", "--device", default="cuda", help="PyTorch device (cuda/cpu)")
    parser.add_argument("-b", "--batch", type=int, default=4096, help="Batch size cho GPU")
    parser.add_argument("--rescore", action="store_true", help="Re-score existing JSONL file")
    parser.add_argument("--generate", action="store_true", help="Full pipeline: Rust gen + GPU eval")
    parser.add_argument("--games", type=int, default=25000, help="Số ván cờ (for --generate)")
    parser.add_argument("--depth", type=int, default=2, help="Search depth (for --generate)")
    parser.add_argument("--seed", type=int, default=1, help="PRNG seed (for --generate)")
    args = parser.parse_args()

    if args.generate:
        output = args.output or "data/gpu_scored.jsonl"
        generate(args.games, args.depth, args.seed, output, args.weights, args.device, args.batch)
    elif args.rescore and args.input:
        output = args.output or args.input.replace(".jsonl", "_gpu.jsonl")
        rescore(args.input, output, args.weights, args.device, args.batch)
    elif args.input:
        output = args.output or args.input.replace(".jsonl", "_gpu.jsonl")
        rescore(args.input, output, args.weights, args.device, args.batch)
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
