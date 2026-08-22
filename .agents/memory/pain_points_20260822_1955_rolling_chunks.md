# BÀI HỌC XƯƠNG MÁU: CƠ CHẾ CUỐN CHIẾU ROLLING CHUNKS (< 100MB) CHO TÁC VỤ 1.000.000 VÁN
# Phiên bản: v34.0.0 | Ngày tạo: 2026-08-22 19:55:00 ICT | Tác giả: Antigravity Agent & HDQB
# Mục đích: Đúc kết kỹ thuật cuốn chiếu Rolling Chunk (Mine ➔ Cloud Sync ➔ Local Purge) đảm bảo an toàn đĩa SSD < 100MB cho 1.000.000 ván cờ.

---

## 1. NGUYÊN TẮC BẢO VỆ ĐĨA SSD CHO TÁC VỤ ĐÀO 1.000.000 VÁN CỜ

1. **Thách Thức Dung Lượng (Disk Overflow Hazard)**:
   - 1.000.000 ván cờ cờ Tướng đa chiều 360 CoT sinh ra khoảng 15,000,000 turns, tương đương **75 GB - 100 GB** dữ liệu văn bản JSONL.
   - Nếu ghi dồn vào một tệp duy nhất, ổ đĩa laptop/MacBook sẽ bị đầy đĩa, gây nghẽn I/O và sập tiến trình giữa chừng.

2. **Giải Pháp Kiến Trúc Rolling Chunks Tự Động (< 100 MB/chunk)**:
   - **Tầng Sink Rust**: Cấu hình `CHUNK_MAX_MB=95.0` (ngưỡng 95 MB). Khi dung lượng chunk hiện tại chuẩn bị vượt ngưỡng, `BufWriter` tự động xả đệm, đóng chunk hiện tại `chunk_00001.jsonl`, và mở ngay `chunk_00002.jsonl`.
   - **Bộ Điều Phối Python (`scripts/rolling_cqrs_360_miner.py`)**: Lắng nghe sự kiện đóng chunk từ Rust qua stdout log, lập tức khởi chạy luồng ngầm `async_upload_worker` tải chunk lên HuggingFace Hub Dataset repository (`hoduyquocbao/xiangqi-r1-360-reasoning-dataset`), và xóa ngay tệp đĩa cục bộ (`os.remove()`) sau khi hoàn tất.
   - **Kết quả**: Ổ đĩa SSD cục bộ luôn được duy trì ở mức **< 100 MB**, triệt tiêu 100% rủi ro tràn đĩa.

---

## 2. LỆNH KHỞI CHẠY CHUẨN QUỐC TẾ

```bash
# 1. Chạy trực tiếp qua Rust Engine với Rolling Chunk tự động:
GAMES=1000000 DEPTH=4 THREADS=4 TRANSFORMERS=4 TT_MB=1024 CHUNK_MAX_MB=95 OUTPUT=data/chunks/xiangqi_r1_360_dataset.jsonl cargo run --release --example 95_cqrs_360_reasoning_generator

# 2. Chạy qua Python Coordinator (Tự động tải lên HuggingFace & Dọn đĩa):
GAMES=1000000 DEPTH=4 THREADS=4 TRANSFORMERS=4 TT_MB=1024 CHUNK_MAX_MB=95 python3 scripts/rolling_cqrs_360_miner.py
```
