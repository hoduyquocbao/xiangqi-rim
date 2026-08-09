#!/usr/bin/env python3
"""
SYNC RELEASE SCRIPT: GitHub Dev -> HuggingFace Production
Tự động kéo bản release mới nhất từ GitHub Dev (hoduyquocbao/xiangqi-rim)
và đẩy (push) trực tiếp lên HuggingFace Production Space (hoduyquocbao/viper).
"""
import subprocess
import sys

def run(cmd):
    print(f"🚀 Running: {cmd}")
    res = subprocess.run(cmd, shell=True, text=True)
    if res.returncode != 0:
        print(f"❌ Command failed with code {res.returncode}")
        sys.exit(res.returncode)

def sync_release():
    print("==================================================")
    print("🔄 BẮT ĐẦU ĐỒNG BỘ RELEASE: GITHUB DEV ➡️ HUGGINGFACE PROD")
    print("==================================================")
    
    # 1. Fetch và reset theo bản release mới nhất từ GitHub Dev
    run("git fetch github main")
    run("git reset --hard github/main")
    
    # 2. Đảm bảo LFS tracking cho các tệp binary (.wasm)
    run("git lfs install")
    run("git lfs track '*.wasm'")
    run("git lfs track '*.bin'")
    
    # 3. Chuyển đổi .wasm sang LFS pointer nếu cần
    run("yes | git lfs migrate import --include='*.wasm' --everything || true")
    
    # 4. Đẩy trực tiếp lên HuggingFace Spaces Production
    run("git push hf main --force")
    
    print("==================================================")
    print("✅ ĐÃ HOÀN TẤT ĐỒNG BỘ RELEASE LÊN HUGGINGFACE PRODUCTION!")
    print("==================================================")

if __name__ == "__main__":
    sync_release()
