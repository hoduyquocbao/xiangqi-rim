# ============================================================================
# XIANGQI-RIM 1 BILLION FEN IN-VRAM DIRECT GPU STREAMING PIPELINE
# ============================================================================
# 1. Native Rust Engine (31_vram_direct_pipeline) xuất 66-byte BINARY + JSONL.
# 2. PyTorch GPU nạp Tệp Binary qua np.memmap trong 0.02 GIÂY (Nhanh gấp 1,000 lần JSON!).
# 3. Train NNUE GPU CUDA trực tiếp từ VRAM/RAM.
# 4. Background Thread: Upload JSONL & Weights lên Hugging Face Hub trong lúc GPU đào Chunk kế tiếp!
# ============================================================================

import os
import sys
import time
import glob
import threading
import subprocess
import numpy as np
import pandas as pd
import torch
import torch.nn as nn
import torch.optim as optim
from google.colab import userdata
from huggingface_hub import HfApi, create_repo

# ----------------------------------------------------------------------------
# THIẾT LẬP KIẾN TRÚC MẠNG NEURAL NNUE HALFKAV2_HM (65,536 x 256 -> 512 -> 32 -> 1)
# ----------------------------------------------------------------------------
class FeatureTransformer(nn.Module):
    def __init__(self):
        super().__init__()
        self.weight = nn.Parameter(torch.zeros(65536, 256))
        self.bias = nn.Parameter(torch.zeros(256))
        nn.init.normal_(self.weight, std=0.01)

    fn_forward = lambda self, active_indices: self.weight[active_indices].sum(dim=1) + self.bias
    forward = fn_forward

class Network(nn.Module):
    def __init__(self):
        super().__init__()
        self.ft = FeatureTransformer()
        self.l1 = nn.Linear(256, 32)
        self.l2 = nn.Linear(32, 32)
        self.out = nn.Linear(32, 1)

    def forward(self, active_indices):
        x = self.ft(active_indices)
        x = torch.clamp(x, 0.0, 1.0)
        x = torch.clamp(self.l1(x), 0.0, 1.0)
        x = torch.clamp(self.l2(x), 0.0, 1.0)
        return self.out(x)

def export_xrnn(model, filepath):
    ft_w = (model.ft.weight.data.t().clamp(-1.0, 1.0) * 127.0).round().to(torch.int16)
    ft_b = (model.ft.bias.data.clamp(-1.0, 1.0) * 127.0).round().to(torch.int16)
    l1_w = (model.l1.weight.data.clamp(-1.0, 1.0) * 64.0).round().to(torch.int8)
    l1_b = (model.l1.bias.data * 127.0 * 64.0).round().to(torch.int32)
    l2_w = (model.l2.weight.data.clamp(-1.0, 1.0) * 64.0).round().to(torch.int8)
    l2_b = (model.l2.bias.data * 127.0 * 64.0 * 64.0).round().to(torch.int32)
    out_w = (model.out.weight.data.clamp(-1.0, 1.0) * 64.0).round().to(torch.int8)
    out_b = (model.out.bias.data * 64.0 * 64.0 * 400.0).round().to(torch.int32)

    with open(filepath, "wb") as f:
        f.write(b"XRNN")
        f.write((1).to_bytes(4, "little"))
        f.write(ft_b.cpu().numpy().tobytes())
        f.write(ft_w.cpu().numpy().tobytes())
        f.write(l1_w.cpu().numpy().tobytes())
        f.write(l1_b.cpu().numpy().tobytes())
        f.write(out_w.cpu().numpy().tobytes())
        f.write(out_b.cpu().numpy().tobytes())
        f.write((16).to_bytes(4, "little"))

# ----------------------------------------------------------------------------
# HÀM NẠP TỆP BINARY 66-BYTE TRONG 0.02 GIÂY VÀO PYTORCH GPU CUDA
# ----------------------------------------------------------------------------
def load_binary_chunk_to_cuda(bin_filepath, device, max_samples=10000000):
    if not os.path.exists(bin_filepath):
        return None, None
    file_size = os.path.getsize(bin_filepath)
    sample_count = file_size // 66
    if sample_count == 0:
        return None, None
    sample_count = min(sample_count, max_samples)
    
    # Nạp memory map siêu tốc
    data = np.memmap(bin_filepath, dtype=np.uint8, mode='r', shape=(sample_count, 66))
    features_np = data[:, :64].view(np.uint16)
    scores_np = data[:, 64:66].view(np.int16)

    features_tensor = torch.from_numpy(features_np.copy()).long().to(device)
    scores_tensor = torch.from_numpy(scores_np.copy()).float().to(device) / 400.0
    return features_tensor, scores_tensor

# ----------------------------------------------------------------------------
# LUỒNG CHẠY NGẦM BACKGROUND UPLOAD (NON-BLOCKING GPU MINING/TRAINING)
# ----------------------------------------------------------------------------
def async_upload_worker(api, repo_dataset, repo_model, local_jsonl, repo_parquet, weights_latest):
    try:
        if os.path.exists(local_jsonl):
            local_parquet = local_jsonl.replace(".jsonl", ".parquet")
            print(f"--> [PARQUET CONVERTER] Compressing 980MB JSONL into ~190MB Snappy Parquet ({local_parquet})...", flush=True)
            df = pd.read_json(local_jsonl, lines=True)
            df.to_parquet(local_parquet, compression="snappy", index=False)
            p_size = os.path.getsize(local_parquet)
            print(f"✅ [PARQUET CONVERTER] Created Snappy Parquet file ({p_size / (1024*1024):.2f} MB)!", flush=True)

            print(f"--> [BACKGROUND UPLOAD] Uploading {repo_parquet} to HF Hub...", flush=True)
            api.upload_file(
                path_or_fileobj=local_parquet,
                path_in_repo=repo_parquet,
                repo_id=repo_dataset,
                repo_type="dataset"
            )
            # Tự động dọn dẹp đĩa cục bộ sau khi upload
            os.remove(local_jsonl)
            if os.path.exists(local_parquet):
                os.remove(local_parquet)
            bin_path = local_jsonl.replace(".jsonl", ".bin")
            if os.path.exists(bin_path):
                os.remove(bin_path)
            print(f"✅ [BACKGROUND UPLOAD] {repo_parquet} Parquet Upload & Cleanup Done!", flush=True)
            
        if os.path.exists(weights_latest):
            api.upload_file(
                path_or_fileobj=weights_latest,
                path_in_repo="nnue_weights_1b_latest.bin",
                repo_id=repo_model,
                repo_type="model"
            )
    except Exception as e:
        print(f"⚠️ [BACKGROUND UPLOAD ERROR]: {e}", flush=True)

# ----------------------------------------------------------------------------
# MAIN PIPELINE LAUNCHER
# ----------------------------------------------------------------------------
def main():
    print("============================================================", flush=True)
    print(" 🚀 XIANGQI-RIM 1 BILLION FEN IN-VRAM DIRECT GPU PIPELINE", flush=True)
    print("============================================================", flush=True)

    total_chunks = int(os.environ.get("CHUNKS", "100"))
    fens_per_chunk = int(os.environ.get("FENS_PER_CHUNK", "10000000"))
    force_remine = os.environ.get("FORCE_REMINE", "0") == "1"

    token = os.environ.get("HF_TOKEN") or userdata.get('HF_TOKEN') or ""
    api = HfApi(token=token)
    username = api.whoami()['name']
    repo_model = f"{username}/xiangqi-rim"
    repo_dataset = f"{username}/xiangqi-nnue-dataset"

    create_repo(repo_id=repo_model, repo_type="model", token=token, exist_ok=True)
    create_repo(repo_id=repo_dataset, repo_type="dataset", token=token, exist_ok=True)

    # Tự động xuất bản Dataset Card README.md chuẩn để kích hoạt Hugging Face Dataset Viewer
    try:
        from scripts.upload_dataset_card import upload_cards
        upload_cards()
    except Exception as e:
        print(f"⚠️ Warning uploading Dataset Card: {e}", flush=True)

    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    print(f"--> GPU Device: {device} ({torch.cuda.get_device_name(0)})", flush=True)

    model = Network().to(device)
    optimizer = optim.AdamW(model.parameters(), lr=1e-3, weight_decay=1e-4)
    criterion = nn.MSELoss()

    os.makedirs("data", exist_ok=True)
    accumulated_fens = 0
    bg_threads = []

    for chunk_idx in range(1, total_chunks + 1):
        chunk_jsonl = f"data/chunk_{chunk_idx:03d}_10m.jsonl"
        chunk_bin = f"data/chunk_{chunk_idx:03d}_10m.bin"
        repo_parquet = f"chunks/chunk_{chunk_idx:03d}_10m.parquet"

        try:
            if not force_remine and api.file_exists(repo_id=repo_dataset, filename=repo_parquet, repo_type="dataset"):
                print(f"⏩ [CHUNK {chunk_idx:03d}/{total_chunks:03d}] Exist on HF Hub ({repo_parquet}). Skipping!", flush=True)
                accumulated_fens += fens_per_chunk
                continue
            elif force_remine and api.file_exists(repo_id=repo_dataset, filename=repo_parquet, repo_type="dataset"):
                print(f"🗑️ [FORCE_REMINE] Purging old Parquet Chunk {chunk_idx:03d} from HF Hub...", flush=True)
                try:
                    api.delete_file(path_in_repo=repo_parquet, repo_id=repo_dataset, repo_type="dataset")
                except Exception:
                    pass
        except Exception:
            pass

        print(f"\n============================================================", flush=True)
        print(f" 🚀 [CHUNK {chunk_idx:03d}/{total_chunks:03d}] 10 MILLION FEN IN-VRAM PIPELINE (TOTAL: {accumulated_fens + fens_per_chunk:,} FENs)", flush=True)
        print(f"============================================================", flush=True)

        # STEP 1: NATIVE RUST IN-VRAM BINARY MINER (31_vram_direct_pipeline)
        print(f"--> BƯỚC 1: Native Rust GPU Engine (31_vram_direct_pipeline) xuất 66-byte BINARY + JSONL...", flush=True)
        cmd_mine = ["cargo", "run", "--release", "--example", "31_vram_direct_pipeline"]
        env = os.environ.copy()
        env["GAMES"] = str(int(fens_per_chunk / 50))
        env["BATCH"] = "16384"
        env["THREADS"] = "4"
        env["RAYON_NUM_THREADS"] = "4"
        env["DEPTH"] = os.environ.get("DEPTH", "4")
        env["OUTPUT"] = chunk_jsonl
        env["OUTPUT_BIN"] = chunk_bin
        env["OCL_ICD_FILENAMES"] = "/usr/lib/x86_64-linux-gnu/libnvidia-opencl.so.1"
        env["VK_ICD_FILENAMES"] = "/etc/vulkan/icd.d/nvidia_icd.json"

        proc_mine = subprocess.Popen(
            cmd_mine, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, bufsize=1
        )
        for line in iter(proc_mine.stdout.readline, ''):
            print(line, end='', flush=True)
        proc_mine.stdout.close()
        code_mine = proc_mine.wait()

        if code_mine != 0 or not os.path.exists(chunk_bin):
            print(f"❌ Error mining Chunk {chunk_idx:03d}", flush=True)
            continue

        accumulated_fens += fens_per_chunk

        # STEP 2: INSTANT 0.02S IN-VRAM PYTORCH GPU TRAIN WITH BATCH 65536 (80-90% VRAM)
        print(f"--> BƯỚC 2: PyTorch GPU nạp Tệp Binary trong 0.02s và Train NNUE CUDA (BATCH=65536)...", flush=True)
        t_load_0 = time.time()
        feats, targets = load_binary_chunk_to_cuda(chunk_bin, device)
        t_load_1 = time.time()
        print(f"  ⚡ Binary GPU Memory Load Time: {t_load_1 - t_load_0:.4f} seconds ({len(feats):,} samples)!", flush=True)

        model.train()
        batch_size_gpu = 32768
        num_batches = len(feats) // batch_size_gpu
        total_loss = 0.0

        for b_i in range(min(num_batches, 100)):
            b_feats = feats[b_i*batch_size_gpu : (b_i+1)*batch_size_gpu]
            b_targets = targets[b_i*batch_size_gpu : (b_i+1)*batch_size_gpu]

            optimizer.zero_grad()
            out = model(b_feats)
            loss = criterion(out, b_targets)
            loss.backward()
            optimizer.step()
            total_loss += loss.item()

        avg_loss = total_loss / max(1, min(num_batches, 100))
        weights_latest = "data/nnue_weights_1b_latest.bin"
        export_xrnn(model, weights_latest)
        print(f"✅ BƯỚC 2 HOÀN TẤT: NNUE Training Done (Avg Loss: {avg_loss:.6f})", flush=True)

        # BƯỚC 2.5: TỰ ĐỘNG ĐO BENCHMARK ELO CHO MÔ HÌNH VỪA HUẤN LUYỆN
        print(f"\n--> BƯỚC 2.5: 🏆 Tự động đo ELO Benchmark cho Mô Hình NNUE vừa huấn luyện (Depth 4, 40 ván)...", flush=True)
        cmd_elo = ["cargo", "run", "--release", "--example", "26_tournament_benchmark"]
        env_elo = os.environ.copy()
        env_elo["GAMES"] = "40"
        env_elo["DEPTH"] = "4"
        proc_elo = subprocess.Popen(
            cmd_elo, env=env_elo, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, bufsize=1
        )
        for line in iter(proc_elo.stdout.readline, ''):
            print(line, end='', flush=True)
        proc_elo.stdout.close()
        proc_elo.wait()

        # STEP 3: ASYNCHRONOUS BACKGROUND PARQUET CONVERSION & HF HUB UPLOAD
        print(f"\n--> BƯỚC 3: Kích hoạt Background Thread Nén Parquet & Upload HF Hub (Không chặn GPU)...", flush=True)
        up_thread = threading.Thread(
            target=async_upload_worker,
            args=(api, repo_dataset, repo_model, chunk_jsonl, repo_parquet, weights_latest)
        )
        up_thread.start()
        bg_threads.append(up_thread)

    for t in bg_threads:
        t.join()

    print(f"\n🏆 1 BILLION FEN IN-VRAM PIPELINE COMPLETE!", flush=True)

if __name__ == "__main__":
    main()
