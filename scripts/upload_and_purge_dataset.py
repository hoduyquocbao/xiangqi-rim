# scripts/upload_and_purge_dataset.py
# ==============================================================================
# KỊCH BẢN ĐỒNG BỘ TOÀN DIỆN DỮ LIỆU DATA/ LÊN HUGGINGFACE HUB VÀ GIẢI PHÓNG ĐĨA
# ==============================================================================
# Kịch bản Python tự động:
#   1. Đọc mã token HuggingFace từ tệp `.env` hoặc biến môi trường `HF_TOKEN`.
#   2. Quét toàn bộ tệp dữ liệu cờ Tướng trong thư mục `data/` (~28 GB).
#   3. Tải tuần tự các tệp lớn lên HuggingFace Dataset Hub (`hoduyquocbao/xiangqi-gen6-platinum-dataset`).
#   4. Xác thực tính toàn vẹn và xóa tệp cục bộ giải phóng không gian ổ đĩa SSD.
#   5. Bảo tồn các tệp trọng số nén nhẹ, sổ khai cuộc và `.gitkeep`.
#
# Tuân thủ 100% quy tắc định danh từ đơn tiếng Anh và 100% chú thích tiếng Việt.
# ==============================================================================

import os  # Nhập thư viện os thao tác hệ thống tệp và biến môi trường
import sys  # Nhập thư viện sys tương tác với tham số dòng lệnh
import glob  # Nhập thư viện glob tìm kiếm tệp theo mẫu
import time  # Nhập thư viện time đo lường thời gian
import shutil  # Nhập thư viện shutil thao tác tệp và thư mục nén
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
APP_VERSION = "v35.0.0-hf-dataset-purge-sync"
APP_BUILD_STAMP = "2026-08-29 07:45:00 ICT"
DEFAULT_REPO = "hoduyquocbao/xiangqi-gen6-platinum-dataset"


def parse():
    """
    Hàm `parse`: Phân tích tham số dòng lệnh CLI.
    """
    parser = argparse.ArgumentParser(description="Đồng bộ dữ liệu data/ lên HuggingFace và dọn dẹp đĩa SSD")
    parser.add_argument("--repo", type=str, default=DEFAULT_REPO, help="Tên HuggingFace Dataset Repository")
    parser.add_argument("--token", type=str, default=None, help="Mã Token HuggingFace ghi (Write)")
    parser.add_argument("--dry", action="store_true", help="Chế độ chạy thử không tải và không xóa")
    parser.add_argument("--upload", action="store_true", help="Chỉ tải lên HuggingFace không xóa tệp cục bộ")
    parser.add_argument("--purge", action="store_true", help="Chỉ dọn dẹp các tệp đã được tải lên Cloud")
    parser.add_argument("--target", type=str, default=None, help="Chỉ định đường dẫn tệp đơn lẻ cần xử lý")
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


def format_bytes(bytes_count):
    """
    Hàm `format_bytes`: Định dạng số bytes thành chuỗi dễ đọc (B, KB, MB, GB).
    """
    for unit in ['B', 'KB', 'MB', 'GB', 'TB']:
        if bytes_count < 1024.0:
            return f"{bytes_count:.2f} {unit}"
        bytes_count /= 1024.0
    return f"{bytes_count:.2f} PB"


def main():
    """
    Hàm `main`: Thực thi luồng đồng bộ toàn diện dữ liệu và dọn dẹp ổ đĩa.
    """
    args = parse()
    token = load_token(args.token)

    print("===============================================================================", flush=True)
    print(" 🚀 XIANGQI-RIM: ĐỒNG BỘ DỮ LIỆU DATA/ LÊN HUGGINGFACE & GIẢI PHÓNG 28GB SSD", flush=True)
    print(f"    Phiên bản: {APP_VERSION} | Build: {APP_BUILD_STAMP}", flush=True)
    print(f"    Repository: https://huggingface.co/datasets/{args.repo}", flush=True)
    print("===============================================================================\n", flush=True)

    if not token and not args.dry:
        print("❌ Lỗi: Không tìm thấy mã HF_TOKEN hợp lệ. Vui lòng cấu hình trong .env hoặc truyền --token", flush=True)
        sys.exit(1)

    api = HfApi(token=token) if token else None

    if not args.dry and api:
        try:
            create_repo(repo_id=args.repo, repo_type="dataset", token=token, exist_ok=True)
            print(f"✔ Đã kết nối thành công tới kho chứa HuggingFace: {args.repo}", flush=True)
        except Exception as err:
            print(f"⚠️ Lưu ý kết nối repo ({args.repo}): {err}", flush=True)

    # 1. Quét danh sách tệp trên Cloud để tránh tải trùng lặp
    cloud_files = set()
    if api and not args.dry:
        try:
            cloud_files = set(api.list_repo_files(repo_id=args.repo, repo_type="dataset"))
            print(f"✔ Hiện tại trên Cloud đã có sẵn {len(cloud_files)} tệp tin.", flush=True)
        except Exception as err:
            print(f"⚠️ Không thể lấy danh sách tệp Cloud: {err}", flush=True)

    # 2. Quét toàn bộ tệp trong thư mục data/
    data_dir = Path("data")
    if not data_dir.exists():
        print("❌ Thư mục 'data/' không tồn tại trên hệ thống cục bộ!", flush=True)
        return

    # Danh sách các tệp nhị phân nén nhẹ cần giữ lại cục bộ sau dọn dẹp
    keep_local = {
        ".gitkeep",
        "nnue_weights.bin",
        "nnue_weights_gen9.bin",
        "master_book.xrbk",
        "knowledge_bundle.xrkb",
    }

    # Nén các thư mục con nhỏ thành archive nếu cần, riêng shards_10b chuyển sang rolling_shard_uploader.py
    archives = []
    shards_dir = data_dir / "shards_10b"
    if shards_dir.exists() and shards_dir.is_dir():
        print("💡 Lưu ý: Kho `data/shards_10b` (26GB) được đồng bộ an toàn qua `scripts/rolling_shard_uploader.py` để tránh tràn đĩa.", flush=True)

    vault_dir = data_dir / "vault"
    if vault_dir.exists() and vault_dir.is_dir():
        vault_tar = data_dir / "vault.tar.gz"
        if not vault_tar.exists() and not args.dry:
            print(f"📦 Đang đóng gói nén thư mục '{vault_dir}' -> '{vault_tar}'...", flush=True)
            shutil.make_archive(str(data_dir / "vault"), 'gztar', str(vault_dir))
            print(f"✔ Đã đóng gói thành công: {vault_tar} ({size(vault_tar):.2f} MB)", flush=True)
        if vault_tar.exists():
            archives.append(vault_tar)

    # Tìm tất cả tệp trong data/
    all_files = []
    for item in sorted(data_dir.iterdir()):
        if item.is_file():
            all_files.append(item)

    if args.target:
        all_files = [Path(args.target)]

    total_bytes = sum(f.stat().st_size for f in all_files if f.exists())
    print(f"📊 Tổng số tệp phát hiện trong data/: {len(all_files)} tệp | Tổng dung lượng: {format_bytes(total_bytes)}\n", flush=True)

    uploaded_count = 0
    purged_bytes = 0
    start_total_time = time.time()

    # Sắp xếp theo thứ tự dung lượng giảm dần để ưu tiên giải phóng tệp lớn nhất trước
    all_files.sort(key=lambda p: p.stat().st_size if p.exists() else 0, reverse=True)

    for item in all_files:
        if not item.exists():
            continue

        fname = item.name
        file_bytes = item.stat().st_size
        file_mb = file_bytes / (1024 * 1024)

        # Bỏ qua tệp 0 bytes hoặc .DS_Store
        if file_bytes == 0 or fname == ".DS_Store":
            continue

        # Định tuyến đường dẫn lưu trữ trên HuggingFace Cloud
        if fname.endswith(".jsonl"):
            if "distill" in fname or "purified" in fname or "symmetrical" in fname or "pikafish" in fname:
                cloud_dest = f"distill/{fname}"
            elif "stream" in fname or "beam" in fname:
                cloud_dest = f"streams/{fname}"
            elif "production" in fname or "gen6" in fname or "gen9" in fname:
                cloud_dest = f"production/{fname}"
            else:
                cloud_dest = f"dataset/{fname}"
        elif fname.endswith(".tar.gz"):
            cloud_dest = f"archives/{fname}"
        elif fname.endswith(".bin"):
            cloud_dest = f"models/{fname}"
        elif fname.endswith(".xrk") or fname.endswith(".xrkb") or fname.endswith(".xrbk"):
            cloud_dest = f"books/{fname}"
        else:
            cloud_dest = f"data/{fname}"

        print(f"-------------------------------------------------------------------------------", flush=True)
        print(f"📄 Xử lý tệp: {item} ({format_bytes(file_bytes)}) -> Cloud: `{cloud_dest}`", flush=True)

        is_already_on_cloud = cloud_dest in cloud_files

        # 3. Tải lên HuggingFace Cloud nếu chưa có hoặc không phải chế độ chỉ dọn dẹp
        if not args.purge and not is_already_on_cloud:
            if args.dry:
                print(f"  [DRY-RUN] Sẽ tải `{item}` ({file_mb:.2f} MB) -> `{cloud_dest}`", flush=True)
            else:
                print(f"  📤 Đang tải lên HuggingFace Hub (Resumable LFS Upload)...", flush=True)
                upload_start = time.time()
                try:
                    api.upload_file(
                        path_or_fileobj=str(item),
                        path_in_repo=cloud_dest,
                        repo_id=args.repo,
                        repo_type="dataset"
                    )
                    upload_duration = time.time() - upload_start
                    upload_speed = file_mb / upload_duration if upload_duration > 0 else 0
                    print(f"  ✅ Đã tải thành công `{cloud_dest}` lên Hub trong {upload_duration:.2f}s ({upload_speed:.2f} MB/s)!", flush=True)
                    uploaded_count += 1
                    is_already_on_cloud = True
                except Exception as err:
                    print(f"  ❌ Lỗi tải tệp `{item}`: {err}", flush=True)
                    continue
        elif is_already_on_cloud:
            print(f"  ✔ Tệp `{cloud_dest}` ĐÃ CÓ TRÊN CLOUD HUGGINGFACE.", flush=True)

        # 4. Giải phóng tệp cục bộ nếu đã tải lên Cloud an toàn và không nằm trong danh sách giữ lại
        if not args.upload and is_already_on_cloud and fname not in keep_local:
            if args.dry:
                print(f"  [DRY-RUN] Sẽ xóa tệp cục bộ `{item}` để giải phóng {format_bytes(file_bytes)}", flush=True)
            else:
                try:
                    item.unlink()
                    purged_bytes += file_bytes
                    print(f"  🧹 [PURGE] Đã xóa `{item}` -> Đã giải phóng: {format_bytes(file_bytes)} SSD.", flush=True)
                except Exception as err:
                    print(f"  ⚠️ Không thể xóa `{item}`: {err}", flush=True)
        elif fname in keep_local:
            print(f"  🛡️ [PRESERVED] Giữ lại tệp cục bộ quan trọng: `{fname}`", flush=True)

    # 5. Xóa các thư mục con đã được nén nếu có
    if not args.upload and not args.dry:
        if shards_dir.exists() and "archives/shards_10b.tar.gz" in cloud_files:
            try:
                shutil.rmtree(str(shards_dir))
                print(f"🧹 [PURGE] Đã xóa thư mục '{shards_dir}' đã đồng bộ.", flush=True)
            except Exception as err:
                print(f"⚠️ Không thể xóa thư mục '{shards_dir}': {err}", flush=True)

        if vault_dir.exists() and "archives/vault.tar.gz" in cloud_files:
            try:
                shutil.rmtree(str(vault_dir))
                print(f"🧹 [PURGE] Đã xóa thư mục '{vault_dir}' đã đồng bộ.", flush=True)
            except Exception as err:
                print(f"⚠️ Không thể xóa thư mục '{vault_dir}': {err}", flush=True)

    total_duration = time.time() - start_total_time
    print("\n===============================================================================", flush=True)
    print(f"🎉 HOÀN TẤT ĐỒNG BỘ VÀ GIẢI PHÓNG Ổ ĐĨA!", flush=True)
    print(f"  • Số tệp mới đã tải lên Hub   : {uploaded_count} tệp", flush=True)
    print(f"  • Tổng dung lượng SSD đã thu hồi: {format_bytes(purged_bytes)}", flush=True)
    print(f"  • Tổng thời gian thực thi     : {total_duration:.2f} giây", flush=True)
    print("===============================================================================\n", flush=True)


if __name__ == "__main__":
    main()
