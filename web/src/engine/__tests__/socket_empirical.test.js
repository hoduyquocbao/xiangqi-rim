// web/src/engine/__tests__/socket_empirical.test.js
// Empirical test harness for WebSocket driver (web/src/engine/socket.js)

import { describe, test, expect, beforeEach } from 'vitest';
import { Driver } from '../socket.js';

class MockWebSocket {
  static OPEN = 1;
  static CONNECTING = 0;
  static CLOSED = 3;

  constructor(url) {
    this.url = url;
    this.readyState = MockWebSocket.CONNECTING;
    this.sent = [];
    MockWebSocket.lastInstance = this;

    setTimeout(() => {
      this.readyState = MockWebSocket.OPEN;
      if (this.onopen) this.onopen();
    }, 10);
  }

  send(data) {
    this.sent.push(data);
  }

  close() {
    this.readyState = MockWebSocket.CLOSED;
    if (this.onclose) this.onclose();
  }

  triggerMessage(data) {
    if (this.onmessage) {
      this.onmessage({ data });
    }
  }

  triggerError(err) {
    if (this.onerror) {
      this.onerror(err);
    }
  }
}

globalThis.WebSocket = MockWebSocket;

describe('WebSocket Driver Empirical Tests', () => {
  let driver;

  beforeEach(() => {
    driver = new Driver();
  });

  test('Connects to default ws://127.0.0.1:8888/ws URL', async () => {
    const events = [];
    driver.listen((type, data) => events.push({ type, data }));

    const initPromise = driver.init();
    expect(driver.status).toBe('connecting');
    expect(driver.url).toBe('ws://127.0.0.1:8888/ws');

    await initPromise;
    expect(driver.status).toBe('ready');
    expect(events).toContainEqual({ type: 'ready', data: null });
  });

  test('Connects to custom WebSocket URL', async () => {
    await driver.init('ws://127.0.0.1:9090/ws');
    expect(driver.url).toBe('ws://127.0.0.1:9090/ws');
    expect(driver.status).toBe('ready');
  });

  test('Parses JSON response frame "info"', async () => {
    const received = [];
    driver.listen((type, data) => received.push({ type, data }));
    await driver.init();

    const infoPayload = {
      type: 'info',
      depth: 12,
      score: 350,
      nodes: 125000,
      pv: ['h2e2', 'h8e8', 'i0h0']
    };

    MockWebSocket.lastInstance.triggerMessage(JSON.stringify(infoPayload));

    expect(received).toContainEqual({
      type: 'info',
      data: infoPayload
    });
  });

  test('Parses JSON response frame "bestmove" and maps to "search" event', async () => {
    const received = [];
    driver.listen((type, data) => received.push({ type, data }));
    await driver.init();

    const bestmovePayload = {
      type: 'bestmove',
      bestmove: 'h2e2',
      ponder: 'h8e8'
    };

    MockWebSocket.lastInstance.triggerMessage(JSON.stringify(bestmovePayload));

    // Notice: data.type === 'bestmove' is emitted as 'search' event!
    expect(received).toContainEqual({
      type: 'search',
      data: bestmovePayload
    });
  });

  test('Handles generic/other JSON response frames', async () => {
    const received = [];
    driver.listen((type, data) => received.push({ type, data }));
    await driver.init();

    const customPayload = {
      type: 'perft',
      nodes: 2057
    };

    MockWebSocket.lastInstance.triggerMessage(JSON.stringify(customPayload));

    expect(received).toContainEqual({
      type: 'perft',
      data: customPayload
    });
  });

  test('Handles malformed non-JSON frame by emitting error event', async () => {
    const received = [];
    driver.listen((type, data) => received.push({ type, data }));
    await driver.init();

    MockWebSocket.lastInstance.triggerMessage('INVALID_NON_JSON_DATA');

    expect(received.length).toBe(2); // ready + error
    expect(received[1].type).toBe('error');
    expect(received[1].data).toContain('SyntaxError');
  });

  test('Formats outbound search command JSON payload correctly', async () => {
    await driver.init();
    const fen = 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1';
    driver.position(fen);
    driver.search(8, 4000);

    const sent = MockWebSocket.lastInstance.sent;
    expect(sent.length).toBe(1);
    const parsed = JSON.parse(sent[0]);
    expect(parsed).toEqual({
      action: 'search',
      fen: fen,
      depth: 8,
      time: 4000
    });
  });

  test('Formats outbound eval command JSON payload correctly', async () => {
    await driver.init();
    driver.eval();

    const sent = MockWebSocket.lastInstance.sent;
    expect(sent.length).toBe(1);
    const parsed = JSON.parse(sent[0]);
    expect(parsed).toEqual({
      action: 'eval',
      fen: 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1'
    });
  });

  test('Formats outbound perft command JSON payload correctly', async () => {
    await driver.init();
    driver.perft(3);

    const sent = MockWebSocket.lastInstance.sent;
    expect(sent.length).toBe(1);
    const parsed = JSON.parse(sent[0]);
    expect(parsed).toEqual({
      action: 'perft',
      fen: 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1',
      depth: 3
    });
  });

  test('Formats outbound stop command JSON payload correctly', async () => {
    await driver.init();
    driver.stop();

    const sent = MockWebSocket.lastInstance.sent;
    expect(sent.length).toBe(1);
    const parsed = JSON.parse(sent[0]);
    expect(parsed).toEqual({ action: 'stop' });
  });

  test('Handles socket closure gracefully', async () => {
    const received = [];
    driver.listen((type, data) => received.push({ type, data }));
    await driver.init();

    MockWebSocket.lastInstance.close();
    expect(driver.status).toBe('closed');
    expect(received).toContainEqual({ type: 'close', data: null });
  });
});
