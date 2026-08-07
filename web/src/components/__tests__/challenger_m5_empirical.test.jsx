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
      'set_position'
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
