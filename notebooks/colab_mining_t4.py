#!/usr/bin/env python3
# ============================================================================
# XIANGQI-RIM: COLAB T4 GPU MINING NOTEBOOK
# ============================================================================
# BẮT BUỘC sử dụng T4 GPU trên Colab.
# Pipeline 2 pha:
#   Phase 1 (Rust CPU): Tạo vị trí nhanh với depth thấp (CPU rất nhanh ở depth 1-2)
#   Phase 2 (T4 GPU):   Batch NNUE evaluation trên PyTorch T4 (~500K pos/s)
#
# Multi-Instance (4 notebooks song song):
#   Notebook 1: SEED=1, GAMES=25000
#   Notebook 2: SEED=2, GAMES=25000
#   Notebook 3: SEED=3, GAMES=25000
#   Notebook 4: SEED=4, GAMES=25000
#
# Mở Google Colab → Runtime → Change runtime type → T4 GPU
# ============================================================================

# %% [markdown]
# # 🏯 Xiangqi-RIM: Colab T4 GPU Data Mining
# **Bắt buộc T4 GPU** — PyTorch NNUE batch evaluation trên T4.

# %% Cell 1: Kiểm tra T4 GPU + Cài đặt Dependencies (~1 phút)
import subprocess
import os
import time
import sys

print("=" * 60)
print(" BƯỚC 1: KIỂM TRA T4 GPU & CÀI ĐẶT")
print("=" * 60)

# Kiểm tra GPU
result = subprocess.run(
    "nvidia-smi --query-gpu=name,memory.total,driver_version --format=csv,noheader",
    shell=True, capture_output=True, text=True
)
gpu_info = result.stdout.strip()
if "T4" not in gpu_info and "Tesla" not in gpu_info:
    print(f"⚠️ GPU hiện tại: {gpu_info}")
    print("❌ KHÔNG PHẢI T4! Vui lòng chọn Runtime → Change runtime type → T4 GPU")
    sys.exit(1)
print(f"✅ GPU: {gpu_info}")

# Kiểm tra CUDA
result = subprocess.run("nvcc --version 2>/dev/null | grep release", shell=True, capture_output=True, text=True)
print(f"CUDA: {result.stdout.strip()}")

# Kiểm tra PyTorch + CUDA
import torch
print(f"PyTorch: {torch.__version__}")
print(f"CUDA available: {torch.cuda.is_available()}")
if torch.cuda.is_available():
    print(f"CUDA device: {torch.cuda.get_device_name(0)}")
    print(f"VRAM: {torch.cuda.get_device_properties(0).total_memory / (1024**3):.1f} GB")
else:
    print("❌ CUDA không khả dụng! Kiểm tra lại runtime type.")
    sys.exit(1)

# CPU info
result = subprocess.run("nproc", shell=True, capture_output=True, text=True)
cores = int(result.stdout.strip())
print(f"CPU cores: {cores}")

result = subprocess.run("free -h | grep Mem | awk '{print $2}'", shell=True, capture_output=True, text=True)
print(f"RAM: {result.stdout.strip()}")

# %% Cell 2: Cài đặt Rust + Clone Repo + Build (~5 phút)
print("=" * 60)
print(" BƯỚC 2: RUST TOOLCHAIN & BUILD ENGINE")
print("=" * 60)

start = time.time()

# Cài Rust
if not os.path.exists("/root/.cargo/bin/rustc"):
    subprocess.run(
        "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y",
        shell=True, check=True
    )
os.environ["PATH"] += ":/root/.cargo/bin"

result = subprocess.run("/root/.cargo/bin/rustc --version", shell=True, capture_output=True, text=True)
print(f"Rust: {result.stdout.strip()}")

# Clone repo
if not os.path.exists("xiangqi-rim"):
    subprocess.run("git clone https://github.com/hoduyquocbao/xiangqi-rim.git", shell=True, check=True)
else:
    subprocess.run("cd xiangqi-rim && git pull", shell=True, check=True)

# Build release
os.chdir("xiangqi-rim")
subprocess.run(
    "/root/.cargo/bin/cargo build --release --example 20_parallel_mine",
    shell=True, check=True
)

elapsed = time.time() - start
print(f"\n✅ Build thành công trong {elapsed:.0f}s")

# %% Cell 3: Tải NNUE Weights từ HuggingFace
print("=" * 60)
print(" BƯỚC 3: TẢI NNUE WEIGHTS TỪ HUGGINGFACE")
print("=" * 60)

# Kiểm tra weights local
weights_path = "data/nnue_weights_gen5.bin"
if not os.path.exists(weights_path):
    print("  Weights không có local, tải từ HuggingFace...")
    subprocess.run("pip install -q huggingface_hub", shell=True, check=True)
    from huggingface_hub import hf_hub_download
    hf_hub_download(
        repo_id="hoduyquocbao/xiangqi-r1-dataset",
        filename="weights/nnue_weights_gen5.bin",
        local_dir=".",
        repo_type="dataset"
    )
    # Di chuyển vào data/
    os.makedirs("data", exist_ok=True)
    if os.path.exists("weights/nnue_weights_gen5.bin"):
        import shutil
        shutil.copy2("weights/nnue_weights_gen5.bin", weights_path)
    print(f"  ✅ Weights: {weights_path} ({os.path.getsize(weights_path):,} bytes)")
else:
    print(f"  ✅ Weights sẵn có: {weights_path} ({os.path.getsize(weights_path):,} bytes)")

# %% Cell 4: CẤU HÌNH MINING
# ============================================================
# ⚠️ SỬA CÁC GIÁ TRỊ SAU THEO TỪNG COLAB INSTANCE
# ============================================================

GAMES = 25000       # Số ván cờ (25000 × 4 instances = 100000)
SEED = 1            # Base seed (1, 2, 3, 4 cho mỗi instance)
DEPTH_GEN = 2       # Depth cho Phase 1 (CPU gen positions — nhanh)
GPU_BATCH = 8192    # Batch size cho T4 GPU (T4 có 16GB VRAM)
OUTPUT = f"data/gen7_gpu_seed{SEED}.jsonl"

print("=" * 60)
print(" CẤU HÌNH T4 GPU MINING PIPELINE")
print("=" * 60)
print(f"  GAMES      = {GAMES:,}")
print(f"  SEED       = {SEED}")
print(f"  DEPTH_GEN  = {DEPTH_GEN} (Phase 1 CPU — gen nhanh)")
print(f"  GPU_BATCH  = {GPU_BATCH} (Phase 2 GPU — T4 batch eval)")
print(f"  OUTPUT     = {OUTPUT}")

# Ước tính thời gian
# Phase 1: depth 2 → ~50 ván/s trên 2 cores → 25000 ván ≈ 8 phút
# Phase 2: ~500K pos/s trên T4 → 4.5M positions ≈ 9 giây
phase1_eta = GAMES / 50 / 60  # phút
phase2_eta = (GAMES * 180) / 500000 / 60  # phút
print(f"\n  Phase 1 ETA: ~{phase1_eta:.0f} phút (Rust CPU depth {DEPTH_GEN})")
print(f"  Phase 2 ETA: ~{phase2_eta:.1f} phút (T4 GPU batch NNUE)")
print(f"  Tổng ETA: ~{phase1_eta + phase2_eta:.0f} phút")
print(f"  Mẫu dự kiến: ~{GAMES * 180:,}")

# %% Cell 5: PHASE 1 — Rust Engine Gen Positions (CPU, fast)
print("=" * 60)
print(" PHASE 1: RUST ENGINE → GEN POSITIONS (CPU)")
print("=" * 60)
print(f"  Depth {DEPTH_GEN}, {GAMES:,} ván, SEED={SEED}")

start = time.time()
os.makedirs("data", exist_ok=True)

temp_output = OUTPUT + ".raw.jsonl"
env = os.environ.copy()
env["GAMES"] = str(GAMES)
env["DEPTH"] = str(DEPTH_GEN)
env["SEED"] = str(SEED)
env["THREADS"] = str(min(os.cpu_count() or 2, 4))
env["OUTPUT"] = temp_output

process = subprocess.Popen(
    ["./target/release/examples/20_parallel_mine"],
    env=env,
    stdout=subprocess.PIPE,
    stderr=subprocess.STDOUT,
    text=True,
    bufsize=1
)

for line in process.stdout:
    print(line, end="", flush=True)

code = process.wait()
elapsed = time.time() - start

if code == 0:
    count = sum(1 for _ in open(temp_output))
    print(f"\n✅ Phase 1 hoàn tất: {count:,} positions trong {elapsed:.0f}s")
else:
    print(f"\n❌ Phase 1 thất bại (exit code {code})")
    sys.exit(1)

# %% Cell 6: PHASE 2 — T4 GPU NNUE Batch Evaluation
import json
import numpy as np

print("=" * 60)
print(" PHASE 2: T4 GPU NNUE BATCH EVALUATION")
print("=" * 60)

# --- Hằng số NNUE ---
DIM = 256
BOTH = 512
HIDDEN = 32
TOTAL = 65536
SCALE_OUT = 16
QFT = 127.0
QHI = 64.0
QOU = 64.0

PIECE_MAP = {
    'R': 0, 'N': 1, 'B': 2, 'A': 3, 'K': 4, 'C': 5, 'P': 6,
    'r': 7, 'n': 8, 'b': 9, 'a': 10, 'k': 11, 'c': 12, 'p': 13,
}

# --- Load NNUE weights ---
print("  Loading NNUE weights...")
import struct

with open(weights_path, "rb") as f:
    magic = struct.unpack("<I", f.read(4))[0]
    version = struct.unpack("<I", f.read(4))[0]
    ft_bias = np.frombuffer(f.read(DIM * 2), dtype=np.int16).astype(np.float32) / QFT
    ft_weight = np.frombuffer(f.read(TOTAL * DIM * 2), dtype=np.int16).astype(np.float32) / QFT
    ft_weight = ft_weight.reshape(TOTAL, DIM)
    h_weight = np.frombuffer(f.read(HIDDEN * BOTH), dtype=np.int8).astype(np.float32) / QHI
    h_weight = h_weight.reshape(HIDDEN, BOTH)
    h_bias = np.frombuffer(f.read(HIDDEN * 4), dtype=np.int32).astype(np.float32) / (QFT * QHI)
    o_weight = np.frombuffer(f.read(HIDDEN), dtype=np.int8).astype(np.float32) / QOU
    o_weight = o_weight.reshape(1, HIDDEN)
    o_bias = np.frombuffer(f.read(4), dtype=np.int32).astype(np.float32) / (QFT * QHI * QOU)

print(f"  ✅ NNUE v{version}: FT={ft_weight.shape}, H={h_weight.shape}, O={o_weight.shape}")

# --- Build PyTorch model on T4 ---
print("  Building PyTorch NNUE model on CUDA (T4)...")
import torch
import torch.nn as nn

device = "cuda"

ft_w_tensor = torch.from_numpy(ft_weight).to(device)    # [65536, 256]
ft_b_tensor = torch.from_numpy(ft_bias).to(device)      # [256]
h_linear = nn.Linear(BOTH, HIDDEN, bias=True).to(device)
h_linear.weight.data = torch.from_numpy(h_weight).to(device)
h_linear.bias.data = torch.from_numpy(h_bias).to(device)
o_linear = nn.Linear(HIDDEN, 1, bias=True).to(device)
o_linear.weight.data = torch.from_numpy(o_weight).to(device)
o_linear.bias.data = torch.from_numpy(o_bias.reshape(1)).to(device)
print(f"  ✅ Model on {device}: {torch.cuda.get_device_name(0)}")

# --- FEN Parser & Feature Extractor ---
def parse_fen(fen):
    parts = fen.split()
    board_str = parts[0]
    side = 0 if len(parts) > 1 and parts[1] == 'w' else 1
    grid = [15] * 90
    row, col = 0, 0
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

def extract_features(grid, side):
    king_piece = 4 if side == 0 else 11
    king_sq = -1
    for i in range(90):
        if grid[i] == king_piece:
            king_sq = i
            break
    if king_sq < 0:
        return []
    features = []
    for sq in range(90):
        piece = grid[sq]
        if piece < 14:
            owner = piece // 7
            kind = piece % 7
            if owner == side:
                idx = king_sq * 630 + kind * 90 + sq
            else:
                idx = king_sq * 630 + (kind + 7) * 90 + sq
            if idx < TOTAL:
                features.append(idx)
    return features

# --- Load positions ---
print(f"  Loading positions from {temp_output}...")
samples = []
with open(temp_output, "r") as f:
    for line in f:
        line = line.strip()
        if line:
            samples.append(json.loads(line))
total = len(samples)
print(f"  Tổng mẫu: {total:,}")

# --- GPU Batch Evaluation ---
print(f"  Bắt đầu T4 GPU batch evaluation (batch={GPU_BATCH})...")
max_feat = 32
scored = []
start = time.time()

for offset in range(0, total, GPU_BATCH):
    chunk = samples[offset:offset + GPU_BATCH]
    size = len(chunk)

    # Chuẩn bị feature tensors
    stm = torch.full((size, max_feat), -1, dtype=torch.long, device=device)
    opp = torch.full((size, max_feat), -1, dtype=torch.long, device=device)

    for i, s in enumerate(chunk):
        grid, side = parse_fen(s["fen"])
        sf = extract_features(grid, side)
        of = extract_features(grid, 1 - side)
        for j, f in enumerate(sf[:max_feat]):
            stm[i, j] = f
        for j, f in enumerate(of[:max_feat]):
            opp[i, j] = f

    # GPU forward pass
    with torch.no_grad():
        # Feature Transform: sparse gather + accumulate
        stm_acc = ft_b_tensor.unsqueeze(0).expand(size, -1).clone()
        opp_acc = ft_b_tensor.unsqueeze(0).expand(size, -1).clone()

        for k in range(max_feat):
            idx_s = stm[:, k]
            mask_s = idx_s >= 0
            if mask_s.any():
                stm_acc[mask_s] += ft_w_tensor[idx_s[mask_s]]

            idx_o = opp[:, k]
            mask_o = idx_o >= 0
            if mask_o.any():
                opp_acc[mask_o] += ft_w_tensor[idx_o[mask_o]]

        # Clipped ReLU
        stm_acc = torch.clamp(stm_acc, 0.0, 1.0)
        opp_acc = torch.clamp(opp_acc, 0.0, 1.0)

        # Concat [batch, 512]
        combined = torch.cat([stm_acc, opp_acc], dim=1)

        # Hidden 512→32, ClippedReLU
        hidden_out = torch.clamp(h_linear(combined), 0.0, 1.0)

        # Output 32→1
        result = o_linear(hidden_out).squeeze(1) * SCALE_OUT
        gpu_scores = result.cpu().numpy().tolist()

    # Ghi kết quả
    for i, s in enumerate(chunk):
        s["score"] = int(round(gpu_scores[i]))
        s["gpu"] = True
        scored.append(s)

    # Progress
    done = min(offset + GPU_BATCH, total)
    elapsed = time.time() - start
    speed = done / elapsed if elapsed > 0 else 0
    eta = (total - done) / speed if speed > 0 else 0
    print(f"\r  🔥 T4 GPU: {done:,}/{total:,} ({100*done/total:.0f}%) | {speed:.0f} pos/s | ETA: {eta:.0f}s", end="", flush=True)

print()
elapsed = time.time() - start
print(f"  ✅ GPU eval hoàn tất: {total:,} positions trong {elapsed:.1f}s ({total/elapsed:.0f} pos/s)")

# --- Ghi output ---
print(f"\n  Ghi kết quả vào {OUTPUT}...")
with open(OUTPUT, "w") as f:
    for s in scored:
        f.write(json.dumps(s, ensure_ascii=False) + "\n")

size = os.path.getsize(OUTPUT)
print(f"  ✅ Output: {OUTPUT} ({size/(1024*1024):.1f} MB, {len(scored):,} mẫu)")

# Cleanup temp
if os.path.exists(temp_output):
    os.remove(temp_output)

# %% Cell 7: XÁC MINH KẾT QUẢ
print("=" * 60)
print(" BƯỚC 7: XÁC MINH KẾT QUẢ")
print("=" * 60)

scores_list = [s["score"] for s in scored[:100000]]
print(f"  Tổng mẫu: {len(scored):,}")
print(f"  File: {OUTPUT} ({os.path.getsize(OUTPUT)/(1024*1024):.1f} MB)")
print(f"  Score range: [{min(scores_list)}, {max(scores_list)}]")
print(f"  Score mean: {sum(scores_list)/len(scores_list):.1f}")
print(f"  GPU evaluated: {sum(1 for s in scored if s.get('gpu'))}")

# Mẫu đầu tiên
print(f"\n  Sample: {json.dumps(scored[0])}")

# %% Cell 8: UPLOAD LÊN GOOGLE DRIVE
from google.colab import drive

print("=" * 60)
print(" BƯỚC 8: UPLOAD LÊN GOOGLE DRIVE")
print("=" * 60)

drive.mount("/content/drive")
target = "/content/drive/MyDrive/xiangqi-mining"
os.makedirs(target, exist_ok=True)

import shutil
destination = os.path.join(target, os.path.basename(OUTPUT))
shutil.copy2(OUTPUT, destination)
print(f"✅ Uploaded: {destination} ({os.path.getsize(destination)/(1024*1024):.1f} MB)")

# %% Cell 9 (TÙY CHỌN): UPLOAD TRỰC TIẾP LÊN HUGGINGFACE
# HF_TOKEN = "hf_YOUR_TOKEN_HERE"  # Thay bằng token thật từ huggingface.co/settings/tokens
# REPO = "hoduyquocbao/xiangqi-r1-dataset"
#
# from huggingface_hub import HfApi
# api = HfApi(token=HF_TOKEN)
# api.upload_file(
#     path_or_fileobj=OUTPUT,
#     path_in_repo=os.path.basename(OUTPUT),
#     repo_id=REPO,
#     repo_type="dataset",
#     commit_message=f"feat: T4 GPU mining SEED={SEED} GAMES={GAMES}"
# )
# print(f"✅ Uploaded to HuggingFace: {REPO}")
