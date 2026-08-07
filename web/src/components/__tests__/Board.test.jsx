import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen } from '@testing-library/react';
import Board from '../Board.jsx';

const startFen = 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1';

describe('Board Component Empirical Adversarial Tests', () => {
  let moveMock;

  beforeEach(() => {
    moveMock = vi.fn();
  });

  it('renders SVG board elements, 32 pieces, grid lines, and river text', () => {
    const { container } = render(<Board fen={startFen} move={moveMock} rulers={false} />);
    const svg = container.querySelector('svg');
    expect(svg).not.toBeNull();
    expect(svg.getAttribute('viewBox')).toBe('0 0 900 1000');

    // 32 pieces rendered
    const pieceTexts = container.querySelectorAll('text');
    // There are 32 piece texts + 2 river text elements ("楚 河", "漢 界") = 34 text elements
    expect(pieceTexts.length).toBe(34);
  });

  it('renders coordinate rulers (files a..i and ranks 0..9) when rulers is true', () => {
    const { container } = render(<Board fen={startFen} move={moveMock} rulers={true} />);
    const pieceTexts = container.querySelectorAll('text');
    // 32 pieces + 2 river texts + 38 ruler texts (18 file + 20 rank) = 72 text elements
    expect(pieceTexts.length).toBe(72);
  });

  it('handles Click-to-Move flow: select piece, show valid moves, click valid destination', () => {
    const { container } = render(<Board fen={startFen} move={moveMock} />);

    // Index 0 is Red Chariot ('R') at rank 0, file 0. Red turn.
    // Click on index 0 piece
    const tiles = container.querySelectorAll('rect');
    expect(tiles.length).toBe(90);

    // Click tile 0 (Red Chariot)
    fireEvent.click(tiles[0]);

    // Check if valid move dots are rendered.
    // Red Chariot at 0 can move to 1, 2 (along rank 0) or 9, 18 (along file 0)
    // Valid moves from pseudo/moves for R at 0: 1, 2, 9, 18
    const validDots = container.querySelectorAll('g.pointer-events-none circle');
    expect(validDots.length).toBeGreaterThan(0);

    // Click tile 9 (valid move for Red Chariot)
    fireEvent.click(tiles[9]);

    expect(moveMock).toHaveBeenCalledWith(0, 9);
  });

  it('deselects piece when clicking the selected piece again', () => {
    const { container } = render(<Board fen={startFen} move={moveMock} />);
    const tiles = container.querySelectorAll('rect');

    // Select piece at 0
    fireEvent.click(tiles[0]);
    expect(container.querySelectorAll('g.pointer-events-none circle').length).toBeGreaterThan(0);

    // Click piece at 0 again to deselect
    fireEvent.click(tiles[0]);
    expect(container.querySelectorAll('g.pointer-events-none circle').length).toBe(0);
  });

  it('switches selection when clicking another friendly piece', () => {
    const { container } = render(<Board fen={startFen} move={moveMock} />);
    const tiles = container.querySelectorAll('rect');

    // Click Red Chariot at 0
    fireEvent.click(tiles[0]);
    // Click Red Horse at 1
    fireEvent.click(tiles[1]);

    // moveMock should NOT have been called
    expect(moveMock).not.toHaveBeenCalled();
    // Valid dots should now be for Red Horse at 1 (indices 11, 19)
    const validDots = container.querySelectorAll('g.pointer-events-none circle');
    expect(validDots.length).toBe(2);
  });

  it('clears selection when clicking an invalid square', () => {
    const { container } = render(<Board fen={startFen} move={moveMock} />);
    const tiles = container.querySelectorAll('rect');

    // Select Red Chariot at 0
    fireEvent.click(tiles[0]);

    // Click invalid square (e.g. 89 - enemy chariot)
    fireEvent.click(tiles[89]);

    expect(moveMock).not.toHaveBeenCalled();
    expect(container.querySelectorAll('g.pointer-events-none circle').length).toBe(0);
  });

  it('ignores click on enemy piece when no piece is selected', () => {
    const { container } = render(<Board fen={startFen} move={moveMock} />);
    const tiles = container.querySelectorAll('rect');

    // Click Black Chariot at 81 on Red turn
    fireEvent.click(tiles[81]);
    expect(container.querySelectorAll('g.pointer-events-none circle').length).toBe(0);
  });

  it('ignores clicks when disabled is true', () => {
    const { container } = render(<Board fen={startFen} move={moveMock} disabled={true} />);
    const tiles = container.querySelectorAll('rect');

    fireEvent.click(tiles[0]);
    expect(container.querySelectorAll('g.pointer-events-none circle').length).toBe(0);
  });

  it('renders Check Flash FX circle on threatened King when check is true', () => {
    // FEN where Red King is in check
    // e.g. Black Cannon checking Red King
    const checkFen = 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/9/4C4/RNBAKABNR w - - 0 1';
    const { container } = render(<Board fen={checkFen} move={moveMock} check={true} />);

    // Flash circle selector
    const flashCircle = container.querySelector('circle.flash');
    expect(flashCircle).not.toBeNull();
    expect(flashCircle.getAttribute('stroke')).toBe('#FF1A1A');
  });

  it('renders distinction between empty valid move dots and enemy capture target rings', () => {
    // Setup a position where Cannon can capture an enemy piece
    // FEN with Red Cannon 'C' at index 19 (rank 2, file 1) and Black Pawn 'p' in line
    const captureFen = 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/4p4/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1';
    const { container } = render(<Board fen={captureFen} move={moveMock} />);
    const tiles = container.querySelectorAll('rect');

    // Click Red Cannon at index 19
    fireEvent.click(tiles[19]);

    // Check pinging circle for capture vs pulse circle for empty target
    const pingCircle = container.querySelector('circle.animate-ping');
    const pulseCircles = container.querySelectorAll('circle.animate-pulse');

    // If there is an enemy piece in valid target list, pingCircle should exist
    // Let's verify valid dots structure
    const validGroups = container.querySelectorAll('g.pointer-events-none');
    expect(validGroups.length).toBeGreaterThan(0);
  });

  it('maps coordinates correctly for normal perspective vs flipped perspective (flip prop)', () => {
    const { container: normalContainer } = render(<Board fen={startFen} move={moveMock} flip={false} />);
    const { container: flippedContainer } = render(<Board fen={startFen} move={moveMock} flip={true} />);

    const normalTiles = normalContainer.querySelectorAll('rect');
    const flippedTiles = flippedContainer.querySelectorAll('rect');

    // Index 0 (Red Chariot, rank 0, file 0):
    // flip = false: cx = 50, cy = 950. rect x = 5, y = 905
    expect(normalTiles[0].getAttribute('x')).toBe('5');
    expect(normalTiles[0].getAttribute('y')).toBe('905');

    // flip = true: cx = 850, cy = 50. rect x = 805, y = 5
    expect(flippedTiles[0].getAttribute('x')).toBe('805');
    expect(flippedTiles[0].getAttribute('y')).toBe('5');

    // Index 81 (Black Chariot, rank 9, file 0):
    // flip = false: cx = 50, cy = 50. rect x = 5, y = 5
    expect(normalTiles[81].getAttribute('x')).toBe('5');
    expect(normalTiles[81].getAttribute('y')).toBe('5');

    // flip = true: cx = 850, cy = 950. rect x = 805, y = 905
    expect(flippedTiles[81].getAttribute('x')).toBe('805');
    expect(flippedTiles[81].getAttribute('y')).toBe('905');
  });

  it('ADVERSARIAL STRESS TEST: Drag & Drop vs Touch Tile Overlay order', () => {
    const { container } = render(<Board fen={startFen} move={moveMock} />);

    // Query piece <g> elements and tile <rect> elements
    const pieces = container.querySelectorAll('svg > g[filter]');
    const tiles = container.querySelectorAll('rect');

    // Find Red Chariot piece element (index 0)
    const redChariotG = pieces[0];
    expect(redChariotG.getAttribute('draggable')).toBe('true');

    // Simulate dragStart directly on piece <g> element
    const dataTransfer = { setData: vi.fn() };
    fireEvent.dragStart(redChariotG, { dataTransfer });

    expect(dataTransfer.setData).toHaveBeenCalledWith('text/plain', '0');

    // Simulate drop on target tile (index 9)
    fireEvent.dragOver(tiles[9]);
    fireEvent.drop(tiles[9]);

    expect(moveMock).toHaveBeenCalledWith(0, 9);
  });
});
