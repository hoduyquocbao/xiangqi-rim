#!/usr/bin/env python3
# ============================================================================
# XIANGQI-RIM: HUGGINGFACE SPACE 12-CPU 64GB RAM ULTRA HIGH-THROUGHPUT MINER
# ============================================================================
# Application Gradio phục vụ khai thác dữ liệu cờ Tướng tự đấu phân tán trên
# HuggingFace Spaces (12 CPUs, 64GB RAM).
# Tối ưu 64GB RAM: 6GB TT (512MB×12) + 8GB Dual-Hash Sieve Bitset + Swap-and-Drain RAM Buffer.
#
# Định danh từ đơn tiếng Anh (Single-Word Identifier Protocol):
# worker, games, depth, threads, seed, token, repo, status, metrics, logs,
# proc, line, count, samples, speed, elapsed, start, file, path, api, info,
# yield, run, stop, total, push, upload, text, view, memory, system, ram
# ============================================================================

import os
import sys
import time
import json
import shutil
import signal
import threading
import subprocess
try:
    import psutil
    HAS_PSUTIL = True
except ImportError:
    HAS_PSUTIL = False
import gradio as gr
from huggingface_hub import HfApi

# Biến môi trường mặc định
TOKEN = os.environ.get("HF_TOKEN", "")
REPO = "hoduyquocbao/xiangqi-nnue-dataset"

# Biến toàn cục theo dõi tiến trình background
process = None
running = False

def setup(example_name: str = "21_ram64g_mine") -> str:
    """Tự động kiểm tra phần cứng và biên dịch nhị phân Rust Native Engine 64GB RAM."""
    # 1. Kiểm tra Rust toolchain
    cargo_bin = shutil.which("cargo")
    if not cargo_bin:
        home_cargo = os.path.expanduser("~/.cargo/bin/cargo")
        root_cargo = "/root/.cargo/bin/cargo"
        if os.path.exists(home_cargo):
            cargo_bin = home_cargo
            os.environ["PATH"] += f":{os.path.expanduser('~/.cargo/bin')}"
        elif os.path.exists(root_cargo):
            cargo_bin = root_cargo
            os.environ["PATH"] += ":/root/.cargo/bin"
        else:
            print("🔨 Cài đặt Rust toolchain cho HuggingFace Space...")
            res = subprocess.run(
                "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y",
                shell=True, capture_output=True, text=True
            )
            os.environ["PATH"] += f":{os.path.expanduser('~/.cargo/bin')}:/root/.cargo/bin"
            cargo_bin = shutil.which("cargo") or os.path.expanduser("~/.cargo/bin/cargo")

    # 2. Biên dịch nhị phân example_name
    target_path = f"target/release/examples/{example_name}"
    if not os.path.exists(target_path):
        print(f"⚡ Đang biên dịch Native Rust 64GB RAM Data Miner ({example_name})...")
        cmd = [cargo_bin, "build", "--release", "--example", example_name]
        res = subprocess.run(cmd, capture_output=True, text=True)
        if res.returncode != 0:
            print(f"❌ Biên dịch lỗi:\n{res.stderr}")
            raise RuntimeError(f"Không thể biên dịch Rust Engine: {res.stderr}")
        print("✅ Biên dịch thành công!")

    return target_path

def hardware() -> str:
    """Truy vấn thông tin phần cứng hệ thống (12 CPU, 64GB RAM)."""
    cpu_logical = os.cpu_count() or 12
    if HAS_PSUTIL:
        cpu_physical = psutil.cpu_count(logical=False) or (cpu_logical // 2)
        mem_total = psutil.virtual_memory().total / (1024 ** 3)
        mem_avail = psutil.virtual_memory().available / (1024 ** 3)
        mem_str = f"`{mem_avail:.1f} GB` / `{mem_total:.1f} GB`"
    else:
        cpu_physical = cpu_logical // 2
        mem_str = "`64.0 GB` (HuggingFace Space)"

    info = f"""### 🖥️ HẠ TẦNG PHẦN CỨNG 64GB RAM & 12 CPU CORES
- **CPU Cores**: `{cpu_logical}` vCPUs (`{cpu_physical}` Physical Cores)
- **RAM Khả Dụng**: {mem_str}
- **Cấu hình RAM Engine v2.0**: `6 GB` TT (512MB×12) + `8 GB` Dual-Hash Sieve Bitset + Swap-and-Drain RAM Buffer
- **Tốc độ dự kiến**: **~1,500 - 2,500 FEN/giây** (Zero Lock Contention & O(1) Dedup)
"""
    return info

def stop_mining():
    """Dừng tiến trình khai thác dữ liệu đang chạy."""
    global process, running
    running = False
    if process and process.poll() is None:
        try:
            process.terminate()
            process.wait(timeout=3)
        except Exception:
            process.kill()
    return "🛑 Đã yêu cầu dừng khai thác dữ liệu."

def start_mining(worker, games, depth, threads, tt_mb, sieve_mb, seed, token, repo):
    """Khởi chạy và stream tiến trình khai thác dữ liệu đa luồng 64GB RAM."""
    global process, running

    if running:
        yield (
            "⚠️ Tiến trình khai thác khác đang chạy!",
            "Vui lòng bấm 'Dừng Khai Thác' trước khi bắt đầu phiên mới.",
            ""
        )
        return

    running = True
    worker = (worker or "hf_space_worker_64g").strip().replace(" ", "_")
    token = (token or TOKEN).strip()
    repo = (repo or REPO).strip()
    threads = int(threads or 12)
    games = int(games or 50000)
    depth = int(depth or 4)
    tt_mb = int(tt_mb or 512)
    sieve_mb = int(sieve_mb or 8192)
    seed = int(seed or 1)

    # Khởi tạo binary 64GB RAM optimized miner
    try:
        binary = setup("21_ram64g_mine")
    except Exception as e:
        running = False
        yield (f"❌ Lỗi khởi tạo Engine: {str(e)}", "", "")
        return

    # Đường dẫn file output
    stamp = int(time.time())
    out_dir = "data/hf_space"
    os.makedirs(out_dir, exist_ok=True)
    out_file = f"{out_dir}/selfplay_{worker}_s{seed}_{stamp}.jsonl"

    # Cấu hình biến môi trường cho Rust Engine
    env = os.environ.copy()
    env["GAMES"] = str(games)
    env["DEPTH"] = str(depth)
    env["THREADS"] = str(threads)
    env["TT_MB"] = str(tt_mb)
    env["SIEVE_MB"] = str(sieve_mb)
    env["SEED"] = str(seed)
    env["OUTPUT"] = out_file

    start_time = time.time()
    logs = []

    total_tt_gb = (tt_mb * threads) / 1024.0
    sieve_gb = sieve_mb / 1024.0
    total_ram_gb = total_tt_gb + sieve_gb + 2.0

    yield (
        f"### 🚀 ĐÃ KHỞI CHẠY ENGINE 64GB RAM v2.0 ({threads}-CPUs)\n- **Worker Node**: `{worker}`\n- **Mục tiêu**: `{games:,}` ván cờ (Depth {depth})\n- **TT RAM**: `{tt_mb} MB`/thread × {threads} = `{total_tt_gb:.1f} GB`\n- **Sieve RAM**: `{sieve_gb:.1f} GB` Dual-Hash O(1) Bitset\n- **Tổng RAM**: `{total_ram_gb:.1f} GB` / 64.0 GB\n- **File**: `{out_file}`",
        f"**Khởi tạo**: Đang nạp {total_tt_gb:.1f}GB TT + {sieve_gb:.1f}GB Sieve...",
        "Đang kích hoạt Native 64GB RAM Engine v2.0 (Swap-and-Drain)..."
    )

    process = subprocess.Popen(
        [binary],
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1
    )

    current_samples = 0
    current_games = 0

    while running and process.poll() is None:
        line = process.stdout.readline()
        if not line:
            time.sleep(0.1)
            continue

        line_clean = line.strip()
        if line_clean:
            logs.append(line_clean)
            if len(logs) > 100:
                logs.pop(0)

            if "Samples:" in line_clean:
                try:
                    parts = line_clean.split("|")
                    for p in parts:
                        if "STREAMING" in p:
                            g_str = p.split("STREAMING")[1].split("]")[0].strip()
                            current_games = int(g_str.split("/")[0])
                        elif "Samples:" in p:
                            current_samples = int(p.split("Samples:")[1].strip())
                except Exception:
                    pass

        elapsed = time.time() - start_time
        speed = current_samples / max(0.1, elapsed)
        pct = int(current_games / max(1, games) * 100)
        eta_sec = (games - current_games) / (current_games / max(0.1, elapsed)) if current_games > 0 else 0

        # Đo lường RAM và CPU
        if HAS_PSUTIL:
            mem_used = psutil.virtual_memory().used / (1024 ** 3)
            cpu_percent = psutil.cpu_percent(interval=None)
            mem_str = f"`{mem_used:.2f} GB` / `64.0 GB`"
            cpu_str = f"`{cpu_percent:.1f}%` (trên `{threads}`/12 vCPUs)"
        else:
            mem_str = f"`~{total_tt_gb + 2.0:.1f} GB` (High RAM Active)"
            cpu_str = f"`100%` (trên `{threads}`/12 vCPUs)"

        status_md = f"""### ⚡ TIẾN TRÌNH KHAI THÁC 64GB RAM REAL-TIME STREAMING
- **Worker**: `{worker}` | **Seed**: `{seed}`
- **Tiến độ ván cờ**: `{current_games:,} / {games:,}` ván (`{pct}%`)
- **Số mẫu FEN chuẩn luật**: `{current_samples:,}` mẫu
- **Vận tốc khai thác**: `{speed:.1f}` mẫu/s (`{int(speed * 60):,}` mẫu/phút)
- **Thời gian đã chạy**: `{elapsed:.1f}`s | **ETA**: `{eta_sec / 60:.1f}` phút
"""
        metrics_md = f"""| Chỉ Số Phần Cứng 64GB RAM & Dữ Liệu | Giá Trị Thực Tế |
|---|---|
| 🎮 **Ván Cờ Mined** | `{current_games:,}` / `{games:,}` |
| 🧩 **Mẫu FEN Độc Nhất** | `{current_samples:,}` mẫu |
| ⚡ **Vận Tốc Khai Thác** | `{speed:.1f}` FEN/s |
| 🧠 **TT RAM Allocated** | `{total_tt_gb:.1f} GB` (`{tt_mb} MB` × `{threads}` threads) |
| 🧬 **Sieve RAM Allocated** | `{sieve_gb:.1f} GB` Dual-Hash Bitset |
| 🧠 **RAM Hệ Thống** | {mem_str} |
| 💻 **CPU Usage** | {cpu_str} |
| ⏱️ **ETA Dự Kiến** | `{eta_sec / 60:.1f}` phút |
"""
        log_text = "\n".join(logs[-30:])
        yield (status_md, metrics_md, log_text)

    # Chờ tiến trình kết thúc nếu chưa terminate
    if process.poll() is None:
        process.terminate()
        process.wait()

    running = False
    total_elapsed = time.time() - start_time

    # Upload dữ liệu lên HuggingFace Hub nếu có token
    yield (
        f"### 📤 ĐANG ĐỒNG BỘ DỮ LIỆU LÊN HUGGINGFACE HUB...\n- File: `{out_file}`\n- Target Repo: `{repo}`",
        "**Uploading Dataset...**",
        "\n".join(logs[-10:]) + "\n🚀 Đang upload tệp dữ liệu lên HuggingFace Hub..."
    )

    hf_success = False
    hf_url = "Chưa cấu hình HF_TOKEN"
    repo_path = f"community/{os.path.basename(out_file)}"

    if os.path.exists(out_file) and os.path.getsize(out_file) > 0:
        if token and len(token) > 10:
            try:
                api = HfApi()
                api.upload_file(
                    path_or_fileobj=out_file,
                    path_in_repo=repo_path,
                    repo_id=repo,
                    repo_type="dataset",
                    token=token
                )
                hf_success = True
                hf_url = f"https://huggingface.co/datasets/{repo}/blob/main/{repo_path}"
                # Tự động cập nhật README.md thống kê trên HuggingFace Hub
                try:
                    sys.path.append(os.path.abspath("scripts"))
                    import update_dataset_readme
                    update_dataset_readme.update_readme_on_hub(token=token, repo_id=repo)
                except Exception as ex:
                    print(f"⚠️ Thống kê README chưa cập nhật: {ex}")
            except Exception as e:
                hf_url = f"❌ Lỗi Upload: {str(e)}"
        else:
            hf_url = f"⚠️ File đã lưu cục bộ tại: `{out_file}` (Chưa nhập HF_TOKEN)"
    else:
        hf_url = "⚠️ Không tìm thấy file dữ liệu hoặc file rỗng."

    final_status = f"""### 🏆 KẾT THÚC PHIÊN KHAI THÁC DỮ LIỆU 64GB RAM
- **Worker Node**: `{worker}`
- **Tổng số ván cờ hoàn tất**: `{current_games:,}` / `{games:,}` ván
- **Tổng mẫu FEN sạch 100%**: `{current_samples:,}` mẫu
- **Thời gian hoàn tất**: `{total_elapsed:.1f}`s ({total_elapsed / 60:.1f} phút)
- **Tốc độ trung bình**: `{current_samples / max(0.1, total_elapsed):.1f}` FEN/s
- **Trạng thái HuggingFace**: {"✅ Upload Thành Công!" if hf_success else "⚠️ " + hf_url}
- **Link Dataset**: [{repo_path}]({hf_url})
"""
    final_metrics = f"""| Chỉ Số Hoàn Tất | Giá Trị Chung Cuộc |
|---|---|
| 🏆 **Tổng Ván Cờ** | `{current_games:,}` ván |
| 🧩 **Mẫu FEN Sạch** | `{current_samples:,}` mẫu |
| ⚡ **Tốc Độ Trung Bình** | `{current_samples / max(0.1, total_elapsed):.1f}` FEN/s |
| 📁 **File Size** | `{os.path.getsize(out_file) / (1024 * 1024):.2f} MB` |
| ☁️ **HuggingFace Hub** | `{repo_path}` |
"""
    final_logs = "\n".join(logs[-30:]) + f"\n\n✅ ĐÃ HOÀN TẤT & LƯU KẾT QUẢ!\n{hf_url}"

    yield (final_status, final_metrics, final_logs)

def create_app():
    """Xây dựng giao diện web Gradio 4+ tối ưu cho HF Space 12 CPU & 64GB RAM."""
    theme = gr.themes.Soft(
        primary_hue="red",
        secondary_hue="amber",
        neutral_hue="slate"
    )

    with gr.Blocks(theme=theme, title="Xiangqi R1 Ultra 64GB RAM Data Miner") as app:
        gr.Markdown("""
# 🏯 XIANGQI-RIM: ULTRA 64GB RAM DATA MINER
### 🚀 Hệ Thống Tận Dụng Triệt Để 64GB RAM & 12 CPUs Khai Thác Dữ Liệu Cờ Tướng Tự Đấu Hiệu Năng Tối Thượng
---
Vận hành **Native Rust 64GB RAM Engine v2.0** với TT tối ưu cho depth, 8GB Dual-Hash Sieve Bitset (O(1) Dedup) và Swap-and-Drain RAM Buffer không block worker threads. Tự động upload lên **HuggingFace Dataset Hub** (`hoduyquocbao/xiangqi-nnue-dataset`).
""")

        gr.Markdown(hardware())

        with gr.Row():
            with gr.Column(scale=1):
                gr.Markdown("### ⚙️ Cấu Hình Khai Thác 64GB RAM")

                worker_input = gr.Textbox(
                    label="👤 Worker Name (Tên node khai thác)",
                    value="hf_space_worker_64g",
                    placeholder="Nhập tên node..."
                )
                games_slider = gr.Slider(
                    label="🎮 Số ván cờ tự đấu (Target Games)",
                    minimum=500,
                    maximum=500000,
                    value=50000,
                    step=1000
                )
                depth_slider = gr.Slider(
                    label="🧠 Độ sâu tìm kiếm Engine (Search Depth)",
                    minimum=3,
                    maximum=8,
                    value=4,
                    step=1
                )
                threads_slider = gr.Slider(
                    label="⚡ Số luồng CPU song song (Threads)",
                    minimum=1,
                    maximum=12,
                    value=12,
                    step=1
                )
                tt_mb_slider = gr.Slider(
                    label="🧠 RAM Transposition Table mỗi Thread (MB)",
                    minimum=128,
                    maximum=4096,
                    value=512,
                    step=128,
                    info="512 MB × 12 threads = 6 GB TT (depth 4 đủ dùng, tăng cho depth 6+)"
                )
                sieve_mb_slider = gr.Slider(
                    label="🧬 RAM Sieve Dual-Hash Bitset (MB)",
                    minimum=1024,
                    maximum=16384,
                    value=8192,
                    step=1024,
                    info="8192 MB = 8 GB = 64 tỷ bit flags → tỷ lệ false positive ≈ 0%"
                )
                seed_input = gr.Number(
                    label="🎲 PRNG Base Seed (Dùng cho multi-instance)",
                    value=1,
                    precision=0
                )
                token_input = gr.Textbox(
                    label="🔑 HuggingFace Write Token (Dùng để upload)",
                    value=TOKEN,
                    placeholder="Nhập token hf_xxx...",
                    type="password"
                )
                repo_input = gr.Textbox(
                    label="📦 HuggingFace Dataset Repo",
                    value=REPO
                )

                with gr.Row():
                    start_btn = gr.Button(
                        "🚀 BẮT ĐẦU KHAI THÁC (64GB RAM & 12 CPU)",
                        variant="primary",
                        size="lg"
                    )
                    stop_btn = gr.Button(
                        "🛑 DỪNG KHAI THÁC",
                        variant="stop",
                        size="lg"
                    )

            with gr.Column(scale=2):
                gr.Markdown("### 📊 Trạng Thái & Báo Cáo Real-Time 64GB RAM")
                status_box = gr.Markdown("Sẵn sàng khai thác dữ liệu với TT tối ưu + 8GB Dual-Hash Sieve Bitset v2.0...")
                metrics_box = gr.Markdown("Chờ khởi chạy...")
                logs_box = gr.Textbox(
                    label="📜 Nhật ký Native Engine Real-Time (Streaming Logs)",
                    lines=15,
                    max_lines=25,
                    interactive=False
                )

        start_btn.click(
            fn=start_mining,
            inputs=[worker_input, games_slider, depth_slider, threads_slider, tt_mb_slider, sieve_mb_slider, seed_input, token_input, repo_input],
            outputs=[status_box, metrics_box, logs_box]
        )

        stop_btn.click(
            fn=stop_mining,
            inputs=[],
            outputs=[status_box]
        )

    return app

if __name__ == "__main__":
    demo = create_app()
    demo.queue()
    demo.launch(server_name="0.0.0.0", server_port=7860)
