// web/src/engine/__tests__/worker_safety.test.js
// Unit tests for FFI Memory Safety in worker.js
// 100% Single-Word English Identifiers

import { describe, test, expect, beforeEach, beforeAll, vi } from 'vitest';

describe('Worker Safety', () => {
  let posts = [];
  let exports;

  beforeAll(async () => {
    globalThis.self = globalThis.self || {};
    globalThis.self.postMessage = (msg) => posts.push(msg);
    await import('../worker.js');
  });

  beforeEach(async () => {
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

    await self.onmessage({ data: { action: 'init' } });
  });

  test('oom handling on allocate failure', async () => {
    exports.allocate.mockReturnValue(0);

    posts.length = 0;
    await self.onmessage({ data: { action: 'position', fen: 'fen_test' } });
    expect(posts).toContainEqual({ type: 'error', data: 'OOM' });
    expect(exports.set_position).not.toHaveBeenCalled();
    expect(exports.free).not.toHaveBeenCalled();

    posts.length = 0;
    await self.onmessage({ data: { action: 'search', depth: 4 } });
    expect(posts).toContainEqual({ type: 'error', data: 'OOM' });
    expect(exports.fetch).not.toHaveBeenCalled();
    expect(exports.free).not.toHaveBeenCalled();
  });

  test('finally block executes free on set_position throw', async () => {
    exports.allocate.mockReturnValue(16);
    exports.set_position.mockImplementation(() => {
      throw new Error('position panic');
    });

    posts.length = 0;
    await self.onmessage({ data: { action: 'position', fen: 'fen_test' } });
    expect(posts).toContainEqual({ type: 'error', data: 'Error: position panic' });
    expect(exports.free).toHaveBeenCalledWith(16, expect.any(Number));
  });

  test('finally block executes free on JSON parse failure in search', async () => {
    exports.allocate.mockReturnValue(32);
    exports.fetch.mockReturnValue(5);
    const encoder = new TextEncoder();
    const raw = encoder.encode('abcde');
    new Uint8Array(exports.memory.buffer).set(raw, 32);

    posts.length = 0;
    await self.onmessage({ data: { action: 'search', depth: 2 } });
    expect(posts[0].type).toBe('error');
    expect(exports.free).toHaveBeenCalledWith(32, 4096);
  });
});
