// Unit tests for Milestone 5 Bot Arena Component
// 100% Single-word English Identifiers

import React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, act } from '@testing-library/react';
import Arena from '../Arena.jsx';
import { instance as engine } from '../../engine/engine.js';

const start = 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1';

describe('Milestone 5 Bot Arena Suite', () => {
  it('renders arena controls, depth sliders, live metrics, and scoreboard', () => {
    render(
      <Arena
        fen={start}
        turn="w"
        move={vi.fn()}
        reset={vi.fn()}
        board={new Array(90).fill('.')}
        check={false}
      />
    );

    expect(screen.getByText(/BOT ARENA \(SELF-PLAY\)/i)).toBeTruthy();
    expect(screen.getByText(/PAUSED/i)).toBeTruthy();
    expect(screen.getAllByText(/START SELF-PLAY/i).length).toBeGreaterThan(0);
    expect(screen.getByText(/STEP \(1 MOVE\)/i)).toBeTruthy();
    expect(screen.getByText(/RESET ARENA/i)).toBeTruthy();
    expect(screen.getByText(/RED BOT/i)).toBeTruthy();
    expect(screen.getByText(/BLACK BOT/i)).toBeTruthy();
    expect(screen.getByText(/SPEED \(NPS\)/i)).toBeTruthy();
    expect(screen.getByText(/SEARCH NODES/i)).toBeTruthy();
    expect(screen.getByText(/LIVE ARENA MOVE FEED/i)).toBeTruthy();
  });

  it('toggles start and pause self-play states on button click', () => {
    render(
      <Arena
        fen={start}
        turn="w"
        move={vi.fn()}
        reset={vi.fn()}
        board={new Array(90).fill('.')}
      />
    );

    const btn = screen.getAllByText(/START SELF-PLAY/i)[0];
    fireEvent.click(btn);

    expect(screen.getByText(/AUTO ARENA RUNNING/i)).toBeTruthy();
    expect(screen.getByText(/PAUSE ARENA/i)).toBeTruthy();

    const stop = screen.getByText(/PAUSE ARENA/i);
    fireEvent.click(stop);

    expect(screen.getByText(/PAUSED/i)).toBeTruthy();
  });

  it('triggers step move calculation when step button is clicked', () => {
    const search = vi.spyOn(engine, 'search').mockImplementation(() => {});
    render(
      <Arena
        fen={start}
        turn="w"
        move={vi.fn()}
        reset={vi.fn()}
        board={new Array(90).fill('.')}
      />
    );

    const btn = screen.getByText(/STEP \(1 MOVE\)/i);
    fireEvent.click(btn);

    expect(search).toHaveBeenCalled();
    search.mockRestore();
  });

  it('triggers reset match callback when reset button is clicked', () => {
    const reset = vi.fn();
    render(
      <Arena
        fen={start}
        turn="w"
        move={vi.fn()}
        reset={reset}
        board={new Array(90).fill('.')}
      />
    );

    const btn = screen.getByText(/RESET ARENA/i);
    fireEvent.click(btn);

    expect(reset).toHaveBeenCalled();
  });

  // Kiểm thử cập nhật giá trị độ sâu bot khi người dùng thay đổi thanh trượt slider
  it('updates bot depth values on depth slider input changes', () => {
    render(
      <Arena
        fen={start}
        turn="w"
        move={vi.fn()}
        reset={vi.fn()}
        board={new Array(90).fill('.')}
      />
    );

    const sliders = screen.getAllByRole('slider');
    // Thanh trượt độ sâu Bot Đỏ (vị trí 0)
    const red = sliders[0];
    // Bọc thao tác thay đổi giá trị trong act để đồng bộ hóa React state
    act(() => {
      fireEvent.change(red, { target: { value: '10' } });
    });

    expect(red.value).toBe('10');
  }, 15000);

  it('displays critical blunder warning indicator when centipawn drop occurs', async () => {
    render(
      <Arena
        fen={start}
        turn="w"
        move={vi.fn()}
        reset={vi.fn()}
        board={new Array(90).fill('.')}
      />
    );

    // Emit search result with score drop > 150 wrapped in act
    act(() => {
      engine.emit('search', {
        bestmove: 'C2.5',
        score: -200,
        nodes: 5000,
        nps: 100000
      });
    });

    const alert = await screen.findByText(/BLUNDER!/i);
    expect(alert).toBeTruthy();
  });
});
