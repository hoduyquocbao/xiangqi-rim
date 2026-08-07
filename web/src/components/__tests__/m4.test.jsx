// Unit tests for Milestone 4 Components (Eval, Explorer, Modal, Panel)
// 100% Single-word English Identifiers

import React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import Eval from '../Eval.jsx';
import Explorer from '../Explorer.jsx';
import Modal from '../Modal.jsx';
import Panel from '../Panel.jsx';

describe('Milestone 4 Component Suite', () => {
  describe('Eval Component', () => {
    it('renders centipawn evaluation score and Win Rate', () => {
      render(<Eval score={150} />);
      expect(screen.getByText('EVALUATION')).toBeTruthy();
      expect(screen.getByText('+1.50')).toBeTruthy();
    });

    it('handles negative score for black advantage', () => {
      render(<Eval score={-200} />);
      expect(screen.getByText('-2.00')).toBeTruthy();
    });
  });

  describe('Explorer Component', () => {
    it('renders empty variation state when line is empty', () => {
      render(<Explorer line={[]} />);
      expect(screen.getByText(/No PV variation stream available/i)).toBeTruthy();
    });

    it('renders interactive PV move chips', () => {
      const pick = vi.fn();
      render(<Explorer line={['C2.5', 'H8+7']} pick={pick} active={0} />);

      expect(screen.getByText('C2.5')).toBeTruthy();
      expect(screen.getByText('H8+7')).toBeTruthy();

      fireEvent.click(screen.getByText('H8+7'));
      expect(pick).toHaveBeenCalledWith(1, 'H8+7');
    });
  });

  describe('Panel Component', () => {
    it('renders engine controls and depth slider', () => {
      const level = vi.fn();
      const search = vi.fn();

      render(<Panel depth={8} level={level} search={search} status="ready" />);

      expect(screen.getByText('ENGINE CONTROLS')).toBeTruthy();
      expect(screen.getByText('DEPTH 8')).toBeTruthy();
      expect(screen.getByText('START AI CALCULATE')).toBeTruthy();

      fireEvent.click(screen.getByText('START AI CALCULATE'));
      expect(search).toHaveBeenCalled();
    });
  });

  describe('Modal Component', () => {
    it('does not render when show is false', () => {
      const { container } = render(<Modal show={false} />);
      expect(container.firstChild).toBeNull();
    });

    it('renders FEN and PGN tabs when show is true', () => {
      render(<Modal show={true} fen="rnbakabnr/9/9/9/9/9/9/9/9/RNBAKABNR w - - 0 1" history={[]} />);

      expect(screen.getByText('FEN EDITOR')).toBeTruthy();
      expect(screen.getByText('PGN GAME RECORD')).toBeTruthy();
    });
  });
});
