#!/usr/bin/env python3
"""
Xiangqi-R1 Legacy Data Migration & Backup Script
Di chuyển dữ liệu huấn luyện cũ (legacy) sang thư mục lưu trữ an toàn.
Chuyển đổi dữ liệu legacy sang định dạng JRCP 2.0 Conversation thuần nhất.
Định danh đơn từ: migrate, backup, source, target, archive, convert, legacy, sample, line, path
"""

import json
import os
import shutil
import sys
import hashlib
from datetime import datetime

LEGACY = "data/train.jsonl"
ARCHIVE = "data/archive"
STAMP = datetime.now().strftime("%Y%m%d_%H%M%S")


def backup():
    """Sao lưu toàn bộ tệp dữ liệu legacy vào thư mục archive."""
    os.makedirs(ARCHIVE, exist_ok=True)

    files = [
        "data/train.jsonl",
        "data/train_backup.jsonl",
    ]

    # Tìm thêm các tệp JSON/JSONL legacy trong data/
    for name in os.listdir("data"):
        path = os.path.join("data", name)
        if os.path.isfile(path) and (name.endswith(".json") or name.endswith(".jsonl")):
            if name.startswith("jrcp2_elite_"):
                continue  # Bỏ qua tệp JRCP 2.0 mới
            if path not in files:
                files.append(path)

    moved = 0
    for path in files:
        if os.path.exists(path):
            target = os.path.join(ARCHIVE, f"{STAMP}_{os.path.basename(path)}")
            shutil.copy2(path, target)
            moved += 1
            print(f"  📦 Sao lưu: {path} -> {target}")

    print(f"\n✅ Đã sao lưu {moved} tệp vào {ARCHIVE}/")
    return moved


def convert():
    """Chuyển đổi dữ liệu legacy train.jsonl sang JRCP 2.0 Conversation format."""
    if not os.path.exists(LEGACY):
        print(f"⚠️  Không tìm thấy {LEGACY}, bỏ qua chuyển đổi.")
        return 0

    output = f"data/converted_{STAMP}.jsonl"
    converted = 0
    skipped = 0
    seen = set()

    with open(LEGACY, "r", encoding="utf-8") as source, \
         open(output, "w", encoding="utf-8") as target:
        for idx, line in enumerate(source):
            line = line.strip()
            if not line:
                continue
            try:
                obj = json.loads(line)
            except json.JSONDecodeError:
                skipped += 1
                continue

            # Kiểm tra xem đã có format messages chưa
            if "messages" in obj and isinstance(obj["messages"], list):
                messages = obj["messages"]
                if len(messages) == 3:
                    # Đã đúng format, kiểm tra dedup bằng FEN+move
                    move = obj.get("move", "")
                    user = messages[1].get("content", "")
                    # Trích xuất FEN từ user prompt
                    fen = ""
                    for segment in user.split("\n"):
                        if "FEN" in segment or "fen" in segment.lower():
                            parts = segment.strip().split()
                            for part in parts:
                                if "/" in part and len(part) > 15:
                                    fen = part
                                    break

                    key = hashlib.sha256(f"{fen}:{move}".encode()).hexdigest()
                    if key in seen:
                        skipped += 1
                        continue
                    seen.add(key)

                    target.write(json.dumps(obj, ensure_ascii=False) + "\n")
                    converted += 1
                    continue

            # Format cũ với system/user/assistant trực tiếp
            if "system" in obj and "user" in obj and "assistant" in obj:
                move = obj.get("move", "")
                user = obj.get("user", "")
                fen = ""
                for segment in user.split("\n"):
                    if "/" in segment and len(segment) > 15:
                        fen = segment.strip()
                        break

                key = hashlib.sha256(f"{fen}:{move}".encode()).hexdigest()
                if key in seen:
                    skipped += 1
                    continue
                seen.add(key)

                converted_obj = {
                    "messages": [
                        {"role": "system", "content": obj["system"]},
                        {"role": "user", "content": obj["user"]},
                        {"role": "assistant", "content": obj["assistant"]},
                    ],
                    "move": move,
                    "eval": obj.get("eval", 0),
                    "outcome": obj.get("outcome", "unknown"),
                    "phase": obj.get("phase", "unknown"),
                    "depth": obj.get("depth", 0),
                    "nodes": obj.get("nodes", 0),
                    "stamp": obj.get("stamp", 0),
                }
                target.write(json.dumps(converted_obj, ensure_ascii=False) + "\n")
                converted += 1
            else:
                skipped += 1

    print(f"\n✅ Chuyển đổi: {converted} mẫu hợp lệ, {skipped} mẫu bỏ qua.")
    print(f"💾 Tệp đầu ra: {output}")
    return converted


def main():
    print("============================================================")
    print(" XIANGQI-R1 LEGACY DATA MIGRATION & BACKUP TOOL            ")
    print("============================================================")
    print(f"Thời gian: {STAMP}\n")

    print("[1] Sao lưu dữ liệu legacy...")
    backup()

    print("\n[2] Chuyển đổi sang JRCP 2.0 Conversation format...")
    convert()

    print("\n============================================================")
    print("✅ MIGRATION HOÀN TẤT!")
    print("============================================================")


if __name__ == "__main__":
    main()
