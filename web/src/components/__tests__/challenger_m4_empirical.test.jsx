// Empiric Adversarial Stress Test Suite for Milestone 4 Components
// Single-word English Identifiers strictly followed

import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, act } from '@testing-library/react';
import Eval from '../Eval.jsx';
import Explorer from '../Explorer.jsx';
import Modal from '../Modal.jsx';
import Panel from '../Panel.jsx';

describe('Empiric Milestone 4 Stress Test Suite', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  describe('Eval Component Stress & Edge Testing', () => {
    it('handles zero score neutral position', () => {
      render(<Eval score={0} />);
      expect(screen.getByText('EVALUATION')).toBeTruthy();
      expect(screen.getByText('0.00')).toBeTruthy();
      expect(screen.getAllByText('50%')).toHaveLength(2);
    });

    it('handles extreme positive score clamping to maximum bounds', () => {
      render(<Eval score={10000} />);
      expect(screen.getByText('+100.00')).toBeTruthy();
      expect(screen.getByText('98%')).toBeTruthy();
    });

    it('handles extreme negative score clamping to minimum bounds', () => {
      render(<Eval score={-10000} />);
      expect(screen.getByText('-100.00')).toBeTruthy();
      expect(screen.getByText('98%')).toBeTruthy(); // Vermilion side shows 100 - 2 = 98%
    });

    it('handles default prop fallback gracefully', () => {
      render(<Eval />);
      expect(screen.getByText('0.00')).toBeTruthy();
    });
  });

  describe('Explorer Component Interaction & Stress Testing', () => {
    it('handles empty line array input', () => {
      render(<Explorer line={[]} />);
      expect(screen.getByText(/0 MOVES/i)).toBeTruthy();
      expect(screen.getByText(/No PV variation stream available/i)).toBeTruthy();
    });

    it('renders large PV stream of moves and triggers selection callback', () => {
      const pick = vi.fn();
      const line = Array.from({ length: 20 }, (_, i) => `Move-${i + 1}`);
      render(<Explorer line={line} pick={pick} active={5} />);

      expect(screen.getByText('20 MOVES')).toBeTruthy();
      expect(screen.getByText('Move-6')).toBeTruthy();

      fireEvent.click(screen.getByText('Move-6'));
      expect(pick).toHaveBeenCalledWith(5, 'Move-6');
    });

    it('handles out of bounds active selection safely', () => {
      const line = ['C2.5', 'H8+7'];
      const { container } = render(<Explorer line={line} active={99} />);
      expect(container).toBeTruthy();
    });
  });

  describe('Panel Component Control Logic & State Testing', () => {
    // Kiểm thử hiển thị giá trị mặc định và phản hồi khi người dùng thay đổi mức độ sâu trên slider
    it('renders default values and responds to slider level changes', () => {
      const level = vi.fn();
      render(<Panel depth={6} level={level} status="ready" />);

      const sliders = screen.getAllByRole('slider');
      const slider = sliders[0];
      expect(slider).toBeTruthy();
      expect(slider.value).toBe('6');

      // Bọc thao tác kích hoạt sự kiện thay đổi trong act để đồng bộ hóa React state
      act(() => {
        fireEvent.change(slider, { target: { value: '10' } });
      });
      expect(level).toHaveBeenCalledWith(10);
    }, 15000);

    it('toggles search button state when status changes to searching', () => {
      const stop = vi.fn();
      render(<Panel status="searching" stop={stop} />);

      const button = screen.getByText('STOP AI SEARCH');
      expect(button).toBeTruthy();

      fireEvent.click(button);
      expect(stop).toHaveBeenCalled();
    });

    it('triggers hint, undo, redo, flip, open callbacks', () => {
      const hint = vi.fn();
      const undo = vi.fn();
      const redo = vi.fn();
      const flip = vi.fn();
      const open = vi.fn();

      render(
        <Panel
          hint={hint}
          undo={undo}
          redo={redo}
          flip={flip}
          open={open}
          undoable={true}
          redoable={true}
          status="ready"
        />
      );

      fireEvent.click(screen.getByText('BEST MOVE HINT'));
      expect(hint).toHaveBeenCalled();

      fireEvent.click(screen.getByText('UNDO'));
      expect(undo).toHaveBeenCalled();

      fireEvent.click(screen.getByText('REDO'));
      expect(redo).toHaveBeenCalled();

      fireEvent.click(screen.getByText('FLIP'));
      expect(flip).toHaveBeenCalled();

      fireEvent.click(screen.getByText('FEN / PGN PARSER & EDITOR'));
      expect(open).toHaveBeenCalled();
    });

    it('disables undo and redo buttons when undoable/redoable are false', () => {
      render(<Panel undoable={false} redoable={false} status="ready" />);
      const undoBtn = screen.getByText('UNDO');
      const redoBtn = screen.getByText('REDO');
      expect(undoBtn.disabled).toBe(true);
      expect(redoBtn.disabled).toBe(true);
    });
  });

  describe('Modal Component State & Parser Integration Testing', () => {
    it('returns null when show is false', () => {
      const { container } = render(<Modal show={false} />);
      expect(container.firstChild).toBeNull();
    });

    it('switches tabs and calls apply upon submitting FEN string', () => {
      const apply = vi.fn();
      const close = vi.fn();

      render(
        <Modal
          show={true}
          fen="rnbakabnr/9/9/9/9/9/9/9/9/RNBAKABNR w - - 0 1"
          apply={apply}
          close={close}
        />
      );

      const textarea = screen.getByRole('textbox');
      expect(textarea.value).toBe('rnbakabnr/9/9/9/9/9/9/9/9/RNBAKABNR w - - 0 1');

      fireEvent.change(textarea, {
        target: { value: 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1' }
      });

      fireEvent.click(screen.getByText('APPLY & LOAD'));
      expect(apply).toHaveBeenCalledWith(
        'fen',
        'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1'
      );
      expect(close).toHaveBeenCalled();
    });

    it('switches to PGN tab and loads PGN text correctly', () => {
      const apply = vi.fn();
      render(<Modal show={true} history={['C2.5', 'H8+7']} apply={apply} />);

      fireEvent.click(screen.getByText('PGN GAME RECORD'));
      const textarea = screen.getByRole('textbox');
      expect(textarea.value).toContain('1. C2.5 H8+7');

      fireEvent.click(screen.getByText('APPLY & LOAD'));
      expect(apply).toHaveBeenCalledWith('pgn', expect.objectContaining({
        moves: ['C2.5', 'H8+7']
      }));
    });

    it('copies text to clipboard when copy button clicked', async () => {
      const mockWriteText = vi.fn().mockResolvedValue(undefined);
      Object.assign(navigator, {
        clipboard: {
          writeText: mockWriteText
        }
      });

      render(<Modal show={true} fen="rnbakabnr/9/9/9/9/9/9/9/9/RNBAKABNR w - - 0 1" />);

      const copyBtn = screen.getByText('COPY TO CLIPBOARD');
      fireEvent.click(copyBtn);

      expect(mockWriteText).toHaveBeenCalledWith('rnbakabnr/9/9/9/9/9/9/9/9/RNBAKABNR w - - 0 1');
      expect(screen.getByText('COPIED!')).toBeTruthy();
    });
  });
});
