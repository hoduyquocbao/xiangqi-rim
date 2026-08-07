// Empirical Verification Harness for PGN Notation Parsing
// 100% Single-word English Identifiers

import { describe, it, expect } from 'vitest';
import { parse } from '../pgn.js';

describe('Empirical Verification Suite', () => {
  it('parses single move C2.5 correctly', () => {
    const text = 'C2.5';
    const res = parse(text);
    expect(res.moves).toEqual(['C2.5']);
  });

  it('parses single move R1.2 correctly', () => {
    const text = 'R1.2';
    const res = parse(text);
    expect(res.moves).toEqual(['R1.2']);
  });

  it('parses single move C8.9 correctly', () => {
    const text = 'C8.9';
    const res = parse(text);
    expect(res.moves).toEqual(['C8.9']);
  });

  it('parses formatted PGN line 1. C2.5 H8.7 2. R1.2 correctly', () => {
    const text = '1. C2.5 H8.7 2. R1.2';
    const res = parse(text);
    expect(res.moves).toEqual(['C2.5', 'H8.7', 'R1.2']);
  });

  it('parses multi move line with dot notation sequence', () => {
    const text = '1. C2.5 H8.7 2. R1.2 C8.9 3. H2+3 R9+1';
    const res = parse(text);
    expect(res.moves).toEqual(['C2.5', 'H8.7', 'R1.2', 'C8.9', 'H2+3', 'R9+1']);
  });

  it('parses unspaced move numbers like 1.C2.5 2.R1.2 without truncation', () => {
    const text = '1.C2.5 2.R1.2 3.C8.9';
    const res = parse(text);
    expect(res.moves).toEqual(['C2.5', 'R1.2', 'C8.9']);
  });
});
