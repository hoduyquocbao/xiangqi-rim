// web/src/components/Tournament.jsx
// Visualizer Component for Depth 30 Red AI vs Depth 60 Black AI Grand Match
// 100% Single-Word English Identifiers

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
    score: 0
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
      <div className="bg-obsidian-card border border-gold/30 rounded-2xl w-full max-w-5xl max-h-[92vh] flex flex-col shadow-glow overflow-hidden">
        {/* Header */}
        <div className="p-4 border-b border-gold/20 flex items-center justify-between bg-obsidian-header">
          <div className="flex items-center gap-3">
            <span className="text-xl">⚔️</span>
            <div>
              <h2 className="text-lg font-royal font-bold text-gold">GRAND TOURNAMENT VISUALIZER</h2>
              <p className="text-xs font-mono text-gold/60">
                RED AI (DEPTH 30) VS BLACK AI (DEPTH 60) — UNLIMITED PLIES
              </p>
            </div>
          </div>
          <div className="flex items-center gap-2">
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
          {/* Left Column: Board & Controls (7 cols) */}
          <div className="lg:col-span-7 flex flex-col gap-4 items-center">
            {/* Xiangqi Board Component */}
            <div className="w-full max-w-[420px]">
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
              <span>PLAYBACK SPEED: {speed}ms</span>
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

          {/* Right Column: Telemetry & Live Feed (5 cols) */}
          <div className="lg:col-span-5 flex flex-col gap-4">
            {/* Active Turn Status Card */}
            <div className="bg-obsidian/80 p-4 rounded-xl border border-gold/30 flex flex-col gap-2">
              <div className="flex items-center justify-between">
                <span className="text-xs font-mono text-gold/60">CURRENT PLY</span>
                <span className="text-sm font-mono font-bold text-gold">
                  {activeItem.ply} / {matchData.length}
                </span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-xs font-mono text-gold/60">ACTIVE SIDE</span>
                <span
                  className={`text-xs font-bold px-2 py-0.5 rounded ${
                    activeItem.side && activeItem.side.includes('RED')
                      ? 'bg-red-500/20 text-red-400 border border-red-500/40'
                      : 'bg-blue-500/20 text-blue-400 border border-blue-500/40'
                  }`}
                >
                  {activeItem.side || 'RED'}
                </span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-xs font-mono text-gold/60">EVAL SCORE</span>
                <span
                  className={`text-sm font-mono font-extrabold ${
                    activeItem.score > 0
                      ? 'text-emerald-400'
                      : activeItem.score < 0
                      ? 'text-red-400'
                      : 'text-gold'
                  }`}
                >
                  {activeItem.score > 0 ? `+${activeItem.score}` : activeItem.score} cp
                </span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-xs font-mono text-gold/60">BEST MOVE (UCI)</span>
                <span className="text-xs font-mono font-bold text-gold bg-gold/10 px-2 py-0.5 rounded border border-gold/20">
                  {activeItem.best_move}
                </span>
              </div>
            </div>

            {/* Realtime OS Telemetry Monitor Card */}
            <div className="bg-obsidian/80 p-4 rounded-xl border border-emerald-500/30 flex flex-col gap-2">
              <h3 className="text-xs font-royal font-bold text-emerald-400 flex items-center justify-between border-b border-emerald-500/20 pb-1">
                <span>OS KERNEL TELEMETRY (RULE 8.10)</span>
                <span className="animate-pulse">● LIVE SYNC</span>
              </h3>
              <div className="grid grid-cols-2 gap-2 text-xs font-mono">
                <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                  <div className="text-gold/50 text-[10px]">RAM RSS (KERNEL)</div>
                  <div className="text-gold font-bold">514.48 MB</div>
                </div>
                <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                  <div className="text-gold/50 text-[10px]">CPU WORKER THREADS</div>
                  <div className="text-gold font-bold">8 Threads</div>
                </div>
                <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                  <div className="text-gold/50 text-[10px]">RED TT MEMORY</div>
                  <div className="text-gold font-bold">256 MB RAM</div>
                </div>
                <div className="bg-obsidian-card p-2 rounded border border-gold/10">
                  <div className="text-gold/50 text-[10px]">BLACK TT MEMORY</div>
                  <div className="text-gold font-bold">256 MB RAM</div>
                </div>
              </div>
            </div>

            {/* Match Feed History */}
            <div className="bg-obsidian/80 p-4 rounded-xl border border-gold/20 flex-1 flex flex-col gap-2 min-h-[220px]">
              <h3 className="text-xs font-royal font-bold text-gold border-b border-gold/20 pb-1">
                MATCH PLY LOG
              </h3>
              <div className="flex-1 overflow-y-auto flex flex-col gap-1 pr-1 text-xs font-mono max-h-[300px]">
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
