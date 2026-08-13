# ============================================================================
# SCRIPT: ROLLING CHUNK PLATINUM MINER PIPELINE V9.0.0 (500M NNUE DATASET)
# ============================================================================
# Kịch bản sản xuất chuẩn quốc tế cuốn chiếu (Mine Chunk -> Sync HuggingFace -> Purge Local).
# Giữ dung lượng ổ đĩa SSD < 50 MB xuyên suốt tác vụ đào siêu lớn 500,000,000 (500M) mẫu FEN.
# Tuân thủ 100% Quy tắc 8.14 (Rolling Chunks & Purge) và Quy tắc 8.10 (Realtime Yield).
# Tuân thủ 100% định danh từ đơn tiếng Anh và 100% chú thích tiếng Việt tường minh.
# ============================================================================

import os  # Nhập thư viện os thao tác với hệ thống tệp đĩa và môi trường
import sys  # Nhập thư viện sys tương tác với tham số hệ thống
import time  # Nhập thư viện time đo lường thời gian thực thi
import subprocess  # Nhập thư viện subprocess thực thi tiến trình Rust Engine

def async_upload_worker(api, chunk_file, cloud_repo_path, repo, chunk_idx, backup_dir, chunk_file_name, chunk_fen_count):
    """
    Hàm `async_upload_worker`: Luồng ngầm tải tệp dữ liệu chunk lên Hugging Face Hub bất đồng bộ.
    Giúp 4 luồng Search CPU không bao giờ bị dừng chờ mạng!
    """
    if api and os.path.exists(chunk_file):
        size_mb = os.path.getsize(chunk_file) / (1024 * 1024)
        print(f"📤 [CHUNK {chunk_idx}] [ASYNC CLOUD SYNC] Bắt đầu tải ngầm lên HuggingFace ({size_mb:.2f} MB | {chunk_fen_count:,} FENs)...", flush=True)
        try:
            api.upload_file(
                path_or_fileobj=chunk_file,
                path_in_repo=cloud_repo_path,
                repo_id=repo,
                repo_type="dataset"
            )
            print(f"✔ [CHUNK {chunk_idx}] [ASYNC CLOUD SYNC] Đồng bộ Cloud `{cloud_repo_path}` thành công!", flush=True)
            if os.path.exists(chunk_file):
                os.remove(chunk_file)
                print(f"🧹 [CHUNK {chunk_idx}] [ASYNC CLOUD SYNC] Đã dọn dẹp tệp đĩa `os.remove({chunk_file})`.", flush=True)
        except Exception as e:
            print(f"⚠️ [CHUNK {chunk_idx}] Lỗi đồng bộ Cloud: {e}. Tiến hành bảo toàn tệp đĩa cục bộ.", flush=True)
            if os.path.exists(chunk_file):
                backup_file = os.path.join(backup_dir, chunk_file_name)
                os.rename(chunk_file, backup_file)
                print(f"🛡️ [CHUNK {chunk_idx}] [BẢO TOÀN DỮ LIỆU] Đã chuyển vào sao lưu: `{backup_file}`.", flush=True)

import threading  # Nhập threading hỗ trợ chạy luồng ngầm bất đồng bộ cho Cloud Sync

# ----------------------------------------------------------------------------
# 1. HẰNG SỐ PHIÊN BẢN VÀ CẤU HÌNH DÂY CHUYỀN
# ----------------------------------------------------------------------------
VERSION = "v9.0.0-platinum-500m-rolling"  # Hằng số phiên bản kịch bản
STAMP = "2026-08-13 03:45:00 ICT"  # Hằng số mốc thời gian đóng gói build
REPO = "hoduyquocbao/xiangqi-gen6-platinum-dataset"  # Tên kho chứa Dataset HuggingFace Hub
try:
    from huggingface_hub import HfApi, create_repo
except ImportError:
    HfApi = None
    create_repo = None

def read_token():
    """
    Hàm `read_token`: Đọc token HF an toàn từ biến môi trường hoặc google.colab.userdata.
    """
    token = os.environ.get("HF_TOKEN")  # Thử đọc token từ biến môi trường OS
    if not token:
        try:
            from google.colab import userdata  # Nhập module userdata đọc bí mật Colab
            token = userdata.get("HF_TOKEN")  # Lấy token từ Colab Secret
        except Exception:
            token = None  # Gán None nếu không tìm thấy
    return token

def execute_platinum_miner(chunk_id, games, depth, threads, batch_size, tt_mb, output_path):
    """
    Hàm `execute_platinum_miner`: Kích hoạt động cơ Ultra SOTA Binary Miner v12.0.0 (Example 93).
    """
    # Lập câu lệnh thực thi binary ví dụ 93 (Ultra SOTA Binary Payload Miner)
    cmd = [
        "./target/release/examples/93_ultra_sota_binary_miner"
    ]
    
    # Thiết lập biến môi trường cấu hình động cho tiến trình Rust
    env = os.environ.copy()
    env["AUTO_TUNE"] = "1"  # Kích hoạt tự động dò tìm cấu hình phần cứng tốc độ nhanh nhất!
    env["GAMES"] = str(games)  # Số ván cờ per chunk
    env["DEPTH"] = str(depth)  # Độ sâu tìm kiếm per ply
    env["THREADS"] = str(threads)  # Số luồng CPU vật lý
    env["BATCH_SIZE"] = str(batch_size)  # Điểm vàng GPU Batch size B* = 256
    env["TT_MB"] = str(tt_mb)  # Dung lượng Shared TT RAM
    env["MAX_NODES"] = str(os.environ.get("MAX_NODES", "300000"))  # Giới hạn 300K nút/nước (~0.03s/move)
    env["MAX_PLIES"] = str(os.environ.get("MAX_PLIES", "128"))  # Giới hạn 128 plies/ván chuẩn SOTA
    env["OUTPUT"] = output_path  # Đường dẫn tệp đĩa chunk JSONL
    
    # Thực thi lệnh và xả đệm log trực tiếp per Rule 8.10
    process = subprocess.Popen(
        cmd,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1
    )
    
    # Đọc log dòng theo dòng xả đệm real-time
    for line in iter(process.stdout.readline, ""):
        print(line, end="", flush=True)
        
    process.stdout.close()
    return_code = process.wait()
    return return_code == 0

def format_time(seconds):
    """
    Hàm `format_time`: Định dạng số giây thành chuỗi hiển thị đọc được (Giờ - Phút - Giây).
    """
    hrs = int(seconds // 3600)
    mins = int((seconds % 3600) // 60)
    secs = int(seconds % 60)
    if hrs > 0:
        return f"{hrs}h {mins:02d}m {secs:02d}s"
    elif mins > 0:
        return f"{mins}m {secs:02d}s"
    else:
        return f"{secs}s"

def main():
    """
    Hàm `main`: Điểm khởi chạy dây chuyền cuốn chiếu 500M Platinum Miner với telemetry ETA.
    """
    print("===============================================================================")
    print("💎 XIANGQI-RIM: 500M NNUE PLATINUM MINER ROLLING CHUNK PIPELINE (ETA ENHANCED)")
    print(f"   Engine Version : {VERSION}")
    print(f"   Build Timestamp: {STAMP}")
    print("===============================================================================")
    sys.stdout.flush()

    # Tham số cấu hình chiến dịch 500M (Nâng cấp 1GB RAM Shared TT)
    total_target_chunks = int(os.environ.get("TARGET_CHUNKS", "50"))  # Tổng số chunks
    games_per_chunk = int(os.environ.get("GAMES_PER_CHUNK", "1024"))  # 1024 ván / chunk (Giảm phân mảnh đĩa)
    mining_depth = int(os.environ.get("MINING_DEPTH", "8"))  # Depth 8 SOTA Standard
    threads_count = int(os.environ.get("THREADS", "4"))  # 4 luồng vật lý
    batch_capacity = int(os.environ.get("BATCH_SIZE", "256"))  # B* = 256
    tt_capacity_mb = int(os.environ.get("TT_MB", "1024"))  # 1024MB (1GB) RAM Shared TT!

    # Xác định thư mục trên Cloud dựa theo MINING_DEPTH (Tránh thay lẫn lộn, chuẩn hóa 100%)
    if mining_depth == 4:
        stage_dir = "gen6_depth4_chunks"
        prefix = "chunk_gen6_d4"
    elif mining_depth == 8:
        stage_dir = "gen8_depth8_chunks"
        prefix = "chunk_gen8_d8"
    else:
        stage_dir = f"gen{mining_depth}_depth{mining_depth}_chunks"
        prefix = f"chunk_gen{mining_depth}_d{mining_depth}"

    backup_dir = f"data/backed_up_chunks/{stage_dir}"
    os.makedirs(backup_dir, exist_ok=True)

    token = read_token()
    api = None
    existing_max_idx = 0
    if token:
        try:
            create_repo(repo_id=REPO, repo_type="dataset", token=token, exist_ok=True)
            api = HfApi(token=token)
            print(f"✔ Đã kết nối kho chứa HuggingFace Dataset: {REPO}")

            # 🌟 CƠ CHẾ TỰ ĐỘNG KHÁM PHÁ BẢO TOÀN DỮ LIỆU CŨ (CRDT AUTO-DISCOVERY PROTOCOL)
            repo_files = api.list_repo_files(repo_id=REPO, repo_type="dataset")
            import re
            for fname in repo_files:
                if fname.startswith(f"{stage_dir}/"):
                    m = re.search(r"(\d{4,6})\.jsonl", fname)
                    if m:
                        idx_val = int(m.group(1))
                        if idx_val > existing_max_idx:
                            existing_max_idx = idx_val
            if existing_max_idx > 0:
                print(f"✔ [CRDT SMART DISCOVERY] Đã phát hiện {existing_max_idx} lô dữ liệu cũ trong `{stage_dir}/` trên Cloud! Tự động nối tiếp từ Lô #{existing_max_idx + 1:04d} (CẤM GHI ĐÈ 100%).")
        except Exception as e:
            print(f"⚠️ Cảnh báo kết nối kho HuggingFace: {e}. Sẽ bảo toàn dữ liệu cục bộ.")
    else:
        print("⚠️ CHƯA ĐẶT HF_TOKEN TRÊN MACBOOK. Dữ liệu đào sẽ được BẢO TOÀN VĨNH CỬU CỤC BỘ (Safe Local Backup).")
    sys.stdout.flush()

    session_tag = time.strftime('%Y%m%d_%H%M%S')  # Mốc thời gian duy nhất cho phiên đào hiện tại

    print("\n⚡ THÔNG SỐ CHIẾN DỊCH KHAI THÁC 500M AN TOÀN BẢO TOÀN DỮ LIỆU:")
    print(f"   • Tên phiên làm việc duy nhất : {session_tag}")
    print(f"   • Phân loại Kho Cloud        : `{stage_dir}/` (Chuẩn hóa 3 Giai Đoạn)")
    print(f"   • Chỉ số lô bắt đầu (Start)   : Lô #{existing_max_idx + 1:04d}")
    print(f"   • Số lô cuốn chiếu mục tiêu : {total_target_chunks} Chunks")
    print(f"   • Số ván cờ mỗi lô (Chunk) : {games_per_chunk} Ván")
    print(f"   • Độ sâu tìm kiếm (Depth)  : Depth {mining_depth}")
    print(f"   • Luồng CPU Workers        : {threads_count} Luồng vật lý")
    print(f"   • Điểm vàng GPU Batch      : B* = {batch_capacity}")
    print(f"   • Dung lượng Shared TT     : {tt_capacity_mb} MB RAM")
    print(f"   • Thư mục sao lưu an toàn : {backup_dir}")
    print("-------------------------------------------------------------------------------\n")
    sys.stdout.flush()

    start_campaign = time.time()
    total_fen_harvested = 0

    start_idx = existing_max_idx + 1
    end_idx = existing_max_idx + total_target_chunks

    for loop_counter, chunk_idx in enumerate(range(start_idx, end_idx + 1), 1):
        chunk_file_name = f"{prefix}_{session_tag}_{chunk_idx:04d}.jsonl"
        chunk_file = f"data/{chunk_file_name}"  # Tên tệp chunk cục bộ độc nhất
        cloud_repo_path = f"{stage_dir}/{chunk_file_name}"  # Đường dẫn kho Cloud độc nhất (Giai đoạn minh bạch)

        print(f"\n🚀 [CHUNK {chunk_idx}/{end_idx} | Lô #{loop_counter}/{total_target_chunks}] Bắt đầu khai thác {games_per_chunk} ván cờ...")
        sys.stdout.flush()
        sys.stdout.flush()
        
        chunk_start_time = time.time()
        # 1. Khai thác dữ liệu chunk bằng Rust Engine
        success = execute_platinum_miner(
            chunk_id=chunk_idx,
            games=games_per_chunk,
            depth=mining_depth,
            threads=threads_count,
            batch_size=batch_capacity,
            tt_mb=tt_capacity_mb,
            output_path=chunk_file
        )
        
        if not success:
            print(f"⚠️ Cảnh báo lỗi tiến trình tại Chunk {chunk_idx}. Vẫn bảo toàn các chunk đã đào.")

        # Đếm số mẫu FEN thu hoạch được trong chunk hiện tại (nếu có)
        chunk_fen_count = 0
        if os.path.exists(chunk_file):
            try:
                with open(chunk_file, "r", encoding="utf-8") as f:
                    chunk_fen_count = sum(1 for line in f if line.strip())
                total_fen_harvested += chunk_fen_count
            except Exception:
                pass
            
        # 2. KHỞI CHẠY LUỒNG NGẦM TẢI DỮ LIỆU LÊN CLOUD HUGGINGFACE (ASYNC BACKGROUND UPLOAD)
        # Giúp 4 luồng Search CPU không bao giờ bị dừng chờ mạng, duy trì 100% CPU liên tục!
        if api and os.path.exists(chunk_file):
            upload_thread = threading.Thread(
                target=async_upload_worker,
                args=(api, chunk_file, cloud_repo_path, REPO, chunk_idx, backup_dir, chunk_file_name, chunk_fen_count),
                daemon=True
            )
            upload_thread.start()
        elif os.path.exists(chunk_file):
            backup_file = os.path.join(backup_dir, chunk_file_name)
            os.rename(chunk_file, backup_file)
            print(f"🛡️ [CHUNK {chunk_idx}] [BẢO TOÀN DỮ LIỆU] Đã chuyển vào sao lưu cục bộ: `{backup_file}`.", flush=True)

        # 4. TÍNH TOÁN BẢNG TELEMETRY VÀ THỜI GIAN DỰ KIẾN CÒN LẠI (ETA TELEMETRY)
        elapsed_sec = time.time() - start_campaign
        completed_chunks = loop_counter
        remaining_chunks = total_target_chunks - completed_chunks
        avg_time_per_chunk = elapsed_sec / completed_chunks
        eta_sec = avg_time_per_chunk * remaining_chunks
        progress_pct = (completed_chunks / total_target_chunks) * 100.0
        finish_timestamp = time.strftime('%Y-%m-%d %H:%M:%S ICT', time.localtime(time.time() + eta_sec))

        print("\n-------------------------------------------------------------------------------")
        print(f"📊 [BÁO CÁO TIẾN ĐỘ CHUNK {chunk_idx}/{total_target_chunks} | {progress_pct:.1f}% Hoàn Thành]")
        print(f"   • Thời gian đã chạy (Elapsed)      : {format_time(elapsed_sec)}")
        print(f"   • Thời gian trung bình mỗi lô      : {format_time(avg_time_per_chunk)} / chunk")
        print(f"   • THỜI GIAN DỰ KIẾN CÒN LẠI (ETA)  : {format_time(eta_sec)}")
        print(f"   • MỐC THỜI GIAN HOÀN THÀNH DỰ KIẾN : {finish_timestamp}")
        print(f"   • Tổng số mẫu FEN đã thu hoạch     : {total_fen_harvested:,} mẫu FEN")
        print("-------------------------------------------------------------------------------\n")
        sys.stdout.flush()

    total_time = time.time() - start_campaign
    print("\n===============================================================================")
    print("🏆 HOÀN THÀNH CHIẾN DỊCH KHAI THÁC CUỐN CHIẾU 500M PLATINUM DATASET!")
    print(f"   • Tổng thời gian chiến dịch: {format_time(total_time)} ({total_time / 3600:.2f} giờ)")
    print(f"   • Tổng số mẫu FEN thu hoạch: {total_fen_harvested:,} mẫu hợp lệ")
    print(f"   • Kho chứa HuggingFace Hub  : https://huggingface.co/datasets/{REPO}")
    print("===============================================================================")
    sys.stdout.flush()

if __name__ == "__main__":
    main()
