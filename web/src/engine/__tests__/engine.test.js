// web/src/engine/__tests__/engine.test.js
// Unit tests for XiangRust Engine Facade and Drivers
// 100% Single-Word English Identifiers

import { describe, test, expect, beforeEach, vi } from 'vitest';
import { Engine } from '../engine.js';
import { Driver as Wasm } from '../wasm.js';
import { Driver as Socket } from '../socket.js';

describe('Engine', () => {
  let engine;

  beforeEach(() => {
    engine = new Engine();
  });

  test('init', async () => {
    expect(engine.mode()).toBe('wasm');
    const spy = vi.spyOn(engine.wasm, 'init').mockResolvedValue();
    await engine.init('wasm', { path: '/xiangrust.wasm' });
    expect(spy).toHaveBeenCalledWith('/xiangrust.wasm');
  });

  test('mode', () => {
    expect(engine.mode()).toBe('wasm');
    engine.mode('socket');
    expect(engine.mode()).toBe('socket');
  });

  test('position', () => {
    const fen = 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1';
    const wasm = vi.spyOn(engine.wasm, 'position');
    const socket = vi.spyOn(engine.socket, 'position');

    engine.position(fen);

    expect(engine.fen).toBe(fen);
    expect(wasm).toHaveBeenCalledWith(fen);
    expect(socket).toHaveBeenCalledWith(fen);
  });

  test('search', () => {
    const wasm = vi.spyOn(engine.wasm, 'search').mockImplementation(() => {});
    engine.search(6, 3000);
    expect(wasm).toHaveBeenCalledWith(6, 3000);

    engine.mode('socket');
    const socket = vi.spyOn(engine.socket, 'search').mockImplementation(() => {});
    engine.search(8, 5000);
    expect(socket).toHaveBeenCalledWith(8, 5000);
  });

  test('eval', () => {
    const wasm = vi.spyOn(engine.wasm, 'eval').mockImplementation(() => {});
    engine.eval();
    expect(wasm).toHaveBeenCalled();

    engine.mode('socket');
    const socket = vi.spyOn(engine.socket, 'eval').mockImplementation(() => {});
    engine.eval();
    expect(socket).toHaveBeenCalled();
  });

  test('perft', () => {
    const wasm = vi.spyOn(engine.wasm, 'perft').mockImplementation(() => {});
    engine.perft(2);
    expect(wasm).toHaveBeenCalledWith(2);

    engine.mode('socket');
    const socket = vi.spyOn(engine.socket, 'perft').mockImplementation(() => {});
    engine.perft(3);
    expect(socket).toHaveBeenCalledWith(3);
  });

  test('stop', () => {
    const wasm = vi.spyOn(engine.wasm, 'stop').mockImplementation(() => {});
    engine.stop();
    expect(wasm).toHaveBeenCalled();

    engine.mode('socket');
    const socket = vi.spyOn(engine.socket, 'stop').mockImplementation(() => {});
    engine.stop();
    expect(socket).toHaveBeenCalled();
  });

  test('hash', () => {
    const wasm = vi.spyOn(engine.wasm, 'hash').mockImplementation(() => {});
    const socket = vi.spyOn(engine.socket, 'hash').mockImplementation(() => {});

    engine.hash(512);

    expect(wasm).toHaveBeenCalledWith(512);
    expect(socket).toHaveBeenCalledWith(512);
  });

  test('driver hash', () => {
    const wasm = new Wasm();
    const wasmSpy = vi.spyOn(wasm, 'send').mockImplementation(() => {});
    wasm.hash(1024);
    expect(wasmSpy).toHaveBeenCalledWith({ action: 'hash', mb: 1024 });

    const socket = new Socket();
    const socketSpy = vi.spyOn(socket, 'send').mockImplementation(() => {});
    socket.hash(2048);
    expect(socketSpy).toHaveBeenCalledWith({ action: 'set_hash', mb: 2048 });
  });

  test('listen', () => {
    const listener = vi.fn();
    const unsub = engine.listen(listener);

    engine.emit('ready', null);
    expect(listener).toHaveBeenCalledWith('ready', null);

    unsub();
    engine.emit('ready', null);
    expect(listener).toHaveBeenCalledTimes(1);
  });

  test('subscriber exception safety in engine facade', () => {
    const spy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const faulty = vi.fn().mockImplementation(() => {
      throw new Error('faulty subscriber');
    });
    const valid = vi.fn();

    engine.listen(faulty);
    engine.listen(valid);

    expect(() => engine.emit('ready', null)).not.toThrow();
    expect(faulty).toHaveBeenCalledWith('ready', null);
    expect(valid).toHaveBeenCalledWith('ready', null);
    expect(spy).toHaveBeenCalled();
    spy.mockRestore();
  });

  test('subscriber exception safety in wasm driver', () => {
    const spy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const wasm = new Wasm();
    const faulty = vi.fn().mockImplementation(() => {
      throw new Error('wasm subscriber fail');
    });
    const valid = vi.fn();

    wasm.listen(faulty);
    wasm.listen(valid);

    expect(() => wasm.emit('search', { depth: 4 })).not.toThrow();
    expect(faulty).toHaveBeenCalledWith('search', { depth: 4 });
    expect(valid).toHaveBeenCalledWith('search', { depth: 4 });
    expect(spy).toHaveBeenCalled();
    spy.mockRestore();
  });

  test('subscriber exception safety in socket driver', () => {
    const spy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const socket = new Socket();
    const faulty = vi.fn().mockImplementation(() => {
      throw new Error('socket subscriber fail');
    });
    const valid = vi.fn();

    socket.listen(faulty);
    socket.listen(valid);

    expect(() => socket.emit('info', { score: 100 })).not.toThrow();
    expect(faulty).toHaveBeenCalledWith('info', { score: 100 });
    expect(valid).toHaveBeenCalledWith('info', { score: 100 });
    expect(spy).toHaveBeenCalled();
    spy.mockRestore();
  });
});
