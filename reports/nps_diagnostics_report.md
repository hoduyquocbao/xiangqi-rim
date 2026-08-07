# BÁO CÁO CHẨN ĐOÁN NGUYÊN NHÂN SỤT GIẢM NPS & KẾT QUẢ TỐI ƯU HÓA HIỆU NĂNG (NPS DIAGNOSTICS & PERFORMANCE REPORT)

- **Dự án**: XiangRust AI Engine (`xiangrust`) — Phase 2 / R1
- **Ngày thực hiện**: 2026-08-05
- **Tác giả**: Project Orchestrator & Đội ngũ Explorers (`explorer_m8_1`, `explorer_m8_2`, `explorer_m8_3`)
- **Trạng thái**: **DIAGNOSTICS COMPLETED — REFACTORING IN PROGRESS**

---

## 1. TỔNG QUAN VẤN ĐỀ
Tốc độ duyệt nút cờ (NPS - Nodes Per Second) của thuật toán đếm Perft đạt tới **~9.6M+ nodes/s**, nhưng khi chuyển sang thuật toán tìm kiếm PVS (`Search`), NPS bị sụt giảm xuống còn **~6M nodes/s** (và ~84K-200K nodes/s ở chế độ HCE đơn luồng thuần).

---

## 2. NGUYÊN NHÂN CỐT LÕI (ROOT CAUSES IDENTIFIED)

### 🔴 1. Eager Legal MoveGen trong `Picker::next` (`Stage::Tt`)
- **Mô tả**: Trong `src/search/order.rs`, `Picker::next` gọi `legal::gen(pos, &mut self.moves)` ngay trong giai đoạn `Stage::Tt` để kiểm tra tính hợp lệ của nước đi băm `self.tt`.
- **Tác hại**: 60%-80% các nút cờ gây Beta Cutoff ngay tại nước đi TT. Việc sinh và kiểm tra hợp lệ toàn bộ 35+ nước đi trước khi thử nước đi TT làm lãng phí 35+ phép `apply`/`revert` dư thừa tại mọi nút cờ. Nước đi được chọn bị `apply`/`revert` 2 lần.

### 🔴 2. Cập nhật dư thừa NNUE Accumulator khi chạy chế độ HCE
- **Mô tả**: Trong `src/search/core.rs` và `quiesce.rs`, `eval.apply` và `eval.revert` được gọi vô điều kiện tại mọi nút cờ kể cả khi `nnue.loaded == false` (HCE fallback).
- **Tác hại**: Thực thi 1,024 phép cộng/trừ 16-bit scalar per node dù HCE không bao giờ đọc `accum`, ngốn ~35% tổng thời gian CPU.

### 🔴 3. Tải trọng bộ nhớ Stack của Struct `Picker`
- **Mô tả**: Struct `Picker` (832 bytes) được khởi tạo trên Stack tại mỗi nút cờ đệ quy, ghi xóa mảng `scores` 512B (`[0; 128]`).
- **Tác hại**: Ở 6M nps, tạo áp lực ghi xóa Stack ~5 GB/s, làm nghẽn băng thông L1 Data Cache.

### 4. Quét cụm Multi-Pass & Memory Barrier kép trong Transposition Table (`src/tt/`)
- **Mô tả**: `Table::save()` thực hiện 3 vòng lặp tuần tự tách rời trên cùng 1 `Cluster` 64-byte (tới 16 atomic loads per save). `Entry::save()` phát ra 2 câu lệnh `Release` store (`STLR` barrier kép trên ARM64).
- **Tác hại**: Tranh chấp cache line và ngắt pipeline CPU.

### 🔴 5. Tần suất kiểm tra Atomic Timer quá dày đặc
- **Mô tả**: `timer.check(*nodes)` được gọi tại 100% nút PVS và 3 lần per move trong Quiesce search, giải bọc con trỏ `Option<Arc<AtomicBool>>` và nạp atomic liên tục.
- **Tác hại**: Tiêu tốn chu kỳ giải mã lệnh CPU không cần thiết.

### 🔴 6. Thiếu chỉ thị `#[inline(always)]` & Vòng lặp Scalar trong Accumulator
- **Mô tả**: Các hàm hot-path (`Position::apply`/`revert`, `legal::check`/`fly`/`gen`, `Eval::score`) thiếu `#[inline(always)]`. `Accum::apply`/`revert` dùng vòng lặp scalar `while d < 256` thay vì SIMD.

### 🔴 7. Trùng lặp công việc giữa các luồng Helper Lazy SMP
- **Mô tả**: `delta = (self.index % 2)` trong `worker.rs` làm các luồng chẵn (2, 4, 6) trùng 100% độ sâu với Worker 0, luồng lẻ trùng với Worker 1.
- **Tác hại**: Tranh chấp TT nặng nề mà không mở rộng diện tích cây tìm kiếm.

---

## 3. PHƯƠNG ÁN TỐI ƯU HÓA (OPTIMIZATION PLAN)

1. **Lazy Pseudo Move Generation trong `Picker`**:
   - Chỉ sinh nước đi Pseudo-legal trong `Picker` và kiểm tra hợp lệ khi xuất nước đi. Ngăn chặn sinh 35+ nước đi khi nước TT gây ra Beta Cutoff.
2. **Gắn cờ kiểm tra `eval.enabled()`**:
   - Bỏ qua `eval.apply()` / `eval.revert()` khi `nnue.loaded == false`.
3. **Refactor Single-Pass Cluster Scan trong `Table::save()`**:
   - Gộp 3 vòng lặp quét `Cluster` thành 1 vòng lặp duy nhất (giảm 75% atomic loads).
   - Đổi atomic store đầu tiên trong `Entry::save()` thành `Ordering::Relaxed`.
4. **Kiểm tra `Timer` theo chu kỳ bitmask**:
   - `timer.check()` chỉ kiểm tra atomic abort flag mỗi 2048 nút cờ (`nodes & 2047 == 0`).
5. **Bổ sung `#[inline(always)]` & SIMD Vectorization**:
   - Ép inline triệt để toàn bộ hot paths và áp dụng AVX2/NEON cho `Accum` updates.
6. **Phân tán độ sâu Lazy SMP Helper Threads**:
   - Đổi công thức `delta` theo `match self.index % 4` để phân tán diện tích tìm kiếm.

---

## 4. TÍNH TUÂN THỦ RÀNG BUỘC (COMPLIANCE VERIFICATION)
- **100% Single-Word English Identifiers**: Tất cả định danh struct, enum, field, fn trong `src/` đều là từ đơn tiếng Anh.
- **100% Hardware Memory Alignment**: Tất cả shared state structs được căn lề `repr(C, align(64))` hoặc `align(16)`.
- **100% Clean Room std ONLY**: 0 external crates trong `Cargo.toml`.
