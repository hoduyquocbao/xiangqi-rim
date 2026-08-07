// Adversarial & Edge-Case Component Tests for Milestone 4
// 100% Single-word English Identifiers

import React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import Eval from '../Eval.jsx';
import Explorer from '../Explorer.jsx';
import Modal from '../Modal.jsx';
import Panel from '../Panel.jsx';

describe('Milestone 4 Adversarial Component Testing', () => {
  describe('Eval Boundary Cases', () => {
    it('handles zero score (50% balance point)', () => {
      render(<Eval score={0} />);
      expect(screen.getByText('0.00')).toBeTruthy();
      expect(screen.getAllByText('50%').length).toBe(2);
    });

    it('caps extreme scores within 2% to 98% range', () => {
      render(<Eval score={10000} />);
      expect(screen.getByText('98%')).toBeTruthy();
      expect(screen.getByText('+100.00')).toBeTruthy();

      render(<Eval score={-10000} />);
      expect(screen.getAllByText('98%').length).toBeGreaterThan(0);
      expect(screen.getByText('-100.00')).toBeTruthy();
    });
  });

  describe('Explorer Edge Cases', () => {
    it('handles long PV line with 30 moves without error', () => {
      const line = Array.from({ length: 30 }, (_, i) => `M${i + 1}`);
      const pick = vi.fn();
      render(<Explorer line={line} pick={pick} active={5} />);

      expect(screen.getByText('30 MOVES')).toBeTruthy();
      expect(screen.getByText('M1')).toBeTruthy();
      expect(screen.getByText('M30')).toBeTruthy();

      fireEvent.click(screen.getByText('M10'));
      expect(pick).toHaveBeenCalledWith(9, 'M10');
    });

    it('handles missing pick callback gracefully', () => {
      render(<Explorer line={['C2.5']} />);
      expect(() => fireEvent.click(screen.getByText('C2.5'))).not.toThrow();
    });
  });

  describe('Panel State Transitions', () => {
    it('renders searching state with Stop button and disables controls', () => {
      const stop = vi.fn();
      render(<Panel status="searching" stop={stop} undoable={true} redoable={true} />);

      expect(screen.getByText('STOP AI SEARCH')).toBeTruthy();
      expect(screen.getByText('BEST MOVE HINT').disabled).toBe(true);
      expect(screen.getByText('UNDO').disabled).toBe(true);
      expect(screen.getByText('REDO').disabled).toBe(true);

      fireEvent.click(screen.getByText('STOP AI SEARCH'));
      expect(stop).toHaveBeenCalled();
    });
  });

  describe('Modal Interactive Controls', () => {
    it('switches between FEN and PGN tabs and triggers load callback', () => {
      const apply = vi.fn();
      const close = vi.fn();

      render(
        <Modal
          show={true}
          fen="rnbakabnr/9/9/9/9/9/9/9/9/RNBAKABNR w - - 0 1"
          history={['C2.5', 'H8+7']}
          apply={apply}
          close={close}
        />
      );

      // Verify default FEN tab content
      expect(screen.getByText(/FEN Position String/i)).toBeTruthy();

      // Switch to PGN tab
      fireEvent.click(screen.getByText('PGN GAME RECORD'));
      expect(screen.getByText(/PGN Match Record Notation/i)).toBeTruthy();

      // Apply & Load
      fireEvent.click(screen.getByText('APPLY & LOAD'));
      expect(apply).toHaveBeenCalled();
      expect(close).toHaveBeenCalled();
    });
  });
});
