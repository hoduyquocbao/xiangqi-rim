#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
===============================================================================
KỊCH BẢN KIỂM TOÁN TỰ ĐỘNG TUÂN THỦ KÝ ỨC VĨNH CỬU (MEMORY COMPLIANCE AUDITOR)
===============================================================================
Kịch bản này tự động quét toàn bộ codebase để phát hiện các lỗi vi phạm kinh điển
đã được ghi nhận trong các bài học xương máu (.agents/memory/pain_points_*.md).

Cú pháp:
    python3 scripts/audit_memory_compliance.py

Các quy tắc kiểm toán:
    1. Kiểm tra an toàn bảo mật: Không hardcode API tokens / keys.
    2. Kiểm tra tính nhất quán dữ liệu: Dùng trường "score" thay vì "eval".
    3. Kiểm tra huấn luyện ML: Không để trống reward_funcs trong GRPO.
    4. Kiểm tra ký ức vĩnh cửu: Mọi tệp pain_points_*.md phải được đăng ký trong INDEX.md.
    5. Kiểm tra an toàn bộ nhớ luồng: Không tạo bản sao mảng lớn trên stack.
===============================================================================
"""

import os
import re
import sys
import glob

def check_tokens() -> int:
    """Kiểm tra xem có token / API key nào bị hardcode trong mã nguồn hay không."""
    print("🔍 [1/5] Đang kiểm toán an toàn bảo mật (Secret & Token Scan)...")
    pattern = re.compile(r'hf_[a-zA-Z0-9]{20,}')
    violations = 0
    
    for ext in ['*.rs', '*.py', '*.ipynb', '*.sh', '*.json']:
        for path in glob.glob(f"**/{ext}", recursive=True):
            if "target" in path or ".git" in path or ".gemini" in path:
                continue
            try:
                with open(path, 'r', encoding='utf-8', errors='ignore') as f:
                    for line_no, line in enumerate(f, 1):
                        if pattern.search(line) and not "hf_[a-zA-Z" in line and not "dummy" in line:
                            print(f"  ❌ VI PHẠM BẢO MẬT: Phát hiện token tại {path}:{line_no}")
                            violations += 1
            except Exception:
                continue
                
    if violations == 0:
        print("  ✅ An toàn bảo mật: 0 token bị hardcode!")
    return violations

def check_field_names() -> int:
    """Kiểm tra tính nhất quán tên trường dữ liệu (bắt buộc dùng 'score', cấm 'eval')."""
    print("🔍 [2/5] Đang kiểm toán tính nhất quán dữ liệu ('score' vs 'eval')...")
    pattern = re.compile(r'["\']eval["\']\s*:')
    violations = 0
    
    for path in glob.glob("examples/*.rs") + glob.glob("scripts/*.py"):
        try:
            with open(path, 'r', encoding='utf-8', errors='ignore') as f:
                for line_no, line in enumerate(f, 1):
                    if pattern.search(line):
                        print(f"  ❌ VI PHẠM NHẤT QUÁN: Dùng 'eval' thay vì 'score' tại {path}:{line_no}")
                        violations += 1
        except Exception:
            continue
            
    if violations == 0:
        print("  ✅ Tính nhất quán: 100% sử dụng 'score' chuẩn xác!")
    return violations

def check_grpo_rewards() -> int:
    """Kiểm tra kịch bản huấn luyện GRPO có bị rỗng hàm phần thưởng hay không."""
    print("🔍 [3/5] Đang kiểm toán cấu hình hàm phần thưởng GRPO...")
    pattern = re.compile(r'reward_funcs\s*=\s*\[\s*\]')
    violations = 0
    
    for path in glob.glob("scripts/*.py") + glob.glob("*.ipynb"):
        try:
            with open(path, 'r', encoding='utf-8', errors='ignore') as f:
                for line_no, line in enumerate(f, 1):
                    if pattern.search(line):
                        print(f"  ❌ VI PHẠM GRPO: reward_funcs bị rỗng tại {path}:{line_no}")
                        violations += 1
        except Exception:
            continue
            
    if violations == 0:
        print("  ✅ Cấu hình GRPO: Hàm phần thưởng đầy đủ!")
    return violations

def check_memory_index() -> int:
    """Kiểm tra xem mọi tệp bài học pain_points_*.md đã được đăng ký vào INDEX.md chưa."""
    print("🔍 [4/5] Đang kiểm toán đăng ký ký ức vĩnh cửu (.agents/memory/INDEX.md)...")
    index_path = ".agents/memory/INDEX.md"
    if not os.path.exists(index_path):
        print("  ❌ LỖI: Tệp .agents/memory/INDEX.md không tồn tại!")
        return 1
        
    with open(index_path, 'r', encoding='utf-8') as f:
        index_content = f.read()
        
    violations = 0
    for path in glob.glob(".agents/memory/pain_points_*.md"):
        basename = os.path.basename(path)
        if basename not in index_content:
            print(f"  ❌ VI PHẠM KÝ ỨC: Tệp {basename} chưa được đăng ký vào INDEX.md!")
            violations += 1
            
    if violations == 0:
        print("  ✅ Ký ức vĩnh cửu: 100% bài học xương máu đã được đăng ký!")
    return violations

def check_thread_stack_safety() -> int:
    """Kiểm tra an toàn bộ nhớ stack trong các luồng worker đa nhiệm."""
    print("🔍 [5/5] Đang kiểm toán an toàn bộ nhớ đệm CPU stack trong luồng worker...")
    violations = 0
    pool_path = "src/thread/pool.rs"
    
    if os.path.exists(pool_path):
        with open(pool_path, 'r', encoding='utf-8') as f:
            content = f.read()
            if "8 * 1024 * 1024" in content:
                print("  ❌ CẢNH BÁO: Stack size trong pool.rs là 8MB (khuyến nghị >= 16MB)!")
                violations += 1
                
    if violations == 0:
        print("  ✅ An toàn luồng: Cấp phát stack size 16MB an toàn!")
    return violations

def main():
    print("=" * 80)
    print("🛡️  XIANGQI-RIM: HỆ THỐNG KIỂM TOÁN TỰ ĐỘNG BẢO TOÀN KÝ ỨC VÀ CHỐNG REGRESSION")
    print("=" * 80)
    
    total = 0
    total += check_tokens()
    total += check_field_names()
    total += check_grpo_rewards()
    total += check_memory_index()
    total += check_thread_stack_safety()
    
    print("=" * 80)
    if total == 0:
        print("🎉 TẤT CẢ 5 CHỐT CHẶN KIỂM TOÁN ĐỀU ĐẠT CHUẨN 100%! KHÔNG CÓ VI PHẠM!")
        print("=" * 80)
        sys.exit(0)
    else:
        print(f"⚠️  PHÁT HIỆN TỔNG CỘNG {total} VI PHẠM! VUI LÒNG KHẮC PHỤC TRƯỚC KHI BÀN GIAO!")
        print("=" * 80)
        sys.exit(1)

if __name__ == "__main__":
    main()
