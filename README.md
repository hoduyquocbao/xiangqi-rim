---
title: Xiangqi R1 Data Miner
emoji: 🏯
colorFrom: red
colorTo: amber
sdk: gradio
sdk_version: 5.16.0
app_file: app.py
pinned: false
license: mit
short_description: High-throughput 12-CPU 64GB RAM Xiangqi self-play data miner
---

# 🏯 Xiangqi-RIM: HuggingFace Space 12-CPU Data Miner

Dịch vụ ứng dụng Gradio khai thác dữ liệu cờ Tướng tự đấu phân tán trên **HuggingFace Spaces** (12 CPUs, 64GB RAM).

## 🚀 Tính Năng Nổi Bật

- **Tối ưu 12-CPU**: Tự động tận dụng tối đa 12 luồng CPU song song trên HF Space container.
- **Tự động biên dịch Rust**: Tự kiểm tra và cài đặt Rust toolchain + biên dịch `20_parallel_mine` tự động khi ứng dụng khởi chạy.
- **Báo cáo Real-Time**: Theo dõi số mẫu FEN, số ván cờ, vận tốc khai thác (samples/s), CPU usage, RAM usage (trên 64GB RAM) theo thời gian thực.
- **Tự động Upload HuggingFace**: Tự động đẩy tập dữ liệu mined lên `hoduyquocbao/xiangqi-r1-dataset` khi hoàn thành ván cờ.

## 🛠️ Hướng Dẫn Deploy Lên HuggingFace Space

1. Tạo một **Space mới** trên HuggingFace: https://huggingface.co/new-space
2. Chọn **Space SDK: Gradio**
3. Cài đặt biến môi trường Secret:
   - Key: `HF_TOKEN`
   - Value: `hf_xxx...` (Write token của bạn)
4. Push toàn bộ mã nguồn repo này vào HuggingFace Space:
   ```bash
   git remote add space https://huggingface.co/spaces/YOUR_USERNAME/xiangqi-r1-miner
   git push space main
   ```
