import React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/react';
import Board from '../Board.jsx';

const startFen = 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1';
const nextFen = 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/1NBAKABNR b - - 0 1'; // Red chariot at 0 moved

describe('Board Component Adversarial Defect Suite', () => {
  it('BUG 1 PROOF: Transparent <rect> overlay sitting on top of <g> piece blocks Drag & Drop from starting', () => {
    const moveMock = vi.fn();
    const { container } = render(<Board fen={startFen} move={moveMock} />);

    // In SVG DOM, pointer events at piece coordinates (50, 950) hit the top-most element (<rect key="tile-0">)
    const tiles = container.querySelectorAll('rect');
    const tile0 = tiles[0];

    // Verify <rect> does NOT have draggable="true"
    expect(tile0.getAttribute('draggable')).toBeNull();

    // When user attempts to drag from tile 0 in browser, dragstart fires on tile0 (<rect>):
    const dataTransfer = { setData: vi.fn() };
    fireEvent.dragStart(tile0, { dataTransfer });

    // Since <rect> has no onDragStart handler, dataTransfer.setData is NOT called!
    expect(dataTransfer.setData).not.toHaveBeenCalled();

    // Now try to drop on tile 9:
    fireEvent.dragOver(tiles[9]);
    fireEvent.drop(tiles[9]);

    // moveMock is NOT called because state.drag was never set!
    expect(moveMock).not.toHaveBeenCalled();
  });

  it('BUG 2 REGRESSION TEST: Canceled or off-board drag correctly clears valid move dots and selection state', () => {
    const moveMock = vi.fn();
    const { container } = render(<Board fen={startFen} move={moveMock} />);

    const pieces = container.querySelectorAll('svg > g[filter]');
    const redChariotG = pieces[0];

    // User starts dragging piece 0
    const dataTransfer = { setData: vi.fn() };
    fireEvent.dragStart(redChariotG, { dataTransfer });

    // Valid move dots are displayed
    expect(container.querySelectorAll('g.pointer-events-none circle').length).toBeGreaterThan(0);

    // User releases mouse outside board (dragEnd fired on piece, no drop event)
    fireEvent.dragEnd(redChariotG);

    // FIX VERIFIED: valid move dots are cleared because onDragEnd handler calls clear()
    const remainingDots = container.querySelectorAll('g.pointer-events-none circle');
    expect(remainingDots.length).toBe(0); // Asserts selection state cleared after dragEnd!
  });

  it('BUG 3 REGRESSION TEST: Updating FEN prop while piece is selected clears stale selection and move dots', () => {
    const moveMock = vi.fn();
    const { container, rerender } = render(<Board fen={startFen} move={moveMock} />);
    const tiles = container.querySelectorAll('rect');

    // Click tile 0 to select Red Chariot
    fireEvent.click(tiles[0]);
    expect(container.querySelectorAll('g.pointer-events-none circle').length).toBeGreaterThan(0);

    // External FEN update occurs (e.g. opponent move or reset)
    rerender(<Board fen={nextFen} move={moveMock} />);

    // FIX VERIFIED: state.select and valid dots are cleared on FEN update
    const dotsAfterFenUpdate = container.querySelectorAll('g.pointer-events-none circle');
    expect(dotsAfterFenUpdate.length).toBe(0); // Stale state correctly cleared!
  });
});
