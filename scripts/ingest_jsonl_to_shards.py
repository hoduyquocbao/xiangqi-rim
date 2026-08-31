# scripts/ingest_jsonl_to_shards.py
# ==============================================================================
# KỊCH BẢN NẠP DỮ LIỆU JSONL VÀO KHO SHARDS NHỊ PHÂN O(1) & PERPETUAL VAULT
# ==============================================================================
# 100% chú thích tiếng Việt & 100% định danh từ đơn tiếng Anh (Single-Word).
# ==============================================================================

import os
import sys
import subprocess
from pathlib import Path

def main():
    if len(sys.argv) < 2:
        print("ℹ️ Cú pháp sử dụng: python3 scripts/ingest_jsonl_to_shards.py <path_to_jsonl>")
        print("   Ví dụ: python3 scripts/ingest_jsonl_to_shards.py data/selfplay_samples.jsonl")
        sys.exit(1)

    jsonl_path = sys.argv[1]
    if not os.path.exists(jsonl_path):
        print(f"❌ Lỗi: Tệp `{jsonl_path}` không tồn tại!")
        sys.exit(1)

    print(f"🚀 Đang kích hoạt Example 140 để nạp `{jsonl_path}` vào 1,024 Shards O(1)...")
    cmd = [
        "cargo", "run", "--release", "--example", "140_perpetual_tt_two_way_sync",
        "--", "--ingest", jsonl_path
    ]
    subprocess.run(cmd, check=True)

if __name__ == "__main__":
    main()
