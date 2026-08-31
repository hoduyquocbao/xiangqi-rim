import json

with open("gpu_t4_multiturn_miner.py", "r", encoding="utf-8") as f:
    lines = f.readlines()

def find_line(pattern):
    for i, line in enumerate(lines):
        if pattern in line:
            return i
    raise ValueError(f"Pattern {pattern} not found")

sec1_idx = 0
sec2_idx = find_line("# PHẦN II:")
sec3_idx = find_line("# PHẦN III:")
sec4_idx = find_line("# PHẦN IV:")
sec5_idx = find_line("# PHẦN V: HÀM TẠO MẪU")
sec6_idx = find_line("# PHẦN VI:")

sec1_code = "".join(lines[sec1_idx:sec2_idx])
sec2_code = "".join(lines[sec2_idx:sec3_idx])
sec3_code = "".join(lines[sec3_idx:sec4_idx])
sec4_code = "".join(lines[sec4_idx:sec5_idx])
sec5_code = "".join(lines[sec5_idx:sec6_idx])
sec6_code = "".join(lines[sec6_idx:])

print(f"Sec 1 bounds: {sec1_idx}..{sec2_idx} ({len(sec1_code)} chars)")
print(f"Sec 2 bounds: {sec2_idx}..{sec3_idx} ({len(sec2_code)} chars)")
print(f"Sec 3 bounds: {sec3_idx}..{sec4_idx} ({len(sec3_code)} chars)")
print(f"Sec 4 bounds: {sec4_idx}..{sec5_idx} ({len(sec4_code)} chars)")
print(f"Sec 5 bounds: {sec5_idx}..{sec6_idx} ({len(sec5_code)} chars)")
print(f"Sec 6 bounds: {sec6_idx}..end ({len(sec6_code)} chars)")

with open("colab_gpu_multiturn_v17.ipynb", "r", encoding="utf-8") as f:
    nb = json.load(f)

for c in nb["cells"]:
    cid = c.get("metadata", {}).get("id")
    if cid == "PvdX_tmi32Bi":
        c["source"] = ["# @title ⚙️ SECTION 1: SYSTEM ENVIRONMENT & CONSTANTS { display-mode: \"form\" }\n" + sec1_code]
    elif cid == "Rx49nr7bjS9F":
        c["source"] = ["# @title ♟️ SECTION 2: PHYSICAL XIANGQI BOARD ENGINE & 32D JRCP 5.0 ANALYZER { display-mode: \"form\" }\n" + sec2_code]
    elif cid == "sec3_eval":
        c["source"] = ["# @title 🧠 SECTION 3: PYTORCH DEEP RESIDUAL EVALUATOR (5M PARAMETERS) { display-mode: \"form\" }\n" + sec3_code]
    elif cid == "sp8l_Fh2nzdx":
        c["source"] = ["# @title 🧪 SECTION 4: 43 GEOMETRY UNIT TESTS & DATA VALIDATOR & HTML REPORT { display-mode: \"form\" }\n" + sec4_code]
    elif cid == "okDiPzrln0l7":
        c["source"] = ["# @title 🔮 SECTION 5: 32D JRCP 5.0 THOUGHT TRAJECTORY GENERATOR { display-mode: \"form\" }\n" + sec5_code]
    elif cid == "WYjjL-RWn1hx":
        c["source"] = ["# @title 🚀 SECTION 6: MINIMAX TENSOR SEARCH & MULTI-TURN MINER { display-mode: \"form\" }\n" + sec6_code]

with open("colab_gpu_multiturn_v17.ipynb", "w", encoding="utf-8") as f:
    json.dump(nb, f, ensure_ascii=False, indent=1)

with open("notebooks/02_multi_turn_mining/01_colab_gpu_multiturn_v17.ipynb", "w", encoding="utf-8") as f:
    json.dump(nb, f, ensure_ascii=False, indent=1)

print("✅ Dynamically rebuilt clean notebook files matching exact section headers!")
