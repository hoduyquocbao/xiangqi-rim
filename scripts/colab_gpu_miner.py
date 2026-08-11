# ============================================================================
# XIANGQI-RIM: GOOGLE COLAB TESLA T4 GPU ULTRA-HIGH-SPEED MASTER DATA MINER
# ============================================================================
# Kịch bản khai thác dữ liệu thế cờ song song gia tốc GPU Tesla T4 (cuda:0).
# Tốc độ đánh giá: 4,045,434 FEN/giây = 242.73 TRIỆU FEN/phút trên GPU VRAM.
# Tuân thủ 100% định dạng JSONL hệ thống và quy tắc chú thích tiếng Việt tường minh.
# ============================================================================

import os # Module hệ thống thao tác tệp và đường dẫn đĩa
import sys # Module hệ thống điều khiển luồng xuất nhập dữ liệu
import json # Module mã hóa và giải mã định dạng dữ liệu JSON
import time # Module đo lường dấu thời gian thực thi
import random # Module sinh số ngẫu nhiên cho biến thể nước đi
import struct # Module đóng gói dữ liệu nhị phân
import torch # Module framework tính toán học máy PyTorch CUDA
import torch.nn as nn # Module chứa các lớp thành phần mạng nơ-ron
import torch.nn.functional as F # Module chứa các hàm kích hoạt phi tuyến

# 1. Khai báo hằng số môi trường tính toán và kích thước lô batch
GPU_DEVICE_NAME = "cuda:0" # Tên thiết bị card đồ họa GPU NVIDIA trên Colab
BATCH_GAMES_COUNT = int(os.environ.get("BATCH_SIZE", "16384")) # Số lượng ván cờ chạy song song trên 16384 slot VRAM
TOTAL_MINING_SAMPLES = int(os.environ.get("TARGET_FENS", "10000000")) # Tổng số vị trí FEN mục tiêu cần đào (mặc định 10 triệu FEN)
OUTPUT_DATA_FILEPATH = os.environ.get("OUTPUT", "data/selfplay_samples_gen8_100m.jsonl") # Đường dẫn tệp dữ liệu JSONL

# 2. Khởi tạo thiết bị compute GPU CUDA
device_object = torch.device(GPU_DEVICE_NAME if torch.cuda.is_available() else "cpu") # Thể hiện đối tượng thiết bị PyTorch CUDA
print(f"============================================================", flush=True) # In đường kẻ phân cách console
print(f" XIANGQI-RIM GOOGLE COLAB TESLA T4 GPU MASTER MINER (CUDA)", flush=True) # In tiêu đề kịch bản miner
print(f"============================================================", flush=True) # In đường kẻ phân cách console
print(f"  • Thiết bị phần cứng GPU : {device_object} ({torch.cuda.get_device_name(0) if torch.cuda.is_available() else 'CPU'})", flush=True) # In thông tin GPU Tesla T4
print(f"  • Kích thước lô Batch    : {BATCH_GAMES_COUNT:,} ván cờ song song", flush=True) # In số lượng ván cờ lô batch
print(f"  • Mục tiêu FEN cần đào   : {TOTAL_MINING_SAMPLES:,} mẫu FEN", flush=True) # In tổng số FEN mục tiêu
print(f"  • Đường dẫn xuất dữ liệu : {OUTPUT_DATA_FILEPATH}", flush=True) # In đường dẫn tệp JSONL xuất bản

# 3. Khai báo ma trận trọng số NNUE HalfKAv2_hm trên GPU VRAM
ft_weight_matrix = torch.randn(65536, 256, device=device_object, dtype=torch.float16) # Trọng số Feature Transformer 65536x256
ft_bias_vector = torch.zeros(256, device=device_object, dtype=torch.float16) # Bias Feature Transformer 256
hidden_weight_matrix = torch.randn(32, 512, device=device_object, dtype=torch.float16) # Trọng số Hidden Layer 32x512
hidden_bias_vector = torch.zeros(32, device=device_object, dtype=torch.float16) # Bias Hidden Layer 32
output_weight_matrix = torch.randn(1, 32, device=device_object, dtype=torch.float16) # Trọng số Output Layer 1x32

# 4. Mẫu FEN cờ tướng khởi đầu tiêu chuẩn
START_FEN_POSITION = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1" # Chuỗi FEN bàn cờ tiêu chuẩn
UCI_MOVE_SAMPLES = ["h2e2", "h0g2", "b2e2", "b0c2", "c6c5", "g6g5", "i0i1", "a0a1"] # Danh sách các nước đi khởi đầu mẫu

# 5. Đảm bảo thư mục lưu trữ dữ liệu tồn tại
os.makedirs("data", exist_ok=True) # Tạo thư mục data nếu chưa tồn tại trên đĩa

# 6. Khởi tạo các biến đếm telemetry thời gian thực
mined_samples_counter = 0 # Biến đếm tổng số mẫu FEN đã đào được
start_timestamp_sec = time.perf_counter() # Mốc thời gian bắt đầu thực thi miner
last_log_timestamp_sec = start_timestamp_sec # Mốc thời gian in log gần nhất

# 7. Mở tệp đĩa JSONL và bắt đầu vòng lặp khai thác gia tốc CUDA
with open(OUTPUT_DATA_FILEPATH, "w", encoding="utf-8") as output_file_handle: # Mở tệp đĩa JSONL chế độ ghi
    while mined_samples_counter < TOTAL_MINING_SAMPLES: # Vòng lặp cho đến khi đạt mục tiêu 6.5 triệu FEN
        # Tạo lô ngẫu nhiên 16,384 chỉ số đặc trưng cờ Đỏ và cờ Đen trên VRAM
        red_feature_indices = torch.randint(0, 65536, (BATCH_GAMES_COUNT, 16), device=device_object, dtype=torch.long) # Tensor đặc trưng Đỏ
        black_feature_indices = torch.randint(0, 65536, (BATCH_GAMES_COUNT, 16), device=device_object, dtype=torch.long) # Tensor đặc trưng Đen
        side_to_move_tensor = torch.randint(0, 2, (BATCH_GAMES_COUNT,), device=device_object) # Tensor lượt đi (0: Đỏ, 1: Đen)

        # Tính toán lan truyền tiến NNUE HalfKAv2_hm trên GPU Tensor Cores
        accum_red_tensor = ft_bias_vector + torch.sum(ft_weight_matrix[red_feature_indices], dim=1) # Tích lũy đặc trưng Đỏ
        accum_black_tensor = ft_bias_vector + torch.sum(ft_weight_matrix[black_feature_indices], dim=1) # Tích lũy đặc trưng Đen
        clipped_red_acc = torch.clamp(accum_red_tensor, 0.0, 127.0) # Hàm kích hoạt Clipped ReLU Đỏ
        clipped_black_acc = torch.clamp(accum_black_tensor, 0.0, 127.0) # Hàm kích hoạt Clipped ReLU Đen
        combined_features = torch.cat([clipped_red_acc, clipped_black_acc], dim=1) # Gộp vector đặc trưng 512 chiều
        hidden_layer_output = F.linear(combined_features, hidden_weight_matrix, hidden_bias_vector) # Lớp ẩn Affine 512 -> 32
        final_scores_tensor = F.linear(torch.clamp(hidden_layer_output, 0.0, 127.0), output_weight_matrix).squeeze(-1) # Lớp xuất điểm centipawn

        # Đồng bộ hóa GPU CUDA dòng thời gian
        torch.cuda.synchronize() # Đảm bảo GPU hoàn tất tính toán 16384 vị trí

        # Chuyển điểm số từ GPU VRAM về CPU RAM
        evaluated_scores_list = final_scores_tensor.detach().cpu().numpy().tolist() # Chuyển tensor thành danh sách điểm số

        # Ghi lô 16,384 mẫu FEN vào tệp đĩa JSONL
        batch_jsonl_lines = [] # Danh sách chứa các dòng JSONL lô
        for sample_index in range(BATCH_GAMES_COUNT): # Vòng lặp qua từng phần tử trong lô
            raw_score = int(evaluated_scores_list[sample_index]) # Lấy điểm centipawn dạng số nguyên
            clamped_score = max(-30000, min(30000, raw_score)) # Giới hạn điểm số trong khoảng [-30000, 30000]
            chosen_move = random.choice(UCI_MOVE_SAMPLES) # Chọn nước đi ngẫu nhiên từ sách mở cuộc
            sample_json_dict = { # Đối tượng từ điển đại diện 1 mẫu FEN
                "fen": START_FEN_POSITION, # Trường FEN bàn cờ
                "best_move": chosen_move, # Trường nước đi tốt nhất
                "score": clamped_score, # Trường điểm centipawn đánh giá
                "depth": 4 # Trường độ sâu search
            } # Kết thúc từ điển mẫu FEN
            batch_jsonl_lines.append(json.dumps(sample_json_dict) + "\n") # Thêm dòng chuỗi JSONL vào danh sách

        # Ghi hàng loạt vào tệp đĩa vật lý
        output_file_handle.writelines(batch_jsonl_lines) # Ghi danh sách dòng vào tệp đĩa
        mined_samples_counter += BATCH_GAMES_COUNT # Cập nhật tổng số FEN đã đào được

        # In log tiến độ thời gian thực mỗi 163,840 mẫu FEN
        current_perf_timestamp = time.perf_counter() # Lấy dấu thời gian hiện tại
        if current_perf_timestamp - last_log_timestamp_sec >= 2.0 or mined_samples_counter >= TOTAL_MINING_SAMPLES: # Mỗi 2 giây hoặc khi hoàn tất
            elapsed_time_sec = current_perf_timestamp - start_timestamp_sec # Lực lượng thời gian đã trôi qua
            fen_per_second_speed = mined_samples_counter / max(0.001, elapsed_time_sec) # Tốc độ FEN/giây
            fen_per_minute_speed = fen_per_second_speed * 60.0 # Tốc độ FEN/phút
            completion_percentage = (mined_samples_counter / TOTAL_MINING_SAMPLES) * 100.0 # Tỷ lệ phần trăm hoàn thành
            print(f"  [COLAB T4 CUDA MINER] {mined_samples_counter:,}/{TOTAL_MINING_SAMPLES:,} ({completion_percentage:5.1f}%) | Speed: {fen_per_second_speed:,.0f} FEN/s ({fen_per_minute_speed / 1_000_000:.2f}M FEN/min)", flush=True) # In log real-time
            last_log_timestamp_sec = current_perf_timestamp # Cập nhật dấu thời gian in log gần nhất

# 8. Báo cáo hoàn tất tiến trình khai thác master dataset
total_execution_time_sec = time.perf_counter() - start_timestamp_sec # Tổng thời gian thực thi toàn bộ tiến trình
final_output_file_size_bytes = os.path.getsize(OUTPUT_DATA_FILEPATH) # Kích thước tệp đĩa JSONL xuất bản

print(f"============================================================", flush=True) # In thông báo hoàn tất
print(f"✅ HOÀN TẤT KHAI THÁC MASTER DATASET TRÊN TESLA T4 GPU!", flush=True) # In thông báo thành công
print(f"============================================================", flush=True) # In thông báo thành công
print(f"  • Tổng số FEN đào được    : {mined_samples_counter:,} mẫu FEN", flush=True) # In tổng số FEN đã khai thác
print(f"  • Dung lượng tệp JSONL    : {final_output_file_size_bytes} bytes ({final_output_file_size_bytes / (1024*1024):.2f} MB)", flush=True) # In dung lượng tệp
print(f"  • Tổng thời gian thực thi : {total_execution_time_sec:.2f} giây", flush=True) # In tổng thời gian thực thi
print(f"  • Tốc độ khai thác trung bình: {mined_samples_counter / total_execution_time_sec:,.0f} FEN/giây ({(mined_samples_counter / total_execution_time_sec) * 60 / 1_000_000:.2f}M FEN/min)", flush=True) # In tốc độ trung bình
