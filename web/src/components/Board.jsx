// Linh kiện Bàn cờ Cờ Tướng Đồ Họa Hoàng Gia Tương Tác
// Định danh đơn từ tiếng Anh: Board, labels, fen, move, flip, disabled, check, state, update, click, start, over, drop, clear, point, board, turn, rank, file, x, y, index, piece, color, type, style, grid, svg, cx, cy, f, r, enemy, red, selected, king, threat, parsed, glow, ruby, dark, shadow, target, rulers, labelFile

import React, { useState, useEffect } from 'react';
import { parse, moves } from '../rules/rules.js';

// Bảng tra cứu nhãn ký tự chữ Hán Hoàng Gia cho các loại quân cờ
const labels = {
  K: '帥', A: '仕', B: '相', N: '傌', R: '俥', C: '炮', P: '兵',
  k: '將', a: '士', b: '象', n: '馬', r: '車', c: '砲', p: '卒'
};

export default function Board({ fen, move, lastMove = null, flip = false, disabled = false, check = false, rulers = true }) {
  const parsed = parse(fen);
  const board = parsed.board;
  const turn = parsed.turn;

  const [state, update] = useState({
    select: null,
    valid: [],
    drag: null,
    hover: null
  });

  const [prevMove, setPrevMove] = useState(null);
  const prevBoardRef = React.useRef(board);

  // Tự động phát hiện nước đi vừa thực hiện từ vị trí cũ (from) sang vị trí mới (to) khi FEN thay đổi
  useEffect(() => {
    const prev = prevBoardRef.current;
    if (prev && prev.length === 90 && board && board.length === 90 && prev !== board) {
      let fromIdx = -1;
      let toIdx = -1;
      for (let i = 0; i < 90; i++) {
        if (prev[i] !== '.' && board[i] === '.') {
          fromIdx = i;
        } else if (prev[i] !== board[i] && board[i] !== '.') {
          toIdx = i;
        }
      }
      if (fromIdx !== -1 && toIdx !== -1) {
        setPrevMove({ from: fromIdx, to: toIdx });
      }
    }
    prevBoardRef.current = board;
  }, [fen]);

  // Dọn dẹp trạng thái chọn khi fen thay đổi
  useEffect(() => {
    clear();
  }, [fen]);

  const activeMove = lastMove || prevMove;

  // Xác định màu sắc laser tương phản cao theo phe thực hiện nước đi (Red Side vs Black Side)
  const movedPiece = activeMove && activeMove.to >= 0 && board[activeMove.to] !== '.' ? board[activeMove.to] : null;
  const isRedMove = movedPiece ? movedPiece === movedPiece.toUpperCase() : (turn === 'b');
  const strokeColor = isRedMove ? '#FF0055' : '#00F0FF';
  const gradId = isRedMove ? 'laser-red' : 'laser-black';

  // Xử lý sự kiện nhấp chọn ô cờ (Click-to-Move)
  const click = (index) => {
    if (disabled) return;

    if (state.select === null) {
      const piece = board[index];
      if (piece !== '.') {
        const color = piece === piece.toUpperCase() ? 'w' : 'b';
        if (color === turn) {
          update({
            ...state,
            select: index,
            valid: moves(board, index, turn)
          });
        }
      }
    } else {
      if (state.select === index) {
        clear();
      } else if (state.valid.includes(index)) {
        if (move) move(state.select, index);
        clear();
      } else {
        const piece = board[index];
        if (piece !== '.') {
          const color = piece === piece.toUpperCase() ? 'w' : 'b';
          if (color === turn) {
            update({
              ...state,
              select: index,
              valid: moves(board, index, turn)
            });
            return;
          }
        }
        clear();
      }
    }
  };

  // Xử lý bắt đầu kéo quân cờ (Drag & Drop)
  const start = (index, event) => {
    if (disabled) return;
    const piece = board[index];
    if (piece !== '.') {
      const color = piece === piece.toUpperCase() ? 'w' : 'b';
      if (color === turn) {
        update({
          ...state,
          drag: index,
          select: index,
          valid: moves(board, index, turn)
        });
        if (event.dataTransfer) {
          event.dataTransfer.setData('text/plain', index.toString());
        }
      }
    }
  };

  // Xử lý rê quân cờ qua ô cờ (Drag Over)
  const over = (index, event) => {
    if (event) event.preventDefault();
    if (state.hover !== index) {
      update({
        ...state,
        hover: index
      });
    }
  };

  // Xử lý thả quân cờ xuống ô cờ (Drop)
  const drop = (index, event) => {
    if (event) event.preventDefault();
    if (state.drag !== null && state.valid.includes(index)) {
      if (move) move(state.drag, index);
    }
    clear();
  };

  // Xóa trạng thái chọn và gợi ý nước đi
  const clear = () => {
    update({
      select: null,
      valid: [],
      drag: null,
      hover: null
    });
  };

  // Ánh xạ chỉ số ô cờ 1D sang vị trí tọa độ SVG (cx, cy)
  const point = (index) => {
    const file = index % 9;
    const rank = Math.floor(index / 9);

    const f = flip ? 8 - file : file;
    const r = flip ? rank : 9 - rank;

    const cx = 50 + f * 100;
    const cy = 50 + r * 100;
    return { cx, cy };
  };

  return (
    <div className="relative w-full max-w-2xl mx-auto select-none p-3 glass rounded-2xl shadow-glow border-2 border-gold/40 bg-obsidian-card/60 flex flex-col gap-3">
      {/* 🟢 Thanh Trực Quan Hóa 2 Phe (Red Imperial vs Black Royal Army Side Bar) */}
      <div className="w-full flex items-center justify-between px-4 py-2 bg-obsidian-dark/80 rounded-xl border border-gold/30 shadow-inner text-xs font-bold">
        {/* Phe Đen (Black Side) */}
        <div className={`flex items-center gap-2.5 px-3 py-1.5 rounded-lg transition-all ${
          turn === 'b'
            ? 'bg-cyan-950/90 border-2 border-cyan-400 text-cyan-300 shadow-[0_0_12px_rgba(0,240,255,0.4)] scale-105'
            : 'opacity-60 text-slate-400 border border-slate-700'
        }`}>
          <div className="w-7 h-7 rounded-full bg-slate-900 border border-cyan-400 flex items-center justify-center font-serif text-sm font-bold text-cyan-300 shadow">
            將
          </div>
          <div className="flex flex-col">
            <span className="text-[10px] tracking-wider uppercase text-cyan-400 font-extrabold">PHE ĐEN</span>
            <span className="text-xs">{disabled ? 'AI THÁI THƯỢNG' : (turn === 'b' ? '▶ ĐANG SUY NGHĨ...' : 'CHỜ LƯỢT')}</span>
          </div>
        </div>

        {/* VS Status Badge */}
        <div className="flex flex-col items-center justify-center px-2">
          <span className="text-[10px] text-gold/60 tracking-widest font-mono font-bold">VS</span>
          <span className={`text-[10px] font-extrabold px-2.5 py-0.5 rounded-full uppercase tracking-wider ${
            turn === 'w' ? 'bg-red-500/20 text-red-400 border border-red-500/50 shadow-[0_0_8px_rgba(255,0,85,0.3)]' : 'bg-cyan-500/20 text-cyan-300 border border-cyan-400/50 shadow-[0_0_8px_rgba(0,240,255,0.3)]'
          }`}>
            LƯỢT {turn === 'w' ? '🔴 ĐỎ' : '🖤 ĐEN'}
          </span>
        </div>

        {/* Phe Đỏ (Red Side) */}
        <div className={`flex items-center gap-2.5 px-3 py-1.5 rounded-lg transition-all ${
          turn === 'w'
            ? 'bg-red-950/90 border-2 border-red-500 text-red-300 shadow-[0_0_12px_rgba(255,0,85,0.4)] scale-105'
            : 'opacity-60 text-slate-400 border border-slate-700'
        }`}>
          <div className="flex flex-col text-right">
            <span className="text-[10px] tracking-wider uppercase text-red-400 font-extrabold">PHE ĐỎ</span>
            <span className="text-xs">{turn === 'w' ? '▶ ĐANG SUY NGHĨ...' : 'CHỜ LƯỢT'}</span>
          </div>
          <div className="w-7 h-7 rounded-full bg-red-950 border border-red-500 flex items-center justify-center font-serif text-sm font-bold text-red-300 shadow">
            帥
          </div>
        </div>
      </div>

      <svg viewBox="0 0 900 1000" className="w-full h-full aspect-[9/10]">
        {/* Định nghĩa các hiệu ứng phát sáng Gradients Hoàng Gia */}
        <defs>
          <linearGradient id="laser-red" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#FF3366" stopOpacity="1" />
            <stop offset="100%" stopColor="#FF0055" stopOpacity="0.9" />
          </linearGradient>
          <linearGradient id="laser-black" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#00F0FF" stopOpacity="1" />
            <stop offset="100%" stopColor="#00A3FF" stopOpacity="0.9" />
          </linearGradient>
          <radialGradient id="glow" cx="50%" cy="50%" r="50%">
            <stop offset="0%" stopColor="#F3E5AB" stopOpacity="0.8" />
            <stop offset="100%" stopColor="#D4AF37" stopOpacity="0" />
          </radialGradient>
          <radialGradient id="ruby" cx="35%" cy="35%" r="65%">
            <stop offset="0%" stopColor="#2A0808" />
            <stop offset="70%" stopColor="#160303" />
            <stop offset="100%" stopColor="#0D0000" />
          </radialGradient>
          <radialGradient id="dark" cx="35%" cy="35%" r="65%">
            <stop offset="0%" stopColor="#1F242D" />
            <stop offset="70%" stopColor="#0F1318" />
            <stop offset="100%" stopColor="#05070A" />
          </radialGradient>
          <filter id="shadow" x="-20%" y="-20%" width="140%" height="140%">
            <feDropShadow dx="0" dy="6" stdDeviation="6" floodColor="#000000" floodOpacity="0.6" />
          </filter>
        </defs>

        {/* 1. Lưới Đường Kẻ Bàn Cờ Hoàng Kim */}
        <g stroke="#D4AF37" strokeWidth="3" strokeOpacity="0.85">
          {/* Đường ngang 10 hàng */}
          {[0, 1, 2, 3, 4, 5, 6, 7, 8, 9].map((r) => (
            <line key={`h-${r}`} x1="50" y1={50 + r * 100} x2="850" y2={50 + r * 100} />
          ))}

          {/* Đường dọc 9 lộ (Ngắt ở khu vực Sông) */}
          {[0, 8].map((f) => (
            <line key={`outer-${f}`} x1={50 + f * 100} y1="50" x2={50 + f * 100} y2="950" />
          ))}
          {[1, 2, 3, 4, 5, 6, 7].map((f) => (
            <React.Fragment key={`inner-${f}`}>
              <line x1={50 + f * 100} y1="50" x2={50 + f * 100} y2="450" />
              <line x1={50 + f * 100} y1="550" x2={50 + f * 100} y2="950" />
            </React.Fragment>
          ))}

          {/* Cửu Cung Đỏ & Đen (Đường chéo X) */}
          <line x1="350" y1="50" x2="550" y2="250" />
          <line x1="550" y1="50" x2="350" y2="250" />
          <line x1="350" y1="750" x2="550" y2="950" />
          <line x1="550" y1="750" x2="350" y2="950" />
        </g>

        {/* 2. Hiển thị Thước Tọa Độ Bàn Cờ (Coordinate Rulers: Files a..i & Ranks 0..9) */}
        {rulers && (
          <g fill="#D4AF37" fillOpacity="0.8" fontSize="20" fontFamily="monospace" fontWeight="bold" textAnchor="middle">
            {/* Tọa độ Cột (Files a..i) phía trên và phía dưới */}
            {[0, 1, 2, 3, 4, 5, 6, 7, 8].map((file) => {
              const f = flip ? 8 - file : file;
              const cx = 50 + f * 100;
              const labelFile = String.fromCharCode(97 + file); // 'a'..'i'
              return (
                <React.Fragment key={`file-rule-${file}`}>
                  <text x={cx} y="24">{labelFile}</text>
                  <text x={cx} y="988">{labelFile}</text>
                </React.Fragment>
              );
            })}

            {/* Tọa độ Hàng (Ranks 0..9) bên trái và bên phải */}
            {[0, 1, 2, 3, 4, 5, 6, 7, 8, 9].map((rank) => {
              const r = flip ? rank : 9 - rank;
              const cy = 50 + r * 100 + 6;
              return (
                <React.Fragment key={`rank-rule-${rank}`}>
                  <text x="20" y={cy}>{rank}</text>
                  <text x="880" y={cy}>{rank}</text>
                </React.Fragment>
              );
            })}
          </g>
        )}

        {/* 3. Văn bản Chữ Hán Sông "Sở Hà Hán Giới" */}
        <g fill="#D4AF37" fillOpacity="0.6" fontSize="36" fontFamily="serif" fontWeight="bold" textAnchor="middle">
          <text x="250" y="512" transform="rotate(-90 250 500)" style={{ writingMode: 'horizontal-tb' }}>
            楚 河
          </text>
          <text x="650" y="512" transform="rotate(-90 650 500)" style={{ writingMode: 'horizontal-tb' }}>
            漢 界
          </text>
        </g>

        {/* 4. Hiển Thị Đường Nối Nước Đi Hoàng Gia Tương Phản Rực Rỡ (Royal Neon Laser Path) */}
        {activeMove && activeMove.from !== undefined && activeMove.to !== undefined && activeMove.from >= 0 && activeMove.to >= 0 && (
          <g className="pointer-events-none">
            {(() => {
              const p1 = point(activeMove.from);
              const p2 = point(activeMove.to);
              const dx = p2.cx - p1.cx;
              const dy = p2.cy - p1.cy;
              const angle = Math.atan2(dy, dx) * (180 / Math.PI);
              const dist = Math.hypot(dx, dy);

              return (
                <g key={`move-path-${activeMove.from}-${activeMove.to}`}>
                  {/* Dải Hào Quang Phát Sáng Tương Phản Rực Rỡ Dưới Đường Nối */}
                  <line
                    x1={p1.cx} y1={p1.cy}
                    x2={p2.cx} y2={p2.cy}
                    stroke={strokeColor}
                    strokeWidth="12"
                    strokeOpacity="0.3"
                    strokeLinecap="round"
                  />

                  {/* Đường Nối Laser Neon Nối Tĩnh Đỉnh Cao (Royal Neon Laser Beam) */}
                  <line
                    x1={p1.cx} y1={p1.cy}
                    x2={p2.cx} y2={p2.cy}
                    stroke={`url(#${gradId})`}
                    strokeWidth="5"
                    strokeLinecap="round"
                    strokeDasharray="8 4"
                    filter="url(#shadow)"
                  />

                  {/* Vòng Tròn Định Vị Vị Trí Cũ (Origin Marker) */}
                  <circle
                    cx={p1.cx} cy={p1.cy} r="38"
                    fill="none"
                    stroke={strokeColor}
                    strokeWidth="2.5"
                    strokeDasharray="4 2"
                    strokeOpacity="0.8"
                  />
                  <circle cx={p1.cx} cy={p1.cy} r="8" fill={strokeColor} fillOpacity="0.8" />

                  {/* Mũi Tên Hướng Vector Di Chuyển Tương Phản Rực Rỡ */}
                  {dist > 40 && (
                    <g transform={`translate(${p2.cx - (dx / dist) * 42}, ${p2.cy - (dy / dist) * 42}) rotate(${angle})`}>
                      <polygon points="-12,-8 5,0 -12,8" fill={strokeColor} filter="url(#shadow)" />
                    </g>
                  )}

                  {/* Vòng Viền Vị Trí Mới (Target Arrival Ring) */}
                  <circle
                    cx={p2.cx} cy={p2.cy} r="42"
                    fill="none"
                    stroke={strokeColor}
                    strokeWidth="3"
                    strokeOpacity="0.95"
                  />
                </g>
              );
            })()}
          </g>
        )}

        {/* 5. Mạng lưới các ô cảm ứng sự kiện Click & Drop trong suốt (Vẽ TRƯỚC các quân cờ) */}
        {Array.from({ length: 90 }).map((_, index) => {
          const { cx, cy } = point(index);
          return (
            <rect
              key={`tile-${index}`}
              x={cx - 45}
              y={cy - 45}
              width="90"
              height="90"
              fill="transparent"
              onClick={() => click(index)}
              onDragOver={(e) => over(index, e)}
              onDrop={(e) => drop(index, e)}
            />
          );
        })}

        {/* 6. Hiển thị Gợi Ý Nước Đi Hợp Lệ & Vòng Khóa Mục Tiêu */}
        {state.valid.map((target) => {
          const { cx, cy } = point(target);
          const enemy = board[target] !== '.';
          return (
            <g key={`valid-${target}`} transform={`translate(${cx}, ${cy})`} className="pointer-events-none">
              {enemy ? (
                <>
                  <circle r="44" fill="none" stroke="#FF1A1A" strokeWidth="3.5" />
                  <circle r="38" fill="none" stroke="#DC143C" strokeWidth="2" strokeDasharray="6 3" />
                </>
              ) : (
                <circle r="12" fill="#D4AF37" fillOpacity="0.85" />
              )}
            </g>
          );
        })}

        {/* 7. Hiển thị 32 Quân Cờ Hoàng Gia */}
        {board.map((piece, index) => {
          if (piece === '.') return null;

          const { cx, cy } = point(index);
          const red = piece === piece.toUpperCase();
          const selected = state.select === index;
          const hovered = state.hover === index;
          const king = piece.toUpperCase() === 'K';
          const threat = check && king && ((red && turn === 'w') || (!red && turn === 'b'));
          const isLastTarget = activeMove && activeMove.to === index;
          const isLastOrigin = activeMove && activeMove.from === index;

          return (
            <g
              key={`piece-${index}`}
              transform={`translate(${cx}, ${cy})`}
              onClick={() => click(index)}
              onMouseEnter={() => update((prev) => ({ ...prev, hover: index }))}
              onMouseLeave={() => update((prev) => ({ ...prev, hover: null }))}
              onDragStart={(e) => start(index, e)}
              onDragEnd={clear}
              onDragOver={(e) => over(index, e)}
              onDrop={(e) => drop(index, e)}
              draggable={!disabled && ((red && turn === 'w') || (!red && turn === 'b'))}
              className="cursor-pointer piece-smooth"
              filter="url(#shadow)"
            >
              {/* Hiệu ứng Chiếu Tướng Flash */}
              {threat && (
                <circle r="46" fill="none" stroke="#FF1A1A" strokeWidth="6" className="flash" />
              )}

              {/* Vòng viền Highlight khi Nước Đi Vừa Đáp Đất (Last Move Target) */}
              {isLastTarget && (
                <circle r="44" fill="none" stroke="#FFD700" strokeWidth="4" className="animate-pulse" />
              )}

              {/* Vòng viền Highlight khi được chọn hoặc Hover */}
              {(selected || hovered) && (
                <circle r="45" fill="none" stroke="#D4AF37" strokeWidth={selected ? '5' : '3'} strokeOpacity={selected ? '1' : '0.7'} className="animate-pulse" />
              )}

              {/* Thân quân cờ 3D Ngọc Cẩm Thạch */}
              <circle r={hovered ? '40' : '38'} fill={red ? 'url(#ruby)' : 'url(#dark)'} stroke="#D4AF37" strokeWidth="3" className="transition-all duration-150" />

              {/* Vòng vành trong Hoàng Kim */}
              <circle r="32" fill="none" stroke="#D4AF37" strokeWidth="1" strokeOpacity="0.6" strokeDasharray="4 2" />

              {/* Chữ Hán danh xưng quân cờ Hoàng Gia */}
              <text
                y="11"
                textAnchor="middle"
                fontSize="34"
                fontWeight="bold"
                fontFamily="serif"
                fill={red ? '#8B0000' : '#D4AF37'}
                style={{
                  textShadow: red ? '0 0 8px rgba(220, 20, 60, 0.8)' : '0 0 8px rgba(212, 175, 55, 0.8)'
                }}
              >
                {labels[piece]}
              </text>
            </g>
          );
        })}
      </svg>
    </div>
  );
}
