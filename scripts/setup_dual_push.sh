#!/usr/bin/env bash
# ============================================================================
# XIANGQI-RIM: LOCAL DUAL-PUSH CONFIGURATION SCRIPT (AUTO TOKEN AUTH)
# ============================================================================
# Cấu hình Git Remote để khi gõ `git push origin main` ở máy cục bộ,
# code sẽ được đẩy ĐỒNG THỜI lên cả 2 nền tảng: GitHub và HuggingFace Space.
# ============================================================================

set -e

GITHUB_REPO="https://github.com/hoduyquocbao/xiangqi-rim.git"

# Lấy token HuggingFace đã đăng nhập trên máy
HF_TOKEN_VAL=""
if command -v python3 &>/dev/null; then
    HF_TOKEN_VAL=$(python3 -c "from huggingface_hub import get_token; print(get_token() or '')" 2>/dev/null || true)
fi

if [ -n "$HF_TOKEN_VAL" ]; then
    HF_SPACE_REPO="https://hoduyquocbao:${HF_TOKEN_VAL}@huggingface.co/spaces/hoduyquocbao/xiangqi-rim"
else
    HF_SPACE_REPO="https://huggingface.co/spaces/hoduyquocbao/xiangqi-rim"
fi

echo "============================================================"
echo "⚙️ CẤU HÌNH LOCAL DUAL-PUSH (GITHUB + HUGGINGFACE SPACE)"
echo "============================================================"

# Xóa các push URL cũ nếu có
git remote set-url --delete --push origin git@github.com:hoduyquocbao/xiangqi-rim.git 2>/dev/null || true
git remote set-url --delete --push origin "$GITHUB_REPO" 2>/dev/null || true
git remote set-url --delete --push origin "https://huggingface.co/spaces/hoduyquocbao/xiangqi-rim" 2>/dev/null || true

# Xóa bớt pattern URL chứa token cũ nếu có
for url in $(git remote get-url --all --push origin 2>/dev/null || true); do
    if [[ "$url" == *"huggingface.co"* ]]; then
        git remote set-url --delete --push origin "$url" 2>/dev/null || true
    fi
done

# Đăng ký lại 2 push URL HTTPS
git remote set-url --add --push origin "$GITHUB_REPO"
git remote set-url --add --push origin "$HF_SPACE_REPO"

echo "✅ Đã cấu hình Git Remote origin!"
echo "   Push URL 1 (GitHub): $GITHUB_REPO"
echo "   Push URL 2 (HF Space): https://huggingface.co/spaces/hoduyquocbao/xiangqi-rim (Auto Token Authenticated)"
echo ""
echo "🚀 Từ bây giờ, chỉ cần gõ: git push origin main"
echo "   Code sẽ tự động đẩy lên CẢ 2 NỀN TẢNG cùng một lúc!"
echo "============================================================"
