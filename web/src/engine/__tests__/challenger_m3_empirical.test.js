// web/src/engine/__tests__/challenger_m3_empirical.test.js
// Challenger 3 Empirical Verification Harness for Dual-Engine Core Integration
// 100% Single-Word English Identifiers

import { describe, test, expect, beforeEach, beforeAll, vi } from 'vitest';
import { Engine } from '../engine.js';
import { Driver as Wasm } from '../wasm.js';
import { Driver as Socket } from '../socket.js';

describe('Challenger 3 Empirical FFI Memory Leak & Exception Safety', () => {
  let posts = [];
  let exports;

  beforeAll(async () => {
    globalThis.self = globalThis.self || {};
    globalThis.self.postMessage = (msg) => posts.push(msg);
    await import('../worker.js');
  });

  beforeEach(() => {
    posts.length = 0;
    globalThis.self.postMessage = (msg) => posts.push(msg);

    exports = {
      allocate: vi.fn(),
      free: vi.fn(),
      set_position: vi.fn(),
      fetch: vi.fn(),
      search: vi.fn(),
      evaluate: vi.fn(),
      perft: vi.fn(),
      init: vi.fn(),
      memory: { buffer: new ArrayBuffer(1024) }
    };

    globalThis.fetch = vi.fn().mockResolvedValue({
      arrayBuffer: () => Promise.resolve(new ArrayBuffer(10))
    });
    globalThis.WebAssembly = {
      instantiate: vi.fn().mockResolvedValue({ instance: { exports } })
    };
  });

  test('FFI Memory Safety: free is called when set_position throws in worker', async () => {
    await self.onmessage({ data: { action: 'init' } });
    exports.allocate.mockReturnValue(128);
    exports.set_position.mockImplementation(() => {
      throw new Error('Rust WASM Panic during set_position');
    });

    posts.length = 0;
    await self.onmessage({ data: { action: 'position', fen: 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1' } });

    expect(posts).toContainEqual({ type: 'error', data: 'Error: Rust WASM Panic during set_position' });
    expect(exports.free).toHaveBeenCalledTimes(1);
    expect(exports.free).toHaveBeenCalledWith(128, expect.any(Number));
  });

  test('FFI Memory Safety: free is called when JSON.parse throws in worker search', async () => {
    await self.onmessage({ data: { action: 'init' } });
    exports.allocate.mockReturnValue(256);
    exports.fetch.mockReturnValue(10);

    const encoder = new TextEncoder();
    const badJson = encoder.encode('{ invalid json content }');
    new Uint8Array(exports.memory.buffer).set(badJson, 256);

    posts.length = 0;
    await self.onmessage({ data: { action: 'search', depth: 6, time: 3000 } });

    expect(posts.length).toBe(1);
    expect(posts[0].type).toBe('error');
    expect(posts[0].data).toContain('SyntaxError');
    expect(exports.free).toHaveBeenCalledTimes(1);
    expect(exports.free).toHaveBeenCalledWith(256, 4096);
  });

  test('FFI Memory Safety: free is called when postMessage throws in worker search', async () => {
    await self.onmessage({ data: { action: 'init' } });
    exports.allocate.mockReturnValue(512);
    exports.fetch.mockReturnValue(13);

    const encoder = new TextEncoder();
    const validJson = encoder.encode('{"depth":6}');
    new Uint8Array(exports.memory.buffer).set(validJson, 512);

    globalThis.self.postMessage = vi.fn().mockImplementation((msg) => {
      if (msg.type === 'search') throw new Error('Worker postMessage channel closed');
    });

    await self.onmessage({ data: { action: 'search', depth: 6, time: 3000 } });

    expect(exports.free).toHaveBeenCalledTimes(1);
    expect(exports.free).toHaveBeenCalledWith(512, 4096);
  });

  test('Subscriber Exception Safety: emit delivers to all non-faulty subscribers despite failures', () => {
    const spy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const engine = new Engine();
    const results = [];

    const sub0 = vi.fn(() => results.push(0));
    const sub1 = vi.fn(() => { throw new Error('Subscriber 1 exception'); });
    const sub2 = vi.fn(() => results.push(2));
    const sub3 = vi.fn(() => results.push(3));
    const sub4 = vi.fn(() => { throw new Error('Subscriber 4 exception'); });
    const sub5 = vi.fn(() => results.push(5));

    engine.listen(sub0);
    engine.listen(sub1);
    engine.listen(sub2);
    engine.listen(sub3);
    engine.listen(sub4);
    engine.listen(sub5);

    expect(() => engine.emit('test', { key: 'val' })).not.toThrow();

    expect(sub0).toHaveBeenCalledWith('test', { key: 'val' });
    expect(sub1).toHaveBeenCalledWith('test', { key: 'val' });
    expect(sub2).toHaveBeenCalledWith('test', { key: 'val' });
    expect(sub3).toHaveBeenCalledWith('test', { key: 'val' });
    expect(sub4).toHaveBeenCalledWith('test', { key: 'val' });
    expect(sub5).toHaveBeenCalledWith('test', { key: 'val' });

    expect(results).toEqual([0, 2, 3, 5]);
    expect(spy).toHaveBeenCalledTimes(2);
    spy.mockRestore();
  });
});
