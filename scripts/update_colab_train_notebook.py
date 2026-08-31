# scripts/update_colab_train_notebook.py
# ==============================================================================
# CẬP NHẬT NOTEBOOK COLAB T4 GPU TRAINER VỚI KHO PLATINUM DATASET VÀ SHARDS
# ==============================================================================

import json
from pathlib import Path

path = Path("notebooks/colab_train_nnue_t4.ipynb")
if not path.exists():
    print(f"❌ Không tìm thấy tệp {path}")
    exit(1)

with open(path, "r", encoding="utf-8") as f:
    nb = json.load(f)

# Cập nhật cell 3 (Tải dataset) và cell 4 (Cấu hình)
for cell in nb.get("cells", []):
    source = "".join(cell.get("source", []))
    if "repos_to_scan" in source:
        new_source = [
            "# Cell 3: Tải & Tổng hợp Tập Dữ Liệu từ HuggingFace (Multi-Repo)\n",
            "print(\"=\" * 60)\n",
            "print(\" BƯỚC 3: TẢI & TỔNG HỢP TẬP DỮ LIỆU TỪ HUGGINGFACE\")\n",
            "print(\"=\" * 60)\n",
            "\n",
            "subprocess.run(\"pip install -q huggingface_hub\", shell=True, check=True)\n",
            "from huggingface_hub import HfApi, hf_hub_download\n",
            "\n",
            "repos_to_scan = [\n",
            "    \"hoduyquocbao/xiangqi-gen6-platinum-dataset\",\n",
            "    \"hoduyquocbao/xiangqi-nnue-dataset\",\n",
            "    \"hoduyquocbao/xiangqi-r1-dataset\"\n",
            "]\n",
            "api = HfApi()\n",
            "\n",
            "os.makedirs(\"data/raw\", exist_ok=True)\n",
            "local_files = []\n",
            "\n",
            "for repo_id in repos_to_scan:\n",
            "    try:\n",
            "        print(f\"🔍 Đang quét file dataset từ repo {repo_id}...\")\n",
            "        files = api.list_repo_files(repo_id=repo_id, repo_type=\"dataset\")\n",
            "        jsonl_files = [f for f in files if f.endswith(\".jsonl\")]\n",
            "        print(f\"  Phát hiện {len(jsonl_files)} file dữ liệu JSONL trong {repo_id}:\")\n",
            "        for f in jsonl_files:\n",
            "            print(f\"   - {f}\")\n",
            "            try:\n",
            "                target_path = hf_hub_download(repo_id=repo_id, filename=f, local_dir=\"data/raw\", repo_type=\"dataset\")\n",
            "                local_files.append(target_path)\n",
            "                size_mb = os.path.getsize(target_path) / (1024 * 1024)\n",
            "                print(f\"     ✅ Đã tải {f} ({size_mb:.1f} MB)\")\n",
            "            except Exception as e:\n",
            "                print(f\"     ⚠️ Không thể tải {f}: {e}\")\n",
            "    except Exception as err:\n",
            "        print(f\"  ⚠️ Lỗi khi quét repo {repo_id}: {err}\")\n",
            "\n",
            "# % % Cell 4: CẤU HÌNH HUẤN LUYỆN REAL-TIME FORM\n",
            "# @title ⚙️ CẤU HÌNH HUẤN LUYỆN NNUE REAL-TIME { display-mode: \"form\" }\n",
            "\n",
            "variable_epochs = 15 # @param {\"type\":\"slider\",\"min\":1,\"max\":50,\"step\":1}\n",
            "variable_batch_size = 16384 # @param {\"type\":\"slider\",\"min\":2048,\"max\":65536,\"step\":2048}\n",
            "variable_lr = 0.001 # @param {\"type\":\"number\"}\n",
            "variable_weight_decay = 1e-5 # @param {\"type\":\"number\"}\n",
            "variable_output_name = \"nnue_weights_gen11.bin\" # @param {\"type\":\"string\"}\n",
            "\n",
            "EPOCHS = int(variable_epochs)\n",
            "BATCH_SIZE = int(variable_batch_size)\n",
            "LR = float(variable_lr)\n",
            "WEIGHT_DECAY = float(variable_weight_decay)\n",
            "OUTPUT_NAME = str(variable_output_name).strip()\n",
            "OUTPUT_BIN = f\"data/{OUTPUT_NAME}\"\n",
            "\n",
            "print(\"=\" * 60)\n",
            "print(\" CẤU HÌNH HUẤN LUYỆN NNUE T4 GPU\")\n",
            "print(\"=\" * 60)\n",
            "print(f\"  EPOCHS       = {EPOCHS}\")\n",
            "print(f\"  BATCH_SIZE   = {BATCH_SIZE:,}\")\n",
            "print(f\"  LEARNING_RATE= {LR}\")\n",
            "print(f\"  WEIGHT_DECAY = {WEIGHT_DECAY}\")\n",
            "print(f\"  OUTPUT_BIN   = {OUTPUT_BIN}\")\n"
        ]
        cell["source"] = new_source

with open(path, "w", encoding="utf-8") as f:
    json.dump(nb, f, ensure_ascii=False, indent=1)

print(f"✅ Đã cập nhật thành công {path}!")
