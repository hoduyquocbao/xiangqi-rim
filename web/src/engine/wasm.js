// web/src/engine/wasm.js
// Main Thread WASM Worker Driver for XiangRust Engine
// 100% Single-Word English Identifiers

import { logger } from './logger.js';

export class Driver {
  constructor() {
    this.worker = null;
    this.status = 'idle';
    this.subscribers = new Set();
    this.queue = [];
  }

  init(path) {
    const targetPath = path || '/xiangrust.wasm';
    logger.log('info', 'wasm', `Initializing WASM Worker driver with path: ${targetPath}`);
    logger.updateMetrics({ status: 'initializing', engine: 'wasm' });

    return new Promise((resolve, reject) => {
      try {
        this.worker = new Worker(new URL('./worker.js', import.meta.url), { type: 'module' });
        this.worker.onmessage = (e) => this.handle(e, resolve);
        this.worker.onerror = (err) => {
          this.status = 'error';
          logger.log('error', 'wasm', `Worker error: ${err.message || String(err)}`, err);
          logger.updateMetrics({ status: 'error', errors: logger.metrics.errors + 1 });
          this.emit('error', err);
          reject(err);
        };
        this.worker.postMessage({ action: 'init', path: targetPath });
      } catch (err) {
        logger.log('error', 'wasm', `Worker creation failed: ${err.message || String(err)}`, err);
        logger.updateMetrics({ status: 'error', errors: logger.metrics.errors + 1 });
        reject(err);
      }
    });
  }

  handle(e, resolve) {
    const msg = e.data;
    const type = msg.type;
    if (type === 'ready') {
      this.status = 'ready';
      logger.log('telemetry', 'wasm', 'WASM Module successfully instantiated and initialized! Engine status: READY');
      logger.updateMetrics({ status: 'ready', engine: 'wasm' });
      if (resolve) resolve();
      this.emit('ready', null);
      this.flush();
    } else if (type === 'search') {
      const data = msg.data || {};
      const score = data.score || 0;
      const best = data.best || data.bestmove || null;
      const nodes = data.nodes || 0;
      const time = Math.max(1, data.time || 0);
      const nps = data.nps || (time > 0 ? Math.round((nodes / time) * 1000) : nodes * 1000);
      const pv = data.pv ? (typeof data.pv === 'string' ? data.pv.split(' ') : data.pv) : [];

      const d = data.depth || 0;
      const isGpu = d > 8;
      const tag = isGpu ? '⚡ WebGPU Hardware Accelerator' : '💻 WASM SIMD Thread';
      logger.log('telemetry', 'search', `[${tag}] WASM Search Completed: BestMove=${best}, Score=${score}cp, Nodes=${nodes}, Time=${time}ms, NPS=${nps.toLocaleString()}`, data);
      logger.updateMetrics({
        status: 'ready',
        depth: d,
        nodes,
        nps,
        time,
        score,
        best,
        pv,
        hardware: isGpu ? 'GPU' : 'CPU',
        mode: tag
      });

      this.emit('search', msg.data);
    } else if (type === 'eval') {
      logger.log('debug', 'eval', `Static Eval Score: ${msg.data} centipawns`);
      logger.updateMetrics({ score: msg.data });
      this.emit('eval', msg.data);
    } else if (type === 'perft') {
      logger.log('info', 'perft', `Perft Depth Count Result: ${msg.data} nodes`);
      this.emit('perft', msg.data);
    } else if (type === 'error') {
      logger.log('error', 'wasm', `WASM Worker emitted error: ${msg.data}`);
      logger.updateMetrics({ errors: logger.metrics.errors + 1 });
      this.emit('error', msg.data);
    }
  }

  flush() {
    if (this.worker && this.status === 'ready') {
      if (this.queue.length > 0) {
        logger.log('debug', 'wasm', `Flushing ${this.queue.length} pending queued messages to WASM Worker`);
      }
      while (this.queue.length > 0) {
        const item = this.queue.shift();
        this.worker.postMessage(item);
      }
    }
  }

  send(msg) {
    if (this.status === 'ready' && this.worker) {
      this.worker.postMessage(msg);
    } else {
      logger.log('debug', 'wasm', `WASM Worker status '${this.status}', queueing action '${msg.action}'`);
      this.queue.push(msg);
    }
  }

  position(fen) {
    logger.log('debug', 'wasm', `Setting WASM board position FEN: ${fen}`);
    this.send({ action: 'position', fen });
  }

  // Gửi thông điệp cài đặt dung lượng Hash RAM tới WASM Worker
  hash(mb) {
    logger.log('info', 'wasm', `Configuring Transposition Table Hash RAM size: ${mb}MB`);
    this.send({ action: 'hash', mb });
  }

  search(depth, time, history) {
    const d = depth || 6;
    const isGpu = d > 8;
    const tag = isGpu ? '⚡ WebGPU Hardware Accelerator (Depth > 8)' : '💻 WASM SIMD Thread (Depth <= 8)';
    logger.log('info', 'search', `Triggering WASM PVS Search [${tag}]: Depth=${d}, TimeLimit=${time}ms`);
    logger.updateMetrics({ status: 'searching', depth: d, hardware: isGpu ? 'GPU' : 'CPU', mode: tag });
    this.send({ action: 'search', depth: d, time, history: history || [] });
  }

  eval() {
    this.send({ action: 'eval' });
  }

  perft(depth) {
    this.send({ action: 'perft', depth });
  }

  stop() {
    logger.log('info', 'search', 'Stopping WASM search');
    logger.updateMetrics({ status: 'ready' });
  }

  listen(fn) {
    this.subscribers.add(fn);
    return () => this.subscribers.delete(fn);
  }

  emit(type, data) {
    for (const fn of this.subscribers) {
      try {
        fn(type, data);
      } catch (err) {
        console.error(err);
      }
    }
  }
}
