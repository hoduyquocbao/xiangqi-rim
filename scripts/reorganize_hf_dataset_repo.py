#!/usr/bin/env python3
# ==============================================================================
# SCRIPT DỌN DẸP VÀ CHUẨN HÓA CẤU TRÚC KHO THỦY TỔ HUGGINGFACE DATASET
# ==============================================================================
# `reorganize_hf_dataset_repo.py` chịu trách nhiệm:
#   1. Di chuyển toàn bộ tệp thử nghiệm cũ (chunks/, legacy_chunks/) sang `gen8_preview_v1/`.
#   2. Giữ kho sạch sẽ, minh bạch 100% cho 3 giai đoạn:
#      - gen6_depth4_chunks/   : 20M FENs Depth 4 cho Huấn luyện NNUE Gen 6
#      - gen8_depth8_chunks/   : Upgraded FENs Depth 8 cho Huấn luyện NNUE Gen 8
#      - gen12_depth12_chunks/ : Super-Master FENs Depth 12 cho Huấn luyện NNUE Gen 12
# ==============================================================================

import os
import sys
from huggingface_hub import HfApi

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
    print("🧹 XIANGQI-RIM: REORGANIZE HUGGINGFACE REPO TO CLEAR 3-STAGE PIPELINE")
    print("===============================================================================")
    sys.stdout.flush()

    try:
        repo_files = api.list_repo_files(repo_id=REPO, repo_type="dataset")
        print(f"✔ Tìm thấy {len(repo_files)} tệp tin trên kho HuggingFace Repo `{REPO}`.")
    except Exception as e:
        print(f"❌ Không thể đọc danh sách tệp HuggingFace: {e}")
        return

    moved_count = 0
    for fname in sorted(repo_files):
        if fname in [".gitattributes", "README.md"]:
            continue
            
        # Nếu tệp thuộc các thư mục thử nghiệm cũ (chunks/, legacy_chunks/, upgraded_depth8_chunks/)
        if (fname.startswith("chunks/") or fname.startswith("legacy_chunks/") or fname.startswith("upgraded_depth8_chunks/")) and not fname.startswith("gen8_preview_v1/"):
            new_path = f"gen8_preview_v1/{fname}"
            print(f"📦 Di chuyển tệp thử nghiệm: `{fname}` -> `{new_path}`...")
            sys.stdout.flush()
            try:
                # 1. Tải tệp tạm từ Cloud về
                from huggingface_hub import hf_hub_download
                local_path = hf_hub_download(repo_id=REPO, filename=fname, repo_type="dataset")
                
                # 2. Upload tệp sang vị trí mới gen8_preview_v1/
                api.upload_file(
                    path_or_fileobj=local_path,
                    path_in_repo=new_path,
                    repo_id=REPO,
                    repo_type="dataset"
                )
                
                # 3. Xóa vị trí cũ
                api.delete_file(path_in_repo=fname, repo_id=REPO, repo_type="dataset")
                
                print(f"✔ Đã phân loại `{fname}` sang `{new_path}` thành công!")
                moved_count += 1
            except Exception as e:
                print(f"⚠️ Lỗi di chuyển `{fname}`: {e}")

    print("\n===============================================================================")
    print(f"🎉 ĐÃ HOÀN TẤT DỌN DẸP HUGGINGFACE REPO: Phân loại {moved_count} tệp thử nghiệm sang `gen8_preview_v1/`!")
    print("===============================================================================")

if __name__ == "__main__":
    main()
