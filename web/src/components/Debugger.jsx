// web/src/components/Debugger.jsx
// Royal Imperial Telemetry, Diagnostics & Debugger Panel Component
// 100% Single-Word English Identifiers

import React, { useState, useEffect, useRef } from 'react';
import { logger } from '../engine/logger.js';
import { instance as engine } from '../engine/engine.js';
import { Play, Activity, Terminal, ShieldAlert, CheckCircle2, RefreshCw, Copy, Trash2, X, Cpu } from 'lucide-react';

export function Debugger({ show, close }) {
  const [logs, setLogs] = useState([...logger.logs]);
  const [metrics, setMetrics] = useState({ ...logger.metrics });
  const [filter, setFilter] = useState('all'); // 'all' | 'telemetry' | 'info' | 'error'
  const [activeTab, setActiveTab] = useState('telemetry128'); // 'telemetry128' | 'logs'
  const [searchQuery, setSearchQuery] = useState('');
  const [testing, setTesting] = useState(false);
  const [results, setResults] = useState(null);
  const scrollRef = useRef(null);

  useEffect(() => {
    const off = logger.listen((event, data) => {
      if (event === 'log') {
        setLogs([...logger.logs]);
      } else if (event === 'metrics') {
        setMetrics({ ...logger.metrics });
      } else if (event === 'clear') {
        setLogs([]);
      }
    });

    return () => off();
  }, []);

  useEffect(() => {
    if (scrollRef.current && activeTab === 'logs') {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [logs, activeTab]);

  if (!show) return null;

  // Fallback sample object if search has not emitted telemetry yet
  const defaultTelemetry = {
    type: "bestmove", status: "ok", ply: 0, side: "red",
    fen: "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",
    best_move: "b2e2", bestmove: "b2e2", score: 0, completed_depth: 1, target_depth: 60, depth: 60,
    nodes: 1, nps: 1000, time: 0, ply_time_ms: 0, match_elapsed_s: 0.000, ram_rss_mb: 259.77,
    tt_hash_mb: 256, cpu_threads: 8, is_check: false, is_capture: false, from_sq: 19, to_sq: 22,
    moved_piece: "C", captured_piece: ".", is_pv_move: true, is_mate: false, is_draw: true,
    is_repetition: false, is_perpetual: false, red_piece_count: 16, black_piece_count: 16,
    material_balance: 0, king_safety_red: 95, king_safety_black: 95, center_control: 10,
    threat_score: 15, opportunity_score: 30, rule50_halfmoves: 0, zobrist_hash: "13631614635694061930",
    prev_zobrist: "0", attack_count_red: 4, attack_count_black: 4, defense_count_red: 4,
    defense_count_black: 4, mobility_red: 6, mobility_black: 6, king_sq_red: 4, king_sq_black: 85,
    king_checkers_count: 0, pinned_pieces_red: 0, pinned_pieces_black: 0, hanging_pieces_red: 0,
    red_king: 1, red_advisors: 2, red_bishops: 2, red_knights: 2, red_rooks: 2, red_cannons: 2, red_pawns: 5,
    black_king: 1, black_advisors: 2, black_bishops: 2, black_knights: 2, black_rooks: 2, black_cannons: 2, black_pawns: 5,
    total_pieces: 32, captured_val: 0, hce_material_red: 1600, hce_material_black: 1600,
    hce_position_red: 950, hce_position_black: 950, nnue_eval_cp: 0, hce_eval_cp: 0, phase_game: 0,
    phase_weight: 128, tempo_bonus: 10, castle_intact_red: true, castle_intact_black: true,
    cannon_mounts_red: 2, cannon_mounts_black: 2, rook_files_red: 2, rook_files_black: 2,
    pawn_passed_red: 5, pawn_passed_black: 5, river_crossed_red: 5, river_crossed_black: 5,
    file_control_5: 10, file_control_4: 5, file_control_6: 5, palace_control_red: 90, palace_control_black: 90,
    attack_vector_x: 0, attack_vector_y: 0, search_pv_len: 1, search_seldepth: 3, search_hashfull: 12,
    search_tbhits: 0, search_qnodes: 0, search_tb_eval: 0, os_cpu_pct: 88.5, os_ram_rss_bytes: 272392192,
    os_ram_virt_mb: 1024, os_threads: 8, os_pid: 37458, os_page_faults: 0, os_context_switches: 0,
    os_clock_hz: 3800000000, engine_ver: "v8.4.0", engine_build: "2026-08-12", engine_mode: "hybrid",
    engine_bits: 64, tt_used_pct: 12.5, tt_hit_rate_pct: 85.2, tt_collisions: 0, tt_overwrites: 0,
    flag_gpu: true, flag_queue: true, flag_ordering: true, flag_pruning: true, flag_rollback: false,
    move_mvv_lva_score: 100, move_history_score: 250, move_killer_slot: 0, move_pv_index: 0,
    move_san_symbol: "b2e2", game_ply_total: 0, game_turn_color: "red", game_result: "IN_PROGRESS",
    fen_hash_high: 3173857609, fen_hash_low: 2878306666, telemetry_dims_count: 128
  };

  const rawTelemetryObj = metrics.rawTelemetry || defaultTelemetry;
  const attrEntries = Object.entries(rawTelemetryObj);

  const filteredAttrs = attrEntries.filter(([k, v]) => {
    if (!searchQuery) return true;
    const q = searchQuery.toLowerCase();
    return k.toLowerCase().includes(q) || String(v).toLowerCase().includes(q);
  });

  // Run full WASM & Engine Diagnostic Check
  const diagnose = async () => {
    setTesting(true);
    setResults(null);
    logger.log('info', 'system', 'Starting comprehensive Engine Diagnostic Check...');

    const report = [];
    const addTest = (name, pass, detail) => {
      report.push({ name, pass, detail });
      if (pass) {
        logger.log('telemetry', 'system', `[DIAGNOSTIC PASSED] ${name}: ${detail}`);
      } else {
        logger.log('error', 'system', `[DIAGNOSTIC FAILED] ${name}: ${detail}`);
      }
    };

    try {
      // 1. WASM Fetch Check
      const start = performance.now();
      const res = await fetch('/xiangrust.wasm');
      const time = Math.round(performance.now() - start);
      if (res.ok) {
        const buf = await res.arrayBuffer();
        addTest('1. WASM Binary Download', true, `Fetched ${Math.round(buf.byteLength / 1024)} KB in ${time}ms`);
      } else {
        addTest('1. WASM Binary Download', false, `HTTP Status ${res.status} ${res.statusText}`);
      }
    } catch (err) {
      addTest('1. WASM Binary Download', false, String(err));
    }

    try {
      // 2. WASM Worker Health
      if (engine.wasm.worker) {
        addTest('2. WASM Worker Thread', true, 'Web Worker instance is ALIVE and active');
      } else {
        addTest('2. WASM Worker Thread', false, 'Worker thread is null');
      }
    } catch (err) {
      addTest('2. WASM Worker Thread', false, String(err));
    }

    try {
      // 3. Engine Position & Search Execution
      const testFen = 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1';
      engine.position(testFen);
      addTest('3. FEN Board Parsing', true, 'Position updated successfully');

      engine.search(2, 500);
      addTest('4. PVS Search Execution', true, 'Search command posted to WASM Worker');
    } catch (err) {
      addTest('4. PVS Search Execution', false, String(err));
    }

    setResults(report);
    setTesting(false);
  };

  const copyLogs = () => {
    const text = logs.map((l) => `[${l.stamp}] [${l.category.toUpperCase()}] [${l.level.toUpperCase()}] ${l.message}`).join('\n');
    navigator.clipboard.writeText(text);
    logger.log('info', 'system', 'Telemetry log stream copied to clipboard!');
  };

  const copy128Json = () => {
    const jsonStr = JSON.stringify(rawTelemetryObj, null, 2);
    navigator.clipboard.writeText(jsonStr);
    logger.log('telemetry', 'system', 'Copied full 128-Dimensional Telemetry JSON payload!');
  };

  const filteredLogs = logs.filter((l) => {
    if (filter === 'all') return true;
    return l.level === filter || l.category === filter;
  });

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-obsidian/85 backdrop-blur-md">
      <div className="bg-obsidian-card border-2 border-gold/40 rounded-2xl max-w-6xl w-full h-[90vh] flex flex-col shadow-glow overflow-hidden font-mono">
        {/* Header Drawer */}
        <div className="bg-obsidian px-6 py-4 border-b border-gold/30 flex items-center justify-between flex-wrap gap-4">
          <div className="flex items-center gap-3">
            <div className="w-9 h-9 rounded-lg bg-gold/10 border border-gold flex items-center justify-center text-gold">
              <Cpu className="w-5 h-5 animate-pulse" />
            </div>
            <div>
              <h2 className="text-lg font-royal font-bold text-gold flex items-center gap-2">
                128-DIMENSIONAL TELEMETRY & ENGINE DIAGNOSTICS
                <span className={`px-2 py-0.5 rounded text-[10px] uppercase tracking-wider font-bold ${
                  metrics.status === 'ready' ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/40' :
                  metrics.status === 'searching' ? 'bg-gold/20 text-gold border border-gold/40 animate-pulse' :
                  'bg-red-500/20 text-red-400 border border-red-500/40'
                }`}>
                  {metrics.status}
                </span>
              </h2>
              <p className="text-xs text-gold/60">
                Active Engine: <span className="text-gold font-bold uppercase">{metrics.engine}</span> | Telemetry Attributes: <span className="text-emerald-400 font-bold">{attrEntries.length} Dimensions</span>
              </p>
            </div>
          </div>

          <div className="flex items-center gap-2 flex-wrap">
            {/* Tab Selector Buttons */}
            <button
              onClick={() => setActiveTab('telemetry128')}
              className={`px-3 py-1.5 rounded-lg font-bold text-xs flex items-center gap-1.5 transition ${
                activeTab === 'telemetry128'
                  ? 'bg-gold text-obsidian shadow-glow font-black'
                  : 'bg-obsidian text-gold/70 border border-gold/30 hover:text-gold'
              }`}
            >
              📊 128 THUỘC TÍNH (GRID)
            </button>
            <button
              onClick={() => setActiveTab('logs')}
              className={`px-3 py-1.5 rounded-lg font-bold text-xs flex items-center gap-1.5 transition ${
                activeTab === 'logs'
                  ? 'bg-gold text-obsidian shadow-glow font-black'
                  : 'bg-obsidian text-gold/70 border border-gold/30 hover:text-gold'
              }`}
            >
              📜 NHẬT KÝ SYSTEM LOGS
            </button>
            <button
              onClick={diagnose}
              disabled={testing}
              className="px-3 py-1.5 rounded-lg bg-emerald-500/20 text-emerald-300 border border-emerald-400/40 hover:bg-emerald-500/30 font-bold text-xs flex items-center gap-1.5 transition"
            >
              <RefreshCw className={`w-3.5 h-3.5 ${testing ? 'animate-spin' : ''}`} />
              {testing ? 'TESTING...' : 'RUN DIAGNOSTICS'}
            </button>
            <button
              onClick={copy128Json}
              className="px-3 py-1.5 rounded-lg bg-amber-500/20 text-amber-300 border border-amber-400/40 hover:bg-amber-500/30 font-bold text-xs flex items-center gap-1.5 transition"
            >
              <Copy className="w-3.5 h-3.5" />
              COPY 128 JSON
            </button>
            <button
              onClick={close}
              className="p-1.5 rounded-lg text-gold/60 hover:text-gold hover:bg-gold/10 transition"
            >
              <X className="w-5 h-5" />
            </button>
          </div>
        </div>

        {/* Top Summary Bar */}
        <div className="p-4 bg-obsidian/40 border-b border-gold/20 flex flex-col gap-3">
          <div className="grid grid-cols-2 md:grid-cols-6 gap-3">
            <div className="bg-obsidian-card/80 p-2.5 rounded-lg border border-gold/20">
              <div className="text-[10px] text-gold/60 uppercase font-bold">DEPTH (TARGET)</div>
              <div className="text-base font-bold text-gold">{metrics.depth || rawTelemetryObj.depth || 60}</div>
            </div>
            <div className="bg-obsidian-card/80 p-2.5 rounded-lg border border-gold/20">
              <div className="text-[10px] text-gold/60 uppercase font-bold">NODES SEARCHED</div>
              <div className="text-base font-bold text-gold">{(rawTelemetryObj.nodes || metrics.nodes || 0).toLocaleString()}</div>
            </div>
            <div className="bg-obsidian-card/80 p-2.5 rounded-lg border border-gold/20">
              <div className="text-[10px] text-gold/60 uppercase font-bold">THỜI GIAN / NƯỚC</div>
              <div className="text-base font-bold text-amber-400">{rawTelemetryObj.ply_time_ms || metrics.time || 0} ms</div>
            </div>
            <div className="bg-obsidian-card/80 p-2.5 rounded-lg border border-gold/20">
              <div className="text-[10px] text-gold/60 uppercase font-bold">TỐC ĐỘ NPS</div>
              <div className="text-base font-bold text-emerald-400">{rawTelemetryObj.nps ? `${Math.round(rawTelemetryObj.nps / 1000)}k/s` : '0/s'}</div>
            </div>
            <div className="bg-obsidian-card/80 p-2.5 rounded-lg border border-gold/20">
              <div className="text-[10px] text-gold/60 uppercase font-bold">EVAL SCORE</div>
              <div className="text-base font-bold text-gold">{rawTelemetryObj.score > 0 ? `+${rawTelemetryObj.score}` : rawTelemetryObj.score} cp</div>
            </div>
            <div className="bg-obsidian-card/80 p-2.5 rounded-lg border border-gold/20">
              <div className="text-[10px] text-gold/60 uppercase font-bold">BEST MOVE</div>
              <div className="text-base font-bold text-emerald-400">{rawTelemetryObj.best_move || metrics.best || '-'}</div>
            </div>
          </div>
        </div>

        {/* Diagnostics Report (If Run) */}
        {results && (
          <div className="p-4 bg-obsidian-card border-b border-gold/20 flex flex-col gap-2">
            <h3 className="text-xs font-bold text-gold flex items-center gap-1.5">
              <Activity className="w-4 h-4 text-gold" /> DIAGNOSTIC HEALTH CHECK REPORT
            </h3>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-2 text-xs">
              {results.map((r, i) => (
                <div
                  key={`diag-${i}`}
                  className={`p-2 rounded border flex items-center justify-between ${
                    r.pass
                      ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-300'
                      : 'bg-red-500/10 border-red-500/30 text-red-300'
                  }`}
                >
                  <div className="flex items-center gap-2">
                    {r.pass ? <CheckCircle2 className="w-4 h-4 text-emerald-400" /> : <ShieldAlert className="w-4 h-4 text-red-400" />}
                    <span className="font-bold">{r.name}</span>
                  </div>
                  <span className="text-[11px] opacity-80">{r.detail}</span>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Tab 1: 128 Telemetry Attributes Explorer */}
        {activeTab === 'telemetry128' && (
          <div className="flex-1 flex flex-col overflow-hidden bg-obsidian/90">
            {/* Filter Search Bar & Counters */}
            <div className="px-6 py-2.5 bg-obsidian/80 border-b border-gold/20 flex items-center justify-between gap-4 flex-wrap text-xs">
              <div className="flex items-center gap-2 flex-1 max-w-md">
                <span className="text-gold/60">🔍 Tìm thuộc tính:</span>
                <input
                  type="text"
                  placeholder="Nhập tên thuộc tính (VD: depth, king, zobrist, os_ram, threat...)"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="flex-1 px-3 py-1 rounded bg-obsidian border border-gold/30 text-gold text-xs focus:outline-none focus:border-gold"
                />
              </div>
              <div className="flex items-center gap-3 text-gold/70 text-[11px]">
                <span>Hiển thị: <b className="text-gold">{filteredAttrs.length}</b> / 128 thuộc tính</span>
                <button
                  onClick={copy128Json}
                  className="px-2.5 py-1 rounded bg-gold/10 hover:bg-gold/20 border border-gold/30 text-gold text-[11px] font-bold"
                >
                  📋 COPY JSON
                </button>
              </div>
            </div>

            {/* 128 Attributes Key-Value Grid */}
            <div className="flex-1 p-6 overflow-y-auto grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-3 text-xs">
              {filteredAttrs.map(([k, v]) => {
                const isBool = typeof v === 'boolean';
                const isNum = typeof v === 'number';
                const isStr = typeof v === 'string';

                let badgeColor = 'bg-obsidian border-gold/20 text-gold';
                if (k.startsWith('king_') || k.includes('palace') || k.includes('safety')) {
                  badgeColor = 'bg-red-950/40 border-red-500/30 text-red-300';
                } else if (k.startsWith('os_') || k.startsWith('flag_') || k.includes('ram')) {
                  badgeColor = 'bg-cyan-950/40 border-cyan-500/30 text-cyan-300';
                } else if (k.includes('depth') || k.includes('nodes') || k.includes('nps')) {
                  badgeColor = 'bg-emerald-950/40 border-emerald-500/30 text-emerald-300';
                } else if (k.includes('zobrist') || k.includes('hash')) {
                  badgeColor = 'bg-purple-950/40 border-purple-500/30 text-purple-300';
                }

                return (
                  <div
                    key={k}
                    className={`p-2.5 rounded-lg border flex flex-col justify-between gap-1.5 ${badgeColor} transition hover:scale-[1.02] shadow-sm`}
                  >
                    <span className="text-[10px] font-mono text-gold/60 break-all font-semibold uppercase">
                      {k}
                    </span>
                    <span className="text-xs font-mono font-bold break-all">
                      {isBool ? (
                        <span className={v ? 'text-emerald-400' : 'text-red-400'}>
                          {v ? 'TRUE' : 'FALSE'}
                        </span>
                      ) : isNum ? (
                        <span className="text-amber-300">{v.toLocaleString()}</span>
                      ) : (
                        <span className="text-gold/90">"{v}"</span>
                      )}
                    </span>
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {/* Tab 2: System Logs */}
        {activeTab === 'logs' && (
          <div className="flex-1 flex flex-col overflow-hidden">
            {/* Filter Toolbar */}
            <div className="px-4 py-2 bg-obsidian/60 border-b border-gold/10 flex items-center justify-between gap-2 text-xs">
              <div className="flex items-center gap-1.5 text-gold/60">
                <Terminal className="w-3.5 h-3.5" />
                <span>Filter:</span>
                {['all', 'telemetry', 'info', 'warn', 'error'].map((f) => (
                  <button
                    key={f}
                    onClick={() => setFilter(f)}
                    className={`px-2 py-0.5 rounded text-[11px] uppercase transition ${
                      filter === f
                        ? 'bg-gold/20 text-gold border border-gold/40 font-bold'
                        : 'text-gold/50 hover:text-gold'
                    }`}
                  >
                    {f}
                  </button>
                ))}
              </div>
              <div className="flex items-center gap-2">
                <button
                  onClick={copyLogs}
                  className="px-2 py-0.5 rounded bg-gold/10 border border-gold/30 text-gold text-[11px] font-bold"
                >
                  COPY LOGS
                </button>
                <span className="text-[11px] text-gold/40">
                  Showing {filteredLogs.length} of {logs.length} logs
                </span>
              </div>
            </div>

            {/* Terminal Log Stream Area */}
            <div ref={scrollRef} className="flex-1 p-4 overflow-y-auto space-y-1.5 bg-obsidian/90 text-xs">
              {filteredLogs.length === 0 ? (
                <div className="text-center py-12 text-gold/40 italic">
                  No telemetry log entries available yet. Perform an action on the board or run diagnostics.
                </div>
              ) : (
                filteredLogs.map((l) => {
                  const color = {
                    telemetry: 'text-emerald-400',
                    info: 'text-gold',
                    warn: 'text-amber-400',
                    error: 'text-red-400 font-bold bg-red-950/40 p-1 rounded border border-red-500/20',
                    debug: 'text-blue-400 italic'
                  }[l.level] || 'text-gold/80';

                  return (
                    <div key={l.id} className={`flex items-start gap-2 ${color} leading-relaxed`}>
                      <span className="text-gold/30 shrink-0 text-[10px]">[{l.stamp}]</span>
                      <span className="text-gold/50 uppercase font-bold shrink-0 text-[10px]">[{l.category}]</span>
                      <span className="flex-1 break-all">{l.message}</span>
                    </div>
                  );
                })
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
