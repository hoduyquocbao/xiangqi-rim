# scripts/deploy_dataset.py
# ============================================================================
# TRIỂN KHAI DỮ LIỆU CỜ TƯỚNG TƯ DUY SÂU R1 CHUẨN LÊN HUGGINGFACE DATASETS HUB
# ============================================================================
# Định danh đơn từ tiếng Anh: token, repo, cmd, proc, path, file, data,
# content, payload, req, res, err, deploy, commit, push, card, api, total,
# local, remote, merged, added, batch, readme, files
# ============================================================================

import subprocess
import json
import glob
import os
from huggingface_hub import HfApi
try:
    from scripts.hub import fetch, verify, merge, save, push
except ImportError:
    from hub import fetch, verify, merge, save, push

token = os.environ.get("HF_TOKEN", "")
repo = "hoduyquocbao/xiangqi-r1-dataset"

def readme(total=0):
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

- **Tổng số mẫu cờ tư duy sâu hiện tại**: {total:,} mẫu.

## 📊 Cấu Trúc Dữ Liệu Đa Chiều (Multi-Modal Data Schema)

Mỗi mẫu dữ liệu chứa biểu diễn 3 chiều đầy đủ:
1. **`Ma Trận Bàn Cờ 2D (9x10)`**: Biểu diễn văn bản trực quan 9x10 các quân cờ Đỏ (In hoa) & Đen (In thường).
2. **`Chuỗi Chuẩn FEN (Forsyth-Edwards Notation)`**: Định dạng FEN chuẩn của động cơ cờ quốc tế.
3. **`Lịch Sử Nước Đi PGN (Move History)`**: Chuỗi các nước đi lịch sử từ đầu ván đấu.

- **`prompt`**: Trạng thái bàn cờ đa chiều (Ma trận 2D + FEN + PGN) kèm yêu cầu suy nghĩ trong thẻ `<thought>`.
- **`completion`**: Chuỗi suy luận sâu 4 bước chuẩn DeepSeek-R1 (Phân tích FEN, PGN, Tướng & Chiến thuật) kèm nước đi UCI cuối cùng.
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

    local = []
    for path in files:
        try:
            with open(path, "r", encoding="utf-8") as f:
                batch = json.load(f)
                for item in batch:
                    if verify(item):
                        local.append(item)
        except Exception as e:
            print(f"⚠️ Lỗi đọc tệp {path}: {e}")

    print(f"📊 Đã đọc được {len(local):,} mẫu cờ mới từ local!")

    # 1. Pull remote dataset hiện tại từ Hub (Nếu có)
    remote = fetch(repo=repo, token=token, filename="train.jsonl")

    # 2. Merge & Deduplicate
    merged, added = merge(remote=remote, local=local)

    # 3. Ghi dữ liệu nguyên tử ra đĩa cục bộ
    card = readme(total=len(merged))
    save(samples=merged, card=card)

    # 4. Push không phá hủy lên Hub
    return push(repo=repo, token=token, retries=3)

if __name__ == "__main__":
    deploy()
