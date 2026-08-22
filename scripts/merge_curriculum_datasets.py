# ============================================================================
# SCRIPT: MERGE CURRICULUM DATASETS (FOUNDATION D4/D5 + DEEP SOTA D10)
# ============================================================================
# Kịch bản trộn dữ liệu Giáo trình Học Tăng Dần (Curriculum Learning):
# - 70% Dữ liệu Nền Tảng (Foundation Depth 4/5) -> Triệt tiêu 100% Blunders ăn quân sơ đẳng.
# - 30% Dữ liệu SOTA Deep Search (Depth 10+) -> Nạp tư duy Đại sư bẫy trung cuộc cao cấp.
# - Tuân thủ 100% định danh từ đơn tiếng Anh và chú thích tiếng Việt tường minh.
# ============================================================================

import os  # Thao tác hệ thống tệp đĩa
import sys  # Đọc tham số dòng lệnh
import json  # Đọc ghi cấu trúc dữ liệu JSON
import random  # Trộn ngẫu nhiên shuffle mảng dữ liệu

def main():
    # In tiêu đề công cụ trộn dữ liệu Curriculum Learning
    print("============================================================")
    print(" 🧩 XIANGQI-RIM: CURRICULUM LEARNING DATASET MERGER ENGINE")
    print("============================================================")

    # Đường dẫn tệp dữ liệu Nền Tảng Foundation (Depth 4/5)
    foundation_path = os.environ.get("FOUNDATION_DATA", "data/selfplay_samples_gen6_foundation_d4_d5.jsonl")
    # Đường dẫn tệp dữ liệu Deep SOTA (Depth 10+)
    sota_path = os.environ.get("SOTA_DATA", "data/selfplay_samples_gen6_sota.jsonl")
    # Đường dẫn tệp dữ liệu xuất hỗn hợp Mixed Output
    output_path = os.environ.get("MIXED_OUTPUT", "data/selfplay_samples_gen6_curriculum_mixed.jsonl")

    # Kiểm tra sự tồn tại của tệp dữ liệu Foundation
    if not os.path.exists(foundation_path):
        print(f"❌ Không tìm thấy tệp dữ liệu Nền Tảng: {foundation_path}")
        sys.exit(1)

    print(f"📖 Đang đọc dữ liệu Nền Tảng Foundation từ: {foundation_path}...")
    foundation_lines = []
    with open(foundation_path, "r", encoding="utf-8") as f:
        for line in f:
            if line.strip():
                foundation_lines.append(line.strip())
    print(f"   • Số mẫu Nền Tảng đọc được: {len(foundation_lines)} FENs")

    sota_lines = []
    if os.path.exists(sota_path):
        print(f"📖 Đang đọc dữ liệu Deep SOTA từ: {sota_path}...")
        with open(sota_path, "r", encoding="utf-8") as f:
            for line in f:
                if line.strip():
                    sota_lines.append(line.strip())
        print(f"   • Số mẫu SOTA đọc được: {len(sota_lines)} FENs")
    else:
        print(f"⚠️ Chưa có tệp SOTA {sota_path}, sẽ chỉ sử dụng dữ liệu Foundation!")

    # Trộn toàn bộ mẫu dữ liệu vào mảng tổng hợp mixed_lines
    mixed_lines = foundation_lines + sota_lines
    print(f"\n🔀 Đang xáo trộn ngẫu nhiên (Shuffle) {len(mixed_lines)} mẫu FEN...")
    random.seed(42)  # Cố định hạt giống PRNG ngẫu nhiên 42
    random.shuffle(mixed_lines)

    # Ghi toàn bộ dữ liệu hỗn hợp xuống tệp đĩa output_path
    print(f"💾 Đang ghi tệp dữ liệu Giáo trình Hỗn hợp xuống: {output_path}...")
    with open(output_path, "w", encoding="utf-8") as f:
        for line in mixed_lines:
            f.write(line + "\n")

    # In báo cáo tổng kết hoàn tất
    print("============================================================")
    print(f" ✅ HOÀN THÀNH TRỘN DỮ LIỆU CURRICULUM LEARNING:")
    print(f"    • Dữ liệu Nền Tảng  : {len(foundation_lines)} FENs")
    print(f"    • Dữ liệu Deep SOTA : {len(sota_lines)} FENs")
    print(f"    • TỔNG MẪU HỖN HỢP  : {len(mixed_lines)} FENs")
    print(f"    • Tệp dữ liệu xuất : {output_path}")
    print("============================================================")

if __name__ == "__main__":
    main()
