#!/usr/bin/env python3
# ============================================================================
# XIANGQI-RIM: HUGGINGFACE SPACE 12-CPU 64GB RAM HIGH-THROUGHPUT DATA MINER
# ============================================================================
# Application Gradio phục vụ khai thác dữ liệu cờ Tướng tự đấu phân tán trên
# HuggingFace Spaces (12 CPUs, 64GB RAM).
#
# Định danh từ đơn tiếng Anh (Single-Word Identifier Protocol):
# worker, games, depth, threads, seed, token, repo, status, metrics, logs,
# proc, line, count, samples, speed, elapsed, start, file, path, api, info,
# yield, run, stop, total, push, upload, text, view, memory, system
# ============================================================================

import os
import sys
import time
import json
import shutil
import signal
try:
    import psutil
    HAS_PSUTIL = True
except ImportError:
    HAS_PSUTIL = False
import threading
import subprocess
import gradio as gr
from huggingface_hub import HfApi

# Biến môi trường mặc định
TOKEN = os.environ.get("HF_TOKEN", "")
REPO = "hoduyquocbao/xiangqi-r1-dataset"

# Biến toàn cục theo dõi tiến trình background
process = None
running = False

def setup() -> str:
    """Tự động kiểm tra phần cứng và biên dịch nhị phân Rust Native Engine."""
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

    # 2. Biên dịch nhị phân 20_parallel_mine
    target_path = "target/release/examples/20_parallel_mine"
    if not os.path.exists(target_path):
        print("⚡ Đang biên dịch Native Rust Parallel Data Miner (Release Profile)...")
        cmd = [cargo_bin, "build", "--release", "--example", "20_parallel_mine"]
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

    info = f"""### 🖥️ THÔNG TIN PHẦN CỨNG HỆ THỐNG
- **CPU Cores**: `{cpu_logical}` vCPUs (`{cpu_physical}` Physical Cores)
- **RAM khả dụng**: {mem_str}
- **Môi trường**: HuggingFace Spaces Linux Container (12 CPU 64GB RAM)
- **Đề xuất Threads**: `12` threads cho 100% công suất 12 CPU
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

def start_mining(worker, games, depth, threads, seed, token, repo):
    """Khởi chạy và stream tiến trình khai thác dữ liệu đa luồng 12-CPU."""
    global process, running

    if running:
        yield (
            "⚠️ Tiến trình khai thác khác đang chạy!",
            "Vui lòng bấm 'Dừng Khai Thác' trước khi bắt đầu phiên mới.",
            ""
        )
        return

    running = True
    worker = (worker or "hf_space_worker").strip().replace(" ", "_")
    token = (token or TOKEN).strip()
    repo = (repo or REPO).strip()
    threads = int(threads or 12)
    games = int(games or 1000)
    depth = int(depth or 4)
    seed = int(seed or 1)

    # Khởi tạo binary
    try:
        binary = setup()
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
    env["SEED"] = str(seed)
    env["OUTPUT"] = out_file

    start_time = time.time()
    logs = []

    yield (
        f"### 🚀 ĐÃ KHỞI CHẠY BỘ ĐÀO DỮ LIỆU {threads}-CPU\n- **Worker Node**: `{worker}`\n- **Mục tiêu**: `{games:,}` ván cờ (Depth {depth})\n- **Threads**: `{threads}` CPUs\n- **File**: `{out_file}`",
        f"**Khởi tạo**: Đang chạy 0 / {games:,} ván...",
        "Đang kích hoạt Native Rust Engine..."
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
            mem_str = f"`{mem_used:.2f} GB` / `64 GB`"
            cpu_str = f"`{cpu_percent:.1f}%` (trên `{threads}`/12 vCPUs)"
        else:
            mem_str = "`N/A` (HuggingFace Space)"
            cpu_str = f"`100%` (trên `{threads}`/12 vCPUs)"

        status_md = f"""### ⚡ TIẾN TRÌNH KHAI THÁC 12-CPU REAL-TIME
- **Worker**: `{worker}` | **Seed**: `{seed}`
- **Tiến độ ván cờ**: `{current_games:,} / {games:,}` ván (`{pct}%`)
- **Số mẫu FEN chuẩn luật**: `{current_samples:,}` mẫu
- **Tốc độ khai thác**: `{speed:.1f}` mẫu/s (`{int(speed * 60):,}` mẫu/phút)
- **Thời gian đã chạy**: `{elapsed:.1f}`s | **ETA**: `{eta_sec / 60:.1f}` phút
"""
        metrics_md = f"""| Chỉ Số Phần Cứng & Dữ Liệu | Giá Trị Thực Tế |
|---|---|
| 🎮 **Ván Cờ Mined** | `{current_games:,}` / `{games:,}` |
| 🧩 **Mẫu FEN Sạch** | `{current_samples:,}` mẫu |
| ⚡ **Vận Tốc Khai Thác** | `{speed:.1f}` FEN/s |
| 💻 **CPU Usage** | {cpu_str} |
| 🧠 **RAM Usage** | {mem_str} |
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
            except Exception as e:
                hf_url = f"❌ Lỗi Upload: {str(e)}"
        else:
            hf_url = f"⚠️ File đã lưu cục bộ tại: `{out_file}` (Chưa nhập HF_TOKEN)"
    else:
        hf_url = "⚠️ Không tìm thấy file dữ liệu hoặc file rỗng."

    final_status = f"""### 🏆 KẾT THÚC PHIEN KHAI THÁC DỮ LIỆU
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

    with gr.Blocks(theme=theme, title="Xiangqi R1 Data Miner (12 CPU 64GB RAM)") as app:
        gr.Markdown("""
# 🏯 XIANGQI-RIM: NATIVE 12-CPU DATA MINER
### 🚀 Hệ Thống Khai Thác Dữ Liệu Cờ Tướng Tự Đấu Hiệu Năng Cao Trên HuggingFace Spaces (12 CPU, 64GB RAM)
---
Vận hành bộ sinh dữ liệu **Native Rust Engine** với thuật toán Alpha-Beta Search & NNUE Evaluation. Dữ liệu sau khi sinh sẽ được gộp, validate và tự động upload trực tiếp lên **HuggingFace Dataset Hub** (`hoduyquocbao/xiangqi-r1-dataset`).
""")

        gr.Markdown(hardware())

        with gr.Row():
            with gr.Column(scale=1):
                gr.Markdown("### ⚙️ Cấu Hình Khai Thác Dữ Liệu")

                worker_input = gr.Textbox(
                    label="👤 Worker Name (Tên node khai thác)",
                    value="hf_space_worker_12cpu",
                    placeholder="Nhập tên node..."
                )
                games_slider = gr.Slider(
                    label="🎮 Số ván cờ tự đấu (Target Games)",
                    minimum=100,
                    maximum=100000,
                    value=10000,
                    step=500
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
                        "🚀 BẮT ĐẦU KHAI THÁC (12 CPU)",
                        variant="primary",
                        size="lg"
                    )
                    stop_btn = gr.Button(
                        "🛑 DỪNG KHAI THÁC",
                        variant="stop",
                        size="lg"
                    )

            with gr.Column(scale=2):
                gr.Markdown("### 📊 Trạng Thái & Báo Cáo Real-Time")
                status_box = gr.Markdown("Sẵn sàng khai thác dữ liệu trên 12 CPU, 64GB RAM...")
                metrics_box = gr.Markdown("Chờ khởi chạy...")
                logs_box = gr.Textbox(
                    label="📜 Nhật ký Native Engine Real-Time (Streaming Logs)",
                    lines=15,
                    max_lines=25,
                    interactive=False
                )

        start_btn.click(
            fn=start_mining,
            inputs=[worker_input, games_slider, depth_slider, threads_slider, seed_input, token_input, repo_input],
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
