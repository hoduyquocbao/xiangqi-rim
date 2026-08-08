// Empirical Verification Harness for PGN Notation Parsing
// 100% Single-word English Identifiers

import { describe, it, expect } from 'vitest';
import { parse as parsePgn } from '../pgn.js';
import { parse as parseFen, uciToMove } from '../rules.js';

describe('Empirical Verification Suite', () => {
  it('parses single move C2.5 correctly', () => {
    const text = 'C2.5';
    const res = parsePgn(text);
    expect(res.moves).toEqual(['C2.5']);
  });

  it('parses single move R1.2 correctly', () => {
    const text = 'R1.2';
    const res = parsePgn(text);
    expect(res.moves).toEqual(['R1.2']);
  });

  it('parses single move C8.9 correctly', () => {
    const text = 'C8.9';
    const res = parsePgn(text);
    expect(res.moves).toEqual(['C8.9']);
  });

  it('parses formatted PGN line 1. C2.5 H8.7 2. R1.2 correctly', () => {
    const text = '1. C2.5 H8.7 2. R1.2';
    const res = parsePgn(text);
    expect(res.moves).toEqual(['C2.5', 'H8.7', 'R1.2']);
  });

  it('parses multi move line with dot notation sequence', () => {
    const text = '1. C2.5 H8.7 2. R1.2 C8.9 3. H2+3 R9+1';
    const res = parsePgn(text);
    expect(res.moves).toEqual(['C2.5', 'H8.7', 'R1.2', 'C8.9', 'H2+3', 'R9+1']);
  });

  it('parses unspaced move numbers like 1.C2.5 2.R1.2 without truncation', () => {
    const text = '1.C2.5 2.R1.2 3.C8.9';
    const res = parsePgn(text);
    expect(res.moves).toEqual(['C2.5', 'R1.2', 'C8.9']);
  });

  it('parses PGN with black move dots like 1... Hh8-g6 correctly', () => {
    const text = '1. Che2-e6 1... Hh8-g6';
    const res = parsePgn(text);
    expect(res.moves).toEqual(['Che2-e6', 'Hh8-g6']);
  });

  it('verifies uciToMove strict bounds checking for file j and invalid coordinates', () => {
    const out = uciToMove('j0a0');
    expect(out).toBeNull();

    const valid = uciToMove('a0a1');
    expect(valid).toEqual({ from: 0, to: 9 });
  });

  it('defends parse(fen) against null, non-string, or incomplete row counts', () => {
    const res1 = parseFen(null);
    expect(res1.board).toHaveLength(90);
    expect(res1.turn).toBe('w');

    const res2 = parseFen(12345);
    expect(res2.board).toHaveLength(90);
    expect(res2.turn).toBe('w');

    const res3 = parseFen('rnbakabnr/9 w');
    expect(res3.board).toHaveLength(90);
    expect(res3.turn).toBe('w');
  });
});
