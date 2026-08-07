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

    # 2. Tạo Model Repository 0.5B: hoduyquocbao/xiangqi-r1-0.5b (Siêu nhẹ, siêu nhanh)
    if create("xiangqi-r1-0.5b", "model"):
        model_05b_readme = """---
license: mit
pipeline_tag: text-generation
tags:
- xiangqi
- r1
- grpo
- qwen2.5-0.5b
- unsloth
library_name: transformers
---

# Xiangqi-R1 0.5B: Ultra-Fast Ultra-Lightweight Reasoner LLM for Chinese Chess

Mô hình AI Cờ Tướng siêu nhẹ 0.5B được huấn luyện bằng thuật toán GRPO dựa trên Qwen2.5-0.5B-Instruct.
Yêu cầu VRAM < 1.5GB, phản hồi cực nhanh (< 50ms/token), thích hợp chạy suy luận trực tiếp trên trình duyệt WebGPU/WASM hoặc GPU phổ thông.
"""
        commit("hoduyquocbao/xiangqi-r1-0.5b", "model", "README.md", model_05b_readme, "Initial 0.5B model README documentation")

    # 3. Tạo Model Repository 7B: hoduyquocbao/xiangqi-r1
    create("xiangqi-r1", "model")
