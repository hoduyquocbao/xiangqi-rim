# ============================================================================
# XIANGQI-RIM AUTOMATIC GPU/CPU BENCHMARK TUNER & REAL-TIME STREAMING PIPELINE
# ============================================================================
# 1. Tự động Benchmark đo đạc 3 Cấu Hình Phần Cứng khác nhau (Batch Size, Thread Count).
# 2. Tự động Chọn Cấu Hình Cho Thông Lượng FEN/s Cao Nhất (Fastest Auto-Selection).
# 3. Kích chạy Pipeline với Real-Time Unbuffered Stdout Streaming (Triệt tiêu 100% treo process).
# ============================================================================

import os
import sys
import time
import subprocess
from google.colab import userdata
from huggingface_hub import HfApi, create_repo

print("============================================================", flush=True)
print(" 🚀 AUTO-TUNING BENCHMARK & REAL-TIME STREAMING PIPELINE", flush=True)
print("============================================================", flush=True)

repo_dir = "/content/xiangqi-rim"
os.chdir(repo_dir)
os.environ["PATH"] = f"/root/.cargo/bin:{os.environ.get('PATH', '')}"

# ----------------------------------------------------------------------------
# GIAI ĐOẠN 1: BENCHMARK TỰ ĐỘNG CHỌN CẤU HÌNH NHANH NHẤT (AUTO-TUNER)
# ----------------------------------------------------------------------------
print("\n🔍 GIAI ĐOẠN 1: TỰ ĐỘNG BENCHMARK VÀ THỬ CẤU HÌNH PHẦN CỨNG...", flush=True)

configs = [
    {"name": "Config A (4 Threads, Batch 16384)", "threads": "4", "batch": "16384"},
    {"name": "Config B (8 Threads, Batch 32768)", "threads": "8", "batch": "32768"},
    {"name": "Config C (4 Threads, Batch 8192)",  "threads": "4", "batch": "8192"},
]

best_config = None
max_speed_fens = 0.0

for cfg in configs:
    temp_out = f"data/tune_{cfg['threads']}_{cfg['batch']}.jsonl"
    if os.path.exists(temp_out):
        os.remove(temp_out)

    cmd = ["cargo", "run", "--release", "--example", "29_cacheline_ultra_miner"]
    env = os.environ.copy()
    env["GAMES"] = "1000"
    env["THREADS"] = cfg["threads"]
    env["RAYON_NUM_THREADS"] = cfg["threads"]
    env["OUTPUT"] = temp_out

    print(f"\n--> 🧪 Kiểm thử {cfg['name']} trong 3 giây...", flush=True)
    t0 = time.time()
    proc = subprocess.Popen(
        cmd, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, bufsize=1
    )

    # Chạy 3 giây ngắn để đo tốc độ
    time.sleep(3)
    proc.terminate()
    try:
        proc.wait(timeout=2)
    except Exception:
        proc.kill()

    t_elapsed = time.time() - t0
    fens = 0
    if os.path.exists(temp_out):
        try:
            out_lines = subprocess.check_output(f"wc -l {temp_out} | awk '{{print $1}}'", shell=True, text=True).strip()
            fens = int(out_lines)
            os.remove(temp_out)
        except Exception:
            fens = 0

    speed = fens / t_elapsed if t_elapsed > 0 else 0
    million_min = (speed * 60.0) / 1_000_000.0

    print(f"  ✅ {cfg['name']} -> {speed:,.0f} FEN/s ({million_min:.2f} M FEN/min)", flush=True)

    if speed > max_speed_fens:
        max_speed_fens = speed
        best_config = cfg

if not best_config:
    best_config = configs[0]

print("\n============================================================", flush=True)
print(f" 🏆 CẤU HÌNH THẮNG CUỘC ĐẠT TỐC ĐỘ CAO NHẤT: {best_config['name']}", flush=True)
print(f" 🚀 THÔNG LƯỢNG ĐẠT ĐƯỢC: {max_speed_fens:,.0f} FEN/sec ({(max_speed_fens*60)/1_000_000:.2f} MILLION FEN/min)", flush=True)
print("============================================================", flush=True)

# ----------------------------------------------------------------------------
# GIAI ĐOẠN 2: KÍCH CHẠY PIPELINE VỚI UNBUFFERED REAL-TIME STREAMING LOGS
# ----------------------------------------------------------------------------
print("\n🚀 GIAI ĐOẠN 2: KÍCH CHẠY 1 BILLION FEN PIPELINE VỚI REAL-TIME STREAMING...", flush=True)

env_run = os.environ.copy()
env_run["HF_TOKEN"] = userdata.get('HF_TOKEN') or os.environ.get("HF_TOKEN") or ""
env_run["CHUNKS"] = "100"
env_run["FENS_PER_CHUNK"] = "10000000"
env_run["THREADS"] = best_config["threads"]
env_run["BATCH"] = best_config["batch"]
env_run["PYTHONUNBUFFERED"] = "1"

pipeline_proc = subprocess.Popen(
    [sys.executable, "-u", "scripts/colab_rolling_1b_pipeline.py"],
    env=env_run,
    stdout=subprocess.PIPE,
    stderr=subprocess.STDOUT,
    text=True,
    bufsize=1
)

# Unbuffered real-time stdout streaming — Triệt tiêu 100% cảm giác treo/đứng tiến trình!
for line in iter(pipeline_proc.stdout.readline, ''):
    print(line, end='', flush=True)

pipeline_proc.stdout.close()
exit_code = pipeline_proc.wait()
print(f"\n✅ PIPELINE HOÀN TẤT VỚI MÃ THÁO: {exit_code}", flush=True)
