# scripts/share.py
# ============================================================================
# TỰ ĐỘNG KHAI THÁC DỮ LIỆU CỜ TƯỚNG THỰC TẾ TỪ RUST ENGINE VÀ ĐẨY HUGGINGFACE
# ============================================================================
# Định danh đơn từ tiếng Anh: token, repo, cmd, proc, path, file, data,
# content, encoded, payload, req, res, err, mine, loop, count, push, delay
# ============================================================================

import subprocess
import urllib.request
import json
import base64
import glob
import os

token = os.environ.get("HF_TOKEN", "")
repo = "hoduyquocbao/xiangqi-r1-dataset"

def push():
    print("============================================================")
    print(" KHAI THÁC DỮ LIỆU CỜ THỰC TẾ TỪ RUST ENGINE & ĐẨY HUGGINGFACE ")
    print("============================================================")
    
    print("🚀 Đang khởi chạy Native Rust Engine Self-Play...")
    cmd = ["cargo", "run", "--release", "--example", "17_mine_dataset"]
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True, check=True)
        print(" Output từ Rust Engine:")
        print(proc.stdout)
    except Exception as err:
        print(f"❌ Lỗi chạy Rust Engine: {err}")
        return False

    files = sorted(glob.glob("data/real_mined_*.json"), key=os.path.getmtime, reverse=True)
    if not files:
        print("⚠️ Không tìm thấy tệp real_mined_*.json nào trong data/")
        return False

    latest_file = files[0]
    print(f"📦 Đang đẩy tệp thực tế: {latest_file}...")

    with open(latest_file, "r", encoding="utf-8") as f:
        content = f.read()

    encoded = base64.b64encode(content.encode("utf-8")).decode("utf-8")
    rel_path = f"data/{os.path.basename(latest_file)}"
    url = f"https://huggingface.co/api/datasets/{repo}/commit/main"
    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json"
    }
    payload = json.dumps({
        "summary": f"Native Rust Engine self-play dataset push: {os.path.basename(latest_file)}",
        "operations": [
            {
                "action": "create",
                "path": rel_path,
                "encoding": "base64",
                "content": encoded
            }
        ]
    }).encode("utf-8")

    try:
        req = urllib.request.Request(url, data=payload, headers=headers, method="POST")
        with urllib.request.urlopen(req) as res:
            print(f"✅ ĐÃ ĐẨY THÀNH CÔNG DỮ LIỆU CỜ THỰC TẾ LÊN HUGGINGFACE HUB!")
            print(f"🔗 File: https://huggingface.co/datasets/{repo}/blob/main/{rel_path}")
            return True
    except Exception as err:
        print(f"❌ LỖI ĐẨY HUGGINGFACE HUB: {err}")
        return False

if __name__ == "__main__":
    push()
