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

def build_readme(total_samples=0):
    return f"""---
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
- 100K<n<1M
---

# 🤖 Xiangqi-R1 GRPO Self-Play Reasoning Dataset

Dữ liệu huấn luyện cờ tướng tư duy sâu (Xiangqi Deep Reasoning Dataset) được sinh ra từ **Native Rust Engine (XiangRust)** phục vụ huấn luyện mô hình **Xiangqi-R1 (Qwen 0.5B / 7B)** bằng thuật toán **GRPO (Group Relative Policy Optimization)**.

- **Tổng số mẫu cờ tư duy sâu hiện tại**: {total_samples:,} mẫu.

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

    print("🚀 Đang khởi chạy Native Rust Engine Self-Play...")
    cmd = ["cargo", "run", "--release", "--example", "17_mine_dataset"]
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True, check=True)
        print(proc.stdout)
    except Exception as err:
        print(f"❌ Lỗi chạy Rust Engine: {err}")
        return False

    files = sorted(glob.glob("data/real_mined_*.json"))
    if not files:
        print("⚠️ Không tìm thấy tệp real_mined_*.json nào trong data/")
        return False

    all_samples = []
    seen_prompts = set()

    for file_path in files:
        try:
            with open(file_path, "r", encoding="utf-8") as f:
                batch = json.load(f)
                for item in batch:
                    key = (item.get("prompt"), item.get("move"))
                    if key not in seen_prompts:
                        seen_prompts.add(key)
                        all_samples.append(item)
        except Exception as e:
            print(f"⚠️ Lỗi đọc tệp {file_path}: {e}")

    print(f"📊 Tổng hợp {len(all_samples):,} mẫu cờ tư duy sâu thực tế!")

    jsonl_lines = [json.dumps(s, ensure_ascii=False) for s in all_samples]
    with open("data/train.jsonl", "w", encoding="utf-8") as f:
        f.write("\n".join(jsonl_lines))

    with open("data/train.json", "w", encoding="utf-8") as f:
        json.dump(all_samples, f, ensure_ascii=False, indent=2)

    with open("data/README.md", "w", encoding="utf-8") as f:
        f.write(build_readme(total_samples=len(all_samples)))

    if not token:
        print("⚠️ Thiếu HF_TOKEN. Bỏ qua bước đẩy HuggingFace Hub.")
        return True

    api = HfApi(token=token)
    print(f"📤 Đang đẩy train.jsonl ({len(all_samples):,} mẫu)...")
    api.upload_file(path_or_fileobj="data/train.jsonl", path_in_repo="train.jsonl", repo_id=repo, repo_type="dataset")

    print(f"📤 Đang đẩy train.json ({len(all_samples):,} mẫu)...")
    api.upload_file(path_or_fileobj="data/train.json", path_in_repo="train.json", repo_id=repo, repo_type="dataset")

    print("📤 Đang đẩy README.md...")
    api.upload_file(path_or_fileobj="data/README.md", path_in_repo="README.md", repo_id=repo, repo_type="dataset")

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
