// web/src/engine/logger.js
// Royal Imperial Telemetry, Metrics & Debugger Event Bus for XiangRust Engine
// 100% Single-Word English Identifiers

class Bus {
  constructor() {
    this.logs = [];
    this.limit = 500;
    this.subscribers = new Set();
    this.metrics = {
      status: 'idle',
      engine: 'wasm',
      hardware: 'CPU',
      mode: 'CPU SIMD Engine (Depth <= 8)',
      depth: 0,
      nodes: 0,
      nps: 0,
      time: 0,
      score: 0,
      best: null,
      pv: [],
      memory: '0 KB',
      errors: 0
    };
  }

  log(level, category, message, details = null) {
    const entry = {
      id: Date.now() + Math.random(),
      stamp: new Date().toLocaleTimeString(),
      level,     // 'info' | 'warn' | 'error' | 'telemetry' | 'debug'
      category,  // 'wasm' | 'socket' | 'search' | 'board' | 'system'
      message,
      details
    };

    this.logs.push(entry);
    if (this.logs.length > this.limit) {
      this.logs.shift();
    }

    // Styled Console output in Browser DevTools
    const style = {
      info: 'color: #D4AF37; font-weight: bold;',
      warn: 'color: #FFA500; font-weight: bold;',
      error: 'color: #FF4D4D; font-weight: bold; background: #2A0000; padding: 2px 4px; border-radius: 2px;',
      telemetry: 'color: #00E676; font-weight: bold;',
      debug: 'color: #64B5F6; font-style: italic;'
    }[level] || 'color: #D4AF37;';

    console.log(`%c[${entry.stamp}] [${category.toUpperCase()}] ${message}`, style, details || '');

    this.notify('log', entry);
  }

  updateMetrics(patch) {
    this.metrics = { ...this.metrics, ...patch };
    this.notify('metrics', this.metrics);
  }

  listen(fn) {
    this.subscribers.add(fn);
    return () => this.subscribers.delete(fn);
  }

  notify(event, data) {
    for (const fn of this.subscribers) {
      try {
        fn(event, data);
      } catch (err) {
        console.error('Logger subscriber error:', err);
      }
    }
  }

  clear() {
    this.logs = [];
    this.notify('clear', null);
  }
}

export const logger = new Bus();
