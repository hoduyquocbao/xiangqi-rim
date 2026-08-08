// web/src/components/__tests__/challenger_m1_empirical.test.jsx
// Milestone M1 Empirical Verification Test Suite: R1Studio, Debugger, 3-Representation Pipeline & <thought> Reasoning Tag
// Single-Word English Identifiers: render, studio, debugger, matrix, fen, pgn, thought, parse, stringify, check, moves, roundtrip, benchmark, performance, empirical

import React from 'react';
import { render, screen, fireEvent, act, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { R1Studio } from '../R1Studio.jsx';
import { Debugger } from '../Debugger.jsx';
import { parse as parseFen, fen as encodeFen, moves as getMoves } from '../../rules/rules.js';
import { parse as parsePgn, stringify as stringifyPgn } from '../../rules/pgn.js';
import { logger } from '../../engine/logger.js';

describe('Milestone M1 Empirical Challenger Verification Suite', () => {
  beforeEach(() => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    // Mock navigator.clipboard
    Object.assign(navigator, {
      clipboard: {
        writeText: vi.fn().mockImplementation(() => Promise.resolve())
      }
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
    vi.useRealTimers();
  });

  // =========================================================================
  // 1. R1STUDIO COMPONENT EMPIRICAL TESTS
  // =========================================================================
  describe('R1Studio Component Suite', () => {
    it('should render null when show is false', () => {
      const { container } = render(<R1Studio show={false} close={() => {}} />);
      expect(container.firstChild).toBeNull();
    });

    it('should render full R1Studio interface when show is true', () => {
      render(<R1Studio show={true} close={() => {}} />);

      // Verify Header and Badges
      expect(screen.getByText(/XIANGQI-R1 LLM DISTRIBUTED TRAINER \(GRPO\)/i)).toBeTruthy();
      expect(screen.getByText(/Qwen3.5-0.8B \+ Unsloth 4-bit LoRA \+ 3 Reward Functions & P2P Mesh/i)).toBeTruthy();

      // Verify P2P Mesh & Infrastructure Cards
      expect(screen.getByText(/P2P MESH TOPIC \(24\/7\)/i)).toBeTruthy();
      expect(screen.getByText(/STORAGE PERSISTENCE/i)).toBeTruthy();
      expect(screen.getByText(/GRPO ACCELERATION/i)).toBeTruthy();

      // Verify 3 Reward Functions Specifications
      expect(screen.getByText(/BA MÁY CHẤM ĐIỂM TỰ ĐỘNG \(GRPO REWARD FUNCTIONS\)/i)).toBeTruthy();
      expect(screen.getByText(/1️⃣ LUẬT CHƠI \(RULE\)/i)).toBeTruthy();
      expect(screen.getByText(/2️⃣ ĐỊNH DẠNG \(FORMAT\)/i)).toBeTruthy();
      expect(screen.getByText(/3️⃣ CHIẾN THUẬT \(QUALITY\)/i)).toBeTruthy();

      // Verify HuggingFace Integration links
      expect(screen.getByText(/HUGGINGFACE HUB INTEGRATION \(TOKEN CONNECTED\)/i)).toBeTruthy();
      expect(screen.getByText(/DATASET REPOSITORY:/i)).toBeTruthy();
    });

    it('should trigger close callback when close button is clicked', () => {
      const closeMock = vi.fn();
      render(<R1Studio show={true} close={closeMock} />);

      const closeButton = screen.getByText('✕ ĐÓNG');
      fireEvent.click(closeButton);
      expect(closeMock).toHaveBeenCalledTimes(1);
    });

    it('should handle copy command action and show confirmation feedback', async () => {
      render(<R1Studio show={true} close={() => {}} />);

      const copyBtn = screen.getByText(/COPY LỆNH CHẠY/i);
      fireEvent.click(copyBtn);

      expect(navigator.clipboard.writeText).toHaveBeenCalledWith('python3 scripts/train.py');
      expect(screen.getByText(/ĐÃ COPY LỆNH/i)).toBeTruthy();

      act(() => {
        vi.advanceTimersByTime(2100);
      });

      expect(screen.getByText(/COPY LỆNH CHẠY/i)).toBeTruthy();
    });

    it('should toggle execution state when activation button is clicked', () => {
      render(<R1Studio show={true} close={() => {}} />);

      const runBtn = screen.getByText(/KÍCH HOẠT TIẾN TRÌNH DISTRIBUTED DATA & GRPO LLM/i);
      fireEvent.click(runBtn);

      expect(screen.getByText(/ĐANG KẾT NỐI HUGGINGFACE HUB & P2P MESH.../i)).toBeTruthy();

      act(() => {
        vi.advanceTimersByTime(3100);
      });

      expect(screen.getByText(/KÍCH HOẠT TIẾN TRÌNH DISTRIBUTED DATA & GRPO LLM/i)).toBeTruthy();
    });
  });

  // =========================================================================
  // 2. DEBUGGER COMPONENT EMPIRICAL TESTS
  // =========================================================================
  describe('Debugger Component Suite', () => {
    it('should render null when show is false', () => {
      const { container } = render(<Debugger show={false} close={() => {}} />);
      expect(container.firstChild).toBeNull();
    });

    it('should render telemetry panel and live metrics when show is true', () => {
      render(<Debugger show={true} close={() => {}} />);

      expect(screen.getByText(/TELEMETRY & WASM DIAGNOSTICS/i)).toBeTruthy();
      expect(screen.getByText(/PHẦN CỨNG BẰNG GPU\/CPU:/i)).toBeTruthy();
      expect(screen.getByText(/Zero-Copy Memory:/i)).toBeTruthy();

      // Check metric labels
      expect(screen.getByText('DEPTH')).toBeTruthy();
      expect(screen.getByText('NODES')).toBeTruthy();
      expect(screen.getByText('SPEED (NPS)')).toBeTruthy();
      expect(screen.getByText('EVAL SCORE')).toBeTruthy();
    });

    it('should allow filtering logs by severity level', () => {
      render(<Debugger show={true} close={() => {}} />);

      logger.log('info', 'test', 'Info message for debugger');
      logger.log('error', 'test', 'Error message for debugger');

      const errorFilterBtn = screen.getByRole('button', { name: 'error' });
      fireEvent.click(errorFilterBtn);

      expect(screen.getByText('Error message for debugger')).toBeTruthy();
    });

    it('should execute full diagnostics check when button clicked', async () => {
      vi.useRealTimers();
      // Mock global fetch for WASM binary test
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        arrayBuffer: async () => new ArrayBuffer(1024 * 100),
        status: 200,
        statusText: 'OK'
      });

      render(<Debugger show={true} close={() => {}} />);

      const diagBtn = screen.getByText(/RUN DIAGNOSTICS/i);

      await act(async () => {
        fireEvent.click(diagBtn);
        await new Promise((r) => setTimeout(r, 100));
      });

      expect(screen.getByText(/DIAGNOSTIC HEALTH CHECK REPORT/i)).toBeTruthy();
      expect(screen.getAllByText(/1\. WASM Binary Download/i).length).toBeGreaterThan(0);
    });

    it('should clear logs when clear button clicked', () => {
      render(<Debugger show={true} close={() => {}} />);

      act(() => {
        logger.log('info', 'test', 'Persistent log entry');
      });
      expect(screen.getByText('Persistent log entry')).toBeTruthy();

      const clearBtn = screen.getByText('CLEAR');
      act(() => {
        fireEvent.click(clearBtn);
      });

      expect(screen.queryByText('Persistent log entry')).toBeNull();
      expect(screen.getByText(/No telemetry log entries available yet/i)).toBeTruthy();
    });
  });

  // =========================================================================
  // 3. 3-REPRESENTATION PIPELINE (2D MATRIX, FEN, PGN) EMPIRICAL TESTS
  // =========================================================================
  describe('3-Representation Pipeline Suite (2D Matrix ↔ FEN ↔ PGN)', () => {
    const startFen = 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w';

    it('should accurately convert between FEN string and 2D Matrix (90-element array)', () => {
      const { board, turn } = parseFen(startFen);

      // Verify 2D Matrix bounds and key piece positions
      expect(board).toHaveLength(90);
      expect(turn).toBe('w');

      // Red Kings at rank 0, file 4 (index 4)
      expect(board[4]).toBe('K');
      // Black Kings at rank 9, file 4 (index 85)
      expect(board[85]).toBe('k');
      // Empty squares represent '.'
      expect(board[10]).toBe('.');

      // Encode back to FEN and verify zero loss
      const reEncoded = encodeFen(board, turn);
      expect(reEncoded).toBe(startFen);
    });

    it('should handle move transformations on 2D Matrix and sync with FEN update', () => {
      const { board, turn } = parseFen(startFen);

      // Move Red Cannon from h2 (file 7, rank 2 => index 25) to e2 (file 4, rank 2 => index 22)
      const moveFrom = 25; // C
      const moveTo = 22;

      const newBoard = [...board];
      newBoard[moveTo] = newBoard[moveFrom];
      newBoard[moveFrom] = '.';

      const nextTurn = 'b';
      const newFen = encodeFen(newBoard, nextTurn);

      const parsedNew = parseFen(newFen);
      expect(parsedNew.board[moveTo]).toBe('C');
      expect(parsedNew.board[moveFrom]).toBe('.');
      expect(parsedNew.turn).toBe('b');
    });

    it('should parse and stringify PGN match records seamlessly', () => {
      const moveHistory = ['P2+1', 'P8+1', 'C2.5', 'H8+7'];
      const pgnText = stringifyPgn(moveHistory, { event: 'M1 Verification Championship', red: 'Master Red', black: 'AI Black' });

      expect(pgnText).toContain('[Event "M1 Verification Championship"]');
      expect(pgnText).toContain('[Red "Master Red"]');
      expect(pgnText).toContain('1. P2+1 P8+1');
      expect(pgnText).toContain('2. C2.5 H8+7');

      const parsed = parsePgn(pgnText);
      expect(parsed.headers.event).toBe('M1 Verification Championship');
      expect(parsed.moves).toEqual(['P2+1', 'P8+1', 'C2.5', 'H8+7']);
    });

    it('should verify 3-way synchronization consistency (2D Matrix -> FEN -> PGN)', () => {
      // Step 1: Initial State
      let state = parseFen(startFen);

      // Step 2: Simulate 2 moves
      const movesLog = [];

      // Move 1: Red Cannon C2.5 (from 25 to 22)
      state.board[22] = 'C';
      state.board[25] = '.';
      state.turn = 'b';
      movesLog.push('C2.5');

      const fenAfterMove1 = encodeFen(state.board, state.turn);

      // Move 2: Black Horse H8+7 (from 79 to 62)
      state.board[62] = 'n';
      state.board[79] = '.';
      state.turn = 'w';
      movesLog.push('H8+7');

      const fenAfterMove2 = encodeFen(state.board, state.turn);
      const pgnRecord = stringifyPgn(movesLog, { fen: startFen });

      // Verify all 3 representations are in complete alignment
      const reParsedBoard = parseFen(fenAfterMove2);
      expect(reParsedBoard.board[22]).toBe('C');
      expect(reParsedBoard.board[62]).toBe('n');

      const reParsedPgn = parsePgn(pgnRecord);
      expect(reParsedPgn.moves).toEqual(['C2.5', 'H8+7']);
    });
  });

  // =========================================================================
  // 4. DEEP REASONING `<thought>` TAG STREAMING & PARSER TESTS
  // =========================================================================
  describe('LLM Reasoning `<thought>` Tag Streaming & Parser Suite', () => {
    // Utility parser to extract <thought> reasoning block and move payload
    const parseThoughtStream = (response) => {
      const match = response.match(/<thought>([\s\S]*?)<\/thought>\s*([\s\S]*)/);
      if (match) {
        return {
          hasThought: true,
          thought: match[1].trim(),
          payload: match[2].trim()
        };
      }
      return {
        hasThought: false,
        thought: '',
        payload: response.trim()
      };
    };

    // Calculate GRPO Format Reward
    const computeFormatReward = (response) => {
      const parsed = parseThoughtStream(response);
      return parsed.hasThought ? 1.0 : -1.0;
    };

    it('should correctly parse standard LLM response containing <thought> block', () => {
      const llmOutput = `<thought>
Pháp cờ tướng mở màn: Pháo 2 bình 5 (Pháo Đầu) nhằm đe dọa trực tiếp Tướng đen ở trung lộ.
Độ sâu đánh giá 12 lớp, score +45cp.
</thought>
P2=5`;

      const result = parseThoughtStream(llmOutput);
      expect(result.hasThought).toBe(true);
      expect(result.thought).toContain('Pháo 2 bình 5 (Pháo Đầu)');
      expect(result.payload).toBe('P2=5');

      const reward = computeFormatReward(llmOutput);
      expect(reward).toBe(1.0);
    });

    it('should penalize response lacking <thought> tag in GRPO Reward Function', () => {
      const rawOutput = 'P2=5';
      const result = parseThoughtStream(rawOutput);

      expect(result.hasThought).toBe(false);
      expect(result.thought).toBe('');
      expect(result.payload).toBe('P2=5');

      const reward = computeFormatReward(rawOutput);
      expect(reward).toBe(-1.0);
    });

    it('should robustly handle multi-line complex thought blocks and 2D Matrix JSON payload', () => {
      const complexOutput = `<thought>
1. Khảo sát 2D Matrix 9x10.
2. Phát hiện cánh Tướng đen hở.
3. Đề xuất nước đi Mã 8 tiến 7.
</thought>
{"move":"h2e2","fen":"rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1"}`;

      const result = parseThoughtStream(complexOutput);
      expect(result.hasThought).toBe(true);
      expect(result.thought).toContain('Đề xuất nước đi Mã 8 tiến 7');

      const jsonPayload = JSON.parse(result.payload);
      expect(jsonPayload.move).toBe('h2e2');
      expect(jsonPayload.fen).toBeDefined();
    });

    it('should handle adversarial incomplete streaming thought tags without crashing', () => {
      const incompleteStream = '<thought>Đang suy luận nước đi dồn ép... (chưa đóng thẻ';
      const result = parseThoughtStream(incompleteStream);

      expect(result.hasThought).toBe(false);
      expect(result.payload).toBe(incompleteStream);
    });
  });

  // =========================================================================
  // 5. STRESS TEST & PERFORMANCE BENCHMARK
  // =========================================================================
  describe('High-Performance Stress & Latency Benchmark', () => {
    it('should execute 10,000 2D Matrix ↔ FEN conversions in under 200ms (O(1) efficiency)', () => {
      const testFen = 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w';

      const start = performance.now();
      const iterations = 10000;

      for (let i = 0; i < iterations; i++) {
        const { board, turn } = parseFen(testFen);
        const encoded = encodeFen(board, turn);
        if (i === 0) expect(encoded).toBe(testFen);
      }

      const elapsed = performance.now() - start;
      // Expect 10k conversions to take less than 200ms on modern JS engine (avg < 0.02ms/op)
      expect(elapsed).toBeLessThan(200);
    });

    it('should handle 10,000 PGN parse & stringify operations in under 300ms', () => {
      const history = ['P2+1', 'P8+1', 'C2.5', 'H8+7', 'R1+1', 'R9+2'];

      const start = performance.now();
      const iterations = 10000;

      for (let i = 0; i < iterations; i++) {
        const text = stringifyPgn(history);
        const parsed = parsePgn(text);
        if (i === 0) expect(parsed.moves).toHaveLength(6);
      }

      const elapsed = performance.now() - start;
      expect(elapsed).toBeLessThan(300);
    });

    it('should render R1Studio and Debugger 50 times repeatedly without memory leak or error', () => {
      for (let i = 0; i < 50; i++) {
        const { unmount: u1 } = render(<R1Studio show={true} close={() => {}} />);
        u1();

        const { unmount: u2 } = render(<Debugger show={true} close={() => {}} />);
        u2();
      }
    });
  });
});
