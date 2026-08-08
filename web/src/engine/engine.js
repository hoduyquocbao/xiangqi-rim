// web/src/engine/engine.js
// Unified Dual-Engine Facade State Manager for XiangRust
// 100% Single-Word English Identifiers

import { Driver as Wasm } from './wasm.js';
import { Driver as Socket } from './socket.js';
import { Driver as Llm } from './llm.js';

export class Engine {
  constructor() {
    this.active = 'wasm';
    this.wasm = new Wasm();
    this.socket = new Socket();
    this.llm = new Llm();
    this.subscribers = new Set();
    this.fen = 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1';

    this.wasm.listen((type, data) => this.pipe('wasm', type, data));
    this.socket.listen((type, data) => this.pipe('socket', type, data));
    this.llm.listen((type, data) => this.pipe('llm', type, data));
  }

  async init(mode = 'wasm', config = {}) {
    this.active = mode;
    if (mode === 'wasm') {
      await this.wasm.init(config.path);
    } else if (mode === 'socket') {
      await this.socket.init(config.url);
    } else if (mode === 'llm') {
      await this.llm.init(config.url);
    } else if (mode === 'hybrid') {
      await this.wasm.init(config.path);
      await this.socket.init(config.url);
    }
  }

  mode(type) {
    if (type && type !== this.active) {
      this.active = type;
      this.emit('mode', this.active);
    }
    return this.active;
  }

  driver() {
    if (this.active === 'socket') return this.socket;
    if (this.active === 'llm') return this.llm;
    return this.wasm;
  }

  position(fen) {
    this.fen = fen;
    this.wasm.position(fen);
    this.socket.position(fen);
    this.llm.position(fen);
  }

  // Cấu hình dung lượng Hash RAM Transposition Table (MB)
  hash(mb) {
    this.wasm.hash(mb);
    this.socket.hash(mb);
  }

  search(depth, time, history) {
    if (this.active === 'hybrid') {
      // Hybrid Mode: Tính toán tức thì trên WASM local và đồng thời đẩy tính toán GPU sâu về BFF Server
      this.wasm.search(depth, time, history);
      this.socket.search((depth || 6) + 2, time || 2000, history);
    } else {
      if (history !== undefined) {
        this.driver().search(depth, time, history);
      } else {
        this.driver().search(depth, time);
      }
    }
  }

  eval() {
    this.driver().eval();
  }

  perft(depth) {
    this.driver().perft(depth);
  }

  stop() {
    this.wasm.stop();
    this.socket.stop();
  }

  pipe(source, type, data) {
    if (source === this.active || this.active === 'hybrid') {
      this.emit(type, { ...data, source });
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

export const instance = new Engine();
