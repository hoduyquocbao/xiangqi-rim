// Component Bảng Điều Khiển Engine (Engine Control Panel)
// Định danh đơn từ tiếng Anh: Panel, depth, level, hash, alloc, sizes, hint, undo, redo, flip, open, mode, engine, search, stop, status, reset, card, flex, text, icon, searching, undoable, redoable, e, rulers, toggleRulers

import React, { useState, useEffect } from 'react';

// Danh sách các mốc dung lượng RAM băm Transposition Table (MB)
const sizes = [128, 256, 512, 1024, 2048, 4096, 8192];
// Danh sách các mốc thời gian suy nghĩ tối đa mỗi nước (ms)
const times = [500, 1000, 2000, 5000, 10000, 30000, 60000];

export default function Panel({
  depth = 6,
  level,
  hash = 256,
  alloc,
  timeLimit = 2000,
  setTimeLimit,
  playMode = 'ai',
  setPlayMode,
  hint,
  undo,
  redo,
  flip,
  rulers = true,
  toggleRulers,
  open,
  openGym,
  openAudit,
  undoable = false,
  redoable = false,
  status = 'ready',
  search,
  stop
}) {
  const searching = status === 'searching';
  const [govMode, setGovMode] = useState('turbo');
  const [hwInfo, setHwInfo] = useState({
    cpu: 'Intel i5-8259U (4 Physical Cores)',
    gpu: 'Intel Iris Plus Graphics 655 (Metal Native)',
    shards: '2,720,604',
    nps: '1,635.1 FEN/s'
  });

  // Tự động thăm dò thông số phần cứng & trạng thái 1,024 Shards NVMe
  useEffect(() => {
    const fetchHw = async () => {
      try {
        const res = await fetch('/api/v1/diagnostics/hardware');
        if (res.ok) {
          const data = await res.json();
          setHwInfo((prev) => ({
            ...prev,
            cpu: data.cpu_model || prev.cpu,
            gpu: data.gpu_adapter || prev.gpu,
            shards: data.shards_count ? Number(data.shards_count).toLocaleString() : prev.shards
          }));
        }
      } catch (_) {}
    };
    fetchHw();
  }, []);

  // Xử lý nạp nóng Governor mode (Eco, Balanced, Turbo) không gián đoạn
  const handleGovernor = async (mode) => {
    setGovMode(mode);
    try {
      await fetch('/api/v1/campaign/config', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ mode })
      });
    } catch (_) {}
  };

  return (
    <div className="glass rounded-xl p-5 border border-gold/20 flex flex-col gap-5 shadow-glow">
      <h2 className="text-base font-royal font-bold text-gold border-b border-gold/20 pb-2 flex items-center justify-between">
        <span>ENGINE CONTROLS</span>
        <span className="text-xs font-mono text-emerald-400 font-semibold uppercase">
          {status}
        </span>
      </h2>

      {/* 🚀 BẢNG ĐIỀU PHỐI TÀI NGUYÊN GOVERNOR & 1,024 SHARDS NVME */}
      <div className="flex flex-col gap-2 bg-obsidian/90 p-3 rounded-lg border border-gold/30 shadow-inner">
        <div className="flex items-center justify-between">
          <span className="text-[11px] font-bold text-amber-400 tracking-wider flex items-center gap-1">
            ⚡ CPU GOVERNOR (NẠP NÓNG 0MS)
          </span>
          <span className="text-[10px] font-mono text-emerald-400 font-bold bg-emerald-950/80 px-1.5 py-0.5 rounded border border-emerald-500/40">
            💎 {hwInfo.shards} SHARDS
          </span>
        </div>

        {/* 3 Nút Gạt Governor Mode */}
        <div className="grid grid-cols-3 gap-1.5">
          <button
            onClick={() => handleGovernor('eco')}
            className={`py-1.5 px-2 text-[10px] font-bold rounded transition border ${
              govMode === 'eco'
                ? 'bg-emerald-600 text-white border-emerald-400 shadow-md font-black'
                : 'border-emerald-800/40 text-emerald-400/70 hover:bg-emerald-950/50 hover:text-emerald-300'
            }`}
          >
            🌿 ECO (1 Luồng)
          </button>
          <button
            onClick={() => handleGovernor('balanced')}
            className={`py-1.5 px-2 text-[10px] font-bold rounded transition border ${
              govMode === 'balanced'
                ? 'bg-sky-600 text-white border-sky-400 shadow-md font-black'
                : 'border-sky-800/40 text-sky-400/70 hover:bg-sky-950/50 hover:text-sky-300'
            }`}
          >
            ⚖️ BALANCED (2L)
          </button>
          <button
            onClick={() => handleGovernor('turbo')}
            className={`py-1.5 px-2 text-[10px] font-bold rounded transition border ${
              govMode === 'turbo'
                ? 'bg-gradient-to-r from-amber-500 to-red-600 text-white border-amber-300 shadow-lg font-black animate-pulse'
                : 'border-amber-800/40 text-amber-400/70 hover:bg-amber-950/50 hover:text-amber-300'
            }`}
          >
            🚀 TURBO (4 Cores)
          </button>
        </div>

        {/* Thông tin phần cứng chi tiết */}
        <div className="text-[9px] font-mono text-gold/60 flex flex-col gap-0.5 border-t border-gold/10 pt-1.5 mt-0.5">
          <div className="flex justify-between">
            <span>CPU:</span>
            <span className="text-gold/90 font-semibold truncate max-w-[170px]">{hwInfo.cpu}</span>
          </div>
          <div className="flex justify-between">
            <span>GPU:</span>
            <span className="text-gold/90 font-semibold truncate max-w-[170px]">{hwInfo.gpu}</span>
          </div>
        </div>
      </div>

      {/* Bộ Chuyển Chế Độ Chơi & Sắp Cờ: Huấn Luyện AI | Chơi 2 Người | Sắp Cờ */}
      <div className="flex flex-col gap-2">
        <span className="text-xs font-semibold text-gold/70 uppercase">
          CHẾ ĐỘ CHƠI & PHÂN TÍCH
        </span>
        <div className="grid grid-cols-4 gap-1 bg-obsidian/80 p-1.5 rounded-lg border border-gold/20">
          <button
            onClick={() => setPlayMode && setPlayMode('ai')}
            className={`py-2 px-1 text-[10px] font-bold rounded transition flex items-center justify-center gap-0.5 ${
              playMode === 'ai'
                ? 'bg-gold text-slate-950 shadow-md font-black'
                : 'text-gold/70 hover:text-gold hover:bg-gold/10'
            }`}
          >
            🤖 TRAINER
          </button>
          <button
            onClick={() => setPlayMode && setPlayMode('auto')}
            className={`py-2 px-1 text-[10px] font-bold rounded transition flex items-center justify-center gap-0.5 ${
              playMode === 'auto'
                ? 'bg-amber-400 text-slate-950 shadow-md font-black animate-pulse'
                : 'text-gold/70 hover:text-gold hover:bg-gold/10'
            }`}
          >
            ⚡ AUTOPLAY
          </button>
          <button
            onClick={() => setPlayMode && setPlayMode('pvp')}
            className={`py-2 px-1 text-[10px] font-bold rounded transition flex items-center justify-center gap-0.5 ${
              playMode === 'pvp'
                ? 'bg-gold text-slate-950 shadow-md font-black'
                : 'text-gold/70 hover:text-gold hover:bg-gold/10'
            }`}
          >
            👥 2 NGƯỜI
          </button>
          <button
            onClick={() => {
              if (setPlayMode) setPlayMode('editor');
              if (open) open();
            }}
            className={`py-2 px-1 text-[10px] font-bold rounded transition flex items-center justify-center gap-0.5 ${
              playMode === 'editor'
                ? 'bg-gold text-slate-950 shadow-md font-black'
                : 'text-gold/70 hover:text-gold hover:bg-gold/10'
            }`}
          >
            🧩 SẮP CỜ
          </button>
        </div>
      </div>

      {/* Chọn độ sâu AI (Depth Selector Slider 4..12) */}
      <div className="flex flex-col gap-2">
        <div className="flex justify-between items-center text-xs">
          <span className="text-gold/70 font-semibold">AI DEPTH LEVEL</span>
          <span className="text-gold font-bold font-mono bg-gold/10 border border-gold/30 px-2 py-0.5 rounded">
            DEPTH {depth}
          </span>
        </div>
        <input
          type="range"
          aria-label="AI DEPTH LEVEL"
          min="4"
          max="60"
          value={depth}
          onChange={(e) => level && level(Number(e.target.value))}
          className="w-full accent-gold bg-obsidian h-2 rounded-lg cursor-pointer"
        />
        <div className="flex justify-between text-[10px] text-gold/40 font-mono">
          <span>EASY (4)</span>
          <span>MEDIUM (8)</span>
          <span>HARD (12)</span>
        </div>
      </div>

      {/* Chọn Dung lượng Hash RAM Transposition Table (128MB .. 8GB) */}
      <div className="flex flex-col gap-2">
        <div className="flex justify-between items-center text-xs">
          <span className="text-gold/70 font-semibold">HASH RAM MEMORY</span>
          <span className="text-gold font-bold font-mono bg-gold/10 border border-gold/30 px-2 py-0.5 rounded">
            {hash >= 1024 ? `${hash / 1024}GB` : `${hash}MB`}
          </span>
        </div>
        <input
          type="range"
          aria-label="Hash RAM Memory"
          min="0"
          max="6"
          step="1"
          value={sizes.indexOf(hash) >= 0 ? sizes.indexOf(hash) : 1}
          onChange={(e) => alloc && alloc(sizes[Number(e.target.value)])}
          className="w-full accent-gold bg-obsidian h-2 rounded-lg cursor-pointer"
        />
        <div className="flex justify-between text-[10px] text-gold/40 font-mono">
          <span>128MB</span>
          <span>1GB</span>
          <span>8GB</span>
        </div>
      </div>

      {/* Chọn Thời gian Suy nghĩ Tối đa / Nước đi (Move Time Limit: 0.5s .. 60s) */}
      <div className="flex flex-col gap-2">
        <div className="flex justify-between items-center text-xs">
          <span className="text-gold/70 font-semibold">THỜI GIAN / NƯỚC (TIME LIMIT)</span>
          <span className="text-amber-400 font-bold font-mono bg-amber-500/10 border border-amber-500/30 px-2 py-0.5 rounded">
            {timeLimit >= 1000 ? `${(timeLimit / 1000).toFixed(1)}s` : `${timeLimit}ms`}
          </span>
        </div>
        <input
          type="range"
          aria-label="Move Time Limit"
          min="0"
          max="6"
          step="1"
          value={times.indexOf(timeLimit) >= 0 ? times.indexOf(timeLimit) : 2}
          onChange={(e) => setTimeLimit && setTimeLimit(times[Number(e.target.value)])}
          className="w-full accent-amber-400 bg-obsidian h-2 rounded-lg cursor-pointer"
        />
        <div className="flex justify-between text-[10px] text-gold/40 font-mono">
          <span>0.5s</span>
          <span>2.0s</span>
          <span>60.0s</span>
        </div>
      </div>

      {/* Các nút lệnh tính toán AI Hint & Stop */}
      <div className="grid grid-cols-2 gap-2">
        <button
          onClick={() => (searching ? stop && stop() : search && search())}
          className={`py-2.5 rounded text-xs font-bold transition-all shadow-glow flex items-center justify-center gap-1 ${
            searching
              ? 'bg-vermilion text-gold border border-vermilion hover:bg-vermilion/80'
              : 'bg-gold text-obsidian border border-gold hover:bg-gold/90'
          }`}
        >
          {searching ? 'STOP AI SEARCH' : 'START AI CALCULATE'}
        </button>

        <button
          onClick={hint}
          disabled={searching}
          className="py-2.5 rounded bg-obsidian-card border border-gold/40 text-xs font-bold text-gold hover:bg-gold/10 hover:border-gold disabled:opacity-40 transition flex items-center justify-center gap-1"
        >
          BEST MOVE HINT
        </button>
      </div>

      {/* Các nút thao tác xem lại ván đấu Undo / Redo / Flip / Toggle Rulers */}
      <div className="grid grid-cols-4 gap-1.5">
        <button
          onClick={undo}
          disabled={!undoable || searching}
          className="py-2 rounded bg-obsidian-card border border-gold/30 text-xs font-semibold text-gold/80 hover:border-gold hover:text-gold disabled:opacity-30 transition"
        >
          UNDO
        </button>
        <button
          onClick={redo}
          disabled={!redoable || searching}
          className="py-2 rounded bg-obsidian-card border border-gold/30 text-xs font-semibold text-gold/80 hover:border-gold hover:text-gold disabled:opacity-30 transition"
        >
          REDO
        </button>
        <button
          onClick={flip}
          className="py-2 rounded bg-obsidian-card border border-gold/30 text-xs font-semibold text-gold/80 hover:border-gold hover:text-gold transition"
        >
          FLIP
        </button>
        <button
          onClick={toggleRulers}
          className={`py-2 rounded bg-obsidian-card border text-xs font-semibold transition ${
            rulers
              ? 'border-gold text-gold bg-gold/10 font-bold'
              : 'border-gold/30 text-gold/50 hover:text-gold'
          }`}
        >
          RULER
        </button>
      </div>

      {/* Nút Mở Modal FEN / PGN Parser, GYM Trainer & Audit Scanner */}
      <div className="grid grid-cols-3 gap-1.5">
        <button
          onClick={open}
          className="py-2 rounded border border-gold/40 bg-gold/5 text-gold text-[11px] font-bold hover:bg-gold/15 hover:border-gold transition flex items-center justify-center"
        >
          FEN / PGN PARSER & EDITOR
        </button>
        <button
          onClick={openGym}
          className="py-2 rounded border border-amber-500/50 bg-amber-500/10 text-amber-400 text-[11px] font-bold hover:bg-amber-500/20 hover:border-amber-400 transition flex items-center justify-center shadow-glow"
        >
          🏋️ GYM
        </button>
        <button
          onClick={openAudit}
          className="py-2 rounded border border-indigo-500/50 bg-indigo-500/10 text-indigo-400 text-[11px] font-bold hover:bg-indigo-500/20 hover:border-indigo-400 transition flex items-center justify-center shadow-glow"
        >
          🛡️ AUDIT
        </button>
      </div>
    </div>
  );
}
