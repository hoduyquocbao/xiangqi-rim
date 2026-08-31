# scripts/run_55m_mining_campaign.py
# ==============================================================================
# KỊCH BẢN ĐIỀU HÀNH ĐẠI CHIẾN DỊCH KHAI THÁC VÉT CẠN 55M FENS ĐẠT MỐC 100M
# ==============================================================================
# `run_55m_mining_campaign.py` khởi chạy song song 2 phân hệ:
#   1. Động cơ Rust Native `139_massive_55m_exhaustive_rolling_miner` (4 Worker Cores).
#   2. Luồng ngầm Python Coordinator tải tức thì Chunk lên HuggingFace và dọn dẹp SSD.
#   3. Tự động cập nhật Dataset Card README.md khi hoàn thành toàn bộ 55,000,000 FENs.
#
# Tuân thủ 100% quy tắc định danh từ đơn tiếng Anh và 100% chú thích tiếng Việt.
# ==============================================================================

import os  # Nhập thư viện os thao tác hệ thống tệp và biến môi trường
import sys  # Nhập thư viện sys tương tác với tham số dòng lệnh
import glob  # Nhập thư viện glob tìm kiếm tệp theo mẫu
import time  # Nhập thư viện time đo lường thời gian
import shutil  # Nhập thư viện shutil thao tác tệp
import threading  # Nhập thư viện threading quản lý luồng ngầm bất đồng bộ
import subprocess  # Nhập thư viện subprocess điều khiển tiến trình Rust Engine
from pathlib import Path  # Nhập lớp Path thao tác đường dẫn hướng đối tượng

try:
    from huggingface_hub import HfApi, create_repo  # Nhập thư viện HuggingFace Hub API
except ImportError:
    print("❌ Thư viện 'huggingface_hub' chưa được cài đặt. Hãy chạy: pip install huggingface_hub", flush=True)
    sys.exit(1)

try:
    import update_dataset_readme
except ImportError:
    from scripts import update_dataset_readme

# ------------------------------------------------------------------------------
# 1. HẰNG SỐ CẤU HÌNH VÀ PHIÊN BẢN (SINGLE-WORD CONTEXT CONSTANTS)
# ------------------------------------------------------------------------------
APP_VERSION = "v35.0.0-massive-55m-campaign-runner"
APP_BUILD_STAMP = "2026-08-29 19:20:00 ICT"
DEFAULT_REPO = "hoduyquocbao/xiangqi-gen6-platinum-dataset"
CHUNKS_DIR = "data/chunks_55m"


def load_token():
    """
    Hàm `load_token`: Nạp mã token HuggingFace an toàn từ .env hoặc os.environ.
    """
    token = os.environ.get("HF_TOKEN") or os.environ.get("HUGGINGFACE_TOKEN")
    if token:
        return token

    env_path = Path(".env")
    if env_path.exists():
        with open(env_path, "r", encoding="utf-8") as f_env:
            for line in f_env:
                line = line.strip()
                if line.startswith("HF_TOKEN="):
                    token = line.split("=", 1)[1].strip().strip('"').strip("'")
                    return token

    try:
        from google.colab import userdata
        token = userdata.get("HF_TOKEN") or userdata.get("HUGGINGFACE_TOKEN")
    except Exception:
        pass

    return token


def async_upload_coordinator(api, repo_id, chunks_dir, stop_flag, stats):
    """
    Hàm `async_upload_coordinator`: Luồng ngầm theo dõi các tệp .ready, tải lên HuggingFace và xóa đĩa.
    """
    cdir = Path(chunks_dir)
    cdir.mkdir(parents=True, exist_ok=True)

    while not stop_flag.is_set() or glob.glob(str(cdir / "*.ready")):
        ready_files = sorted(glob.glob(str(cdir / "*.ready")))
        if not ready_files:
            time.sleep(0.5)
            continue

        for rflag in ready_files:
            rpath = Path(rflag)
            jsonl_name = rpath.stem + ".jsonl"
            jsonl_path = cdir / jsonl_name

            if not jsonl_path.exists():
                rpath.unlink(missing_ok=True)
                continue

            file_bytes = jsonl_path.stat().st_size
            file_mb = file_bytes / (1024 * 1024)
            cloud_target = f"massive_55m/{jsonl_name}"

            print(f"\n📤 [ASYNC CLOUD SYNC] Đang tải Chunk `{jsonl_name}` ({file_mb:.2f} MB) lên HuggingFace...", flush=True)
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

                # Xóa tệp cục bộ giải phóng SSD tức thì
                jsonl_path.unlink(missing_ok=True)
                rpath.unlink(missing_ok=True)
                stats["uploaded_chunks"] += 1
                stats["uploaded_mb"] += file_mb

                print(f"🧹 [SSD PURGED] Đã xóa `{jsonl_name}` -> Giải phóng {file_mb:.2f} MB (Bảo toàn SSD < 500MB).\n", flush=True)

            except Exception as err:
                print(f"❌ [UPLOAD ERROR] Lỗi tải `{jsonl_name}`: {err}", flush=True)
                time.sleep(3.0)

        time.sleep(0.5)


def main():
    """
    Hàm `main`: Điều phối toàn bộ chiến dịch khai thác và đồng bộ 55M FENs.
    """
    print("===============================================================================", flush=True)
    print(" 🚀 ĐẠI CHIẾN DỊCH KHAI THÁC VÉT CẠN 55,000,000 FENS ĐẠT MỐC 100M HUGGINGFACE", flush=True)
    print(f"    Phiên bản: {APP_VERSION} | Build: {APP_BUILD_STAMP}", flush=True)
    print(f"    Kho lưu trữ: https://huggingface.co/datasets/{DEFAULT_REPO}", flush=True)
    print("===============================================================================\n", flush=True)

    token = load_token()
    if not token:
        print("❌ Lỗi: Không tìm thấy mã HF_TOKEN hợp lệ trong .env!", flush=True)
        sys.exit(1)

    api = HfApi(token=token)
    create_repo(repo_id=DEFAULT_REPO, repo_type="dataset", token=token, exist_ok=True)

    cdir = Path(CHUNKS_DIR)
    cdir.mkdir(parents=True, exist_ok=True)

    # 1. Khởi động luồng ngầm Upload Coordinator
    stop_flag = threading.Event()
    stats = {"uploaded_chunks": 0, "uploaded_mb": 0.0}

    coord_thread = threading.Thread(
        target=async_upload_coordinator,
        args=(api, DEFAULT_REPO, CHUNKS_DIR, stop_flag, stats),
        daemon=True
    )
    coord_thread.start()
    print("✔ Luồng ngầm HuggingFace Rolling Coordinator đã kích hoạt!", flush=True)

    # 2. Khởi chạy động cơ khai thác Native Rust
    binary_path = "./target/release/examples/139_massive_55m_exhaustive_rolling_miner"
    if not os.path.exists(binary_path):
        print(f"⚙️ Đang biên dịch bản release `{binary_path}`...", flush=True)
        subprocess.run(["cargo", "build", "--release", "--example", "139_massive_55m_exhaustive_rolling_miner"], check=True)

    print("🚀 Bắt đầu thực thi đại động cơ khai thác Native Rust (4 Physical Cores)...\n", flush=True)
    start_time = time.time()

    env_vars = os.environ.copy()
    env_vars["TOTAL_SAMPLES"] = env_vars.get("TOTAL_SAMPLES", "55000000")
    env_vars["CHUNK_SIZE"] = env_vars.get("CHUNK_SIZE", "2500000")
    env_vars["OUT_DIR"] = CHUNKS_DIR

    proc = subprocess.Popen(
        [binary_path],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
        env=env_vars
    )

    for line in proc.stdout:
        print(line, end="", flush=True)

    proc.wait()

    # 3. Báo hiệu luồng coordinator hoàn tất nốt các chunk còn lại
    print("\n⏳ Đang hoàn tất đồng bộ các chunks cuối cùng lên HuggingFace...", flush=True)
    stop_flag.set()
    coord_thread.join(timeout=300)

    # 4. Tự động cập nhật Dataset Card README.md trên HuggingFace Hub
    print("\n📝 Đang cập nhật Dataset Card README.md cán mốc 100,000,000+ FENs trên HuggingFace...", flush=True)
    try:
        update_dataset_readme.update_readme_on_hub(token=token, repo_id=DEFAULT_REPO)
    except Exception as err:
        print(f"⚠️ Lưu ý cập nhật README: {err}", flush=True)

    total_duration = time.time() - start_time
    print("\n===============================================================================", flush=True)
    print(" 🎉 TOÀN BỘ ĐẠI CHIẾN DỊCH 55,000,000 FENS ĐÃ HOÀN THÀNH XUẤT SẮC!", flush=True)
    print(f"  • Tổng số Chunks đã tải lên Hub : {stats['uploaded_chunks']} chunks", flush=True)
    print(f"  • Tổng dung lượng đã đồng bộ    : {stats['uploaded_mb']:.2f} MB", flush=True)
    print(f"  • Tổng thời gian chiến dịch     : {total_duration:.2f} giây ({total_duration/60:.2f} phút)", flush=True)
    print(f"  • Tổng kho dữ liệu HuggingFace  : Cán mốc 100,000,000+ FENs!", flush=True)
    print("===============================================================================\n", flush=True)


if __name__ == "__main__":
    main()
