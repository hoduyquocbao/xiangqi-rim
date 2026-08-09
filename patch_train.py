import json

nb_path = 'train.ipynb'
nb = json.load(open(nb_path))

# Patch Cell 2: Token Loading
cell2_code = """# 2. Khai báo Token HuggingFace và Đăng nhập Hub (Ưu tiên Colab Secrets)
import os, sys, re, json, glob, torch
from huggingface_hub import login, HfApi
from unsloth import FastLanguageModel
from datasets import Dataset, load_dataset
from trl import GRPOTrainer, GRPOConfig

HF_TOKEN = None
# 1. Thử lấy từ Google Colab Secrets (userdata)
try:
    from google.colab import userdata
    HF_TOKEN = userdata.get('HF_TOKEN')
    if HF_TOKEN:
        print("🔑 Đã nạp HF_TOKEN từ Google Colab Secrets (userdata)!")
except Exception:
    pass

# 2. Thử lấy từ os.environ
if not HF_TOKEN:
    HF_TOKEN = os.environ.get("HF_TOKEN", "")
    if HF_TOKEN:
        print("🔑 Đã nạp HF_TOKEN từ os.environ!")

if HF_TOKEN:
    try:
        login(token=HF_TOKEN)
        print("✅ Đã đăng nhập HuggingFace Hub thành công!")
    except Exception as err:
        print(f"⚠️ Đăng nhập HuggingFace Hub thất bại: {err}")
else:
    print("⚠️ Không tìm thấy HF_TOKEN. Vui lòng cài Secret 'HF_TOKEN' trên Colab hoặc set os.environ.")
"""

nb['cells'][2]['source'] = [line + '\n' for line in cell2_code.split('\n')]
if nb['cells'][2]['source'][-1] == '\n':
    nb['cells'][2]['source'].pop()

# Patch Cell 7: Resume checkpoint
cell7_code = """# 7. Cấu hình GRPOTrainer Tốc Độ Siêu Tốc (FP16 Tensor Cores & Auto Resume Checkpoint)
output_dir = f"outputs/xiangqi-r1-{VARIANT}"
args = GRPOConfig(
    output_dir=output_dir,
    learning_rate=1e-5,
    adam_beta1=0.9,
    adam_beta2=0.99,
    weight_decay=0.1,
    warmup_steps=5,
    lr_scheduler_type="cosine",
    optim="adamw_8bit",
    fp16=True,
    bf16=False,
    logging_steps=1,
    per_device_train_batch_size=2,
    gradient_accumulation_steps=2,
    num_generations=4,
    max_prompt_length=512,
    max_completion_length=128,
    max_steps=200,
    save_steps=50,
    dataloader_num_workers=2,
    dataloader_pin_memory=True,
    report_to="none",
)

trainer = GRPOTrainer(
    model=model,
    processing_class=tokenizer,
    reward_funcs=[syntax, rule, quality],
    args=args,
    train_dataset=dataset,
)

# Tự động phát hiện & Khôi phục từ Checkpoint nếu có
resume_checkpoint = None
if os.path.exists(output_dir):
    checkpoints = sorted([d for d in os.listdir(output_dir) if d.startswith("checkpoint-")])
    if checkpoints:
        resume_checkpoint = os.path.join(output_dir, checkpoints[-1])
        print(f"🔄 Tìm thấy Checkpoint: {resume_checkpoint}. Tự động khôi phục quá trình huấn luyện!")

print("============================================================")
print(f"🚀 BẮT ĐẦU HUẤN LUYỆN GRPO XIANGQI-R1 ({VARIANT.upper()}) (FP16 TENSOR CORES)")
print("============================================================")
trainer.train(resume_from_checkpoint=resume_checkpoint)
"""

nb['cells'][7]['source'] = [line + '\n' for line in cell7_code.split('\n')]
if nb['cells'][7]['source'][-1] == '\n':
    nb['cells'][7]['source'].pop()

json.dump(nb, open(nb_path, 'w'), indent=1, ensure_ascii=False)
print(f"✅ Patched {nb_path} successfully!")
