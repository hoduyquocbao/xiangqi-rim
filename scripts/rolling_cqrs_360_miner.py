# ============================================================================
# KỊCH BẢN 35: BỘ ĐIỀU PHỐI CUỐN CHIẾU ROLLING CHUNK MINER 1.000.000 VÁN CỜ
# ============================================================================
# Kịch bản sản xuất chuẩn quốc tế cuốn chiếu (Mine Chunk -> Upload HuggingFace -> Purge Local).
# Giữ dung lượng ổ đĩa SSD MacBook / Colab luôn < 100 MB xuyên suốt tác vụ 1.000.000 ván cờ.
# Tự động hóa 100% việc xoay vòng chunk, đẩy lên HuggingFace Hub, và dọn dẹp đĩa tức thì.
# Tuân thủ 100% Quy tắc 8.14 (Rolling Chunks & HF Purge) và Quy tắc 8.10 (Realtime Yield).
# ============================================================================

import os  # Nhập thư viện os thao tác hệ thống tệp và biến môi trường
import sys  # Nhập thư viện sys tương tác với tham số dòng lệnh
import time  # Nhập thư viện time đo lường thời gian thực thi
import glob  # Nhập thư viện glob tìm kiếm tệp theo mẫu
import shutil  # Nhập thư viện shutil thao tác tệp và thư mục
import threading  # Nhập thư viện threading quản lý luồng ngầm bất đồng bộ
import subprocess  # Nhập thư viện subprocess điều khiển tiến trình Rust Engine
from pathlib import Path  # Nhập lớp Path xử lý đường dẫn hướng đối tượng

# ----------------------------------------------------------------------------
# 1. HẰNG SỐ PHIÊN BẢN VÀ CẤU HÌNH DÂY CHUYỀN
# ----------------------------------------------------------------------------
APP_VERSION = "v34.0.0-rolling-coordinator-1m-games"
APP_BUILD_STAMP = "2026-08-22 19:45:00 ICT"
DEFAULT_REPO = "hoduyquocbao/xiangqi-r1-360-reasoning-dataset"

try:
    from huggingface_hub import HfApi, create_repo
except ImportError:
    HfApi = None
    create_repo = None


def read_hf_token():
    """
    Hàm `read_hf_token`: Đọc token HuggingFace an toàn từ biến môi trường hoặc Colab userdata.
    """
    token = os.environ.get("HF_TOKEN") or os.environ.get("HUGGINGFACE_TOKEN")
    if not token:
        try:
            from google.colab import userdata
            token = userdata.get("HF_TOKEN") or userdata.get("HUGGINGFACE_TOKEN")
        except Exception:
            token = None
    return token


def ensure_hf_repo_and_readme(api, repo_id):
    """
    Hàm `ensure_hf_repo_and_readme`: Khởi tạo kho Dataset và cập nhật Dataset Card README.md trên HuggingFace Hub.
    """
    if not api:
        return
    try:
        api.create_repo(repo_id=repo_id, repo_type="dataset", exist_ok=True, private=False)
        print(f"📦 [HF HUB ENGINE] Đã xác thực/tạo kho chứa Dataset: https://huggingface.co/datasets/{repo_id}", flush=True)
    except Exception as e:
        print(f"⚠️ [HF HUB ENGINE] Ghi chú khởi tạo Repo: {e}", flush=True)

    readme_content = f"""---
license: apache-2.0
task_categories:
- reinforcement-learning
- text-generation
- conversational
language:
- vi
- en
tags:
- xiangqi
- chinese-chess
- deepseek-r1
- cot
- 360-reasoning
- cqrs-es
- tri-tier-pipeline
size_categories:
- 100K<n<1M
pretty_name: Xiangqi-R1 360-Degree DeepSeek-R1 Multi-Turn Dataset (1M Games)
---

# 🏯 Xiangqi-R1 360-Degree DeepSeek-R1 Full-Game Multi-Turn Dataset (1,000,000 Games)

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Dataset Version](https://img.shields.io/badge/Version-v34.0--Tri--Tier--Rolling-green.svg)](https://huggingface.co/datasets/{repo_id})
[![Task: DeepSeek-R1 RL Ready](https://img.shields.io/badge/Task-DeepSeek--R1_RL_Ready-purple.svg)](https://huggingface.co/datasets/{repo_id})

## 📌 TỔNG QUAN DỮ LIỆU
Tập dữ liệu tự đấu cờ Tướng quy mô lớn 1.000.000 ván cờ hoàn chỉnh với chuỗi suy tưởng **360 độ (14 chiều kích chiến thuật)** chuẩn DeepSeek-R1 `<thought>` được sinh bởi động cơ Rust **Tri-Tier Decoupled CQRS Pipeline Engine (v34.0)**.
"""
    try:
        readme_path = Path("README_DATASET_TEMP.md")
        with open(readme_path, "w", encoding="utf-8") as f_rm:
            f_rm.write(readme_content.strip())
        api.upload_file(
            path_or_fileobj=str(readme_path),
            path_in_repo="README.md",
            repo_id=repo_id,
            repo_type="dataset"
        )
        if readme_path.exists():
            readme_path.unlink()
        print(f"📝 [HF HUB ENGINE] Cập nhật thành công Dataset Card README.md trên HuggingFace ({repo_id})!", flush=True)
    except Exception as e:
        print(f"⚠️ [HF HUB ENGINE] Ghi chú cập nhật README: {e}", flush=True)


def async_upload_worker(api, chunk_file, repo_id, chunk_name):
    """
    Hàm `async_upload_worker`: Luồng ngầm tải tệp chunk lên HuggingFace Hub và dọn dẹp đĩa cục bộ.
    """
    if not api or not os.path.exists(chunk_file):
        return

    size_mb = os.path.getsize(chunk_file) / (1024 * 1024)
    cloud_path = f"data/{chunk_name}"
    print(f"\n📤 [ASYNC CLOUD SYNC] Đang tải ngầm Chunk `{chunk_name}` ({size_mb:.2f} MB) lên HuggingFace Hub...", flush=True)

    try:
        api.upload_file(
            path_or_fileobj=chunk_file,
            path_in_repo=cloud_path,
            repo_id=repo_id,
            repo_type="dataset"
        )
        print(f"✔ [ASYNC CLOUD SYNC] Đồng bộ thành công `{cloud_path}` lên HuggingFace Hub!", flush=True)

        if os.path.exists(chunk_file):
            os.remove(chunk_file)
            print(f"🧹 [LOCAL PURGE] Đã dọn dẹp đĩa `os.remove({chunk_file})` (Bảo toàn dung lượng SSD < 100MB).\n", flush=True)
    except Exception as e:
        print(f"⚠️ [ASYNC CLOUD SYNC] Lỗi tải lên `{chunk_name}`: {e}. Giữ tệp đĩa cục bộ để thử lại sau.", flush=True)


def main():
    print("===============================================================================")
    print(f"💎 XIANGQI-RIM: ROLLING CHUNK MINER COORDINATOR ({APP_VERSION})")
    print("   🔥 BỘ ĐIỀU PHỐI TỰ ĐỘNG CUỐN CHIẾU 1.000.000 VÁN CỜ (MINE ➔ SYNC ➔ PURGE)")
    print("===============================================================================")

    total_games = int(os.environ.get("GAMES", "1000000"))
    depth = int(os.environ.get("DEPTH", "4"))
    threads = int(os.environ.get("THREADS", "4"))
    transformers = int(os.environ.get("TRANSFORMERS", "4"))
    tt_mb = int(os.environ.get("TT_MB", "1024"))
    chunk_max_mb = float(os.environ.get("CHUNK_MAX_MB", "95.0"))
    dataset_repo = os.environ.get("DATASET_REPO", DEFAULT_REPO)
    chunks_dir = Path("data/chunks")
    chunks_dir.mkdir(parents=True, exist_ok=True)

    token = read_hf_token()
    api = HfApi(token=token) if (token and HfApi) else None

    if api:
        ensure_hf_repo_and_readme(api, dataset_repo)

    print(f"⚡ THÔNG SỐ ĐIỀU PHỐI DÂY CHUYỀN 1.000.000 VÁN:")
    print(f"   • Tổng số ván cờ mục tiêu     : {total_games:,} ván")
    print(f"   • Độ sâu tìm kiếm (Depth)     : {depth}")
    print(f"   • Luồng Tầng 1 (Producers)    : {threads} Threads")
    print(f"   • Luồng Tầng 2 (Transformers) : {transformers} Threads")
    print(f"   • Dung lượng Shared TT        : {tt_mb} MB")
    print(f"   • Giới hạn Chunk Dung lượng   : {chunk_max_mb:.1f} MB / Chunk (< 100 MB)")
    print(f"   • Thư mục lưu trữ Chunk       : {chunks_dir}")
    print(f"   • Kho Dataset HuggingFace Hub : https://huggingface.co/datasets/{dataset_repo}")
    print(f"   • Trạng thái HF Cloud Sync    : {'✅ ĐÃ KẾT NỐI (Tự động đẩy và xóa đĩa)' if api else '⚠️ KHÔNG CÓ TOKEN (Lưu trữ cục bộ)'}")
    print("-------------------------------------------------------------------------------\n")

    # Xây dựng câu lệnh thực thi ví dụ 95
    cmd = [
        "cargo", "run", "--release", "--example", "95_cqrs_360_reasoning_generator"
    ]

    env = os.environ.copy()
    env["GAMES"] = str(total_games)
    env["DEPTH"] = str(depth)
    env["THREADS"] = str(threads)
    env["TRANSFORMERS"] = str(transformers)
    env["TT_MB"] = str(tt_mb)
    env["CHUNK_MAX_MB"] = str(chunk_max_mb)
    env["OUTPUT"] = str(chunks_dir / "xiangqi_r1_360_dataset.jsonl")

    process = subprocess.Popen(
        cmd,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1
    )

    # Đọc log realtime và bắt sự kiện đóng chunk để upload ngầm
    for line in iter(process.stdout.readline, ""):
        print(line, end="", flush=True)

        if "📦 [ROLLING CHUNK ROTATION] Đóng Chunk" in line or "📦 [FINAL CHUNK FLUSHED] Chunk" in line:
            if "Đã lưu:" in line:
                saved_path = line.split("Đã lưu:")[-1].strip()
                if os.path.exists(saved_path):
                    chunk_name = os.path.basename(saved_path)
                    if api:
                        up_thread = threading.Thread(
                            target=async_upload_worker,
                            args=(api, saved_path, dataset_repo, chunk_name),
                            daemon=True
                        )
                        up_thread.start()

    process.stdout.close()
    return_code = process.wait()

    if return_code == 0:
        print("\n🎉 TOÀN BỘ QUY TRÌNH KHAI THÁC CUỐN CHIẾU 1.000.000 VÁN HOÀN TẤT!", flush=True)
    else:
        print(f"\n❌ Tiến trình kết thúc với mã lỗi: {return_code}", flush=True)


if __name__ == "__main__":
    main()
