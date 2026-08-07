// Empirical Stress and Edge-Case Test Suite for PGN Parser & Encoder
// 100% Single-word English Identifiers: describe, it, expect, parse, stringify, text, result, moves, tags, output, parsed, bulk, i, iccs

import { describe, it, expect } from 'vitest';
import { parse, stringify } from '../pgn.js';

describe('PGN Stress & Boundary Empirical Testing', () => {
  it('preserves move notation with digits and dots like C2.5 and R1.2 intact', () => {
    const text = '1. C2.5 H8.7 2. R1.2 C8.9';
    const parsed = parse(text);
    expect(parsed.moves).toEqual(['C2.5', 'H8.7', 'R1.2', 'C8.9']);
    expect(parsed.moves[0]).toBe('C2.5');
    expect(parsed.moves[2]).toBe('R1.2');
  });

  it('parses complex Xiangqi game record with FEN tag and comments', () => {
    const text = `
      [Event "World Xiangqi Championship 2026"]
      [Site "Hanoi, Vietnam"]
      [Date "2026.08.06"]
      [Round "5"]
      [Red "Lai Ly Huynh"]
      [Black "Wang Tianyi"]
      [Result "1-0"]
      [FEN "rnbakabnr/9/9/9/9/9/9/9/9/RNBAKABNR w - - 0 1"]

      { Opening: Central Cannon vs Same Direction Cannon }
      1. C2.5 H8+7 { Standard response }
      2. H2+3 R9+1 3. R1.2 H2+3
      4. R2+6 C8.9 5. R2.7 C9/2
      6. H8+7 R9.4 7. C8.9 R4+4 1-0
    `;

    const result = parse(text);
    expect(result.headers.event).toBe('World Xiangqi Championship 2026');
    expect(result.headers.site).toBe('Hanoi, Vietnam');
    expect(result.headers.date).toBe('2026.08.06');
    expect(result.headers.round).toBe('5');
    expect(result.headers.red).toBe('Lai Ly Huynh');
    expect(result.headers.black).toBe('Wang Tianyi');
    expect(result.headers.result).toBe('1-0');
    expect(result.headers.fen).toBe('rnbakabnr/9/9/9/9/9/9/9/9/RNBAKABNR w - - 0 1');

    expect(result.moves).toEqual([
      'C2.5', 'H8+7',
      'H2+3', 'R9+1',
      'R1.2', 'H2+3',
      'R2+6', 'C8.9',
      'R2.7', 'C9/2',
      'H8+7', 'R9.4',
      'C8.9', 'R4+4'
    ]);
  });

  it('handles empty, invalid, and boundary inputs gracefully without throwing', () => {
    expect(parse(null)).toEqual({ headers: {}, moves: [] });
    expect(parse(undefined)).toEqual({ headers: {}, moves: [] });
    expect(parse(12345)).toEqual({ headers: {}, moves: [] });
    expect(parse('')).toEqual({ headers: {}, moves: [] });
    expect(parse('   \n\n   ')).toEqual({ headers: {}, moves: [] });
  });

  it('parses ICCS coordinate move notation PGNs correctly', () => {
    const iccs = `
      [Event "ICCS Match"]
      1. h2e2 h8g6 2. h1g3 i9h9 3. i1h1 b9c7 1/2-1/2
    `;
    const result = parse(iccs);
    expect(result.moves).toEqual(['h2e2', 'h8g6', 'h1g3', 'i9h9', 'i1h1', 'b9c7']);
    expect(result.headers.event).toBe('ICCS Match');
  });

  it('verifies round-trip symmetry between stringify and parse', () => {
    const moves = ['C2.5', 'H8+7', 'H2+3', 'R9+1', 'R1.2', 'C8.9', 'R2+6'];
    const tags = {
      event: 'Roundtrip Test',
      site: 'Test Room',
      date: '2026-08-06',
      red: 'Grandmaster Red',
      black: 'Grandmaster Black',
      result: '0-1'
    };

    const output = stringify(moves, tags);
    const parsed = parse(output);

    expect(parsed.moves).toEqual(moves);
    expect(parsed.headers.event).toBe(tags.event);
    expect(parsed.headers.site).toBe(tags.site);
    expect(parsed.headers.date).toBe(tags.date);
    expect(parsed.headers.red).toBe(tags.red);
    expect(parsed.headers.black).toBe(tags.black);
    expect(parsed.headers.result).toBe(tags.result);
  });

  it('handles high volume move sequences (stress testing 1000 moves)', () => {
    const bulk = [];
    for (let i = 0; i < 1000; i++) {
      bulk.push(`M${i}`);
    }

    const output = stringify(bulk, { event: 'Stress Test' });
    const parsed = parse(output);

    expect(parsed.moves.length).toBe(1000);
    expect(parsed.moves[0]).toBe('M0');
    expect(parsed.moves[999]).toBe('M999');
  });
});
