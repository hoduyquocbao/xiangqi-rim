# scripts/dump_shards_to_jsonl.py
# ==============================================================================
# KỊCH BẢN TRÍCH XUẤT (DUMP) KHO SHARDS NHỊ PHÂN O(1) RA TỆP VĂN BẢN JSONL
# ==============================================================================
# 100% chú thích tiếng Việt & 100% định danh từ đơn tiếng Anh (Single-Word).
# ==============================================================================

import os
import sys
import subprocess
from pathlib import Path

def main():
    if len(sys.argv) < 2:
        print("ℹ️ Cú pháp sử dụng: python3 scripts/dump_shards_to_jsonl.py <output_path_jsonl>")
        print("   Ví dụ: python3 scripts/dump_shards_to_jsonl.py data/dumped_shards.jsonl")
        sys.exit(1)

    out_path = sys.argv[1]
    print(f"🚀 Đang kích hoạt Example 140 để trích xuất 1,024 Shards O(1) -> `{out_path}`...")
    cmd = [
        "cargo", "run", "--release", "--example", "140_perpetual_tt_two_way_sync",
        "--", "--dump", out_path
    ]
    subprocess.run(cmd, check=True)

if __name__ == "__main__":
    main()
