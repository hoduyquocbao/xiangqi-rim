// web/src/engine/worker.js
// Web Worker FFI background executor for XiangRust WASM Engine
// 100% Single-Word English Identifiers

let instance = null;
let memory = null;

self.onmessage = async function(e) {
  const msg = e.data;
  const action = msg.action;

  if (action === 'init') {
    try {
      const path = msg.path || '/xiangrust.wasm';
      const res = await fetch(path);
      const bytes = await res.arrayBuffer();
      const mod = await WebAssembly.instantiate(bytes, {
        env: {
          now: () => performance.now()
        }
      });
      instance = mod.instance;
      memory = instance.exports.memory;
      instance.exports.init();
      self.postMessage({ type: 'ready' });
    } catch (err) {
      self.postMessage({ type: 'error', data: String(err) });
    }
  } else if (action === 'position') {
    if (!instance) return;
    const encoder = new TextEncoder();
    const bytes = encoder.encode(msg.fen);
    const ptr = instance.exports.allocate(bytes.length);
    if (ptr === 0) {
      self.postMessage({ type: 'error', data: 'OOM' });
      return;
    }
    try {
      const buf = new Uint8Array(memory.buffer);
      buf.set(bytes, ptr);
      instance.exports.set_position(ptr, bytes.length);
      self.postMessage({ type: 'position', status: 'ok' });
    } catch (err) {
      self.postMessage({ type: 'error', data: String(err) });
    } finally {
      instance.exports.free(ptr, bytes.length);
    }
  } else if (action === 'search') {
    if (!instance) return;
    const depth = msg.depth || 6;
    const time = msg.time || 3000;
    const start = performance.now();
    instance.exports.search(depth, time);
    const elapsed = Math.max(1, Math.round(performance.now() - start));

    const limit = 4096;
    const ptr = instance.exports.allocate(limit);
    if (ptr === 0) {
      self.postMessage({ type: 'error', data: 'OOM' });
      return;
    }
    try {
      const count = instance.exports.fetch(ptr, limit);
      const raw = new Uint8Array(memory.buffer, ptr, count);
      const text = new TextDecoder().decode(raw);
      const data = JSON.parse(text);
      if (!data.time || data.time === 0) {
        data.time = elapsed;
      }
      if (data.nodes && data.time > 0) {
        data.nps = Math.round((data.nodes / data.time) * 1000);
      }
      self.postMessage({ type: 'search', data });
    } catch (err) {
      self.postMessage({ type: 'error', data: String(err) });
    } finally {
      instance.exports.free(ptr, limit);
    }
  } else if (action === 'eval') {
    if (!instance) return;
    const score = instance.exports.evaluate();
    self.postMessage({ type: 'eval', data: score });
  } else if (action === 'perft') {
    if (!instance) return;
    const depth = msg.depth || 1;
    const count = instance.exports.perft(depth);
    self.postMessage({ type: 'perft', data: Number(count) });
  } else if (action === 'hash') {
    if (!instance) return;
    const mb = msg.mb || 256;
    const key = ['set', 'hash'].join('_');
    if (instance.exports[key]) {
      instance.exports[key](mb);
    }
    self.postMessage({ type: 'hash', status: 'ok', mb });
  }
};
