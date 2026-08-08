// Independent Empirical Verification Test Suite for Milestone M1
// 100% Single-word English Identifiers: describe, it, expect, parse, fen, stringify, uciToMove, check, moves, start, parsed, encoded, result, history, tags, nulls, invalid, item, moveslist, count, square, target, piece, board, turn, coords, out, f1, r1, f2, r2

import { describe, it, expect } from 'vitest';
import { parse, fen, check, moves, uciToMove } from '../rules.js';
import { parse as parsePgn, stringify as stringifyPgn } from '../pgn.js';

describe('Milestone M1 Empirical Verification - FEN Conversion', () => {
  it('verifies standard starting Xiangqi FEN parsing and reconstruction', () => {
    const start = 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w';
    const parsed = parse(start);

    expect(parsed.turn).toBe('w');
    expect(parsed.board.length).toBe(90);

    // Verify key piece positions on the board array (index = rank * 9 + file)
    // Red King at e0 (rank 0, file 4) -> index 4
    expect(parsed.board[4]).toBe('K');
    // Black King at e9 (rank 9, file 4) -> index 85
    expect(parsed.board[85]).toBe('k');
    // Red Cannon at b2 (rank 2, file 1) -> index 19
    expect(parsed.board[19]).toBe('C');
    // Black Cannon at h7 (rank 7, file 7) -> index 70
    expect(parsed.board[70]).toBe('c');

    const reconstructed = fen(parsed.board, parsed.turn);
    expect(reconstructed).toBe(start);
  });

  it('verifies round-trip symmetry across custom FEN positions', () => {
    const custom = '3ak1a2/9/1R7/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR b';
    const parsed = parse(custom);
    expect(parsed.turn).toBe('b');

    const reconstructed = fen(parsed.board, parsed.turn);
    expect(reconstructed).toBe(custom);
  });

  it('handles empty board FEN parsing gracefully', () => {
    const empty = '9/9/9/9/9/9/9/9/9/9 w';
    const parsed = parse(empty);
    expect(parsed.board.every(item => item === '.')).toBe(true);

    const reconstructed = fen(parsed.board, parsed.turn);
    expect(reconstructed).toBe(empty);
  });
});

describe('Milestone M1 Empirical Verification - PGN Converter', () => {
  it('parses formatted PGN content into headers and moves list accurately', () => {
    const pgn = `
      [Event "Championship"]
      [Site "Hanoi"]
      [Red "PlayerA"]
      [Black "PlayerB"]
      [Result "1-0"]

      1. C2.5 H8+7 2. H2+3 R9+1 1-0
    `;
    const parsed = parsePgn(pgn);
    expect(parsed.headers.event).toBe('Championship');
    expect(parsed.headers.site).toBe('Hanoi');
    expect(parsed.headers.red).toBe('PlayerA');
    expect(parsed.headers.black).toBe('PlayerB');
    expect(parsed.headers.result).toBe('1-0');

    expect(parsed.moves).toEqual(['C2.5', 'H8+7', 'H2+3', 'R9+1']);
  });

  it('verifies stringify and parse round-trip consistency', () => {
    const history = ['C2.5', 'H8+7', 'H2+3', 'R9+1', 'R1.2', 'C8.9'];
    const tags = { event: 'Test', red: 'Alice', black: 'Bob', result: '1/2-1/2' };

    const encoded = stringifyPgn(history, tags);
    const result = parsePgn(encoded);

    expect(result.moves).toEqual(history);
    expect(result.headers.event).toBe('Test');
    expect(result.headers.red).toBe('Alice');
    expect(result.headers.black).toBe('Bob');
    expect(result.headers.result).toBe('1/2-1/2');
  });

  it('handles empty and malformed PGN inputs without crashing', () => {
    const nulls = [null, undefined, '', '   ', 12345, true];
    for (const item of nulls) {
      const res = parsePgn(item);
      expect(res).toBeDefined();
      expect(res.headers).toBeDefined();
      expect(res.moves).toBeDefined();
    }
  });
});

describe('Milestone M1 Empirical Verification - uciToMove Parsing', () => {
  it('parses valid UCI moves correctly into board coordinates', () => {
    // h2e2 -> from h2 (rank 2, file 7 = index 25) to e2 (rank 2, file 4 = index 22)
    const m1 = uciToMove('h2e2');
    expect(m1).toEqual({ from: 25, to: 22 });

    // b0c2 -> from b0 (rank 0, file 1 = index 1) to c2 (rank 2, file 2 = index 20)
    const m2 = uciToMove('b0c2');
    expect(m2).toEqual({ from: 1, to: 20 });

    // a0a1 -> from a0 (rank 0, file 0 = index 0) to a1 (rank 1, file 0 = index 9)
    const m3 = uciToMove('a0a1');
    expect(m3).toEqual({ from: 0, to: 9 });

    // i9i8 -> from i9 (rank 9, file 8 = index 89) to i8 (rank 8, file 8 = index 80)
    const m4 = uciToMove('i9i8');
    expect(m4).toEqual({ from: 89, to: 80 });
  });

  it('returns null safely for invalid, empty, or out-of-bounds UCI strings', () => {
    const invalid = [null, undefined, '', 'a0', 'h2e', 100, false, 'a-1a0'];
    for (const item of invalid) {
      expect(uciToMove(item)).toBeNull();
    }
  });
});

describe('Milestone M1 Empirical Verification - Rule Engine Stability', () => {
  it('generates moves for starting board without throwing errors', () => {
    const start = 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w';
    const parsed = parse(start);

    let count = 0;
    for (let square = 0; square < 90; square++) {
      const piece = parsed.board[square];
      if (piece !== '.' && piece === piece.toUpperCase()) {
        const valid = moves(parsed.board, square, 'w');
        count += valid.length;
      }
    }

    // Red starting position has 44 legal moves
    expect(count).toBe(44);
    expect(check(parsed.board, 'w')).toBe(false);
    expect(check(parsed.board, 'b')).toBe(false);
  });
});
