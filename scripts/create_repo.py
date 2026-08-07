# scripts/create_repo.py
# ============================================================================
# TỰ ĐỘNG TẠO VÀ KHỞI TẠO REPOSITORIES DATASETS & MODEL (0.5B & 7B) TRÊN HUGGINGFACE HUB
# ============================================================================
# Định danh đơn từ tiếng Anh: token, name, type, data, req, res, code, create, commit, base
# ============================================================================

import urllib.request
import json
import base64

import os

token = os.environ.get("HF_TOKEN", "")

def create(name, repo_type):
    url = "https://huggingface.co/api/repos/create"
    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json"
    }
    payload = json.dumps({"name": name, "type": repo_type, "private": False}).encode("utf-8")
    
    try:
        req = urllib.request.Request(url, data=payload, headers=headers, method="POST")
        with urllib.request.urlopen(req) as res:
            result = json.loads(res.read().decode("utf-8"))
            print(f"✅ ĐÃ TẠO THÀNH CÔNG {repo_type.upper()} REPOSITORY: {result.get('url')}")
            return True
    except urllib.error.HTTPError as err:
        if err.code == 409:
            print(f"ℹ️ REPOSITORY {name} ({repo_type}) ĐÃ TỒN TẠI TỪ TRƯỚC TRÊN HUGGINGFACE HUB.")
            return True
        else:
            print(f"❌ LỖI KHI TẠO {name} ({repo_type}): {err.code} - {err.read().decode('utf-8')}")
            return False
    except Exception as err:
        print(f"❌ LỖI KẾT NỐI: {err}")
        return False

def commit(repo_id, repo_type, path, content, summary):
    prefix = "datasets" if repo_type == "dataset" else "models"
    url = f"https://huggingface.co/api/{prefix}/{repo_id}/commit/main"
    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json"
    }
    encoded = base64.b64encode(content.encode("utf-8")).decode("utf-8")
    payload = json.dumps({
        "summary": summary,
        "operations": [
            {
                "action": "create",
                "path": path,
                "encoding": "base64",
                "content": encoded
            }
        ]
    }).encode("utf-8")

    try:
        req = urllib.request.Request(url, data=payload, headers=headers, method="POST")
        with urllib.request.urlopen(req) as res:
            print(f"📄 Đã khởi tạo {path} cho {repo_id} ({repo_type}) thành công!")
            return True
    except Exception as err:
        print(f"⚠️ Commit {path} ({repo_id}): {err}")
        return False

if __name__ == "__main__":
    print("============================================================")
    print(" KHỞI TẠO TỰ ĐỘNG HUGGINGFACE REPOSITORIES CHO XIANGQI-R1 ")
    print("============================================================")
    
    # 1. Tạo Dataset Repository: hoduyquocbao/xiangqi-r1-dataset
    create("xiangqi-r1-dataset", "dataset")

    # 2. Tạo Model Repository 0.8B: hoduyquocbao/xiangqi-r1-0.8b (Qwen 3.5 0.8B hiện đại)
    if create("xiangqi-r1-0.8b", "model"):
        model_08b_readme = """---
license: mit
pipeline_tag: text-generation
tags:
- xiangqi
- r1
- grpo
- qwen3.5-0.8b
- unsloth
library_name: transformers
---

# Xiangqi-R1 0.8B: Qwen 3.5 0.8B Deep Reasoning Model for Chinese Chess

Mô hình AI Cờ Tướng hiện đại 0.8B được huấn luyện từ Qwen 3.5 0.8B bằng thuật toán GRPO với tập dữ liệu tự đấu đa chiều 3-in-1 (Ma trận 2D + Chuỗi FEN + Lịch sử PGN).
Yêu cầu VRAM < 1.8GB, thời gian phản hồi cực nhanh (< 1s), đáp ứng tốt cả thiết bị di động và GPU phổ thông.
"""
        commit("hoduyquocbao/xiangqi-r1-0.8b", "model", "README.md", model_08b_readme, "Initial 0.8B model README documentation")

    # 3. Tạo Model Repository 0.5B: hoduyquocbao/xiangqi-r1-0.5b
    create("xiangqi-r1-0.5b", "model")

    # 4. Tạo Model Repository 7B: hoduyquocbao/xiangqi-r1
    create("xiangqi-r1", "model")
