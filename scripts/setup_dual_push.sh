#!/usr/bin/env bash
# ============================================================================
# XIANGQI-RIM: LOCAL DUAL-PUSH CONFIGURATION SCRIPT
# ============================================================================
# Cấu hình Git Remote để khi gõ `git push origin main` ở máy cục bộ,
# code sẽ được đẩy ĐỒNG THỜI lên cả 2 nền tảng: GitHub và HuggingFace Space.
# ============================================================================

set -e

GITHUB_REPO="git@github.com:hoduyquocbao/xiangqi-rim.git"
HF_SPACE_REPO="https://huggingface.co/spaces/hoduyquocbao/xiangqi-rim"

echo "============================================================"
echo "⚙️ CẤU HÌNH LOCAL DUAL-PUSH (GITHUB + HUGGINGFACE SPACE)"
echo "============================================================"

# Cấu hình remote `origin` đẩy đồng thời 2 nơi
git remote set-url --add --push origin "$GITHUB_REPO" 2>/dev/null || true
git remote set-url --add --push origin "$HF_SPACE_REPO" 2>/dev/null || true

echo "✅ Đã cấu hình Git Remote origin!"
echo "   Push URL 1: $GITHUB_REPO"
echo "   Push URL 2: $HF_SPACE_REPO"
echo ""
echo "🚀 Từ bây giờ, chỉ cần gõ: git push origin main"
echo "   Code sẽ tự động đẩy lên CẢ 2 NỀN TẢNG cùng một lúc!"
echo "============================================================"
