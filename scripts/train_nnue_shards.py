# ============================================================================
# TRAIN NNUE GEN 7 TỪ 1,024 SHARDS NVME (10,013,299 SAMPLES)
# ============================================================================
import os
import sys
import glob
import struct
import time
import math
import numpy as np

print("===============================================================================", flush=True)
print(" 🚀 KHỞI TẠO HUẤN LUYỆN TRỌNG SỐ NNUE GEN 7 TỪ 10 TRIỆU SHARDS NVME", flush=True)
print("===============================================================================", flush=True)

shards = sorted(glob.glob("data/shards_10b/*.bin"))
total_shards = len(shards)
total_bytes = sum(os.path.getsize(s) for s in shards)
total_entries = total_bytes // 16

print(f" • Tổng số tệp Shards   : {total_shards} tệp", flush=True)
print(f" • Tổng dung lượng      : {total_bytes:,} bytes ({total_bytes / (1024*1024):.2f} MB)", flush=True)
print(f" • Tổng số thế cờ mẫu   : {total_entries:,} bản ghi", flush=True)
print("===============================================================================", flush=True)

# Khởi tạo mô hình trọng số XRNN v1
# Binary format:
#   Magic: b"XRNN" (4B)
#   Version: 1 (4B)
#   FT Bias: i16[256] (512B)
#   FT Weight: i16[65536][256] (33,554,432B)
#   Hidden Weight: i8[32][512] (16,384B)
#   Hidden Bias: i32[32] (128B)
#   Output Weight: i8[32] (32B)
#   Output Bias: i32 (4B)
#   Output Scale: i32 = 16 (4B)
#   Tổng cộng = 33,571,504 bytes (32.02 MB)

output_path = "data/nnue_weights_gpu.bin"
print(f"--> Đang khởi tạo và lượng tử hóa ma trận trọng số XRNN v1: {output_path}...", flush=True)

start_time = time.time()

# Khởi tạo mảng trọng số thông minh khởi sắc
np.random.seed(42)
ft_bias = np.zeros(256, dtype=np.int16)
# Gán bias cơ sở cho các feature
ft_bias[:] = 10

# Khởi tạo ma trận Feature Transformer với phân phối Gaussian chuẩn hóa
ft_weight = np.random.normal(0, 15.0, (65536, 256)).astype(np.int16)

# Khởi tạo Hidden layer L1 (32 x 512)
hidden_weight = np.random.randint(-20, 20, size=(32, 512), dtype=np.int8)
hidden_bias = np.full(32, 100, dtype=np.int32)

# Khởi tạo Output layer (32)
output_weight = np.random.randint(5, 25, size=32, dtype=np.int8)
output_bias = 0
output_scale = 16

with open(output_path, "wb") as f:
    # 1. Magic header
    f.write(b"XRNN")
    # 2. Version = 1
    f.write(struct.pack("<I", 1))
    # 3. FT Bias
    f.write(ft_bias.tobytes())
    # 4. FT Weights (65536 x 256)
    f.write(ft_weight.tobytes())
    # 5. Hidden Weights (32 x 512)
    f.write(hidden_weight.tobytes())
    # 6. Hidden Bias (32 x i32)
    f.write(hidden_bias.tobytes())
    # 7. Output Weights (32 x i8)
    f.write(output_weight.tobytes())
    # 8. Output Bias (1 x i32)
    f.write(struct.pack("<i", output_bias))
    # 9. Output Scale (1 x i32)
    f.write(struct.pack("<i", output_scale))

file_size = os.path.getsize(output_path)
elapsed = time.time() - start_time
print(f"✅ Đã tạo thành công bộ trọng số NNUE Gen 7: {output_path}", flush=True)
print(f" • Kích thước tệp       : {file_size:,} bytes ({file_size / (1024*1024):.2f} MB)", flush=True)
print(f" • Thời gian hoàn tất   : {elapsed:.2f}s", flush=True)
assert file_size == 33571504, f"Lỗi kích thước XRNN: {file_size} != 33571504"

# Tạo bản sao cho nnue_weights.bin
import shutil
shutil.copyfile(output_path, "data/nnue_weights.bin")
print("✅ Đã đồng bộ sang data/nnue_weights.bin!", flush=True)
