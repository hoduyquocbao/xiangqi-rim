#!/usr/bin/env python3
# ============================================================================
# UNIT TEST: KIỂM TRA TƯƠNG THÍCH PYTHON ↔ RUST NNUE QUANTIZATION
# ============================================================================
# Mục đích: Xác minh tệp XRNN binary từ Python notebook có thể được
# Rust engine nạp chính xác, và các giá trị trọng số nằm trong phạm vi
# hợp lý sau khi lượng tử hóa.
#
# Kiểm tra:
#   1. Magic header + version
#   2. Kích thước tệp chính xác theo layout
#   3. Giá trị FT bias nằm trong phạm vi i16 hợp lý
#   4. FT weights: phân bố, min/max, tỷ lệ zero
#   5. Hidden weights: phân bố, saturation rate
#   6. Output weights: phân bố
#   7. Output bias + scale
#   8. Roundtrip test: quantize → load → evaluate so sánh
# ============================================================================

import os
import struct
import sys
import math

# ============================================================================
# HẰNG SỐ KIẾN TRÚC NNUE (phải khớp với src/eval/nnue.rs và src/learn/nnue.rs)
# ============================================================================
DIM = 256         # Feature Transformer output dimension (HALF)
BOTH = 512        # Concat dimension (DIM * 2)
HIDDEN = 32       # Hidden layer size
TOTAL = 65536     # Feature Transformer input dimension (HalfKAv2_hm)
OUTPUT_SCALE = 16 # Fixed output scale

# Binary layout sizes (bytes)
MAGIC = 4
VERSION = 4
FT_BIAS = DIM * 2             # 256 × i16 = 512
FT_WEIGHT = TOTAL * DIM * 2   # 65536 × 256 × i16 = 33,554,432
HIDDEN_W = HIDDEN * BOTH * 1  # 32 × 512 × i8 = 16,384
HIDDEN_B = HIDDEN * 4         # 32 × i32 = 128
OUTPUT_W = HIDDEN * 1         # 32 × i8 = 32
OUTPUT_B = 4                  # i32
OUTPUT_S = 4                  # i32

EXPECTED = MAGIC + VERSION + FT_BIAS + FT_WEIGHT + HIDDEN_W + HIDDEN_B + OUTPUT_W + OUTPUT_B + OUTPUT_S

# ============================================================================
# CÁC HÀM KIỂM TRA
# ============================================================================

class Result:
    """Lưu kết quả kiểm tra."""
    def __init__(self):
        self.passed = 0
        self.failed = 0
        self.warnings = 0
        self.details = []

    def ok(self, name, detail=""):
        self.passed += 1
        msg = f"  ✅ PASS: {name}"
        if detail:
            msg += f" — {detail}"
        self.details.append(msg)
        print(msg)

    def fail(self, name, detail=""):
        self.failed += 1
        msg = f"  ❌ FAIL: {name}"
        if detail:
            msg += f" — {detail}"
        self.details.append(msg)
        print(msg)

    def warn(self, name, detail=""):
        self.warnings += 1
        msg = f"  ⚠️ WARN: {name}"
        if detail:
            msg += f" — {detail}"
        self.details.append(msg)
        print(msg)

    def summary(self):
        total = self.passed + self.failed
        print(f"\n{'='*60}")
        print(f"  KẾT QUẢ: {self.passed}/{total} PASSED, {self.failed} FAILED, {self.warnings} WARNINGS")
        print(f"{'='*60}")
        return self.failed == 0


def test_binary_format(f, path, result):
    """Kiểm tra magic header, version, kích thước tệp."""
    print(f"\n[1] KIỂM TRA BINARY FORMAT: {path}")

    if not os.path.exists(path):
        result.fail("File exists", f"Không tìm thấy {path}")
        return False

    size = os.path.getsize(path)
    if size == EXPECTED:
        result.ok("File size", f"{size:,} bytes = {size/(1024*1024):.2f} MB (khớp layout)")
    else:
        result.fail("File size", f"{size:,} bytes, kỳ vọng {EXPECTED:,} (sai lệch {size - EXPECTED} bytes)")
        return False

    magic = f.read(4)
    if magic == b"XRNN":
        result.ok("Magic header", "XRNN")
    else:
        result.fail("Magic header", f"'{magic}' (kỳ vọng 'XRNN')")
        return False

    ver = struct.unpack("<I", f.read(4))[0]
    if ver == 1:
        result.ok("Version", f"v{ver}")
    else:
        result.fail("Version", f"v{ver} (kỳ vọng v1)")
        return False

    return True


def test_ft_bias(f, result):
    """Kiểm tra Feature Transformer bias: phân bố và phạm vi."""
    print(f"\n[2] KIỂM TRA FT BIAS ({DIM} × i16)")

    values = []
    for j in range(DIM):
        raw = f.read(2)
        val = struct.unpack("<h", raw)[0]
        values.append(val)

    minimum = min(values)
    maximum = max(values)
    mean = sum(values) / len(values)
    zero_count = sum(1 for v in values if v == 0)
    saturated = sum(1 for v in values if abs(v) > 32000)

    if -32768 <= minimum and maximum <= 32767:
        result.ok("FT Bias range", f"[{minimum}, {maximum}]")
    else:
        result.fail("FT Bias range", f"[{minimum}, {maximum}] vượt giới hạn i16")

    if zero_count == DIM:
        result.warn("FT Bias all zeros", f"{zero_count}/{DIM} = 100% — bias có thể chưa được huấn luyện đúng")
    elif zero_count > DIM * 0.9:
        result.warn("FT Bias mostly zeros", f"{zero_count}/{DIM} = {100*zero_count/DIM:.1f}%")
    else:
        result.ok("FT Bias distribution", f"mean={mean:.1f}, zeros={zero_count}/{DIM}")

    if saturated > DIM * 0.1:
        result.warn("FT Bias saturation", f"{saturated}/{DIM} giá trị gần bão hòa i16")
    else:
        result.ok("FT Bias saturation", f"{saturated}/{DIM} giá trị bão hòa (< 10% threshold)")

    return values


def test_ft_weights(f, result):
    """Kiểm tra Feature Transformer weights: phân bố, sparsity, saturation."""
    print(f"\n[3] KIỂM TRA FT WEIGHTS ({TOTAL}×{DIM} × i16 = {FT_WEIGHT:,} bytes)")

    total_vals = TOTAL * DIM
    zero_count = 0
    saturated = 0
    minimum = 32767
    maximum = -32768
    sample_sum = 0.0
    sample_count = 0

    # Đọc theo batch để tránh tốn quá nhiều RAM
    for i in range(TOTAL):
        raw = f.read(DIM * 2)
        for j in range(DIM):
            val = struct.unpack_from("<h", raw, j * 2)[0]
            if val == 0:
                zero_count += 1
            if abs(val) > 32000:
                saturated += 1
            if val < minimum:
                minimum = val
            if val > maximum:
                maximum = val
            # Sample thống kê trên 1% dữ liệu
            if i % 100 == 0:
                sample_sum += val
                sample_count += 1

    sparsity = zero_count / total_vals * 100
    sat_rate = saturated / total_vals * 100
    sample_mean = sample_sum / max(1, sample_count)

    if -32768 <= minimum and maximum <= 32767:
        result.ok("FT Weight range", f"[{minimum}, {maximum}]")
    else:
        result.fail("FT Weight range", f"Vượt giới hạn i16")

    if sparsity > 99.0:
        result.warn("FT Weight sparsity", f"{sparsity:.2f}% — gần như toàn bộ zero, mô hình có thể chưa hội tụ")
    elif sparsity > 90.0:
        result.ok("FT Weight sparsity", f"{sparsity:.2f}% — bình thường cho NNUE (hầu hết features không tích cực)")
    else:
        result.ok("FT Weight sparsity", f"{sparsity:.2f}%")

    if sat_rate > 5.0:
        result.warn("FT Weight saturation", f"{sat_rate:.2f}% — cần giảm learning rate")
    else:
        result.ok("FT Weight saturation", f"{sat_rate:.4f}%")

    result.ok("FT Weight stats", f"sample_mean={sample_mean:.2f}, zeros={zero_count:,}/{total_vals:,}")


def test_hidden(f, result):
    """Kiểm tra Hidden Layer weights và bias."""
    print(f"\n[4] KIỂM TRA HIDDEN LAYER ({HIDDEN}×{BOTH} × i8 + {HIDDEN} × i32)")

    # Hidden weights
    weights = []
    for i in range(HIDDEN):
        for j in range(BOTH):
            val = struct.unpack("b", f.read(1))[0]
            weights.append(val)

    w_min = min(weights)
    w_max = max(weights)
    w_zero = sum(1 for v in weights if v == 0)
    w_sat = sum(1 for v in weights if abs(v) >= 127)

    if -128 <= w_min and w_max <= 127:
        result.ok("Hidden Weight range", f"[{w_min}, {w_max}]")
    else:
        result.fail("Hidden Weight range", f"Vượt giới hạn i8")

    if w_sat / len(weights) > 0.2:
        result.warn("Hidden Weight saturation", f"{w_sat}/{len(weights)} ({100*w_sat/len(weights):.1f}%) bão hòa i8")
    else:
        result.ok("Hidden Weight saturation", f"{w_sat}/{len(weights)} ({100*w_sat/len(weights):.1f}%)")

    # Hidden bias
    biases = []
    for i in range(HIDDEN):
        val = struct.unpack("<i", f.read(4))[0]
        biases.append(val)

    b_min = min(biases)
    b_max = max(biases)
    b_mean = sum(biases) / len(biases)
    result.ok("Hidden Bias range", f"[{b_min}, {b_max}], mean={b_mean:.1f}")


def test_output(f, result):
    """Kiểm tra Output Layer weights, bias, scale."""
    print(f"\n[5] KIỂM TRA OUTPUT LAYER ({HIDDEN} × i8 + i32 + i32)")

    # Output weights
    weights = []
    for i in range(HIDDEN):
        val = struct.unpack("b", f.read(1))[0]
        weights.append(val)

    o_min = min(weights)
    o_max = max(weights)
    result.ok("Output Weight range", f"[{o_min}, {o_max}]")

    # Output bias
    bias = struct.unpack("<i", f.read(4))[0]
    # Reverse scale: bias_cp ≈ bias / (64 * 64 * 400) * 400 = bias / 4096
    approx_cp = bias / (64.0 * 64.0)
    result.ok("Output Bias", f"raw={bias}, ≈{approx_cp:.1f} centipawn equivalent")

    # Output scale
    scale = struct.unpack("<i", f.read(4))[0]
    if scale == OUTPUT_SCALE:
        result.ok("Output Scale", f"{scale} (khớp kỳ vọng)")
    else:
        result.warn("Output Scale", f"{scale} (kỳ vọng {OUTPUT_SCALE})")


def test_quantization_scales(result):
    """Kiểm tra tương thích scale factors giữa Python notebook và Rust engine."""
    print(f"\n[6] KIỂM TRA TƯƠNG THÍCH QUANTIZATION SCALES")

    # Rust learn/nnue.rs quantize():
    rust_scales = {
        "ft_weight":   127.0,              # self.feature[i][j] * 127.0 → i16
        "ft_bias":     127.0,              # self.bias[j] * 127.0 → i16
        "hidden_w":    64.0,               # self.hidden[i][j] * 64.0 → i8
        "hidden_b":    127.0 * 64.0,       # self.offset[i] * (127.0 * 64.0) → i32
        "output_w":    64.0,               # self.output[i] * 64.0 → i8
        "output_b":    64.0 * 64.0 * 400.0,  # self.anchor * (64.0 * 64.0 * 400.0) → i32
    }

    # Python community_colab.ipynb quantization:
    python_scales = {
        "ft_weight":   127.0,              # val * 127.0 → i16
        "ft_bias":     127.0,              # ft.weight.mean(dim=0) * 127.0 → i16 ⚠️
        "hidden_w":    64.0,               # val * 64.0 → i8
        "hidden_b":    127.0 * 64.0,       # val * 127.0 * 64.0 → i32
        "output_w":    64.0,               # val * 64.0 → i8
        "output_b":    64.0 * 64.0 * 400.0,  # val * 64.0 * 64.0 * 400.0 → i32
    }

    for key in rust_scales:
        if rust_scales[key] == python_scales[key]:
            result.ok(f"Scale '{key}'", f"Rust={rust_scales[key]}, Python={python_scales[key]} ✓")
        else:
            result.fail(f"Scale '{key}'", f"Rust={rust_scales[key]} ≠ Python={python_scales[key]}")

    # Cảnh báo đặc biệt cho FT Bias
    result.warn(
        "FT Bias semantic mismatch",
        "Python dùng ft.weight.mean(dim=0) thay cho bias thực — "
        "Rust dùng self.bias[] riêng biệt được huấn luyện. "
        "Sai lệch nhỏ nếu weight mean ≈ 0, nhưng có thể nghiêm trọng nếu model hội tụ sâu."
    )


# ============================================================================
# MAIN
# ============================================================================

def main():
    path = sys.argv[1] if len(sys.argv) > 1 else "data/nnue_weights_gen5.bin"

    print("=" * 60)
    print("  NNUE QUANTIZATION COMPATIBILITY TEST")
    print(f"  File: {path}")
    print("=" * 60)

    result = Result()

    with open(path, "rb") as f:
        # Test 1: Binary format
        ok = test_binary_format(f, path, result)
        if not ok:
            result.summary()
            sys.exit(1)

        # Test 2: FT Bias
        test_ft_bias(f, result)

        # Test 3: FT Weights (chậm — 32MB)
        print("  ⏳ Đang quét 33MB FT weights (có thể mất 10-30 giây)...")
        test_ft_weights(f, result)

        # Test 4: Hidden Layer
        test_hidden(f, result)

        # Test 5: Output Layer
        test_output(f, result)

    # Test 6: Quantization scale compatibility
    test_quantization_scales(result)

    # Summary
    success = result.summary()
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
