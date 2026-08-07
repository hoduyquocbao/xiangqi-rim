// Unit test suite for PGN parser & encoder
// 100% Single-word English Identifiers

import { describe, it, expect } from 'vitest';
import { parse, stringify } from '../pgn.js';

describe('PGN Parser & Encoder', () => {
  it('parses valid PGN string with headers and moves', () => {
    const text = `
      [Event "XiangRust Championship"]
      [Red "Player Red"]
      [Black "Player Black"]

      1. Che2-e6 Hh8-g6 2. Hb1-c3 *
    `;

    const parsed = parse(text);
    expect(parsed.headers.event).toBe('XiangRust Championship');
    expect(parsed.headers.red).toBe('Player Red');
    expect(parsed.headers.black).toBe('Player Black');
    expect(parsed.moves).toEqual(['Che2-e6', 'Hh8-g6', 'Hb1-c3']);
  });

  it('stringifies move history into PGN format', () => {
    const history = ['C2.5', 'H8+7', 'H2+3'];
    const pgn = stringify(history, { event: 'Test Game' });

    expect(pgn).toContain('[Event "Test Game"]');
    expect(pgn).toContain('1. C2.5 H8+7');
    expect(pgn).toContain('2. H2+3 *');
  });

  it('handles empty inputs gracefully', () => {
    expect(parse('')).toEqual({ headers: {}, moves: [] });
    expect(stringify([])).toContain('*');
  });
});
