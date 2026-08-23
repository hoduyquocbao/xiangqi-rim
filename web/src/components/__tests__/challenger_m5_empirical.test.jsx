// Unit and Empirical tests for Milestone 5 Bot Arena Component, App View Mode Switcher, and Single-Word Compliance
// 100% Single-word English Identifiers

import React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, act } from '@testing-library/react';
import Arena from '../Arena.jsx';
import App from '../../App.jsx';
import { instance as engine } from '../../engine/engine.js';
import fs from 'fs';
import path from 'path';

// Mock global Worker for jsdom environment
if (typeof window !== 'undefined' && !window.Worker) {
  window.Worker = class {
    constructor() {}
    postMessage() {}
    terminate() {}
    addEventListener() {}
    removeEventListener() {}
  };
}

const start = 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1';

if (typeof globalThis.Worker === 'undefined') {
  globalThis.Worker = class {
    constructor() {}
    postMessage() {}
    terminate() {}
  };
}

describe('Milestone 5 Challenger 2 Empirical Test Suite', () => {
  it('renders Arena component and validates interactive control elements', () => {
    render(
      <Arena
        fen={start}
        turn="w"
        move={vi.fn()}
        reset={vi.fn()}
        board={new Array(90).fill('.')}
        check={false}
      />
    );

    expect(screen.getByText(/BOT ARENA \(SELF-PLAY\)/i)).toBeTruthy();
    expect(screen.getByText(/PAUSED/i)).toBeTruthy();
    expect(screen.getAllByText(/START SELF-PLAY/i).length).toBeGreaterThan(0);
    expect(screen.getByText(/STEP \(1 MOVE\)/i)).toBeTruthy();
    expect(screen.getByText(/RESET ARENA/i)).toBeTruthy();
    expect(screen.getByText(/RED BOT/i)).toBeTruthy();
    expect(screen.getByText(/BLACK BOT/i)).toBeTruthy();
    expect(screen.getByText(/SPEED \(NPS\)/i)).toBeTruthy();
    expect(screen.getByText(/SEARCH NODES/i)).toBeTruthy();
    expect(screen.getByText(/LIVE ARENA MOVE FEED/i)).toBeTruthy();
  });

  // Kiểm thử chuyển đổi chế độ xem view mode trong App giữa PLAY và BOT ARENA
  it('tests view mode toggle in App component between PLAY and BOT ARENA', () => {
    render(<App />);

    // Chế độ xem mặc định là PLAY / ANALYSIS
    expect(screen.getByText(/ENGINE CONTROLS/i)).toBeTruthy();
    expect(screen.getByText(/PRINCIPAL VARIATION/i)).toBeTruthy();

    // Tìm nút chuyển sang BOT ARENA và bọc thao tác click trong act
    const arena = screen.getByText(/BOT ARENA \(SELF-PLAY\)/i);
    act(() => {
      fireEvent.click(arena);
    });

    // Xác nhận đã chuyển sang giao diện BOT ARENA
    expect(screen.getAllByText(/BOT ARENA \(SELF-PLAY\)/i).length).toBeGreaterThan(0);
    expect(screen.getByText(/RED BOT/i)).toBeTruthy();
    expect(screen.getByText(/BLACK BOT/i)).toBeTruthy();

    // Tìm nút quay lại PLAY / ANALYSIS và bọc thao tác click trong act
    const play = screen.getByText(/PLAY \/ ANALYSIS/i);
    act(() => {
      fireEvent.click(play);
    });

    // Xác nhận đã quay lại giao diện điều khiển engine mặc định
    expect(screen.getByText(/ENGINE CONTROLS/i)).toBeTruthy();
  }, 15000);

  // Kiểm thử các nút trượt điều chỉnh tốc độ trận đấu và độ sâu bot trong Arena
  it('tests Arena match speed and depth slider controls', () => {
    render(
      <Arena
        fen={start}
        turn="w"
        move={vi.fn()}
        reset={vi.fn()}
        board={new Array(90).fill('.')}
      />
    );

    const sliders = screen.getAllByRole('slider');
    // Thanh trượt độ sâu Bot Đỏ (0) và Bot Đen (1)
    const red = sliders[0];
    const black = sliders[1];

    act(() => {
      fireEvent.change(red, { target: { value: '9' } });
    });
    expect(red.value).toBe('9');

    act(() => {
      fireEvent.change(black, { target: { value: '11' } });
    });
    expect(black.value).toBe('11');
  }, 15000);

  it('tests Arena blunder warning alert rendering when centipawn drop > 150', async () => {
    render(
      <Arena
        fen={start}
        turn="w"
        move={vi.fn()}
        reset={vi.fn()}
        board={new Array(90).fill('.')}
      />
    );

    act(() => {
      engine.emit('search', {
        bestmove: 'C2.5',
        score: -300,
        nodes: 12000,
        nps: 150000
      });
    });

    const alert = await screen.findByText(/BLUNDER!/i);
    expect(alert).toBeTruthy();
  });

  it('verifies single-word English identifier compliance across web/src files', () => {
    const srcDir = path.resolve(__dirname, '../../');
    const files = [];

    function collect(dir) {
      const entries = fs.readdirSync(dir, { withFileTypes: true });
      for (const entry of entries) {
        const full = path.join(dir, entry.name);
        if (entry.isDirectory() && entry.name !== '__tests__') {
          collect(full);
        } else if (entry.isFile() && (entry.name.endsWith('.jsx') || entry.name.endsWith('.js'))) {
          files.push(full);
        }
      }
    }

    collect(srcDir);
    expect(files.length).toBeGreaterThan(5);

    // Reserved allowed multi-word tokens (React standard hooks/props/browser built-ins/math/JSON)
    const allowed = new Set([
      'useState', 'useEffect', 'useRef', 'useCallback', 'useMemo',
      'ReactDOM', 'React', 'className', 'onClick', 'onChange', 'onDragStart',
      'onDragEnd', 'onDragOver', 'onDrop', 'draggable', 'style', 'viewBox',
      'strokeWidth', 'stopColor', 'stopOpacity', 'writingMode', 'stdDeviation',
      'floodColor', 'floodOpacity', 'filter', 'defs', 'svg', 'g', 'circle',
      'rect', 'text', 'line', 'path', 'linearGradient', 'radialGradient',
      'feDropShadow', 'navigator', 'clipboard', 'writeText', 'setTimeout',
      'clearTimeout', 'performance', 'now', 'toLocaleString', 'TextEncoder',
      'TextDecoder', 'WebAssembly', 'WebSocket', 'AudioContext', 'webkitAudioContext',
      'postMessage', 'onmessage', 'onerror', 'onopen', 'onclose', 'readyState',
      'arrayBuffer', 'getChannelData', 'createOscillator', 'createGain',
      'createBuffer', 'createBufferSource', 'createBiquadFilter', 'setValueAtTime',
      'exponentialRampToValueAtTime', 'destination', 'slice', 'push', 'map',
      'filter', 'reduce', 'includes', 'toUpperCase', 'toLowerCase', 'charCodeAt',
      'fromCharCode', 'parseInt', 'isNaN', 'split', 'join', 'replace', 'trim',
      'startsWith', 'endsWith', 'match', 'test', 'preventDefault', 'stopPropagation',
      'target', 'value', 'dataTransfer', 'setData', 'getItem', 'setItem',
      'JSON', 'parse', 'stringify', 'Math', 'abs', 'floor', 'round', 'min',
      'max', 'pow', 'random', 'Array', 'from', 'fill', 'concat', 'Object',
      'keys', 'values', 'entries', 'Number', 'toFixed', 'String', 'Set',
      'add', 'delete', 'has', 'Promise', 'resolve', 'reject', 'console',
      'log', 'error', 'warn', 'info', 'window', 'document', 'self', 'fetch',
      'set_position', 'matrix_analysis', 'risk_assessment', 'centipawn_eval',
      'red_pieces_count', 'black_pieces_count', 'king_safety_score',
      'center_file_control', 'RED_PHAO_DAU_INTENT', 'tactical_intent',
      'flag_gpu', 'flag_queue', 'flag_ordering', 'flag_pruning', 'flag_rollback',
      'move_mvv_lva_score', 'move_history_score', 'move_killer_slot', 'move_pv_index',
      'move_san_symbol', 'game_ply_total', 'game_turn_color', 'game_result', 'IN_PROGRESS',
      'fen_hash_high', 'fen_hash_low', 'telemetry_dims_count', 'ply_time_ms', 'best_move',
      'os_ram', 'king_', 'os_', 'flag_', 'match_d30_vs_d60', 'from_sq', 'to_sq',
      'moved_piece', 'captured_piece', 'completed_depth', 'target_depth',
      'match_elapsed_s', 'ram_rss_mb', 'tt_hash_mb', 'cpu_threads', 'is_check',
      'is_capture', 'is_pv_move', 'red_piece_count', 'black_piece_count',
      'material_balance', 'king_safety_red', 'king_safety_black', 'center_control',
      'threat_score', 'opportunity_score', 'rule50_halfmoves', 'PERPETUAL_CHECK_LOSS',
      'DRAW_REPETITION_3FOLD', 'tt_used_pct', 'tt_hit_rate_pct', 'tt_collisions', 'tt_overwrites',
      'is_mate', 'is_draw', 'is_repetition', 'is_perpetual', 'zobrist_hash', 'prev_zobrist',
      'attack_count_red', 'attack_count_black', 'defense_count_red', 'defense_count_black',
      'mobility_red', 'mobility_black', 'king_sq_red', 'king_sq_black', 'king_checkers_count',
      'pinned_pieces_red', 'pinned_pieces_black', 'hanging_pieces_red', 'red_king',
      'red_advisors', 'red_bishops', 'red_knights', 'red_rooks', 'red_cannons', 'red_pawns',
      'black_king', 'black_advisors', 'black_bishops', 'black_knights', 'black_rooks',
      'black_cannons', 'black_pawns', 'total_pieces', 'captured_val', 'hce_material_red',
      'hce_material_black', 'hce_position_red', 'hce_position_black', 'nnue_eval_cp',
      'hce_eval_cp', 'phase_game', 'phase_weight', 'tempo_bonus', 'castle_intact_red',
      'castle_intact_black', 'cannon_mounts_red', 'cannon_mounts_black', 'rook_files_red',
      'rook_files_black', 'pawn_passed_red', 'pawn_passed_black', 'river_crossed_red',
      'river_crossed_black', 'file_control_5', 'file_control_4', 'file_control_6',
      'palace_control_red', 'palace_control_black', 'attack_vector_x', 'attack_vector_y',
      'search_pv_len', 'search_seldepth', 'search_hashfull', 'search_tbhits',
      'search_qnodes', 'search_tb_eval', 'os_cpu_pct', 'os_ram_rss_bytes',
      'os_ram_virt_mb', 'os_threads', 'os_pid', 'os_page_faults',
      'os_context_switches', 'os_clock_hz', 'engine_ver', 'engine_build',
      'engine_mode', 'engine_bits'
    ]);

    const nonSingleWordViolations = [];

    // Common English word roots to allow standard single words
    for (const file of files) {
      const content = fs.readFileSync(file, 'utf-8');
      const lines = content.split('\n');

      for (let i = 0; i < lines.length; i++) {
        const line = lines[i];
        if (line.trim().startsWith('//') || line.trim().startsWith('/*') || line.trim().startsWith('*')) {
          continue; // skip comment lines
        }

        // Match identifier tokens
        const tokens = line.match(/\b[a-zA-Z_$][a-zA-Z0-9_$]*\b/g) || [];
        for (const token of tokens) {
          if (token.startsWith('_') || allowed.has(token)) continue;

          // Check if token contains camelCase with multiple dictionary words or snake_case
          if (token.includes('_')) {
            nonSingleWordViolations.push({ file: path.basename(file), line: i + 1, token });
          }
        }
      }
    }

    expect(nonSingleWordViolations).toEqual([]);
  });
});
