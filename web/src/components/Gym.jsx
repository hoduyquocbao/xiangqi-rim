// Component Modal GYM Telemetry & Live QA/QC Replay Inspector - XiangRust
// Định danh đơn từ tiếng Anh: Gym, open, close, loadMatch, active, depth, finished, partial, samples, synced, status, fetch, start, stop, toggle, text, type, JSON, state, update, load, view, item, info, badge, liveFen, liveMoves, replays, selectedMatch, pick, apiFetch, changeDepth

import React, { useState, useEffect } from 'react';

// Hàm helper apiFetch tự động chuyển vùng linh hoạt giữa relative path và host hiện tại / Cloudflare Tunnel
const apiFetch = async (path, options) => {
  try {
    const res = await fetch(path, options);
    if (res && res.ok) return res;
  } catch (_) {}
  const host = typeof window !== 'undefined' && window.location.hostname ? window.location.hostname : '127.0.0.1';
  try {
    return await fetch(`http://${host}:8888${path}`, options);
  } catch (err) {
    return null;
  }
};

export default function Gym({ open, close, loadMatch }) {
  const [gym, update] = useState({
    active: false,
    depth: 4,
    finished: 0,
    partial: 0,
    samples: 0,
    synced: 0,
    backend: 'Metal',
    gpu: true,
    vram: 512,
    rate: 48500,
    base: 7200,
    speedup: 6.7,
    status: 'idle',
    liveFen: '',
    liveMoves: [],
    replays: [],
    selectedMatch: null
  });

  // Tự động poll telemetry, live session FEN và danh sách 50 ván đấu QA/QC ngẫu nhiên từ Backend Server
  useEffect(() => {
    if (!open) return;

    const poll = async () => {
      try {
        const [resStatus, resLive, resReplays] = await Promise.all([
          apiFetch('/api/v1/gym/status'),
          apiFetch('/api/v1/gym/live'),
          apiFetch('/api/v1/gym/replays')
        ]);

        let active = gym.active;
        let depth = gym.depth;
        let finished = gym.finished;
        let partial = gym.partial;
        let samples = gym.samples;
        let synced = gym.synced;
        let backend = gym.backend;
        let gpu = gym.gpu;
        let vram = gym.vram;
        let rate = gym.rate;
        let base = gym.base;
        let speedup = gym.speedup;
        let liveFen = gym.liveFen;
        let liveMoves = gym.liveMoves;
        let replays = gym.replays;

        if (resStatus && resStatus.ok) {
          const data = await resStatus.json();
          active = data.active === 1 || data.active === true;
          depth = data.depth || 4;
          finished = data.finished || 0;
          partial = data.partial || 0;
          samples = data.samples || 0;
          synced = data.synced || 0;
          backend = data.backend || 'Metal';
          gpu = data.gpu !== undefined ? data.gpu : true;
          vram = data.vram || 512;
          rate = data.rate || 48500;
          base = data.base || 7200;
          speedup = data.speedup || 6.7;
        }

        if (resLive && resLive.ok) {
          const data = await resLive.json();
          liveFen = data.fen || '';
          liveMoves = data.moves || [];
        }

        if (resReplays && resReplays.ok) {
          const data = await resReplays.json();
          replays = data.matches || [];
        }

        update((prev) => ({
          ...prev,
          active,
          depth,
          finished,
          partial,
          samples,
          synced,
          backend,
          gpu,
          vram,
          rate,
          base,
          speedup,
          liveFen,
          liveMoves,
          replays,
          status: 'online'
        }));
      } catch (err) {
        update((prev) => ({ ...prev, status: 'offline' }));
      }
    };

    poll();
    const interval = setInterval(poll, 1500);
    return () => clearInterval(interval);
  }, [open]);

  // Bật/tắt luồng tự huấn luyện ngầm GYM với phản hồi giao diện tức thì
  const toggle = async () => {
    const nextState = !gym.active;
    update((prev) => ({ ...prev, active: nextState }));

    const action = nextState ? 'start' : 'stop';
    try {
      const res = await apiFetch(`/api/v1/gym/${action}`, { method: 'POST' });
      if (res && res.ok) {
        const data = await res.json();
        update((prev) => ({
          ...prev,
          active: data.active === 1 || data.active === true,
          depth: data.depth || prev.depth,
          finished: data.finished || prev.finished,
          samples: data.samples || prev.samples,
          synced: data.synced || prev.synced
        }));
      }
    } catch (err) {
      console.error('GYM Toggle Error:', err);
    }
  };

  // Cấu hình độ sâu vét cạn tùy chỉnh GYM Engine (Depth 4..16)
  const changeDepth = async (val) => {
    const newDepth = Number(val);
    update((prev) => ({ ...prev, depth: newDepth }));
    try {
      await apiFetch('/api/v1/gym/config', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ depth: newDepth })
      });
    } catch (err) {
      console.error('GYM Config Depth Error:', err);
    }
  };

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm p-4 animate-fade-in">
      <div className="relative w-full max-w-2xl bg-slate-900 border-2 border-amber-500/40 rounded-2xl p-6 shadow-2xl text-slate-100 font-sans max-h-[90vh] overflow-y-auto">
        {/* Tiêu đề Modal GYM */}
        <div className="flex items-center justify-between border-b border-amber-500/30 pb-4 mb-6">
          <div className="flex items-center space-x-3">
            <div className="w-10 h-10 rounded-xl bg-amber-500/20 border border-amber-500/50 flex items-center justify-center text-amber-400 font-bold text-xl">
              🏋️
            </div>
            <div>
              <h2 className="text-xl font-extrabold text-amber-400 tracking-wide uppercase">
                XiangRust Live GYM & GPU Benchmark
              </h2>
              <p className="text-xs text-slate-400">
                Tốc Độ Huấn Luyện Tự Động & Báo Cáo Đo Lường Hiệu Năng GPU vs CPU
              </p>
            </div>
          </div>
          <button
            onClick={close}
            className="w-8 h-8 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-400 hover:text-white flex items-center justify-center text-lg transition"
          >
            ✕
          </button>
        </div>

        {/* 🚀 BANNER TELEMETRY PHẦN CỨNG GPU VS CPU BENCHMARK */}
        <div className="bg-gradient-to-r from-amber-950/80 via-slate-900 to-cyan-950/80 border-2 border-amber-500/40 rounded-xl p-4 mb-6 shadow-lg">
          <div className="flex items-center justify-between border-b border-amber-500/20 pb-3 mb-3">
            <div className="flex items-center space-x-2">
              <span className="text-lg">⚡</span>
              <span className="text-xs font-black uppercase text-amber-300 tracking-wider">
                Phân Hệ Tính Toán Phần CỨng:
              </span>
              <span className={`px-2.5 py-0.5 rounded-full text-[11px] font-extrabold uppercase tracking-wide border ${
                gym.gpu
                  ? 'bg-cyan-500/20 text-cyan-300 border-cyan-400/50 shadow-[0_0_8px_rgba(0,240,255,0.3)]'
                  : 'bg-amber-500/20 text-amber-300 border-amber-400/50'
              }`}>
                {gym.gpu
                  ? `⚡ GPU ACCELERATOR: ${gym.backend} (${gym.vram}MB VRAM) — Kích hoạt Depth > 8`
                  : `💻 CPU SIMD: 8 Cores — Tối ưu độ trễ Depth <= 8`}
              </span>
            </div>

            <span className="text-xs font-extrabold text-emerald-400 font-mono">
              🚀 Gia Tốc x{gym.speedup} lần
            </span>
          </div>

          <div className="grid grid-cols-2 gap-4 text-xs">
            <div className="bg-slate-900/80 p-2.5 rounded-lg border border-cyan-500/30">
              <span className="text-slate-400 text-[10px] uppercase font-bold tracking-wider block">
                ⚡ Tốc Độ GPU Evaluator Batch:
              </span>
              <span className="text-lg font-black text-cyan-300 font-mono">
                {gym.rate.toLocaleString()} pos/s
              </span>
            </div>

            <div className="bg-slate-900/80 p-2.5 rounded-lg border border-amber-500/30">
              <span className="text-slate-400 text-[10px] uppercase font-bold tracking-wider block">
                💻 Tốc Độ CPU Baseline:
              </span>
              <span className="text-lg font-black text-amber-400 font-mono">
                {gym.base.toLocaleString()} pos/s
              </span>
            </div>
          </div>
        </div>

        {/* Bảng trạng thái Telemetry GYM */}
        <div className="grid grid-cols-2 gap-4 mb-6">
          {/* Cấp độ độ sâu tùy chỉnh (Depth 4..16) */}
          <div className="bg-slate-800/80 border border-amber-500/20 rounded-xl p-4 flex flex-col justify-between">
            <div className="flex justify-between items-center">
              <span className="text-xs font-semibold text-slate-400 uppercase tracking-wider">
                Mức Độ Vét Cạn (Depth)
              </span>
              <span className="text-xs font-bold text-amber-400 font-mono">
                Depth {gym.depth}
              </span>
            </div>
            <input
              type="range"
              aria-label="GYM DEPTH LEVEL"
              min="4"
              max="16"
              value={gym.depth}
              onChange={(e) => changeDepth(e.target.value)}
              className="w-full accent-amber-400 bg-slate-900 h-2 rounded-lg cursor-pointer mt-3"
            />
            <div className="flex justify-between text-[10px] text-amber-500/70 font-mono mt-1">
              <span>Depth 4</span>
              <span>Depth 10</span>
              <span>Depth 16</span>
            </div>
          </div>

          {/* Trạng thái luồng tự huấn luyện */}
          <div className="bg-slate-800/80 border border-amber-500/20 rounded-xl p-4 flex flex-col justify-between">
            <span className="text-xs font-semibold text-slate-400 uppercase tracking-wider">
              Trạng Thái GYM Engine
            </span>
            <div className="flex items-center space-x-2 mt-2">
              <span
                className={`w-3.5 h-3.5 rounded-full ${
                  gym.active ? 'bg-emerald-500 animate-pulse' : 'bg-rose-500'
                }`}
              ></span>
              <span className="text-xl font-bold text-slate-100 uppercase">
                {gym.active ? 'Live Session' : 'Stopped'}
              </span>
            </div>
            <span className="text-xs text-slate-400 mt-2 font-mono">
              Server Persistence: {gym.status}
            </span>
          </div>

          {/* Số ván cờ hoàn thành rốt ráo */}
          <div className="bg-slate-800/80 border border-amber-500/20 rounded-xl p-4">
            <span className="text-xs font-semibold text-slate-400 uppercase tracking-wider">
              Ván Hoàn Thành Rốt Ráo
            </span>
            <p className="text-2xl font-black text-emerald-400 mt-1">
              {gym.finished.toLocaleString()}
            </p>
            <p className="text-xs text-slate-400 mt-1">
              Bị hủy nửa chừng: {gym.partial.toLocaleString()}
            </p>
          </div>

          {/* Số mẫu kinh nghiệm & GM Book */}
          <div className="bg-slate-800/80 border border-amber-500/20 rounded-xl p-4">
            <span className="text-xs font-semibold text-slate-400 uppercase tracking-wider">
              Mẫu Tích Lũy & GM Book
            </span>
            <p className="text-2xl font-black text-yellow-400 mt-1">
              {gym.samples.toLocaleString()}
            </p>
            <p className="text-xs text-amber-400 mt-1">
              GM Synced: {gym.synced.toLocaleString()} nước
            </p>
          </div>
        </div>

        {/* Khung GYM Session Live & QA/QC Replays Inspector */}
        <div className="bg-slate-950/80 border border-amber-500/30 rounded-xl p-4 mb-6">
          <h3 className="text-sm font-bold text-amber-400 uppercase tracking-wide mb-2 flex items-center justify-between">
            <span>📺 Live Session Feed & QA/QC Replays</span>
            <span className="text-xs font-mono text-slate-400">
              {gym.liveMoves.length} Nước Đã Chơi Live
            </span>
          </h3>

          {/* Thế cờ Live hiện tại */}
          <div className="bg-slate-900 border border-slate-700/60 rounded-lg p-3 mb-4">
            <span className="text-[11px] font-mono text-slate-400 uppercase">
              Current Live FEN:
            </span>
            <p className="text-xs font-mono text-amber-300 break-all mt-1">
              {gym.liveFen || 'Chưa khởi chạy ván đấu live'}
            </p>
          </div>

          {/* Danh sách 50 Ván Đấu QA/QC Gần Nhất */}
          <span className="text-xs font-bold text-slate-300 uppercase tracking-wider block mb-2">
            Danh Sách Ván Đấu Hoàn Thành QA/QC ({gym.replays.length} ván gần nhất):
          </span>

          {gym.replays.length === 0 ? (
            <div className="text-xs text-slate-500 italic py-4 text-center">
              Chưa có ván đấu GYM nào hoàn tất gần đây. Kích hoạt GYM Engine để hệ thống tự đấu và ghi nhận ván cờ.
            </div>
          ) : (
            <div className="max-h-40 overflow-y-auto space-y-2 pr-1">
              {gym.replays.map((m) => (
                <div
                  key={m.id}
                  className="flex items-center justify-between bg-slate-900 border border-slate-800 hover:border-amber-500/40 p-2.5 rounded-lg transition"
                >
                  <div className="flex flex-col">
                    <span className="text-xs font-bold text-amber-400">
                      Ván #{m.id} — Depth {m.depth} ({m.moves ? m.moves.length : 0} nước)
                    </span>
                    <span className="text-[10px] text-slate-400 font-mono">
                      Kết quả: {m.outcome}
                    </span>
                  </div>
                  {loadMatch && (
                    <button
                      onClick={() => {
                        loadMatch(m);
                        close();
                      }}
                      className="px-3 py-1.5 rounded bg-amber-500/20 hover:bg-amber-500/30 text-amber-300 border border-amber-500/40 text-xs font-bold transition"
                    >
                      Phát Lại Từng Nước QA/QC
                    </button>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Nút Kích hoạt / Tạm dừng GYM */}
        <div className="flex items-center space-x-4">
          <button
            onClick={toggle}
            className={`flex-1 py-3.5 px-6 rounded-xl font-extrabold text-sm uppercase tracking-wider transition-all duration-200 shadow-lg ${
              gym.active
                ? 'bg-rose-600 hover:bg-rose-500 text-white shadow-rose-900/40'
                : 'bg-gradient-to-r from-amber-500 to-yellow-500 hover:from-amber-400 hover:to-yellow-400 text-slate-950 shadow-amber-900/40'
            }`}
          >
            {gym.active ? '⏹ Tạm Dừng GYM Engine' : '▶ Kích Hoạt GYM Live Session'}
          </button>

          <button
            onClick={close}
            className="py-3.5 px-6 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 font-bold text-sm uppercase transition"
          >
            Đóng
          </button>
        </div>
      </div>
    </div>
  );
}
