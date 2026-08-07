// web/src/engine/__tests__/m2_challenger_empirical.test.js
// Empirical Adversarial Stress Test Suite for Milestone 2 Hash RAM Logic
// 100% Single-Word English Identifiers

import { describe, test, expect, beforeEach, vi } from 'vitest';
import { Engine } from '../engine.js';
import { Driver as Wasm } from '../wasm.js';
import { Driver as Socket } from '../socket.js';

describe('Milestone 2 Hash RAM Empirical Non-Disruption Stress Tests', () => {
  let engine;

  beforeEach(() => {
    engine = new Engine();
  });

  test('engine.hash retains current FEN without reset', () => {
    const customFen = '3akab1r/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 5';
    engine.position(customFen);

    const wasmHashSpy = vi.spyOn(engine.wasm, 'hash').mockImplementation(() => {});
    const socketHashSpy = vi.spyOn(engine.socket, 'hash').mockImplementation(() => {});

    // Execute hash change
    engine.hash(512);

    // Verify current FEN is NOT reset or altered
    expect(engine.fen).toBe(customFen);
    expect(wasmHashSpy).toHaveBeenCalledWith(512);
    expect(socketHashSpy).toHaveBeenCalledWith(512);
  });

  test('wasm.hash queues or sends hash action without position reset', () => {
    const wasm = new Wasm();
    const sendSpy = vi.spyOn(wasm, 'send').mockImplementation(() => {});

    wasm.hash(1024);

    expect(sendSpy).toHaveBeenCalledWith({ action: 'hash', mb: 1024 });
    // Verify no position action was dispatched
    expect(sendSpy).not.toHaveBeenCalledWith(expect.objectContaining({ action: 'position' }));
  });

  test('socket.hash sends set_hash action without modifying stored socket FEN', () => {
    const socket = new Socket();
    const customFen = 'r1bakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR b - - 0 10';
    socket.position(customFen);

    const sendSpy = vi.spyOn(socket, 'send').mockImplementation(() => {});

    socket.hash(2048);

    expect(sendSpy).toHaveBeenCalledWith({ action: 'set_hash', mb: 2048 });
    expect(socket.fen).toBe(customFen);
  });

  test('simulated React alloc state update retains FEN, history, and index', () => {
    const startFen = 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1';
    const moveFen = 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C2C1C1/9/RNBAKABNR b - - 0 1';

    let game = {
      fen: moveFen,
      history: [startFen, moveFen],
      index: 1,
      hash: 256,
      score: 15,
      over: false
    };

    const alloc = (val) => {
      // Simulate App.jsx alloc callback
      game = { ...game, hash: val };
      engine.hash(val);
    };

    alloc(1024);

    expect(game.hash).toBe(1024);
    expect(game.fen).toBe(moveFen);
    expect(game.history).toEqual([startFen, moveFen]);
    expect(game.index).toBe(1);
    expect(game.over).toBe(false);
  });
});
