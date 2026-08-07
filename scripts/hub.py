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
from typing import List, Dict, Tuple, Set
from huggingface_hub import HfApi, hf_hub_download
from huggingface_hub.utils import RepositoryNotFoundError, EntryNotFoundError, HfHubHTTPError

def verify(item: Dict) -> bool:
    """Xác minh cấu trúc của mẫu cờ tư duy (Schema Validation)."""
    if not isinstance(item, dict):
        return False
    keys = {"prompt", "completion", "move", "stamp"}
    return keys.issubset(item.keys()) and bool(item.get("prompt")) and bool(item.get("move"))

def key(item: Dict) -> str:
    """Tạo mã băm SHA256 O(1) từ bộ khóa (prompt, move) để tiết kiệm bộ nhớ RAM."""
    raw = f"{item.get('prompt', '')}||{item.get('move', '')}"
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

def merge(remote: List[Dict], local: List[Dict]) -> Tuple[List[Dict], int]:
    """Hợp nhất dữ liệu remote và local, khử trùng lặp theo bộ khóa (prompt, move)."""
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

def save(samples: List[Dict], card: str):
    """Ghi dữ liệu nguyên tử (Atomic Write) ra tệp cục bộ qua tệp tạm .tmp."""
    os.makedirs("data", exist_ok=True)
    
    # Ghi train.jsonl nguyên tử (.tmp -> os.replace)
    tmp = "data/train.jsonl.tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        for s in samples:
            f.write(json.dumps(s, ensure_ascii=False) + "\n")
    os.replace(tmp, "data/train.jsonl")
    
    # Ghi train.json nguyên tử (.tmp -> os.replace)
    draft = "data/train.json.tmp"
    with open(draft, "w", encoding="utf-8") as f:
        json.dump(samples, f, ensure_ascii=False, indent=2)
    os.replace(draft, "data/train.json")
    
    # Ghi README.md nguyên tử (.tmp -> os.replace)
    info = "data/README.md.tmp"
    with open(info, "w", encoding="utf-8") as f:
        f.write(card)
    os.replace(info, "data/README.md")

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

    files = [
        ("data/train.jsonl", "train.jsonl"),
        ("data/train.json", "train.json"),
        ("data/README.md", "README.md")
    ]
    
    for attempt in range(1, retries + 1):
        try:
            print(f"📤 Đang đẩy dataset lên Hub (Lần thử {attempt}/{retries})...")
            for local, remote in files:
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
