# scripts/drain_and_upload_chunks_1b.py
# ==============================================================================
# KỊCH BẢN ĐẨY TUẦN TỰ & XÓA NGAY CÁC CHUNK TỒN ĐỌNG TRÊN ĐĨA CỤC BỘ
# ==============================================================================
# Tải từng tệp chunk lên Hugging Face và xóa ngay lập tức để thu hồi toàn bộ SSD.
# ==============================================================================

import os  # Thao tác hệ thống tệp
import sys  # Thao tác CLI
import glob  # Tìm tệp
import time  # Đo thời gian
from pathlib import Path  # Đường dẫn
from huggingface_hub import HfApi  # API HuggingFace
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from scripts.rolling_55m_hf_coordinator import load_token, size

def main():
    token = load_token()
    if not token:
        print("❌ Lỗi: Không tìm thấy HF_TOKEN!", flush=True)
        sys.exit(1)

    api = HfApi(token=token)
    repo_id = "hoduyquocbao/xiangqi-gen6-platinum-dataset"
    cdir = Path("data/chunks_1b")

    jsonl_files = sorted(glob.glob(str(cdir / "*.jsonl")))
    print(f"📦 Tìm thấy {len(jsonl_files)} chunks tồn đọng cần upload và dọn dẹp SSD...", flush=True)

    uploaded_count = 0
    total_freed_mb = 0.0

    for jpath_str in jsonl_files:
        jpath = Path(jpath_str)
        rflag = jpath.with_suffix(".ready")
        file_mb = size(jpath)
        cloud_target = f"massive_1b/{jpath.name}"

        print(f"\n📤 [DRAIN UPLOAD] Đang tải `{jpath.name}` ({file_mb:.2f} MB) -> `{cloud_target}`...", flush=True)
        start_t = time.time()

        try:
            api.upload_file(
                path_or_fileobj=str(jpath),
                path_in_repo=cloud_target,
                repo_id=repo_id,
                repo_type="dataset"
            )
            dur = time.time() - start_t
            spd = file_mb / dur if dur > 0 else 0
            print(f"✔ [XÁC NHẬN CLOUD] Tải thành công trong {dur:.2f}s ({spd:.2f} MB/s)!", flush=True)

            # Xóa ngay lập tức
            jpath.unlink(missing_ok=True)
            rflag.unlink(missing_ok=True)
            uploaded_count += 1
            total_freed_mb += file_mb

            print(f"🧹 [ĐÃ GIẢI PHÓNG] Xóa `{jpath.name}` -> Đã thu hồi tổng cộng {total_freed_mb/1024:.2f} GB SSD.", flush=True)

        except Exception as err:
            print(f"❌ [LỖI UPLOAD] {err}", flush=True)
            # Nếu lỗi do No space left on device khi tạm cache, xóa bớt các file cache tạm của huggingface_hub nếu có
            time.sleep(2.0)

    from scripts import update_dataset_readme
    update_dataset_readme.update_readme_on_hub(token=token, repo_id=repo_id)
    print(f"\n🎉 HOÀN TẤT DỌN DẸP! Đã tải và giải phóng {uploaded_count} chunks ({total_freed_mb/1024:.2f} GB)!", flush=True)


if __name__ == "__main__":
    main()
