import json

nb_path = 'train_community.ipynb'
nb = json.load(open(nb_path))

cell3_code = """# 3. Reward Functions & Huấn Luyện 150 Steps GRPO (Tích hợp Resume & Rule Validation)
import os, re, json as _json

def reward_format(completions, **kwargs):
    \"\"\"Thưởng completion có chứa JSON hợp lệ với key 'bestmove'.\"\"\"
    rewards = []
    for text in completions:
        score = 0.0
        try:
            match = re.search(r'\\{[^}]+\\}', text)
            if match:
                obj = _json.loads(match.group())
                if 'bestmove' in obj:
                    score += 0.5
                    move = obj['bestmove']
                    if isinstance(move, str) and re.match(r'^[a-i][0-9][a-i][0-9]$', move):
                        score += 0.5
        except Exception:
            pass
        rewards.append(score)
    return rewards

def reward_rule(prompts, completions, **kwargs):
    \"\"\"Thưởng completion có nước đi hợp lệ theo luật cờ Tướng bàn 9x10.\"\"\"
    rewards = []
    for text in completions:
        score = 0.0
        try:
            match = re.search(r'\\{[^}]+\\}', text)
            if match:
                obj = _json.loads(match.group())
                move = obj.get('bestmove', '')
                if isinstance(move, str) and re.match(r'^[a-i][0-9][a-i][0-9]$', move):
                    # Kiểm tra ô xuất phát != ô đích
                    if move[:2] != move[2:]:
                        score += 0.5
                    # Kiểm tra tọa độ nằm trong bàn cờ 9x10
                    fc, fr = ord(move[0]) - ord('a'), int(move[1])
                    tc, tr = ord(move[2]) - ord('a'), int(move[3])
                    if 0 <= fc <= 8 and 0 <= fr <= 9 and 0 <= tc <= 8 and 0 <= tr <= 9:
                        score += 0.5
        except Exception:
            pass
        rewards.append(score)
    return rewards

def reward_thought(completions, **kwargs):
    \"\"\"Thưởng completion có thẻ <thought> dài và chi tiết.\"\"\"
    rewards = []
    for text in completions:
        score = 0.0
        if '<thought>' in text and '</thought>' in text:
            thought = text.split('<thought>')[1].split('</thought>')[0]
            score = min(len(thought) / 200.0, 1.0)
        rewards.append(score)
    return rewards

output_dir = "output_community"
args = GRPOConfig(
    output_dir=output_dir,
    learning_rate=5e-6,
    adam_beta1=0.9,
    adam_beta2=0.99,
    weight_decay=0.1,
    warmup_ratio=0.1,
    lr_scheduler_type="cosine",
    optim="adamw_8bit",
    logging_steps=10,
    max_steps=150,
    save_steps=50,
    max_prompt_length=512,
    max_completion_length=256,
    num_generations=4,
    report_to="none"
)

trainer = GRPOTrainer(
    model=model,
    processing_class=tokenizer,
    reward_funcs=[reward_format, reward_thought, reward_rule],
    args=args,
    train_dataset=ds
)

# Kiểm tra & Tự động khôi phục từ Checkpoint nếu có
resume_checkpoint = None
if os.path.exists(output_dir):
    checkpoints = sorted([d for d in os.listdir(output_dir) if d.startswith('checkpoint-')])
    if checkpoints:
        resume_checkpoint = os.path.join(output_dir, checkpoints[-1])
        print(f"🔄 Tự động khôi phục huấn luyện từ Checkpoint: {resume_checkpoint}")

print("🔥 Bắt đầu phiên huấn luyện cộng đồng Colab T4 (150 Steps GRPO)...")
trainer.train(resume_from_checkpoint=resume_checkpoint)

model.save_lora("community_adapter")
print("💾 Đã xuất tệp adapter_model.safetensors thành công tại community_adapter/!")
"""

nb['cells'][3]['source'] = [line + '\n' for line in cell3_code.split('\n')]
if nb['cells'][3]['source'][-1] == '\n':
    nb['cells'][3]['source'].pop()

json.dump(nb, open(nb_path, 'w'), indent=1, ensure_ascii=False)
print(f"✅ Patched {nb_path} successfully!")
