import json

nb_path = 'community_colab.ipynb'
nb = json.load(open(nb_path))

# Patch Cell 3
cell3_code = """# 3. Launch Gradio Distributed Miner GUI (Non-blocking Web Interface)
from scripts.community_miner_gradio import create_ui

print("=" * 60)
print("🌐 BƯỚC 3: KHỞI CHẠY INTERACTIVE GRADIO MINER GUI")
print("=" * 60)
print("💡 Giao diện Web GUI sẽ chạy ở chế độ non-blocking.")
print("   Bạn có thể sử dụng Web GUI để khai thác dữ liệu cờ Tướng trực tiếp trên trình duyệt,")
print("   HOẶC tiếp tục chạy Cell 4 bên dưới để huấn luyện mô hình NNUE trên GPU T4.")

demo = create_ui()
demo.queue().launch(share=True, prevent_thread_lock=True)
print("🚀 Gradio Community Data Miner Launched Successfully!")
"""

nb['cells'][3]['source'] = [line + '\n' for line in cell3_code.split('\n')]
if nb['cells'][3]['source'][-1] == '\n':
    nb['cells'][3]['source'].pop()

# Patch Cell 4 dataset loading & tqdm in epoch loop
cell4_source = nb['cells'][4]['source']
cell4_text = ''.join(cell4_source)

# Replace dataset download part to check local mined data first
old_ds_code = """clean_dataset_path = hf_hub_download(
    repo_id=repo_id,
    filename="data/selfplay_samples_gen5.jsonl",
    repo_type="dataset",
    token=HF_TOKEN if HF_TOKEN else None
)"""

new_ds_code = """# Ưu tiên kiểm tra tệp dữ liệu vừa mine cục bộ trong data/ hoặc /content/
local_mined = [f for f in glob.glob("data/*.jsonl") + glob.glob("/content/*.jsonl") if os.path.exists(f) and os.path.getsize(f) > 0]
if local_mined:
    clean_dataset_path = local_mined[0]
    print(f"📁 Tìm thấy dữ liệu mine cục bộ: {clean_dataset_path} ({os.path.getsize(clean_dataset_path)/1024/1024:.1f} MB)")
else:
    print(f"📥 Tải dataset mới nhất từ HuggingFace Hub ({repo_id})...")
    clean_dataset_path = hf_hub_download(
        repo_id=repo_id,
        filename="data/selfplay_samples_gen5.jsonl",
        repo_type="dataset",
        token=HF_TOKEN if HF_TOKEN else None
    )"""

if old_ds_code in cell4_text:
    cell4_text = cell4_text.replace(old_ds_code, new_ds_code)

# Add tqdm import and wrap epoch loop
old_train_loop = """print(f"🔥 Training NNUE Model on GPU T4 ({EPOCHS} Epochs, Early Stopping patience={MAX_PATIENCE})...")
for epoch in range(1, EPOCHS + 1):"""

new_train_loop = """try:
    from tqdm.notebook import tqdm
except ImportError:
    from tqdm import tqdm

print(f"🔥 Training NNUE Model on GPU T4 ({EPOCHS} Epochs, Early Stopping patience={MAX_PATIENCE})...")
pbar = tqdm(range(1, EPOCHS + 1), desc="Training NNUE Epochs", unit="epoch")
for epoch in pbar:"""

if old_train_loop in cell4_text:
    cell4_text = cell4_text.replace(old_train_loop, new_train_loop)

# Update pbar description inside test eval block
old_eval_log = """print(f"  [GPU] Epoch {epoch:3d}/{EPOCHS:3d} | Train MSE: {train_mse:.6f} ({train_mae:.1f}cp) | Test MSE: {test_mse:.6f} ({test_mae:.1f}cp) {improved}")"""
new_eval_log = """pbar.set_postfix({"Train_MSE": f"{train_mse:.5f}", "Test_MSE": f"{test_mse:.5f}", "Test_MAE": f"{test_mae:.1f}cp"})
            print(f"  [GPU] Epoch {epoch:3d}/{EPOCHS:3d} | Train MSE: {train_mse:.6f} ({train_mae:.1f}cp) | Test MSE: {test_mse:.6f} ({test_mae:.1f}cp) {improved}")"""

if old_eval_log in cell4_text:
    cell4_text = cell4_text.replace(old_eval_log, new_eval_log)

nb['cells'][4]['source'] = [line + '\n' for line in cell4_text.split('\n')]
if nb['cells'][4]['source'][-1] == '\n':
    nb['cells'][4]['source'].pop()

json.dump(nb, open(nb_path, 'w'), indent=1, ensure_ascii=False)
print(f"✅ Patched {nb_path} successfully!")
