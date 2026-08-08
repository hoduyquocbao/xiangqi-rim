#!/usr/bin/env python3
# ============================================================================
# XIANGQI-RIM: COLAB T4 MINING NOTEBOOK
# ============================================================================
# Chạy trên Google Colab để mining dữ liệu huấn luyện NNUE quy mô lớn.
# Compile Rust engine trực tiếp trên Colab, chạy mining, upload Google Drive.
#
# Hướng dẫn:
#   1. Mở Google Colab: https://colab.research.google.com
#   2. Chọn Runtime → Change runtime type → T4 GPU (hoặc CPU)
#   3. Copy từng cell vào notebook và chạy tuần tự
#
# Multi-Instance (4 notebooks song song):
#   Notebook 1: SEED=1, OUTPUT=gen6_part1.jsonl, GAMES=25000
#   Notebook 2: SEED=2, OUTPUT=gen6_part2.jsonl, GAMES=25000
#   Notebook 3: SEED=3, OUTPUT=gen6_part3.jsonl, GAMES=25000
#   Notebook 4: SEED=4, OUTPUT=gen6_part4.jsonl, GAMES=25000
# ============================================================================

# %% [markdown]
# # 🏯 Xiangqi-RIM: Colab T4 Data Mining
# Mining dữ liệu huấn luyện NNUE quy mô lớn trên Google Colab.

# %% Cell 1: Cài đặt Rust Toolchain (~2 phút)
import os
import subprocess
import time

print("=" * 60)
print(" BƯỚC 1: CÀI ĐẶT RUST TOOLCHAIN")
print("=" * 60)

start = time.time()

# Cài đặt Rust qua rustup (không tương tác)
subprocess.run(
    "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y",
    shell=True, check=True
)

# Thêm Cargo vào PATH
os.environ["PATH"] += ":/root/.cargo/bin"

# Xác minh cài đặt
result = subprocess.run(
    "/root/.cargo/bin/rustc --version",
    shell=True, capture_output=True, text=True
)
print(f"Rust: {result.stdout.strip()}")

result = subprocess.run(
    "/root/.cargo/bin/cargo --version",
    shell=True, capture_output=True, text=True
)
print(f"Cargo: {result.stdout.strip()}")

elapsed = time.time() - start
print(f"\n✅ Rust cài đặt thành công trong {elapsed:.0f} giây")

# %% Cell 2: Clone Repo + Build Release (~3-5 phút)
print("=" * 60)
print(" BƯỚC 2: CLONE REPO & BUILD RELEASE")
print("=" * 60)

start = time.time()

# Clone repository
if not os.path.exists("xiangqi-rim"):
    subprocess.run(
        "git clone https://github.com/hoduyquocbao/xiangqi-rim.git",
        shell=True, check=True
    )
else:
    print("Repository đã tồn tại, pull latest...")
    subprocess.run("cd xiangqi-rim && git pull", shell=True, check=True)

# Build release binary
os.chdir("xiangqi-rim")
subprocess.run(
    "/root/.cargo/bin/cargo build --release --example 20_parallel_mine",
    shell=True, check=True
)

elapsed = time.time() - start
print(f"\n✅ Build thành công trong {elapsed:.0f} giây")

# Kiểm tra binary
result = subprocess.run(
    "ls -lh target/release/examples/20_parallel_mine",
    shell=True, capture_output=True, text=True
)
print(f"Binary: {result.stdout.strip()}")

# %% Cell 3: Kiểm tra Phần cứng Colab
print("=" * 60)
print(" BƯỚC 3: KIỂM TRA PHẦN CỨNG COLAB")
print("=" * 60)

# CPU
result = subprocess.run("nproc", shell=True, capture_output=True, text=True)
cores = int(result.stdout.strip())
print(f"CPU Cores: {cores}")

# RAM
result = subprocess.run(
    "free -h | grep Mem | awk '{print $2}'",
    shell=True, capture_output=True, text=True
)
print(f"RAM: {result.stdout.strip()}")

# GPU
result = subprocess.run(
    "nvidia-smi --query-gpu=name,memory.total --format=csv,noheader 2>/dev/null || echo 'Không có GPU'",
    shell=True, capture_output=True, text=True
)
print(f"GPU: {result.stdout.strip()}")

# Disk
result = subprocess.run(
    "df -h / | tail -1 | awk '{print $4}'",
    shell=True, capture_output=True, text=True
)
print(f"Disk trống: {result.stdout.strip()}")

# Đề xuất THREADS
physical = max(1, cores // 2)
print(f"\nĐề xuất THREADS={physical} (physical cores)")

# %% Cell 4: CẤU HÌNH MINING
# ============================================================
# ⚠️ SỬA CÁC GIÁ TRỊ SAU THEO TỪNG COLAB INSTANCE
# ============================================================

GAMES = 25000       # Số ván cờ (25000 × 4 instances = 100000)
DEPTH = 4           # Độ sâu search
SEED = 1            # Base seed (1, 2, 3, 4 cho mỗi instance)
THREADS = 2         # Số threads (Colab Free = 2, Pro High-RAM = 4)
OUTPUT = "data/gen6_colab_part1.jsonl"  # Tên file output

print("=" * 60)
print(" CẤU HÌNH MINING")
print("=" * 60)
print(f"  GAMES   = {GAMES}")
print(f"  DEPTH   = {DEPTH}")
print(f"  SEED    = {SEED}")
print(f"  THREADS = {THREADS}")
print(f"  OUTPUT  = {OUTPUT}")

# Ước tính thời gian (dựa trên benchmark: 2.0 ván/s @ 4 threads i5-8259U)
# Colab 2 cores Xeon ước tính ~1.2 ván/s
speed = 1.2 * (THREADS / 2)
eta = GAMES / speed / 3600
print(f"\n  Tốc độ ước tính: ~{speed:.1f} ván/s")
print(f"  ETA: ~{eta:.1f} giờ")
print(f"  Mẫu dự kiến: ~{GAMES * 180:,}")

# %% Cell 5: CHẠY MINING
print("=" * 60)
print(" BƯỚC 5: BẮT ĐẦU MINING")
print("=" * 60)

start = time.time()

# Tạo thư mục data nếu chưa có
os.makedirs("data", exist_ok=True)

# Chạy mining
env = os.environ.copy()
env["GAMES"] = str(GAMES)
env["DEPTH"] = str(DEPTH)
env["SEED"] = str(SEED)
env["THREADS"] = str(THREADS)
env["OUTPUT"] = OUTPUT

process = subprocess.Popen(
    ["./target/release/examples/20_parallel_mine"],
    env=env,
    stdout=subprocess.PIPE,
    stderr=subprocess.STDOUT,
    text=True,
    bufsize=1
)

# Stream output real-time
for line in process.stdout:
    print(line, end="", flush=True)

code = process.wait()
elapsed = time.time() - start

if code == 0:
    print(f"\n✅ Mining hoàn tất trong {elapsed:.0f} giây ({elapsed/3600:.1f} giờ)")
else:
    print(f"\n❌ Mining thất bại với exit code {code}")

# %% Cell 6: XÁC MINH KẾT QUẢ
import json

print("=" * 60)
print(" BƯỚC 6: XÁC MINH KẾT QUẢ")
print("=" * 60)

if os.path.exists(OUTPUT):
    # Đếm dòng
    count = 0
    scores = []
    with open(OUTPUT, "r") as f:
        for line in f:
            line = line.strip()
            if line:
                count += 1
                if count <= 100000:
                    try:
                        d = json.loads(line)
                        scores.append(d.get("score", 0))
                    except Exception:
                        pass

    size = os.path.getsize(OUTPUT)
    print(f"  Tổng mẫu: {count:,}")
    print(f"  File size: {size / (1024*1024):.1f} MB")

    if scores:
        print(f"  Score range: [{min(scores)}, {max(scores)}]")
        print(f"  Score mean: {sum(scores)/len(scores):.1f}")

    # Kiểm tra mẫu đầu
    with open(OUTPUT, "r") as f:
        first = json.loads(f.readline())
        print(f"  Fields: {list(first.keys())}")
        print(f"  Sample: {json.dumps(first)}")
else:
    print(f"  ❌ File {OUTPUT} không tồn tại!")

# %% Cell 7: UPLOAD LÊN GOOGLE DRIVE
from google.colab import drive

print("=" * 60)
print(" BƯỚC 7: UPLOAD LÊN GOOGLE DRIVE")
print("=" * 60)

# Mount Google Drive
drive.mount("/content/drive")

# Tạo thư mục đích
target = "/content/drive/MyDrive/xiangqi-mining"
os.makedirs(target, exist_ok=True)

# Copy file
import shutil
destination = os.path.join(target, os.path.basename(OUTPUT))
shutil.copy2(OUTPUT, destination)

size = os.path.getsize(destination)
print(f"✅ Đã upload {destination} ({size/(1024*1024):.1f} MB)")
print(f"   Tổng mẫu: {count:,}")

# %% Cell 8 (TÙY CHỌN): UPLOAD TRỰC TIẾP LÊN HUGGINGFACE
# ============================================================
# Chỉ chạy cell này NẾU muốn upload trực tiếp lên HuggingFace
# thay vì qua Google Drive
# ============================================================

# HF_TOKEN = "hf_YOUR_TOKEN_HERE"  # Thay bằng token thật
# REPO = "hoduyquocbao/xiangqi-r1-dataset"
#
# subprocess.run("pip install huggingface_hub -q", shell=True, check=True)
#
# from huggingface_hub import HfApi
# api = HfApi(token=HF_TOKEN)
# api.upload_file(
#     path_or_fileobj=OUTPUT,
#     path_in_repo=os.path.basename(OUTPUT),
#     repo_id=REPO,
#     repo_type="dataset",
#     commit_message=f"feat: Colab mining SEED={SEED} GAMES={GAMES} DEPTH={DEPTH}"
# )
# print(f"✅ Uploaded to HuggingFace: {REPO}")
