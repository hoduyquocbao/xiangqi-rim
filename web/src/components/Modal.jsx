// Component Modal Biên Tập & Giải Mã FEN/PGN Hoàng Gia
// Định danh đơn từ tiếng Anh: Modal, show, close, apply, mode, text, fen, pgn, copy, load, save, open, tab, edit, copied, parsed, e

import React, { useState, useEffect } from 'react';
import { parse, stringify } from '../rules/pgn.js';

export default function Modal({ show, close, fen, history, apply }) {
  const [tab, edit] = useState('fen');
  const [text, val] = useState('');
  const [copied, copy] = useState(false);

  useEffect(() => {
    if (tab === 'fen') {
      val(fen || '');
    } else if (tab === 'pgn') {
      val(stringify(history || []));
    }
  }, [tab, fen, history, show]);

  if (!show) return null;

  // Xử lý nạp FEN hoặc PGN mới vào ứng dụng
  const load = () => {
    if (tab === 'fen') {
      apply && apply('fen', text);
    } else if (tab === 'pgn') {
      const parsed = parse(text);
      apply && apply('pgn', parsed);
    }
    close && close();
  };

  // Sao chép nội dung vào Clipboard
  const save = () => {
    navigator.clipboard.writeText(text);
    copy(true);
    setTimeout(() => copy(false), 2000);
  };

  return (
    <div className="fixed inset-0 z-50 bg-obsidian/80 backdrop-blur-md flex items-center justify-center p-4">
      <div className="bg-obsidian-card border-2 border-gold/40 rounded-2xl max-w-xl w-full p-6 shadow-glow flex flex-col gap-4">
        {/* Tiêu đề Modal & Chuyển Tab */}
        <div className="flex items-center justify-between border-b border-gold/20 pb-3">
          <div className="flex items-center gap-2">
            <button
              onClick={() => edit('fen')}
              className={`px-4 py-1.5 rounded text-xs font-royal font-bold transition-all ${
                tab === 'fen'
                  ? 'bg-gold text-obsidian shadow-glow'
                  : 'text-gold/60 hover:text-gold'
              }`}
            >
              FEN EDITOR
            </button>
            <button
              onClick={() => edit('pgn')}
              className={`px-4 py-1.5 rounded text-xs font-royal font-bold transition-all ${
                tab === 'pgn'
                  ? 'bg-gold text-obsidian shadow-glow'
                  : 'text-gold/60 hover:text-gold'
              }`}
            >
              PGN GAME RECORD
            </button>
          </div>

          <button
            onClick={close}
            className="text-gold/60 hover:text-gold text-lg font-bold px-2"
          >
            ✕
          </button>
        </div>

        {/* Thân Modal: Ô nhập nội dung FEN / PGN */}
        <div className="flex flex-col gap-2">
          <label className="text-xs text-gold/70 font-semibold uppercase">
            {tab === 'fen' ? 'FEN Position String' : 'PGN Match Record Notation'}
          </label>
          <textarea
            value={text}
            onChange={(e) => val(e.target.value)}
            rows={tab === 'fen' ? 3 : 8}
            className="w-full bg-obsidian border border-gold/30 rounded-lg p-3 text-xs font-mono text-gold focus:outline-none focus:border-gold shadow-inner resize-none"
            placeholder={tab === 'fen' ? 'Enter FEN string...' : 'Enter PGN text...'}
          />
        </div>

        {/* Chân Modal: Các nút thao tác Copy, Apply, Cancel */}
        <div className="flex items-center justify-between pt-2 border-t border-gold/20">
          <button
            onClick={save}
            className="px-4 py-2 rounded border border-gold/40 text-xs font-bold text-gold hover:bg-gold/10 transition"
          >
            {copied ? 'COPIED!' : 'COPY TO CLIPBOARD'}
          </button>

          <div className="flex items-center gap-2">
            <button
              onClick={close}
              className="px-4 py-2 rounded text-xs font-bold text-gold/60 hover:text-gold transition"
            >
              CANCEL
            </button>
            <button
              onClick={load}
              className="px-5 py-2 rounded bg-gold text-obsidian font-bold text-xs shadow-glow hover:bg-gold/90 transition"
            >
              APPLY & LOAD
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
