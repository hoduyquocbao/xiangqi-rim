#!/usr/bin/env python3
# ============================================================================
# XIANGQI-RIM: SCRIPT GỘP & XÁC MINH DỮ LIỆU MINING ĐA INSTANCE
# ============================================================================
# Gộp nhiều tệp JSONL từ các Colab instance khác nhau thành 1 tệp duy nhất.
# Loại bỏ mẫu trùng lặp theo FEN, xác minh tính hợp lệ của mọi trường dữ liệu.
#
# Cách sử dụng:
#   python3 scripts/merge_mining_results.py data/gen6_part*.jsonl -o data/merged.jsonl
#   python3 scripts/merge_mining_results.py file1.jsonl file2.jsonl file3.jsonl -o output.jsonl
# ============================================================================

import argparse
import json
import os
import sys
from collections import Counter


def validate(sample: dict, index: int, source: str) -> list:
    """Xác minh 1 mẫu dữ liệu JSONL theo tiêu chuẩn engine Xiangqi-RIM."""
    errors = []

    # Kiểm tra trường bắt buộc
    required = ["fen", "best_move", "score", "depth"]
    for field in required:
        if field not in sample:
            errors.append(f"[{source}:{index}] Thiếu trường bắt buộc: '{field}'")

    # Kiểm tra FEN không rỗng
    fen = sample.get("fen", "")
    if not isinstance(fen, str) or len(fen) < 10:
        errors.append(f"[{source}:{index}] FEN không hợp lệ: '{fen}'")

    # Kiểm tra best_move format (ví dụ: a0a1, i9i7)
    move = sample.get("best_move", "")
    if not isinstance(move, str) or len(move) != 4:
        errors.append(f"[{source}:{index}] best_move không hợp lệ: '{move}'")

    # Kiểm tra score là số nguyên
    score = sample.get("score")
    if not isinstance(score, (int, float)):
        errors.append(f"[{source}:{index}] score không phải số: {score}")

    # Kiểm tra depth là số nguyên dương
    depth = sample.get("depth")
    if not isinstance(depth, int) or depth < 1:
        errors.append(f"[{source}:{index}] depth không hợp lệ: {depth}")

    # Kiểm tra trường cấm (field name cũ)
    if "eval" in sample:
        errors.append(f"[{source}:{index}] Phát hiện trường cấm 'eval' (phải dùng 'score')")

    return errors


def merge(inputs: list, output: str, dedup: bool = True):
    """Gộp nhiều tệp JSONL, loại mẫu trùng, xác minh toàn bộ."""
    seen = set()
    total = 0
    duplicates = 0
    invalid = 0
    all_errors = []
    scores = []
    depths = Counter()

    # Tạo thư mục output nếu chưa tồn tại
    directory = os.path.dirname(output)
    if directory:
        os.makedirs(directory, exist_ok=True)

    with open(output, "w", encoding="utf-8") as writer:
        for path in inputs:
            if not os.path.exists(path):
                print(f"⚠️  Bỏ qua tệp không tồn tại: {path}")
                continue

            basename = os.path.basename(path)
            count = 0
            skipped = 0

            with open(path, "r", encoding="utf-8") as reader:
                for index, line in enumerate(reader, start=1):
                    line = line.strip()
                    if not line:
                        continue

                    try:
                        sample = json.loads(line)
                    except json.JSONDecodeError as error:
                        all_errors.append(f"[{basename}:{index}] JSON parse error: {error}")
                        invalid += 1
                        continue

                    # Xác minh mẫu
                    errors = validate(sample, index, basename)
                    if errors:
                        all_errors.extend(errors)
                        invalid += 1
                        continue

                    # Loại mẫu trùng theo FEN
                    fen = sample["fen"]
                    if dedup and fen in seen:
                        duplicates += 1
                        skipped += 1
                        continue
                    seen.add(fen)

                    # Ghi mẫu hợp lệ
                    writer.write(json.dumps(sample, ensure_ascii=False) + "\n")
                    scores.append(sample["score"])
                    depths[sample["depth"]] += 1
                    count += 1
                    total += 1

            print(f"  📄 {basename}: {count:,} mẫu hợp lệ, {skipped:,} trùng lặp")

    # Báo cáo tổng hợp
    print()
    print("=" * 60)
    print("📊 BÁO CÁO GỘP DỮ LIỆU MINING")
    print("=" * 60)
    print(f"  • Tổng mẫu hợp lệ: {total:,}")
    print(f"  • Mẫu trùng lặp (đã loại): {duplicates:,}")
    print(f"  • Mẫu lỗi (đã loại): {invalid:,}")

    if scores:
        print(f"  • Score range: [{min(scores):,}, {max(scores):,}]")
        print(f"  • Score mean: {sum(scores) / len(scores):.1f}")

    if depths:
        print(f"  • Depth distribution: {dict(sorted(depths.items()))}")

    size = os.path.getsize(output)
    if size > 1024 * 1024 * 1024:
        print(f"  • File size: {size / (1024 * 1024 * 1024):.2f} GB")
    else:
        print(f"  • File size: {size / (1024 * 1024):.2f} MB")

    print(f"  • Output: {output}")
    print("=" * 60)

    if all_errors:
        print(f"\n⚠️  {len(all_errors)} lỗi phát hiện:")
        for error in all_errors[:20]:
            print(f"    {error}")
        if len(all_errors) > 20:
            print(f"    ... và {len(all_errors) - 20} lỗi nữa")

    return total, duplicates, invalid


def main():
    parser = argparse.ArgumentParser(
        description="Gộp & Xác minh dữ liệu mining Xiangqi-RIM đa instance"
    )
    parser.add_argument(
        "inputs",
        nargs="+",
        help="Danh sách tệp JSONL đầu vào (hỗ trợ glob: data/gen6_part*.jsonl)"
    )
    parser.add_argument(
        "-o", "--output",
        required=True,
        help="Tệp JSONL đầu ra đã gộp và xác minh"
    )
    parser.add_argument(
        "--no-dedup",
        action="store_true",
        help="Tắt loại mẫu trùng lặp (mặc định: bật)"
    )
    args = parser.parse_args()

    print("============================================================")
    print(" XIANGQI-RIM DATA MERGER & VALIDATOR")
    print("============================================================")
    print(f"  • Tệp đầu vào: {len(args.inputs)} tệp")
    print(f"  • Dedup FEN: {'TẮT' if args.no_dedup else 'BẬT'}")
    print(f"  • Output: {args.output}")
    print()

    total, duplicates, invalid = merge(
        args.inputs,
        args.output,
        dedup=not args.no_dedup
    )

    # Exit code: 0 = thành công, 1 = có lỗi nhưng có dữ liệu, 2 = thất bại
    if total == 0:
        print("\n❌ THẤT BẠI: Không có mẫu hợp lệ nào!")
        sys.exit(2)
    elif invalid > 0:
        print(f"\n⚠️  CẢNH BÁO: {invalid:,} mẫu bị loại do lỗi.")
        sys.exit(1)
    else:
        print(f"\n✅ THÀNH CÔNG: {total:,} mẫu hợp lệ đã gộp.")
        sys.exit(0)


if __name__ == "__main__":
    main()
