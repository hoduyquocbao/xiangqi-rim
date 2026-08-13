#!/usr/bin/env python3
# ==============================================================================
# SCRIPT KỊCH BẢN AUDIT VÀ ĐỒNG BỘ TOÀN BỘ DỮ LIỆU CŨ LÊN HUGGINGFACE HUB
# ==============================================================================
# `sync_legacy_chunks_to_hf.py` chịu trách nhiệm:
#   1. Kiểm toán toàn bộ các tệp `.jsonl` lịch sử trong thư mục `data/`.
#   2. Đảm bảo 100% tệp dữ liệu cũ chưa đẩy lên Cloud sẽ được đăng ký bảo vệ
#      dưới thư mục `legacy_chunks/` hoặc `chunks/` với tên tệp CRDT chống ghi đè.
# ==============================================================================

import os
import glob
import time
from huggingface_hub import HfApi, create_repo

REPO = "hoduyquocbao/xiangqi-gen6-platinum-dataset"

def main():
    token = os.environ.get("HF_TOKEN")
    if not token:
        try:
            from google.colab import userdata
            token = userdata.get("HF_TOKEN")
        except Exception:
            token = None
    api = HfApi(token=token)
    
    print("===============================================================================")
    print("🛡️ XIANGQI-RIM: AUDIT & BẢO VỆ TOÀN BỘ DỮ LIỆU CŨ LÊN HUGGINGFACE HUB")
    print("===============================================================================")
    
    try:
        create_repo(repo_id=REPO, repo_type="dataset", token=token, exist_ok=True)
        print(f"✔ Đã kết nối kho chứa HuggingFace Dataset: {REPO}")
    except Exception as e:
        print(f"❌ Lỗi kết nối HuggingFace: {e}")
        return

    # Lấy danh sách tệp hiện có trên Cloud
    cloud_files = set(api.list_repo_files(repo_id=REPO, repo_type="dataset"))
    print(f"✔ Hiện tại trên Cloud đang bảo tồn {len(cloud_files)} tệp tin.")

    # Quét toàn bộ tệp .jsonl trong data/
    local_files = glob.glob("data/*.jsonl") + glob.glob("data/backed_up_chunks/*.jsonl")
    print(f"🔍 Tìm thấy {len(local_files)} tệp dữ liệu .jsonl trên ổ đĩa máy tính.")

    uploaded_count = 0
    for lfile in sorted(local_files):
        if not os.path.exists(lfile) or os.path.getsize(lfile) == 0:
            continue
            
        fname = os.path.basename(lfile)
        cloud_target = f"legacy_chunks/{fname}"
        
        # Nếu tệp thuộc chunks/ tiêu chuẩn
        if fname.startswith("chunk_platinum_") and len(fname) < 30:
            cloud_target = f"chunks/{fname}"

        # Đẩy tệp lên nếu chưa có trên Cloud
        if cloud_target not in cloud_files:
            size_mb = os.path.getsize(lfile) / (1024 * 1024)
            print(f"📤 Đang đẩy tệp lịch sử: `{lfile}` -> `{cloud_target}` ({size_mb:.2f} MB)...")
            try:
                api.upload_file(
                    path_or_fileobj=lfile,
                    path_in_repo=cloud_target,
                    repo_id=REPO,
                    repo_type="dataset"
                )
                print(f"✔ Đã bảo vệ thành công tệp lịch sử `{cloud_target}` trên HuggingFace Hub!")
                uploaded_count += 1
            except Exception as e:
                print(f"⚠️ Lỗi upload `{lfile}`: {e}")
        else:
            print(f"✔ Tệp lịch sử `{cloud_target}` ĐÃ ĐƯỢC BẢO VỆ AN TOÀN TRÊN CLOUD.")

    print("\n===============================================================================")
    print(f"🎉 HOÀN TẤT BẢO VỆ DỮ LIỆU CŨ: Đã đồng bổ thêm {uploaded_count} tệp lịch sử lên Cloud!")
    print("===============================================================================")

if __name__ == "__main__":
    main()
