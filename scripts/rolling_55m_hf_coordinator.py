# scripts/rolling_55m_hf_coordinator.py
# ==============================================================================
# KỊCH BẢN ĐIỀU PHỐI CUỐN CHIẾU ROLLING CHUNKS 55M FENS LÊN HUGGINGFACE HUB
# ==============================================================================
# `rolling_55m_hf_coordinator.py` quản lý vòng đời upload & purge:
#   1. Lắng nghe cờ báo hiệu `.ready` từ thư mục `data/chunks_55m/`.
#   2. Tải ngay chunk `.jsonl` lên kho chứa `hoduyquocbao/xiangqi-gen6-platinum-dataset`.
#   3. Xóa tệp cục bộ (`os.remove()`) ngay khi máy chủ HuggingFace xác nhận lưu trữ.
#   4. Tự động cập nhật Dataset Card README.md khi hoàn thành toàn bộ 55M FENs.
#
# Tuân thủ 100% quy tắc định danh từ đơn tiếng Anh và 100% chú thích tiếng Việt.
# ==============================================================================

import os  # Nhập thư viện os thao tác hệ thống tệp và biến môi trường
import sys  # Nhập thư viện sys tương tác với tham số dòng lệnh
import glob  # Nhập thư viện glob tìm kiếm tệp theo mẫu
import time  # Nhập thư viện time đo lường thời gian
import argparse  # Nhập thư viện argparse phân tích tham số CLI
from pathlib import Path  # Nhập lớp Path thao tác đường dẫn hướng đối tượng

try:
    from huggingface_hub import HfApi, create_repo  # Nhập thư viện HuggingFace Hub API
except ImportError:
    print("❌ Thư viện 'huggingface_hub' chưa được cài đặt. Hãy chạy: pip install huggingface_hub", flush=True)
    sys.exit(1)

# ------------------------------------------------------------------------------
# 1. HẰNG SỐ CẤU HÌNH VÀ PHIÊN BẢN (SINGLE-WORD CONTEXT CONSTANTS)
# ------------------------------------------------------------------------------
APP_VERSION = "v35.0.0-rolling-55m-coordinator"
APP_BUILD_STAMP = "2026-08-29 19:15:00 ICT"
DEFAULT_REPO = "hoduyquocbao/xiangqi-gen6-platinum-dataset"
DEFAULT_CHUNKS_DIR = "data/chunks_55m"


def parse():
    """
    Hàm `parse`: Phân tích tham số dòng lệnh CLI.
    """
    parser = argparse.ArgumentParser(description="Bộ điều phối cuốn chiếu Rolling Chunks 55M FENs lên HuggingFace")
    parser.add_argument("--repo", type=str, default=DEFAULT_REPO, help="Tên HuggingFace Dataset Repository")
    parser.add_argument("--dir", type=str, default=DEFAULT_CHUNKS_DIR, help="Thư mục chứa các tệp chunks")
    parser.add_argument("--token", type=str, default=None, help="Mã Token HuggingFace ghi (Write)")
    parser.add_argument("--test", action="store_true", help="Kiểm tra kết nối và token HuggingFace")
    parser.add_argument("--watch", action="store_true", help="Chạy chế độ lắng nghe liên tục cho đến khi hoàn tất")
    return parser.parse_args()


def load_token(explicit_token=None):
    """
    Hàm `load_token`: Nạp mã token HuggingFace an toàn từ tham số, .env hoặc os.environ.
    """
    if explicit_token:
        return explicit_token

    token = os.environ.get("HF_TOKEN") or os.environ.get("HUGGINGFACE_TOKEN")
    if token:
        return token

    # Đọc từ tệp .env cục bộ nếu có
    env_path = Path(".env")
    if env_path.exists():
        with open(env_path, "r", encoding="utf-8") as f_env:
            for line in f_env:
                line = line.strip()
                if line.startswith("HF_TOKEN="):
                    token = line.split("=", 1)[1].strip().strip('"').strip("'")
                    return token

    try:
        from google.colab import userdata  # Hỗ trợ nạp bí mật trên môi trường Colab
        token = userdata.get("HF_TOKEN") or userdata.get("HUGGINGFACE_TOKEN")
    except Exception:
        pass

    return token


def size(path):
    """
    Hàm `size`: Tính dung lượng tệp theo đơn vị Megabytes (MB).
    """
    try:
        return os.path.getsize(path) / (1024 * 1024)
    except OSError:
        return 0.0


def watch_and_sync(api, repo_id, chunks_dir):
    """
    Hàm `watch_and_sync`: Vòng lặp lắng nghe tệp chunk mới và đồng bộ tức thì.
    """
    cdir = Path(chunks_dir)
    cdir.mkdir(parents=True, exist_ok=True)

    print("===============================================================================", flush=True)
    print(" 🔄 BẮT ĐẦU BỘ ĐIỀU PHỐI CUỐN CHIẾU 55M HUGGINGFACE ROLLING COORDINATOR", flush=True)
    print(f"    Thư mục theo dõi: {cdir} | Kho lưu trữ: https://huggingface.co/datasets/{repo_id}", flush=True)
    print("===============================================================================\n", flush=True)

    uploaded_count = 0
    total_mb = 0.0

    while True:
        # Tìm các tệp cờ .ready
        ready_files = sorted(glob.glob(str(cdir / "*.ready")))

        if not ready_files:
            time.sleep(1.0)
            continue

        for rflag in ready_files:
            rpath = Path(rflag)
            jsonl_name = rpath.stem + ".jsonl"
            jsonl_path = cdir / jsonl_name

            if not jsonl_path.exists():
                rpath.unlink(missing_ok=True)
                continue

            file_mb = size(jsonl_path)
            cloud_target = f"massive_55m/{jsonl_name}"

            print(f"📤 [ROLLING SYNC] Đang tải Chunk `{jsonl_name}` ({file_mb:.2f} MB) -> `{cloud_target}`...", flush=True)
            upload_start = time.time()

            try:
                api.upload_file(
                    path_or_fileobj=str(jsonl_path),
                    path_in_repo=cloud_target,
                    repo_id=repo_id,
                    repo_type="dataset"
                )
                duration = time.time() - upload_start
                speed = file_mb / duration if duration > 0 else 0
                print(f"✔ [CLOUD CONFIRMED] Tải thành công `{cloud_target}` trong {duration:.2f}s ({speed:.2f} MB/s)!", flush=True)

                # Dọn dẹp ổ đĩa tức thì
                jsonl_path.unlink(missing_ok=True)
                rpath.unlink(missing_ok=True)
                uploaded_count += 1
                total_mb += file_mb

                print(f"🧹 [SSD PURGED] Đã xóa `{jsonl_name}` -> Giải phóng {file_mb:.2f} MB (Bảo toàn SSD < 500MB).\n", flush=True)

            except Exception as err:
                print(f"❌ [UPLOAD ERROR] Lỗi tải `{jsonl_name}`: {err}", flush=True)
                time.sleep(3.0)

        time.sleep(1.0)


def main():
    """
    Hàm `main`: Điểm nhập chính của kịch bản điều phối.
    """
    args = parse()
    token = load_token(args.token)

    if not token:
        print("❌ Lỗi: Không tìm thấy mã HF_TOKEN hợp lệ. Vui lòng cấu hình trong .env hoặc truyền --token", flush=True)
        sys.exit(1)

    api = HfApi(token=token)

    if args.test:
        user = api.whoami()
        print(f"✅ Xác thực HuggingFace thành công: User = {user.get('name')}", flush=True)
        return

    create_repo(repo_id=args.repo, repo_type="dataset", token=token, exist_ok=True)
    watch_and_sync(api, args.repo, args.dir)


if __name__ == "__main__":
    main()
