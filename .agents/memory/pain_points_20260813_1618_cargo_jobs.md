# BÀI HỌC XƯƠNG MÁU: TRÁNH ÉP CỨNG JOBS IN CARGO CONFIG TRÊN CÁC HẠ TẦNG CLOUD VIRTUAL MACHINES (2026-08-13)

---

## 1. NGUYÊN LÝ THẮT NỔ BỘ ĐỆM & NGHẼN CPU CONTEXT SWITCH (`.cargo/config.toml`)

- **Hiện tượng**: Biên dịch `cargo build --release` trên Google Colab GPU T4 tốn tới ~54 giây.
- **Nguyên nhân gốc rễ**: Tệp `.cargo/config.toml` bị ép cứng `jobs = 8`. Trên hạ tầng Google Colab Free Tier (chỉ có 2 CPU Cores), `jobs = 8` buộc Cargo phải spawn 8 tiến trình `rustc` biên dịch song song trên 2 cores. Việc này gây ra hiện tượng **CPU Context Switching Overhead & L2/L3 Cache Thrashing** nặng nề.
- **Giải pháp khắc phục**: Tuyệt đối **KHÔNG** hardcode `jobs = N` trong `.cargo/config.toml`. Hãy để Cargo tự động gọi `nproc` / `num_cpus` của hệ điều hành host để tối ưu hóa chính xác cho từng hạ tầng (2 luồng trên Colab, 8 luồng trên MacBook).
- **Mã commit đã sửa**: Commit `5483fa2` nhánh `dev/tri-tier-architecture`.
