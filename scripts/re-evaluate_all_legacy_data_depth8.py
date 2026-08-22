#!/usr/bin/env python3
# ==============================================================================
# SCRIPT CHUYÊN BIỆT RE-EVALUATE NÂNG CẤP DỮ LIỆU CŨ SANG FULL AUTHENTIC DEPTH 8
# ==============================================================================
# `re-evaluate_all_legacy_data_depth8.py` làm sạch và nâng cấp toàn bộ tệp cũ:
#   1. Quét các tệp dữ liệu cũ trong `data/` (selfplay_samples_gen6_depth4.jsonl, depth5, etc.)
#   2. Gọi binary Rust `./target/release/examples/91_upgrade_legacy_fen_depth8`
#      để re-evaluate 100% thế cờ sang Full Authentic Depth 8 với Anti-Poisoning Schema.
#   3. Tự động đồng bộ các tệp nâng cấp lên HuggingFace Hub under `upgraded_depth8_chunks/`.
# ==============================================================================

import os
import glob
import subprocess
import sys
from huggingface_hub import HfApi, create_repo

REPO = "hoduyquocbao/xiangqi-gen6-platinum-dataset"

def read_token():
    token = os.environ.get("HF_TOKEN")
    if not token:
        try:
            from google.colab import userdata
            token = userdata.get("HF_TOKEN")
        except Exception:
            token = None
    return token

def main():
    token = read_token()
    api = HfApi(token=token)
    
    print("===============================================================================")
    print("💎 XIANGQI-RIM: RE-EVALUATING & UPGRADING ALL LEGACY DATA TO AUTHENTIC DEPTH 8")
    print("===============================================================================")
    sys.stdout.flush()

    try:
        create_repo(repo_id=REPO, repo_type="dataset", token=token, exist_ok=True)
        print(f"✔ Đã kết nối kho chứa HuggingFace Dataset: {REPO}")
    except Exception as e:
        print(f"⚠️ Lỗi kết nối HuggingFace: {e}")

    # Các tệp cũ cần nâng cấp
    legacy_files = [
        "data/selfplay_samples_gen6_depth4.jsonl",
        "data/selfplay_samples_gen6_depth5.jsonl",
    ]

    # Tìm thêm các tệp legacy trong data/
    for fname in glob.glob("data/chunk_platinum_00*.jsonl"):
        if fname not in legacy_files:
            legacy_files.append(fname)

    os.makedirs("data/upgraded_depth8", exist_ok=True)

    for lfile in legacy_files:
        if not os.path.exists(lfile) or os.path.getsize(lfile) == 0:
            continue

        base_name = os.path.basename(lfile)
        out_name = f"upgraded_depth8_{base_name}"
        out_path = os.path.join("data/upgraded_depth8", out_name)

        print(f"\n🚀 Đang nâng cấp tệp lịch sử: `{lfile}` -> `{out_path}` sang Full Depth 8...")
        sys.stdout.flush()

        env = os.environ.copy()
        env["INPUT"] = lfile
        env["OUTPUT"] = out_path
        env["DEPTH"] = "8"
        env["THREADS"] = "4"
        env["TT_MB"] = "1024"

        cmd = ["./target/release/examples/91_upgrade_legacy_fen_depth8"]
        process = subprocess.Popen(cmd, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, bufsize=1)
        for line in iter(process.stdout.readline, ""):
            print(line, end="", flush=True)
        process.stdout.close()
        process.wait()

        # Upload kết quả lên HuggingFace
        if os.path.exists(out_path) and os.path.getsize(out_path) > 0:
            cloud_path = f"upgraded_depth8_chunks/{out_name}"
            size_mb = os.path.getsize(out_path) / (1024 * 1024)
            print(f"📤 Đồng bộ tệp đã nâng cấp sang Cloud: `{cloud_path}` ({size_mb:.2f} MB)...")
            sys.stdout.flush()
            try:
                api.upload_file(
                    path_or_fileobj=out_path,
                    path_in_repo=cloud_path,
                    repo_id=REPO,
                    repo_type="dataset"
                )
                print(f"✔ Đã nâng cấp & đồng bộ thành công `{cloud_path}` trên HuggingFace Hub!")
            except Exception as e:
                print(f"⚠️ Lỗi upload `{cloud_path}`: {e}")

    print("\n===============================================================================")
    print("🎉 HOÀN TẤT CHIẾN DỊCH NÂNG CẤP TOÀN BỘ DỮ LIỆU CỦ SANG FULL AUTHENTIC DEPTH 8!")
    print("===============================================================================")

if __name__ == "__main__":
    main()
