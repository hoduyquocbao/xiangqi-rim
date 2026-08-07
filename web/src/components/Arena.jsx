// web/src/components/Arena.jsx
// Bot Arena (Self-Play) Component for XiangRust Web App
// 100% Single-Word English Identifiers

import React, { useState, useEffect, useRef } from 'react';
import { uciToMove } from '../rules/rules.js';
import { instance as engine } from '../engine/engine.js';

export default function Arena({ fen, turn, move, reset, board = [], check = false }) {
  const [state, update] = useState({
    run: false,
    red: 6,
    black: 8,
    speed: 1000,
    nps: 0,
    nodes: 0,
    time: 0,
    score: 0,
    prev: 0,
    blunder: false,
    wins: {
      red: 0,
      black: 0,
      draws: 0
    },
    feed: []
  });

  const timer = useRef(null);
  const clock = useRef(0);
  const historyRef = useRef([]);

  // Clear history on board reset
  const handleReset = () => {
    historyRef.current = [];
    if (reset) reset();
  };

  // Trigger one bot search step
  const step = () => {
    const depth = turn === 'w' ? state.red : state.black;
    clock.current = performance.now();
    engine.position(fen);
    engine.search(depth, 3000, historyRef.current);
  };

  // Listen to engine search events
  useEffect(() => {
    const off = engine.listen((type, data) => {
      if (type === 'search' && data) {
        const ms = Math.max(1, Math.round(performance.now() - clock.current));
        const count = data.nodes || 0;
        const rate = data.nps || Math.round(count / (ms / 1000));
        const score = data.score || 0;

        // Blunder detection: centipawn score drop > 150
        const drop = turn === 'w' ? state.prev - score : score - state.prev;
        const blunder = drop > 150;

        const best = data.bestmove || data.best || null;
        const item = {
          turn,
          move: best || 'N/A',
          score,
          drop,
          blunder,
          nps: rate,
          nodes: count,
          time: ms
        };

        // 3-Fold Repetition Detection in Self-Play History
        const currentFenBoard = fen.split(' ')[0];
        historyRef.current.push(currentFenBoard);
        const occurrences = historyRef.current.filter((f) => f === currentFenBoard).length;

        if (occurrences >= 3) {
          update((prev) => ({
            ...prev,
            run: false,
            nps: rate,
            nodes: prev.nodes + count,
            time: ms,
            score,
            blunder,
            wins: {
              ...prev.wins,
              draws: prev.wins.draws + 1
            },
            feed: [
              { ...item, move: 'DRAW (3-FOLD REPETITION)' },
              ...prev.feed.slice(0, 49)
            ]
          }));
          engine.stop();
          return;
        }

        update((prev) => ({
          ...prev,
          nps: rate,
          nodes: prev.nodes + count,
          time: ms,
          score,
          prev: score,
          blunder,
          feed: [item, ...prev.feed.slice(0, 49)]
        }));

        if (best) {
          const pair = uciToMove(best);
          if (pair && move) {
            move(pair.from, pair.to);
          }
        }
      }
    });

    return () => off();
  }, [fen, turn, board, state.prev, move]);

  // Auto self-play loop
  useEffect(() => {
    if (state.run) {
      timer.current = setTimeout(() => {
        step();
      }, state.speed);
    }
    return () => {
      if (timer.current) clearTimeout(timer.current);
    };
  }, [state.run, fen, state.speed]);

  const start = () => {
    update((prev) => ({ ...prev, run: true }));
  };

  const pause = () => {
    update((prev) => ({ ...prev, run: false }));
    engine.stop();
  };

  const clear = () => {
    historyRef.current = [];
    update((prev) => ({
      ...prev,
      run: false,
      nps: 0,
      nodes: 0,
      time: 0,
      score: 0,
      prev: 0,
      blunder: false,
      feed: []
    }));
    if (reset) reset();
  };

  return (
    <div className="glass rounded-xl p-5 border border-gold/20 flex flex-col gap-5 shadow-glow w-full">
      {/* Title Bar */}
      <h2 className="text-base font-royal font-bold text-gold border-b border-gold/20 pb-2 flex items-center justify-between">
        <span>BOT ARENA (SELF-PLAY)</span>
        <span className={`text-xs font-mono font-semibold uppercase px-2 py-0.5 rounded ${
          state.run ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/40 animate-pulse' : 'bg-gold/10 text-gold/60 border border-gold/20'
        }`}>
          {state.run ? 'AUTO ARENA RUNNING' : 'PAUSED'}
        </span>
      </h2>

      {/* Control Buttons */}
      <div className="grid grid-cols-3 gap-2">
        <button
          onClick={state.run ? pause : start}
          className={`py-2.5 rounded text-xs font-bold transition-all shadow-glow flex items-center justify-center gap-1 ${
            state.run
              ? 'bg-vermilion text-gold border border-vermilion hover:bg-vermilion/80'
              : 'bg-gold text-obsidian border border-gold hover:bg-gold/90'
          }`}
        >
          {state.run ? 'PAUSE ARENA' : 'START SELF-PLAY'}
        </button>

        <button
          onClick={step}
          disabled={state.run}
          className="py-2.5 rounded bg-obsidian-card border border-gold/40 text-xs font-bold text-gold hover:bg-gold/10 hover:border-gold disabled:opacity-40 transition flex items-center justify-center gap-1"
        >
          STEP (1 MOVE)
        </button>

        <button
          onClick={clear}
          className="py-2.5 rounded bg-obsidian-card border border-gold/30 text-xs font-bold text-gold/80 hover:border-gold hover:text-gold transition flex items-center justify-center gap-1"
        >
          RESET ARENA
        </button>
      </div>

      {/* Bot Depth Selectors */}
      <div className="grid grid-cols-2 gap-4 bg-obsidian-card p-3 rounded-lg border border-gold/20">
        <div className="flex flex-col gap-1.5">
          <div className="flex justify-between items-center text-xs">
            <span className="text-vermilion font-bold flex items-center gap-1">
              <span className="w-2 h-2 rounded-full bg-vermilion inline-block"></span> RED BOT
            </span>
            <span className="text-gold font-mono font-bold text-xs bg-gold/10 px-1.5 py-0.5 rounded border border-gold/30">
              D{state.red}
            </span>
          </div>
          <input
            type="range"
            min="4"
            max="12"
            value={state.red}
            disabled={state.run}
            onChange={(e) => update((prev) => ({ ...prev, red: Number(e.target.value) }))}
            className="w-full accent-gold bg-obsidian h-1.5 rounded cursor-pointer"
          />
        </div>

        <div className="flex flex-col gap-1.5">
          <div className="flex justify-between items-center text-xs">
            <span className="text-gold font-bold flex items-center gap-1">
              <span className="w-2 h-2 rounded-full bg-gold inline-block"></span> BLACK BOT
            </span>
            <span className="text-gold font-mono font-bold text-xs bg-gold/10 px-1.5 py-0.5 rounded border border-gold/30">
              D{state.black}
            </span>
          </div>
          <input
            type="range"
            min="4"
            max="12"
            value={state.black}
            disabled={state.run}
            onChange={(e) => update((prev) => ({ ...prev, black: Number(e.target.value) }))}
            className="w-full accent-gold bg-obsidian h-1.5 rounded cursor-pointer"
          />
        </div>
      </div>

      {/* Realtime Metrics Cards */}
      <div className="grid grid-cols-3 gap-2">
        <div className="bg-obsidian-card p-2.5 rounded border border-gold/20 flex flex-col">
          <span className="text-[10px] text-gold/60 font-semibold uppercase">SPEED (NPS)</span>
          <span className="text-sm font-mono font-bold text-emerald-400">
            {state.nps > 0 ? (state.nps / 1000000).toFixed(2) + 'M' : '0'}
          </span>
        </div>
        <div className="bg-obsidian-card p-2.5 rounded border border-gold/20 flex flex-col">
          <span className="text-[10px] text-gold/60 font-semibold uppercase">SEARCH NODES</span>
          <span className="text-sm font-mono font-bold text-gold">
            {state.nodes > 0 ? (state.nodes / 1000).toFixed(0) + 'K' : '0'}
          </span>
        </div>
        <div className="bg-obsidian-card p-2.5 rounded border border-gold/20 flex flex-col">
          <span className="text-[10px] text-gold/60 font-semibold uppercase">BLUNDER DROP</span>
          <span className={`text-sm font-mono font-bold ${state.blunder ? 'text-vermilion animate-pulse' : 'text-emerald-400'}`}>
            {state.blunder ? 'BLUNDER!' : 'STABLE'}
          </span>
        </div>
      </div>

      {/* Live Move Feed Log Stream */}
      <div className="flex flex-col gap-2">
        <h3 className="text-xs font-royal font-bold text-gold/80 flex items-center justify-between">
          <span>LIVE ARENA MOVE FEED</span>
          <span className="text-[10px] text-gold/40 font-mono">
            DRAWS: {state.wins.draws}
          </span>
        </h3>
        <div className="max-h-48 overflow-y-auto font-mono text-xs flex flex-col gap-1 pr-1 bg-obsidian/80 p-2.5 rounded border border-gold/20">
          {state.feed.length === 0 ? (
            <div className="text-gold/40 text-center py-4 text-[11px]">
              Click START SELF-PLAY to watch bots battle automatically
            </div>
          ) : (
            state.feed.map((item, idx) => (
              <div
                key={`feed-${idx}`}
                className={`flex items-center justify-between px-2 py-1 rounded text-[11px] border ${
                  item.blunder
                    ? 'bg-vermilion/20 border-vermilion/40 text-vermilion font-bold'
                    : item.turn === 'w'
                    ? 'bg-gold/10 border-gold/30 text-gold'
                    : 'bg-obsidian-card border-gold/20 text-gold/80'
                }`}
              >
                <div className="flex items-center gap-2">
                  <span className={`w-2 h-2 rounded-full ${item.turn === 'w' ? 'bg-vermilion' : 'bg-gold'}`}></span>
                  <span className="font-bold">{item.move}</span>
                  {item.blunder && (
                    <span className="text-[9px] bg-vermilion text-gold px-1 rounded font-bold">
                      -{item.drop}cp
                    </span>
                  )}
                </div>
                <div className="flex items-center gap-3 text-[10px] text-gold/60">
                  <span>{item.score > 0 ? `+${item.score}` : item.score}cp</span>
                  <span>{(item.nps / 1000000).toFixed(2)}M/s</span>
                  <span>{item.time}ms</span>
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
