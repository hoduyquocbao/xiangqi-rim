# 📚 Thư Mục Jupyter Notebooks — Xiangqi-R1 Ecosystem

Chào mừng bạn đến với thư mục Jupyter Notebooks của dự án **Xiangqi-R1**!
Toàn bộ các notebook đã được phân loại theo chỉ mục trực quan (`01_`, `02_`, `03_`, `04_`) phục vụ từ người đóng góp cộng đồng cho tới nhà phát triển hệ thống.

---

## 🗺️ Bảng Chỉ Mục Phân Loại Notebooks

```
notebooks/
├── 01_community_mining/            # Phase 1: Đóng góp Khai thác Dữ liệu cờ Tướng
│   ├── 01_mine_jrcp3_depth12.ipynb  # 🔥 KHUYÊN DÙNG #1: Miner JRCP 3.0 Depth 12 (1-Click, Auto Push, SIGTERM, Checkpoint)
│   └── 02_colab_mining_t4_fast.ipynb# GPU T4 Mining Siêu Tốc (Vectorized PyTorch Eval 200K pos/s)
│
├── 02_community_training/          # Phase 2: Đóng góp Huấn luyện Mô hình LLM (GRPO)
│   ├── 01_train_grpo_lora.ipynb     # 🔥 KHUYÊN DÙNG #2: GRPO Trainer Unsloth 4-bit LoRA (3 Rewards, Resume)
│   └── 02_train_grpo_legacy.ipynb   # GRPO Trainer phiên bản cơ sở
│
├── 03_nnue_training/               # Phase 3: Huấn luyện Mạng NNUE Nhị Phân cho Engine Rust
│   ├── 01_train_nnue_xrnn.ipynb     # 🔥 KHUYÊN DÙNG #3: PyTorch EmbeddingBag NNUE Trainer (Xuất weights .bin 33.5MB)
│   └── 02_gradio_miner_and_nnue.ipynb# Gradio Miner Web GUI & NNUE Trainer
│
└── 04_core_maintainer/             # Phase 4: Công cụ Dành cho Core Developers
    ├── 01_master_llm_grpo_train.ipynb# Master LLM GRPO Training (Qwen 0.5B / 0.8B / 7B, Merge 16-bit)
    ├── 02_gradio_mcp_backend.ipynb  # Tesla T4 GPU MCP Server Backend 92.99M NPS
    └── 03_xiangqi_rim_full.ipynb    # Pipeline tổng hợp quy mô lớn
```

---

## 🚀 Hướng Dẫn Nhanh Cho Người Mới Đóng Góp

1. **Nếu bạn muốn treo Colab đóng góp dữ liệu (Dễ nhất & Hiệu quả nhất)**:
   👉 Mở **[`notebooks/01_community_mining/01_mine_jrcp3_depth12.ipynb`](file:///Users/hdqb/workspaces/xiangqi-rim/notebooks/01_community_mining/01_mine_jrcp3_depth12.ipynb)** → Gắn Colab T4 GPU → Nhập secret `HF_TOKEN` → Bấm **Run All**.

2. **Nếu bạn muốn đóng góp GPU T4 huấn luyện LoRA Adapter (~30 phút)**:
   👉 Mở **[`notebooks/02_community_training/01_train_grpo_lora.ipynb`](file:///Users/hdqb/workspaces/xiangqi-rim/notebooks/02_community_training/01_train_grpo_lora.ipynb)** → Bấm **Run All**.

3. **Nếu bạn muốn huấn luyện mô hình NNUE cho Engine Rust (~20 phút)**:
   👉 Mở **[`notebooks/03_nnue_training/01_train_nnue_xrnn.ipynb`](file:///Users/hdqb/workspaces/xiangqi-rim/notebooks/03_nnue_training/01_train_nnue_xrnn.ipynb)** → Bấm **Run All**.

---

### 🛡️ Tính Năng An Toàn Được Đảm Bảo 100%
- **Graceful Shutdown (`SIGTERM`/`SIGINT`)**: Tự động lưu checkpoint và push dữ liệu dở dang lên HuggingFace khi session bị ngắt.
- **Auto Push Periodic**: Đẩy dữ liệu lên HuggingFace Dataset Hub định kỳ mỗi 5 phút.
- **Checkpoint & Resume**: Tự động khôi phục tiến trình mining/training khi rerun.
