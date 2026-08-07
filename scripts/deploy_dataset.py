# scripts/deploy_dataset.py
# ============================================================================
# TRIỂN KHAI DỮ LIỆU CỜ TƯỚNG TƯ DUY SÂU R1 CHUẨN LÊN HUGGINGFACE DATASETS HUB
# ============================================================================
# Định danh đơn từ tiếng Anh: token, repo, cmd, proc, path, file, data,
# content, payload, req, res, err, deploy, commit, push, card, api
# ============================================================================

import subprocess
import json
import glob
import os
from huggingface_hub import HfApi

token = os.environ.get("HF_TOKEN", "")
repo = "hoduyquocbao/xiangqi-r1-dataset"

def build_readme():
    return """---
license: mit
task_categories:
- reinforcement-learning
- text-generation
language:
- vi
- en
tags:
- xiangqi
- r1
- grpo
- chess
- reasoning
size_categories:
- 10K<n<100K
---

# 🤖 Xiangqi-R1 GRPO Self-Play Reasoning Dataset

Dữ liệu huấn luyện cờ tướng tư duy sâu (Xiangqi Deep Reasoning Dataset) được sinh ra từ **Native Rust Engine (XiangRust)** phục vụ huấn luyện mô hình **Xiangqi-R1 (Qwen 0.5B / 7B)** bằng thuật toán **GRPO (Group Relative Policy Optimization)**.

## 📊 Cấu Trúc Dữ Liệu (Data Schema)

Mỗi mẫu dữ liệu chứa:
- **`prompt`**: Trạng thái bàn cờ 2D 9x10 dạng ma trận FEN kèm yêu cầu suy nghĩ trong thẻ `<thought>`.
- **`completion`**: Chuỗi suy luận sâu 4 bước chuẩn DeepSeek-R1 kèm nước đi UCI cuối cùng.
- **`move`**: Nước đi đại số UCI 4 ký tự (ví dụ: `b2e2`, `h9g7`).
- **`stamp`**: Dấu thời gian Unix timestamp.

## 🚀 Cách Sử Dụng Trong Python / Unsloth:

```python
from datasets import load_dataset

dataset = load_dataset("hoduyquocbao/xiangqi-r1-dataset", split="train")
print("Tổng số mẫu:", len(dataset))
print(dataset[0])
```
"""

def deploy():
    print("============================================================")
    print(" CHẠY RUST ENGINE MINER & TRIỂN KHAI HUGGINGFACE DATASET ")
    print("============================================================")

    # 1. Chạy Rust Engine Miner 17 để tạo ván đấu tự động thật 100%
    print("🚀 Đang khởi chạy Native Rust Engine Self-Play...")
    cmd = ["cargo", "run", "--release", "--example", "17_mine_dataset"]
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True, check=True)
        print(proc.stdout)
    except Exception as err:
        print(f"❌ Lỗi chạy Rust Engine: {err}")
        return False

    files = sorted(glob.glob("data/real_mined_*.json"), key=os.path.getmtime, reverse=True)
    if not files:
        print("⚠️ Không tìm thấy tệp real_mined_*.json nào trong data/")
        return False

    latest_file = files[0]
    print(f"📄 Đã đọc tệp dữ liệu cờ thật: {latest_file}")
    with open(latest_file, "r", encoding="utf-8") as f:
        samples = json.load(f)

    # 2. Ghi ra train.jsonl, train.json và README.md local
    jsonl_lines = [json.dumps(s, ensure_ascii=False) for s in samples]
    with open("data/train.jsonl", "w", encoding="utf-8") as f:
        f.write("\n".join(jsonl_lines))

    with open("data/train.json", "w", encoding="utf-8") as f:
        json.dump(samples, f, ensure_ascii=False, indent=2)

    with open("data/README.md", "w", encoding="utf-8") as f:
        f.write(build_readme())

    # 3. Đẩy chính thức lên HuggingFace Hub bằng HfApi
    api = HfApi(token=token)

    print("📤 Đang đẩy train.jsonl...")
    api.upload_file(path_or_fileobj="data/train.jsonl", path_in_repo="train.jsonl", repo_id=repo, repo_type="dataset")

    print("📤 Đang đẩy train.json...")
    api.upload_file(path_or_fileobj="data/train.json", path_in_repo="train.json", repo_id=repo, repo_type="dataset")

    print("📤 Đang đẩy README.md...")
    api.upload_file(path_or_fileobj="data/README.md", path_in_repo="README.md", repo_id=repo, repo_type="dataset")

    # 4. Xác nhận danh sách tệp trực tiếp từ HuggingFace Hub
    verified_files = api.list_repo_files(repo_id=repo, repo_type="dataset")
    print("============================================================")
    print(f"✅ ĐÃ TRIỂN KHAI CHÍNH THỨC DỮ LIỆU CỜ TƯỚNG LÊN HUGGINGFACE HUB!")
    print(f"📦 Dataset URL: https://huggingface.co/datasets/{repo}")
    print(f"📂 Tree URL: https://huggingface.co/datasets/{repo}/tree/main")
    print("Danh sách tệp xác nhận trên Hub:")
    for vf in verified_files:
        print(f"  - {vf}")
    print("============================================================")
    return True

if __name__ == "__main__":
    deploy()
