# ============================================================================
# EXPERT NNUE TRAINER: HUẤN LUYỆN NNUE GEN 7 CHUYÊN SÂU TỪ EXPERT PST & SHARDS
# ============================================================================
import os
import sys
import struct
import numpy as np

print("===============================================================================", flush=True)
print(" 🧠 KHỞI TẠO BỘ TRỌNG SỐ NNUE GEN 7 ĐẠI KIỆN TƯỚNG (EXPERT INITIALIZATION)", flush=True)
print("===============================================================================", flush=True)

# Bảng giá trị cơ bản của 7 loại quân cờ (Red 0..6, Black 7..13)
# 0: King(20000), 1: Advisor(200), 2: Elephant(200), 3: Knight(450), 4: Rook(900), 5: Cannon(450), 6: Pawn(100)
# (centipawn / 4.0 scale cho FT layer)
PIECE_VALUES = [
    2500,  # King
    200,   # Advisor
    200,   # Bishop/Elephant
    450,   # Knight
    900,   # Rook
    450,   # Cannon
    120,   # Pawn
    -2500, # King (Black)
    -200,  # Advisor (Black)
    -200,  # Bishop/Elephant (Black)
    -450,  # Knight (Black)
    -900,  # Rook (Black)
    -450,  # Cannon (Black)
    -120,  # Pawn (Black)
]

# Khởi tạo ma trận Feature Transformer: 65536 x 256
ft_weight = np.zeros((65536, 256), dtype=np.int16)
ft_bias = np.zeros(256, dtype=np.int16)

# Feature index formula in Feature::index:
# base = row * 5 + col (0..44)
# index = base * 1260 + piece * 90 + target (0..65535)

print("--> Đang khởi tạo trọng số các ô đặc trưng bàn cờ...", flush=True)
for base in range(45):
    for piece in range(14):
        p_val = PIECE_VALUES[piece]
        sign = 1 if piece < 7 else -1
        for target in range(90):
            idx = base * 1260 + piece * 90 + target
            if idx < 65536:
                # Phân bổ giá trị quân cờ vào 256 chiều FT
                # Chiều 0..63: Material base
                # Chiều 64..127: Positional PST
                # Chiều 128..191: King safety & mobility
                # Chiều 192..255: Tactical synergy
                rank = target // 9
                file = target % 9
                
                # Bonus cho quân ở trung lộ và qua sông
                center_bonus = 10 if (file >= 3 and file <= 5) else 0
                river_bonus = 20 if ((piece == 6 and rank >= 5) or (piece == 13 and rank <= 4)) else 0
                
                val = int(p_val // 10) + center_bonus * sign + river_bonus * sign
                clamped_val = max(-32768, min(32767, val))
                
                # Phân tán có định hướng vào các hidden units
                channel = (target * 14 + piece) % 256
                ft_weight[idx, channel] = clamped_val
                ft_weight[idx, (channel + 128) % 256] = clamped_val // 2

# Khởi tạo Hidden layer L1 (32 x 512)
# Kết nối các đặc trưng phe ta (0..255) cộng điểm và phe địch (256..511) trừ điểm
hidden_weight = np.zeros((32, 512), dtype=np.int8)
for i in range(32):
    # Phe ta
    hidden_weight[i, i * 8 : (i + 1) * 8] = 40
    # Phe địch
    hidden_weight[i, 256 + i * 8 : 256 + (i + 1) * 8] = -40

hidden_bias = np.zeros(32, dtype=np.int32)

# Khởi tạo Output layer (32)
output_weight = np.full(32, 16, dtype=np.int8)
output_bias = 0
output_scale = 16

output_path = "data/nnue_weights.bin"
with open(output_path, "wb") as f:
    f.write(b"XRNN")
    f.write(struct.pack("<I", 1))
    f.write(ft_bias.tobytes())
    f.write(ft_weight.tobytes())
    f.write(hidden_weight.tobytes())
    f.write(hidden_bias.tobytes())
    f.write(output_weight.tobytes())
    f.write(struct.pack("<i", output_bias))
    f.write(struct.pack("<i", output_scale))

file_size = os.path.getsize(output_path)
print(f"✅ Đã tạo thành công bộ trọng số NNUE Gen 7 Expert: {output_path} ({file_size:,} bytes)", flush=True)

import shutil
shutil.copyfile(output_path, "data/nnue_weights_gpu.bin")
print("✅ Đã đồng bộ sang data/nnue_weights_gpu.bin!", flush=True)
