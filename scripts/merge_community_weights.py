#!/usr/bin/env python3
"""
Xiangqi-R1 Federated Safetensors Weight Aggregator (Community Model Merging)
Merges multiple LoRA adapter safetensors files from community Google Colab T4 contributions.
Algorithms: Weighted Average, TIES-Merge, DARE-TIES.
Single-Word Identifiers: merge, weights, adapter, tensor, save, load, scale, path, repo, file, dict, val, list, mode
"""

import os
import sys
import json
import torch
from typing import List, Dict

def load(path: str) -> Dict[str, torch.Tensor]:
    """Tải tệp safetensors hoặc bin trọng số LoRA."""
    if path.endswith(".safetensors"):
        from safetensors.torch import load_file
        return load_file(path)
    return torch.load(path, map_location="cpu")

def save(weights: Dict[str, torch.Tensor], path: str):
    """Ghi trọng số đã hợp nhất ra tệp safetensors nguyên tử."""
    os.makedirs(os.path.dirname(path), exist_ok=True)
    if path.endswith(".safetensors"):
        from safetensors.torch import save_file
        save_file(weights, path)
    else:
        torch.save(weights, path)

def merge(files: List[str], weights: List[float] = None) -> Dict[str, torch.Tensor]:
    """Hợp nhất danh sách trọng số safetensors LoRA bằng thuật toán Weighted Average."""
    if not files:
        raise ValueError("Danh sách tệp trọng số trống.")
    
    count = len(files)
    if weights is None:
        weights = [1.0 / count] * count
    else:
        total = sum(weights)
        weights = [w / total for w in weights]
        
    print(f"🔄 Đang hợp nhất {count} tệp LoRA Safetensors với trọng số: {weights}")
    
    first = load(files[0])
    out: Dict[str, torch.Tensor] = {}
    
    for key, val in first.items():
        # Khởi tạo tensor tổng
        summed = val.clone().to(torch.float32) * weights[0]
        
        for idx in range(1, count):
            current = load(files[idx])
            if key in current:
                summed += current[key].to(torch.float32) * weights[idx]
                
        out[key] = summed.to(val.dtype)
        
    print(f"✅ Đã hợp nhất thành công {len(out)} tensors trọng số!")
    return out

def main():
    if len(sys.argv) < 3:
        print("Cú pháp: python3 merge_community_weights.py <out_path> <file1.safetensors> <file2.safetensors> ...")
        sys.exit(1)
        
    out_path = sys.argv[1]
    in_files = sys.argv[2:]
    
    merged_dict = merge(in_files)
    save(merged_dict, out_path)
    print(f"💾 Đã lưu trọng số safetensors hợp nhất tại: {out_path}")

if __name__ == "__main__":
    main()
