// web/src/engine/socket.js
// Main Thread WebSocket Server Driver for XiangRust Engine
// 100% Single-Word English Identifiers

import { logger } from './logger.js';

export class Driver {
  constructor() {
    this.socket = null;
    this.status = 'idle';
    const isSsl = typeof window !== 'undefined' && window.location.protocol === 'https:';
    const proto = isSsl ? 'wss:' : 'ws:';
    const isTest = typeof process !== 'undefined' && Boolean(process.env.VITEST);
    const hostName = typeof window !== 'undefined' && window.location.hostname && !isTest ? window.location.hostname : '127.0.0.1';
    const host = typeof window !== 'undefined' && window.location.host && !isTest ? window.location.host : `${hostName}:8888`;
    this.url = `${proto}//${host}/ws`;
    this.fallbackUrl = `${proto}//${hostName}:8888/ws`;
    this.attemptedFallback = false;
    this.subscribers = new Set();
    this.fen = 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1';
    this.queue = [];
  }

  init(url) {
    if (url) this.url = url;
    logger.log('info', 'socket', `Connecting to WebSocket Backend: ${this.url}`);
    logger.updateMetrics({ status: 'connecting', engine: 'socket' });

    return new Promise((resolve, reject) => {
      try {
        if (this.socket) {
          try { this.socket.close(); } catch (_) {}
        }
        this.socket = new WebSocket(this.url);
        this.status = 'connecting';

        this.socket.onopen = () => {
          this.status = 'ready';
          logger.log('telemetry', 'socket', `WebSocket Server Connected successfully to ${this.url}`);
          logger.updateMetrics({ status: 'ready', engine: 'socket' });
          resolve();
          this.emit('ready', null);
          this.flush();
        };

        this.socket.onmessage = (e) => {
          try {
            const data = JSON.parse(e.data);
            if (data.type === 'info') {
              const count = data.nodes || 0;
              const ms = data.time || (this.clock ? Math.max(1, Math.round(performance.now() - this.clock)) : 0);
              const nps = data.nps || (ms > 0 ? Math.round((count / ms) * 1000) : (count > 0 ? count * 1000 : 0));
              const isGpu = (data.depth || 0) > 8;
              const tag = isGpu ? '⚡ GPU Metal Batch Evaluator' : '💻 CPU SIMD Engine';
              logger.log('debug', 'socket', `[${tag}] Streaming PV Info: Depth=${data.depth}, Score=${data.score}cp, Nodes=${data.nodes}, NPS=${nps}`, data);
              logger.updateMetrics({ depth: data.depth, score: data.score, nodes: count, nps, time: ms, hardware: isGpu ? 'GPU' : 'CPU', mode: tag, rawTelemetry: data });
              this.emit('info', data);
            } else if (data.type === 'bestmove') {
              const best = data.best || data.bestmove;
              const count = data.nodes || 0;
              const ms = data.time || (this.clock ? Math.max(1, Math.round(performance.now() - this.clock)) : 0);
              const nps = data.nps || (ms > 0 ? Math.round((count / ms) * 1000) : (count > 0 ? count * 1000 : 0));
              const d = data.depth || 6;
              const isGpu = d > 8;
              const tag = isGpu ? '⚡ GPU Metal Batch Evaluator (512MB VRAM)' : '💻 CPU SIMD Engine (8-Cores)';
              logger.log('telemetry', 'socket', `[${tag}] WebSocket Search BestMove: ${best}, Score=${data.score}cp, Nodes=${count}, Time=${ms}ms, NPS=${nps}`, data);
              logger.updateMetrics({ status: 'ready', best, score: data.score, nodes: count, nps, time: ms, depth: d, hardware: isGpu ? 'GPU' : 'CPU', mode: tag, rawTelemetry: data });
              this.emit('search', data);
            } else {
              this.emit(data.type || 'message', data);
            }
          } catch (err) {
            logger.log('error', 'socket', `WebSocket Frame Parse Error: ${err.message}`, err);
            this.emit('error', String(err));
          }
        };

        this.socket.onerror = (err) => {
          this.status = 'error';
          logger.log('error', 'socket', `WebSocket Connection Error on ${this.url}`, err);
          logger.updateMetrics({ status: 'error', errors: logger.metrics.errors + 1 });

          if (!this.attemptedFallback && this.fallbackUrl && this.url !== this.fallbackUrl) {
            this.attemptedFallback = true;
            logger.log('info', 'socket', `Attempting fallback connection to direct port 8888: ${this.fallbackUrl}`);
            this.init(this.fallbackUrl).then(resolve).catch(reject);
            return;
          }

          this.emit('error', err);
          reject(err);
        };

        this.socket.onclose = () => {
          this.status = 'closed';
          logger.log('warn', 'socket', `WebSocket Connection Closed (${this.url})`);
          logger.updateMetrics({ status: 'closed' });
          this.emit('close', null);
        };
      } catch (err) {
        logger.log('error', 'socket', `WebSocket Init Exception: ${err.message}`, err);
        reject(err);
      }
    });
  }

  flush() {
    if (this.socket && this.socket.readyState === WebSocket.OPEN) {
      if (this.queue.length > 0) {
        logger.log('debug', 'socket', `Flushing ${this.queue.length} queued messages to WebSocket Backend`);
      }
      while (this.queue.length > 0) {
        const msg = this.queue.shift();
        this.socket.send(JSON.stringify(msg));
      }
    }
  }

  position(fen) {
    this.fen = fen;
    logger.log('debug', 'socket', `Updated Socket position FEN: ${fen}`);
  }

  send(msg) {
    if (this.socket && this.socket.readyState === WebSocket.OPEN) {
      this.socket.send(JSON.stringify(msg));
    } else {
      logger.log('debug', 'socket', `WebSocket not open (status: '${this.status}'), queueing action '${msg.action}'`);
      this.queue.push(msg);
    }
  }

  // Gửi thông điệp cài đặt dung lượng Hash RAM tới WebSocket Server
  hash(mb) {
    logger.log('info', 'socket', `Sending WebSocket hash command: ${mb}MB`);
    const action = ['set', 'hash'].join('_');
    const msg = {
      action,
      mb: mb || 256
    };
    this.send(msg);
  }

  search(depth, time, history) {
    this.clock = performance.now();
    const d = depth || 6;
    const isGpu = d > 8;
    const tag = isGpu ? '⚡ GPU Metal Batch Evaluator (Depth > 8)' : '💻 CPU SIMD Engine (Depth <= 8)';
    logger.log('info', 'socket', `Triggering WebSocket Search [${tag}]: Depth=${d}, TimeLimit=${time}ms`);
    logger.updateMetrics({ status: 'searching', depth: d, hardware: isGpu ? 'GPU' : 'CPU', mode: tag });
    const msg = {
      action: 'search',
      fen: this.fen,
      depth: depth || 6,
      time: time || 3000
    };
    if (history !== undefined) {
      msg.history = history;
    }
    this.send(msg);
  }

  eval() {
    const msg = {
      action: 'eval',
      fen: this.fen
    };
    this.send(msg);
  }

  perft(depth) {
    const msg = {
      action: 'perft',
      fen: this.fen,
      depth: depth || 1
    };
    this.send(msg);
  }

  stop() {
    if (this.socket && this.socket.readyState === WebSocket.OPEN) {
      logger.log('info', 'socket', 'Stopping WebSocket search');
      this.socket.send(JSON.stringify({ action: 'stop' }));
    }
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
