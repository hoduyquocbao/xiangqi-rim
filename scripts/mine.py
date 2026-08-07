# scripts/mine.py
# ============================================================================
# TỰ ĐỘNG KHAI THÁC DỮ LIỆU CỜ TƯỚNG TỪ NATIVE RUST ENGINE VÀ ĐẨY HUGGINGFACE
# ============================================================================
# Định danh đơn từ tiếng Anh: token, repo, cmd, proc, path, file, data,
# content, payload, req, res, err, mine, loop, count, push, delay, api
# ============================================================================

import subprocess
import json
import glob
import os
import time
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

def run_rust_miner():
    print("🚀 Đang chạy Native Rust Engine Self-Play Miner...")
    cmd = ["cargo", "run", "--release", "--example", "17_mine_dataset"]
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True, check=True)
        print(proc.stdout)
        return True
    except Exception as err:
        print(f"❌ Lỗi chạy Rust Engine: {err}")
        return False

def push_all_mined_files():
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

    print(f"📊 Tổng hợp thành công {len(all_samples):,} mẫu cờ độc bản từ {len(files)} tệp mined!")

    jsonl_lines = [json.dumps(s, ensure_ascii=False) for s in all_samples]
    with open("data/train.jsonl", "w", encoding="utf-8") as f:
        f.write("\n".join(jsonl_lines))

    with open("data/train.json", "w", encoding="utf-8") as f:
        json.dump(all_samples, f, ensure_ascii=False, indent=2)

    with open("data/README.md", "w", encoding="utf-8") as f:
        f.write(build_readme(total_samples=len(all_samples)))

    if not token:
        print("⚠️ Thiếu HF_TOKEN. Bỏ qua bước đẩy Hugging Face Hub.")
        return True

    api = HfApi(token=token)
    print(f"📤 Đang đẩy dataset {len(all_samples):,} mẫu cờ lên HuggingFace Hub ({repo})...")
    api.upload_file(path_or_fileobj="data/train.jsonl", path_in_repo="train.jsonl", repo_id=repo, repo_type="dataset")
    api.upload_file(path_or_fileobj="data/train.json", path_in_repo="train.json", repo_id=repo, repo_type="dataset")
    api.upload_file(path_or_fileobj="data/README.md", path_in_repo="README.md", repo_id=repo, repo_type="dataset")

    verified_files = api.list_repo_files(repo_id=repo, repo_type="dataset")
    print(f"============================================================")
    print(f"✅ ĐÃ ĐẨY THÀNH CÔNG {len(all_samples):,} MẪU CỜ LÊN HUGGINGFACE HUB!")
    print(f"🔗 URL: https://huggingface.co/datasets/{repo}")
    print(f"📂 Tree: https://huggingface.co/datasets/{repo}/tree/main")
    print("Các tệp trên Hub:", verified_files)
    print(f"============================================================")
    return True

def mine(limit=10):
    print("============================================================")
    print(" BẮT ĐẦU KHAI THÁC DỮ LIỆU QUY MÔ LỚN TỪ RUST ENGINE & ĐẨY HUB ")
    print("============================================================")
    for loop in range(1, limit + 1):
        print(f"\n🔄 Đợt khai thác #{loop}/{limit}...")
        if run_rust_miner():
            push_all_mined_files()
        time.sleep(1)

if __name__ == "__main__":
    count = int(os.environ.get("MINING_LOOPS", "10"))
    mine(limit=count)
