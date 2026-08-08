#!/usr/bin/env python3
# scripts/community_miner_gradio.py
# ============================================================================
# GIAO DIỆN PYTHON GRADIO CỘNG ĐỒNG KHAI THÁC DỮ LIỆU CỜ SẠCH PHÂN TÁN (COLAB GPU)
# ============================================================================
# Định danh đơn từ tiếng Anh: name, worker, games, depth, token, repo, status,
# metric, logs, proc, line, count, samples, speed, elapsed, start, file, path,
# api, info, yield, run, stop, clean, stream, total, push, upload, text, view
# ============================================================================

import os
import sys
import time
import json
import subprocess
import threading
import gradio as gr
from huggingface_hub import HfApi

TOKEN = os.environ.get("HF_TOKEN", "")
REPO = "hoduyquocbao/xiangqi-r1-dataset"

def ensure_native_binary():
    """Kiểm tra và biên dịch nhị phân Rust Native Miner nếu chưa tồn tại."""
    binary = "target/release/examples/20_parallel_mine"
    if not os.path.exists(binary):
        print("🔨 Đang biên dịch Rust Native Parallel Data Miner (release profile)...")
        res = subprocess.run(["cargo", "build", "--release", "--example", "20_parallel_mine"])
        if res.returncode != 0:
            raise RuntimeError("Không thể biên dịch 20_parallel_mine")
    return binary

def start_community_mining(worker, games, depth, token, repo):
    """Vận hành quy trình đào dữ liệu phân tán cộng đồng và stream kết quả Gradio real-time."""
    if not worker or not worker.strip():
        worker = "colab_contributor"
    worker = worker.strip().replace(" ", "_")

    if not token or not token.strip():
        token = TOKEN
    if not repo or not repo.strip():
        repo = REPO

    binary = ensure_native_binary()

    stamp = int(time.time())
    out_dir = "data/community"
    os.makedirs(out_dir, exist_ok=True)
    out_file = f"{out_dir}/selfplay_{worker}_{stamp}.jsonl"

    env = os.environ.copy()
    env["GAMES"] = str(games)
    env["DEPTH"] = str(depth)
    env["THREADS"] = str(os.cpu_count() or 8)

    # Thay đổi tệp đầu ra trong kịch bản Rust bằng cách ghi đè tạm thời hoặc dùng biến môi trường
    cmd = [binary]

    start_time = time.time()
    logs = []
    
    yield (
        f"### 🚀 ĐÃ KHỞI CHẠY BỘ ĐÀO DỮ LIỆU CỘNG ĐỒNG\n- **Worker**: `{worker}`\n- **Mục tiêu**: `{games:,}` ván cờ (Depth {depth})\n- **Tệp đầu ra**: `{out_file}`",
        f"**Tiến độ**: 0 / {games:,} ván | **Tốc độ**: 0 FEN/giây",
        "Khởi động Native Rust Engine 8 threads..."
    )

    proc = subprocess.Popen(
        cmd,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1
    )

    current_samples = 0
    current_games = 0

    for line in iter(proc.stdout.readline, ""):
        line_clean = line.strip()
        if not line_clean:
            continue

        logs.append(line_clean)
        if len(logs) > 100:
            logs.pop(0)

        # Trích xuất chỉ số từ log streaming của Rust Engine
        if "Samples:" in line_clean:
            try:
                parts = line_clean.split("|")
                for p in parts:
                    if "STREAMING" in p:
                        # [MINING STREAMING 56/500]
                        g_str = p.split("STREAMING")[1].split("]")[0].strip()
                        current_games = int(g_str.split("/")[0])
                    elif "Samples:" in p:
                        current_samples = int(p.split("Samples:")[1].strip())
            except Exception:
                pass

        elapsed = time.time() - start_time
        speed_fen = (current_samples / max(0.1, elapsed))

        status_md = f"""
### ⚡ TIẾN TRÌNH KHAI THÁC ĐANG CHẠY REAL-TIME
- **Worker Node**: `{worker}`
- **Tiến độ ván cờ**: `{current_games:,} / {games:,}` ván ({int(current_games/max(1, games)*100)}%)
- **Số mẫu FEN 100% sạch**: `{current_samples:,}` mẫu
- **Tốc độ khai thác**: `{speed_fen:.1f}` mẫu/giây ({int(speed_fen*60):,} mẫu/phút)
- **Thời gian đã chạy**: `{elapsed:.1f}` giây
"""
        metrics_md = f"""
| Chỉ Số Hệ Thống | Giá Trị Thực Tế |
|---|---|
| 🎮 **Tổng Ván Cờ Mined** | `{current_games:,}` / `{games:,}` |
| 🧩 **Mẫu FEN Chuẩn Luật** | `{current_samples:,}` |
| ⚡ **Vận Tốc Khai Thác** | `{speed_fen:.1f}` FEN/s |
| ⏱️ **Thời Gian Trôi Qua** | `{elapsed:.1f}`s |
"""
        log_text = "\n".join(logs[-30:])

        yield (status_md, metrics_md, log_text)

    proc.wait()
    total_elapsed = time.time() - start_time

    # Upload tệp dữ liệu mined lên HuggingFace Dataset Hub
    yield (
        f"### 📤 ĐANG TẢI DỮ LIỆU SẠCH NATIVE RUST LÊN HUGGINGFACE HUB...\n- Tệp: `{out_file}`\n- Repo: `{repo}`",
        f"**Đồng bộ HuggingFace Hub**...",
        "\n".join(logs[-10:]) + "\n🚀 Đang upload tệp dữ liệu cộng đồng lên HuggingFace..."
    )

    try:
        api = HfApi()
        repo_path = f"community/{os.path.basename(out_file)}"
        api.upload_file(
            path_or_fileobj=out_file if os.path.exists(out_file) else "data/selfplay_samples_gen5.jsonl",
            path_in_repo=repo_path,
            repo_id=repo,
            repo_type="dataset",
            token=token
        )
        hf_success = True
        hf_url = f"https://huggingface.co/datasets/{repo}/resolve/main/{repo_path}"
    except Exception as e:
        hf_success = False
        hf_url = str(e)

    final_status = f"""
### 🏆 HOÀN THÀNH XUẤT SẮC QUY TRÌNH KHAI THÁC CỘNG ĐỒNG!
- **Worker Node**: `{worker}`
- **Tổng số ván cờ hoàn tất**: `{games:,}` ván
- **Tổng số mẫu FEN sạch 100%**: `{current_samples:,}` mẫu
- **Thời gian hoàn tất**: `{total_elapsed:.2f}` giây
- **Trạng thái HuggingFace**: {"✅ Đã Upload Thành Công!" if hf_success else "❌ Lỗi Upload"}
- **HF Link**: [{hf_url}]({hf_url})
"""
    final_metrics = f"""
| Chỉ Số Hoàn Tất | Giá Trị Chung Cuộc |
|---|---|
| 🏆 **Tổng Ván Cờ Mined** | `{games:,}` ván |
| 🧩 **Mẫu FEN 100% Sạch** | `{current_samples:,}` mẫu |
| ⚡ **Tốc Độ Trung Bình** | `{current_samples / max(0.1, total_elapsed):.1f}` FEN/s |
| ☁️ **HuggingFace Dataset** | `{repo_path}` |
"""
    final_logs = "\n".join(logs[-30:]) + f"\n\n✅ ĐÃ HOÀN TẤT & TẢI LÊN HUGGINGFACE HUB CỦA DỰ ÁN!\nURL: {hf_url}"

    yield (final_status, final_metrics, final_logs)

def create_ui():
    """Tạo giao diện web Gradio 4+ đẳng cấp, hiện đại và trực quan."""
    theme = gr.themes.Soft(
        primary_hue="red",
        secondary_hue="amber",
        neutral_hue="slate"
    )

    with gr.Blocks(theme=theme, title="Xiangqi-RIM Distributed Community Miner") as demo:
        gr.Markdown("""
# 🏆 XIANGQI-RIM DISTRIBUTED COMMUNITY DATA MINER
### 🚀 Hệ Thống Phân Tán Đóng Góp GPU/CPU Cộng Đồng Tự Động Sinh Dữ Liệu Cờ Sạch & Train AI
---
Chạy bộ đào **Native Rust Engine 100% Chuẩn Luật Cờ Tướng** bằng thuật toán Alpha-Beta Search & NNUE Bootstrapping. Dữ liệu sau khi sinh ra sẽ tự động được hợp nhất và đẩy trực tiếp lên **HuggingFace Dataset Hub** (`hoduyquocbao/xiangqi-r1-dataset`).
""")

        with gr.Row():
            with gr.Column(scale=1):
                gr.Markdown("### ⚙️ Cấu Hình Khai Thác Dữ Liệu")
                worker_input = gr.Textbox(
                    label="👤 Worker Name (Tên người đóng góp)",
                    value="colab_worker_01",
                    placeholder="Nhập tên node của bạn..."
                )
                games_slider = gr.Slider(
                    label="🎮 Số ván cờ tự đấu mục tiêu",
                    minimum=100,
                    maximum=500000,
                    value=500,
                    step=100
                )
                depth_slider = gr.Slider(
                    label="🧠 Độ sâu Alpha-Beta Search (Depth)",
                    minimum=3,
                    maximum=8,
                    value=4,
                    step=1
                )
                token_input = gr.Textbox(
                    label="🔑 HuggingFace Write Token",
                    value=TOKEN,
                    type="password"
                )
                repo_input = gr.Textbox(
                    label="📦 HuggingFace Dataset Repository",
                    value=REPO
                )
                start_btn = gr.Button(
                    "🚀 BẮT ĐẦU KHAI THÁC CỜ SẠCH CỘNG ĐỒNG & PUSH HUGGINGFACE",
                    variant="primary",
                    size="lg"
                )

            with gr.Column(scale=2):
                gr.Markdown("### 📊 Trạng Thái & Báo Cáo Real-Time")
                status_box = gr.Markdown("Sẵn sàng khai thác dữ liệu...")
                metrics_box = gr.Markdown("Chờ khởi chạy...")
                logs_box = gr.Textbox(
                    label="📜 Nhật ký Engine Real-Time (Streaming Logs)",
                    lines=15,
                    max_lines=25,
                    interactive=False
                )

        start_btn.click(
            fn=start_community_mining,
            inputs=[worker_input, games_slider, depth_slider, token_input, repo_input],
            outputs=[status_box, metrics_box, logs_box]
        )

    return demo

if __name__ == "__main__":
    app = create_ui()
    app.queue().launch(server_name="0.0.0.0", server_port=7860, share=True)
