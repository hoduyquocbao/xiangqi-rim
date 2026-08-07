// Component Trích Xuất Cây Nước Đi Biến Thể PV Line Explorer
// Định danh đơn từ tiếng Anh: Explorer, line, pick, active, item, idx, selected

import React from 'react';

export default function Explorer({ line = [], pick, active }) {
  return (
    <div className="glass rounded-xl p-4 border border-gold/20 flex flex-col gap-3 shadow-glow">
      <div className="flex items-center justify-between border-b border-gold/20 pb-2">
        <h3 className="text-sm font-royal font-bold text-gold">
          PRINCIPAL VARIATION (PV)
        </h3>
        <span className="text-[10px] px-2 py-0.5 rounded bg-gold/10 border border-gold/30 text-gold font-mono">
          {line.length} MOVES
        </span>
      </div>

      {/* Hiển thị danh sách chuỗi biến thể PV dạng các nút nhấp tương tác */}
      {line.length === 0 ? (
        <div className="text-xs text-gold/40 py-4 text-center italic">
          No PV variation stream available yet. Start engine search.
        </div>
      ) : (
        <div className="flex flex-wrap gap-1.5 max-h-36 overflow-y-auto pr-1">
          {line.map((item, idx) => {
            const selected = active === idx;
            return (
              <button
                key={`pv-${idx}`}
                onClick={() => pick && pick(idx, item)}
                className={`px-2.5 py-1 rounded text-xs font-mono transition-all flex items-center gap-1 border ${
                  selected
                    ? 'bg-gold text-obsidian border-gold font-bold shadow-glow scale-105'
                    : 'bg-obsidian-card/80 text-gold/80 border-gold/20 hover:border-gold/60 hover:text-gold'
                }`}
              >
                <span className="text-[10px] text-gold/50 font-sans">
                  {idx + 1}.
                </span>
                <span>{item}</span>
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}
