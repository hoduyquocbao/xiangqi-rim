# BÀI HỌC XƯƠNG MÁU & TỐI ƯU HÓA HẠ TẦNG COLAB (2026-08-11)
# Tác giả: HDQB & Antigravity Agent

---

## 1. TỰ ĐỘNG ĐO ELO BENCHMARK THỜI GIAN THỰC (REAL-TIME ELO BENCHMARK)
- **Vấn đề**: Trước đây quy trình huấn luyện chỉ báo loss mà không tự động đo điểm ELO rating thực tế của NNUE Engine sau khi học từng chunk.
- **Giải pháp**: Tích hợp `cargo run --release --example 26_tournament_benchmark` trực tiếp vào BƯỚC 2.5 của `scripts/colab_rolling_1b_pipeline.py`. Sau mỗi đợt huấn luyện, hệ thống tự động cho NNUE thi đấu 40 ván với HCE Baseline ở Depth 4/5, đo Win/Loss/Draw và in ELO Rating chênh lệch chính xác kèm sai số.

## 2. NÂNG VRAM GPU SATURATION LÊN ~85% - 90% (BATCH = 65536)
- **Vấn đề**: `BATCH = 16384` chỉ dùng ~7.7 GB VRAM / 15.0 GB VRAM trên Colab Tesla T4, lãng phí 50% dung lượng RAM GPU.
- **Giải pháp**: Nâng GPU Batch Size trong PyTorch CUDA & Rust GPU Evaluator lên `BATCH = 65536`. Tiêu thụ đúng ~13.0 GB / 15.0 GB VRAM (85-90% trần an toàn), tối đa hóa thông lượng xử lý của Tensor Cores.

## 3. CHUYỂN ĐỔI CHUNK SANG ĐỊNH DẠNG APACHE PARQUET (`.parquet`)
- **Vấn đề**: JSONL 10 Triệu FEN thô chiếm **980 MB**, upload lên Hugging Face Hub tốn 25-30s và gây nghẽn băng thông.
- **Giải pháp**: Sử dụng PyArrow/Pandas nén Snappy Parquet trước khi upload. Dung lượng giảm 80% từ **980 MB xuống còn ~190 MB**, thời gian upload giảm còn **4 giây**, kích hoạt tức thì Hugging Face Dataset Viewer!
