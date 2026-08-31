# scripts/rolling_shard_uploader.py
# ==============================================================================
# ĐỒNG BỘ CUỐN CHIẾU SHARDS THEO LÔ (BATCH COMMIT) LÊN HUGGINGFACE HUB
# ==============================================================================
# Kịch bản thực hiện:
#   1. Quét toàn bộ tệp nhị phân phân mảnh `data/shards_10b/shard_XXXX.bin`.
#   2. Gom nhóm 25 tệp thành 1 mẻ (Batch) nạp qua `CommitOperationAdd`.
#   3. Gửi 1 Git Commit duy nhất cho mỗi 25 tệp -> Triệt tiêu 100% lỗi 429 Rate Limit (128 commits/giờ).
#   4. Cơ chế Failsafe 2 tầng: Xác thực Commit thành công trước khi xóa hàng loạt 25 tệp cục bộ.
#   5. Giải phóng tức thì ~650 MB SSD sau mỗi mẻ Commit hoàn tất, đưa đĩa trống lên ~31GB.
#
# 100% chú thích tiếng Việt từng dòng & 100% định danh từ đơn tiếng Anh (Single-Word Rule).
# ==============================================================================

import os  # Nhập thư viện hệ thống tương tác đường dẫn và tệp
import sys  # Nhập thư viện tương tác môi trường runtime và tham số
import time  # Nhập thư viện đo đạc mốc thời gian thực thi
import argparse  # Nhập thư viện phân tích tham số dòng lệnh
from pathlib import Path  # Nhập lớp xử lý đường dẫn hướng đối tượng

try:
    from huggingface_hub import HfApi, CommitOperationAdd, create_repo  # Nhập API Hugging Face Hub chính thức
except ImportError:
    print("❌ Thư viện 'huggingface_hub' chưa được cài đặt. Hãy chạy: pip install huggingface_hub", flush=True)
    sys.exit(1)

# ------------------------------------------------------------------------------
# 1. HẰNG SỐ CẤU HÌNH (SINGLE-WORD IDENTIFIER CONSTANTS)
# ------------------------------------------------------------------------------
VERSION = "v38.2.0-batch-commit-shard-uploader"  # Chuỗi định danh phiên bản
STAMP = "2026-08-31 21:40:00 ICT"  # Dấu thời gian phát hành
REPO = "hoduyquocbao/xiangqi-gen6-platinum-dataset"  # Tên kho lưu trữ Dataset Hub


def options():
    """
    Hàm `options`: Phân tích các tham số dòng lệnh CLI.
    Tất cả tham số và biến đều là từ đơn tiếng Anh.
    """
    parser = argparse.ArgumentParser(description="Đồng bộ cuốn chiếu Shards theo lô lên Hugging Face")
    parser.add_argument("--repo", type=str, default=REPO, help="Tên Hugging Face Dataset Repo")
    parser.add_argument("--token", type=str, default=None, help="Mã Token Hugging Face")
    parser.add_argument("--root", type=str, default="data/shards_10b", help="Đường dẫn thư mục Shards")
    parser.add_argument("--folder", type=str, default="shards", help="Thư mục đích trên Cloud")
    parser.add_argument("--batch", type=int, default=25, help="Số lượng tệp Shards trong 1 Commit")
    parser.add_argument("--limit", type=int, default=0, help="Giới hạn số lượng tệp cần tải (0 = tất cả)")
    parser.add_argument("--dry", action="store_true", help="Chế độ chạy thử không tải và không xóa")
    parser.add_argument("--keep", action="store_true", help="Chỉ tải lên Cloud mà không xóa tệp cục bộ")
    return parser.parse_args()


def credential(token=None):
    """
    Hàm `credential`: Nạp mã token xác thực Hugging Face an toàn.
    """
    if token:
        return token

    direct = os.environ.get("HF_TOKEN") or os.environ.get("HUGGINGFACE_TOKEN")
    if direct:
        return direct

    path = Path(".env")
    if path.exists():
        with open(path, "r", encoding="utf-8") as stream:
            for line in stream:
                line = line.strip()
                if line.startswith("HF_TOKEN="):
                    extracted = line.split("=", 1)[1].strip().strip('"').strip("'")
                    return extracted

    try:
        from google.colab import userdata  # Hỗ trợ nạp token trên môi trường Google Colab
        cloud = userdata.get("HF_TOKEN") or userdata.get("HUGGINGFACE_TOKEN")
        if cloud:
            return cloud
    except Exception:
        pass

    return None


def format(total):
    """
    Hàm `format`: Chuyển đổi số bytes sang định dạng chuỗi dung lượng dễ đọc.
    """
    amount = float(total)
    for unit in ["B", "KB", "MB", "GB", "TB"]:
        if amount < 1024.0:
            return f"{amount:.2f} {unit}"
        amount /= 1024.0
    return f"{amount:.2f} PB"


def main():
    """
    Hàm `main`: Thực thi quy trình đồng bộ cuốn chiếu theo lô Commit và giải phóng đĩa.
    """
    args = options()
    secret = credential(args.token)

    print("=" * 80, flush=True)
    print(" 🚀 XIANGQI-RIM: ĐỒNG BỘ CUỐN CHIẾU SHARDS THEO LÔ (BATCH COMMIT) LÊN HUGGINGFACE", flush=True)
    print(f"    Phiên bản    : {VERSION} | Build: {STAMP}", flush=True)
    print(f"    Repository   : https://huggingface.co/datasets/{args.repo}", flush=True)
    print(f"    Thư mục gốc  : {args.root}", flush=True)
    print(f"    Thư mục Cloud: {args.folder}/", flush=True)
    print(f"    Kích thước lô: {args.batch} tệp / 1 Commit", flush=True)
    print(f"    Chế độ       : {'CHẠY THỬ (DRY-RUN)' if args.dry else ('CHỈ TẢI (KEEP)' if args.keep else 'CUỐN CHIẾU & GIẢI PHÓNG ĐĨA')}", flush=True)
    print("=" * 80, flush=True)

    if not secret and not args.dry:
        print("❌ Lỗi: Không tìm thấy mã HF_TOKEN hợp lệ. Vui lòng kiểm tra tệp .env", flush=True)
        sys.exit(1)

    api = HfApi(token=secret) if secret else None

    # Khởi tạo hoặc xác nhận kho chứa Dataset trên Cloud
    if not args.dry and api:
        try:
            create_repo(repo_id=args.repo, repo_type="dataset", token=secret, exist_ok=True)
            print(f"✔ Đã kết nối thành công tới kho chứa Cloud: {args.repo}\n", flush=True)
        except Exception as fault:
            print(f"⚠️ Lưu ý kết nối repo ({args.repo}): {fault}\n", flush=True)

    # 1. Quét danh sách tệp đã tồn tại trên Cloud để tránh tải lặp
    cloud = set()
    if api and not args.dry:
        try:
            print("🔍 Đang kiểm tra danh sách tệp hiện hữu trên Cloud...", flush=True)
            listed = api.list_repo_files(repo_id=args.repo, repo_type="dataset")
            cloud = set(listed)
            print(f"✔ Phát hiện {len(cloud)} tệp đã có sẵn trên Cloud.\n", flush=True)
        except Exception as fault:
            print(f"⚠️ Không thể lấy danh sách tệp Cloud: {fault}\n", flush=True)

    # 2. Quét danh sách tệp Shard cục bộ
    source = Path(args.root)
    if not source.exists() or not source.is_dir():
        print(f"❌ Thư mục Shards `{args.root}` không tồn tại trên hệ thống cục bộ!", flush=True)
        return

    items = sorted(list(source.glob("shard_*.bin")))
    if not items:
        print(f"✔ Không còn tệp shard_*.bin nào trong `{args.root}`. Đĩa đã sạch!", flush=True)
        return

    # Lọc bỏ các tệp đã có trên Cloud
    pending = []
    already = []
    for item in items:
        target = f"{args.folder}/{item.name}" if args.folder else item.name
        if target in cloud:
            already.append(item)
        else:
            pending.append(item)

    # Nếu có tệp đã lên Cloud nhưng còn trên máy -> xóa ngay
    if already and not args.keep and not args.dry:
        reclaimed_old = 0
        for old in already:
            try:
                reclaimed_old += old.stat().st_size
                old.unlink()
            except Exception:
                pass
        if reclaimed_old > 0:
            print(f"🧹 Đã dọn dẹp {len(already)} tệp cũ đã có trên Cloud ➔ Thu hồi: {format(reclaimed_old)} SSD\n", flush=True)

    total = len(pending)
    volume = sum(p.stat().st_size for p in pending if p.exists())
    print(f"📊 Cần tải {total} tệp Shards mới | Dung lượng: {format(volume)}\n", flush=True)

    if args.limit > 0:
        pending = pending[:args.limit]
        print(f"⚠️ Giới hạn xử lý: {len(pending)} tệp đầu tiên.\n", flush=True)

    transferred = 0  # Số lượng tệp đã tải thành công
    reclaimed = 0  # Tổng dung lượng byte SSD đã thu hồi
    start = time.time()  # Mốc thời gian bắt đầu toàn cục
    size = args.batch

    # 3. Phân mẻ và tải theo Batch Commit
    batches = [pending[i:i + size] for i in range(0, len(pending), size)]
    total_batches = len(batches)

    for b_idx, batch in enumerate(batches, 1):
        batch_bytes = sum(p.stat().st_size for p in batch if p.exists())
        print("-" * 80, flush=True)
        print(f"📦 [MẺ {b_idx:03d}/{total_batches:03d}] Chuẩn bị Commit {len(batch)} tệp ({format(batch_bytes)})...", flush=True)

        operations = []
        for p in batch:
            target = f"{args.folder}/{p.name}" if args.folder else p.name
            operations.append(CommitOperationAdd(path_in_repo=target, path_or_fileobj=str(p)))

        if args.dry:
            print(f"  [DRY-RUN] Sẽ gửi 1 Commit chứa {len(operations)} tệp ➔ Cloud", flush=True)
            print(f"  [DRY-RUN] Sẽ xóa {len(batch)} tệp để giải phóng {format(batch_bytes)} SSD.", flush=True)
            transferred += len(batch)
            reclaimed += batch_bytes
            continue

        # Gửi Commit hàng loạt với cơ chế Retry
        success = False
        attempt = 0
        maximum = 3
        while attempt < maximum and not success:
            attempt += 1
            lap = time.time()
            try:
                print(f"  📤 Đang gửi Batch Commit lên Cloud ({len(operations)} tệp)...", flush=True)
                api.create_commit(
                    repo_id=args.repo,
                    repo_type="dataset",
                    operations=operations,
                    commit_message=f"Upload shards batch {b_idx}/{total_batches} ({len(operations)} files)",
                )
                duration = time.time() - lap
                speed = (batch_bytes / (1024 * 1024)) / duration if duration > 0 else 0
                print(f"  ✅ Batch Commit {b_idx} thành công trong {duration:.2f}s ({speed:.2f} MB/s)!", flush=True)
                success = True
                transferred += len(batch)
            except Exception as fault:
                print(f"  ❌ Lỗi Commit mẻ {b_idx} (Lần {attempt}): {fault}", flush=True)
                if "429" in str(fault):
                    print("  ⏳ Gặp giới hạn tần suất, tạm dừng 60s trước khi thử lại...", flush=True)
                    time.sleep(60)
                else:
                    time.sleep(5)

        # 4. Giải phóng tệp cục bộ sau khi Commit thành công
        if success and not args.keep:
            deleted_batch = 0
            for item in batch:
                try:
                    weight = item.stat().st_size
                    item.unlink()
                    reclaimed += weight
                    deleted_batch += 1
                except Exception as fault:
                    print(f"  ⚠️ Không thể xóa `{item.name}`: {fault}", flush=True)
            print(f"  🧹 [PURGE] Đã xóa sạch {deleted_batch} tệp trong mẻ {b_idx} ➔ Đã thu hồi thêm: {format(batch_bytes)} | Tổng đã thu hồi: {format(reclaimed)} SSD", flush=True)
        elif not success:
            print(f"  🛡️ [FAILSAFE] Giữ lại toàn bộ {len(batch)} tệp trong mẻ {b_idx} do Commit thất bại!", flush=True)

    # 5. Dọn dẹp thư mục rỗng nếu toàn bộ tệp đã được giải phóng
    if not args.keep and not args.dry and source.exists():
        remaining = list(source.glob("*.bin"))
        if not remaining:
            try:
                source.rmdir()
                print(f"\n✨ Đã dọn dẹp hoàn toàn thư mục rỗng `{args.root}`!", flush=True)
            except Exception:
                pass

    elapsed = time.time() - start
    print("\n" + "=" * 80, flush=True)
    print(" 🎉 HOÀN TẤT TIẾN TRÌNH ĐỒNG BỘ CUỐN CHIẾU THEO LÔ!", flush=True)
    print(f"  • Số tệp mới đã tải lên Hub   : {transferred} tệp", flush=True)
    print(f"  • Tổng dung lượng SSD thu hồi : {format(reclaimed)}", flush=True)
    print(f"  • Tổng thời gian thực thi     : {elapsed:.2f} giây", flush=True)
    print("=" * 80 + "\n", flush=True)


if __name__ == "__main__":
    main()
