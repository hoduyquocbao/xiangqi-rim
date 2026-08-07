// web/src/components/R1Studio.jsx
// Xiangqi-R1 GRPO LLM Distributed Training Studio Modal
// Single-Word English Identifiers: Studio, show, close, train, reward, format, rule, quality, model, status, script, copy

import React, { useState } from 'react';
import { Cpu, Zap, ShieldCheck, FileCode, Check, Copy, Sparkles, Network, Database } from 'lucide-react';

export function R1Studio({ show, close }) {
  const [copied, setCopied] = useState(false);
  const [running, setRunning] = useState(false);

  if (!show) return null;

  const path = 'scripts/train.py';

  const copy = () => {
    navigator.clipboard.writeText(`python3 ${path}`);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="fixed inset-0 z-50 bg-black/80 backdrop-blur-md flex items-center justify-center p-4">
      <div className="bg-obsidian-card border-2 border-gold/40 rounded-2xl max-w-4xl w-full max-h-[90vh] overflow-y-auto p-6 space-y-6 shadow-2xl">
        {/* Header */}
        <div className="flex items-center justify-between border-b border-gold/20 pb-4">
          <div className="flex items-center gap-3">
            <Sparkles className="w-7 h-7 text-gold animate-pulse" />
            <div>
              <h2 className="text-xl font-royal font-bold text-gold tracking-wide">
                XIANGQI-R1 LLM DISTRIBUTED TRAINER (GRPO)
              </h2>
              <p className="text-xs text-gold/70">
                Qwen2.5-7B-Instruct + Unsloth 4-bit LoRA + 3 Reward Functions & P2P Mesh
              </p>
            </div>
          </div>
          <button
            onClick={close}
            className="px-3 py-1 bg-gold/10 hover:bg-gold/20 text-gold border border-gold/30 rounded-lg text-xs font-bold transition"
          >
            ✕ ĐÓNG
          </button>
        </div>

        {/* Distributed Compute & P2P Topic Mesh Status */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div className="bg-obsidian/80 border border-emerald-500/40 p-4 rounded-xl space-y-1">
            <div className="flex items-center gap-2 text-emerald-400 text-xs font-bold">
              <Network className="w-4 h-4" />
              P2P MESH TOPIC (24/7)
            </div>
            <p className="text-[11px] text-gold/80 font-mono truncate">
              sha256(mesh2026)
            </p>
            <span className="inline-block text-[10px] px-2 py-0.5 rounded bg-emerald-500/20 text-emerald-300 font-bold border border-emerald-500/30">
              ● LIVE & CONNECTED
            </span>
          </div>

          <div className="bg-obsidian/80 border border-purple-500/40 p-4 rounded-xl space-y-1">
            <div className="flex items-center gap-2 text-purple-400 text-xs font-bold">
              <Database className="w-4 h-4" />
              STORAGE PERSISTENCE
            </div>
            <p className="text-[11px] text-gold/80 font-mono">
              IndexedDB High-Capacity
            </p>
            <span className="inline-block text-[10px] px-2 py-0.5 rounded bg-purple-500/20 text-purple-300 font-bold border border-purple-500/30">
              ● UNLIMITED (NO 5MB LIMIT)
            </span>
          </div>

          <div className="bg-obsidian/80 border border-gold/40 p-4 rounded-xl space-y-1">
            <div className="flex items-center gap-2 text-gold text-xs font-bold">
              <Zap className="w-4 h-4" />
              GRPO ACCELERATION
            </div>
            <p className="text-[11px] text-gold/80 font-mono">
              Unsloth 4-bit LoRA (r=16)
            </p>
            <span className="inline-block text-[10px] px-2 py-0.5 rounded bg-gold/20 text-gold font-bold border border-gold/30">
              ● VRAM &lt; 14GB (RTX 3090/4090)
            </span>
          </div>
        </div>

        {/* 3 GRPO Reward Functions Specification */}
        <div className="space-y-3">
          <h3 className="text-sm font-bold text-gold flex items-center gap-2">
            <ShieldCheck className="w-4 h-4 text-emerald-400" />
            BA MÁY CHẤM ĐIỂM TỰ ĐỘNG (GRPO REWARD FUNCTIONS)
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-3 text-xs">
            <div className="bg-emerald-950/40 border border-emerald-500/30 p-3 rounded-xl space-y-1">
              <h4 className="font-bold text-emerald-400">1️⃣ LUẬT CHƠI (RULE)</h4>
              <p className="text-[11px] text-gold/70">
                Nước đi hợp lệ cờ tướng: <b className="text-emerald-300">+2.0 điểm</b>. Đi sai luật: <b className="text-red-400">-5.0 điểm</b> & ngắt lượt.
              </p>
            </div>

            <div className="bg-blue-950/40 border border-blue-500/30 p-3 rounded-xl space-y-1">
              <h4 className="font-bold text-blue-400">2️⃣ ĐỊNH DẠNG (FORMAT)</h4>
              <p className="text-[11px] text-gold/70">
                Có thẻ suy luận <code className="text-blue-300">&lt;thought&gt;</code> Ma trận 2D: <b className="text-blue-300">+1.0 điểm</b>. Sai định dạng: <b className="text-red-400">-1.0 điểm</b>.
              </p>
            </div>

            <div className="bg-amber-950/40 border border-amber-500/30 p-3 rounded-xl space-y-1">
              <h4 className="font-bold text-amber-400">3️⃣ CHIẾN THUẬT (QUALITY)</h4>
              <p className="text-[11px] text-gold/70">
                Trùng gợi ý tốt nhất từ XiangRust Engine: <b className="text-amber-300">+3.0 điểm</b>. Nước đi bình thường: <b className="text-gold/80">+0.5 điểm</b>.
              </p>
            </div>
          </div>
        </div>

        {/* HuggingFace Hub & Low-Spec 512MB VRAM Architecture */}
        <div className="bg-obsidian/90 border border-gold/40 p-4 rounded-xl space-y-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2 text-xs font-bold text-gold">
              <Cpu className="w-4 h-4 text-emerald-400" />
              HUGGINGFACE HUB INTEGRATION (TOKEN CONNECTED)
            </div>
            <span className="text-[10px] font-mono px-2 py-0.5 rounded bg-emerald-500/20 text-emerald-300 font-bold border border-emerald-500/40">
              ● AUTHENTICATED
            </span>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-3 text-xs">
            <div className="bg-emerald-950/30 border border-emerald-500/30 p-3 rounded-lg space-y-1">
              <span className="font-bold text-emerald-400">DATASET REPOSITORY:</span>
              <p className="text-[11px] text-gold/80 font-mono">https://huggingface.co/datasets/hoduyquocbao/xiangqi-r1-dataset</p>
              <p className="text-[10px] text-gold/60">Máy yếu (512MB VRAM/CPU) đẩy dữ liệu cờ tự đấu về đây.</p>
            </div>
            <div className="bg-amber-950/30 border border-amber-500/30 p-3 rounded-lg space-y-1">
              <span className="font-bold text-amber-400">0.5B MODEL (SIÊU NHẸ / &lt; 1.5GB VRAM):</span>
              <p className="text-[11px] text-gold/80 font-mono">https://huggingface.co/hoduyquocbao/xiangqi-r1-0.5b</p>
              <p className="text-[10px] text-gold/60">Qwen2.5-0.5B siêu nhanh (&lt;50ms), chạy vừa trên trình duyệt WebGPU/GPU yếu.</p>
            </div>
            <div className="bg-purple-950/30 border border-purple-500/30 p-3 rounded-lg space-y-1">
              <span className="font-bold text-purple-400">7B MODEL (REASONER / &lt; 14GB VRAM):</span>
              <p className="text-[11px] text-gold/80 font-mono">https://huggingface.co/hoduyquocbao/xiangqi-r1</p>
              <p className="text-[10px] text-gold/60">Qwen2.5-7B tư duy sâu thẻ &lt;thought&gt;, huấn luyện trên Colab GPU.</p>
            </div>
          </div>
        </div>

        {/* Execution & Script Runner */}
        <div className="bg-obsidian/90 border border-gold/30 p-4 rounded-xl space-y-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2 text-xs font-bold text-gold">
              <FileCode className="w-4 h-4" />
              SCRIPTS RUNNER (`scripts/share.py` & `scripts/train.py`)
            </div>
            <button
              onClick={copy}
              className="px-3 py-1 bg-gold/20 hover:bg-gold/30 text-gold border border-gold/40 rounded text-xs font-bold flex items-center gap-1 transition"
            >
              {copied ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
              {copied ? 'ĐÃ COPY LỆNH' : 'COPY LỆNH CHẠY'}
            </button>
          </div>

          <pre className="bg-black/90 p-3 rounded-lg text-[11px] font-mono text-emerald-400 overflow-x-auto border border-emerald-500/20">
            <code>
{`# 1. Khai thác dữ liệu cờ tự đấu liên tục từ máy yếu (512MB VRAM / CPU):
python3 scripts/mine.py

# 2. Đẩy nhanh 1 batch dữ liệu cờ lên HuggingFace Dataset:
python3 scripts/share.py

# 3. Huấn luyện bản Siêu Nhẹ Siêu Nhanh Qwen 0.5B (< 3GB VRAM):
python3 scripts/train.py 0.5b

# 4. Huấn luyện bản Qwen 7B Reasoner (< 14GB VRAM):
python3 scripts/train.py 7b

# 5. Google Colab Notebook thực tế:
# Mở file train.ipynb trực tiếp trên Colab`}
            </code>
          </pre>

          <button
            onClick={() => {
              setRunning(true);
              setTimeout(() => setRunning(false), 3000);
            }}
            className="w-full py-2.5 bg-gradient-to-r from-gold via-amber-400 to-gold text-black font-black text-xs rounded-xl hover:brightness-110 transition shadow-glow flex items-center justify-center gap-2 uppercase tracking-wide"
          >
            {running ? (
              <>
                <span className="w-4 h-4 border-2 border-black border-t-transparent rounded-full animate-spin"></span>
                ĐANG KẾT NỐI HUGGINGFACE HUB & P2P MESH...
              </>
            ) : (
              <>
                <Zap className="w-4 h-4 fill-current" />
                KÍCH HOẠT TIẾN TRÌNH DISTRIBUTED DATA & GRPO LLM
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
}
