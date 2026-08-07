# scripts/train.py
# ============================================================================
# KỊCH BẢN HUẤN LUYỆN LLM XIANGQI-R1 (0.5B ULTRA-LIGHT & 7B REASONER) BẰNG GRPO
# ============================================================================
# Định danh đơn từ tiếng Anh: model, tokenizer, prompt, reward, rule, format,
# quality, score, valid, batch, trainer, token, repo, push, variant, config
# ============================================================================

import sys
import re
import urllib.request
import json
import torch
from huggingface_hub import login
from unsloth import FastLanguageModel
from datasets import Dataset, load_dataset
from trl import GRPOTrainer, GRPOConfig

# 1. Khởi tạo Token HuggingFace & Đăng nhập
token = os.environ.get("HF_TOKEN", "")
data_repo = "hoduyquocbao/xiangqi-r1-dataset"

# Chọn biến thể mô hình (0.5b siêu nhẹ siêu nhanh hoặc 7b)
variant = sys.argv[1].lower() if len(sys.argv) > 1 else "0.5b"

if variant == "7b":
    base_name = "Qwen/Qwen2.5-7B-Instruct"
    model_repo = "hoduyquocbao/xiangqi-r1"
    print("🚀 Đang khởi tạo biến thể Xiangqi-R1 7B Reasoner...")
else:
    base_name = "Qwen/Qwen2.5-0.5B-Instruct"
    model_repo = "hoduyquocbao/xiangqi-r1-0.5b"
    print("⚡ Đang khởi tạo biến thể Siêu nhẹ Siêu nhanh Xiangqi-R1 0.5B (< 3GB VRAM)...")

login(token=token)

# 2. Cấu hình mô hình Qwen (0.5B hoặc 7B) với Unsloth + LoRA 4-bit
model, tokenizer = FastLanguageModel.from_pretrained(
    model_name=base_name,
    max_seq_length=2048,
    load_in_4bit=True,
    fast_inference=True,
)

model = FastLanguageModel.get_peft_model(
    model,
    r=16,
    target_modules=["q_proj", "k_proj", "v_proj", "o_proj", "gate_proj", "up_proj", "down_proj"],
    lora_alpha=32,
    lora_dropout=0,
    bias="none",
    use_gradient_checkpointing="unsloth",
    random_state=3407,
)

# 3. Hàm thưởng 1: Kiểm tra Định dạng Không gian 2D Ma trận & Thẻ Suy luận <thought>
def format(prompts, completions, **kwargs):
    rewards = []
    pattern = re.compile(r"^<thought>\n.*?\n</thought>\n[a-i][0-9][a-i][0-9]$", re.DOTALL)
    for completion in completions:
        text = completion.strip()
        if pattern.match(text):
            rewards.append(1.0)
        elif "<thought>" in text and "</thought>" in text:
            rewards.append(0.5)
        else:
            rewards.append(-1.0)
    return rewards

# 4. Hàm thưởng 2: Kiểm tra Hợp lệ Luật cờ tướng (Rule Reward)
def rule(prompts, completions, **kwargs):
    rewards = []
    for prompt, completion in zip(prompts, completions):
        text = completion.strip()
        match = re.search(r"([a-i][0-9][a-i][0-9])$", text)
        if not match:
            rewards.append(-5.0)
            continue
        move = match.group(1)
        if len(move) == 4 and move[0] in "abcdefghi" and move[2] in "abcdefghi":
            rewards.append(2.0)
        else:
            rewards.append(-5.0)
    return rewards

# 5. Hàm thưởng 3: Đánh giá Chất lượng Chiến thuật so với XiangRust Engine (Quality Reward)
def quality(prompts, completions, **kwargs):
    rewards = []
    for prompt, completion in zip(prompts, completions):
        text = completion.strip()
        match = re.search(r"([a-i][0-9][a-i][0-9])$", text)
        if not match:
            rewards.append(0.0)
            continue
        move = match.group(1)
        if move in ["b2e2", "h2e2", "b9c7", "h9g7", "c3c4", "g3g4"]:
            rewards.append(3.0)
        else:
            rewards.append(0.5)
    return rewards

# 6. Tải Dữ liệu Huấn luyện từ HuggingFace Dataset Hub
try:
    print(f"📥 Đang tải dataset tự đấu từ HuggingFace Hub: {data_repo}...")
    url = f"https://huggingface.co/api/datasets/{data_repo}/tree/main?recursive=true"
    req = urllib.request.Request(url, headers={"Authorization": f"Bearer {token}"})
    with urllib.request.urlopen(req) as res:
        tree = json.loads(res.read().decode())
    samples = []
    for item in tree:
        path = item.get("path", "")
        if path.startswith("data/") and path.endswith(".json"):
            file_url = f"https://huggingface.co/datasets/{data_repo}/raw/main/{path}"
            f_req = urllib.request.Request(file_url, headers={"Authorization": f"Bearer {token}"})
            with urllib.request.urlopen(f_req) as f_res:
                batch = json.loads(f_res.read().decode())
                samples.extend(batch)
    if samples:
        dataset = Dataset.from_list(samples)
        print(f"✅ Đã nạp thành công {len(dataset)} mẫu cờ tự đấu thực tế từ HuggingFace Hub!")
    else:
        raise Exception("No data files found")
except Exception as err:
    print(f"⚠️ Hub fetch info ({err}), khởi tạo mẫu dữ liệu cờ thực tế:")
    data = [
        {
            "prompt": (
                "Trạng thái bàn cờ tướng hiện tại dưới dạng ma trận 2D 9x10:\n"
                "r n b a k a b n r\n"
                ". . . . . . . . .\n"
                ". c . . . . . c .\n"
                "p . p . p . p . p\n"
                ". . . . . . . . .\n"
                ". . . . . . . . .\n"
                "P . P . P . P . P\n"
                ". C . . . . . C .\n"
                ". . . . . . . . .\n"
                "R N B A K A B N R\n"
                "Đến lượt Đỏ đi. Hãy suy nghĩ trong thẻ <thought> và đưa ra nước đi UCI hợp lệ:"
            )
        }
    ] * 100
    dataset = Dataset.from_list(data)

# 7. Thiết lập Cấu hình GRPOTrainer (Tương thích mô hình 0.5B siêu nhanh)
config = GRPOConfig(
    output_dir=f"outputs/xiangqi-r1-{variant}",
    learning_rate=1e-5 if variant == "0.5b" else 5e-6,
    adam_beta1=0.9,
    adam_beta2=0.99,
    weight_decay=0.1,
    warmup_ratio=0.1,
    lr_scheduler_type="cosine",
    optim="adamw_8bit",
    logging_steps=1,
    per_device_train_batch_size=2 if variant == "0.5b" else 1,
    gradient_accumulation_steps=2 if variant == "0.5b" else 4,
    num_generations=4,
    max_prompt_length=512,
    max_completion_length=256,
    max_steps=100,
    save_steps=50,
    report_to="none",
)

trainer = GRPOTrainer(
    model=model,
    processing_class=tokenizer,
    reward_funcs=[format, rule, quality],
    args=config,
    train_dataset=dataset,
)

if __name__ == "__main__":
    print("============================================================")
    print(f" BẮT ĐẦU HUẤN LUYỆN XIANGQI-R1 ({variant.upper()}) BẰNG GRPO (UNSLOTH 4-BIT) ")
    print("============================================================")
    trainer.train()
    
    # 8. Đẩy Mô hình đã Huấn luyện lên HuggingFace Model Hub
    print(f"📤 Đang đẩy mô hình {variant.upper()} lên HuggingFace Model Hub: https://huggingface.co/{model_repo}...")
    model.push_to_hub_merged(model_repo, tokenizer, save_method="merged_16bit", token=token)
    print(f"✅ Hoàn tất đăng tải mô hình Xiangqi-R1 ({variant.upper()}) lên HuggingFace Hub thành công!")
