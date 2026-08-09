#!/usr/bin/env python3
# scripts/hub.py
# ============================================================================
# MODULE HỢP NHẤT DỮ LIỆU HUGGINGFACE HUB KHÔNG PHÁ HỦY (NON-DESTRUCTIVE MERGER)
# ============================================================================
# Định danh đơn từ tiếng Anh: token, repo, filename, path, file, item, keys,
# key, raw, hash, remote, local, merged, seen, dup, added, tmp, card, retries,
# attempt, delay, fetch, verify, merge, save, push, draft, info, user, api, cause
# ============================================================================

import os
import json
import time
import hashlib
import glob
from typing import List, Dict, Tuple, Set
from huggingface_hub import HfApi, hf_hub_download
from huggingface_hub.utils import RepositoryNotFoundError, EntryNotFoundError, HfHubHTTPError

REPO = "hoduyquocbao/xiangqi-nnue-dataset"

def verify(item: Dict) -> bool:
    """Xác minh cấu trúc của mẫu cờ tư duy (Schema Validation hỗ trợ cả Legacy & Conversation format)."""
    if not isinstance(item, dict):
        return False
    if "messages" in item and isinstance(item.get("messages"), list):
        msgs = item["messages"]
        if len(msgs) == 3:
            # JRCP 2.0 Conversation format
            roles = [m.get("role") for m in msgs]
            return roles == ["system", "user", "assistant"]
        return len(msgs) > 0
    if "system" in item and "user" in item and "assistant" in item:
        return bool(item.get("system")) and bool(item.get("user")) and bool(item.get("assistant"))
    keys = {"prompt", "completion", "move", "stamp"}
    return keys.issubset(item.keys()) and bool(item.get("prompt")) and bool(item.get("move"))

def key(item: Dict) -> str:
    """Tạo mã băm SHA256 O(1) từ bộ khóa (FEN + move) cho JRCP 2.0 hoặc (prompt + move) cho legacy."""
    if "messages" in item and isinstance(item.get("messages"), list):
        msgs = item["messages"]
        if len(msgs) >= 2:
            user = msgs[1].get("content", "")
            move = item.get("move", "")
            # Trích xuất FEN từ user prompt
            fen = ""
            for segment in user.split("\n"):
                if "/" in segment and len(segment) > 15:
                    parts = segment.strip().split()
                    for part in parts:
                        if "/" in part and len(part) > 15:
                            fen = part
                            break
                    if fen:
                        break
            raw = f"{fen}||{move}"
            return hashlib.sha256(raw.encode("utf-8")).hexdigest()
    raw = f"{item.get('prompt') or item.get('user') or ''}||{item.get('move') or ''}"
    return hashlib.sha256(raw.encode("utf-8")).hexdigest()

def fetch(repo: str, token: str, filename: str = "train.jsonl") -> List[Dict]:
    """Tải dữ liệu lịch sử từ HuggingFace Hub nếu tồn tại.
    Chỉ trả về danh sách rỗng khi tệp hoặc repository chưa tồn tại (404).
    Nảy ngoại lệ dừng tiến trình khi gặp lỗi mạng, timeout hoặc hết token.
    """
    if not token:
        print("⚠️ Không có HF_TOKEN. Bỏ qua tải dữ liệu lịch sử từ Hub.")
        return []
        
    try:
        print(f"📥 Đang tải tệp lịch sử {filename} từ HuggingFace Hub ({repo})...")
        path = hf_hub_download(
            repo_id=repo,
            filename=filename,
            repo_type="dataset",
            token=token,
            force_download=True
        )
        samples = []
        with open(path, "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if line:
                    try:
                        item = json.loads(line)
                        if verify(item):
                            samples.append(item)
                    except Exception:
                        continue
        print(f"✅ Đã tải thành công {len(samples):,} mẫu dữ liệu lịch sử từ Hub!")
        return samples
    except Exception as err:
        cause = getattr(err, "__cause__", None) or err
        if isinstance(err, (RepositoryNotFoundError, EntryNotFoundError, FileNotFoundError)) or isinstance(cause, (RepositoryNotFoundError, EntryNotFoundError, FileNotFoundError)):
            print(f"ℹ️ Dataset chưa tồn tại trên Hub ({cause}). Khởi tạo dataset mới.")
            return []
        if (isinstance(err, HfHubHTTPError) and getattr(err, "response", None) is not None and err.response.status_code == 404) or (isinstance(cause, HfHubHTTPError) and getattr(cause, "response", None) is not None and cause.response.status_code == 404):
            print("ℹ️ Dataset chưa tồn tại trên Hub (HTTP 404). Khởi tạo dataset mới.")
            return []
        print(f"❌ Lỗi kết nối mạng hoặc xác thực khi tải từ Hub: {err}")
        raise err

def collect() -> List[Dict]:
    """Thu thập toàn bộ mẫu JRCP 2.0 từ thư mục data/."""
    samples = []
    patterns = [
        "data/jrcp2_elite_*.jsonl",
        "data/train.jsonl",
    ]
    
    seen_files = set()
    for pattern in patterns:
        for path in sorted(glob.glob(pattern)):
            if path in seen_files:
                continue
            seen_files.add(path)
            count = 0
            with open(path, "r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        item = json.loads(line)
                        if verify(item):
                            samples.append(item)
                            count += 1
                    except Exception:
                        continue
            print(f"  📄 {path}: {count:,} mẫu hợp lệ")
    
    print(f"  📊 Tổng cộng thu thập: {len(samples):,} mẫu cục bộ")
    return samples

def merge(remote: List[Dict], local: List[Dict]) -> Tuple[List[Dict], int]:
    """Hợp nhất dữ liệu remote và local, khử trùng lặp theo bộ khóa (FEN + move)."""
    merged = []
    seen: Set[str] = set()
    dup = 0
    
    # 1. Nạp remote (ưu tiên lịch sử)
    for item in remote:
        k = key(item)
        if k not in seen:
            seen.add(k)
            merged.append(item)
        else:
            dup += 1
            
    # 2. Nạp local (thêm mới)
    added = 0
    for item in local:
        k = key(item)
        if k not in seen:
            seen.add(k)
            merged.append(item)
            added += 1
        else:
            dup += 1
            
    print(f"📊 Kết quả Hợp nhất: Hub cũ={len(remote):,} | Mới thêm={added:,} | Bị trùng={dup:,} | Tổng={len(merged):,}")
    return merged, added

def card(total: int, phases: Dict[str, int], outcomes: Dict[str, int]) -> str:
    """Tạo README.md dataset card cho HuggingFace Hub."""
    return f"""---
license: mit
language:
- vi
- en
task_categories:
- reinforcement-learning
- text-generation
task_ids:
- conversational
- text2text-generation
tags:
- xiangqi
- chinese-chess
- chess-evaluation
- reasoning
- jrcp-3.0
size_categories:
- 100K<n<1M
configs:
- config_name: default
  data_files:
  - split: train
    path:
    - "*.jsonl"
    - "data/*.jsonl"
    - "community/*.jsonl"
---

# Xiangqi-R1 JRCP 2.0 Training Dataset

Bộ dữ liệu huấn luyện đẳng cấp cho mô hình LLM Xiangqi-R1 — AI Cờ Tướng suy luận 14 chiều kích.

## Thống Kê

| Chỉ số | Giá trị |
|---|---|
| Tổng mẫu | **{total:,}** |
| Opening | {phases.get('opening', 0):,} |
| Midgame | {phases.get('midgame', 0):,} |
| Endgame | {phases.get('endgame', 0):,} |
| Win | {outcomes.get('win', 0):,} |
| Loss | {outcomes.get('loss', 0):,} |
| Draw | {outcomes.get('draw', 0):,} |

## Định Dạng

Mỗi mẫu là 1 dòng JSON (JSONL) theo chuẩn JRCP 2.0 Conversation:

```json
{{
  "messages": [
    {{"role": "system", "content": "...JRCP 2.0 System Prompt..."}},
    {{"role": "user", "content": "...Ma trận 2D + FEN + PGN..."}},
    {{"role": "assistant", "content": "...JRCP 2.0 Structured Output JSON..."}}
  ],
  "move": "b2e2",
  "eval": 0,
  "outcome": "draw",
  "phase": "opening",
  "depth": 4,
  "nodes": 12345,
  "stamp": 1786172603
}}
```

## Nguồn Gốc

Dữ liệu được khai thác từ Native Rust Engine (xiangrust) depth=4 tự đấu,
với phân tích 14 chiều kích JRCP 2.0 cho mỗi vị trí bàn cờ.

## Giấy Phép

MIT License — Tự do sử dụng cho nghiên cứu và thương mại.
"""

def save(samples: List[Dict], readme: str):
    """Ghi dữ liệu nguyên tử (Atomic Write) ra tệp cục bộ qua tệp tạm .tmp."""
    os.makedirs("data", exist_ok=True)
    
    # Ghi train.jsonl nguyên tử (.tmp -> os.replace)
    tmp = "data/train.jsonl.tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        for s in samples:
            f.write(json.dumps(s, ensure_ascii=False) + "\n")
    os.replace(tmp, "data/train.jsonl")
    
    # Ghi README.md nguyên tử (.tmp -> os.replace)
    info = "data/README.md.tmp"
    with open(info, "w", encoding="utf-8") as f:
        f.write(readme)
    os.replace(info, "data/README.md")
    
    size_mb = os.path.getsize("data/train.jsonl") / (1024 * 1024)
    print(f"💾 Đã ghi {len(samples):,} mẫu ({size_mb:.1f} MB) vào data/train.jsonl")

def push(repo: str, token: str, retries: int = 3) -> bool:
    """Đẩy tệp đã hợp nhất lên HuggingFace Hub với cơ chế Retry Exponential Backoff."""
    if not token:
        print("⚠️ Thiếu HF_TOKEN. Dữ liệu đã hợp nhất được bảo tồn an toàn tại local data/train.jsonl.")
        return False
        
    api = HfApi(token=token)
    try:
        user = api.whoami()
        print(f"🔐 Đã xác thực tài khoản HuggingFace: {user.get('name')}")
    except Exception as err:
        print(f"❌ Lỗi xác thực token HuggingFace: {err}")
        return False

    # Tạo repo nếu chưa có
    try:
        api.create_repo(repo_id=repo, repo_type="dataset", exist_ok=True)
    except Exception:
        pass

    files = [
        ("data/train.jsonl", "train.jsonl"),
        ("data/README.md", "README.md"),
    ]
    
    for attempt in range(1, retries + 1):
        try:
            print(f"📤 Đang đẩy dataset lên Hub (Lần thử {attempt}/{retries})...")
            for local, remote in files:
                if os.path.exists(local):
                    api.upload_file(
                        path_or_fileobj=local,
                        path_in_repo=remote,
                        repo_id=repo,
                        repo_type="dataset"
                    )
            print(f"✅ Đã triển khai thành công dataset lên HuggingFace Hub ({repo})!")
            return True
        except Exception as err:
            print(f"⚠️ Lỗi kết nối Hub (Lần {attempt}): {err}")
            if attempt < retries:
                delay = 2 ** attempt
                print(f"⏳ Thử lại sau {delay} giây...")
                time.sleep(delay)
            else:
                print(f"❌ Không thể đẩy dataset sau {retries} lần thử. Dữ liệu vẫn được bảo toàn an toàn ở đĩa cục bộ.")
                return False


def main():
    """Hàm chính: Thu thập → Tải Hub → Hợp nhất → Lưu → Đẩy."""
    print("============================================================")
    print(" XIANGQI-R1 JRCP 2.0 DATASET HUB PUBLISHER                ")
    print("============================================================")
    
    token = os.environ.get("HF_TOKEN", "")
    repo = os.environ.get("HF_REPO", REPO)
    
    print(f"Repository: {repo}")
    print()
    
    # 1. Thu thập dữ liệu cục bộ
    print("[1] Thu thập dữ liệu JRCP 2.0 cục bộ...")
    local = collect()
    if not local:
        print("⚠️ Không tìm thấy dữ liệu cục bộ. Thoát.")
        return
    
    # 2. Tải dữ liệu lịch sử từ Hub
    print()
    print("[2] Tải dữ liệu lịch sử từ HuggingFace Hub...")
    remote = fetch(repo, token)
    
    # 3. Hợp nhất khử trùng lặp
    print()
    print("[3] Hợp nhất và khử trùng lặp...")
    merged, added = merge(remote, local)
    
    # 4. Thống kê phases & outcomes
    phases = {}
    outcomes = {}
    for item in merged:
        p = item.get("phase", "unknown")
        o = item.get("outcome", "unknown")
        phases[p] = phases.get(p, 0) + 1
        outcomes[o] = outcomes.get(o, 0) + 1
    
    print(f"  Phases: {phases}")
    print(f"  Outcomes: {outcomes}")
    
    # 5. Tạo README card
    readme = card(len(merged), phases, outcomes)
    
    # 6. Lưu cục bộ
    print()
    print("[4] Lưu dữ liệu hợp nhất...")
    save(merged, readme)
    
    # 7. Đẩy lên Hub
    print()
    print("[5] Đẩy lên HuggingFace Hub...")
    push(repo, token)
    
    print()
    print("============================================================")
    print(f"✅ HOÀN TẤT! Tổng: {len(merged):,} mẫu ({added:,} mới)")
    print("============================================================")


if __name__ == "__main__":
    main()
