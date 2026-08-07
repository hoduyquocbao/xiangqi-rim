# scripts/train.py
# ============================================================================
# KỊCH BẢN HUẤN LUYỆN LLM XIANGQI-R1 BẰNG ALGORITHM GRPO (TESLA T4 FP16 TENSOR CORES)
# ============================================================================
# Định danh đơn từ tiếng Anh: model, tokenizer, prompt, prompts, completion,
# completions, reward, rewards, rule, syntax, quality, score, valid, batch,
# trainer, token, repo, push, variant, config, base, target, files, args,
# dataset, data, parse, fen, grid, active, rows, row, line, scol, srank,
# tcol, trank, srow, trow, piece, kind, ground, idx, match, text, move, stamp,
# path, output, err
# ============================================================================

import os
import sys
import re
import glob

try:
    import torch
    from unsloth import FastLanguageModel
    from datasets import Dataset, load_dataset
    from trl import GRPOTrainer, GRPOConfig
    from huggingface_hub import login
except ImportError:
    torch = None
    FastLanguageModel = None
    Dataset = None
    load_dataset = None
    GRPOTrainer = None
    GRPOConfig = None
    login = None

# 1. Khởi tạo Token HuggingFace & Đăng nhập Hub
token = os.environ.get("HF_TOKEN", "")
repo = "hoduyquocbao/xiangqi-r1-dataset"

# Chọn biến thể mô hình (0.5b, 0.8b, hoặc 7b)
variant = sys.argv[1].lower() if len(sys.argv) > 1 else "0.5b"

if variant == "7b":
    base = "Qwen/Qwen2.5-7B-Instruct"
    target = "hoduyquocbao/xiangqi-r1"
    print("🚀 Đang khởi tạo biến thể Xiangqi-R1 7B Reasoner...")
elif variant == "0.8b":
    base = "hoduyquocbao/xiangqi-r1-0.8b"
    target = "hoduyquocbao/xiangqi-r1-0.8b"
    print("⚡ Đang khởi tạo biến thể Xiangqi-R1 0.8B Model...")
else:
    base = "Qwen/Qwen2.5-Coder-0.5B-Instruct"
    target = "hoduyquocbao/xiangqi-r1-0.5b"
    print("⚡ Đang khởi tạo mô hình Coder Xiangqi-R1 Qwen 2.5 Coder 0.5B (FP16 Tensor Cores)...")

if token and login is not None:
    try:
        login(token=token)
        print("✅ Đã đăng nhập HuggingFace Hub thành công!")
    except Exception as err:
        print(f"⚠️ Đăng nhập HuggingFace Hub thất bại: {err}")

# 2. Định nghĩa Hằng số Biểu thức Chính quy Module (Từ đơn tiếng Anh)
FORMAT = re.compile(r"^\s*<(thought|think)>.*?<\/(\1)>\s*\n?\s*([a-i][0-9][a-i][0-9])\s*$", re.DOTALL)
MOVE = re.compile(r"([a-i][0-9][a-i][0-9])")
FEN = re.compile(r"2\. Chuỗi Chuẩn FEN.*?:?\n([a-zA-Z0-9/]+\s+[wb]\s+-\s+-\s+\d+\s+\d+)")

def parse(fen):
    """Giải mã FEN thành ma trận 2D và bên đến lượt đi ('w' hoặc 'b')."""
    if not isinstance(fen, str):
        return None, None
    parts = fen.split()
    if len(parts) < 2:
        return None, None
    board = parts[0]
    active = parts[1]
    rows = board.split('/')
    if len(rows) != 10:
        return None, None
    grid = []
    for row in rows:
        line = []
        for ch in row:
            if ch.isdigit():
                line.extend(['.'] * int(ch))
            else:
                line.append(ch)
        if len(line) != 9:
            return None, None
        grid.append(line)
    return grid, active

def valid(fen, move):
    """Kiểm tra tính hợp lệ về mặt luật cờ của nước đi dựa trên FEN hiện tại."""
    if not isinstance(move, str) or len(move) != 4:
        return False
    if not (move[1].isdigit() and move[3].isdigit()):
        return False
    grid, active = parse(fen)
    if not grid or not active:
        return False
    scol = ord(move[0]) - ord('a')
    srank = int(move[1])
    tcol = ord(move[2]) - ord('a')
    trank = int(move[3])
    if not (0 <= scol <= 8 and 0 <= tcol <= 8 and 0 <= srank <= 9 and 0 <= trank <= 9):
        return False
    if scol == tcol and srank == trank:
        return False
    srow = 9 - srank
    trow = 9 - trank
    piece = grid[srow][scol]
    if piece in ('.', ' '):
        return False
    if (active == 'w' and not piece.isupper()) or (active == 'b' and not piece.islower()):
        return False
    target = grid[trow][tcol]
    if target not in ('.', ' '):
        if (piece.isupper() and target.isupper()) or (piece.islower() and target.islower()):
            return False
    kind = piece.upper()
    if kind in ('K', 'A'):
        if not (3 <= tcol <= 5):
            return False
        if piece.isupper() and not (0 <= trank <= 2):
            return False
        if piece.islower() and not (7 <= trank <= 9):
            return False
    elif kind == 'B':
        if piece.isupper() and trank > 4:
            return False
        if piece.islower() and trank < 5:
            return False
    elif kind == 'P':
        if piece.isupper():
            if trank < srank:
                return False
            if srank < 5 and (tcol != scol or trank <= srank):
                return False
        else:
            if trank > srank:
                return False
            if srank > 4 and (tcol != scol or trank >= srank):
                return False
    return True

# 3. Hàm thưởng 1: Kiểm tra Cú pháp Thẻ Suy luận <thought>/<think> & Nước đi UCI
def syntax(prompts, completions, **kwargs):
    rewards = []
    for completion in completions:
        text = completion.strip()
        if FORMAT.match(text):
            rewards.append(1.0)
        elif ("<thought>" in text and "</thought>" in text) or ("<think>" in text and "</think>" in text):
            if MOVE.search(text):
                rewards.append(0.5)
            else:
                rewards.append(0.0)
        else:
            rewards.append(-1.0)
    return rewards

# 4. Hàm thưởng 2: Kiểm tra Hợp lệ Luật cờ tướng dựa trên FEN (Rule Reward)
def rule(prompts, completions, **kwargs):
    rewards = []
    for prompt, completion in zip(prompts, completions):
        text = completion.strip()
        match = MOVE.search(text)
        if not match:
            rewards.append(-0.5)
            continue
        move = match.group(1)
        matched = FEN.search(prompt)
        if matched:
            fen = matched.group(1)
            if valid(fen, move):
                rewards.append(2.0)
            else:
                rewards.append(-0.5)
        else:
            if len(move) == 4 and move[0] in "abcdefghi" and move[2] in "abcdefghi":
                rewards.append(1.0)
            else:
                rewards.append(-0.5)
    return rewards

# 5. Hàm thưởng 3: Đánh giá Chất lượng Chiến thuật so với Ground Truth / Engine (Quality Reward)
def quality(prompts, completions, **kwargs):
    rewards = []
    grounds = kwargs.get("move", None)
    for idx, (prompt, completion) in enumerate(zip(prompts, completions)):
        text = completion.strip()
        match = MOVE.search(text)
        if not match:
            rewards.append(0.0)
            continue
        move = match.group(1)
        ground = grounds[idx] if grounds and idx < len(grounds) else None
        if ground and move == ground:
            rewards.append(3.0)
        elif move in ["b2e2", "h2e2", "b9c7", "h9g7", "c3c4", "g3g4"]:
            rewards.append(1.5)
        else:
            rewards.append(0.5)
    return rewards

def main():
    """Hàm chính khởi chạy tiến trình huấn luyện GRPO khi có đầy đủ thư viện ML."""
    if FastLanguageModel is None or GRPOTrainer is None:
        print("⚠️ Thư viện unsloth/trl chưa được cài đặt trong môi trường này. Bỏ qua bước huấn luyện GPU.")
        return

    # Cấu hình mô hình Qwen với Unsloth + LoRA 4-bit (Tắt fast_inference để tránh crash vLLM)
    model, tokenizer = FastLanguageModel.from_pretrained(
        model_name=base,
        max_seq_length=1024,
        load_in_4bit=True,
        fast_inference=False,
    )

    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token
    tokenizer.padding_side = "right"

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

    # Tải Dữ liệu Huấn luyện từ HuggingFace Dataset Hub (Hoặc Fallback Cục Bộ Hợp Nhất)
    try:
        print(f"📥 Đang tải dataset cờ tự đấu 3-in-1 từ HuggingFace Hub: {repo}...")
        dataset = load_dataset(repo, split="train")
        print(f"✅ Đã nạp thành công {len(dataset)} mẫu cờ tư duy sâu thực tế từ HuggingFace Hub!")
    except Exception as err:
        print(f"⚠️ Không thể tải từ Hub ({err}), đang nạp dữ liệu cục bộ hợp nhất:")
        if os.path.exists("data/train.jsonl"):
            path = "data/train.jsonl"
            dataset = load_dataset("json", data_files=path, split="train")
            print(f"✅ Đã nạp {len(dataset)} mẫu cờ từ tệp hợp nhất cục bộ: {path}")
        elif os.path.exists("data/train.json"):
            path = "data/train.json"
            dataset = load_dataset("json", data_files=path, split="train")
            print(f"✅ Đã nạp {len(dataset)} mẫu cờ từ tệp hợp nhất cục bộ: {path}")
        else:
            files = sorted(glob.glob("data/real_mined_*.json"))
            if files:
                dataset = load_dataset("json", data_files=files, split="train")
                print(f"✅ Đã nạp {len(dataset)} mẫu cờ từ {len(files)} tệp cục bộ.")
            else:
                data = [
                    {
                        "prompt": (
                            "Trạng thái bàn cờ tướng hiện tại (Biểu diễn đa chiều: Ma trận 2D, Chuỗi FEN chuẩn, và Lịch sử nước đi PGN):\n\n"
                            "1. Ma Trận Bàn Cờ 2D (9x10):\n"
                            "r n b a k a b n r\n"
                            ". . . . . . . . .\n"
                            ". c . . . . . c .\n"
                            "p . p . p . p . p\n"
                            ". . . . . . . . .\n"
                            ". . . . . . . . .\n"
                            "P . P . P . P . P\n"
                            ". C . . . . . C .\n"
                            ". . . . . . . . .\n"
                            "R N B A K A B N R\n\n"
                            "2. Chuỗi Chuẩn FEN (Forsyth-Edwards Notation):\n"
                            "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1\n\n"
                            "3. Lịch Sử Nước Đi PGN (Move History):\n"
                            "Ván cờ mới bắt đầu (Chưa có nước đi)\n\n"
                            "Đến lượt Đỏ đi. Hãy suy nghĩ sâu sắc trong thẻ <thought> và đưa ra nước đi UCI hợp lệ:"
                        ),
                        "completion": "<thought>\n1. Phân tích FEN & PGN\n2. Quyết định\n</thought>\nb2e2",
                        "move": "b2e2",
                        "stamp": 1700000000
                    }
                ] * 100
                dataset = Dataset.from_list(data)

    # Cấu hình GRPOConfig Tối Ưu Hóa Tối Đa Tốc Độ Tesla T4 FP16 Tensor Cores
    args = GRPOConfig(
        output_dir=f"outputs/xiangqi-r1-{variant}",
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

    print("============================================================")
    print(f" BẮT ĐẦU HUẤN LUYỆN XIANGQI-R1 ({variant.upper()}) BẰNG GRPO (FP16 TENSOR CORES) ")
    print("============================================================")
    trainer.train()

    # Xuất và lưu trọng số hợp nhất 16-bit cục bộ làm dự phòng trước khi đẩy Hub
    output = f"outputs/xiangqi-r1-{variant}-merged"
    print(f"💾 Đang lưu trọng số hợp nhất 16-bit cục bộ tại: {output}...")
    try:
        model.save_pretrained_merged(output, tokenizer, save_method="merged_16bit")
        print(f"✅ Đã lưu trọng số 16-bit cục bộ thành công!")
    except Exception as err:
        print(f"⚠️ Không thể lưu mô hình cục bộ: {err}")

    # Đẩy Mô hình đã Huấn luyện lên HuggingFace Model Hub
    if token:
        try:
            print(f"📤 Đang đẩy mô hình {variant.upper()} lên HuggingFace Model Hub: https://huggingface.co/{target}...")
            model.push_to_hub_merged(target, tokenizer, save_method="merged_16bit", token=token)
            print(f"✅ Hoàn tất đăng tải mô hình Xiangqi-R1 ({variant.upper()}) lên HuggingFace Hub thành công!")
        except Exception as err:
            print(f"⚠️ Đăng tải Hub thất bại ({err}). Trọng số đã được bảo toàn ở bộ nhớ cục bộ {output}.")
    else:
        print(f"⚠️ Không tìm thấy HF_TOKEN. Trọng số mô hình đã được bảo toàn cục bộ tại: {output}")

if __name__ == "__main__":
    main()
