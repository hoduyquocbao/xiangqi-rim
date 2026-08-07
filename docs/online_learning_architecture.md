# ĐẶC TẢ KIẾN TRÚC THUẬT TOÁN AI HỌC THÍCH ỨNG ONLINE & LƯU TRỮ TRÍ NHỚ KINH NGHIỆM LÂU DÀI

Tài liệu đặc tả kiến trúc toán học và quy trình vận hành hệ thống **Online Reinforcement Learning**, **Temporal Difference Learning $TD(\lambda)$**, **Experience Replay Memory**, **Blunder Analysis** và **Persistent Storage** cho Engine Cờ Tướng XiangRust.

---

## 1. TỔNG QUAN HỆ THỐNG HỌC THÍCH ỨNG (ONLINE LEARNING OVERVIEW)

XiangRust triển khai cơ chế **Càng Chơi Càng Thông Minh** (Continuous Adaptive Learning) dựa trên các trụ cột toán học:

1. **Temporal Difference Learning $TD(\lambda)$**: Cập nhật giá trị thế cờ liên tục dựa trên hiệu số điểm đánh giá giữa các bước đi kế tiếp.
2. **Experience Replay Buffer (`src/learn/replay.rs`)**: Lưu vết mảng buffer 10,000+ ván đấu để tái huấn luyện ngẫu nhiên, chống hiện tượng "quên thảm họa" (Catastrophic Forgetting).
3. **Blunder Analysis (`src/learn/blunder.rs`)**: Phát hiện chính xác nước đi sai lầm gây sụt giảm điểm số đột ngột để gán điểm phạt (Penalty Bias) cho cặp `(Position, Move)`.
4. **Persistent Memory Store (`src/learn/store.rs`)**: Lưu trữ bền vững dữ liệu kinh nghiệm xuống tệp nhị phân/JSON trên ổ đĩa và tự động đồng bộ vào Bảng băm Zobrist Opening/Endgame Book.
5. **Adaptive Search Manager (`src/learn/adapt.rs`)**: Tự động điều chỉnh độ sâu suy nghĩ và thời gian dựa trên phương trình độ phức tạp bàn cờ (Board Complexity Equation).

---

## 2. PHƯƠNG TRÌNH TOÁN HỌC & CƠ CHẾ VẬN HÀNH

### 2.1 Phương trình Cập nhật $TD(\lambda)$ (Temporal Difference Updating)
$$V(S_t) \leftarrow V(S_t) + \alpha \left[ R_{t+1} + \gamma V(S_{t+1}) - V(S_t) \right] E(S_t)$$

Trong đó:
- $\alpha$: Hệ số học tập (Learning Rate, mặc định 0.05).
- $\gamma$: Hệ số chiết khấu tương lai (Discount Factor, mặc định 0.99).
- $E(S_t)$: Trầm tích vết lịch sử (Eligibility Trace).

### 2.2 Phương trình Độ Phức Tạp Bàn Cờ (Board Complexity Equation)
$$\text{Complexity} = w_1 \cdot \text{Mobility} + w_2 \cdot \text{Captures} + w_3 \cdot \text{Checks} + w_4 \cdot \text{KingDanger}$$

Dựa trên độ phức tạp, thuật toán `Adaptive` điều chỉnh độ sâu tìm kiếm:
$$\text{Target Depth} = \text{Base Depth} + \left\lfloor \frac{\text{Complexity}}{250} \right\rfloor$$

---

## 3. CÁC MODULE MÃ NGUỒN CỐT LÕI (`src/learn/`)

| Tệp mã nguồn | Vai trò & Chức năng |
|---|---|
| [`src/learn/mod.rs`](file:///Users/hdqb/workspaces/backend-xiangqi-ai-debugger-v1/src/learn/mod.rs) | Định nghĩa module `learn`, xuất khẩu các struct `Trainer`, `Store`, `Replay`, `Blunder`, `Trace`, `Adapt`. |
| [`src/learn/trainer.rs`](file:///Users/hdqb/workspaces/backend-xiangqi-ai-debugger-v1/src/learn/trainer.rs) | Bộ quản lý vòng lặp huấn luyện tự đấu (Self-Training Loop) & cập nhật điểm số. |
| [`src/learn/store.rs`](file:///Users/hdqb/workspaces/backend-xiangqi-ai-debugger-v1/src/learn/store.rs) | Lưu trữ trí nhớ kinh nghiệm bền vững xuống ổ đĩa & đồng bộ vào Zobrist Book. |
| [`src/learn/replay.rs`](file:///Users/hdqb/workspaces/backend-xiangqi-ai-debugger-v1/src/learn/replay.rs) | Bộ đệm Experience Replay Buffer căn lề 64-byte lưu vết ván đấu. |
| [`src/learn/blunder.rs`](file:///Users/hdqb/workspaces/backend-xiangqi-ai-debugger-v1/src/learn/blunder.rs) | Phân tích và gán điểm phạt cho các nước đi ngây thơ/sai lầm. |
| [`src/learn/trace.rs`](file:///Users/hdqb/workspaces/backend-xiangqi-ai-debugger-v1/src/learn/trace.rs) | Quản lýEligibility Traces lưu vết chuỗi nước đi trong ván cờ. |
| [`src/learn/adapt.rs`](file:///Users/hdqb/workspaces/backend-xiangqi-ai-debugger-v1/src/learn/adapt.rs) | Bộ điều chỉnh linh hoạt thời gian & độ sâu theo độ phức tạp thế cờ. |

---

## 4. HƯỚNG DẪN CHẠY VÍ DỤ MẪU (`examples/14_online_learning_and_trainer.rs`)

Biên dịch và chạy thử nghiệm ví dụ tự huấn luyện AI thích ứng:

```bash
cargo run --release --example 14_online_learning_and_trainer
```

Ví dụ mẫu sẽ:
1. Khởi tạo AI Engine với bộ trí nhớ kinh nghiệm `Store`.
2. Cho AI tự đấu 10 ván liên tục và phân tích các nước sai lầm.
3. Cập nhật bảng băm Zobrist Opening Book & Endgame Memory với các nước thắng.
4. Ghi bền vững trí nhớ xuống tệp nhị phân ổ đĩa và kiểm tra tính nhất quán sau khi nạp lại.
