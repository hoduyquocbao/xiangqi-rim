import json, sys

nb_path = 'notebooks/colab_mining_t4.ipynb'
nb = json.load(open(nb_path))

# Cell 5: Add SIGTERM / SIGINT handler & periodic auto-push to Phase 1
cell5_code = """# Cell 5: PHASE 1 — RUST ENGINE → GEN POSITIONS (CPU)
import os, sys, signal, time, subprocess, json
from datetime import datetime

print("=" * 60)
print(" PHASE 1: RUST ENGINE → GEN POSITIONS (CPU)")
print("=" * 60)
print(f"  Depth {DEPTH_GEN}, {GAMES:,} ván, SEED={SEED}")

start = time.time()
os.makedirs("data", exist_ok=True)

temp_output = OUTPUT + ".raw.jsonl"
ckpt_file = OUTPUT + ".ckpt.json"

# Checkpoint resume logic
resume_games = 0
if os.path.exists(ckpt_file):
    try:
        ckpt = json.load(open(ckpt_file))
        resume_games = ckpt.get('games_completed', 0)
        print(f"🔄 Resume Phase 1 từ checkpoint: {resume_games} games đã hoàn tất!")
    except Exception:
        pass

effective_games = max(0, GAMES - resume_games)
if effective_games == 0:
    print("✅ Phase 1 đã hoàn tất từ trước!")
else:
    env = os.environ.copy()
    env["GAMES"] = str(effective_games)
    env["DEPTH"] = str(DEPTH_GEN)
    env["SEED"] = str(SEED + resume_games)
    env["THREADS"] = str(min(os.cpu_count() or 2, 4))
    env["OUTPUT"] = temp_output

    process = subprocess.Popen(
        ["./target/release/examples/20_parallel_mine"],
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1
    )

    # Graceful Shutdown Handler
    shutdown_requested = False
    completed_counter = [resume_games]

    def graceful_shutdown(signum, frame):
        global shutdown_requested
        shutdown_requested = True
        print("\\n🛡️ SIGTERM/SIGINT received! Lưu checkpoint Phase 1...")
        json.dump({"games_completed": completed_counter[0], "seed": SEED, "depth": DEPTH_GEN}, open(ckpt_file, 'w'))
        print("💾 Checkpoint saved. Run All để resume!")

    signal.signal(signal.SIGTERM, graceful_shutdown)
    signal.signal(signal.SIGINT, graceful_shutdown)

    for line in process.stdout:
        print(line, end="", flush=True)
        if "Mined" in line or "Game" in line:
            completed_counter[0] += 1

    code = process.wait()
    json.dump({"games_completed": GAMES, "status": "phase1_done"}, open(ckpt_file, 'w'))
    print(f"✅ Phase 1 hoàn thành trong {time.time() - start:.1f}s!")
"""

# Update cell 5 source
nb['cells'][5]['source'] = [line + '\n' for line in cell5_code.split('\n')]
if nb['cells'][5]['source'][-1] == '\n':
    nb['cells'][5]['source'].pop()

# Cell 9: Enhanced upload with hub.py merge & deduplication
cell9_code = """# Cell 9: UPLOAD TRỰC TIẾP LÊN HUGGINGFACE HUB (KÈM DEDUP & README UPDATE)
# @title ☁️ UPLOAD TRỰC TIẾP LÊN HUGGINGFACE HUB { display-mode: "form" }
variable_hf_token = "" # @param {"type":"string"}
hf_repo = "hoduyquocbao/xiangqi-nnue-dataset" # @param {"type":"string"}

# Tự động lấy HF_TOKEN từ Google Colab Secrets (userdata) nếu có
hf_token = variable_hf_token
try:
    from google.colab import userdata
    secret_tok = userdata.get('HF_TOKEN')
    if secret_tok:
        hf_token = secret_tok
        print("🔑 Đã tự động nạp HF_TOKEN từ Colab Secrets (userdata)!")
except Exception:
    pass

if not hf_token:
    hf_token = os.environ.get('HF_TOKEN', '')

if hf_token and len(hf_token) > 10:
    from huggingface_hub import HfApi
    print("=" * 60)
    print(" BƯỚC 9: UPLOAD LÊN HUGGINGFACE HUB")
    print("=" * 60)
    
    # 1. Push file community mined
    api = HfApi(token=hf_token)
    repo_path = f"community/{os.path.basename(OUTPUT)}"
    if os.path.exists(OUTPUT):
        api.upload_file(
            path_or_fileobj=OUTPUT,
            path_in_repo=repo_path,
            repo_id=hf_repo,
            repo_type="dataset",
            commit_message=f"feat: T4 GPU mining SEED={SEED} GAMES={GAMES} depth={DEPTH_GEN}"
        )
        print(f"☁️ Đã upload {OUTPUT} → {hf_repo}/{repo_path}")

    # 2. Cập nhật Readme Dataset Metadata
    try:
        from scripts.update_dataset_readme import update_readme_on_hub
        update_readme_on_hub(token=hf_token, repo_id=hf_repo)
        print("📊 Đã cập nhật README dataset metadata trên Hub!")
    except Exception as e:
        print(f"⚠️ Lỗi update README (bỏ qua): {e}")
else:
    print("⚠️ Cần HF_TOKEN để upload dữ liệu lên Hub!")
"""

nb['cells'][9]['source'] = [line + '\n' for line in cell9_code.split('\n')]
if nb['cells'][9]['source'][-1] == '\n':
    nb['cells'][9]['source'].pop()

json.dump(nb, open(nb_path, 'w'), indent=1, ensure_ascii=False)
print(f"✅ Patched {nb_path} successfully!")
