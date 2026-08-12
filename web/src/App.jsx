// Khung giao diện chính ứng dụng Web App Cờ Tướng Hoàng Gia (XiangRust) - Milestone 5
// Định danh đơn từ tiếng Anh: App, game, update, play, type, place, capture, check, win, fen, flip, move, history, reset, undo, redo, turn, parsed, start, target, taking, cloned, next, built, sliced, item, idx, checked, sound, score, line, depth, mode, status, show, active, hint, engine, pick, level, apply, index, off, prev, view, board, debug, Debugger, Arena, Board, Eval, Explorer, Panel, Modal, piece, validMoves, pieceColor, rulers, toggleRulers

import React, { useState, useEffect } from 'react';
import Board from './components/Board.jsx';
import Eval from './components/Eval.jsx';
import Explorer from './components/Explorer.jsx';
import Panel from './components/Panel.jsx';
import Modal from './components/Modal.jsx';
import Arena from './components/Arena.jsx';
import History from './components/History.jsx';
import Gym from './components/Gym.jsx';
import Audit from './components/Audit.jsx';
import Studio from './components/Studio.jsx';
import Tournament from './components/Tournament.jsx';
import { R1Studio } from './components/R1Studio.jsx';
import { Debugger } from './components/Debugger.jsx';
import { parse, fen, check, moves as getMoves, uciToMove, hasLegalMoves } from './rules/rules.js';
import * as sound from './sound/audio.js';
import * as store from './storage/store.js';
import { instance as engine } from './engine/engine.js';

const start = 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1';

export default function App() {
  const [game, update] = useState({
    mode: 'wasm',
    view: 'play',
    status: 'ready',
    bot: true,
    playMode: 'ai',
    fen: start,
    flip: false,
    rulers: true,
    history: [start],
    index: 0,
    depth: 6,
    hash: 256,
    score: 0,
    line: [],
    show: false,
    debug: false,
    historyShow: false,
    gymShow: false,
    auditShow: false,
    studioShow: false,
    r1Show: false,
    tournamentShow: false,
    active: null,
    hint: null,
    over: false,
    winner: null,
    reason: null,
    thought: null
  });

  const parsed = parse(game.fen);
  const board = parsed.board;
  const turn = parsed.turn;
  const checked = check(board, turn);

  // Khởi tạo và đăng ký sự kiện từ Engine Facade
  useEffect(() => {
    engine.init(game.mode);

    const off = engine.listen((type, data) => {
      if (type === 'search') {
        if (data) {
          const score = data.score || 0;
          const best = data.bestmove || data.best || null;
          const pvLine = data.pv || (best ? [best] : []);

          update((prev) => {
            const currentTurn = parse(prev.fen).turn;
            const parsedBoard = parse(prev.fen).board;
            const validExist = hasLegalMoves(parsedBoard, currentTurn);

            // Kiểm tra bị chiếu bí (Checkmate) hoặc hết nước đi (Stalemate) thực tế trên bàn cờ
            if (!validExist || (!best && !validExist)) {
              const isMated = check(parsedBoard, currentTurn);
              const winner = isMated ? (currentTurn === 'w' ? 'b' : 'w') : 'draw';
              const reason = isMated ? 'CHECKMATE' : 'STALEMATE';

              if (prev.status !== 'ready' || !prev.over) {
                sound.win();
                store.save({
                  mode: prev.mode,
                  depth: prev.depth,
                  fen: prev.fen,
                  history: prev.history,
                  over: true,
                  winner,
                  reason
                });
              }

              return {
                ...prev,
                score,
                line: [],
                hint: null,
                status: 'ready',
                over: true,
                winner,
                reason
              };
            }

            const nextState = {
              ...prev,
              score,
              line: pvLine,
              hint: best,
              status: 'ready',
              thought: data.thought || prev.thought
            };

            // Nếu đang trong chế độ AI Bot và AI vừa tính toán nước đi tốt nhất cho lượt Đen
            if (prev.bot && prev.view === 'play' && best && !prev.over) {
              if (currentTurn === 'b') {
                const targetMove = uciToMove(best);
                if (targetMove) {
                  const piece = parsedBoard[targetMove.from];

                  if (piece !== '.' && piece === piece.toLowerCase()) {
                    const targetPiece = parsedBoard[targetMove.to];
                    const taking = targetPiece !== '.';

                    const cloned = [...parsedBoard];
                    cloned[targetMove.to] = piece;
                    cloned[targetMove.from] = '.';

                    const builtFen = fen(cloned, 'w');
                    const newHistory = prev.history.slice(0, prev.index + 1);
                    newHistory.push(builtFen);

                    engine.position(builtFen);

                    const nextLegal = hasLegalMoves(cloned, 'w');
                    let gameOver = false;
                    let winner = null;
                    let reason = null;

                    if (!nextLegal) {
                      gameOver = true;
                      const isMated = check(cloned, 'w');
                      winner = isMated ? 'b' : 'draw';
                      reason = isMated ? 'CHECKMATE' : 'STALEMATE';
                      sound.win();
                    } else {
                      if (taking) sound.capture();
                      else sound.place();

                      if (check(cloned, 'w')) sound.check();
                    }

                    const recordState = {
                      ...nextState,
                      fen: builtFen,
                      history: newHistory,
                      index: newHistory.length - 1,
                      over: gameOver,
                      winner,
                      reason
                    };

                    store.save({
                      mode: prev.mode,
                      depth: prev.depth,
                      fen: builtFen,
                      history: newHistory,
                      over: gameOver,
                      winner,
                      reason
                    });

                    return recordState;
                  }
                }
              }
            }

            return nextState;
          });
        }
      } else if (type === 'eval') {
        update((prev) => ({
          ...prev,
          score: typeof data === 'number' ? data : 0
        }));
      }
    });

    return () => off();
  }, [game.mode]);

  // Tự động kích hoạt AI tìm kiếm khi tới lượt Đen trong chế độ Bot (chỉ khi ván chưa kết thúc)
  useEffect(() => {
    if (game.bot && turn === 'b' && game.status === 'ready' && !game.over && game.view === 'play') {
      const currentBoard = parse(game.fen).board;
      if (!hasLegalMoves(currentBoard, 'b')) {
        const isMated = check(currentBoard, 'b');
        sound.win();
        update((prev) => ({
          ...prev,
          over: true,
          winner: isMated ? 'w' : 'draw',
          reason: isMated ? 'CHECKMATE' : 'STALEMATE',
          status: 'ready'
        }));
        return;
      }
      update((prev) => ({ ...prev, status: 'searching' }));
      engine.position(game.fen);
      engine.search(game.depth, 2000, game.history);
    }
  }, [game.fen, game.bot, turn, game.status, game.view, game.depth, game.over]);

  // Kích hoạt phát hiệu ứng âm thanh
  const play = (type) => {
    if (type === 'place') sound.place();
    else if (type === 'capture') sound.capture();
    else if (type === 'check') sound.check();
    else if (type === 'win') sound.win();
  };

  // Thực hiện nước đi mới trên bàn cờ từ Người chơi
  const move = (from, to) => {
    if (game.over) return;
    const piece = board[from];
    if (piece === '.') return;

    const pieceColor = piece === piece.toUpperCase() ? 'w' : 'b';
    if (pieceColor !== turn) return;

    // Kiểm tra nước đi của Người chơi có thuộc danh sách nước đi hợp lệ theo luật cờ tướng không
    const validMoves = getMoves(board, from, turn);
    if (!validMoves.includes(to)) return;

    const target = board[to];
    const taking = target !== '.';

    const cloned = [...board];
    cloned[to] = cloned[from];
    cloned[from] = '.';

    const next = turn === 'w' ? 'b' : 'w';
    const built = fen(cloned, next);

    const history = game.history.slice(0, game.index + 1);
    history.push(built);

    const hasNextMoves = hasLegalMoves(cloned, next);
    let over = false;
    let winner = null;
    let reason = null;

    if (!hasNextMoves) {
      over = true;
      const isMated = check(cloned, next);
      winner = isMated ? turn : 'draw';
      reason = isMated ? 'CHECKMATE' : 'STALEMATE';
      sound.win();
    } else {
      if (taking) play('capture');
      else play('place');

      if (check(cloned, next)) play('check');
    }

    update((prev) => {
      const nextState = {
        ...prev,
        fen: built,
        history,
        index: history.length - 1,
        hint: null,
        line: [],
        active: null,
        over,
        winner,
        reason
      };

      store.save({
        mode: prev.mode,
        depth: prev.depth,
        fen: built,
        history,
        over,
        winner,
        reason
      });

      return nextState;
    });

    engine.position(built);
  };

  // Hoàn nước đi (Undo)
  const undo = () => {
    if (game.index > 0) {
      const idx = game.index - 1;
      const built = game.history[idx];
      update((prev) => ({
        ...prev,
        fen: built,
        index: idx,
        hint: null,
        line: [],
        active: null,
        over: false,
        winner: null,
        reason: null
      }));
      engine.position(built);
    }
  };

  // Đi tiếp nước đi sau đó (Redo)
  const redo = () => {
    if (game.index < game.history.length - 1) {
      const idx = game.index + 1;
      const built = game.history[idx];
      update((prev) => ({
        ...prev,
        fen: built,
        index: idx,
        hint: null,
        line: [],
        active: null
      }));
      engine.position(built);
    }
  };

  // Đặt lại ván đấu mới (Reset)
  const reset = () => {
    update((prev) => ({
      ...prev,
      fen: start,
      history: [start],
      index: 0,
      score: 0,
      line: [],
      hint: null,
      active: null,
      status: 'ready',
      over: false,
      winner: null,
      reason: null
    }));
    engine.position(start);
  };

  // Nạp ván đấu từ lịch sử hoặc QA/QC GYM với khả năng phát lại từng nước (Ply-by-Ply Replay)
  const loadMatch = (match) => {
    if (!match) return;

    let targetFen = start;
    let historyList = [start];

    if (typeof match === 'string') {
      targetFen = match;
      historyList = [match];
    } else if (match.history && Array.isArray(match.history)) {
      historyList = match.history;
      targetFen = match.fen || match.history[match.history.length - 1];
    } else if (match.moves && Array.isArray(match.moves) && match.moves.length > 0) {
      let currFen = start;
      historyList = [start];

      for (let i = 0; i < match.moves.length; i++) {
        const uci = match.moves[i];
        const coords = uciToMove(uci);
        if (!coords) break;

        const parsed = parse(currFen);
        const cloned = [...parsed.board];
        cloned[coords.to] = cloned[coords.from];
        cloned[coords.from] = '.';
        const nextTurn = parsed.turn === 'w' ? 'b' : 'w';
        currFen = fen(cloned, nextTurn);
        historyList.push(currFen);
      }

      targetFen = historyList[historyList.length - 1];
    } else if (match.fen) {
      targetFen = match.fen;
      historyList = [match.fen];
    }

    update((prev) => ({
      ...prev,
      fen: targetFen,
      history: historyList,
      index: historyList.length - 1,
      over: match.over || false,
      winner: match.winner || null,
      reason: match.reason || null,
      hint: null,
      line: [],
      active: null
    }));
    engine.position(targetFen);
  };

  // Kích hoạt AI tính toán nước đi (Search)
  const search = () => {
    if (game.over) return;
    update((prev) => ({ ...prev, status: 'searching' }));
    engine.position(game.fen);
    engine.search(game.depth, 3000, game.history);
  };

  // Dừng tính toán AI (Stop)
  const stop = () => {
    engine.stop();
    update((prev) => ({ ...prev, status: 'ready' }));
  };

  // Nhấp chọn nước đi trong cây PV Explorer
  const pick = (idx, item) => {
    update((prev) => ({ ...prev, active: idx }));
  };

  // Thay đổi cấp độ độ sâu AI
  const level = (val) => {
    update((prev) => ({ ...prev, depth: val }));
  };

  // Cấp phát dung lượng Hash RAM Transposition Table
  const alloc = (val) => {
    update((prev) => ({ ...prev, hash: val }));
    engine.hash(val);
  };

  // Chuyển đổi giữa 3 chế độ: Huấn Luyện AI | Chơi 2 Người | Sắp Cờ
  const setPlayMode = (mode) => {
    if (mode === 'ai') {
      update((prev) => ({ ...prev, playMode: 'ai', bot: true }));
    } else if (mode === 'pvp') {
      update((prev) => ({ ...prev, playMode: 'pvp', bot: false }));
    } else if (mode === 'editor') {
      update((prev) => ({ ...prev, playMode: 'editor', bot: false, studioShow: true }));
    }
  };

  // Áp dụng dữ liệu nạp từ Modal (FEN hoặc PGN)
  const apply = (type, data) => {
    if (type === 'fen' && typeof data === 'string') {
      update((prev) => ({
        ...prev,
        fen: data,
        history: [data],
        index: 0,
        hint: null,
        line: [],
        active: null,
        over: false,
        winner: null,
        reason: null
      }));
      engine.position(data);
    } else if (type === 'pgn' && data && data.moves) {
      reset();
    }
  };

  return (
    <div className="min-h-screen bg-obsidian text-gold flex flex-col font-body select-none">
      {/* Header Hoàng Gia */}
      <header className="border-b border-gold/20 bg-obsidian-card/80 glass px-6 py-4 flex items-center justify-between shadow-glow flex-wrap gap-4">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-full border-2 border-gold flex items-center justify-center bg-obsidian shadow-glow font-royal font-bold text-xl text-gold">
            將
          </div>
          <div>
            <h1 className="text-2xl font-royal font-bold tracking-wider text-gold drop-shadow">
              XIANGRUST
            </h1>
            <p className="text-xs text-gold/60 font-body">
              Royal Imperial Xiangqi Engine Analysis & Bot Arena
            </p>
          </div>
        </div>

        {/* View Mode Switcher (Play / Analysis vs Bot Arena) */}
        <div className="flex items-center gap-2 bg-obsidian-card p-1 rounded-lg border border-gold/20">
          <button
            onClick={() => update((prev) => ({ ...prev, view: 'play' }))}
            className={`px-3 py-1.5 rounded text-xs font-bold transition-all ${
              game.view === 'play'
                ? 'bg-gold text-obsidian shadow-glow font-bold'
                : 'text-gold/70 hover:text-gold'
            }`}
          >
            PLAY / ANALYSIS
          </button>
          <button
            onClick={() => update((prev) => ({ ...prev, view: 'arena' }))}
            className={`px-3 py-1.5 rounded text-xs font-bold transition-all ${
              game.view === 'arena'
                ? 'bg-gold text-obsidian shadow-glow font-bold'
                : 'text-gold/70 hover:text-gold'
            }`}
          >
            BOT ARENA (SELF-PLAY)
          </button>
        </div>

        {/* Dual-Engine Mode Switcher, Telemetry Debugger, R1 Studio & History Button */}
        <div className="flex items-center gap-2 flex-wrap justify-end">
          <button
            onClick={() => update((prev) => ({ ...prev, tournamentShow: true }))}
            className="px-3 py-2 rounded bg-gradient-to-r from-red-600/30 via-amber-500/30 to-blue-600/30 text-gold border border-gold/60 hover:border-gold text-xs font-bold transition flex items-center gap-1.5 shadow-glow animate-pulse"
          >
            <span>⚔️</span>
            GIẢI ĐẤU (D30 vs D60)
          </button>

          <button
            onClick={() => update((prev) => ({ ...prev, r1Show: true }))}
            className="px-3 py-2 rounded bg-gradient-to-r from-gold/20 via-amber-500/20 to-gold/20 text-gold border border-gold/50 hover:border-gold text-xs font-bold transition flex items-center gap-1.5 shadow-glow"
          >
            <span>🤖</span>
            XIANGQI-R1 GRPO STUDIO
          </button>

          <button
            onClick={() => update((prev) => ({ ...prev, historyShow: true }))}
            className="px-3 py-2 rounded bg-obsidian-card text-gold border border-gold/40 hover:bg-gold/10 text-xs font-bold transition flex items-center gap-1.5 shadow-glow"
          >
            <span>📜</span>
            NHẬT KÝ VÁN ĐẤU
          </button>

          <button
            onClick={() => update((prev) => ({ ...prev, debug: true }))}
            className="px-3 py-2 rounded bg-obsidian-card text-emerald-400 border border-emerald-500/40 hover:bg-emerald-500/10 text-xs font-bold transition flex items-center gap-1.5 shadow-glow"
          >
            <span className="w-2 h-2 rounded-full bg-emerald-400 animate-ping"></span>
            TELEMETRY & DEBUGGER
          </button>

          <button
            onClick={() => {
              update((prev) => ({ ...prev, mode: 'llm' }));
              engine.mode('llm');
            }}
            className={`px-3 py-2 rounded border text-xs font-semibold transition-all ${
              game.mode === 'llm'
                ? 'bg-amber-400 text-obsidian border-amber-400 shadow-glow font-bold animate-pulse'
                : 'bg-amber-950/30 text-amber-300 border-amber-500/40 hover:border-amber-400'
            }`}
          >
            🤖 R1 LLM 0.5B (Batch 3)
          </button>
          <button
            onClick={() => {
              update((prev) => ({ ...prev, mode: 'wasm' }));
              engine.mode('wasm');
            }}
            className={`px-3 py-2 rounded border text-xs font-semibold transition-all ${
              game.mode === 'wasm'
                ? 'bg-gold text-obsidian border-gold shadow-glow font-bold'
                : 'bg-obsidian-card text-gold/70 border-gold/30 hover:border-gold'
            }`}
          >
            WASM Client (0ms)
          </button>
          <button
            onClick={() => {
              update((prev) => ({ ...prev, mode: 'socket' }));
              engine.mode('socket');
            }}
            className={`px-3 py-2 rounded border text-xs font-semibold transition-all ${
              game.mode === 'socket'
                ? 'bg-gold text-obsidian border-gold shadow-glow font-bold'
                : 'bg-obsidian-card text-gold/70 border-gold/30 hover:border-gold'
            }`}
          >
            WebSocket Server
          </button>
          <button
            onClick={() => {
              update((prev) => ({ ...prev, mode: 'hybrid' }));
              engine.mode('hybrid');
            }}
            className={`px-3 py-2 rounded border text-xs font-semibold transition-all ${
              game.mode === 'hybrid'
                ? 'bg-emerald-400 text-obsidian border-emerald-400 shadow-glow font-bold animate-pulse'
                : 'bg-obsidian-card text-emerald-400/80 border-emerald-500/40 hover:border-emerald-400'
            }`}
          >
            🌐 Hybrid (Local + Server)
          </button>
        </div>
      </header>

      {/* P2P Network Mesh & High-Capacity Storage Global Status Sub-Bar */}
      <div className="bg-obsidian/90 border-b border-gold/20 px-6 py-1.5 flex items-center justify-between text-[11px] font-mono text-gold/80 flex-wrap gap-2">
        <div className="flex items-center gap-3 flex-wrap">
          <span className="flex items-center gap-1.5 text-amber-300 font-bold bg-amber-500/10 px-2 py-0.5 rounded border border-amber-500/30">
            <span className="w-2 h-2 rounded-full bg-amber-400 animate-pulse"></span>
            🤖 MODEL: hoduyquocbao/xiangqi-r1-0.5b (Batch 3 — 300 Steps GRPO Merged)
          </span>
          <span className="text-gold/40">|</span>
          <span className="flex items-center gap-1.5 text-emerald-400 font-bold">
            <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
            P2P MESH: sha256(mesh2026) [24/7]
          </span>
          <span className="text-gold/40">|</span>
          <span className="text-purple-300 font-bold">
            💾 STORAGE: IndexedDB High-Capacity
          </span>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-gold/60 uppercase">ACTIVE ENGINE:</span>
          <span className="px-2 py-0.5 rounded bg-gold/20 text-gold border border-gold/40 font-bold uppercase">
            {game.mode}
          </span>
        </div>
      </div>

      {/* Dynamic View Layout */}
      {game.view === 'arena' ? (
        <main className="flex-1 max-w-7xl w-full mx-auto p-6 grid grid-cols-1 lg:grid-cols-12 gap-6">
          <section className="lg:col-span-5 flex flex-col gap-4">
            <Arena
              fen={game.fen}
              turn={turn}
              move={move}
              reset={reset}
              board={board}
              check={checked}
            />
          </section>
          <section className="lg:col-span-7 flex flex-col items-center justify-center">
            <Board fen={game.fen} move={move} flip={game.flip} check={checked} rulers={game.rulers} />
          </section>
        </main>
      ) : (
        <main className="flex-1 max-w-7xl w-full mx-auto p-6 grid grid-cols-1 lg:grid-cols-12 gap-6">
          {/* Cột 1: Evaluation & Control Panel */}
          <section className="lg:col-span-3 flex flex-col gap-4">
            <Eval score={game.score} />

            <Panel
              depth={game.depth}
              level={level}
              hash={game.hash}
              alloc={alloc}
              playMode={game.playMode || (game.bot ? 'ai' : 'pvp')}
              setPlayMode={setPlayMode}
              hint={search}
              undo={undo}
              redo={redo}
              flip={() => update((prev) => ({ ...prev, flip: !prev.flip }))}
              rulers={game.rulers}
              toggleRulers={() => update((prev) => ({ ...prev, rulers: !prev.rulers }))}
              open={() => update((prev) => ({ ...prev, show: true }))}
              openGym={() => update((prev) => ({ ...prev, gymShow: true }))}
              openAudit={() => update((prev) => ({ ...prev, auditShow: true }))}
              undoable={game.index > 0}
              redoable={game.index < game.history.length - 1}
              status={game.status}
              search={search}
              stop={stop}
            />
          </section>

          {/* Cột 2: Interactive Graphic Xiangqi Board */}
          <section className="lg:col-span-6 flex flex-col items-center justify-center gap-4">
            {game.over && (
              <div className="w-full bg-red-950/80 border-2 border-gold text-gold p-4 rounded-xl text-center space-y-2 animate-bounce shadow-2xl">
                <h2 className="text-xl font-bold tracking-wide">
                  {game.winner === 'w' ? '🏆 HOÀNG KIM (ĐỎ) CHIẾN THẮNG!' : game.winner === 'b' ? '🏆 THÁI THƯỢNG (ĐEN) CHIẾN THẮNG!' : '🤝 TRẬN CỜ HÒA!'}
                </h2>
                <p className="text-xs text-gold/80">
                  {game.reason === 'CHECKMATE' ? 'Đối thủ đã bị chiếu bí tuyệt đối!' : 'Hết nước đi hợp lệ (Hòa cờ)!'}
                </p>
                <button
                  onClick={reset}
                  className="px-5 py-2 bg-gold text-black font-bold text-xs rounded hover:bg-gold/90 transition-all shadow-md"
                >
                  BẮT ĐẦU VÁN MỚI
                </button>
              </div>
            )}
            <Board fen={game.fen} move={move} flip={game.flip} check={checked} rulers={game.rulers} />
          </section>

          {/* Cột 3: R1 LLM Reasoning & PV Line Explorer & Move History */}
          <section className="lg:col-span-3 flex flex-col gap-4">
            {/* R1 LLM Model Live Thought Chain */}
            <div className="glass rounded-xl p-3 border border-amber-500/40 bg-amber-950/20 flex flex-col gap-2 shadow-glow">
              <div className="flex items-center justify-between border-b border-amber-500/30 pb-1.5 flex-wrap gap-1">
                <span className="text-xs font-bold text-amber-300 flex items-center gap-1.5">
                  <span className="w-2 h-2 rounded-full bg-amber-400 animate-pulse"></span>
                  🤖 R1 LLM BATCH 3 REASONING
                </span>
                <div className="flex items-center gap-1.5">
                  <button
                    onClick={() => {
                      if (!game.thought) return;
                      navigator.clipboard.writeText(game.thought);
                      update((prev) => ({ ...prev, copied: true }));
                      setTimeout(() => update((prev) => ({ ...prev, copied: false })), 2000);
                    }}
                    disabled={!game.thought}
                    title="Copy R1 LLM Thought Content"
                    className={`px-2 py-0.5 rounded text-[10px] font-mono font-bold transition-all flex items-center gap-1 border ${
                      game.copied
                        ? 'bg-emerald-500/30 text-emerald-300 border-emerald-500/50'
                        : game.thought
                        ? 'bg-amber-500/20 text-amber-300 border-amber-500/40 hover:bg-amber-500/40 shadow-glow'
                        : 'bg-gray-800/40 text-gray-500 border-gray-700/40 cursor-not-allowed'
                    }`}
                  >
                    <span>{game.copied ? '✓ COPIED!' : '📋 COPY CONTENT'}</span>
                  </button>
                  <span className="text-[9px] font-mono px-1.5 py-0.5 rounded bg-amber-500/20 text-amber-300 font-bold border border-amber-500/40">
                    {game.mode === 'llm' ? 'ACTIVE MODEL' : 'GRPO 0.5B'}
                  </span>
                </div>
              </div>
              <div className="max-h-40 overflow-y-auto text-[11px] text-amber-100/90 font-mono whitespace-pre-wrap leading-relaxed pr-1 bg-obsidian/60 p-2.5 rounded border border-amber-500/20">
                {game.thought ? game.thought : (
                  <span className="text-amber-400/60 italic">
                    Chuyển sang chế độ "🤖 R1 LLM 0.5B (Batch 3)" ở trên header để đánh trực tiếp với mô hình AI và xem chuỗi suy luận &lt;thought&gt; thời gian thực...
                  </span>
                )}
              </div>
            </div>

            <Explorer line={game.line} pick={pick} active={game.active} />

            <div className="glass rounded-xl p-4 border border-gold/20 flex flex-col gap-3 shadow-glow">
              <h3 className="text-sm font-royal font-bold text-gold border-b border-gold/20 pb-2">
                MOVE HISTORY ({game.history.length - 1})
              </h3>
              <div className="max-h-56 overflow-y-auto text-xs text-gold/80 flex flex-col gap-1 font-mono pr-1">
                {game.history.map((item, idx) => (
                  <div
                    key={`hist-${idx}`}
                    onClick={() => {
                      update((prev) => ({
                        ...prev,
                        fen: item,
                        index: idx
                      }));
                      engine.position(item);
                    }}
                    className={`px-2 py-1 rounded cursor-pointer transition ${
                      game.index === idx
                        ? 'bg-gold/20 text-gold border border-gold/40 font-bold'
                        : 'hover:bg-gold/10'
                    }`}
                  >
                    {idx === 0 ? '0. Initial Board Position' : `${idx}. ${item.split(' ')[0]}`}
                  </div>
                ))}
              </div>
            </div>
          </section>
        </main>
      )}

      {/* Modal FEN / PGN Editor & Parser */}
      <Modal
        show={game.show}
        close={() => update((prev) => ({ ...prev, show: false }))}
        fen={game.fen}
        history={game.history}
        apply={apply}
      />

      {/* Royal Telemetry & WASM Diagnostics Drawer Modal */}
      <Debugger
        show={game.debug}
        close={() => update((prev) => ({ ...prev, debug: false }))}
      />

      {/* Royal Match History & Replay Drawer Modal */}
      <History
        open={game.historyShow}
        close={() => update((prev) => ({ ...prev, historyShow: false }))}
        loadMatch={loadMatch}
      />

      {/* Royal GYM Trainer Telemetry & Background Self-Play Modal */}
      <Gym
        open={game.gymShow}
        close={() => update((prev) => ({ ...prev, gymShow: false }))}
        loadMatch={loadMatch}
      />

      {/* Vulnerability Audit & Dark Pattern Diagnostic Modal */}
      <Audit
        open={game.auditShow}
        close={() => update((prev) => ({ ...prev, auditShow: false }))}
        fen={game.fen}
      />

      {/* Xiangqi-R1 GRPO LLM Distributed Training Studio Modal */}
      <R1Studio
        show={game.r1Show}
        close={() => update((prev) => ({ ...prev, r1Show: false }))}
      />

      {/* Interactive Studio Board Setup Studio Modal */}
      <Studio
        open={game.studioShow}
        close={() => update((prev) => ({ ...prev, studioShow: false }))}
        apply={(newFen) => apply('fen', newFen)}
        initialFen={game.fen}
      />

      {/* Grand Tournament Visualizer Modal (Depth 30 vs Depth 60) */}
      {game.tournamentShow && (
        <Tournament onClose={() => update((prev) => ({ ...prev, tournamentShow: false }))} />
      )}

      {/* Footer */}
      <footer className="border-t border-gold/20 bg-obsidian-card/90 px-6 py-3 text-center text-xs text-gold/50 font-body">
        XiangRust Engine v1.0.0 — Milestone 5 Bot Arena & Self-Play Suite
      </footer>
    </div>
  );
}
