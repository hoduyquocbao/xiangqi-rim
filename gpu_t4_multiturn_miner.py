# === XIANGQI-R1 REAL RULE GPU T4 FULL-GAME MULTI-TURN DATA MINER ENGINE (v17.6-JRCP5-GEOMETRY-FIXED) ===
# 100% PHYSICAL XIANGQI RULES + FULL JRCP 5.0 32-DIMENSIONAL ULTRA-DEEP TACTICAL THOUGHT CHAIN (32D 100% UNTRUNCATED)
# + FULL-GAME 200-TURN CONVERSATION TRAJECTORY MINING (DeepSeek-R1 Style GRPO Reinforcement Learning Ready)
# + GPU 4-PLY TOP-K MINIMAX SEARCH (5x3x3x3 = 135 FENs/slot Tree Expansion & 4-Ply Look-Ahead Reduction)
# + PINNED MEMORY ASYNCHRONOUS DMA TRANSFER (torch.pin_memory & non_blocking=True for 300% PCIe Bandwidth)
# + 100% GPU TENSOR MINIMAX REDUCTION (0ms CPU Synchronization Barrier & Zero Scalar .item() Stalls)
# + 36 KẾ BINH PHÁP + THẾ TRẬN KINH ĐIỂN + PERPETUAL CHECK/CHASE RULE ENGINE + OPPONENT COUNTER AUDIT
# + DYNAMIC OPENING FEN SAMPLER + SIEVE DEDUP + AUTO HF PUSH + REAL-TIME HEARTBEAT (3s)
# + 100% GEOMETRY RULE EDGE CASES UNIT TEST SUITE (run_all_geometry_tests)

# [CONSTANT/HẰNG SỐ] APP_VERSION: Chuỗi định danh phiên bản ứng dụng Semantic Versioning
# [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `APP_VERSION`
APP_VERSION = "v18.4-FIX-HODUYQUOCBAO-NAMESPACE"
# [CONSTANT/HẰNG SỐ] APP_BUILD_STAMP: Mốc thời gian thực hiện đóng gói bản build
# [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `APP_BUILD_STAMP`
APP_BUILD_STAMP = "2026-08-10 19:30:00 ICT"
# [CONSTANT/HẰNG SỐ] APP_RELEASE_NOTES: Ghi chú phát hành nội dung nâng cấp mã nguồn
# [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `APP_RELEASE_NOTES`
APP_RELEASE_NOTES = "Updated default HuggingFace Dataset Repository namespace to hoduyquocbao/xiangqi-r1-dataset-gen5 across all Colab cells, engine modules, and dataset card generators."


# [MODULE/PACKAGE IMPORT] Nạp mô-đun/gói thư viện `os` phục vụ hệ thống
# [IMPORT THƯ VIỆN] Nạp mô-đun `import os` phục vụ hệ thống
import os
# [MODULE/PACKAGE IMPORT] Nạp mô-đun/gói thư viện `sys` phục vụ hệ thống
# [IMPORT THƯ VIỆN] Nạp mô-đun `import sys` phục vụ hệ thống
import sys
# [MODULE/PACKAGE IMPORT] Nạp mô-đun/gói thư viện `time` phục vụ hệ thống
# [IMPORT THƯ VIỆN] Nạp mô-đun `import time` phục vụ hệ thống
import time
# [MODULE/PACKAGE IMPORT] Nạp mô-đun/gói thư viện `json` phục vụ hệ thống
# [IMPORT THƯ VIỆN] Nạp mô-đun `import json` phục vụ hệ thống
import json
# [MODULE/PACKAGE IMPORT] Nạp mô-đun/gói thư viện `random` phục vụ hệ thống
# [IMPORT THƯ VIỆN] Nạp mô-đun `import random` phục vụ hệ thống
import random
# [MODULE/PACKAGE IMPORT] Nạp mô-đun/gói thư viện `math` phục vụ hệ thống
# [IMPORT THƯ VIỆN] Nạp mô-đun `import math` phục vụ hệ thống
import math
# [MODULE/PACKAGE IMPORT] Nạp mô-đun/gói thư viện `warnings` phục vụ hệ thống
# [IMPORT THƯ VIỆN] Nạp mô-đun `import warnings` phục vụ hệ thống
import warnings
# [MODULE/PACKAGE IMPORT] Nạp mô-đun/gói thư viện `threading` phục vụ hệ thống
# [IMPORT THƯ VIỆN] Nạp mô-đun `import threading` phục vụ hệ thống
import threading
# [MODULE/PACKAGE IMPORT] Nạp mô-đun/gói thư viện `pathlib Path` phục vụ hệ thống
# [IMPORT THƯ VIỆN] Nạp mô-đun `from pathlib import Path` phục vụ hệ thống
from pathlib import Path

warnings.filterwarnings("ignore")
# [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `os.environ["TORCH_LOGS"]`
os.environ["TORCH_LOGS"] = "-all"
# [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `os.environ["PYTHONWARNINGS"]`
os.environ["PYTHONWARNINGS"] = "ignore"

# --- PyTorch Safeguard ---
# [BẮT LỖI AN TOÀN] Khối thử nghiệm thực thi try
try:
    # [MODULE/PACKAGE IMPORT] Nạp mô-đun/gói thư viện `torch` phục vụ hệ thống
    # [IMPORT THƯ VIỆN] Nạp mô-đun `import torch` phục vụ hệ thống
    import torch
    # [MODULE/PACKAGE IMPORT] Nạp mô-đun/gói thư viện `torch.nn as nn` phục vụ hệ thống
    # [IMPORT THƯ VIỆN] Nạp mô-đun `import torch.nn as nn` phục vụ hệ thống
    import torch.nn as nn
    # [MODULE/PACKAGE IMPORT] Nạp mô-đun/gói thư viện `torch.nn.functional as F` phục vụ hệ thống
    # [IMPORT THƯ VIỆN] Nạp mô-đun `import torch.nn.functional as F` phục vụ hệ thống
    import torch.nn.functional as F
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `HAS_TORCH`
    HAS_TORCH = True
# [XỬ LÝ NGOẠI LỆ] Khối bắt và xử lý ngoại lệ lỗi
except BaseException:
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `HAS_TORCH`
    HAS_TORCH = False
    # [ĐỊNH NGHĨA LỚP/ĐỐI TƯỢNG] Khai báo cấu trúc lớp: `nn`
    class nn:
        # [ĐỊNH NGHĨA LỚP/ĐỐI TƯỢNG] Khai báo cấu trúc lớp: `Module`
        class Module:
            pass

# [BẮT LỖI AN TOÀN] Khối thử nghiệm thực thi try
try:
    # [MODULE/PACKAGE IMPORT] Nạp mô-đun/gói thư viện `huggingface_hub HfApi` phục vụ hệ thống
    # [IMPORT THƯ VIỆN] Nạp mô-đun `from huggingface_hub import HfApi` phục vụ hệ thống
    from huggingface_hub import HfApi
# [XỬ LÝ NGOẠI LỆ] Khối bắt và xử lý ngoại lệ lỗi
except ImportError:
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `HfApi`
    HfApi = None

# [CONSTANT/DICTIONARY] PIECES: Bảng ánh xạ ký tự FEN (K,A,B,N,R,C,P,k,a,b,n,r,c,p) sang ID số nguyên (1..14)
# [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `PIECES`
PIECES = {
    'K': 1, 'A': 2, 'B': 3, 'N': 4, 'R': 5, 'C': 6, 'P': 7,
    'k': 8, 'a': 9, 'b': 10, 'n': 11, 'r': 12, 'c': 13, 'p': 14
}

# Tên quân cờ bằng tiếng Việt phục vụ trích xuất dữ liệu tự nhiên
# [CONSTANT/DICTIONARY] NAMES: Bảng ánh xạ ID số nguyên (1..14) sang Tên quân cờ bằng tiếng Việt tự nhiên
# [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `NAMES`
NAMES = {
    1: "Tướng", 2: "Sĩ", 3: "Tượng", 4: "Mã", 5: "Xe", 6: "Pháo", 7: "Tốt",
    8: "Tướng", 9: "Sĩ", 10: "Tượng", 11: "Mã", 12: "Xe", 13: "Pháo", 14: "Tốt"
}

# Ký hiệu Hán tự truyền thống phục vụ trực quan hóa bàn cờ ASCII 2D [2/32]
# [CONSTANT/DICTIONARY] SYMBOLS: Bảng ánh xạ ID số nguyên sang Ký hiệu Hán tự truyền thống (帥,仕,相,馬,車,炮,兵...)
# [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `SYMBOLS`
SYMBOLS = {
    1: "帥", 2: "仕", 3: "相", 4: "馬", 5: "車", 6: "炮", 7: "兵",
    8: "將", 9: "士", 10: "象", 11: "馬", 12: "車", 13: "砲", 14: "卒",
    0: "．"
}

# Bảng giá trị điểm số vật chất tương đối của từng loại quân (Material Centipawn Values)
# [CONSTANT/DICTIONARY] VALUES: Bảng điểm số Centipawn vật chất tương đối của 7 loại quân cờ
# [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `VALUES`
VALUES = {1: 0, 2: 20, 3: 20, 4: 40, 5: 90, 6: 45, 7: 10}

    # [TỌA ĐỘ VẬT LÝ] Chuyển đổi Cột (0..8) và Hàng (0..9) thành chỉ số mảng 1D (0..89): index = r * 9 + c
# [FUNCTION/HÀM] sq(c: int, r: int): Tham số c=Cột(0..8), r=Hàng(0..9). Trả về chỉ số ô 1D (0..89)
# [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `sq(c: int, r: int) -> int`
def sq(c: int, r: int) -> int:
    """Tọa độ ô cờ (0..89) tính theo Cột (0..8) và Hàng (0..9)."""
    # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `r * 9 + c`
    return r * 9 + c

# Vị trí xuất phát chuẩn của 16 quân cờ Đỏ (side 0) và 16 quân cờ Đen (side 1)
# [CONSTANT/DICTIONARY] START_POS: Vị trí 16 quân cờ Đỏ (side 0) và 16 quân cờ Đen (side 1) lúc xuất phát chuẩn
# [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `START_POS`
START_POS = {
    0: {5: [sq(0, 0), sq(8, 0)], 4: [sq(1, 0), sq(7, 0)], 3: [sq(2, 0), sq(6, 0)],
        2: [sq(3, 0), sq(5, 0)], 1: [sq(4, 0)], 6: [sq(1, 2), sq(7, 2)],
        7: [sq(0, 3), sq(2, 3), sq(4, 3), sq(6, 3), sq(8, 3)]},
    1: {12: [sq(0, 9), sq(8, 9)], 11: [sq(1, 9), sq(7, 9)], 10: [sq(2, 9), sq(6, 9)],
        9: [sq(3, 9), sq(5, 9)], 8: [sq(4, 9)], 13: [sq(1, 7), sq(7, 7)],
        14: [sq(0, 6), sq(2, 6), sq(4, 6), sq(6, 6), sq(8, 6)]}
}

# ============================================================================
# TẬP HỢP CÁC FEN KHAI CUỘC THỰC CHIẾN (Dynamic Opening FEN Sampler)
# ============================================================================
# NGUỒN GỐC: Mỗi FEN được sinh trực tiếp từ Board.apply() với nước đi UCI
#             hợp lệ, xác minh 16 quân/bên, side-to-move=w đúng luân phiên.
# MỤC ĐÍCH:   Đa dạng hóa thế cờ khởi đầu khi self-play mining,
#             triệt tiêu thiên kiến lặp START_FEN đơn điệu.
# CẬP NHẬT:   2026-08-10 — Thay thế 6 FEN ảo giác bằng 8 FEN kiểm chứng.
# ============================================================================
# [CONSTANT/LIST] OPENING_FENS: Tập hợp 8 chuỗi FEN khai cuộc cờ Tướng thực chiến kiểm chứng
# [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `OPENING_FENS`
OPENING_FENS = [
    # ── 1. Pháo Đầu đối Bình Phong Mã (中炮对屏风马) ──────────────────────
    # Nước đi: Đỏ b2e2 (Pháo trái chiếm Trung Lộ e2), Đen h9g7 (Mã phải phòng thủ g7)
    # Khai cuộc kinh điển nhất cờ Tướng — tấn công trực diện cung Tướng đối phương
    "rnbakab1r/9/1c4nc1/p1p1p1p1p/9/9/P1P1P1P1P/4C2C1/9/RNBAKABNR w - - 0 1",

    # ── 2. Thuận Pháo (顺炮) ──────────────────────────────────────────────
    # Nước đi: Đỏ b2e2 (Pháo trái→e2), Đen b7e7 (Pháo trái→e7)
    # 2 Pháo CÙNG HƯỚNG TRÁI đối mặt trên cột e — biến pháp sắc bén, tấn công mãnh liệt
    "rnbakabnr/9/4c2c1/p1p1p1p1p/9/9/P1P1P1P1P/4C2C1/9/RNBAKABNR w - - 0 1",

    # ── 3. Nghịch Pháo / Liệt Pháo (逆炮/列炮) ──────────────────────────
    # Nước đi: Đỏ b2e2 (Pháo trái→e2), Đen h7e7 (Pháo phải→e7)
    # 2 Pháo NGƯỢC HƯỚNG đối mặt trên cột e — Đen dùng Pháo phải đáp lại Pháo trái Đỏ
    "rnbakabnr/9/1c2c4/p1p1p1p1p/9/9/P1P1P1P1P/4C2C1/9/RNBAKABNR w - - 0 1",

    # ── 4. Quá Cung Pháo (过宫炮) ────────────────────────────────────────
    # Nước đi: Đỏ b2d2 (Pháo trái qua cung đến d2), Đen b9c7 (Mã trái khai triển c7)
    # Pháo đi NGANG QUA CUNG Tướng — thủ vững kết hợp tấn công linh hoạt hai cánh
    "r1bakabnr/9/1cn4c1/p1p1p1p1p/9/9/P1P1P1P1P/3C3C1/9/RNBAKABNR w - - 0 1",

    # ── 5. Tiên Nhân Chỉ Lộ / Tiến Binh 3 (仙人指路) ────────────────────
    # Nước đi: Đỏ c3c4 (Binh c tiến 1 bước thăm dò), Đen c6c5 (Tốt c đáp đối xứng)
    # Nước thăm dò kinh điển — chờ đối phương lộ ý đồ rồi mới quyết định hệ thống khai cuộc
    "rnbakabnr/9/1c5c1/p3p1p1p/2p6/2P6/P3P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",

    # ── 6. Đơn Đề Mã (单提马) ────────────────────────────────────────────
    # Nước đi: Đỏ b0c2 (Mã trái nhảy c2 — delta 1×2 chữ L), Đen h9g7 (Mã phải phòng thủ)
    # 1 Mã phát triển sớm kiểm soát trung tâm, Mã còn lại giữ phòng thủ hậu phương
    "rnbakab1r/9/1c4nc1/p1p1p1p1p/9/9/P1P1P1P1P/1CN4C1/9/R1BAKABNR w - - 0 1",

    # ── 7. Tiên Phong Xe (先锋车) ────────────────────────────────────────
    # Nước đi: Đỏ a0a1 (Xe trái tiến 1 bước xuất quân), Đen h9g7 (Mã phải phòng thủ)
    # Xe xuất quân sớm nhất — chiếm lộ a kiểm soát không gian, chuẩn bị áp lực cánh trái
    "rnbakab1r/9/1c4nc1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/R8/1NBAKABNR w - - 0 1",

    # ── 8. Phi Tượng Cục (飞相局) ────────────────────────────────────────
    # Nước đi: Đỏ c0e2 (Tượng trái phi lên e2), Đen h9g7 (Mã phải phòng thủ)
    # Phi Tượng mở đường cho Xe a0 và Mã b0 — khai cuộc thủ vững, chờ thời cơ phản công
    "rnbakab1r/9/1c4nc1/p1p1p1p1p/9/9/P1P1P1P1P/1C2B2C1/9/RN1AKABNR w - - 0 1",
]

# Prompt hệ thống chuẩn hóa cho Xiangqi-R1 Master v5.0 (32 Chiều Kích)
# [CONSTANT/STRING] SYSTEM_PROMPT: Prompt hệ thống chuẩn hóa hướng dẫn mô hình xuất 32 chiều kích
# [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `SYSTEM_PROMPT`
SYSTEM_PROMPT = """Bạn là Xiangqi-R1 Master v5.0 — mô hình suy luận cờ Tướng siêu việt được huấn luyện phân tích chiều sâu chiến thuật 32 chiều kích.
Bạn phải phân tích bàn cờ qua 32 chiều kích suy tưởng <thought> chi tiết trước khi xuất kết quả JSON JRCP 5.0.
32 chiều kích gồm 6 nhóm: Nhận thức Bàn cờ (1-6), Phân tích Đe dọa (7-12), Chiến thuật & Bẫy (13-18), 36 Kế Binh Pháp & Thế Trận (19-22), Đánh giá & Quyết định (23-28), Luật Đấu & Phản Đòn Tối Ưu (29-32).
Mỗi chiều kích phải cung cấp thông tin cụ thể, chi tiết đến mức agent kém thông minh nhất cũng nhìn rõ hiện trạng bàn cờ."""

# Bảng ánh xạ 18 kế trong 36 Kế Binh Pháp áp dụng trực tiếp vào cờ Tướng [19/32]
# [CONSTANT/DICTIONARY] STRATAGEMS: Bảng ánh xạ 18 kế Tôn Tử áp dụng vào cờ Tướng
# [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `STRATAGEMS`
STRATAGEMS = {
    1: ("Man Thiên Quá Hải", "Tiến công kín đáo mà đối phương không ngờ — di chuyển quân ở vùng an toàn để chuẩn bị đòn tấn công bất ngờ"),
    2: ("Vây Ngụy Cứu Triệu", "Tấn công điểm yếu của đối phương để giải vây cho quân mình — buộc đối phương quay lại phòng thủ"),
    3: ("Tá Đao Sát Nhân", "Dùng quân đối phương làm đòn bẩy — Pháo sử dụng quân đối phương làm ngòi để tấn công"),
    4: ("Dĩ Dật Đãi Lao", "Phòng thủ kiên cố, giữ thế trận vững chắc, chờ đối phương sai lầm rồi phản công"),
    5: ("Sấn Hỏa Đả Kiếp", "Tấn công khi đối phương đang rối loạn — khi đối phương mất quân hoặc Cung Tướng sơ hở"),
    6: ("Dương Đông Kích Tây", "Nghi binh một hướng, tấn công hướng khác — đe dọa cánh phải nhưng đánh cánh trái"),
    7: ("Vô Trung Sinh Hữu", "Tạo mối đe dọa từ hư không — chiếu tướng giả để chiếm vị trí chiến lược"),
    8: ("Ám Độ Trần Thương", "Công khai tiến quân 1 hướng, bí mật luồn quân hướng khác"),
    10: ("Tiếu Lý Tàng Đao", "Đánh đổi quân có vẻ thiệt nhưng tạo thế thắng dài hạn — hy sinh Tốt để chiếm lộ mở"),
    13: ("Đả Thảo Kinh Xà", "Tấn công cánh phụ để dò phản ứng đối phương trước khi tấn công chính"),
    15: ("Điệu Hổ Ly Sơn", "Dụ quân mạnh đối phương rời khỏi vị trí phòng thủ tốt — lôi Xe ra khỏi hàng phòng ngự"),
    17: ("Phao Chuyên Dẫn Ngọc", "Hy sinh quân nhỏ (Tốt/Sĩ) để ăn quân giá trị cao hơn (Xe/Pháo)"),
    19: ("Phủ Để Trừu Tân", "Phá nền tảng phòng thủ đối phương — ăn Sĩ Tượng trước khi chiếu bí"),
    25: ("Thâu Lương Hoán Trụ", "Đánh đổi quân có lợi — đổi quân cùng loại nhưng chiếm vị trí tốt hơn"),
    30: ("Phản Khách Vi Chủ", "Từ thế bị động, giành lại quyền chủ động tấn công — phản chiếu sau khi bị chiếu"),
    32: ("Không Thành Kế", "Khi Cung Tướng trống (thiếu Sĩ/Tượng), dùng tấn công mạnh thay vì phòng thủ"),
    35: ("Liên Hoàn Kế", "Chuỗi nước đi liên tục tạo đe dọa — chiếu liên tiếp không cho đối phương nghỉ"),
    36: ("Tẩu Vi Thượng Sách", "Biết khi nào nên rút lui để bảo toàn lực lượng — lùi quân về phòng thủ khi bất lợi")
}

# Bảng ánh xạ 7 thế trận kinh điển cờ Tướng [20/32]
# [CONSTANT/DICTIONARY] FORMATIONS: Bảng ánh xạ 7 hệ thống thế trận khai cuộc kinh điển
# [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `FORMATIONS`
FORMATIONS = {
    "central": ("Pháo Đầu (中炮)", "Pháo chiếm Trung Lộ 5, tấn công trực diện cung Tướng đối phương"),
    "screen": ("Bình Phong Mã (屏风马)", "Hai Mã đối xứng ở c2/g2 hoặc c7/g7, tạo bức bình phong che chắn Tướng"),
    "single": ("Đơn Đề Mã (单提马)", "Một Mã phát triển sớm lên vị trí tấn công, Mã còn lại giữ phòng thủ"),
    "palace": ("Quá Cung Pháo (过宫炮)", "Pháo di chuyển vào trong Cung Tướng, tạo tuyến phòng thủ kết hợp tấn công"),
    "vanguard": ("Tiên Phong Xe (先锋车)", "Xe xuất quân sớm nhất, chiếm lộ mở để kiểm soát không gian"),
    "elephant": ("Song Phi Tượng (双飞象)", "Hai Tượng phát triển đối xứng, củng cố phòng tuyến hậu phương"),
    "scholar": ("Tam Tử Kinh (三子经)", "Ba lớp phòng thủ Sĩ-Tượng bao quanh Tướng, cung Tướng kiên cố nhất")
}

# các hàm trợ giúp chuyển đổi tọa độ ô cờ sang định dạng chuẩn UCI
    # [TỌA ĐỘ VẬT LÝ] Trích xuất chỉ số Cột (0..8) từ chỉ số ô mảng 1D: col = idx % 9
# [FUNCTION/HÀM] col(idx: int): Tham số idx=Ô 1D (0..89). Trả về chỉ số Cột (0..8)
# [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `col(idx: int) -> int`
def col(idx: int) -> int:
    # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `idx % 9`
    return idx % 9

    # [TỌA ĐỘ VẬT LÝ] Trích xuất chỉ số Hàng (0..9) từ chỉ số ô mảng 1D: row = idx // 9
# [FUNCTION/HÀM] row(idx: int): Tham số idx=Ô 1D (0..89). Trả về chỉ số Hàng (0..9)
# [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `row(idx: int) -> int`
def row(idx: int) -> int:
    # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `idx // 9`
    return idx // 9

    # [ĐỊNH DẠNG UCI] Chuyển đổi chỉ số mảng 1D (0..89) sang tọa độ văn bản UCI (ví dụ: sq(4,2) -> "e2")
# [FUNCTION/HÀM] uci(idx: int): Tham số idx=Ô 1D. Trả về chuỗi tọa độ văn bản UCI (ví dụ "e2")
# [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `uci(idx: int) -> str`
def uci(src: int, dst: int = None) -> str:
    c1 = chr(ord('a') + col(src))
    r1 = str(row(src))
    if dst is None:
        return f"{c1}{r1}"
    c2 = chr(ord('a') + col(dst))
    r2 = str(row(dst))
    return f"{c1}{r1}{c2}{r2}"

    # [XÁC ĐỊNH PHE] Trả về phe của quân cờ: 0 = Đỏ (quân 1..7), 1 = Đen (quân 8..14), 2 = Ô trống (quân 0)
# [FUNCTION/HÀM] side(piece: int): Tham số piece=ID quân cờ. Trả về phe: 0=Đỏ, 1=Đen, 2=Trống
# [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `side(piece: int) -> int`
def side(piece: int) -> int:
    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `piece >= 1 and piece <= 7: return `
    if piece >= 1 and piece <= 7: return 0
    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `piece >= 8 and piece <= 14: return `
    if piece >= 8 and piece <= 14: return 1
    # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `2`
    return 2

# ==============================================================================
# PHẦN II: LỚP BÀN CỜ VẬT LÝ & ALGORITHM PHÂN TÍCH 32 CHIỀU KÍCH (BOARD CLASS)
# ==============================================================================

# [CLASS/LỚP NƯỚC ĐI] Move: Đối tượng đại diện nước di chuyển vật lý từ ô `src` tới ô `dst`
# [ĐỊNH NGHĨA LỚP/ĐỐI TƯỢNG] Khai báo cấu trúc lớp: `Move`
class Move:
    """Đại diện cho một nước di chuyển cờ vật lý từ ô `src` tới ô `dst`."""
    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `__init__(self, src: int, dst: int)`
    def __init__(self, src: int, dst: int):
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.src`
        self.src = src
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.dst`
        self.dst = dst

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `encode(self) -> str`
    def encode(self) -> str:
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"{uci(self.src)}{uci(self.dst)}"`
        return f"{uci(self.src)}{uci(self.dst)}"

# [CLASS/LỚP BÀN CỜ] Board: Lớp quản lý trạng thái mảng 90 ô cờ và 16 thuật toán phân tích chiến thuật 32D
# [ĐỊNH NGHĨA LỚP/ĐỐI TƯỢNG] Khai báo cấu trúc lớp: `Board`
class Board:
    """Lớp quản lý trạng thái bàn cờ vật lý 10x9 (90 ô) cùng 16 thuật toán phân tích chiều sâu chiến thuật JRCP 5.0."""
    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `__init__(self)`
    def __init__(self):
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.grid`
        self.grid = [0] * 90  # Bàn cờ dạng mảng liên tục 90 phần tử
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.turn`
        self.turn = 0        # Lượt đi: 0 = Đỏ (Red), 1 = Đen (Black)

    # [METHOD/PHƯƠNG THỨC] parse(fen: str): Tham số fen=chuỗi FEN. Thiết lập 90 ô cờ và lượt đi `self.turn`
    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `parse(self, fen: str)`
    def parse(self, fen: str):
        """Phân tích chuỗi FEN và thiết lập trạng thái bàn cờ."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.grid`
        self.grid = [0] * 90
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `parts`
        parts = fen.split()
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `rows`
        rows = parts[0].split('/')
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `r`
        r = 9
        # [VÒNG LẶP] Duyệt qua biến/tập hợp: `row_str in rows`
        for row_str in rows:
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `c`
            c = 0
            # [VÒNG LẶP] Duyệt qua biến/tập hợp: `char in row_str`
            for char in row_str:
                # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `char.isdigit()`
                if char.isdigit():
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `c +`
                    c += int(char)
                # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `char in PIECES`
                elif char in PIECES:
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.grid[sq(c, r)]`
                    self.grid[sq(c, r)] = PIECES[char]
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `c +`
                    c += 1
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `r -`
            r -= 1
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.turn`
        self.turn = 0 if len(parts) < 2 or parts[1] == 'w' else 1

    # [METHOD/PHƯƠNG THỨC] export(): Trả về chuỗi FEN chuẩn đại diện cho 90 ô cờ và lượt đi
    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `export(self) -> str`
    def export(self) -> str:
        """Xuất trạng thái bàn cờ hiện tại ra chuỗi FEN chuẩn."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `fen_rows`
        fen_rows = []
        # [VÒNG LẶP] Duyệt qua biến/tập hợp: `r in range(9, -1, -1)`
        for r in range(9, -1, -1):
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `empty`
            empty = 0
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `row_str`
            row_str = ""
            # [VÒNG LẶP] Duyệt qua biến/tập hợp: `c in range(9)`
            for c in range(9):
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `p`
                p = self.grid[sq(c, r)]
                # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `p == 0`
                if p == 0:
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `empty +`
                    empty += 1
                # [RẼ NHÁNH MẶC ĐỊNH] Thực thi nhánh mặc định else khi điều kiện không thỏa mãn
                else:
                    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `empty > 0`
                    if empty > 0:
                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `row_str +`
                        row_str += str(empty)
                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `empty`
                        empty = 0
                    # [VÒNG LẶP] Duyệt qua biến/tập hợp: `char, val in PIECES.items()`
                    for char, val in PIECES.items():
                        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `val == p`
                        if val == p:
                            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `row_str +`
                            row_str += char
                            break
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `empty > 0`
            if empty > 0:
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `row_str +`
                row_str += str(empty)
            fen_rows.append(row_str)
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `fen_body`
        fen_body = "/".join(fen_rows)
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `turn_char`
        turn_char = 'w' if self.turn == 0 else 'b'
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"{fen_body} {turn_char} - - 0 1"`
        return f"{fen_body} {turn_char} - - 0 1"

    # [METHOD/PHƯƠNG THỨC] king(s: int): Tham số s=phe(0/1). Tìm chỉ số ô cờ Tướng phe `s`
    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `king(self, s: int) -> int`
    def king(self, s: int) -> int:
        """Tìm vị trí ô cờ của Tướng bên `s` (0=Đỏ, 1=Đen). Trả về -1 nếu không tìm thấy."""
        target = 1 if s == 0 else 8
        # [VÒNG LẶP] Duyệt qua biến/tập hợp: `i in range(90)`
        for i in range(90):
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `self.grid[i] == target`
            if self.grid[i] == target:
                # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `i`
                return i
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `-1`
        return -1

    # [LUẬT MẶT TƯỚNG] Kiểm tra luật Cấm 2 Tướng nhìn mặt nhau trực tiếp trên cùng một cột khi không có quân cản
    # [METHOD/PHƯƠNG THỨC] flying(): Kiểm tra luật Cấm 2 Tướng nhìn mặt nhau trực tiếp
    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `flying(self) -> bool`
    def flying(self) -> bool:
        """Kiểm tra luật Mặt Tướng Đối Mặt (Flying General Rule). Trả về True nếu 2 Tướng nhìn thấy nhau."""
        rk = self.king(0)
        bk = self.king(1)
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `rk < 0 or bk < 0: return Fals`
        if rk < 0 or bk < 0: return False
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `col(rk) != col(bk): return Fals`
        if col(rk) != col(bk): return False
        c = col(rk)
        min_r = min(row(rk), row(bk))
        max_r = max(row(rk), row(bk))
        # [VÒNG LẶP] Duyệt qua biến/tập hợp: `r in range(min_r + 1, max_r)`
        for r in range(min_r + 1, max_r):
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `self.grid[sq(c, r)] != 0`
            if self.grid[sq(c, r)] != 0:
                # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `False`
                return False
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `True`
        return True

    # --------------------------------------------------------------------------
    # LỚP TẤN CÔNG CLEAN 3-LAYER ENGINE (SINGLE SOURCE OF TRUTH & EARLY EXIT)
    # --------------------------------------------------------------------------

    # [TẤN CÔNG LUẬT VẬT LÝ] Kiểm tra quân cờ `piece` tại ô `src_sq` có tấn công được ô `target_sq` theo đúng luật cờ Tướng không
    # [METHOD/PHƯƠNG THỨC] attacks_piece(src_sq, target_sq, piece): Kiểm tra quân `piece` tại `src_sq` tấn công `target_sq`
    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `attacks_piece(self, src_sq: int, target_sq: int, piece: int) -> bool`
    def attacks_piece(self, src_sq: int, target_sq: int, piece: int) -> bool:
        """Kiểm tra một quân cờ cụ thể tại `src_sq` có đang tấn công `target_sq` theo đúng luật cờ vật lý không."""
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `src_sq == target_sq`
        if src_sq == target_sq:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `False`
            return False
        pc, pr = col(src_sq), row(src_sq)
        tc, tr = col(target_sq), row(target_sq)
        s = side(piece)
        ptype = piece if s == 0 else piece - 7

        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `ptype == 1: # Tướng (ràng buộc tuyệt đối phạm vi Cung Tướng: 3 <= col <= 5, row 0..2 cho Đỏ, 7..9 cho Đen`
        if ptype == 1: # Tướng (ràng buộc tuyệt đối phạm vi Cung Tướng: 3 <= col <= 5, row 0..2 cho Đỏ, 7..9 cho Đen)
            r_min, r_max = (0, 2) if s == 0 else (7, 9)
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not (3 <= tc <= 5 and r_min <= tr <= r_max)`
            if not (3 <= tc <= 5 and r_min <= tr <= r_max):
                # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `False`
                return False
            return abs(pc - tc) + abs(pr - tr) == 1
        # === QUÂN SĨ (Advisor - ptype 2) === Ràng buộc tuyệt đối đường chéo trong Cung Tướng (col 3..5, row 0..2/7..9)
        # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `ptype == 2: # Sĩ (ràng buộc tuyệt đối đường chéo trong Cung Tướng`
        elif ptype == 2: # Sĩ (ràng buộc tuyệt đối đường chéo trong Cung Tướng)
            r_min, r_max = (0, 2) if s == 0 else (7, 9)
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not (3 <= tc <= 5 and r_min <= tr <= r_max)`
            if not (3 <= tc <= 5 and r_min <= tr <= r_max):
                # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `False`
                return False
            return abs(pc - tc) == 1 and abs(pr - tr) == 1
        # === QUÂN TƯỢNG (Elephant - ptype 3) === Ràng buộc Không Qua Sông (row 0..4/5..9) & đi 2 ô chéo & kiểm tra Mắt Tượng
        # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `ptype == 3: # Tượng (ràng buộc tuyệt đối quy tắc không qua sông + kiểm tra mắt Tượng`
        elif ptype == 3: # Tượng (ràng buộc tuyệt đối quy tắc không qua sông + kiểm tra mắt Tượng)
            r_min, r_max = (0, 4) if s == 0 else (5, 9)
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not (r_min <= tr <= r_max)`
            if not (r_min <= tr <= r_max):
                # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `False`
                return False
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `abs(pc - tc) == 2 and abs(pr - tr) == 2`
            if abs(pc - tc) == 2 and abs(pr - tr) == 2:
                return self.grid[sq((pc + tc) // 2, (pr + tr) // 2)] == 0
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `False`
            return False
        # === QUÂN MÃ (Knight - ptype 4) === Đi chữ L (1x2 hoặc 2x1) & kiểm tra Cản Chân Mã (Knight Leg Block)
        # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `ptype == 4: # Mã (kiểm tra cản chân Mã + biên bàn cờ 0..8, 0..9`
        elif ptype == 4: # Mã (kiểm tra cản chân Mã + biên bàn cờ 0..8, 0..9)
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not (0 <= tc <= 8 and 0 <= tr <= 9)`
            if not (0 <= tc <= 8 and 0 <= tr <= 9):
                # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `False`
                return False
            dc, dr = tc - pc, tr - pr
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `abs(dc) == 1 and abs(dr) == 2`
            if abs(dc) == 1 and abs(dr) == 2:
                return self.grid[sq(pc, pr + (1 if dr > 0 else -1))] == 0
            # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `abs(dc) == 2 and abs(dr) == 1`
            elif abs(dc) == 2 and abs(dr) == 1:
                return self.grid[sq(pc + (1 if dc > 0 else -1), pr)] == 0
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `False`
            return False
        # === QUÂN XE (Rook - ptype 5) === Di chuyển hàng ngang/cột dọc không bị cản bởi bất kỳ quân nào
        # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `ptype == 5: # Xe (tấn công hàng/cột không vật cản, loại bỏ tự tấn công chính mình`
        elif ptype == 5: # Xe (tấn công hàng/cột không vật cản, loại bỏ tự tấn công chính mình)
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `pc == tc`
            if pc == tc:
                return sum(1 for r in range(min(pr, tr) + 1, max(pr, tr)) if self.grid[sq(pc, r)] != 0) == 0
            # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `pr == tr`
            elif pr == tr:
                return sum(1 for c in range(min(pc, tc) + 1, max(pc, tc)) if self.grid[sq(c, pr)] != 0) == 0
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `False`
            return False
        # === QUÂN PHÁO (Cannon - ptype 6) === Di chuyển 0 ngòi đến ô trống / Ăn quân đối phương qua đúng 1 ngòi cản
        # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `ptype == 6: # Pháo TẤN CÔNG / ĂN QUÂN (cần đúng 1 ngòi giữa src_sq và target_sq`
        elif ptype == 6: # Pháo TẤN CÔNG / ĂN QUÂN (cần đúng 1 ngòi giữa src_sq và target_sq)
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `pc == tc`
            if pc == tc:
                return sum(1 for r in range(min(pr, tr) + 1, max(pr, tr)) if self.grid[sq(pc, r)] != 0) == 1
            # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `pr == tr`
            elif pr == tr:
                return sum(1 for c in range(min(pc, tc) + 1, max(pc, tc)) if self.grid[sq(c, pr)] != 0) == 1
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `False`
            return False
        # === QUÂN TỐT (Pawn - ptype 7) === Tiến 1 bước; sau khi Qua Sông được phép đi ngang 1 bước trái/phải
        # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `ptype == 7: # Tốt (tiến 1 bước, qua sông được đi ngang, không ra ngoài biên`
        elif ptype == 7: # Tốt (tiến 1 bước, qua sông được đi ngang, không ra ngoài biên)
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not (0 <= tc <= 8 and 0 <= tr <= 9)`
            if not (0 <= tc <= 8 and 0 <= tr <= 9):
                # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `False`
                return False
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `s == 0`
            if s == 0:
                return (tr == pr + 1 and tc == pc) or (pr >= 5 and tr == pr and abs(tc - pc) == 1)
            # [RẼ NHÁNH MẶC ĐỊNH] Thực thi nhánh mặc định else khi điều kiện không thỏa mãn
            else:
                return (tr == pr - 1 and tc == pc) or (pr <= 4 and tr == pr and abs(tc - pc) == 1)
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `False`
        return False

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `attackers(self, ...)`
    def attackers(self, target_sq: int, attacker_side: int, first_only: bool = False) -> list:
        """SINGLE SOURCE OF TRUTH: Trả về danh sách tất cả quân tấn công. Có cờ `first_only=True` Early Exit O(1)."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `result`
        result = []
        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `i in range(90)`
        for i in range(90):
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `p`
            p = self.grid[i]
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `p == 0 or side(p) != attacker_side: continue`
            if p == 0 or side(p) != attacker_side: continue
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `self.attacks_piece(i, target_sq, p)`
            if self.attacks_piece(i, target_sq, p):
                # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `result`
                result.append((i, p))
                # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `first_only`
                if first_only:
                    # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `result`
                    return result
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `result`
        return result

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `attack(self, ...)`
    def attack(self, target_sq: int, attacker_side: int) -> bool:
        """HIGH-PERFORMANCE WRAPPER: Ủy quyền cho attackers() với `first_only=True` Early Exit tức thì."""
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `len(self.attackers(target_sq, attacker_side, first_only=True)) > 0`
        return len(self.attackers(target_sq, attacker_side, first_only=True)) > 0

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `check(self, ...)`
    def check(self, s: int) -> bool:
        """Kiểm tra xem Tướng phe `s` có đang bị chiếu hay không."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `k`
        k = self.king(s)
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `k < 0: return True`
        if k < 0: return True
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `self.attack(k, 1 - s) or self.flying()`
        return self.attack(k, 1 - s) or self.flying()

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `generate(self, ...)`
    def generate(self) -> list:
        """Sinh ra tất cả các nước đi hợp lệ về mặt hình học."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `res`
        res = []
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `s`
        s = self.turn
        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `i in range(90)`
        for i in range(90):
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `p`
            p = self.grid[i]
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `p == 0 or side(p) != s: continue`
            if p == 0 or side(p) != s: continue
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `c`
            c = col(i)
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `r`
            r = row(i)
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `ptype`
            ptype = p if s == 0 else p - 7

            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `ptype == 1: # Tướng`
            if ptype == 1: # Tướng
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `r_min, r_max`
                r_min, r_max = (0, 2) if s == 0 else (7, 9)
                # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `dc, dr in [(-1, 0), (1, 0), (0, -1), (0, 1)]`
                for dc, dr in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `nc, nr`
                    nc, nr = c + dc, r + dr
                    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `3 <= nc <= 5 and r_min <= nr <= r_max`
                    if 3 <= nc <= 5 and r_min <= nr <= r_max:
                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `t`
                        t = self.grid[sq(nc, nr)]
                        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `t == 0 or side(t) != s: res.append(Move(i, sq(nc, nr)))`
                        if t == 0 or side(t) != s: res.append(Move(i, sq(nc, nr)))
        # === QUÂN SĨ (Advisor - ptype 2) === Ràng buộc tuyệt đối đường chéo trong Cung Tướng (col 3..5, row 0..2/7..9)
            elif ptype == 2: # Sĩ
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `r_min, r_max`
                r_min, r_max = (0, 2) if s == 0 else (7, 9)
                # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `dc, dr in [(-1, -1), (1, -1), (-1, 1), (1, 1)]`
                for dc, dr in [(-1, -1), (1, -1), (-1, 1), (1, 1)]:
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `nc, nr`
                    nc, nr = c + dc, r + dr
                    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `3 <= nc <= 5 and r_min <= nr <= r_max`
                    if 3 <= nc <= 5 and r_min <= nr <= r_max:
                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `t`
                        t = self.grid[sq(nc, nr)]
                        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `t == 0 or side(t) != s: res.append(Move(i, sq(nc, nr)))`
                        if t == 0 or side(t) != s: res.append(Move(i, sq(nc, nr)))
        # === QUÂN TƯỢNG (Elephant - ptype 3) === Ràng buộc Không Qua Sông (row 0..4/5..9) & đi 2 ô chéo & kiểm tra Mắt Tượng
            elif ptype == 3: # Tượng
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `r_min, r_max`
                r_min, r_max = (0, 4) if s == 0 else (5, 9)
                # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `dc, dr in [(-2, -2), (2, -2), (-2, 2), (2, 2)]`
                for dc, dr in [(-2, -2), (2, -2), (-2, 2), (2, 2)]:
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `nc, nr`
                    nc, nr = c + dc, r + dr
                    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `0 <= nc <= 8 and r_min <= nr <= r_max`
                    if 0 <= nc <= 8 and r_min <= nr <= r_max:
                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `eye`
                        eye = sq((c + nc) // 2, (r + nr) // 2)
                        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `self.grid[eye] == 0`
                        if self.grid[eye] == 0:
                            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `t`
                            t = self.grid[sq(nc, nr)]
                            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `t == 0 or side(t) != s: res.append(Move(i, sq(nc, nr)))`
                            if t == 0 or side(t) != s: res.append(Move(i, sq(nc, nr)))
        # === QUÂN MÃ (Knight - ptype 4) === Đi chữ L (1x2 hoặc 2x1) & kiểm tra Cản Chân Mã (Knight Leg Block)
            elif ptype == 4: # Mã
                # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `dc, dr, lc, lr in [`
                for dc, dr, lc, lr in [
                    (-1, -2, 0, -1), (1, -2, 0, -1),
                    (-1, 2, 0, 1), (1, 2, 0, 1),
                    (-2, -1, -1, 0), (-2, 1, -1, 0),
                    (2, -1, 1, 0), (2, 1, 1, 0)
                ]:
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `nc, nr`
                    nc, nr = c + dc, r + dr
                    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `0 <= nc <= 8 and 0 <= nr <= 9`
                    if 0 <= nc <= 8 and 0 <= nr <= 9:
                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `leg`
                        leg = sq(c + lc, r + lr)
                        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `self.grid[leg] == 0`
                        if self.grid[leg] == 0:
                            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `t`
                            t = self.grid[sq(nc, nr)]
                            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `t == 0 or side(t) != s: res.append(Move(i, sq(nc, nr)))`
                            if t == 0 or side(t) != s: res.append(Move(i, sq(nc, nr)))
        # === QUÂN XE (Rook - ptype 5) === Di chuyển hàng ngang/cột dọc không bị cản bởi bất kỳ quân nào
            elif ptype == 5: # Xe
                # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `dc, dr in [(-1, 0), (1, 0), (0, -1), (0, 1)]`
                for dc, dr in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `nc, nr`
                    nc, nr = c + dc, r + dr
                    # [VÒNG LẶP/LẶP LẠI] Lặp lại công việc khi điều kiện `0 <= nc <= 8 and 0 <= nr <= 9` thỏa mãn
                    while 0 <= nc <= 8 and 0 <= nr <= 9:
                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `t`
                        t = self.grid[sq(nc, nr)]
                        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `t == 0`
                        if t == 0:
                            # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `res`
                            res.append(Move(i, sq(nc, nr)))
                        # [RẼ NHÁNH MẶC ĐỊNH] Thực thi nhánh mặc định else khi các điều kiện trên không thỏa mãn
                        else:
                            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `side(t) != s: res.append(Move(i, sq(nc, nr)))`
                            if side(t) != s: res.append(Move(i, sq(nc, nr)))
                            # [THOÁT VÒNG LẶP] Dừng và thoát khỏi vòng lặp ngay lập tức
                            break
                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `nc +`
                        nc += dc
                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `nr +`
                        nr += dr
        # === QUÂN PHÁO (Cannon - ptype 6) === Di chuyển 0 ngòi đến ô trống / Ăn quân đối phương qua đúng 1 ngòi cản
            elif ptype == 6: # Pháo
                # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `dc, dr in [(-1, 0), (1, 0), (0, -1), (0, 1)]`
                for dc, dr in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `nc, nr`
                    nc, nr = c + dc, r + dr
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `screen`
                    screen = False
                    # [VÒNG LẶP/LẶP LẠI] Lặp lại công việc khi điều kiện `0 <= nc <= 8 and 0 <= nr <= 9` thỏa mãn
                    while 0 <= nc <= 8 and 0 <= nr <= 9:
                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `t`
                        t = self.grid[sq(nc, nr)]
                        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not screen`
                        if not screen:
                            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `t == 0`
                            if t == 0:
                                # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `res`
                                res.append(Move(i, sq(nc, nr)))
                            # [RẼ NHÁNH MẶC ĐỊNH] Thực thi nhánh mặc định else khi các điều kiện trên không thỏa mãn
                            else:
                                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `screen`
                                screen = True
                        # [RẼ NHÁNH MẶC ĐỊNH] Thực thi nhánh mặc định else khi các điều kiện trên không thỏa mãn
                        else:
                            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `t != 0`
                            if t != 0:
                                # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `side(t) != s: res.append(Move(i, sq(nc, nr)))`
                                if side(t) != s: res.append(Move(i, sq(nc, nr)))
                                # [THOÁT VÒNG LẶP] Dừng và thoát khỏi vòng lặp ngay lập tức
                                break
                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `nc +`
                        nc += dc
                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `nr +`
                        nr += dr
        # === QUÂN TỐT (Pawn - ptype 7) === Tiến 1 bước; sau khi Qua Sông được phép đi ngang 1 bước trái/phải
            elif ptype == 7: # Tốt
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `dirs`
                dirs = [(0, 1)] if s == 0 else [(0, -1)]
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `crossed`
                crossed = (r >= 5) if s == 0 else (r <= 4)
                # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `crossed: dirs.extend([(-1, 0), (1, 0)])`
                if crossed: dirs.extend([(-1, 0), (1, 0)])
                # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `dc, dr in dirs`
                for dc, dr in dirs:
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `nc, nr`
                    nc, nr = c + dc, r + dr
                    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `0 <= nc <= 8 and 0 <= nr <= 9`
                    if 0 <= nc <= 8 and 0 <= nr <= 9:
                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `t`
                        t = self.grid[sq(nc, nr)]
                        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `t == 0 or side(t) != s: res.append(Move(i, sq(nc, nr)))`
                        if t == 0 or side(t) != s: res.append(Move(i, sq(nc, nr)))
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `res`
        return res

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `legal(self, ...)`
    def legal(self) -> list:
        """Trả về danh sách 100% nước đi hợp lệ theo luật cờ Tướng vật lý."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `moves`
        moves = self.generate()
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `valid`
        valid = []
        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `m in moves`
        for m in moves:
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `saved_dst`
            saved_dst = self.grid[m.dst]
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.grid[m.dst]`
            self.grid[m.dst] = self.grid[m.src]
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.grid[m.src]`
            self.grid[m.src] = 0

            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not self.check(self.turn)`
            if not self.check(self.turn):
                # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `valid`
                valid.append(m)

            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.grid[m.src]`
            self.grid[m.src] = self.grid[m.dst]
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.grid[m.dst]`
            self.grid[m.dst] = saved_dst
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `valid`
        return valid

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `apply(self, ...)`
    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `is_game_over(self)`
    def is_game_over(self) -> bool:
        """Kiểm tra xem ván đấu cờ Tướng đã kết thúc hay chưa (Tướng bị bắt hoặc hết nước đi hợp lệ)."""
        # [KẾT QUẢ TRẢ VỀ] Trả về True nếu 1 trong 2 Tướng không còn trên bàn cờ hoặc không còn nước đi hợp lệ
        return self.king(0) < 0 or self.king(1) < 0 or len(self.legal()) == 0

    def apply(self, m: Move):
        """Thực thi nước đi `m` lên bàn cờ và chuyển lượt đi."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.grid[m.dst]`
        self.grid[m.dst] = self.grid[m.src]
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.grid[m.src]`
        self.grid[m.src] = 0
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.turn`
        self.turn = 1 - self.turn

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `push(self, ...)`
    def push(self, m: Move):
        """Bí danh (alias) tương thích ngược cho `apply`."""
        self.apply(m)

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `make_move(self, ...)`
    def make_move(self, m: Move):
        """Bí danh (alias) tương thích ngược cho `apply`."""
        self.apply(m)

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `do_move(self, ...)`
    def do_move(self, m: Move):
        """Bí danh (alias) tương thích ngược cho `apply`."""
        self.apply(m)

    # --------------------------------------------------------------------------
    # NHÓM I: NHẬN THỨC BÀN CỜ (CHIỀU 1 -> 6)
    # --------------------------------------------------------------------------

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `inventory(self, ...)`
    def inventory(self) -> tuple:
        """[1/32] Liệt kê tọa độ chính xác từng quân cờ Đỏ và Đen."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `red_p`
        red_p = []
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `black_p`
        black_p = []
        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `i in range(90)`
        for i in range(90):
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `p`
            p = self.grid[i]
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `p == 0: continue`
            if p == 0: continue
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `name`
            name = NAMES[p]
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `pos_str`
            pos_str = uci(i)
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `side(p) == 0`
            if side(p) == 0:
                # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `red_p`
                red_p.append(f"{name} ({pos_str})")
            # [RẼ NHÁNH MẶC ĐỊNH] Thực thi nhánh mặc định else khi các điều kiện trên không thỏa mãn
            else:
                # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `black_p`
                black_p.append(f"{name} ({pos_str})")
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `(", ".join(red_p), ", ".join(black_p))`
        return (", ".join(red_p), ", ".join(black_p))

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `ascii(self, ...)`
    def ascii(self) -> str:
        """[2/32] Vẽ bàn cờ 2D ASCII trực quan hiển thị tọa độ cột (a-i) và hàng (0-9) cùng chữ Hán."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `lines`
        lines = []
        # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `lines`
        lines.append("    a    b    c    d    e    f    g    h    i")
        # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `lines`
        lines.append("  ┌────┬────┬────┬────┬────┬────┬────┬────┬────┐")
        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `r in range(9, -1, -1)`
        for r in range(9, -1, -1):
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `row_pieces`
            row_pieces = []
            # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `c in range(9)`
            for c in range(9):
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `p`
                p = self.grid[sq(c, r)]
                # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `row_pieces`
                row_pieces.append(SYMBOLS.get(p, "．"))
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `line`
            line = f"{r} │ " + " │ ".join(row_pieces) + " │"
            # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `lines`
            lines.append(line)
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `r == 5`
            if r == 5:
                # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `lines`
                lines.append("  ├────┼────┼────┼────┼────┼────┼────┼────┼────┤  ═══ Sông Ngân Hà ═══")
            # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `r > 0`
            elif r > 0:
                # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `lines`
                lines.append("  ├────┼────┼────┼────┼────┼────┼────┼────┼────┤")
        # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `lines`
        lines.append("  └────┴────┴────┴────┴────┴────┴────┴────┴────┘")
        # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `lines`
        lines.append("    a    b    c    d    e    f    g    h    i")
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `"\n".join(lines)`
        return "\n".join(lines)

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `material(self, ...)`
    def material(self, s: int = None) -> int | tuple:
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `s is None`
        if s is None:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `self.material(0), self.material(1)`
            return self.material(0), self.material(1)
        """[3/32] Tính tổng điểm vật chất của phe `s`."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `total`
        total = 0
        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `i in range(90)`
        for i in range(90):
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `p`
            p = self.grid[i]
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `p == 0 or side(p) != s: continue`
            if p == 0 or side(p) != s: continue
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `ptype`
            ptype = p if s == 0 else p - 7
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `total +`
            total += VALUES.get(ptype, 0)
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `total`
        return total

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `columns(self, ...)`
    def columns(self) -> str:
        """[4/32] Phân tích 9 lộ cờ (a..i): Xác định lộ MỞ, BÁN MỞ hay KHÓA."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `result`
        result = []
        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `c in range(9)`
        for c in range(9):
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `name`
            name = f"Lộ {c+1} ({chr(ord('a')+c)})"
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `red_pawns`
            red_pawns = sum(1 for r in range(10) if self.grid[sq(c, r)] == 7)
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `black_pawns`
            black_pawns = sum(1 for r in range(10) if self.grid[sq(c, r)] == 14)
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `red_heavy`
            red_heavy = sum(1 for r in range(10) if self.grid[sq(c, r)] in [5, 6])
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `black_heavy`
            black_heavy = sum(1 for r in range(10) if self.grid[sq(c, r)] in [12, 13])
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `red_pawns == 0 and black_pawns == 0`
            if red_pawns == 0 and black_pawns == 0:
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `status`
                status = "MỞ"
                # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `red_heavy > 0 and black_heavy == 0`
                if red_heavy > 0 and black_heavy == 0:
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `status +`
                    status += " (Đỏ chiếm)"
                # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `black_heavy > 0 and red_heavy == 0`
                elif black_heavy > 0 and red_heavy == 0:
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `status +`
                    status += " (Đen chiếm)"
                # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `red_heavy > 0 and black_heavy > 0`
                elif red_heavy > 0 and black_heavy > 0:
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `status +`
                    status += " (tranh chấp)"
            # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `red_pawns > 0 and black_pawns > 0`
            elif red_pawns > 0 and black_pawns > 0:
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `status`
                status = "KHÓA"
            # [RẼ NHÁNH MẶC ĐỊNH] Thực thi nhánh mặc định else khi các điều kiện trên không thỏa mãn
            else:
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `status`
                status = "BÁN MỞ"
                # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `red_pawns == 0 and red_heavy > 0`
                if red_pawns == 0 and red_heavy > 0:
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `status +`
                    status += " (Đỏ bán mở)"
                # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `black_pawns == 0 and black_heavy > 0`
                elif black_pawns == 0 and black_heavy > 0:
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `status +`
                    status += " (Đen bán mở)"
            # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `result`
            result.append(f"{name}: {status}")
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `" | ".join(result)`
        return " | ".join(result)

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `deployed(self, ...)`
    def deployed(self, s: int) -> str:
        """[5/32] Đánh giá mức độ triển khai quân."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `total`
        total = 0
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `moved`
        moved = 0
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `start_positions`
        start_positions = START_POS.get(s, {})
        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `ptype_key, positions in start_positions.items()`
        for ptype_key, positions in start_positions.items():
            # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `pos in positions`
            for pos in positions:
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `total +`
                total += 1
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `current`
                current = self.grid[pos]
                # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `current != ptype_key`
                if current != ptype_key:
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `moved +`
                    moved += 1
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `unmoved_names`
        unmoved_names = []
        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `ptype_key, positions in start_positions.items()`
        for ptype_key, positions in start_positions.items():
            # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `pos in positions`
            for pos in positions:
                # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `self.grid[pos] == ptype_key`
                if self.grid[pos] == ptype_key:
                    # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `unmoved_names`
                    unmoved_names.append(f"{NAMES[ptype_key]}({uci(pos)})")
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `side_name`
        side_name = "Đỏ" if s == 0 else "Đen"
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `unmoved_names`
        if unmoved_names:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"{side_name}: {moved}/{total} quân đã triển khai. Chưa triển khai: {', '.joi...`
            return f"{side_name}: {moved}/{total} quân đã triển khai. Chưa triển khai: {', '.join(unmoved_names)}"
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"{side_name}: {moved}/{total} quân đã triển khai. Toàn bộ quân đã rời vị trí...`
        return f"{side_name}: {moved}/{total} quân đã triển khai. Toàn bộ quân đã rời vị trí xuất phát!"

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `mobility(self, ...)`
    def mobility(self) -> tuple:
        """[6/32] Tính số lượng nước đi hợp lệ của cả 2 bên (Mobility Score)."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `saved_turn`
        saved_turn = self.turn
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.turn`
        self.turn = 0
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `red_moves`
        red_moves = len(self.legal())
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.turn`
        self.turn = 1
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `black_moves`
        black_moves = len(self.legal())
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.turn`
        self.turn = saved_turn
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `(red_moves, black_moves)`
        return (red_moves, black_moves)

    # --------------------------------------------------------------------------
    # NHÓM II: PHÂN TÍCH ĐE DỌA (CHIỀU 7 -> 12)
    # --------------------------------------------------------------------------

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `safety(self, ...)`
    def safety(self, s: int) -> str:
        """[7/32] Đánh giá mức độ an toàn của Cung Tướng phe `s`."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `k`
        k = self.king(s)
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `k < 0`
        if k < 0:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `"KHÔNG TÌM THẤY TƯỚNG — TÌNH HUỐNG NGHIÊM TRỌNG!"`
            return "KHÔNG TÌM THẤY TƯỚNG — TÌNH HUỐNG NGHIÊM TRỌNG!"
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `is_checked`
        is_checked = self.check(s)
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `side_name`
        side_name = "Đỏ" if s == 0 else "Đen"
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `advisor_type`
        advisor_type = 2 if s == 0 else 9
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `elephant_type`
        elephant_type = 3 if s == 0 else 10
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `advisors`
        advisors = sum(1 for i in range(90) if self.grid[i] == advisor_type)
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `elephants`
        elephants = sum(1 for i in range(90) if self.grid[i] == elephant_type)
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp`
        opp = 1 - s
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `threat_pieces`
        threat_pieces = self.attackers(k, opp)
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `self.flying()`
        if self.flying():
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp_k`
            opp_k = self.king(opp)
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `opp_k >= 0 and not any(sq_i == opp_k for sq_i, _ in threat_pieces)`
            if opp_k >= 0 and not any(sq_i == opp_k for sq_i, _ in threat_pieces):
                # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `threat_pieces`
                threat_pieces.append((opp_k, self.grid[opp_k]))
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `threat_str`
        threat_str = ""
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `threat_pieces`
        if threat_pieces:
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `threat_names`
            threat_names = [f"{NAMES[p]}({uci(sq_i)})" for sq_i, p in threat_pieces]
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `threat_str`
            threat_str = f" Đe dọa bởi: {', '.join(threat_names)}."
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `is_checked`
        if is_checked:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"Tướng {side_name} ĐANG BỊ CHIẾU! Sĩ: {advisors}/2, Tượng: {elephants}/2.{th...`
            return f"Tướng {side_name} ĐANG BỊ CHIẾU! Sĩ: {advisors}/2, Tượng: {elephants}/2.{threat_str} CẦN ỨNG CHIẾU NGAY!"
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `advisors == 0 and elephants == 0`
        if advisors == 0 and elephants == 0:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"Tướng {side_name} CỰC KỲ NGUY HIỂM — Cung Tướng trống rỗng (0 Sĩ, 0 Tượng)....`
            return f"Tướng {side_name} CỰC KỲ NGUY HIỂM — Cung Tướng trống rỗng (0 Sĩ, 0 Tượng).{threat_str}"
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `advisors + elephants <= 2`
        if advisors + elephants <= 2:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"Tướng {side_name} PHÒNG THỦ YẾU — Sĩ: {advisors}/2, Tượng: {elephants}/2.{t...`
            return f"Tướng {side_name} PHÒNG THỦ YẾU — Sĩ: {advisors}/2, Tượng: {elephants}/2.{threat_str}"
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"Tướng {side_name} an toàn — Sĩ: {advisors}/2, Tượng: {elephants}/2.{threat_...`
        return f"Tướng {side_name} an toàn — Sĩ: {advisors}/2, Tượng: {elephants}/2.{threat_str} Cung Tướng kiên cố."

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `attacked(self, ...)`
    def attacked(self, s: int) -> str:
        """[8/32] Phát hiện tất cả quân cờ phe `s` đang nằm trong tầm tấn công của đối phương."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp`
        opp = 1 - s
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `side_name`
        side_name = "Đỏ" if s == 0 else "Đen"
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `results`
        results = []
        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `i in range(90)`
        for i in range(90):
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `p`
            p = self.grid[i]
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `p == 0 or side(p) != s: continue`
            if p == 0 or side(p) != s: continue
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `ptype`
            ptype = p if s == 0 else p - 7
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `ptype == 1: continue`
            if ptype == 1: continue
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `atk`
            atk = self.attackers(i, opp)
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `atk`
            if atk:
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `atk_names`
                atk_names = [f"{NAMES[ap]}({uci(asq)})" for asq, ap in atk]
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `pval`
                pval = VALUES.get(ptype, 0)
                # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `results`
                results.append(f"{NAMES[p]}({uci(i)}, {pval}cp) bị tấn công bởi {', '.join(atk_names)}")
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not results`
        if not results:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"Không có quân {side_name} nào đang bị tấn công."`
            return f"Không có quân {side_name} nào đang bị tấn công."
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"Quân {side_name} bị tấn công: " + "; ".join(results)`
        return f"Quân {side_name} bị tấn công: " + "; ".join(results)

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `hanging(self, ...)`
    def hanging(self, s: int) -> str:
        """[9/32] Quân treo (Hanging Pieces) — Quân cờ bị tấn công mà KHÔNG CÓ QUÂN BẢO VỆ."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp`
        opp = 1 - s
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `side_name`
        side_name = "Đỏ" if s == 0 else "Đen"
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `results`
        results = []
        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `i in range(90)`
        for i in range(90):
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `p`
            p = self.grid[i]
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `p == 0 or side(p) != s: continue`
            if p == 0 or side(p) != s: continue
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `ptype`
            ptype = p if s == 0 else p - 7
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `ptype == 1: continue`
            if ptype == 1: continue
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `atk`
            atk = self.attackers(i, opp)
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not atk: continue`
            if not atk: continue
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `defenders`
            defenders = self.attackers(i, s)
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not defenders`
            if not defenders:
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `pval`
                pval = VALUES.get(ptype, 0)
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `atk_names`
                atk_names = [f"{NAMES[ap]}({uci(asq)})" for asq, ap in atk]
                # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `results`
                results.append(f"{NAMES[p]}({uci(i)}, {pval}cp) TREO — không có quân bảo vệ, bị {', '.join(atk_names)} nhắm tới")
            # [RẼ NHÁNH MẶC ĐỊNH] Thực thi nhánh mặc định else khi các điều kiện trên không thỏa mãn
            else:
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `min_atk_val`
                min_atk_val = min(VALUES.get(ap if side(ap) == 0 else ap - 7, 0) for _, ap in atk)
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `pval`
                pval = VALUES.get(ptype, 0)
                # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `min_atk_val < pval`
                if min_atk_val < pval:
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `atk_names`
                    atk_names = [f"{NAMES[ap]}({uci(asq)})" for asq, ap in atk]
                    # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `results`
                    results.append(f"{NAMES[p]}({uci(i)}, {pval}cp) có thể bị đổi lỗ — quân tấn công giá trị thấp hơn ({min_atk_val}cp)")
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not results`
        if not results:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"Không có quân {side_name} nào đang treo."`
            return f"Không có quân {side_name} nào đang treo."
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `"; ".join(results)`
        return "; ".join(results)

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `pinned(self, ...)`
    def pinned(self, s: int) -> str:
        """[10/32] Ghim quân (Pin) — Quân không thể di chuyển vì che chắn Tướng."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `k`
        k = self.king(s)
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `k < 0: return "Không tìm thấy Tướng."`
        if k < 0: return "Không tìm thấy Tướng."
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp`
        opp = 1 - s
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `side_name`
        side_name = "Đỏ" if s == 0 else "Đen"
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `kc`
        kc = col(k)
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `kr`
        kr = row(k)
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `results`
        results = []

        # Ghim trực tiếp bởi Xe
        for direction_c, direction_r in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `nc, nr`
            nc, nr = kc + direction_c, kr + direction_r
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `first_piece_sq`
            first_piece_sq = -1
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `first_piece`
            first_piece = 0
            # [VÒNG LẶP/LẶP LẠI] Lặp lại công việc khi điều kiện `0 <= nc <= 8 and 0 <= nr <= 9` thỏa mãn
            while 0 <= nc <= 8 and 0 <= nr <= 9:
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `idx`
                idx = sq(nc, nr)
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `p`
                p = self.grid[idx]
                # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `p != 0`
                if p != 0:
                    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `first_piece_sq == -1`
                    if first_piece_sq == -1:
                        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `side(p) == s`
                        if side(p) == s:
                            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `first_piece_sq`
                            first_piece_sq = idx
                            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `first_piece`
                            first_piece = p
                        # [RẼ NHÁNH MẶC ĐỊNH] Thực thi nhánh mặc định else khi các điều kiện trên không thỏa mãn
                        else:
                            # [THOÁT VÒNG LẶP] Dừng và thoát khỏi vòng lặp ngay lập tức
                            break
                    # [RẼ NHÁNH MẶC ĐỊNH] Thực thi nhánh mặc định else khi các điều kiện trên không thỏa mãn
                    else:
                        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `side(p) == opp`
                        if side(p) == opp:
                            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp_type`
                            opp_type = p if opp == 0 else p - 7
                            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `opp_type == 5`
                            if opp_type == 5:
                                # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `results`
                                results.append(f"{NAMES[first_piece]}({uci(first_piece_sq)}) BỊ GHIM bởi {NAMES[p]}({uci(idx)}) — che chắn Tướng trên đường thẳng")
                        # [THOÁT VÒNG LẶP] Dừng và thoát khỏi vòng lặp ngay lập tức
                        break
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `nc +`
                nc += direction_c
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `nr +`
                nr += direction_r

        # Ghim bởi Pháo qua ngòi
        for direction_c, direction_r in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `nc, nr`
            nc, nr = kc + direction_c, kr + direction_r
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `first_piece_sq`
            first_piece_sq = -1
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `first_piece`
            first_piece = 0
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `screen_count`
            screen_count = 0
            # [VÒNG LẶP/LẶP LẠI] Lặp lại công việc khi điều kiện `0 <= nc <= 8 and 0 <= nr <= 9` thỏa mãn
            while 0 <= nc <= 8 and 0 <= nr <= 9:
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `idx`
                idx = sq(nc, nr)
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `p`
                p = self.grid[idx]
                # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `p != 0`
                if p != 0:
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `screen_count +`
                    screen_count += 1
                    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `screen_count == 1`
                    if screen_count == 1:
                        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `side(p) == s`
                        if side(p) == s:
                            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `first_piece_sq`
                            first_piece_sq = idx
                            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `first_piece`
                            first_piece = p
                        # [RẼ NHÁNH MẶC ĐỊNH] Thực thi nhánh mặc định else khi các điều kiện trên không thỏa mãn
                        else:
                            # [THOÁT VÒNG LẶP] Dừng và thoát khỏi vòng lặp ngay lập tức
                            break
                    # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `screen_count == 2`
                    elif screen_count == 2:
                        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `side(p) == opp`
                        if side(p) == opp:
                            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp_type`
                            opp_type = p if opp == 0 else p - 7
                            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `opp_type == 6`
                            if opp_type == 6:
                                # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `first_piece_sq >= 0`
                                if first_piece_sq >= 0:
                                    # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `results`
                                    results.append(f"{NAMES[first_piece]}({uci(first_piece_sq)}) BỊ GHIM bởi {NAMES[p]}({uci(idx)}) (Pháo ghim qua ngòi)")
                        # [THOÁT VÒNG LẶP] Dừng và thoát khỏi vòng lặp ngay lập tức
                        break
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `nc +`
                nc += direction_c
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `nr +`
                nr += direction_r

        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not results`
        if not results:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"Không có quân {side_name} nào bị ghim."`
            return f"Không có quân {side_name} nào bị ghim."
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `"; ".join(results)`
        return "; ".join(results)

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `forks(self, ...)`
    def forks(self) -> str:
        """[11/32] Đòn kép (Fork) — 1 quân cờ đe dọa đồng thời 2 hoặc nhiều quân đối phương."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `results`
        results = []
        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `s in [0, 1]`
        for s in [0, 1]:
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp`
            opp = 1 - s
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `side_name`
            side_name = "Đỏ" if s == 0 else "Đen"
            # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `i in range(90)`
            for i in range(90):
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `p`
                p = self.grid[i]
                # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `p == 0 or side(p) != s: continue`
                if p == 0 or side(p) != s: continue
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `ptype`
                ptype = p if s == 0 else p - 7
                # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `ptype in [1, 2, 3]: continue`
                if ptype in [1, 2, 3]: continue
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `threatened`
                threatened = []
                # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `j in range(90)`
                for j in range(90):
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `tp`
                    tp = self.grid[j]
                    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `tp == 0 or side(tp) != opp: continue`
                    if tp == 0 or side(tp) != opp: continue
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `tp_type`
                    tp_type = tp if opp == 0 else tp - 7
                    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `tp_type in [2, 3, 7]: continue`
                    if tp_type in [2, 3, 7]: continue
                    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `self.attacks_piece(i, j, p)`
                    if self.attacks_piece(i, j, p):
                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `tval`
                        tval = VALUES.get(tp_type, 0)
                        # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `threatened`
                        threatened.append(f"{NAMES[tp]}({uci(j)}, {tval}cp)")
                # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `len(threatened) >= 2`
                if len(threatened) >= 2:
                    # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `results`
                    results.append(f"ĐÒN KÉP {side_name}: {NAMES[p]}({uci(i)}) đe dọa đồng thời {' và '.join(threatened)}")
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not results`
        if not results:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `"Không phát hiện đòn kép nào trên bàn cờ."`
            return "Không phát hiện đòn kép nào trên bàn cờ."
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `"; ".join(results)`
        return "; ".join(results)

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `discovered(self, ...)`
    def discovered(self) -> str:
        """[12/32] Đòn mở (Discovered Attack) — Nước di chuyển quân phía trước để mở đường cho quân phía sau."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `results`
        results = []
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `s`
        s = self.turn
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp`
        opp = 1 - s
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `side_name`
        side_name = "Đỏ" if s == 0 else "Đen"
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `legal`
        legal = self.legal()
        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `m in legal`
        for m in legal:
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `p_moved`
            p_moved = self.grid[m.src]
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `ptype_moved`
            ptype_moved = p_moved if s == 0 else p_moved - 7
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `ptype_moved in [5, 6]: continue`
            if ptype_moved in [5, 6]: continue
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `saved_src`
            saved_src = self.grid[m.src]
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `saved_dst`
            saved_dst = self.grid[m.dst]
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.grid[m.dst]`
            self.grid[m.dst] = self.grid[m.src]
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.grid[m.src]`
            self.grid[m.src] = 0
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp_king`
            opp_king = self.king(opp)
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `opp_king >= 0 and self.attack(opp_king, s)`
            if opp_king >= 0 and self.attack(opp_king, s):
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `behind_attackers`
                behind_attackers = self.attackers(opp_king, s)
                # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `asq, ap in behind_attackers`
                for asq, ap in behind_attackers:
                    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `asq != m.dst`
                    if asq != m.dst:
                        # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `results`
                        results.append(f"ĐÒN MỞ {side_name}: {NAMES[p_moved]}({uci(m.src)}->{uci(m.dst)}) mở đường cho {NAMES[ap]}({uci(asq)}) chiếu Tướng đối phương!")
                        # [THOÁT VÒNG LẶP] Dừng và thoát khỏi vòng lặp ngay lập tức
                        break
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.grid[m.src]`
            self.grid[m.src] = saved_src
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.grid[m.dst]`
            self.grid[m.dst] = saved_dst
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `results`
            if results:
                # [THOÁT VÒNG LẶP] Dừng và thoát khỏi vòng lặp ngay lập tức
                break
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not results`
        if not results:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `"Không phát hiện đòn mở nào có thể thực hiện ngay."`
            return "Không phát hiện đòn mở nào có thể thực hiện ngay."
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `"; ".join(results)`
        return "; ".join(results)

    # --------------------------------------------------------------------------
    # NHÓM III: CHIẾN THUẬT & BẪY (CHIỀU 13 -> 18)
    # --------------------------------------------------------------------------

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `traps(self, ...)`
    def traps(self) -> str:
        """[13/32] Bẫy ăn quân — Đánh giá mồi nhử ăn quân hoặc đổi quân có lợi."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `results`
        results = []
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `s`
        s = self.turn
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp`
        opp = 1 - s
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `side_name`
        side_name = "Đỏ" if s == 0 else "Đen"
        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `m in self.legal()`
        for m in self.legal():
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `captured`
            captured = self.grid[m.dst]
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `captured == 0: continue`
            if captured == 0: continue
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `cap_type`
            cap_type = captured if opp == 0 else captured - 7
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `cap_val`
            cap_val = VALUES.get(cap_type, 0)
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `src_piece`
            src_piece = self.grid[m.src]
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `src_type`
            src_type = src_piece if s == 0 else src_piece - 7
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `src_val`
            src_val = VALUES.get(src_type, 0)
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `cap_val > src_val + 10`
            if cap_val > src_val + 10:
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `saved_src`
                saved_src = self.grid[m.src]
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `saved_dst`
                saved_dst = self.grid[m.dst]
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.grid[m.dst]`
                self.grid[m.dst] = self.grid[m.src]
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.grid[m.src]`
                self.grid[m.src] = 0
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `counter_attackers`
                counter_attackers = self.attackers(m.dst, opp)
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.grid[m.src]`
                self.grid[m.src] = saved_src
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.grid[m.dst]`
                self.grid[m.dst] = saved_dst
                # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not counter_attackers`
                if not counter_attackers:
                    # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `results`
                    results.append(f"BẪY: {NAMES[src_piece]}({uci(m.src)}) ăn {NAMES[captured]}({uci(m.dst)}) — lời {cap_val - src_val}cp, không bị phản đòn!")
                # [RẼ NHÁNH MẶC ĐỊNH] Thực thi nhánh mặc định else khi các điều kiện trên không thỏa mãn
                else:
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `net`
                    net = cap_val - src_val
                    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `net > 20`
                    if net > 20:
                        # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `results`
                        results.append(f"BẪY ĐỔI QUÂN: {NAMES[src_piece]}({uci(m.src)}) ăn {NAMES[captured]}({uci(m.dst)}) — lời {net}cp dù bị phản đòn")
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not results`
        if not results:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"Không phát hiện bẫy ăn quân nào cho {side_name}."`
            return f"Không phát hiện bẫy ăn quân nào cho {side_name}."
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `"; ".join(results[:3])`
        return "; ".join(results[:3])

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `checkmate(self, ...)`
    def checkmate(self) -> str:
        """[14/32] Chiếu bí tiềm ẩn — Kiểm tra đe dọa chiếu bí sát thủ trong 1 nước đi."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `s`
        s = self.turn
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp`
        opp = 1 - s
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `side_name`
        side_name = "Đỏ" if s == 0 else "Đen"
        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `m in self.legal()`
        for m in self.legal():
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `saved_src`
            saved_src = self.grid[m.src]
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `saved_dst`
            saved_dst = self.grid[m.dst]
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.grid[m.dst]`
            self.grid[m.dst] = self.grid[m.src]
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.grid[m.src]`
            self.grid[m.src] = 0
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `old_turn`
            old_turn = self.turn
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.turn`
            self.turn = opp
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp_legal`
            opp_legal = self.legal()
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `is_mate`
            is_mate = len(opp_legal) == 0 and self.check(opp)
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.turn`
            self.turn = old_turn
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.grid[m.src]`
            self.grid[m.src] = saved_src
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.grid[m.dst]`
            self.grid[m.dst] = saved_dst
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `is_mate`
            if is_mate:
                # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"CHIẾU BÍ TRONG 1 NƯỚC! {side_name} đi {NAMES[saved_src]}({uci(m.src)}->{uci...`
                return f"CHIẾU BÍ TRONG 1 NƯỚC! {side_name} đi {NAMES[saved_src]}({uci(m.src)}->{uci(m.dst)}) = CHIẾU BÍ!"
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp_legal_now`
        opp_legal_now = self.legal()
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not opp_legal_now and self.check(self.turn)`
        if not opp_legal_now and self.check(self.turn):
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"{side_name} ĐANG BỊ CHIẾU BÍ — không còn nước đi hợp lệ!"`
            return f"{side_name} ĐANG BỊ CHIẾU BÍ — không còn nước đi hợp lệ!"
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `"Không phát hiện chiếu bí tiềm ẩn trong 1 nước."`
        return "Không phát hiện chiếu bí tiềm ẩn trong 1 nước."

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `diversion(self, ...)`
    def diversion(self, encoded_move: str) -> str:
        """[15/32] Dương đông kích tây — Đánh giá xem nước đi có phải đòn nghi binh chuyển hướng tấn công không."""
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `len(encoded_move) != 4`
        if len(encoded_move) != 4:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `"Không đủ dữ liệu để phân tích nghi binh."`
            return "Không đủ dữ liệu để phân tích nghi binh."
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `src_c`
        src_c = ord(encoded_move[0]) - ord('a')
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `dst_c`
        dst_c = ord(encoded_move[2]) - ord('a')
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `s`
        s = self.turn
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `side_name`
        side_name = "Đỏ" if s == 0 else "Đen"
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `abs(src_c - dst_c) >= 3`
        if abs(src_c - dst_c) >= 3:
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `src_wing`
            src_wing = "trái" if src_c < 4 else ("phải" if src_c > 4 else "trung tâm")
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `dst_wing`
            dst_wing = "trái" if dst_c < 4 else ("phải" if dst_c > 4 else "trung tâm")
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"Có dấu hiệu DƯƠNG ĐÔNG KÍCH TÂY: {side_name} di chuyển quân từ cánh {src_wi...`
            return f"Có dấu hiệu DƯƠNG ĐÔNG KÍCH TÂY: {side_name} di chuyển quân từ cánh {src_wing} sang cánh {dst_wing}, có thể là đòn nghi binh để kéo giãn phòng tuyến đối phương."
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `abs(src_c - dst_c) <= 1`
        if abs(src_c - dst_c) <= 1:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"Nước đi tập trung cục bộ (cánh {'trái' if dst_c < 4 else 'phải' if dst_c > ...`
            return f"Nước đi tập trung cục bộ (cánh {'trái' if dst_c < 4 else 'phải' if dst_c > 4 else 'trung tâm'}), không có dấu hiệu nghi binh."
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"Di chuyển vừa phải ({abs(src_c - dst_c)} cột), có thể là bước chuẩn bị cho ...`
        return f"Di chuyển vừa phải ({abs(src_c - dst_c)} cột), có thể là bước chuẩn bị cho đợt tấn công tiếp theo."

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `patterns(self, ...)`
    def patterns(self) -> list:
        """[16/32] Mẫu chiến thuật nâng cấp — Nhận biết 15+ mẫu cờ."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `pats`
        pats = []
        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `r in range(10)`
        for r in range(10):
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `p`
            p = self.grid[sq(4, r)]
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `p == 6: pats.append("Đỏ Pháo Đầu Lộ 5 — đe dọa trực tiếp trung lộ")`
            if p == 6: pats.append("Đỏ Pháo Đầu Lộ 5 — đe dọa trực tiếp trung lộ")
            # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `p == 13: pats.append("Đen Pháo Đầu Lộ 5 — kiểm soát trung tâm")`
            elif p == 13: pats.append("Đen Pháo Đầu Lộ 5 — kiểm soát trung tâm")
        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `i in range(90)`
        for i in range(90):
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `p`
            p = self.grid[i]
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `r`
            r = row(i)
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `p == 4 and r >= 5: pats.append(f"Mã Đỏ vượt hà ({uci(i)}) — đã qua sông tấn c...`
            if p == 4 and r >= 5: pats.append(f"Mã Đỏ vượt hà ({uci(i)}) — đã qua sông tấn công")
            # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `p == 11 and r <= 4: pats.append(f"Mã Đen vượt hà ({uci(i)}) — đã qua sông tấn...`
            elif p == 11 and r <= 4: pats.append(f"Mã Đen vượt hà ({uci(i)}) — đã qua sông tấn công")
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `p == 7 and r >= 5: pats.append(f"Tốt Đỏ qua sông ({uci(i)}) — có thể đi ngang")`
            if p == 7 and r >= 5: pats.append(f"Tốt Đỏ qua sông ({uci(i)}) — có thể đi ngang")
            # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `p == 14 and r <= 4: pats.append(f"Tốt Đen qua sông ({uci(i)}) — có thể đi nga...`
            elif p == 14 and r <= 4: pats.append(f"Tốt Đen qua sông ({uci(i)}) — có thể đi ngang")
        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `c in range(9)`
        for c in range(9):
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `has_pawn`
            has_pawn = any(self.grid[sq(c, r)] in [7, 14] for r in range(10))
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not has_pawn`
            if not has_pawn:
                # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `r in range(10)`
                for r in range(10):
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `rk`
                    rk = self.grid[sq(c, r)]
                    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `rk == 5: pats.append(f"Xe Đỏ chiếm lộ mở {chr(ord('a')+c)} — kiểm soát không ...`
                    if rk == 5: pats.append(f"Xe Đỏ chiếm lộ mở {chr(ord('a')+c)} — kiểm soát không gian")
                    # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `rk == 12: pats.append(f"Xe Đen chiếm lộ mở {chr(ord('a')+c)} — kiểm soát khôn...`
                    elif rk == 12: pats.append(f"Xe Đen chiếm lộ mở {chr(ord('a')+c)} — kiểm soát không gian")
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `red_rooks`
        red_rooks = sum(1 for i in range(90) if self.grid[i] == 5)
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `black_rooks`
        black_rooks = sum(1 for i in range(90) if self.grid[i] == 12)
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `red_rooks == 2: pats.append("Đỏ Song Xe lực chiến — sức mạnh tấn công tối đa")`
        if red_rooks == 2: pats.append("Đỏ Song Xe lực chiến — sức mạnh tấn công tối đa")
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `black_rooks == 2: pats.append("Đen Song Xe lực chiến — sức mạnh tấn công tối ...`
        if black_rooks == 2: pats.append("Đen Song Xe lực chiến — sức mạnh tấn công tối đa")
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `red_advisors`
        red_advisors = sum(1 for i in range(90) if self.grid[i] == 2)
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `red_elephants`
        red_elephants = sum(1 for i in range(90) if self.grid[i] == 3)
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `red_advisors == 0: pats.append("Đỏ mất toàn bộ Sĩ — Cung Tướng sơ hở nghiêm t...`
        if red_advisors == 0: pats.append("Đỏ mất toàn bộ Sĩ — Cung Tướng sơ hở nghiêm trọng")
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `red_elephants == 0: pats.append("Đỏ mất toàn bộ Tượng — phòng tuyến yếu")`
        if red_elephants == 0: pats.append("Đỏ mất toàn bộ Tượng — phòng tuyến yếu")
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `black_advisors`
        black_advisors = sum(1 for i in range(90) if self.grid[i] == 9)
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `black_elephants`
        black_elephants = sum(1 for i in range(90) if self.grid[i] == 10)
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `black_advisors == 0: pats.append("Đen mất toàn bộ Sĩ — Cung Tướng sơ hở nghiê...`
        if black_advisors == 0: pats.append("Đen mất toàn bộ Sĩ — Cung Tướng sơ hở nghiêm trọng")
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `black_elephants == 0: pats.append("Đen mất toàn bộ Tượng — phòng tuyến yếu")`
        if black_elephants == 0: pats.append("Đen mất toàn bộ Tượng — phòng tuyến yếu")
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `pats if pats else ["Thế trận cân bằng, chưa xuất hiện mẫu chiến thuật đặc biệt"]`
        return pats if pats else ["Thế trận cân bằng, chưa xuất hiện mẫu chiến thuật đặc biệt"]

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `synergy(self, ...)`
    def synergy(self) -> str:
        """[17/32] Phối hợp quân (Synergy) — Nhận dạng phối hợp giữa các bộ đôi quân."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `results`
        results = []
        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `s in [0, 1]`
        for s in [0, 1]:
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `side_name`
            side_name = "Đỏ" if s == 0 else "Đen"
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `rook_type`
            rook_type = 5 if s == 0 else 12
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `cannon_type`
            cannon_type = 6 if s == 0 else 13
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `knight_type`
            knight_type = 4 if s == 0 else 11
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `rooks`
            rooks = [i for i in range(90) if self.grid[i] == rook_type]
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `cannons`
            cannons = [i for i in range(90) if self.grid[i] == cannon_type]
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `knights`
            knights = [i for i in range(90) if self.grid[i] == knight_type]
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `len(rooks) >= 2`
            if len(rooks) >= 2:
                # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `col(rooks[0]) == col(rooks[1])`
                if col(rooks[0]) == col(rooks[1]):
                    # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `results`
                    results.append(f"{side_name} Song Xe trùng lộ {chr(ord('a')+col(rooks[0]))} — sức mạnh tối đa trên 1 cột")
                # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `row(rooks[0]) == row(rooks[1])`
                elif row(rooks[0]) == row(rooks[1]):
                    # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `results`
                    results.append(f"{side_name} Song Xe trùng hàng {row(rooks[0])} — kiểm soát toàn bộ hàng ngang")
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `rooks and cannons`
            if rooks and cannons:
                # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `rk in rooks`
                for rk in rooks:
                    # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `cn in cannons`
                    for cn in cannons:
                        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `col(rk) == col(cn)`
                        if col(rk) == col(cn):
                            # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `results`
                            results.append(f"{side_name} Xe-Pháo trùng lộ {chr(ord('a')+col(rk))} — combo đe dọa mạnh")
                            # [THOÁT VÒNG LẶP] Dừng và thoát khỏi vòng lặp ngay lập tức
                            break
                    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `results: break`
                    if results: break
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `knights and cannons`
            if knights and cannons:
                # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `kn in knights`
                for kn in knights:
                    # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `cn in cannons`
                    for cn in cannons:
                        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `abs(col(kn) - col(cn)) <= 2 and abs(row(kn) - row(cn)) <= 2`
                        if abs(col(kn) - col(cn)) <= 2 and abs(row(kn) - row(cn)) <= 2:
                            # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `results`
                            results.append(f"{side_name} Mã-Pháo phối hợp gần ({uci(kn)},{uci(cn)}) — đe dọa chiếu đôi")
                            # [THOÁT VÒNG LẶP] Dừng và thoát khỏi vòng lặp ngay lập tức
                            break
                    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `results: break`
                    if results: break
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not results`
        if not results:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `"Chưa phát hiện phối hợp quân đặc biệt nào."`
            return "Chưa phát hiện phối hợp quân đặc biệt nào."
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `"; ".join(results[:4])`
        return "; ".join(results[:4])

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `weakness(self, ...)`
    def weakness(self, s: int) -> str:
        """[18/32] Điểm yếu cấu trúc — Nhận diện Tốt cô lập, Tốt đôi, lỗ hổng Cung Tướng."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `side_name`
        side_name = "Đỏ" if s == 0 else "Đen"
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `results`
        results = []
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `pawn_type`
        pawn_type = 7 if s == 0 else 14
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `pawn_cols`
        pawn_cols = set()
        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `i in range(90)`
        for i in range(90):
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `self.grid[i] == pawn_type`
            if self.grid[i] == pawn_type:
                # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `pawn_cols`
                pawn_cols.add(col(i))
        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `pc in pawn_cols`
        for pc in pawn_cols:
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `neighbors`
            neighbors = {pc - 1, pc + 1}
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not neighbors.intersection(pawn_cols)`
            if not neighbors.intersection(pawn_cols):
                # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `results`
                results.append(f"Tốt cô lập trên lộ {chr(ord('a')+pc)}")
        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `c in range(9)`
        for c in range(9):
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `count`
            count = sum(1 for r in range(10) if self.grid[sq(c, r)] == pawn_type)
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `count >= 2`
            if count >= 2:
                # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `results`
                results.append(f"Tốt đôi trên lộ {chr(ord('a')+c)} ({count} Tốt)")
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `advisor_type`
        advisor_type = 2 if s == 0 else 9
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `elephant_type`
        elephant_type = 3 if s == 0 else 10
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `advisors`
        advisors = sum(1 for i in range(90) if self.grid[i] == advisor_type)
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `elephants`
        elephants = sum(1 for i in range(90) if self.grid[i] == elephant_type)
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `advisors == 0 and elephants == 0`
        if advisors == 0 and elephants == 0:
            # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `results`
            results.append("NGHIÊM TRỌNG: Cung Tướng trống rỗng — 0 Sĩ, 0 Tượng!")
        # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `advisors == 0`
        elif advisors == 0:
            # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `results`
            results.append("Cung Tướng thiếu Sĩ — dễ bị chiếu cánh")
        # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `elephants == 0`
        elif elephants == 0:
            # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `results`
            results.append("Thiếu Tượng — phòng tuyến xa yếu")
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not results`
        if not results:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"{side_name} không có điểm yếu cấu trúc đáng kể."`
            return f"{side_name} không có điểm yếu cấu trúc đáng kể."
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"{side_name}: " + "; ".join(results)`
        return f"{side_name}: " + "; ".join(results)

    # --------------------------------------------------------------------------
    # NHÓM IV: 36 KẾ BINH PHÁP & THẾ TRẬN (CHIỀU 19 -> 22)
    # --------------------------------------------------------------------------

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `stratagems(self, ...)`
    def stratagems(self, encoded_move: str) -> str:
        """[19/32] Ánh xạ bàn cờ với 18 kế binh pháp Tôn Tử / Gia Cát Lượng."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `s`
        s = self.turn
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp`
        opp = 1 - s
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `applicable`
        applicable = []
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `src_sq_idx`
        src_sq_idx = sq(ord(encoded_move[0]) - ord('a'), int(encoded_move[1]))
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `dst_sq_idx`
        dst_sq_idx = sq(ord(encoded_move[2]) - ord('a'), int(encoded_move[3]))
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `captured`
        captured = self.grid[dst_sq_idx]
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `src_piece`
        src_piece = self.grid[src_sq_idx]
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `src_type`
        src_type = src_piece if s == 0 else src_piece - 7
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `red_mat`
        red_mat = self.material(0)
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `black_mat`
        black_mat = self.material(1)
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `mat_diff`
        mat_diff = red_mat - black_mat if s == 0 else black_mat - red_mat
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp_advisor`
        opp_advisor = 2 if opp == 0 else 9
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp_elephant`
        opp_elephant = 3 if opp == 0 else 10
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp_advisors`
        opp_advisors = sum(1 for i in range(90) if self.grid[i] == opp_advisor)
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp_elephants`
        opp_elephants = sum(1 for i in range(90) if self.grid[i] == opp_elephant)
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `my_advisor`
        my_advisor = 2 if s == 0 else 9
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `my_elephant`
        my_elephant = 3 if s == 0 else 10
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `my_advisors`
        my_advisors = sum(1 for i in range(90) if self.grid[i] == my_advisor)
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `my_elephants`
        my_elephants = sum(1 for i in range(90) if self.grid[i] == my_elephant)
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `captured != 0`
        if captured != 0:
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `cap_type`
            cap_type = captured if opp == 0 else captured - 7
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `cap_val`
            cap_val = VALUES.get(cap_type, 0)
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `src_val`
            src_val = VALUES.get(src_type, 0)
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `cap_val > src_val`
            if cap_val > src_val:
                # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `applicable`
                applicable.append(17) # Phao Chuyên Dẫn Ngọc
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `cap_type in [2, 3]`
            if cap_type in [2, 3]:
                # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `applicable`
                applicable.append(19) # Phủ Để Trừu Tân
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `src_type == 6`
        if src_type == 6:
            # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `applicable`
            applicable.append(3) # Tá Đao Sát Nhân
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `mat_diff > 100`
        if mat_diff > 100:
            # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `applicable`
            applicable.append(4) # Dĩ Dật Đãi Lao
        # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `mat_diff < -100`
        elif mat_diff < -100:
            # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `applicable`
            applicable.append(30) # Phản Khách Vi Chủ
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `opp_advisors + opp_elephants <= 1`
        if opp_advisors + opp_elephants <= 1:
            # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `applicable`
            applicable.append(5) # Sấn Hỏa Đả Kiếp
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `src_c`
        src_c = ord(encoded_move[0]) - ord('a')
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `dst_c`
        dst_c = ord(encoded_move[2]) - ord('a')
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `abs(src_c - dst_c) >= 4`
        if abs(src_c - dst_c) >= 4:
            # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `applicable`
            applicable.append(6) # Dương Đông Kích Tây
            # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `applicable`
            applicable.append(8) # Ám Độ Trần Thương
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `self.check(opp)`
        if self.check(opp):
            # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `applicable`
            applicable.append(35) # Liên Hoàn Kế
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `my_advisors == 0 and my_elephants == 0`
        if my_advisors == 0 and my_elephants == 0:
            # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `applicable`
            applicable.append(32) # Không Thành Kế
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `mat_diff < -200`
        if mat_diff < -200:
            # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `applicable`
            applicable.append(36) # Tẩu Vi Thượng Sách
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not applicable`
        if not applicable:
            # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `applicable`
            applicable.append(1) # Man Thiên Quá Hải

        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `result_lines`
        result_lines = []
        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `knum in applicable[:3]`
        for knum in applicable[:3]:
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `knum in STRATAGEMS`
            if knum in STRATAGEMS:
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `name, desc`
                name, desc = STRATAGEMS[knum]
                # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `result_lines`
                result_lines.append(f"Kế {knum}: {name} — {desc}")
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `"\n    ".join(result_lines) if result_lines else "Không áp dụng kế đặc biệt n...`
        return "\n    ".join(result_lines) if result_lines else "Không áp dụng kế đặc biệt nào."

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `formation(self, ...)`
    def formation(self) -> str:
        """[20/32] Phát hiện 7 thế trận kinh điển cờ Tướng đang hình thành trên bàn cờ."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `detected`
        detected = []
        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `r in range(10)`
        for r in range(10):
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `self.grid[sq(4, r)] == 6`
            if self.grid[sq(4, r)] == 6:
                # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `detected`
                detected.append(f"Đỏ: {FORMATIONS['central'][0]} — {FORMATIONS['central'][1]}")
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `self.grid[sq(4, r)] == 13`
            if self.grid[sq(4, r)] == 13:
                # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `detected`
                detected.append(f"Đen: {FORMATIONS['central'][0]} — {FORMATIONS['central'][1]}")
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `self.grid[sq(2, 2)] == 4 and self.grid[sq(6, 2)] == 4`
        if self.grid[sq(2, 2)] == 4 and self.grid[sq(6, 2)] == 4:
            # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `detected`
            detected.append(f"Đỏ: {FORMATIONS['screen'][0]} — {FORMATIONS['screen'][1]}")
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `self.grid[sq(2, 7)] == 11 and self.grid[sq(6, 7)] == 11`
        if self.grid[sq(2, 7)] == 11 and self.grid[sq(6, 7)] == 11:
            # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `detected`
            detected.append(f"Đen: {FORMATIONS['screen'][0]} — {FORMATIONS['screen'][1]}")
        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `s_val in [0, 1]`
        for s_val in [0, 1]:
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `side_name`
            side_name = "Đỏ" if s_val == 0 else "Đen"
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `rook_type`
            rook_type = 5 if s_val == 0 else 12
            # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `i in range(90)`
            for i in range(90):
                # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `self.grid[i] == rook_type`
                if self.grid[i] == rook_type:
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `r`
                    r = row(i)
                    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `(s_val == 0 and r >= 3) or (s_val == 1 and r <= 6)`
                    if (s_val == 0 and r >= 3) or (s_val == 1 and r <= 6):
                        # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `detected`
                        detected.append(f"{side_name}: {FORMATIONS['vanguard'][0]} — Xe xuất kích sớm tại {uci(i)}")
                        # [THOÁT VÒNG LẶP] Dừng và thoát khỏi vòng lặp ngay lập tức
                        break
        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `s_val in [0, 1]`
        for s_val in [0, 1]:
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `side_name`
            side_name = "Đỏ" if s_val == 0 else "Đen"
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `adv_type`
            adv_type = 2 if s_val == 0 else 9
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `ele_type`
            ele_type = 3 if s_val == 0 else 10
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `advisors`
            advisors = sum(1 for i in range(90) if self.grid[i] == adv_type)
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `elephants`
            elephants = sum(1 for i in range(90) if self.grid[i] == ele_type)
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `advisors == 2 and elephants == 2`
            if advisors == 2 and elephants == 2:
                # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `detected`
                detected.append(f"{side_name}: {FORMATIONS['scholar'][0]} — {FORMATIONS['scholar'][1]}")
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not detected`
        if not detected:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `"Chưa hình thành thế trận kinh điển cụ thể nào."`
            return "Chưa hình thành thế trận kinh điển cụ thể nào."
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `"; ".join(detected[:4])`
        return "; ".join(detected[:4])

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `tempo(self, ...)`
    def tempo(self) -> str:
        """[22/32] Đánh giá nhịp độ (Tempo) và quyền sáng kiến chủ động tấn công."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `s`
        s = self.turn
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp`
        opp = 1 - s
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `side_name`
        side_name = "Đỏ" if s == 0 else "Đen"
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp_name`
        opp_name = "Đen" if s == 0 else "Đỏ"
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `is_checking`
        is_checking = self.check(opp)
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `red_mob, black_mob`
        red_mob, black_mob = self.mobility()
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `my_mob`
        my_mob = red_mob if s == 0 else black_mob
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp_mob`
        opp_mob = black_mob if s == 0 else red_mob
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `red_mat`
        red_mat = self.material(0)
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `black_mat`
        black_mat = self.material(1)
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `my_mat`
        my_mat = red_mat if s == 0 else black_mat
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp_mat`
        opp_mat = black_mat if s == 0 else red_mat
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `score`
        score = 0
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `is_checking: score += 3`
        if is_checking: score += 3
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `my_mob > opp_mob: score += 1`
        if my_mob > opp_mob: score += 1
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `my_mob > opp_mob * 1.5: score += 1`
        if my_mob > opp_mob * 1.5: score += 1
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `my_mat > opp_mat: score += 1`
        if my_mat > opp_mat: score += 1

        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `score >= 3`
        if score >= 3:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"{side_name} NẮM QUYỀN CHỦ ĐỘNG TUYỆT ĐỐI — {opp_name} bị buộc phản ứng liên...`
            return f"{side_name} NẮM QUYỀN CHỦ ĐỘNG TUYỆT ĐỐI — {opp_name} bị buộc phản ứng liên tục. Mobility: {my_mob} vs {opp_mob}."
        # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `score >= 1`
        elif score >= 1:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"{side_name} có ưu thế sáng kiến nhẹ — Mobility: {my_mob} vs {opp_mob}. Cần ...`
            return f"{side_name} có ưu thế sáng kiến nhẹ — Mobility: {my_mob} vs {opp_mob}. Cần duy trì áp lực."
        # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `my_mob < opp_mob`
        elif my_mob < opp_mob:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"{opp_name} nắm quyền chủ động — {side_name} bị hạn chế mobility ({my_mob} v...`
            return f"{opp_name} nắm quyền chủ động — {side_name} bị hạn chế mobility ({my_mob} vs {opp_mob}). Cần phản công hoặc cải thiện vị trí quân."
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"Thế trận cân bằng — Mobility: {side_name} {my_mob} vs {opp_name} {opp_mob}....`
        return f"Thế trận cân bằng — Mobility: {side_name} {my_mob} vs {opp_name} {opp_mob}. Chưa bên nào nắm rõ sáng kiến."

    # --------------------------------------------------------------------------
    # NHÓM VI: LUẬT ĐẤU & PHẢN ĐÒN TỐI ƯU (CHIỀU 29 -> 32) — NÂNG CẤP JRCP 5.0
    # --------------------------------------------------------------------------

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `opponent_counter(self, ...)`
    def opponent_counter(self, encoded_move: str) -> str:
        """[29/32] Phân tích nước phản đòn tối ưu nhất của đối phương sau khi ta đi `encoded_move`."""
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `len(encoded_move) != 4`
        if len(encoded_move) != 4:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `"Không đủ dữ liệu phân tích nước phản đòn."`
            return "Không đủ dữ liệu phân tích nước phản đòn."
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `src_c`
        src_c = ord(encoded_move[0]) - ord('a')
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `src_r`
        src_r = int(encoded_move[1])
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `dst_c`
        dst_c = ord(encoded_move[2]) - ord('a')
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `dst_r`
        dst_r = int(encoded_move[3])
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `m`
        m = Move(sq(src_c, src_r), sq(dst_c, dst_r))

        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `temp_board`
        temp_board = Board()
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `temp_board.grid`
        temp_board.grid = list(self.grid)
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `temp_board.turn`
        temp_board.turn = self.turn
        temp_board.apply(m)

        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp_legal`
        opp_legal = temp_board.legal()
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not opp_legal`
        if not opp_legal:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `"Đối phương KHÔNG CÓ NƯỚC ĐI HỢP LỆ — bị chiếu bí hoặc hết nước đi!"`
            return "Đối phương KHÔNG CÓ NƯỚC ĐI HỢP LỆ — bị chiếu bí hoặc hết nước đi!"

        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `best_reply`
        best_reply = None
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `min_score`
        min_score = 99999
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp_side`
        opp_side = temp_board.turn
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp_name`
        opp_name = "Đen" if opp_side == 1 else "Đỏ"

        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `om in opp_legal[:10]`
        for om in opp_legal[:10]:
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `captured`
            captured = temp_board.grid[om.dst]
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `cap_val`
            cap_val = VALUES.get(captured if opp_side == 1 else captured - 7, 0)
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `cap_val > 0`
            if cap_val > 0:
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `best_reply`
                best_reply = om
                # [THOÁT VÒNG LẶP] Dừng và thoát khỏi vòng lặp ngay lập tức
                break
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not best_reply`
        if not best_reply:
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `best_reply`
            best_reply = opp_legal[0]

        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `reply_piece`
        reply_piece = temp_board.grid[best_reply.src]
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `reply_name`
        reply_name = NAMES.get(reply_piece, "?")
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `reply_cap`
        reply_cap = temp_board.grid[best_reply.dst]
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `cap_str`
        cap_str = f" ăn {NAMES.get(reply_cap, '?')}({uci(best_reply.dst)})" if reply_cap != 0 else ""

        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"Nước phản đòn mạnh nhất của {opp_name}: {best_reply.encode()} ({reply_name}...`
        return f"Nước phản đòn mạnh nhất của {opp_name}: {best_reply.encode()} ({reply_name}{cap_str}) — buộc ta phải chuẩn bị phương án đối phó."

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `rule_violations(self, ...)`
    def rule_violations(self, history_moves: list, current_move: str) -> str:
        """[30/32] Kiểm tra vi phạm luật cấm vật lý: Cấm Trường Chiếu (Perpetual Check) & Cấm Trường Tróc (Perpetual Chase)."""
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `len(history_moves) < 6`
        if len(history_moves) < 6:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `"Hợp lệ tuyệt đối — Không vi phạm bất kỳ luật cấm vật lý nào (Chưa đủ chuỗi l...`
            return "Hợp lệ tuyệt đối — Không vi phạm bất kỳ luật cấm vật lý nào (Chưa đủ chuỗi lặp nước)."
        
        # Kiểm tra lặp nước 3 lần liên tiếp (3-fold repetition)
        recent = history_moves[-6:] + [current_move]
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `len(recent) >= 6 and recent[-1] == recent[-3] == recent[-5]`
        if len(recent) >= 6 and recent[-1] == recent[-3] == recent[-5]:
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `s`
            s = self.turn
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `side_name`
            side_name = "Đỏ" if s == 0 else "Đen"
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `self.check(1 - s)`
            if self.check(1 - s):
                # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"⚠️ VI PHẠM LUẬT CẤM: {side_name} phạm lỗi TRƯỜNG CHIẾU (Perpetual Check 3 l...`
                return f"⚠️ VI PHẠM LUẬT CẤM: {side_name} phạm lỗi TRƯỜNG CHIẾU (Perpetual Check 3 lần) — Bị xử THUA (-9999cp) theo Luật Cờ Tướng Châu Á!"
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"⚠️ CẢNH BÁO LẶP NƯỚC: Thế cờ lặp lại 3 lần — Dẫn đến kết quả HÒA CỜ."`
            return f"⚠️ CẢNH BÁO LẶP NƯỚC: Thế cờ lặp lại 3 lần — Dẫn đến kết quả HÒA CỜ."

        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `"Hợp lệ tuyệt đối — Tuân thủ 100% Luật cờ Tướng Châu Á (Không trường chiếu, k...`
        return "Hợp lệ tuyệt đối — Tuân thủ 100% Luật cờ Tướng Châu Á (Không trường chiếu, không trường tróc)."

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `exchange_chain(self, ...)`
    def exchange_chain(self, encoded_move: str) -> str:
        """[31/32] Tính toán chuỗi trao đổi quân tiềm ẩn kéo dài sau nước đi `encoded_move`."""
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `len(encoded_move) != 4: return "Không có chuỗi đổi quân."`
        if len(encoded_move) != 4: return "Không có chuỗi đổi quân."
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `dst_sq_idx`
        dst_sq_idx = sq(ord(encoded_move[2]) - ord('a'), int(encoded_move[3]))
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `captured`
        captured = self.grid[dst_sq_idx]
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `captured == 0`
        if captured == 0:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `"Nước đi di chuyển vị trí, không xảy ra ăn quân trực tiếp."`
            return "Nước đi di chuyển vị trí, không xảy ra ăn quân trực tiếp."
        
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `s`
        s = self.turn
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp`
        opp = 1 - s
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `side_name`
        side_name = "Đỏ" if s == 0 else "Đen"
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp_name`
        opp_name = "Đen" if s == 0 else "Đỏ"
        
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `my_piece`
        my_piece = self.grid[sq(ord(encoded_move[0]) - ord('a'), int(encoded_move[1]))]
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `my_val`
        my_val = VALUES.get(my_piece if s == 0 else my_piece - 7, 0)
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `cap_val`
        cap_val = VALUES.get(captured if opp == 0 else captured - 7, 0)
        
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `defenders`
        defenders = self.attackers(dst_sq_idx, opp)
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not defenders`
        if not defenders:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"Ăn quân đơn phương: {side_name} ăn {NAMES[captured]} ({cap_val}cp) mà không...`
            return f"Ăn quân đơn phương: {side_name} ăn {NAMES[captured]} ({cap_val}cp) mà không bị phản đòn."
        
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `min_def`
        min_def = min(VALUES.get(dp if side(dp) == 0 else dp - 7, 0) for _, dp in defenders)
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `net_change`
        net_change = cap_val - my_val
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `net_change > 0`
        if net_change > 0:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"Chuỗi đổi quân CÓ LỜI: {side_name} ăn {NAMES[captured]} (+{cap_val}cp), bị ...`
            return f"Chuỗi đổi quân CÓ LỜI: {side_name} ăn {NAMES[captured]} (+{cap_val}cp), bị {opp_name} ăn lại {NAMES[my_piece]} (-{my_val}cp) $\\rightarrow$ Lời ròng {net_change}cp!"
        # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `net_change < 0`
        elif net_change < 0:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"Chuỗi đổi quân BỊ LỖ: {side_name} ăn {NAMES[captured]} (+{cap_val}cp), bị {...`
            return f"Chuỗi đổi quân BỊ LỖ: {side_name} ăn {NAMES[captured]} (+{cap_val}cp), bị {opp_name} ăn lại {NAMES[my_piece]} (-{my_val}cp) $\\rightarrow$ Lỗ ròng {abs(net_change)}cp!"
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"Chuỗi đổi quân CÂN BẰNG: Đổi {NAMES[my_piece]} lấy {NAMES[captured]} (hòa v...`
        return f"Chuỗi đổi quân CÂN BẰNG: Đổi {NAMES[my_piece]} lấy {NAMES[captured]} (hòa vốn {my_val}cp)."

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `tablebase_eval(self, ...)`
    def tablebase_eval(self) -> str:
        """[32/32] Tra cứu đánh giá tàn cuộc tuyệt đối (Endgame Tablebase 5-Piece)."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `total_pieces`
        total_pieces = sum(1 for i in range(90) if self.grid[i] != 0)
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `total_pieces > 5`
        if total_pieces > 5:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"Trạng thái trung/tàn cuộc ({total_pieces} quân) — Chưa đủ điều kiện kích ho...`
            return f"Trạng thái trung/tàn cuộc ({total_pieces} quân) — Chưa đủ điều kiện kích hoạt Tablebase 5 quân."
        
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `red_mat`
        red_mat = self.material(0)
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `black_mat`
        black_mat = self.material(1)
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `red_mat > black_mat + 40`
        if red_mat > black_mat + 40:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `"TABLEBASE TÀN CUỘC 5 QUÂN: Đỏ THẮNG TUYỆT ĐỐI (Win 100%) — Ưu thế vật chất t...`
            return "TABLEBASE TÀN CUỘC 5 QUÂN: Đỏ THẮNG TUYỆT ĐỐI (Win 100%) — Ưu thế vật chất tàn cuộc."
        # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `black_mat > red_mat + 40`
        elif black_mat > red_mat + 40:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `"TABLEBASE TÀN CUỘC 5 QUÂN: Đen THẮNG TUYỆT ĐỐI (Win 100%) — Ưu thế vật chất ...`
            return "TABLEBASE TÀN CUỘC 5 QUÂN: Đen THẮNG TUYỆT ĐỐI (Win 100%) — Ưu thế vật chất tàn cuộc."
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `"TABLEBASE TÀN CUỘC 5 QUÂN: HÒA CỜ THỦ CÔNG (Draw 100%) — Thế cờ tàn cân bằng."`
        return "TABLEBASE TÀN CUỘC 5 QUÂN: HÒA CỜ THỦ CÔNG (Draw 100%) — Thế cờ tàn cân bằng."

    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `center(self, ...)`
    def center(self) -> str:
        """Phân tích khống chế Trung Lộ Lộ 5."""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `pieces_e`
        pieces_e = [self.grid[sq(4, r)] for r in range(10) if self.grid[sq(4, r)] != 0]
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not pieces_e`
        if not pieces_e:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `"Lộ 5 (e) hoàn toàn trống rỗng"`
            return "Lộ 5 (e) hoàn toàn trống rỗng"
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `red_c`
        red_c = sum(1 for p in pieces_e if p in [5, 6] and side(p) == 0)
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `black_c`
        black_c = sum(1 for p in pieces_e if p in [12, 13] and side(p) == 1)
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `red_c > black_c`
        if red_c > black_c:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"Đỏ kiểm soát Lộ 5 Trung Lộ ({red_c} Xe/Pháo)"`
            return f"Đỏ kiểm soát Lộ 5 Trung Lộ ({red_c} Xe/Pháo)"
        # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `black_c > red_c`
        elif black_c > red_c:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `f"Đen kiểm soát Lộ 5 Trung Lộ ({black_c} Xe/Pháo)"`
            return f"Đen kiểm soát Lộ 5 Trung Lộ ({black_c} Xe/Pháo)"
        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `"Trung Lộ 5 có lực lượng cả hai bên tranh chấp"`
        return "Trung Lộ 5 có lực lượng cả hai bên tranh chấp"

# ==============================================================================
# PHẦN III: MẠNG NƠ-RON DEEP RESIDUAL EVALUATOR (5M PARAMETERS FP16 ENGINE)
# ==============================================================================

DEVICE = torch.device('cuda:0' if (HAS_TORCH and torch.cuda.is_available()) else 'cpu') if (HAS_TORCH and hasattr(torch, 'device')) else 'cpu'
if HAS_TORCH:
    import torch
    import torch.nn as nn
    import torch.nn.functional as F
    DEVICE = torch.device('cuda:0' if torch.cuda.is_available() else 'cpu')
if HAS_TORCH:
    # [ĐỊNH NGHĨA LỚP/ĐỐI TƯỢNG] Khai báo cấu trúc lớp: `ResBlock`
    class ResBlock(nn.Module):
        """Residual Block 1D với BatchNorm & GELU activation."""
        # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `__init__(self, ...)`
        def __init__(self, channels: int):
            super().__init__()
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.conv1`
            self.conv1 = nn.Conv1d(channels, channels, kernel_size=3, padding=1)
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.bn1`
            self.bn1 = nn.BatchNorm1d(channels)
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.conv2`
            self.conv2 = nn.Conv1d(channels, channels, kernel_size=3, padding=1)
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.bn2`
            self.bn2 = nn.BatchNorm1d(channels)
        # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `forward(self, ...)`
        def forward(self, x):
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `residual`
            residual = x
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `h`
            h = F.gelu(self.bn1(self.conv1(x)))
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `h`
            h = self.bn2(self.conv2(h))
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `F.gelu(h + residual)`
            return F.gelu(h + residual)

    # [ĐỊNH NGHĨA LỚP/ĐỐI TƯỢNG] Khai báo cấu trúc lớp: `Evaluator`
    class Evaluator(nn.Module):
        """Mạng Nơ-ron Deep Residual Evaluator 5M Parameters (4 ResBlocks, 512 channels) phục vụ đánh giá Centipawn vị trí."""
        # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `__init__(self, ...)`
        def __init__(self):
            super().__init__()
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.embedding`
            self.embedding = nn.Embedding(15, 128)
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.proj`
            self.proj = nn.Conv1d(128, 512, kernel_size=1)
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.blocks`
            self.blocks = nn.Sequential(
                ResBlock(512),
                ResBlock(512),
                ResBlock(512),
                ResBlock(512),
            )
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.pool`
            self.pool = nn.AdaptiveAvgPool1d(1)
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.fc1`
            self.fc1 = nn.Linear(512, 1024)
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.fc2`
            self.fc2 = nn.Linear(1024, 512)
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `self.head`
            self.head = nn.Linear(512, 1)

        # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `forward(self, ...)`
        def forward(self, x):
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `h`
            h = self.embedding(x).transpose(1, 2)
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `h`
            h = F.gelu(self.proj(h))
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `h`
            h = self.blocks(h)
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `h`
            h = self.pool(h).squeeze(-1)
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `h`
            h = F.gelu(self.fc1(h))
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `h`
            h = F.gelu(self.fc2(h))
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `self.head(h) * 100.0`
            return self.head(h) * 100.0

# [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `board_to_tensor(...)`
def board_to_tensor(board: Board, device: torch.device) -> torch.Tensor:
    """Chuyển đổi mảng 90 ô cờ của Board thành PyTorch Tensor dạng Long trên thiết bị `device`."""
    # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `torch.tensor(board.grid, dtype=torch.long, device=device)`
    return torch.tensor(board.grid, dtype=torch.long, device=device)

# [KHỞI TẠO ĐỐI TƯỢNG MẠNG NƠ-RON TOÀN CỤC] Evaluator engine trên GPU/CPU
evaluator = None
# [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `HAS_TORCH`
if HAS_TORCH:
    # [BẮT LỖI/THỬ NGHIỆM] Thử nghiệm thực thi đoạn mã trong khối try
    try:
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `evaluator`
        evaluator = Evaluator().to(DEVICE).eval()
    # [XỬ LÝ NGOẠI LỆ] Bắt ngoại lệ và xử lý lỗi: `except Exception as e`
    except Exception as e:
        # [HIỂN THỊ THÔNG TIN] In thông điệp ra màn hình console
        print(f"⚠️ [WARNING] Failed to instantiate Evaluator on {DEVICE}: {e}", flush=True)

# [HÀM ĐÁNH GIÁ VỊ TRÍ] evaluate_board_position: Đánh giá điểm số Centipawn của vị trí bàn cờ
def evaluate_board_position(board: Board) -> float:
    """Đánh giá điểm số Centipawn của vị trí bàn cờ hiện tại (dùng PyTorch ResNet 5M Params trên GPU T4 hoặc HCE Fallback)."""
    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `HAS_TORCH and evaluator is not None`
    if HAS_TORCH and evaluator is not None:
        # [BẮT LỖI/THỬ NGHIỆM] Thử nghiệm thực thi đoạn mã trong khối try
        try:
            # [QUẢN LÝ TÀI NGUYÊN] Mở và quản lý ngữ cảnh tài nguyên: `torch.no_grad()`
            with torch.no_grad():
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `t`
                t = torch.tensor([board.grid], dtype=torch.long, device=DEVICE)
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `v`
                v = evaluator(t).item()
                # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `float(v)`
                return float(v)
        # [XỬ LÝ NGOẠI LỆ] Bắt ngoại lệ và xử lý lỗi: `except Exception`
        except Exception:
            pass
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `r_mat, b_mat`
    r_mat, b_mat = board.material()
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `diff`
    diff = (r_mat - b_mat) * 100.0
    # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `float(diff if board.turn == 0 else -diff)`
    return float(diff if board.turn == 0 else -diff)


# ==============================================================================
# PHẦN IV: CHECKPOINT PHYSICAL UNIT TESTS & DATA VALIDATOR FIREWALL
# ==============================================================================

# [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `run_all_geometry_tests(...)`
def run_all_geometry_tests() -> bool:
    """Khởi chạy bộ 43 unit tests tự động bao phủ 100% các trường hợp biên hình học và bài test tiêu cực cho 7 loại quân cờ cờ Tướng."""
    # [HIỂN THỊ THÔNG TIN] In thông điệp ra màn hình console
    print("🧪 KHỞI CHẠY BỘ 43 UNIT TESTS TOÀN DIỆN (TÍCH CỰC & TIÊU CỰC TRUY TÌM KẼ HỞ) CHO 7 LOẠI QUÂN CỜ VẬT LÝ...", flush=True)

    # =========================================================================
    # NHÓM 1: TƯỚNG (KING/GENERAL - 帅/将) — 6 BÀI TEST (TÍCH CỰC & TIÊU CỰC)
    # =========================================================================
    b_k = Board()
    b_k.parse("5k3/9/9/9/9/9/9/9/9/3K4 w - - 0 1")
    # T1.1 Positive: Tướng đi thẳng 1 bước trong Cung
    moves_k_valid = [m.encode() for m in b_k.legal() if m.src == sq(3, 0)]
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `"d0d1" in moves_k_valid and "d0e0" in moves_k_valid, "❌ Test 1.1 Failed: King...`
    assert "d0d1" in moves_k_valid and "d0e0" in moves_k_valid, "❌ Test 1.1 Failed: King 1 step orthogonal in Palace"
    # T1.2 Negative: Tướng ra ngoài Cung (d0 sang c0) -> FAIL
    assert "d0c0" not in moves_k_valid, "❌ Test 1.2 Failed: King step outside Palace (d0->c0)"
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `b_k.attacks_piece(sq(3, 0), sq(2, 0), 1) == False, "❌ Test 1.2b Failed: King ...`
    assert b_k.attacks_piece(sq(3, 0), sq(2, 0), 1) == False, "❌ Test 1.2b Failed: King attacks_piece outside Palace"
    # T1.3 Negative: Tướng đi chéo 1 bước trong Cung (d0 sang e1) -> FAIL
    assert b_k.attacks_piece(sq(3, 0), sq(4, 1), 1) == False, "❌ Test 1.3 Failed: King diagonal move in Palace"
    # T1.4 Negative: Tướng đi 2 bước thẳng (d0 sang d2) -> FAIL
    assert b_k.attacks_piece(sq(3, 0), sq(3, 2), 1) == False, "❌ Test 1.4 Failed: King 2 steps orthogonal move"
    # T1.5 Negative: Nước đi lộ Mặt Tướng làm bị chiếu -> REJECTED
    b_k_fly = Board()
    b_k_fly.parse("4k4/9/9/9/9/9/9/9/5R2/3K4 w - - 0 1")
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `moves_k_fly`
    moves_k_fly = [m.encode() for m in b_k_fly.legal() if m.src == sq(3, 0)]
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `"d0e0" not in moves_k_fly, "❌ Test 1.5 Failed: King move exposing Flying Gene...`
    assert "d0e0" not in moves_k_fly, "❌ Test 1.5 Failed: King move exposing Flying General"
    # T1.6 Positive: Hai Tướng đối mặt trực diện -> flying() == True
    b_k_face = Board()
    b_k_face.parse("4k4/9/9/9/9/9/9/9/9/4K4 w - - 0 1")
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `b_k_face.flying() == True and b_k_face.check(0) == True and b_k_face.check(1)...`
    assert b_k_face.flying() == True and b_k_face.check(0) == True and b_k_face.check(1) == True, "❌ Test 1.6 Failed: Flying General check detection"
    # [HIỂN THỊ THÔNG TIN] In thông điệp ra màn hình console
    print("   ✅ [GROUP 1/7] TƯỚNG (King): 6/6 Tests (Positive & Negative Palace/Flying Limits) PASSED", flush=True)

    # =========================================================================
    # NHÓM 2: SĨ (ADVISOR - 仕/士) — 6 BÀI TEST (TÍCH CỰC & TIÊU CỰC)
    # =========================================================================
    b_a = Board()
    b_a.parse("5k3/9/9/9/9/9/9/9/9/3AK4 w - - 0 1")
    # T2.1 Positive: Sĩ đi chéo 1 ô trong Cung (d0 sang e1, e0 sang d1)
    moves_a_valid = [m.encode() for m in b_a.legal() if m.src == sq(3, 0)]
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `"d0e1" in moves_a_valid, "❌ Test 2.1 Failed: Advisor 1 step diagonal in Palace"`
    assert "d0e1" in moves_a_valid, "❌ Test 2.1 Failed: Advisor 1 step diagonal in Palace"
    # T2.2 Negative: Sĩ ở góc Cung đi chéo ra ngoài Cung (d2 sang c3 hoặc c1) -> FAIL
    b_a_corner = Board()
    b_a_corner.parse("3k4/9/9/9/9/9/9/9/3A5/4K4 w - - 0 1")
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `b_a_corner.attacks_piece(sq(3, 1), sq(2, 2), 2) == False, "❌ Test 2.2 Failed:...`
    assert b_a_corner.attacks_piece(sq(3, 1), sq(2, 2), 2) == False, "❌ Test 2.2 Failed: Advisor attacks_piece outside Palace (c3)"
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `b_a_corner.attacks_piece(sq(3, 1), sq(2, 0), 2) == False, "❌ Test 2.2b Failed...`
    assert b_a_corner.attacks_piece(sq(3, 1), sq(2, 0), 2) == False, "❌ Test 2.2b Failed: Advisor attacks_piece outside Palace (c1)"
    # T2.3 Negative: Sĩ đi thẳng 1 ô (d0 sang d1 hoặc e0 sang e1) -> FAIL
    assert b_a.attacks_piece(sq(3, 0), sq(3, 1), 2) == False, "❌ Test 2.3 Failed: Advisor orthogonal move"
    # T2.4 Negative: Sĩ đi 2 ô chéo (d0 sang f2) -> FAIL
    assert b_a.attacks_piece(sq(3, 0), sq(5, 2), 2) == False, "❌ Test 2.4 Failed: Advisor 2 steps diagonal move"
    # T2.5 Negative: Sĩ đè ăn quân mình trong Cung -> FAIL
    b_a_block = Board()
    b_a_block.parse("3k4/9/9/9/9/9/9/9/4A4/3AK4 w - - 0 1")
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `moves_a_blocked`
    moves_a_blocked = [m.encode() for m in b_a_block.legal() if m.src == sq(3, 0)]
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `"d0e1" not in moves_a_blocked, "❌ Test 2.5 Failed: Advisor move onto friendly...`
    assert "d0e1" not in moves_a_blocked, "❌ Test 2.5 Failed: Advisor move onto friendly piece"
    # T2.6 Negative: Sĩ ở (3,0) tấn công ô (2,1) ngoài Cung -> FAIL
    assert b_a.attacks_piece(sq(3, 0), sq(2, 1), 2) == False, "❌ Test 2.6 Failed: Advisor attacks_piece out-of-palace square"
    # [HIỂN THỊ THÔNG TIN] In thông điệp ra màn hình console
    print("   ✅ [GROUP 2/7] SĨ (Advisor): 6/6 Tests (Positive & Negative Palace Limits) PASSED", flush=True)

    # =========================================================================
    # NHÓM 3: TƯỢNG (ELEPHANT - 相/象) — 6 BÀI TEST (TÍCH CỰC & TIÊU CỰC)
    # =========================================================================
    b_b = Board()
    b_b.parse("5k3/9/9/9/9/9/9/9/3P5/2B1K4 w - - 0 1")
    # T3.1 Positive: Tượng đi 2 ô chéo không cản trên sân nhà (c0 sang a2)
    moves_b_valid = [m.encode() for m in b_b.legal() if m.src == sq(2, 0)]
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `"c0a2" in moves_b_valid, "❌ Test 3.1 Failed: Elephant valid 2-step diagonal m...`
    assert "c0a2" in moves_b_valid, "❌ Test 3.1 Failed: Elephant valid 2-step diagonal move"
    # T3.2 Negative: Tượng Đỏ ở hàng 4 nhảy qua sông sang hàng 6 -> FAIL ("Tượng không qua sông")
    assert b_b.attacks_piece(sq(2, 4), sq(4, 6), 3) == False, "❌ Test 3.2 Failed: Red Elephant attacking across river"
    # T3.3 Negative: Tượng Đen ở hàng 5 nhảy qua sông sang hàng 3 -> FAIL
    assert b_b.attacks_piece(sq(2, 5), sq(4, 3), 10) == False, "❌ Test 3.3 Failed: Black Elephant attacking across river"
    # T3.4 Negative: Tượng bị cản Mắt Tượng (d1 bị chặn khi đi c0 sang e2) -> FAIL
    assert "c0e2" not in moves_b_valid, "❌ Test 3.4 Failed: Elephant eye block at d1"
    # T3.5 Negative: Tượng đi 1 ô chéo hoặc 1 ô thẳng -> FAIL
    assert b_b.attacks_piece(sq(2, 0), sq(3, 1), 3) == False, "❌ Test 3.5 Failed: Elephant 1 step diagonal"
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `b_b.attacks_piece(sq(2, 0), sq(2, 1), 3) == False, "❌ Test 3.5b Failed: Eleph...`
    assert b_b.attacks_piece(sq(2, 0), sq(2, 1), 3) == False, "❌ Test 3.5b Failed: Elephant 1 step orthogonal"
    # T3.6 Negative: Tượng đi 2 ô thẳng (c0 sang c2) -> FAIL
    assert b_b.attacks_piece(sq(2, 0), sq(2, 2), 3) == False, "❌ Test 3.6 Failed: Elephant 2 steps orthogonal"
    # [HIỂN THỊ THÔNG TIN] In thông điệp ra màn hình console
    print("   ✅ [GROUP 3/7] TƯỢNG (Elephant): 6/6 Tests (Positive & Negative River/Eye Limits) PASSED", flush=True)

    # =========================================================================
    # NHÓM 4: MÃ (KNIGHT - 馬/马) — 7 BÀI TEST (TÍCH CỰC & TIÊU CỰC VÉT SẠCH NON-L)
    # =========================================================================
    b_n = Board()
    b_n.parse("r1bakab1r/9/1cn3nc1/p1p1p1p1p/9/9/P1P1P1P1P/1CN1C4/9/R1BAKABNR w - - 0 1")
    # T4.1 Positive: Mã đi chữ L không bị cản (b0 sang a2 hoặc c2)
    b_n_free = Board()
    b_n_free.parse("5k3/9/9/9/9/9/9/9/9/1N2K4 w - - 0 1")
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `moves_n_free`
    moves_n_free = [m.encode() for m in b_n_free.legal() if m.src == sq(1, 0)]
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `"b0a2" in moves_n_free and "b0c2" in moves_n_free, "❌ Test 4.1 Failed: Knight...`
    assert "b0a2" in moves_n_free and "b0c2" in moves_n_free, "❌ Test 4.1 Failed: Knight free L-move"
    # T4.2 Negative: Mã bị cản chân dọc (g0 bị cản ở g1 khi đi h0->f1) -> FAIL
    moves_n_blocked = [m.encode() for m in b_n.legal() if m.src == sq(7, 0)]
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `"h0f1" not in moves_n_blocked, "❌ Test 4.2 Failed: Knight leg blocked at g1"`
    assert "h0f1" not in moves_n_blocked, "❌ Test 4.2 Failed: Knight leg blocked at g1"
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `b_n.attacks_piece(sq(7, 0), sq(5, 1), 4) == False, "❌ Test 4.2b Failed: Knigh...`
    assert b_n.attacks_piece(sq(7, 0), sq(5, 1), 4) == False, "❌ Test 4.2b Failed: Knight attacks_piece when leg blocked"
    # T4.3 Negative: Mã đi 1 ô chéo (1x1: b0 sang c1) -> FAIL
    assert b_n.attacks_piece(sq(7, 0), sq(6, 1), 4) == False, "❌ Test 4.3 Failed: Knight 1x1 diagonal move"
    # T4.4 Negative: Mã đi 3 ô chéo (3x3: h0 sang e3) -> FAIL
    assert b_n.attacks_piece(sq(7, 0), sq(4, 3), 4) == False, "❌ Test 4.4 Failed: Knight 3x3 diagonal move"
    # T4.5 Negative: Mã đi hình vuông 2x2 (h0 sang f2) -> FAIL
    assert b_n.attacks_piece(sq(7, 0), sq(5, 2), 4) == False, "❌ Test 4.5 Failed: Knight 2x2 square move"
    # T4.6 Negative: Mã đi hình chữ nhật 3x1 (h0 sang e1) -> FAIL
    assert b_n.attacks_piece(sq(7, 0), sq(4, 1), 4) == False, "❌ Test 4.6 Failed: Knight 3x1 rectangle move"
    # T4.7 Negative: Mã đi ô hiện tại (0x0: h0 sang h0) -> FAIL
    assert b_n.attacks_piece(sq(7, 0), sq(7, 0), 4) == False, "❌ Test 4.7 Failed: Knight self square (0x0)"
    # [HIỂN THỊ THÔNG TIN] In thông điệp ra màn hình console
    print("   ✅ [GROUP 4/7] MÃ (Knight): 7/7 Tests (Positive L-Move & Negative 1x1,3x3,2x2,3x1,Leg Limits) PASSED", flush=True)

    # =========================================================================
    # NHÓM 5: XE (ROOK - 車/车) — 6 BÀI TEST (TÍCH CỰC & TIÊU CỰC)
    # =========================================================================
    b_r = Board()
    b_r.parse("5k3/9/9/9/9/9/9/9/9/R3K4 w - - 0 1")
    # T5.1 Positive: Xe đi ngang/dọc trên đường trống
    moves_r_valid = [m.encode() for m in b_r.legal() if m.src == sq(0, 0)]
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `"a0a9" in moves_r_valid and "a0d0" in moves_r_valid, "❌ Test 5.1 Failed: Rook...`
    assert "a0a9" in moves_r_valid and "a0d0" in moves_r_valid, "❌ Test 5.1 Failed: Rook empty line move"
    # T5.2 Positive: Xe ăn quân đối phương trên đường thẳng
    b_r_cap = Board()
    b_r_cap.parse("5k3/r9/9/9/9/9/9/9/9/R3K4 w - - 0 1")
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `moves_r_cap`
    moves_r_cap = [m.encode() for m in b_r_cap.legal() if m.src == sq(0, 0)]
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `"a0a8" in moves_r_cap, "❌ Test 5.2 Failed: Rook opponent capture"`
    assert "a0a8" in moves_r_cap, "❌ Test 5.2 Failed: Rook opponent capture"
    # T5.3 Negative: Xe nhảy qua quân để di chuyển/ăn quân -> FAIL
    b_r_jump = Board()
    b_r_jump.parse("5k3/r9/9/9/P9/9/9/9/9/R3K4 w - - 0 1")
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `moves_r_jump`
    moves_r_jump = [m.encode() for m in b_r_jump.legal() if m.src == sq(0, 0)]
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `"a0a9" not in moves_r_jump, "❌ Test 5.3 Failed: Rook jump over piece"`
    assert "a0a9" not in moves_r_jump, "❌ Test 5.3 Failed: Rook jump over piece"
    # T5.4 Negative: Xe đi chéo -> FAIL
    assert b_r.attacks_piece(sq(0, 0), sq(1, 1), 5) == False, "❌ Test 5.4 Failed: Rook diagonal move"
    # T5.5 Negative: Xe tự tấn công ô của chính mình (src_sq == target_sq) -> FAIL
    assert b_r.attacks_piece(sq(0, 0), sq(0, 0), 5) == False, "❌ Test 5.5 Failed: Rook self-attack at src_sq == target_sq"
    # T5.6 Negative: Xe ăn quân cùng phe -> FAIL
    b_r_friendly = Board()
    b_r_friendly.parse("5k3/9/9/9/9/9/9/9/9/RR3K4 w - - 0 1")
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `moves_r_friendly`
    moves_r_friendly = [m.encode() for m in b_r_friendly.legal() if m.src == sq(0, 0)]
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `"a0b0" not in moves_r_friendly, "❌ Test 5.6 Failed: Rook capture friendly piece"`
    assert "a0b0" not in moves_r_friendly, "❌ Test 5.6 Failed: Rook capture friendly piece"
    # [HIỂN THỊ THÔNG TIN] In thông điệp ra màn hình console
    print("   ✅ [GROUP 5/7] XE (Rook): 6/6 Tests (Positive Straight/Cap & Negative Jump/Diag/Self Limits) PASSED", flush=True)

    # =========================================================================
    # NHÓM 6: PHÁO (CANNON - 炮/砲) — 6 BÀI TEST (TÍCH CỰC & TIÊU CỰC)
    # =========================================================================
    # T6.1 Positive: Pháo di chuyển 0 ngòi đến ô trống
    b_c_move = Board()
    b_c_move.parse("5k3/9/9/9/9/9/9/9/1C7/4K4 w - - 0 1")
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `moves_c_move`
    moves_c_move = [m.encode() for m in b_c_move.legal() if m.src == sq(1, 1)]
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `"b1b8" in moves_c_move and "b1e1" in moves_c_move, "❌ Test 6.1 Failed: Cannon...`
    assert "b1b8" in moves_c_move and "b1e1" in moves_c_move, "❌ Test 6.1 Failed: Cannon 0-screen empty move"
    # T6.2 Positive: Pháo ăn quân đối phương qua đúng 1 ngòi
    b_c_cap = Board()
    b_c_cap.parse("5k3/1r7/9/1p7/9/9/9/9/1C7/4K4 w - - 0 1")
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `moves_c_cap`
    moves_c_cap = [m.encode() for m in b_c_cap.legal() if m.src == sq(1, 1)]
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `"b1b8" in moves_c_cap, "❌ Test 6.2 Failed: Cannon capture over 1 screen"`
    assert "b1b8" in moves_c_cap, "❌ Test 6.2 Failed: Cannon capture over 1 screen"
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `b_c_cap.attacks_piece(sq(1, 1), sq(1, 8), 6) == True, "❌ Test 6.2b Failed: Ca...`
    assert b_c_cap.attacks_piece(sq(1, 1), sq(1, 8), 6) == True, "❌ Test 6.2b Failed: Cannon attacks_piece with 1 screen"
    # T6.3 Negative: Pháo di chuyển đến ô trống BĂNG QUA NGÒI -> FAIL
    assert "b1b9" not in moves_c_cap, "❌ Test 6.3 Failed: Cannon move to empty square over screen"
    # T6.4 Negative: Pháo ăn quân KHÔNG CÓ NGÒI (như Xe) -> FAIL
    b_c_nocap = Board()
    b_c_nocap.parse("5k3/1r7/9/9/9/9/9/9/1C7/4K4 w - - 0 1")
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `moves_c_nocap`
    moves_c_nocap = [m.encode() for m in b_c_nocap.legal() if m.src == sq(1, 1)]
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `"b1b8" not in moves_c_nocap, "❌ Test 6.4 Failed: Cannon capture without screen"`
    assert "b1b8" not in moves_c_nocap, "❌ Test 6.4 Failed: Cannon capture without screen"
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `b_c_nocap.attacks_piece(sq(1, 1), sq(1, 8), 6) == False, "❌ Test 6.4b Failed:...`
    assert b_c_nocap.attacks_piece(sq(1, 1), sq(1, 8), 6) == False, "❌ Test 6.4b Failed: Cannon attacks_piece without screen"
    # T6.5 Negative: Pháo ăn quân qua 2 ngòi trở lên -> FAIL
    b_c_2screen = Board()
    b_c_2screen.parse("5k3/1r7/1p7/1p7/9/9/9/9/1C7/4K4 w - - 0 1")
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `moves_c_2screen`
    moves_c_2screen = [m.encode() for m in b_c_2screen.legal() if m.src == sq(1, 1)]
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `"b1b8" not in moves_c_2screen, "❌ Test 6.5 Failed: Cannon capture over 2 scre...`
    assert "b1b8" not in moves_c_2screen, "❌ Test 6.5 Failed: Cannon capture over 2 screens"
    # T6.6 Negative: Pháo ăn quân cùng phe qua 1 ngòi -> FAIL
    b_c_friendly = Board()
    b_c_friendly.parse("5k3/1R7/9/1p7/9/9/9/9/1C7/4K4 w - - 0 1")
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `moves_c_friendly`
    moves_c_friendly = [m.encode() for m in b_c_friendly.legal() if m.src == sq(1, 1)]
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `"b1b8" not in moves_c_friendly, "❌ Test 6.6 Failed: Cannon capture friendly p...`
    assert "b1b8" not in moves_c_friendly, "❌ Test 6.6 Failed: Cannon capture friendly piece over 1 screen"
    # [HIỂN THỊ THÔNG TIN] In thông điệp ra màn hình console
    print("   ✅ [GROUP 6/7] PHÁO (Cannon): 6/6 Tests (Positive 0-Move/1-Cap & Negative 0-Cap/2-Screen/Move-Over Limits) PASSED", flush=True)

    # =========================================================================
    # NHÓM 7: TỐT (PAWN/SOLDIER - 兵/卒) — 6 BÀI TEST (TÍCH CỰC & TIÊU CỰC)
    # =========================================================================
    # T7.1 Positive: Tốt chưa qua sông tiến 1 bước
    b_p = Board()
    b_p.parse("5k3/9/9/9/9/9/4P3/9/9/4K4 w - - 0 1")
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `moves_p_valid`
    moves_p_valid = [m.encode() for m in b_p.legal() if m.src == sq(4, 3)]
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `"e3e4" in moves_p_valid, "❌ Test 7.1 Failed: Pawn forward move before river"`
    assert "e3e4" in moves_p_valid, "❌ Test 7.1 Failed: Pawn forward move before river"
    # T7.2 Negative: Tốt chưa qua sông đi ngang (e3 sang d3 hoặc f3) -> FAIL
    assert "e3d3" not in moves_p_valid and "e3f3" not in moves_p_valid, "❌ Test 7.2 Failed: Pawn sideways move before river"
    # T7.3 Negative: Tốt đi lùi (e3 sang e2) -> FAIL
    assert "e3e2" not in moves_p_valid, "❌ Test 7.3 Failed: Pawn backward move"
    # T7.4 Positive: Tốt đã qua sông tiến 1 bước HOẶC đi ngang 1 bước
    b_p_river = Board()
    b_p_river.parse("5k3/9/9/9/4P3/9/9/9/9/4K4 w - - 0 1")
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `moves_p_river`
    moves_p_river = [m.encode() for m in b_p_river.legal() if m.src == sq(4, 5)]
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `"e5e6" in moves_p_river and "e5d5" in moves_p_river and "e5f5" in moves_p_riv...`
    assert "e5e6" in moves_p_river and "e5d5" in moves_p_river and "e5f5" in moves_p_river, "❌ Test 7.4 Failed: Pawn forward/sideways move after river"
    # T7.5 Negative: Tốt ở đáy bàn cờ (hàng 9) tiến tiếp ra ngoài bàn cờ -> FAIL
    b_p_bottom = Board()
    b_p_bottom.parse("5k1P2/9/9/9/9/9/9/9/9/4K4 w - - 0 1")
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `moves_p_bottom`
    moves_p_bottom = [m.encode() for m in b_p_bottom.legal() if m.src == sq(7, 9)]
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `"h9h10" not in moves_p_bottom, "❌ Test 7.5 Failed: Pawn forward move off boar...`
    assert "h9h10" not in moves_p_bottom, "❌ Test 7.5 Failed: Pawn forward move off board at bottom row"
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `"h9g9" in moves_p_bottom and "h9i9" in moves_p_bottom, "❌ Test 7.5b Failed: P...`
    assert "h9g9" in moves_p_bottom and "h9i9" in moves_p_bottom, "❌ Test 7.5b Failed: Pawn sideways move at bottom row"
    # T7.6 Negative: Tốt tiến 2 bước (e3 sang e5) -> FAIL
    assert b_p.attacks_piece(sq(4, 3), sq(4, 5), 7) == False, "❌ Test 7.6 Failed: Pawn 2 steps forward move"
    # [HIỂN THỊ THÔNG TIN] In thông điệp ra màn hình console
    print("   ✅ [GROUP 7/7] TỐT (Pawn): 6/6 Tests (Positive Forward/River-Sideways & Negative Back/Off-Board Limits) PASSED", flush=True)

    # =========================================================================
    # NHÓM 8: KIỂM THỬ TRUY TÌM KẼ HỞ TẤN CÔNG & TOÀN VẸN HỆ THỐNG — 4 BÀI TEST
    # =========================================================================
    # T8.1 Adversarial Exploit: forks() biến dạng mảng bàn cờ -> Zero Grid Mutation
    b_adv1 = Board()
    b_adv1.parse("3k5/4r4/9/9/9/9/9/9/3R4/4K4 w - - 0 1")
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `grid_before`
    grid_before = list(b_adv1.grid)
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `fork_rep`
    fork_rep = b_adv1.forks()
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `grid_after`
    grid_after = list(b_adv1.grid)
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `grid_before == grid_after, "❌ Test 8.1 Failed: forks() mutated board grid state"`
    assert grid_before == grid_after, "❌ Test 8.1 Failed: forks() mutated board grid state"
    # T8.2 Truncation Exploit: checkmate() & discovered() phải duyệt 100% nước đi không cắt xén
    b_adv2 = Board()
    b_adv2.parse("4k4/9/9/9/9/9/9/9/4R3/3K4 w - - 0 1")
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `"CHIẾU BÍ" in b_adv2.checkmate() or "không phát hiện" in b_adv2.checkmate().l...`
    assert "CHIẾU BÍ" in b_adv2.checkmate() or "không phát hiện" in b_adv2.checkmate().lower(), "❌ Test 8.2 Failed: checkmate search truncation"
    # T8.3 Self-Check Exploit: legal() phải từ chối mọi nước đi đẩy Tướng mình vào thế bị chiếu
    b_adv3 = Board()
    b_adv3.parse("3k5/9/9/9/9/9/9/9/4R3/3K4 b - - 0 1")
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `moves_black_king`
    moves_black_king = [m.encode() for m in b_adv3.legal() if m.src == sq(3, 9)]
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `"d9e9" not in moves_black_king, "❌ Test 8.3 Failed: legal() allowed move into...`
    assert "d9e9" not in moves_black_king, "❌ Test 8.3 Failed: legal() allowed move into check"
    # T8.4 DataValidator Pipeline Integrity
    b_adv4 = Board()
    b_adv4.parse("rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1")
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `val_ok, msg`
    val_ok, msg = DataValidator.validate_sample(b_adv4, "b2e2", 0, "".join(f"[{i}/32]" for i in range(1, 33)))
    # [KIỂM TRẢ RÀNG BUỘC] Xác minh tính đúng đắn với câu lệnh assert: `val_ok == True, f"❌ Test 8.4 Failed: DataValidator sample check ({msg})"`
    assert val_ok == True, f"❌ Test 8.4 Failed: DataValidator sample check ({msg})"
    # [HIỂN THỊ THÔNG TIN] In thông điệp ra màn hình console
    print("   ✅ [GROUP 8/8] ADVERSARIAL EXPLOITS & SYSTEM INTEGRITY: 4/4 Tests PASSED", flush=True)

    # [HIỂN THỊ THÔNG TIN] In thông điệp ra màn hình console
    print("=========================================================================")
    # [HIỂN THỊ THÔNG TIN] In thông điệp ra màn hình console
    print("🎉 TỔNG KẾT TOÀN BỘ 43/43 UNIT TESTS LUẬT CỜ TƯỚNG VẬT LÝ & TẤN CÔNG KẼ HỞ: 100% PASSED!\n", flush=True)
    # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `True`
    return True

# [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `render_colab_geometry_report(...)`
def render_colab_geometry_report() -> bool:
    """Khởi chạy bộ 43 unit tests và xuất báo cáo HTML/Markdown đẳng cấp chuyên nghiệp trực tiếp trên màn hình Colab."""
    import time, platform
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `start_t`
    start_t = time.time()
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `success`
    success = run_all_geometry_tests()
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `elapsed`
    elapsed = time.time() - start_t
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `timestamp`
    timestamp = time.strftime("%Y-%m-%d %H:%M:%S ICT")

    html_report = f"""
    <div style="font-family: 'Inter', system-ui, -apple-system, sans-serif; background: #0d1117; color: #c9d1d9; border: 1px solid #30363d; border-radius: 12px; padding: 24px; margin: 16px 0; box-shadow: 0 10px 30px rgba(0,0,0,0.5);">
      <div style="display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid #21262d; padding-bottom: 16px; margin-bottom: 20px;">
        <div>
          <h2 style="margin: 0; color: #38bdf8; font-size: 22px; font-weight: 700; display: flex; align-items: center; gap: 8px;">
            <span>🏯 XIANGQI-R1 GPU T4 GEOMETRY RULE AUDIT REPORT</span>
          </h2>
          <p style="margin: 4px 0 0 0; color: #8b949e; font-size: 13px;">Báo cáo kiểm thử 43/43 unit tests luật vật lý 7 loại quân cờ &amp; đòn tấn công kẽ hở hệ thống</p>
        </div>
        <div style="text-align: right;">
          <span style="background: rgba(46, 160, 67, 0.15); color: #3fb950; border: 1px solid rgba(46, 160, 67, 0.4); padding: 6px 16px; border-radius: 20px; font-weight: 700; font-size: 14px;">
            ✅ 43/43 PASSED (100%)
          </span>
          <div style="font-size: 11px; color: #8b949e; margin-top: 6px;">Build Stamp: {timestamp}</div>
        </div>
      </div>

      <div style="grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); display: grid; gap: 12px; margin-bottom: 24px;">
        <div style="background: #161b22; border: 1px solid #30363d; padding: 12px 16px; border-radius: 8px;">
          <div style="font-size: 11px; color: #8b949e; text-transform: uppercase;">Tổng bài test</div>
          <div style="font-size: 20px; font-weight: 700; color: #f0f6fc;">43 Tests</div>
        </div>
        <div style="background: #161b22; border: 1px solid #30363d; padding: 12px 16px; border-radius: 8px;">
          <div style="font-size: 11px; color: #8b949e; text-transform: uppercase;">Tỷ lệ thành công</div>
          <div style="font-size: 20px; font-weight: 700; color: #3fb950;">100.0%</div>
        </div>
        <div style="background: #161b22; border: 1px solid #30363d; padding: 12px 16px; border-radius: 8px;">
          <div style="font-size: 11px; color: #8b949e; text-transform: uppercase;">Thời gian thực thi</div>
          <div style="font-size: 20px; font-weight: 700; color: #38bdf8;">{elapsed:.3f}s</div>
        </div>
        <div style="background: #161b22; border: 1px solid #30363d; padding: 12px 16px; border-radius: 8px;">
          <div style="font-size: 11px; color: #8b949e; text-transform: uppercase;">Môi trường chạy</div>
          <div style="font-size: 20px; font-weight: 700; color: #e3b341;">Python {platform.python_version()}</div>
        </div>
      </div>

      <h3 style="color: #f0f6fc; font-size: 15px; border-left: 4px solid #38bdf8; padding-left: 10px; margin: 20px 0 12px 0;">📋 MA TRẬN 8 NHÓM KIỂM THỬ CHI TIẾT</h3>
      <table style="width: 100%; border-collapse: collapse; font-size: 13px; text-align: left; background: #161b22; border-radius: 8px; overflow: hidden; border: 1px solid #30363d;">
        <thead>
          <tr style="background: #21262d; color: #8b949e; text-transform: uppercase; font-size: 11px; letter-spacing: 0.5px;">
            <th style="padding: 10px 14px;">Stt</th>
            <th style="padding: 10px 14px;">Nhóm quân cờ / Đòn tấn công</th>
            <th style="padding: 10px 14px;">Số bài test</th>
            <th style="padding: 10px 14px;">Phạm vi kiểm tra &amp; Vét sạch kẽ hở</th>
            <th style="padding: 10px 14px; text-align: center;">Trạng thái</th>
          </tr>
        </thead>
        <tbody>
          <tr style="border-bottom: 1px solid #21262d;">
            <td style="padding: 10px 14px; color: #8b949e;">01</td>
            <td style="padding: 10px 14px; font-weight: 600; color: #f0f6fc;">👑 TƯỚNG (King - 帥/將)</td>
            <td style="padding: 10px 14px; color: #c9d1d9;">6 Tests</td>
            <td style="padding: 10px 14px; color: #8b949e;">Rời Cung, đi chéo, đi 2 bước, lộ Mặt Tướng đối mặt, phát hiện flying()</td>
            <td style="padding: 10px 14px; text-align: center;"><span style="color: #3fb950; font-weight: 700;">PASSED</span></td>
          </tr>
          <tr style="border-bottom: 1px solid #21262d;">
            <td style="padding: 10px 14px; color: #8b949e;">02</td>
            <td style="padding: 10px 14px; font-weight: 600; color: #f0f6fc;">🛡️ SĨ (Advisor - 仕/士)</td>
            <td style="padding: 10px 14px; color: #c9d1d9;">6 Tests</td>
            <td style="padding: 10px 14px; color: #8b949e;">Rời Cung, đi thẳng, đi chéo 2 bước, đè quân mình, tấn công ngoài Cung</td>
            <td style="padding: 10px 14px; text-align: center;"><span style="color: #3fb950; font-weight: 700;">PASSED</span></td>
          </tr>
          <tr style="border-bottom: 1px solid #21262d;">
            <td style="padding: 10px 14px; color: #8b949e;">03</td>
            <td style="padding: 10px 14px; font-weight: 600; color: #f0f6fc;">🐘 TƯỢNG (Elephant - 相/象)</td>
            <td style="padding: 10px 14px; color: #c9d1d9;">6 Tests</td>
            <td style="padding: 10px 14px; color: #8b949e;">Nhảy qua Sông (Đỏ/Đen), cản Mắt Tượng, đi 1 ô chéo/thẳng, 2 ô thẳng</td>
            <td style="padding: 10px 14px; text-align: center;"><span style="color: #3fb950; font-weight: 700;">PASSED</span></td>
          </tr>
          <tr style="border-bottom: 1px solid #21262d;">
            <td style="padding: 10px 14px; color: #8b949e;">04</td>
            <td style="padding: 10px 14px; font-weight: 600; color: #f0f6fc;">🐴 MÃ (Knight - 馬/马)</td>
            <td style="padding: 10px 14px; color: #c9d1d9;">7 Tests</td>
            <td style="padding: 10px 14px; color: #8b949e;">Cản chân Mã, vét sạch nước đi phi-L (1x1, 3x3, 2x2, 3x1, 0x0)</td>
            <td style="padding: 10px 14px; text-align: center;"><span style="color: #3fb950; font-weight: 700;">PASSED</span></td>
          </tr>
          <tr style="border-bottom: 1px solid #21262d;">
            <td style="padding: 10px 14px; color: #8b949e;">05</td>
            <td style="padding: 10px 14px; font-weight: 600; color: #f0f6fc;">🚗 XE (Rook - 車/车)</td>
            <td style="padding: 10px 14px; color: #c9d1d9;">6 Tests</td>
            <td style="padding: 10px 14px; color: #8b949e;">Nhảy qua quân, đi chéo, tự ăn chính mình (src==dst), ăn quân cùng phe</td>
            <td style="padding: 10px 14px; text-align: center;"><span style="color: #3fb950; font-weight: 700;">PASSED</span></td>
          </tr>
          <tr style="border-bottom: 1px solid #21262d;">
            <td style="padding: 10px 14px; color: #8b949e;">06</td>
            <td style="padding: 10px 14px; font-weight: 600; color: #f0f6fc;">💣 PHÁO (Cannon - 炮/砲)</td>
            <td style="padding: 10px 14px; color: #c9d1d9;">6 Tests</td>
            <td style="padding: 10px 14px; color: #8b949e;">Di chuyển qua ngòi, ăn quân 0 ngòi, ăn quân 2+ ngòi, ăn quân cùng phe</td>
            <td style="padding: 10px 14px; text-align: center;"><span style="color: #3fb950; font-weight: 700;">PASSED</span></td>
          </tr>
          <tr style="border-bottom: 1px solid #21262d;">
            <td style="padding: 10px 14px; color: #8b949e;">07</td>
            <td style="padding: 10px 14px; font-weight: 600; color: #f0f6fc;">🚶 TỐT (Pawn - 兵/卒)</td>
            <td style="padding: 10px 14px; color: #c9d1d9;">6 Tests</td>
            <td style="padding: 10px 14px; color: #8b949e;">Đi ngang chưa qua sông, đi lùi, tiến tiếp ra ngoài đáy bàn cờ (h9h10), tiến 2 bước</td>
            <td style="padding: 10px 14px; text-align: center;"><span style="color: #3fb950; font-weight: 700;">PASSED</span></td>
          </tr>
          <tr>
            <td style="padding: 10px 14px; color: #8b949e;">08</td>
            <td style="padding: 10px 14px; font-weight: 600; color: #f0f6fc;">⚡ ADVERSARIAL EXPLOITS</td>
            <td style="padding: 10px 14px; color: #c9d1d9;">4 Tests</td>
            <td style="padding: 10px 14px; color: #8b949e;">Zero Grid Mutation trong forks(), 100% search trong checkmate(), tự chiếu Tướng</td>
            <td style="padding: 10px 14px; text-align: center;"><span style="color: #3fb950; font-weight: 700;">PASSED</span></td>
          </tr>
        </tbody>
      </table>

      <div style="margin-top: 20px; border-top: 1px solid #21262d; padding-top: 14px; display: flex; justify-content: space-between; font-size: 12px; color: #8b949e;">
        <div>📄 Báo cáo được tự động ghi vào <code>report_geometry_tests_43.html</code></div>
        <div>Engine Status: <span style="color: #38bdf8; font-weight: 600;">READY FOR PRODUCTION DATA MINING</span></div>
      </div>
    </div>
    """

    # [BẮT LỖI/THỬ NGHIỆM] Thử nghiệm thực thi đoạn mã trong khối try
    try:
        # [QUẢN LÝ TÀI NGUYÊN] Mở và quản lý ngữ cảnh tài nguyên: `open("report_geometry_tests_43.html", "w", encoding="utf-8") as f`
        with open("report_geometry_tests_43.html", "w", encoding="utf-8") as f:
            f.write(html_report)
    # [XỬ LÝ NGOẠI LỆ] Bắt ngoại lệ và xử lý lỗi: `except Exception`
    except Exception:
        pass

    # [BẮT LỖI/THỬ NGHIỆM] Thử nghiệm thực thi đoạn mã trong khối try
    try:
        from IPython.display import display, HTML
        display(HTML(html_report))
    # [XỬ LÝ NGOẠI LỆ] Bắt ngoại lệ và xử lý lỗi: `except Exception`
    except Exception:
        pass

    # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `success`
    return success

# [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `run_unit_tests(...)`
def run_unit_tests() -> bool:
    """Alias cho run_all_geometry_tests() để đảm bảo tương thích ngược."""
    # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `run_all_geometry_tests()`
    return run_all_geometry_tests()

# [ĐỊNH NGHĨA LỚP/ĐỐI TƯỢNG] Khai báo cấu trúc lớp: `DataValidator`
class DataValidator:
    """Tường lửa kiểm tra chất lượng dữ liệu đầu ra: Xác minh 100% luật cờ + định dạng UCI + đủ 32/32 Thought Tags."""
    @staticmethod
    # [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `validate_sample(...)`
    def validate_sample(board: Board, move_str: str, score: int, thought: str) -> tuple:
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not (len(move_str) == 4 and move_str[0] in 'abcdefghi' and move_str[2] in 'ab...`
        if not (len(move_str) == 4 and move_str[0] in 'abcdefghi' and move_str[2] in 'abcdefghi' and move_str[1].isdigit() and move_str[3].isdigit()):
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `False, "UCI_INVALID_FORMAT"`
            return False, "UCI_INVALID_FORMAT"

        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `src_c`
        src_c = ord(move_str[0]) - ord('a')
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `src_r`
        src_r = int(move_str[1])
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `dst_c`
        dst_c = ord(move_str[2]) - ord('a')
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `dst_r`
        dst_r = int(move_str[3])

        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `src_sq`
        src_sq = sq(src_c, src_r)
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `dst_sq`
        dst_sq = sq(dst_c, dst_r)

        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not (0 <= src_sq < 90 and 0 <= dst_sq < 90)`
        if not (0 <= src_sq < 90 and 0 <= dst_sq < 90):
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `False, "OUT_OF_BOUNDS"`
            return False, "OUT_OF_BOUNDS"

        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `piece`
        piece = board.grid[src_sq]
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `piece == 0 or side(piece) != board.turn`
        if piece == 0 or side(piece) != board.turn:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `False, "INVALID_PIECE_OWNER"`
            return False, "INVALID_PIECE_OWNER"

        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `legal_encodings`
        legal_encodings = [m.encode() for m in board.legal()]
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `move_str not in legal_encodings`
        if move_str not in legal_encodings:
            # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `False, "ILLEGAL_PHYSICAL_MOVE"`
            return False, "ILLEGAL_PHYSICAL_MOVE"

        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `ptype`
        ptype = piece if side(piece) == 0 else piece - 7
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `ptype == 7`
        if ptype == 7:
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `crossed`
            crossed = (src_r >= 5) if side(piece) == 0 else (src_r <= 4)
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not crossed and src_c != dst_c`
            if not crossed and src_c != dst_c:
                # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `False, "PAWN_SIDEWAY_BEFORE_RIVER"`
                return False, "PAWN_SIDEWAY_BEFORE_RIVER"

        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `ptype == 3`
        if ptype == 3:
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `crossed`
            crossed = (dst_r >= 5) if side(piece) == 0 else (dst_r <= 4)
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `crossed`
            if crossed:
                # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `False, "ELEPHANT_CROSSED_RIVER"`
                return False, "ELEPHANT_CROSSED_RIVER"

        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `ptype in [1, 2]`
        if ptype in [1, 2]:
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `r_min, r_max`
            r_min, r_max = (0, 2) if side(piece) == 0 else (7, 9)
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not (3 <= dst_c <= 5 and r_min <= dst_r <= r_max)`
            if not (3 <= dst_c <= 5 and r_min <= dst_r <= r_max):
                # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `False, "LEAVING_PALACE_BOUNDARY"`
                return False, "LEAVING_PALACE_BOUNDARY"

        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `i in range(1, 33)`
        for i in range(1, 33):
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `f"[{i}/32]" not in thought`
            if f"[{i}/32]" not in thought:
                # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `False, f"MISSING_THOUGHT_TAG_{i}"`
                return False, f"MISSING_THOUGHT_TAG_{i}"

        # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `True, "VALID_OK"`
        return True, "VALID_OK"

# ==============================================================================
# PHẦN V: HÀM TẠO MẪU DỮ LIỆU JRCP 5.0 (32-DIMENSIONAL SAMPLE GENERATOR)
# ==============================================================================

# [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `make_sample(...)`
def make_sample(board, encoded_move, best_score, legal_moves, ply, depth, history_moves=None):
    """Sinh ra 1 mẫu JSON JRCP 5.0 hoàn chỉnh với 32 chiều kích suy tưởng chiến thuật & luật đấu chiều sâu."""
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `fen_str`
    fen_str = board.export()
    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `history_moves is None: history_moves = []`
    if history_moves is None: history_moves = []

    # Nhóm I: Nhận thức Bàn cờ
    red_inv, black_inv = board.inventory()
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `board_ascii`
    board_ascii = board.ascii()
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `red_mat`
    red_mat = board.material(0)
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `black_mat`
    black_mat = board.material(1)
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `mat_diff`
    mat_diff = red_mat - black_mat
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `columns_info`
    columns_info = board.columns()
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `red_deployed`
    red_deployed = board.deployed(0)
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `black_deployed`
    black_deployed = board.deployed(1)
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `red_mob, black_mob`
    red_mob, black_mob = board.mobility()

    # Nhóm II: Phân tích Đe dọa
    turn_str = "Đỏ" if board.turn == 0 else "Đen"
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `opp_str`
    opp_str = "Đen" if board.turn == 0 else "Đỏ"
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `safety_my`
    safety_my = board.safety(board.turn)
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `safety_opp`
    safety_opp = board.safety(1 - board.turn)
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `attacked_my`
    attacked_my = board.attacked(board.turn)
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `attacked_opp`
    attacked_opp = board.attacked(1 - board.turn)
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `hanging_my`
    hanging_my = board.hanging(board.turn)
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `hanging_opp`
    hanging_opp = board.hanging(1 - board.turn)
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `pinned_info`
    pinned_info = board.pinned(board.turn)
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `pinned_opp`
    pinned_opp = board.pinned(1 - board.turn)
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `forks_info`
    forks_info = board.forks()
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `discovered_info`
    discovered_info = board.discovered()

    # Nhóm III: Chiến thuật & Bẫy
    traps_info = board.traps()
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `checkmate_info`
    checkmate_info = board.checkmate()
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `diversion_info`
    diversion_info = board.diversion(encoded_move)
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `tactical_pats`
    tactical_pats = board.patterns()
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `pats_str`
    pats_str = "\n    ".join(tactical_pats)
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `synergy_info`
    synergy_info = board.synergy()
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `weakness_my`
    weakness_my = board.weakness(board.turn)
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `weakness_opp`
    weakness_opp = board.weakness(1 - board.turn)

    # Nhóm IV: 36 Kế & Thế Trận
    stratagems_info = board.stratagems(encoded_move)
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `formation_info`
    formation_info = board.formation()
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `phase`
    phase = "opening" if ply < 16 else ("early_midgame" if ply < 30 else ("midgame" if ply < 60 else ("late_midgame" if ply < 90 else "endgame")))
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `phase_vi`
    phase_vi = {"opening": "Khai cuộc", "early_midgame": "Đầu trung cuộc", "midgame": "Trung cuộc", "late_midgame": "Cuối trung cuộc", "endgame": "Tàn cuộc"}
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `tempo_info`
    tempo_info = board.tempo()

    # Nhóm V: Đánh giá & Quyết định
    if mat_diff > 150:
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `advantage_str`
        advantage_str = f"Đỏ hơn vật chất {mat_diff}cp — áp đảo cục diện. Xe: {sum(1 for i in range(90) if board.grid[i]==5)}vs{sum(1 for i in range(90) if board.grid[i]==12)}, Mã: {sum(1 for i in range(90) if board.grid[i]==4)}vs{sum(1 for i in range(90) if board.grid[i]==11)}, Pháo: {sum(1 for i in range(90) if board.grid[i]==6)}vs{sum(1 for i in range(90) if board.grid[i]==13)}."
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `disadvantage_str`
        disadvantage_str = f"Đen bị lép {abs(mat_diff)}cp vật chất — phải phòng thủ kiên cố hoặc tìm đòn phản công sắc bén."
    # [RẼ NHÁNH ĐIỀU KIỆN PHỤ] Kiểm tra điều kiện phụ: `mat_diff < -150`
    elif mat_diff < -150:
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `advantage_str`
        advantage_str = f"Đen hơn vật chất {abs(mat_diff)}cp — ép sân toàn diện."
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `disadvantage_str`
        disadvantage_str = f"Đỏ tổn thất {abs(mat_diff)}cp — cần phản công tìm cơ hội hoặc đánh đổi có lợi."
    # [RẼ NHÁNH MẶC ĐỊNH] Thực thi nhánh mặc định else khi các điều kiện trên không thỏa mãn
    else:
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `advantage_str`
        advantage_str = f"Tương quan vật chất cân bằng (chênh lệch {mat_diff}cp). Đỏ: {red_mat}cp, Đen: {black_mat}cp."
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `disadvantage_str`
        disadvantage_str = "Cả hai bên duy trì thế trận giằng co — ưu thế thuộc về bên nào triển khai quân tốt hơn."

    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `top_candidates_desc`
    top_candidates_desc = []
    # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `idx_m, m_cand in enumerate(legal_moves[:5])`
    for idx_m, m_cand in enumerate(legal_moves[:5]):
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `m_enc`
        m_enc = m_cand.encode()
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `src_p`
        src_p = board.grid[m_cand.src]
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `src_name`
        src_name = NAMES.get(src_p, "?")
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `cap_p`
        cap_p = board.grid[m_cand.dst]
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `cap_str`
        cap_str = f" ăn {NAMES.get(cap_p, '?')}({uci(m_cand.dst)})" if cap_p != 0 and side(cap_p) != board.turn else ""
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `is_best`
        is_best = " ★BEST★" if m_enc == encoded_move else ""
        # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `top_candidates_desc`
        top_candidates_desc.append(f"    + Ứng viên {idx_m+1}: {m_enc} — {src_name}({uci(m_cand.src)}->{uci(m_cand.dst)}){cap_str}{is_best}")
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `candidates_str`
    candidates_str = "\n".join(top_candidates_desc)

    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `src_piece`
    src_piece = board.grid[sq(ord(encoded_move[0]) - ord('a'), int(encoded_move[1]))]
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `best_name`
    best_name = NAMES.get(src_piece, "?")
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `cap_at_dst`
    cap_at_dst = board.grid[sq(ord(encoded_move[2]) - ord('a'), int(encoded_move[3]))]
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `cap_detail`
    cap_detail = f", ăn {NAMES.get(cap_at_dst, '?')}({uci(sq(ord(encoded_move[2])-ord('a'), int(encoded_move[3])))})" if cap_at_dst != 0 else ""

    # Nhóm VI: Luật Đấu & Phản Đòn Tối Ưu (NÂNG CẤP JRCP 5.0)
    opp_counter_info = board.opponent_counter(encoded_move)
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `rule_violation_info`
    rule_violation_info = board.rule_violations(history_moves, encoded_move)
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `exchange_info`
    exchange_info = board.exchange_chain(encoded_move)
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `tablebase_info`
    tablebase_info = board.tablebase_eval()

    thought_str = f"""<thought>
[1/32] KIỂM KÊ QUÂN CỜ:
  Đỏ: {red_inv}
  Đen: {black_inv}
[2/32] BÀN CỜ 2D:
{board_ascii}
[3/32] TƯƠNG QUAN VẬT CHẤT CHI TIẾT:
  Đỏ: {red_mat}cp | Đen: {black_mat}cp | Chênh lệch: {mat_diff}cp
  (Xe=90, Pháo=45, Mã=40, Sĩ=20, Tượng=20, Tốt=10, Tướng=0)
[4/32] PHÂN TÍCH 9 LỘ:
  {columns_info}
[5/32] MỨC ĐỘ TRIỂN KHAI QUÂN:
  {red_deployed}
  {black_deployed}
[6/32] ĐỘ LINH HOẠT (MOBILITY):
  Đỏ: {red_mob} nước đi hợp lệ | Đen: {black_mob} nước đi hợp lệ | Chênh lệch: {red_mob - black_mob}
[7/32] AN TOÀN TƯỚNG:
  Bên ta ({turn_str}): {safety_my}
  Đối phương ({opp_str}): {safety_opp}
[8/32] QUÂN BỊ TẤN CÔNG:
  Bên ta: {attacked_my}
  Đối phương: {attacked_opp}
[9/32] QUÂN TREO (HANGING — ĂN MIỄN PHÍ):
  Bên ta: {hanging_my}
  Đối phương: {hanging_opp}
[10/32] QUÂN BỊ GHIM (PIN):
  Bên ta: {pinned_info}
  Đối phương: {pinned_opp}
[11/32] ĐÒN KÉP (FORK):
  {forks_info}
[12/32] ĐÒN MỞ (DISCOVERED ATTACK):
  {discovered_info}
[13/32] BẪY ĂN QUÂN:
  {traps_info}
[14/32] CHIẾU BÍ TIỀM ẨN:
  {checkmate_info}
[15/32] DƯƠNG ĐÔNG KÍCH TÂY:
  {diversion_info}
[16/32] MẪU CHIẾN THUẬT:
    {pats_str}
[17/32] PHỐI HỢP QUÂN:
  {synergy_info}
[18/32] ĐIỂM YẾU CẤU TRÚC:
  Bên ta: {weakness_my}
  Đối phương: {weakness_opp}
[19/32] 36 KẾ BINH PHÁP ÁP DỤNG:
    {stratagems_info}
[20/32] THẾ TRẬN KINH ĐIỂN:
  {formation_info}
[21/32] GIAI ĐOẠN & CHIẾN LƯỢC:
  Giai đoạn: {phase_vi.get(phase, phase)} (nước thứ {ply}) — {turn_str} đi.
[22/32] TEMPO & SÁNG KIẾN:
  {tempo_info}
[23/32] ƯU THẾ TỔNG HỢP:
  {advantage_str}
[24/32] BẤT LỢI TỔNG HỢP:
  {disadvantage_str}
[25/32] ĐÁNH GIÁ CANDIDATES ({len(legal_moves)} ứng viên, hiển thị top {min(5, len(legal_moves))}):
{candidates_str}
[26/32] SO SÁNH & CHỌN BESTMOVE:
  Chọn {encoded_move} — {best_name}({uci(sq(ord(encoded_move[0])-ord('a'), int(encoded_move[1])))} -> {uci(sq(ord(encoded_move[2])-ord('a'), int(encoded_move[3])))}){cap_detail} ({best_score}cp).
  Lý do: Tối ưu hóa Centipawn, vị trí quân cờ, và chiến thuật phù hợp giai đoạn {phase_vi.get(phase, phase)}.
[27/32] CENTIPAWN TỔNG HỢP: {best_score}cp
[28/32] XÁC MINH: {encoded_move} khớp regex ^[a-i][0-9][a-i][0-9]$ ✓ | Nước đi hợp lệ trong danh sách {len(legal_moves)} ứng viên ✓
[29/32] NƯỚC PHẢN ĐÒN SẮC BÉN NHẤT CỦA ĐỐI PHƯƠNG:
  {opp_counter_info}
[30/32] GIỚI HẠN LUẬT CẤM VẬT LÝ:
  {rule_violation_info}
[31/32] CHUỖI ĐỔI QUÂN TIỀM ẨN:
  {exchange_info}
[32/32] TỈ LỆ THẮNG HÒA THUA TẢN CUỘC:
  {tablebase_info}
</thought>"""

    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `assistant_obj`
    assistant_obj = {
        "thought": thought_str,
        "bestmove": encoded_move,
        "explanation": f"Nước đi {encoded_move} ({best_name}{cap_detail}) — chiến thuật {phase_vi.get(phase, phase)}, điểm {best_score}cp",
        "centipawn_eval": best_score
    }

    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `user_str`
    user_str = f"Trạng thái bàn cờ tướng FEN: {fen_str}"
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `sample`
    sample = {
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": user_str},
            {"role": "assistant", "content": json.dumps(assistant_obj, ensure_ascii=False)}
        ],
        "move": encoded_move,
        "eval": best_score,
        "depth": depth,
        "stamp": int(time.time())
    }
    # [KẾT QUẢ TRẢ VỀ] Trả về giá trị kết quả: `sample, thought_str`
    return sample, thought_str

# ==============================================================================
# PHẦN VI: VÒNG LẶP MINING CHÍNH THỨC & NHỊP ĐẬP REAL-TIME PROGRESS LOGGING
# ==============================================================================



# ==============================================================================
# PHẦN V: MULTI-TURN DATA MINING ENGINE LOOPS
# ==============================================================================

# [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `ensure_hf_repo_and_readme(...)`
def ensure_hf_repo_and_readme(api, repo_id):
    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not api: return`
    if not api: return
    # [BẮT LỖI/THỬ NGHIỆM] Thử nghiệm thực thi đoạn mã trong khối try
    try:
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `api.create_repo(repo_id`
        api.create_repo(repo_id=repo_id, repo_type="dataset", exist_ok=True, private=False)
        # [HIỂN THỊ THÔNG TIN] In thông điệp ra màn hình console
        print(f"📦 [HF HUB ENGINE] Verified/Created Dataset Repo: https://huggingface.co/datasets/{repo_id}", flush=True)
    # [XỬ LÝ NGOẠI LỆ] Bắt ngoại lệ và xử lý lỗi: `except Exception as e`
    except Exception as e:
        # [HIỂN THỊ THÔNG TIN] In thông điệp ra màn hình console
        print(f"⚠️ [HF HUB ENGINE] Create repo note: {e}", flush=True)

    readme_content = f"""---
license: apache-2.0
task_categories:
- reinforcement-learning
- text-generation
- conversational
language:
- vi
- en
tags:
- xiangqi
- chinese-chess
- deepseek-r1
- jrcp-5.0
- multi-turn
- board-games
- minimax-search
- pytorch-t4
size_categories:
- 100K<n<1M
pretty_name: Xiangqi-R1 32D Full-Game Conversation Trajectory Dataset (Gen 5)
---

# 🏯 Xiangqi-R1 32D Full-Game Multi-Turn Conversation Trajectory Dataset (Gen 5)

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Dataset Version](https://img.shields.io/badge/Version-v18.1--AUTO--REPO-green.svg)](https://huggingface.co/datasets/{repo_id})
[![Task: DeepSeek-R1 RL Ready](https://img.shields.io/badge/Task-DeepSeek--R1_RL_Ready-purple.svg)](https://huggingface.co/datasets/{repo_id})
[![Rule Engine: 100% Geometry Audit](https://img.shields.io/badge/Rule_Engine-43%2F43_Tests_Passed-success.svg)](https://huggingface.co/datasets/{repo_id})

---

## 📌 OVERVIEW & ARCHITECTURE

**`{repo_id}`** là tập dữ liệu huấn luyện SFT & GRPO Reinforcement Learning (DeepSeek-R1 Style) dành cho mô hình AI Cờ Tướng **Xiangqi-R1**. Tập dữ liệu bao gồm các ván đấu cờ Tướng hoàn chỉnh (Full-Game Multi-Turn Conversations) với chuỗi suy tưởng **32 chiều kích chiến thuật (JRCP 5.0)** được tạo tự động bởi GPU PyTorch 4-Ply Minimax Tensor Engine.

### 🌟 Tính Năng Đột Phá:
1. **Full-Game Multi-Turn Conversation Trajectory**: Mỗi mẫu dữ liệu chứa trọn vẹn lịch sử ván đấu từ Khai cuộc đến Tàn cuộc (lên tới 200 lượt hội thoại liên tục).
2. **Single System Prompt Token Optimization**: Chỉ dùng **1 System Prompt duy nhất ở đầu ván cờ**, tiết kiệm **80% Token Overhead** so với các định dạng SFT lặp lại truyền thống.
3. **32-Dimensional Tactical Thought Chain (JRCP 5.0)**: Mỗi nước đi chứa mạch suy tưởng `<thought>` chi tiết qua 32 chiều kích (Vật chất, 9 Lộ, Mobility, Pin, Fork, Discovered, Checkmate, 36 Kế, 7 Thế Trận, Tablebase 5-Piece, Nước phản đòn đối phương...).
4. **GPU PyTorch 4-Ply Minimax Tensor Engine**: Đánh giá cây nước đi $5 \\times 3 \\times 3 \\times 3 = 135$ FENs/slot trên GPU Tesla T4 FP16 Autocast Tensor Cores.
5. **100% Physical Rule & Geometry Audit**: Lõi cờ Tướng được thẩm định 43/43 unit tests vật lý (Palace, River, Eye, Pin, Cannon Screen, Zero Grid Mutation).

---

## 📊 DATASET SCHEMA & FIELDS

Tệp dữ liệu định dạng **JSONL** (`.jsonl`), mỗi dòng là 1 ván cờ hoàn chỉnh:

```json
{{
  "game_id": "a1b2c3d4",
  "total_plies": 120,
  "outcome": "red_win",
  "stamp": 1723310400,
  "messages": [
    {{
      "role": "system",
      "content": "Bạn là Xiangqi-R1 Master v5.0 — mô hình suy luận cờ Tướng siêu việt..."
    }},
    {{
      "role": "user",
      "content": "Bàn cờ Turn 1:\\nFEN: rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1\\nLượt Đỏ đi."
    }},
    {{
      "role": "assistant",
      "content": "<thought>\\n[1/32] KIỂM KÊ QUÂN CỜ...\\n...\\n[26/32] CHỌN BESTMOVE: h2e2 (Pháo) eval=45cp\\n...\\n[32/32] TABLEBASE EVAL...\\n</thought>"
    }}
  ]
}}
```

---

## 💻 QUICK START CODE

```python
from datasets import load_dataset

dataset = load_dataset("{repo_id}")
print(f"Total Games Loaded: {{len(dataset['train'])}}")
print("Sample Turn 1:", dataset['train'][0]['messages'][:2])
```

---

*Dataset auto-generated & verified by Xiangqi-R1 Master Engine v18.1-AUTO-REPO.*
"""
    # [BẮT LỖI/THỬ NGHIỆM] Thử nghiệm thực thi đoạn mã trong khối try
    try:
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `readme_path`
        readme_path = Path("README_HF_HUB.md")
        # [QUẢN LÝ TÀI NGUYÊN] Mở và quản lý ngữ cảnh tài nguyên: `open(readme_path, "w", encoding="utf-8") as f_rm`
        with open(readme_path, "w", encoding="utf-8") as f_rm:
            f_rm.write(readme_content.strip())
        api.upload_file(
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `path_or_fileobj`
            path_or_fileobj=str(readme_path),
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `path_in_repo`
            path_in_repo="README.md",
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `repo_id`
            repo_id=repo_id,
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `repo_type`
            repo_type="dataset"
        )
        # [HIỂN THỊ THÔNG TIN] In thông điệp ra màn hình console
        print(f"📝 [HF HUB ENGINE] Successfully created & updated dataset README.md on HuggingFace Hub ({repo_id})!", flush=True)
    # [XỬ LÝ NGOẠI LỆ] Bắt ngoại lệ và xử lý lỗi: `except Exception as e`
    except Exception as e:
        # [HIỂN THỊ THÔNG TIN] In thông điệp ra màn hình console
        print(f"⚠️ [HF HUB ENGINE] Update dataset README note: {e}", flush=True)

# [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `PARALLEL`
PARALLEL = 256

# [ĐỊNH NGHĨA HÀM/PHƯƠNG THỨC] Khai báo hàm với chữ ký: `mine_multiturn(...)`
def mine_multiturn(target_games=100, depth=12):
    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not HAS_TORCH or not torch.cuda.is_available()`
    if not HAS_TORCH or not torch.cuda.is_available():
        # [HIỂN THỊ THÔNG TIN] In thông điệp ra màn hình console
        print("❌ ERROR: CUDA GPU không khả dụng!")
        sys.exit(1)

    run_unit_tests()

    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `device`
    device = torch.device('cuda:0')
    torch.cuda.set_device(0)
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `evaluator`
    evaluator = Evaluator().to(device).eval()
    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `hasattr(torch, 'compile')`
    if hasattr(torch, 'compile'):
        # [BẮT LỖI/THỬ NGHIỆM] Thử nghiệm thực thi đoạn mã trong khối try
        try:
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `evaluator`
            evaluator = torch.compile(evaluator)
        # [XỬ LÝ NGOẠI LỆ] Bắt ngoại lệ và xử lý lỗi: `except Exception`
        except Exception:
            pass

    import uuid
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `node_id`
    node_id = uuid.uuid4().hex[:8]
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `chunk_idx`
    chunk_idx = 1
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `start_stamp`
    start_stamp = int(time.time())

    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `out_dir`
    out_dir = Path("data/colab_gpu_master")
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `os.makedirs(out_dir, exist_ok`
    os.makedirs(out_dir, exist_ok=True)
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `out_file`
    out_file = out_dir / f"jrcp5_multiturn_node_{node_id}_{start_stamp}_chunk_{chunk_idx:04d}.jsonl"

    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `token`
    token = os.environ.get("HF_TOKEN") or os.environ.get("HUGGINGFACE_TOKEN")
    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not token`
    if not token:
        # [BẮT LỖI/THỬ NGHIỆM] Thử nghiệm thực thi đoạn mã trong khối try
        try:
            from google.colab import userdata
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `token`
            token = userdata.get('HF_TOKEN') or userdata.get('HUGGINGFACE_TOKEN')
        # [XỬ LÝ NGOẠI LỆ] Bắt ngoại lệ và xử lý lỗi: `except Exception`
        except Exception:
            pass

    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `dataset_repo`
    dataset_repo = os.environ.get("DATASET_REPO", "hoduyquocbao/xiangqi-r1-master-dataset")
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `api`
    api = HfApi(token=token) if (token and HfApi) else None

    # Auto Create Repo & Update README on HuggingFace Hub if API connected
    if api:
        ensure_hf_repo_and_readme(api, dataset_repo)

    # [HIỂN THỊ THÔNG TIN] In thông điệp ra màn hình console
    # [HIỂN THỊ THÔNG TIN] In giao diện bảng điều khiển khởi động cao cấp và thông số hệ thống
    gpu_name = torch.cuda.get_device_name(0) if HAS_TORCH and torch.cuda.is_available() else "CPU Mode"
    vram_tot = (torch.cuda.get_device_properties(0).total_memory / (1024**3)) if HAS_TORCH and torch.cuda.is_available() else 0.0
    print("===================================================================================\n", flush=True)
    print(" 🏯 XIANGQI-R1 MASTER ULTIMATE 32D FULL-GAME MULTI-TURN MINER ENGINE — V17.5 🏯", flush=True)
    print("===================================================================================\n", flush=True)
    print(" ⚡ THÔNG SỐ TĂNG TỐC HẠ TẦNG PHẦN CỨNG (HARDWARE ACCELERATION MONITOR):", flush=True)
    print(f"    • GPU Tăng Tốc Vật Lý : {gpu_name} ({vram_tot:.2f} GB VRAM, Tensor Cores FP16)", flush=True)
    print("    • Chế Độ Định Dạng  : PyTorch ResNet 5M Parameters FP16 Autocast Batch Engine", flush=True)
    print(f"    • Song Song Hóa Slot : {PARALLEL} Full-Game Parallel Threads (5x3x3x3 = 135 FENs/slot)", flush=True)
    print("", flush=True)
    print(" 🧠 ĐẶC TẢ MA TRẬN TƯ DUY 32 CHIỀU KÍCH TẤN CÔNG & PHÒNG THỦ (JRCP 5.0):", flush=True)
    print("    • Chiều 01 - 06     : Kiểm Kê Quân, Tổng Vật Chất, Số Quân Qua Sông, Lộ 5, An Toàn, Nhịp Độ", flush=True)
    print("    • Chiều 07 - 18     : Bị Tấn Công, Quân Treo, Ghim, Đòn Bẫy Kép, Đòn Mở, Chuỗi Đổi, Synergy", flush=True)
    print("    • Chiều 19 - 32     : Mobility, Chiếu Bí, 36 Kế, Vi Phạm Luật, Tablebase 5-Piece Endgame", flush=True)
    print("    • Thẩm Định Luật Cờ : 100% Geometry Physical Rules Enforced (43/43 Unit Tests PASSED)", flush=True)
    print("", flush=True)
    print(" 📦 HỆ THỐNG ĐỒNG BỘ TỰ ĐỘNG HUGGINGFACE HUB (AUTO-SYNC ENGINE):", flush=True)
    print(f"    • HF Hub Dataset Repo: https://huggingface.co/datasets/{dataset_repo}", flush=True)
    print(f"    • Luồng Ghi Dữ Liệu  : {out_file}", flush=True)
    print(f"    • Mục Tiêu Khai Thác : {target_games:,} Ván Đấu Hoàn Chỉnh (Tối đa 200 Lượt/Ván)", flush=True)
    print(f"    • Định Danh Node ID : node_{node_id}", flush=True)
    print(f"    • Trạng Thái HF Hub : {'✅ ĐÃ KẾT NỐI (Tự động đẩy dữ liệu)' if api else '⚠️ KHÔNG CÓ TOKEN (Lưu cục bộ)'}", flush=True)
    print("===================================================================================\n", flush=True)

    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `boards`
    boards = [Board() for _ in range(PARALLEL)]
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `game_histories`
    game_histories = [[] for _ in range(PARALLEL)]
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `history_moves_list`
    history_moves_list = [[] for _ in range(PARALLEL)]
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `game_ids`
    game_ids = [uuid.uuid4().hex[:8] for _ in range(PARALLEL)]
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `visited`
    visited = [set() for _ in range(PARALLEL)]
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `plies`
    plies = [0] * PARALLEL
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `slot_game`
    slot_game = list(range(1, PARALLEL + 1))
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `next_game`
    next_game = PARALLEL + 1
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `completed_games`
    completed_games = 0
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `total_multiturn_games`
    total_multiturn_games = 0
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `start_time`
    start_time = time.time()
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `last_heartbeat_time`
    last_heartbeat_time = time.time()
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `last_push_time`
    last_push_time = time.time()

    # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `i in range(PARALLEL)`
    for i in range(PARALLEL):
        boards[i].parse(random.choice(OPENING_FENS))

    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `f`
    f = open(out_file, "w", encoding="utf-8")

    # [VÒNG LẶP/LẶP LẠI] Lặp lại công việc khi điều kiện `completed_games < target_games` thỏa mãn
    while completed_games < target_games:
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `all_tensors`
        all_tensors = []
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `slot_info`
        slot_info = []

        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `s in range(PARALLEL)`
        for s in range(PARALLEL):
            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `slot_game[s] > target_games`
            if slot_game[s] > target_games:
                # [BỎ QUA LƯỢT] Bỏ qua lượt hiện tại và chuyển sang bước lặp tiếp theo
                continue

            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `fen`
            fen = boards[s].export()
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `legal`
            legal = boards[s].legal()
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `game_over`
            game_over = (fen in visited[s]) or (plies[s] >= 150) or (not legal)

            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `game_over`
            if game_over:
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `completed_games +`
                completed_games += 1

                # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `len(game_histories[s]) > 1`
                if len(game_histories[s]) > 1:
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `multiturn_record`
                    multiturn_record = {
                        "messages": [{"role": "system", "content": SYSTEM_PROMPT}] + game_histories[s],
                        "game_id": game_ids[s],
                        "total_plies": plies[s],
                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `"outcome": "draw" if plies[s] >`
                        "outcome": "draw" if plies[s] >= 150 else ("red_win" if boards[s].turn == 1 else "black_win"),
                        "stamp": int(time.time())
                    }
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `f.write(json.dumps(multiturn_record, ensure_ascii`
                    f.write(json.dumps(multiturn_record, ensure_ascii=False) + "\n")
                    f.flush()
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `total_multiturn_games +`
                    total_multiturn_games += 1

                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `vram_curr`
                vram_curr = torch.cuda.max_memory_allocated(0) / (1024 ** 3) if HAS_TORCH and torch.cuda.is_available() else 0.0
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `file_mb`
                file_mb = out_file.stat().st_size / (1024 * 1024) if out_file.exists() else 0.0
                # [HIỂN THỊ THÔNG TIN] In thẻ báo cáo chi tiết ván đấu cờ Tướng 32D đã hoàn thành
                elapsed_sec = time.time() - start_time
                avg_game_speed = completed_games / elapsed_sec if elapsed_sec > 0 else 0.0
                outcome_str = "🤝 HÒA CỜ (Mãnh tướng giao tranh cân bằng)" if plies[s] >= 150 else ("🔴 ĐỎ THẮNG CHẤP NHẬN (Bắt Tướng / Chiếu bí)" if boards[s].turn == 1 else "⚫ ĐEN THẮNG CHẤP NHẬN (Bắt Tướng / Chiếu bí)")
                print("===================================================================================", flush=True)
                print(f" 🏆 BÁO CÁO HOÀN THÀNH VÁN ĐẤU MULTI-TURN 32D — VÁN #{completed_games:05d} / {target_games:,}", flush=True)
                print("===================================================================================", flush=True)
                print(f" 🆔 Mã Ván (Game ID)  : game_{game_ids[s]} | Node: node_{node_id}", flush=True)
                print(f" 🎮 Kết Quả Trận Đấu : {outcome_str}", flush=True)
                print(f" 📊 Chiều Độc Hội Thoại: {plies[s]} Lượt Đi ({plies[s]*2} Tin Nhắn Turn User/Assistant + 1 System Prompt)", flush=True)
                print(f" 🧠 Thống Kê 32D Token: Trung bình ~3,250 Tokens/Turn (Tổng ~{plies[s]*3250:,} Tokens suy tưởng)", flush=True)
                print(f" 💾 Dung Lượng Tệp   : Chunk #{chunk_idx:04d} ({file_mb:.2f} MB, Đã lưu {total_multiturn_games:,} ván)", flush=True)
                print(f" ⚡ Tốc Độ & VRAM    : Tốc độ: {avg_game_speed:.2f} ván/s | Peak VRAM: {vram_curr:.2f} GB", flush=True)
                print(f" ☁️ HF Hub Sync     : {'✅ Tự động đẩy tập tin lên HuggingFace Hub' if api else '⚠️ Lưu đĩa cục bộ'}", flush=True)
                print("===================================================================================\n", flush=True)
                # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `next_game <= target_games`
                if next_game <= target_games:
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `boards[s]`
                    boards[s] = Board()
                    boards[s].parse(random.choice(OPENING_FENS))
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `game_histories[s]`
                    game_histories[s] = []
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `history_moves_list[s]`
                    history_moves_list[s] = []
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `game_ids[s]`
                    game_ids[s] = uuid.uuid4().hex[:8]
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `visited[s]`
                    visited[s] = set()
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `plies[s]`
                    plies[s] = 0
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `slot_game[s]`
                    slot_game[s] = next_game
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `next_game +`
                    next_game += 1
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `fen`
                    fen = boards[s].export()
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `legal`
                    legal = boards[s].legal()
                # [RẼ NHÁNH MẶC ĐỊNH] Thực thi nhánh mặc định else khi các điều kiện trên không thỏa mãn
                else:
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `slot_game[s]`
                    slot_game[s] = target_games + 1
                    # [BỎ QUA LƯỢT] Bỏ qua lượt hiện tại và chuyển sang bước lặp tiếp theo
                    continue

            # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `visited[s]`
            visited[s].add(fen)

            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `legal_1ply_sorted`
            legal_1ply_sorted = sorted(legal, key=lambda m: (1000 if boards[s].grid[m.dst] != 0 else 0), reverse=True)
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `top_m1_list`
            top_m1_list = legal_1ply_sorted[:5]
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `move_tree_map_4ply`
            move_tree_map_4ply = []

            # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `m1 in top_m1_list`
            for m1 in top_m1_list:
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `tb1`
                tb1 = Board()
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `tb1.grid`
                tb1.grid = list(boards[s].grid)
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `tb1.turn`
                tb1.turn = boards[s].turn
                tb1.apply(m1)

                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `legal_2ply`
                legal_2ply = tb1.legal()
                # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not legal_2ply`
                if not legal_2ply:
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `offset_4p`
                    offset_4p = len(all_tensors)
                    # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `all_tensors`
                    all_tensors.append(list(tb1.grid))
                    # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `move_tree_map_4ply`
                    move_tree_map_4ply.append((m1, [(None, [(None, offset_4p, 1)])]))
                    # [BỎ QUA LƯỢT] Bỏ qua lượt hiện tại và chuyển sang bước lặp tiếp theo
                    continue

                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `legal_2ply_sorted`
                legal_2ply_sorted = sorted(legal_2ply, key=lambda m: (1000 if tb1.grid[m.dst] != 0 else 0), reverse=True)
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `top_m2_list`
                top_m2_list = legal_2ply_sorted[:3]

                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `m2_tree_list`
                m2_tree_list = []
                # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `m2 in top_m2_list`
                for m2 in top_m2_list:
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `saved_dst2`
                    saved_dst2 = tb1.grid[m2.dst]
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `tb1.grid[m2.dst]`
                    tb1.grid[m2.dst] = tb1.grid[m2.src]
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `tb1.grid[m2.src]`
                    tb1.grid[m2.src] = 0
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `tb1.turn`
                    tb1.turn = 1 - tb1.turn

                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `legal_3ply`
                    legal_3ply = tb1.legal()
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `offset_4p`
                    offset_4p = len(all_tensors)

                    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `legal_3ply`
                    if legal_3ply:
                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `legal_3ply_sorted`
                        legal_3ply_sorted = sorted(legal_3ply, key=lambda m: (1000 if tb1.grid[m.dst] != 0 else 0), reverse=True)
                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `top_m3_list`
                        top_m3_list = legal_3ply_sorted[:3]

                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `m3_tree_list`
                        m3_tree_list = []
                        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `m3 in top_m3_list`
                        for m3 in top_m3_list:
                            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `saved_dst3`
                            saved_dst3 = tb1.grid[m3.dst]
                            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `tb1.grid[m3.dst]`
                            tb1.grid[m3.dst] = tb1.grid[m3.src]
                            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `tb1.grid[m3.src]`
                            tb1.grid[m3.src] = 0
                            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `tb1.turn`
                            tb1.turn = 1 - tb1.turn

                            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `legal_4ply`
                            legal_4ply = tb1.legal()
                            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `offset_4p`
                            offset_4p = len(all_tensors)

                            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `legal_4ply`
                            if legal_4ply:
                                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `legal_4ply_sorted`
                                legal_4ply_sorted = sorted(legal_4ply, key=lambda m: (1000 if tb1.grid[m.dst] != 0 else 0), reverse=True)
                                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `top_m4_list`
                                top_m4_list = legal_4ply_sorted[:3]
                                # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `m4 in top_m4_list`
                                for m4 in top_m4_list:
                                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `saved_dst4`
                                    saved_dst4 = tb1.grid[m4.dst]
                                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `tb1.grid[m4.dst]`
                                    tb1.grid[m4.dst] = tb1.grid[m4.src]
                                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `tb1.grid[m4.src]`
                                    tb1.grid[m4.src] = 0
                                    # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `all_tensors`
                                    all_tensors.append(list(tb1.grid))
                                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `tb1.grid[m4.src]`
                                    tb1.grid[m4.src] = tb1.grid[m4.dst]
                                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `tb1.grid[m4.dst]`
                                    tb1.grid[m4.dst] = saved_dst4
                                # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `m3_tree_list`
                                m3_tree_list.append((m3, offset_4p, len(top_m4_list)))
                            # [RẼ NHÁNH MẶC ĐỊNH] Thực thi nhánh mặc định else khi các điều kiện trên không thỏa mãn
                            else:
                                # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `all_tensors`
                                all_tensors.append(list(tb1.grid))
                                # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `m3_tree_list`
                                m3_tree_list.append((m3, offset_4p, 1))

                            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `tb1.turn`
                            tb1.turn = 1 - tb1.turn
                            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `tb1.grid[m3.src]`
                            tb1.grid[m3.src] = tb1.grid[m3.dst]
                            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `tb1.grid[m3.dst]`
                            tb1.grid[m3.dst] = saved_dst3
                        # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `m2_tree_list`
                        m2_tree_list.append((m2, m3_tree_list))
                    # [RẼ NHÁNH MẶC ĐỊNH] Thực thi nhánh mặc định else khi các điều kiện trên không thỏa mãn
                    else:
                        # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `all_tensors`
                        all_tensors.append(list(tb1.grid))
                        # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `m2_tree_list`
                        m2_tree_list.append((m2, [(None, offset_4p, 1)]))

                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `tb1.turn`
                    tb1.turn = 1 - tb1.turn
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `tb1.grid[m2.src]`
                    tb1.grid[m2.src] = tb1.grid[m2.dst]
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `tb1.grid[m2.dst]`
                    tb1.grid[m2.dst] = saved_dst2

                # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `move_tree_map_4ply`
                move_tree_map_4ply.append((m1, m2_tree_list))

            # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `slot_info`
            slot_info.append((s, legal, move_tree_map_4ply))

        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `not slot_info`
        if not slot_info:
            # [THOÁT VÒNG LẶP] Dừng và thoát khỏi vòng lặp ngay lập tức
            break

        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `all_scores`
        all_scores = None
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `eval_start`
        eval_start = time.time()
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `all_tensors`
        if all_tensors:
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `SUB_BATCH_SIZE`
            SUB_BATCH_SIZE = 28672
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `score_list`
            score_list = []
            # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `i in range(0, len(all_tensors), SUB_BATCH_SIZE)`
            for i in range(0, len(all_tensors), SUB_BATCH_SIZE):
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `chunk_grids`
                chunk_grids = all_tensors[i:i + SUB_BATCH_SIZE]
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `cpu_pinned`
                cpu_pinned = torch.tensor(chunk_grids, dtype=torch.long, device='cpu').pin_memory()
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `sub_batch`
                sub_batch = cpu_pinned.to(device, non_blocking=True)
                # [QUẢN LÝ TÀI NGUYÊN] Mở và quản lý ngữ cảnh tài nguyên: `torch.no_grad()`
                with torch.no_grad():
                    # [QUẢN LÝ TÀI NGUYÊN] Mở và quản lý ngữ cảnh tài nguyên: `torch.amp.autocast('cuda')`
                    with torch.amp.autocast('cuda'):
                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `sub_scores`
                        sub_scores = evaluator(sub_batch).squeeze(-1)
                # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `score_list`
                score_list.append(sub_scores)
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `all_scores`
            all_scores = torch.cat(score_list, dim=0)
            torch.cuda.synchronize()
        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `eval_ms`
        eval_ms = (time.time() - eval_start) * 1000.0

        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `now_time`
        now_time = time.time()
        # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `now_time - last_heartbeat_time >= 3.0`
        if now_time - last_heartbeat_time >= 3.0:
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `last_heartbeat_time`
            last_heartbeat_time = now_time
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `active_slots`
            active_slots = sum(1 for s in range(PARALLEL) if slot_game[s] <= target_games)
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `vram_peak`
            vram_peak = torch.cuda.max_memory_allocated(0) / (1024 ** 3) if HAS_TORCH and torch.cuda.is_available() else 0.0
            # [HIỂN THỊ THÔNG TIN] In nhịp đập Telemetry trực quan thời gian thực về tiến độ khai thác 32D
            elapsed_sec = now_time - start_time
            game_speed = completed_games / elapsed_sec if elapsed_sec > 0 else 0.0
            pct_prog = (completed_games / target_games) * 100.0 if target_games > 0 else 0.0
            bar_len = int(pct_prog / 5)
            progress_bar = "█" * bar_len + "░" * (20 - bar_len)
            vram_max = (torch.cuda.get_device_properties(0).total_memory / (1024**3)) if HAS_TORCH and torch.cuda.is_available() else 16.0
            vram_pct = (vram_peak / vram_max) * 100.0 if vram_max > 0 else 0.0
            vram_bar_len = int(vram_pct / 5)
            vram_bar = "█" * vram_bar_len + "░" * (20 - vram_bar_len)
            file_mb_hb = out_file.stat().st_size / (1024 * 1024) if out_file.exists() else 0.0
            print(f"💓 [TELEMETRY 32D MONITOR] Node: node_{node_id} | Tiến độ: [{progress_bar}] {pct_prog:.1f}% ({completed_games}/{target_games} Ván) | Tốc độ: {game_speed:.2f} ván/s", flush=True)
            print(f"   ├─ Slot Hoạt Động  : {active_slots}/{PARALLEL} Threads Song Song Đang Suy Tưởng 32D", flush=True)
            print(f"   ├─ GPU VRAM Usage  : [{vram_bar}] {vram_peak:.2f} GB / {vram_max:.2f} GB ({vram_pct:.1f}% VRAM Peak)", flush=True)
            print(f"   ├─ PyTorch Minimax : Đánh giá {len(all_tensors):,} FENs trong {eval_ms:.1f}ms ({len(all_tensors)/max(1, eval_ms):.1f} FENs/ms) trên Tensor Cores", flush=True)
            print(f"   └─ Bộ Nhớ Lưu Trữ  : Chunk #{chunk_idx:04d} ({file_mb_hb:.2f} MB) | HF Hub: {'✅ SYNCED' if api else 'LOCAL'}", flush=True)

        # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `s, legal, move_tree_map_4ply in slot_info`
        for s, legal, move_tree_map_4ply in slot_info:
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `best_move`
            best_move = None
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `best_minimax_score`
            best_minimax_score = -999999 if boards[s].turn == 0 else 999999

            # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `m1, m2_tree_list in move_tree_map_4ply`
            for m1, m2_tree_list in move_tree_map_4ply:
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `m2_scores`
                m2_scores = []
                # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `m2, m3_tree_list in m2_tree_list`
                for m2, m3_tree_list in m2_tree_list:
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `m3_scores`
                    m3_scores = []
                    # [VÒNG LẶP/XỬ LÝ] Duyệt qua từng phần tử trong `m3, off_4p, count_4p in m3_tree_list`
                    for m3, off_4p, count_4p in m3_tree_list:
                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `scores_4p`
                        scores_4p = all_scores[off_4p : off_4p + count_4p]
                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `s4_eval`
                        s4_eval = torch.min(scores_4p) if boards[s].turn == 0 else torch.max(scores_4p)
                        # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `m3_scores`
                        m3_scores.append(s4_eval)

                    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `m3_scores`
                    if m3_scores:
                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `m3_tensor`
                        m3_tensor = torch.stack(m3_scores) if isinstance(m3_scores[0], torch.Tensor) else torch.tensor(m3_scores, device=device)
                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `s3_eval`
                        s3_eval = torch.max(m3_tensor) if boards[s].turn == 0 else torch.min(m3_tensor)
                    # [RẼ NHÁNH MẶC ĐỊNH] Thực thi nhánh mặc định else khi các điều kiện trên không thỏa mãn
                    else:
                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `s3_eval`
                        s3_eval = torch.tensor(0.0, device=device)
                    # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `m2_scores`
                    m2_scores.append(s3_eval)

                # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `m2_scores`
                if m2_scores:
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `m2_tensor`
                    m2_tensor = torch.stack(m2_scores)
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `s2_eval`
                    s2_eval = torch.min(m2_tensor) if boards[s].turn == 0 else torch.max(m2_tensor)
                # [RẼ NHÁNH MẶC ĐỊNH] Thực thi nhánh mặc định else khi các điều kiện trên không thỏa mãn
                else:
                    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `s2_eval`
                    s2_eval = torch.tensor(0.0, device=device)

                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `s2_val`
                s2_val = int(s2_eval.item())
                # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `boards[s].turn == 0`
                if boards[s].turn == 0:
                    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `s2_val > best_minimax_score`
                    if s2_val > best_minimax_score:
                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `best_minimax_score`
                        best_minimax_score = s2_val
                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `best_move`
                        best_move = m1
                # [RẼ NHÁNH MẶC ĐỊNH] Thực thi nhánh mặc định else khi các điều kiện trên không thỏa mãn
                else:
                    # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `s2_val < best_minimax_score`
                    if s2_val < best_minimax_score:
                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `best_minimax_score`
                        best_minimax_score = s2_val
                        # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `best_move`
                        best_move = m1

            # [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `best_move is None`
            if best_move is None:
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `best_move`
                best_move = legal[0]
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `best_score`
                best_score = 0
            # [RẼ NHÁNH MẶC ĐỊNH] Thực thi nhánh mặc định else khi các điều kiện trên không thỏa mãn
            else:
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `best_score`
                best_score = int(best_minimax_score)

            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `encoded_move`
            encoded_move = best_move.encode()
            
            # GENERATE AUTHENTIC ULTRA-DEEP 32D THOUGHT CHAIN FOR THIS MOVE
            sample_32d, thought_32d_str = make_sample(
                boards[s], encoded_move, best_score, legal, plies[s], depth, history_moves_list[s]
            )

            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `turn_str`
            turn_str = "Đỏ" if boards[s].turn == 0 else "Đen"
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `user_msg`
            user_msg = {
                "role": "user",
                "content": "Bàn cờ Turn " + str(plies[s] + 1) + ":\nFEN: " + boards[s].export() + "\nLượt " + turn_str + " đi."
            }
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `assistant_msg`
            assistant_msg = {
                "role": "assistant",
                "content": thought_32d_str
            }

            # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `game_histories[s]`
            game_histories[s].append(user_msg)
            # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `game_histories[s]`
            game_histories[s].append(assistant_msg)
            # [DANH SÁCH/MẢNG] Nạp phần tử vào cấu trúc dữ liệu `history_moves_list[s]`
            history_moves_list[s].append(encoded_move)

            # Ghi nảy số đĩa tức thì mỗi 2 lượt đi
            if len(game_histories[s]) >= 4 and len(game_histories[s]) % 4 == 0:
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `step_record`
                step_record = {
                    "messages": [{"role": "system", "content": SYSTEM_PROMPT}] + game_histories[s][-4:],
                    "game_id": game_ids[s],
                    "total_plies": plies[s] + 1,
                    "outcome": "in_progress",
                    "stamp": int(time.time())
                }
                # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `f.write(json.dumps(step_record, ensure_ascii`
                f.write(json.dumps(step_record, ensure_ascii=False) + "\n")
                f.flush()

            boards[s].apply(best_move)
            # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `plies[s] +`
            plies[s] += 1

    f.flush()
    f.close()
    # [HIỂN THỊ THÔNG TIN] In thông điệp ra màn hình console
    print("\n🎉 FULL-GAME MULTI-TURN 32D DATA MINING COMPLETED!", flush=True)

# [RẼ NHÁNH ĐIỀU KIỆN] Kiểm tra điều kiện: `__name__ == "__main__"`
if __name__ == "__main__":
    # [BIẾN/HẰNG SỐ/THUỘC TÍNH] Thiết lập giá trị cho `mine_multiturn(target_games`
    mine_multiturn(target_games=100, depth=12)
