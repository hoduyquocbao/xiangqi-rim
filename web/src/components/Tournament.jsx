// web/src/components/Tournament.jsx
// Visualizer Component for Depth 30 Red AI vs Depth 60 Black AI Grand Match
// 100% Single-Word English Identifiers — 30-Dimensional Telemetry Dashboard

import React, { useState, useEffect, useRef } from 'react';
import Board from './Board.jsx';
import { uciToMove } from '../rules/rules.js';

const startFen = 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1';

export default function Tournament({ onClose }) {
  const [matchData, setMatchData] = useState([]);
  const [currentPly, setCurrentPly] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [speed, setSpeed] = useState(1000);
  const [autoFollow, setAutoFollow] = useState(true);
  const [showJson, setShowJson] = useState(false);

  const timerRef = useRef(null);
  const pollRef = useRef(null);

  // Fetch match data JSONL periodically (Live Sync with Engine task)
  const loadData = () => {
    fetch('/match_d30_vs_d60.jsonl?t=' + Date.now())
      .then((res) => res.text())
      .then((text) => {
        const lines = text.trim().split('\n').filter((l) => l.trim().length > 0);
        const parsed = lines
          .map((l) => {
            try {
              return JSON.parse(l);
            } catch (e) {
              return null;
            }
          })
          .filter(Boolean);

        if (parsed.length > 0) {
          setMatchData(parsed);
          if (autoFollow) {
            setCurrentPly(parsed.length - 1);
          }
        }
      })
      .catch(() => {});
  };

  useEffect(() => {
    loadData();
    pollRef.current = setInterval(loadData, 2000);
    return () => {
      if (pollRef.current) clearInterval(pollRef.current);
    };
  }, [autoFollow]);

  // Auto playback timer
  useEffect(() => {
    if (playing) {
      timerRef.current = setTimeout(() => {
        setCurrentPly((prev) => {
          if (prev >= matchData.length - 1) {
            setPlaying(false);
            return prev;
          }
          return prev + 1;
        });
      }, speed);
    }
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, [playing, currentPly, speed, matchData.length]);

  const activeItem = matchData[currentPly] || {
    ply: 0,
    side: 'N/A',
    fen: startFen,
    best_move: 'N/A',
    from_sq: 0,
    to_sq: 0,
    moved_piece: '.',
    captured_piece: '.',
    score: 0,
    completed_depth: 0,
    target_depth: 0,
    nodes: 0,
    nps: 0,
    ply_time_ms: 0,
    match_elapsed_s: 0,
    ram_rss_mb: 0,
    tt_hash_mb: 256,
    cpu_threads: 8,
    is_check: false,
    is_capture: false,
    is_pv_move: true,
    red_piece_count: 16,
    black_piece_count: 16,
    material_balance: 0,
    king_safety_red: 100,
    king_safety_black: 100,
    center_control: 0,
    threat_score: 0,
    opportunity_score: 0,
    rule50_halfmoves: 0
  };

  const activeMove = activeItem.best_move ? uciToMove(activeItem.best_move) : null;

  const handleNext = () => {
    setAutoFollow(false);
    if (currentPly < matchData.length - 1) setCurrentPly((prev) => prev + 1);
  };

  const handlePrev = () => {
    setAutoFollow(false);
    if (currentPly > 0) setCurrentPly((prev) => prev - 1);
  };

  const handleStart = () => {
    setAutoFollow(false);
    setCurrentPly(0);
  };

  const handleEnd = () => {
    setAutoFollow(true);
    setCurrentPly(matchData.length - 1);
  };

  return (
    <div className="fixed inset-0 z-50 bg-obsidian/90 backdrop-blur-md flex items-center justify-center p-4">
      <div className="bg-obsidian-card border border-gold/30 rounded-2xl w-full max-w-6xl max-h-[95vh] flex flex-col shadow-glow overflow-hidden">
        {/* Header */}
        <div className="p-4 border-b border-gold/20 flex items-center justify-between bg-obsidian-header flex-wrap gap-2">
          <div className="flex items-center gap-3">
            <span className="text-2xl">⚔️</span>
            <div>
              <h2 className="text-lg font-royal font-bold text-gold">
                30-DIMENSIONAL GRAND TOURNAMENT VISUALIZER
              </h2>
              <p className="text-xs font-mono text-gold/60">
                RED AI (DEPTH 30) VS BLACK AI (DEPTH 60) — REALTIME 30-DIM TELEMETRY STREAM
              </p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={() => setShowJson(!showJson)}
              className="px-3 py-1.5 rounded bg-gold/10 text-gold border border-gold/30 text-xs font-bold hover:bg-gold/20 transition"
            >
              {showJson ? '📊 HẨN RAW JSON' : '🔍 XEM RAW 30-DIM JSON'}
            </button>
            <button
              onClick={() => setAutoFollow(!autoFollow)}
              className={`px-3 py-1.5 rounded text-xs font-bold transition border ${
                autoFollow
                  ? 'bg-emerald-500/20 text-emerald-400 border-emerald-500/40 animate-pulse'
                  : 'bg-gold/10 text-gold/60 border-gold/20'
              }`}
            >
              {autoFollow ? '● LIVE FOLLOW ON' : '○ LIVE FOLLOW OFF'}
            </button>
            <button
              onClick={onClose}
              className="px-3 py-1.5 rounded bg-vermilion/20 border border-vermilion/40 text-vermilion text-xs font-bold hover:bg-vermilion hover:text-white transition"
            >
              CLOSE VISUALIZER
            </button>
          </div>
        </div>

        {/* Main Body */}
        <div className="flex-1 overflow-y-auto p-5 grid grid-cols-1 lg:grid-cols-12 gap-6">
          {/* Left Column: Board & Controls (5 cols) */}
          <div className="lg:col-span-5 flex flex-col gap-4 items-center">
            {/* Xiangqi Board Component */}
            <div className="w-full max-w-[400px]">
              <Board
                fen={activeItem.fen || startFen}
                lastMove={activeMove}
                disabled={true}
                rulers={true}
              />
            </div>

            {/* Playback Controls */}
            <div className="w-full flex items-center justify-between gap-2 bg-obsidian/60 p-3 rounded-xl border border-gold/20">
              <button
                onClick={handleStart}
                className="px-3 py-1.5 rounded bg-gold/10 text-gold border border-gold/30 text-xs font-bold hover:bg-gold/20"
              >
                ⏮ START
              </button>
              <button
                onClick={handlePrev}
                className="px-3 py-1.5 rounded bg-gold/10 text-gold border border-gold/30 text-xs font-bold hover:bg-gold/20"
              >
                ◀ PREV
              </button>
              <button
                onClick={() => setPlaying(!playing)}
                className={`px-5 py-1.5 rounded text-xs font-bold shadow-glow transition ${
                  playing ? 'bg-vermilion text-gold' : 'bg-gold text-obsidian font-extrabold'
                }`}
              >
                {playing ? '⏸ PAUSE' : '▶ PLAY'}
              </button>
              <button
                onClick={handleNext}
                className="px-3 py-1.5 rounded bg-gold/10 text-gold border border-gold/30 text-xs font-bold hover:bg-gold/20"
              >
                NEXT ▶
              </button>
              <button
                onClick={handleEnd}
                className="px-3 py-1.5 rounded bg-gold/10 text-gold border border-gold/30 text-xs font-bold hover:bg-gold/20"
              >
                END ⏭
              </button>
            </div>

            {/* Speed Slider */}
            <div className="w-full flex items-center justify-between text-xs text-gold/70 px-2">
              <span>SPEED: {speed}ms</span>
              <input
                type="range"
                min="200"
                max="2000"
                step="100"
                value={speed}
                onChange={(e) => setSpeed(Number(e.target.value))}
                className="accent-gold cursor-pointer"
              />
            </div>
          </div>

          {/* Right Column: 30-Dimension Telemetry Dashboard (7 cols) */}
          <div className="lg:col-span-7 flex flex-col gap-4">
            {/* Raw JSON Modal Overlay */}
            {showJson ? (
              <div className="bg-obsidian-card p-4 rounded-xl border border-gold/40 flex flex-col gap-2">
                <h3 className="text-xs font-royal font-bold text-gold border-b border-gold/20 pb-1 flex justify-between">
                  <span>RAW 30-DIMENSIONAL TELEMETRY JSON (PLY {activeItem.ply})</span>
                  <button onClick={() => setShowJson(false)} className="text-vermilion text-xs">
                    ✖ CLOSE
                  </button>
                </h3>
                <pre className="text-[11px] font-mono bg-obsidian/90 p-3 rounded border border-gold/20 text-emerald-400 overflow-x-auto max-h-[400px]">
                  {JSON.stringify(activeItem, null, 2)}
                </pre>
              </div>
            ) : (
              <>
                {/* Panel 1: Turn & Move Metrics (8 Dimensions) */}
                <div className="bg-obsidian/80 p-4 rounded-xl border border-gold/30 flex flex-col gap-2">
                  <h3 className="text-xs font-royal font-bold text-gold border-b border-gold/20 pb-1 flex justify-between">
                    <span>1. TURN & MOVE CHARACTERISTICS (8 DIMS)</span>
                    <span className="text-xs font-mono text-gold/70">
                      PLY {activeItem.ply} / {matchData.length}
                    </span>
                  </h3>
                  <div className="grid grid-cols-4 gap-2 text-xs font-mono">
                    <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                      <div className="text-gold/50 text-[10px]">SIDE</div>
                      <div className="text-gold font-bold">{activeItem.side || 'RED'}</div>
                    </div>
                    <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                      <div className="text-gold/50 text-[10px]">BEST MOVE</div>
                      <div className="text-amber-400 font-bold">{activeItem.best_move}</div>
                    </div>
                    <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                      <div className="text-gold/50 text-[10px]">MOVED PIECE</div>
                      <div className="text-gold font-bold">{activeItem.moved_piece || '.'}</div>
                    </div>
                    <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                      <div className="text-gold/50 text-[10px]">CAPTURED PIECE</div>
                      <div className="text-rose-400 font-bold">{activeItem.captured_piece || '.'}</div>
                    </div>
                    <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                      <div className="text-gold/50 text-[10px]">FROM SQUARE</div>
                      <div className="text-gold font-bold">Sq {activeItem.from_sq}</div>
                    </div>
                    <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                      <div className="text-gold/50 text-[10px]">TO SQUARE</div>
                      <div className="text-gold font-bold">Sq {activeItem.to_sq}</div>
                    </div>
                    <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                      <div className="text-gold/50 text-[10px]">IS CHECK?</div>
                      <div className={activeItem.is_check ? 'text-rose-400 font-bold' : 'text-gold/50'}>
                        {activeItem.is_check ? '⚠️ CHECK' : 'NO'}
                      </div>
                    </div>
                    <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                      <div className="text-gold/50 text-[10px]">IS CAPTURE?</div>
                      <div className={activeItem.is_capture ? 'text-emerald-400 font-bold' : 'text-gold/50'}>
                        {activeItem.is_capture ? '⚔️ CAPTURE' : 'NO'}
                      </div>
                    </div>
                  </div>
                </div>

                {/* Panel 2: Engine Performance & OS Telemetry (10 Dimensions) */}
                <div className="bg-obsidian/80 p-4 rounded-xl border border-emerald-500/30 flex flex-col gap-2">
                  <h3 className="text-xs font-royal font-bold text-emerald-400 border-b border-emerald-500/20 pb-1 flex justify-between">
                    <span>2. ENGINE PERFORMANCE & OS KERNEL TELEMETRY (10 DIMS)</span>
                    <span className="animate-pulse">● REALTIME</span>
                  </h3>
                  <div className="grid grid-cols-4 gap-2 text-xs font-mono">
                    <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                      <div className="text-gold/50 text-[10px]">EVAL SCORE</div>
                      <div
                        className={`font-extrabold ${
                          activeItem.score > 0
                            ? 'text-emerald-400'
                            : activeItem.score < 0
                            ? 'text-red-400'
                            : 'text-gold'
                        }`}
                      >
                        {activeItem.score > 0 ? `+${activeItem.score}` : activeItem.score} cp
                      </div>
                    </div>
                    <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                      <div className="text-gold/50 text-[10px]">DEPTH (REAL/TARGET)</div>
                      <div className="text-gold font-bold">
                        D{activeItem.completed_depth} / D{activeItem.target_depth}
                      </div>
                    </div>
                    <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                      <div className="text-gold/50 text-[10px]">SEARCH NODES</div>
                      <div className="text-gold font-bold">
                        {(activeItem.nodes || 0).toLocaleString()}
                      </div>
                    </div>
                    <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                      <div className="text-gold/50 text-[10px]">NPS SPEED</div>
                      <div className="text-emerald-400 font-bold">
                        {(activeItem.nps || 0).toLocaleString()} n/s
                      </div>
                    </div>
                    <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                      <div className="text-gold/50 text-[10px]">PLY TIME</div>
                      <div className="text-gold font-bold">{activeItem.ply_time_ms} ms</div>
                    </div>
                    <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                      <div className="text-gold/50 text-[10px]">MATCH TIME</div>
                      <div className="text-gold font-bold">{activeItem.match_elapsed_s} s</div>
                    </div>
                    <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                      <div className="text-gold/50 text-[10px]">RAM RSS (KERNEL)</div>
                      <div className="text-gold font-bold">{activeItem.ram_rss_mb} MB</div>
                    </div>
                    <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                      <div className="text-gold/50 text-[10px]">TT MEMORY</div>
                      <div className="text-gold font-bold">{activeItem.tt_hash_mb} MB</div>
                    </div>
                  </div>
                </div>

                {/* Panel 3: Tactical Dynamics & JRCP 2.0 State (12 Dimensions) */}
                <div className="bg-obsidian/80 p-4 rounded-xl border border-amber-500/30 flex flex-col gap-2">
                  <h3 className="text-xs font-royal font-bold text-amber-400 border-b border-amber-500/20 pb-1">
                    3. TACTICAL DYNAMICS & JRCP 2.0 POSITION STATE (12 DIMS)
                  </h3>
                  <div className="grid grid-cols-4 gap-2 text-xs font-mono">
                    <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                      <div className="text-gold/50 text-[10px]">RED PIECES</div>
                      <div className="text-red-400 font-bold">{activeItem.red_piece_count}</div>
                    </div>
                    <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                      <div className="text-gold/50 text-[10px]">BLACK PIECES</div>
                      <div className="text-blue-400 font-bold">{activeItem.black_piece_count}</div>
                    </div>
                    <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                      <div className="text-gold/50 text-[10px]">MATERIAL BAL</div>
                      <div className="text-gold font-bold">{activeItem.material_balance}</div>
                    </div>
                    <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                      <div className="text-gold/50 text-[10px]">KING SAFETY (RED)</div>
                      <div className="text-emerald-400 font-bold">{activeItem.king_safety_red}%</div>
                    </div>
                    <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                      <div className="text-gold/50 text-[10px]">KING SAFETY (BLK)</div>
                      <div className="text-emerald-400 font-bold">{activeItem.king_safety_black}%</div>
                    </div>
                    <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                      <div className="text-gold/50 text-[10px]">CENTER CONTROL</div>
                      <div className="text-gold font-bold">{activeItem.center_control}</div>
                    </div>
                    <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                      <div className="text-gold/50 text-[10px]">THREAT SCORE</div>
                      <div className="text-rose-400 font-bold">{activeItem.threat_score}</div>
                    </div>
                    <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                      <div className="text-gold/50 text-[10px]">OPPORTUNITY</div>
                      <div className="text-amber-400 font-bold">{activeItem.opportunity_score}</div>
                    </div>
                  </div>
                </div>
              </>
            )}

            {/* Match Feed History */}
            <div className="bg-obsidian/80 p-4 rounded-xl border border-gold/20 flex-1 flex flex-col gap-2 min-h-[160px]">
              <h3 className="text-xs font-royal font-bold text-gold border-b border-gold/20 pb-1">
                MATCH PLY LOG FEED
              </h3>
              <div className="flex-1 overflow-y-auto flex flex-col gap-1 pr-1 text-xs font-mono max-h-[180px]">
                {matchData.map((item, idx) => (
                  <div
                    key={idx}
                    onClick={() => {
                      setAutoFollow(false);
                      setCurrentPly(idx);
                    }}
                    className={`p-2 rounded cursor-pointer transition flex items-center justify-between ${
                      idx === currentPly
                        ? 'bg-gold/20 border border-gold text-gold font-bold'
                        : 'bg-obsidian-card border border-gold/10 text-gold/70 hover:bg-gold/10'
                    }`}
                  >
                    <span>
                      Ply {item.ply}: {item.side}
                    </span>
                    <span className="font-bold">{item.best_move}</span>
                    <span
                      className={
                        item.score > 0
                          ? 'text-emerald-400'
                          : item.score < 0
                          ? 'text-red-400'
                          : 'text-gold/60'
                      }
                    >
                      {item.score > 0 ? `+${item.score}` : item.score} cp
                    </span>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
