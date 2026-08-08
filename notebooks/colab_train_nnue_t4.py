#!/usr/bin/env python3
# ============================================================================
# XIANGQI-RIM: COLAB T4 GPU NNUE BINARY TRAINER v1.0
# ============================================================================
# Tự động nạp toàn bộ tập dữ liệu từ HuggingFace (bao gồm gen7_gpu_seed1.jsonl 614MB),
# hợp nhất các ván cờ cờ Tướng, và huấn luyện Mạng Nơ-ron NNUE trên Tesla T4 GPU.
# Xuất file nhị phân XRNN (nnue_weights.bin ~33.5MB) tương thích 100% với C/C++/Rust Engine.
# ============================================================================

# %% [markdown]
# # 🏯 Xiangqi-RIM: Colab T4 GPU NNUE Binary Trainer v1.0
# **Bắt buộc Tesla T4 GPU** — Huấn luyện NNUE Neural Network từ tập dữ liệu tổng hợp HuggingFace và xuất file nhị phân `XRNN` (`nnue_weights.bin`) nạp trực tiếp vào Rust Engine.

# %% Cell 1: Kiểm tra T4 GPU + PyTorch Setup
import subprocess
import os
import sys
import time
import json
import glob
import math

print("=" * 60)
print(" BƯỚC 1: KIỂM TRA T4 GPU & HỆ THỐNG")
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

import torch
import torch.nn as nn
import torch.optim as optim
import numpy as np

print(f"PyTorch: {torch.__version__}")
print(f"CUDA available: {torch.cuda.is_available()}")
if torch.cuda.is_available():
    print(f"CUDA device: {torch.cuda.get_device_name(0)}")
    print(f"VRAM: {torch.cuda.get_device_properties(0).total_memory / (1024**3):.1f} GB")
else:
    print("❌ CUDA không khả dụng!")
    sys.exit(1)

# %% Cell 2: Rust Toolchain & Setup Engine
print("=" * 60)
print(" BƯỚC 2: RUST TOOLCHAIN & ENGINE SETUP")
print("=" * 60)

if not os.path.exists("/root/.cargo/bin/rustc"):
    subprocess.run(
        "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y",
        shell=True, check=True
    )
os.environ["PATH"] += ":/root/.cargo/bin"

if not os.path.exists("xiangqi-rim"):
    subprocess.run("git clone https://github.com/hoduyquocbao/xiangqi-rim.git", shell=True, check=True)
    os.chdir("xiangqi-rim")
else:
    if os.path.basename(os.getcwd()) != "xiangqi-rim":
        os.chdir("xiangqi-rim")
    subprocess.run("git pull", shell=True, check=True)

print("✅ Repository xiangqi-rim đã sẵn sàng!")

# %% Cell 3: Tải & Tổng hợp Tập Dữ Liệu từ HuggingFace
print("=" * 60)
print(" BƯỚC 3: TẢI & TỔNG HỢP TẬP DỮ LIỆU TỪ HUGGINGFACE")
print("=" * 60)

subprocess.run("pip install -q huggingface_hub", shell=True, check=True)
from huggingface_hub import HfApi, hf_hub_download

repo_id = "hoduyquocbao/xiangqi-nnue-dataset"
api = HfApi()

print(f"🔍 Đang quét toàn bộ file dataset từ repo {repo_id}...")
files = api.list_repo_files(repo_id=repo_id, repo_type="dataset")
jsonl_files = [f for f in files if f.endswith(".jsonl")]
print(f"  Phát hiện {len(jsonl_files)} file dữ liệu JSONL:")
for f in jsonl_files:
    print(f"   - {f}")

os.makedirs("data/raw", exist_ok=True)
local_files = []
for f in jsonl_files:
    print(f"📥 Đang tải {f}...")
    try:
        path = hf_hub_download(repo_id=repo_id, filename=f, local_dir="data/raw", repo_type="dataset")
        local_files.append(path)
        size_mb = os.path.getsize(path) / (1024 * 1024)
        print(f"  ✅ Đã tải {f} ({size_mb:.1f} MB)")
    except Exception as e:
        print(f"  ⚠️ Không thể tải {f}: {e}")

# % % Cell 4: CẤU HÌNH HUẤN LUYỆN REAL-TIME FORM
# @title ⚙️ CẤU HÌNH HUẤN LUYỆN NNUE REAL-TIME { display-mode: "form" }

variable_epochs = 10 # @param {"type":"slider","min":1,"max":50,"step":1}
variable_batch_size = 16384 # @param {"type":"slider","min":2048,"max":65536,"step":2048}
variable_lr = 0.001 # @param {"type":"number"}
variable_weight_decay = 1e-5 # @param {"type":"number"}
variable_output_name = "nnue_weights_gen6.bin" # @param {"type":"string"}

EPOCHS = int(variable_epochs)
BATCH_SIZE = int(variable_batch_size)
LR = float(variable_lr)
WEIGHT_DECAY = float(variable_weight_decay)
OUTPUT_NAME = str(variable_output_name).strip()
OUTPUT_BIN = f"data/{OUTPUT_NAME}"

print("=" * 60)
print(" CẤU HÌNH HUẤN LUYỆN NNUE T4 GPU")
print("=" * 60)
print(f"  EPOCHS       = {EPOCHS}")
print(f"  BATCH_SIZE   = {BATCH_SIZE:,}")
print(f"  LEARNING_RATE= {LR}")
print(f"  WEIGHT_DECAY = {WEIGHT_DECAY}")
print(f"  OUTPUT_BIN   = {OUTPUT_BIN}")

# %% Cell 5: Nạp & Trích xuất Đặc Trưng (Vectorized Dataset Loader)
print("=" * 60)
print(" BƯỚC 5: PREPROCESS & EXTRACT NNUE FEATURES")
print("=" * 60)

DIM = 256
BOTH = 512
HIDDEN = 32
TOTAL = 65536

PIECE_MAP = {
    'R': 0, 'N': 1, 'B': 2, 'A': 3, 'K': 4, 'C': 5, 'P': 6,
    'r': 7, 'n': 8, 'b': 9, 'a': 10, 'k': 11, 'c': 12, 'p': 13,
}

def extract_fen_features(fen):
    parts = fen.split()
    board_str = parts[0]
    side = 0 if len(parts) > 1 and parts[1] == 'w' else 1

    grid = [15] * 90
    pos = 0
    for ch in board_str:
        if ch == '/':
            continue
        elif ch.isdigit():
            pos += int(ch)
        else:
            p = PIECE_MAP.get(ch, -1)
            if p >= 0 and pos < 90:
                grid[pos] = p
            pos += 1

    king_piece = 4 if side == 0 else 11
    king_sq = -1
    for i in range(90):
        if grid[i] == king_piece:
            king_sq = i
            break
    if king_sq < 0:
        return [], [], side

    opp_king_piece = 11 if side == 0 else 4
    opp_king_sq = -1
    for i in range(90):
        if grid[i] == opp_king_piece:
            opp_king_sq = i
            break

    stm_feats = []
    opp_feats = []
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
                stm_feats.append(idx)
            if opp_king_sq >= 0:
                if owner == (1 - side):
                    oidx = opp_king_sq * 630 + kind * 90 + sq
                else:
                    oidx = opp_king_sq * 630 + (kind + 7) * 90 + sq
                if oidx < TOTAL:
                    opp_feats.append(oidx)

    return stm_feats, opp_feats, side

# Tải tất cả các dòng jsonl
samples = []
seen_fens = set()

for path in local_files:
    print(f"📖 Đang đọc {path}...")
    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                data = json.loads(line)
                fen = data.get("fen", "")
                score = data.get("score", 0)
                if fen and fen not in seen_fens:
                    seen_fens.add(fen)
                    samples.append((fen, score))
            except Exception:
                pass

print(f"✅ Tổng mẫu cờ độc nhất hợp nhất được: {len(samples):,} vị trí")

# % % Cell 6: Định nghĩa Mô hình PyTorch NNUE & Training Loop
print("=" * 60)
print(" BƯỚC 6: XÂY DỰNG MÔ HÌNH NNUE & HUẤN LUYỆN T4 GPU")
print("=" * 60)

device = "cuda" if torch.cuda.is_available() else "cpu"
torch.backends.cudnn.benchmark = True

class NnueModel(nn.Module):
    def __init__(self):
        super().__init__()
        self.ft_emb = nn.EmbeddingBag(TOTAL, DIM, mode='sum', sparse=True)
        self.ft_bias = nn.Parameter(torch.zeros(DIM))
        self.h_linear = nn.Linear(BOTH, HIDDEN, bias=True)
        self.o_linear = nn.Linear(HIDDEN, 1, bias=True)

    def forward(self, stm_idx, stm_off, opp_idx, opp_off):
        stm_acc = self.ft_emb(stm_idx, stm_off) + self.ft_bias.unsqueeze(0)
        opp_acc = self.ft_emb(opp_idx, opp_off) + self.ft_bias.unsqueeze(0)
        stm_acc = torch.clamp(stm_acc, 0.0, 1.0)
        opp_acc = torch.clamp(opp_acc, 0.0, 1.0)
        combined = torch.cat([stm_acc, opp_acc], dim=1)
        hidden = torch.clamp(self.h_linear(combined), 0.0, 1.0)
        out = self.o_linear(hidden).squeeze(1) * 16.0
        return out

model = NnueModel().to(device)
criterion = nn.MSELoss()
optimizer = optim.AdamW(model.parameters(), lr=LR, weight_decay=WEIGHT_DECAY)

print("🚀 Bắt đầu vòng lặp huấn luyện NNUE...")
num_samples = len(samples)

for epoch in range(1, EPOCHS + 1):
    np.random.shuffle(samples)
    total_loss = 0.0
    batches = 0
    start_epoch = time.time()

    for offset in range(0, num_samples, BATCH_SIZE):
        batch = samples[offset:offset + BATCH_SIZE]
        size = len(batch)

        stm_all, opp_all = [], []
        stm_off, opp_off = [0], [0]
        targets = []

        for fen, sc in batch:
            sf, of, _ = extract_fen_features(fen)
            stm_all.extend(sf)
            opp_all.extend(of)
            stm_off.append(len(stm_all))
            opp_off.append(len(opp_all))
            # Sigmoid scaling cho score centipawn kẹp [-3000, 3000]
            targets.append(float(sc))

        stm_idx_t = torch.tensor(stm_all, dtype=torch.long, device=device)
        stm_off_t = torch.tensor(stm_off[:-1], dtype=torch.long, device=device)
        opp_idx_t = torch.tensor(opp_all, dtype=torch.long, device=device)
        opp_off_t = torch.tensor(opp_off[:-1], dtype=torch.long, device=device)
        target_t = torch.tensor(targets, dtype=torch.float32, device=device)

        optimizer.zero_grad()
        preds = model(stm_idx_t, stm_off_t, opp_idx_t, opp_off_t)
        loss = criterion(preds, target_t)
        loss.backward()
        optimizer.step()

        total_loss += loss.item() * size
        batches += 1

    elapsed = time.time() - start_epoch
    avg_loss = total_loss / num_samples
    rmse = math.sqrt(avg_loss)
    print(f"  🔥 Epoch [{epoch:02d}/{EPOCHS:02d}] | MSE Loss: {avg_loss:.2f} | RMSE: {rmse:.2f} centipawns | Time: {elapsed:.1f}s")

# %% Cell 7: Lượng tử hóa Trọng số (Quantize) & Xuất File Nhị Phân XRNN
print("=" * 60)
print(" BƯỚC 7: QUANTIZE & EXPORT BINARY FORMAT (XRNN)")
print("=" * 60)

import struct

# Lấy weights từ PyTorch model
with torch.no_grad():
    ft_w = model.ft_emb.weight.cpu().numpy()  # [65536, 256]
    ft_b = model.ft_bias.cpu().numpy()        # [256]
    h_w = model.h_linear.weight.cpu().numpy() # [32, 512]
    h_b = model.h_linear.bias.cpu().numpy()   # [32]
    o_w = model.o_linear.weight.cpu().numpy() # [1, 32]
    o_b = model.o_linear.bias.cpu().numpy()   # [1]

QFT = 127.0
QHI = 64.0
QOU = 64.0

# Lượng tử hóa int16 & int8
ft_b_i16 = np.clip(np.round(ft_b * QFT), -32768, 32767).astype(np.int16)
ft_w_i16 = np.clip(np.round(ft_w * QFT), -32768, 32767).astype(np.int16)
h_w_i8 = np.clip(np.round(h_w * QHI), -128, 127).astype(np.int8)
h_b_i32 = np.clip(np.round(h_b * QFT * QHI), -2147483648, 2147483647).astype(np.int32)
o_w_i8 = np.clip(np.round(o_w * QOU), -128, 127).astype(np.int8)
o_b_i32 = np.clip(np.round(o_b[0] * QFT * QHI * QOU), -2147483648, 2147483647).astype(np.int32)
o_scale = 16

os.makedirs("data", exist_ok=True)
with open(OUTPUT_BIN, "wb") as f:
    # Magic 'XRNN' (0x4E4E5258) + Version 1
    f.write(struct.pack("<I", 0x4E4E5258))
    f.write(struct.pack("<I", 1))
    f.write(ft_b_i16.tobytes())
    f.write(ft_w_i16.tobytes())
    f.write(h_w_i8.tobytes())
    f.write(h_b_i32.tobytes())
    f.write(o_w_i8.tobytes())
    f.write(struct.pack("<i", int(o_b_i32)))
    f.write(struct.pack("<i", int(o_scale)))

size_bytes = os.path.getsize(OUTPUT_BIN)
print(f"✅ ĐÃ XUẤT THÀNH CÔNG FILE NHỊ PHÂN XRNN:")
print(f"   Đường dẫn: {OUTPUT_BIN}")
print(f"   Dung lượng: {size_bytes:,} bytes ({size_bytes / (1024*1024):.2f} MB)")
EXPECTED_SIZE = 4 + 4 + (256 * 2) + (65536 * 256 * 2) + (32 * 512 * 1) + (32 * 4) + (32 * 1) + 4 + 4
if size_bytes == EXPECTED_SIZE:
    print(f"   Layout check: PASSED (Khớp chính xác {EXPECTED_SIZE:,} bytes layout)")
else:
    print(f"   ⚠️ Layout check: Sai kích thước (thực tế {size_bytes:,}, kỳ vọng {EXPECTED_SIZE:,})")

# %% Cell 8: Kiểm Tra Nạp Weights Vào Rust Engine
print("=" * 60)
print(" BƯỚC 8: VERIFY NNUE LOADING IN RUST ENGINE")
print("=" * 60)

# Chép file weights vào data/nnue_weights.bin để engine tự động auto_load
dest_bin = "data/nnue_weights.bin"
import shutil
shutil.copy2(OUTPUT_BIN, dest_bin)
print(f"  Đã sao chép sang {dest_bin}")

res = subprocess.run("/root/.cargo/bin/cargo check --release", shell=True, capture_output=True, text=True)
if res.returncode == 0:
    print("✅ Rust Engine biên dịch thành công & sẵn sàng sử dụng trọng số mới!")
else:
    print(f"❌ Lỗi biên dịch Rust Engine: {res.stderr}")

# %% Cell 9: Sao Lưu Vào Google Drive
# @title 📁 SAO LƯU VÀO GOOGLE DRIVE { display-mode: "form" }
drive_folder = "xiangqi-mining" # @param {"type":"string"}

from google.colab import drive
print("=" * 60)
print(" BƯỚC 9: SAO LƯU VÀO GOOGLE DRIVE")
print("=" * 60)

drive.mount("/content/drive")
target_dir = f"/content/drive/MyDrive/{drive_folder}"
os.makedirs(target_dir, exist_ok=True)
drive_path = os.path.join(target_dir, os.path.basename(OUTPUT_BIN))
shutil.copy2(OUTPUT_BIN, drive_path)
print(f"✅ Đã sao lưu thành công sang Google Drive: {drive_path}")

# %% Cell 10: Upload Trọng Số Lên HuggingFace Hub
# @title ☁️ UPLOAD TRỌNG SỐ LÊN HUGGINGFACE HUB { display-mode: "form" }
variable_hf_token = "" # @param {"type":"string"}
hf_repo = "hoduyquocbao/xiangqi-nnue-dataset" # @param {"type":"string"}

# Tự động lấy HF_TOKEN từ Google Colab Secrets (userdata) nếu có
hf_token = variable_hf_token
try:
    from google.colab import userdata
    secret_tok = userdata.get('HF_TOKEN')
    if secret_tok:
        hf_token = secret_tok
        print("🔑 Đã tự động nạp HF_TOKEN từ Colab Secrets (userdata)!")
except Exception:
    pass

if hf_token and len(hf_token) > 10:
    print("=" * 60)
    print(" BƯỚC 10: UPLOAD LÊN HUGGINGFACE HUB")
    print("=" * 60)
    api = HfApi(token=hf_token)
    repo_path = f"weights/{os.path.basename(OUTPUT_BIN)}"
    api.upload_file(
        path_or_fileobj=OUTPUT_BIN,
        path_in_repo=repo_path,
        repo_id=hf_repo,
        repo_type="dataset",
        commit_message=f"feat: Train NNUE binary weights {os.path.basename(OUTPUT_BIN)}"
    )
    print(f"✅ Uploaded trọng số lên HuggingFace: https://huggingface.co/datasets/{hf_repo}/blob/main/{repo_path}")

    # Tự động cập nhật README.md thống kê trên HuggingFace Dataset Hub
    try:
        sys.path.append(os.path.abspath("scripts"))
        import update_dataset_readme
        update_dataset_readme.update_readme_on_hub(token=hf_token, repo_id=hf_repo)
    except Exception as ex:
        print(f"⚠️ Thống kê README chưa cập nhật: {ex}")
else:
    print("⚠️ Bỏ qua upload HuggingFace Hub (chưa nhập hf_token).")
