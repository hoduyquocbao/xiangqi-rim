// Component Studio Sắp Cờ Tương Tác - XiangRust Studio Board Editor
// Định danh đơn từ tiếng Anh: Studio, open, close, apply, fen, board, turn, pick, active, clear, reset, place, remove, item, idx, piece, color, rank, file, side, list, labels, parsed, update, state, select

import React, { useState, useEffect } from 'react';
import { parse } from '../rules/rules.js';

// Nhãn chữ Hán Hoàng Gia cho khay quân cờ
const labels = {
  K: '帥', A: '仕', B: '相', N: '傌', R: '俥', C: '炮', P: '兵',
  k: '將', a: '士', b: '象', n: '馬', r: '車', c: '砲', p: '卒'
};

// Danh sách quân cờ Đỏ và Đen trong khay Studio
const redPieces = ['K', 'A', 'B', 'N', 'R', 'C', 'P'];
const blackPieces = ['k', 'a', 'b', 'n', 'r', 'c', 'p'];

export default function Studio({ open, close, apply, initialFen }) {
  const defaultFen = 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1';
  const emptyFen = '4k4/9/9/9/9/9/9/9/9/4K4 w - - 0 1';

  const [state, update] = useState({
    fen: initialFen || defaultFen,
    pick: null,
    select: null
  });

  useEffect(() => {
    if (initialFen) {
      update((prev) => ({ ...prev, fen: initialFen }));
    }
  }, [initialFen, open]);

  if (!open) return null;

  const parsed = parse(state.fen);
  const board = parsed.board;
  const turn = parsed.turn;

  // Chuyển đổi mảng board [90] thành chuỗi FEN chuẩn
  const exportFen = (newBoard, newTurn) => {
    let out = '';
    for (let r = 9; r >= 0; r--) {
      let count = 0;
      for (let f = 0; f < 9; f++) {
        const sq = r * 9 + f;
        const p = newBoard[sq];
        if (p === '.') {
          count++;
        } else {
          if (count > 0) {
            out += count;
            count = 0;
          }
          out += p;
        }
      }
      if (count > 0) out += count;
      if (r > 0) out += '/';
    }
    out += ` ${newTurn || turn} - - 0 1`;
    return out;
  };

  // Đặt hoặc thay đổi quân cờ tại ô square
  const handleSquareClick = (index) => {
    const cloned = [...board];
    if (state.pick) {
      // Đặt quân đang chọn từ khay vào ô
      cloned[index] = state.pick;
      const newFen = exportFen(cloned, turn);
      update({ ...state, fen: newFen, select: null });
    } else if (state.select !== null) {
      if (state.select === index) {
        // Bỏ chọn ô
        update({ ...state, select: null });
      } else {
        // Di chuyển quân từ ô select sang ô index
        cloned[index] = cloned[state.select];
        cloned[state.select] = '.';
        const newFen = exportFen(cloned, turn);
        update({ ...state, fen: newFen, select: null });
      }
    } else {
      if (cloned[index] !== '.') {
        update({ ...state, select: index });
      }
    }
  };

  // Xóa quân cờ khỏi ô được chọn
  const removePiece = (index) => {
    const cloned = [...board];
    cloned[index] = '.';
    const newFen = exportFen(cloned, turn);
    update({ ...state, fen: newFen, select: null });
  };

  // Xóa sạch bàn cờ chỉ giữ lại Tướng 2 bên
  const clearBoard = () => {
    update({ fen: emptyFen, pick: null, select: null });
  };

  // Khôi phục bàn cờ về vị trí ban đầu mặc định
  const resetBoard = () => {
    update({ fen: defaultFen, pick: null, select: null });
  };

  // Đổi lượt đi (Đỏ / Đen)
  const toggleTurn = () => {
    const nextTurn = turn === 'w' ? 'b' : 'w';
    const newFen = exportFen(board, nextTurn);
    update({ ...state, fen: newFen });
  };

  // Hoàn tất sắp cờ và nạp vào bàn đấu chính
  const confirmSetup = () => {
    if (apply) apply(state.fen);
    close();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm p-4 animate-fade-in">
      <div className="relative w-full max-w-3xl bg-slate-900 border-2 border-amber-500/40 rounded-2xl p-6 shadow-2xl text-slate-100 font-sans max-h-[95vh] overflow-y-auto">
        {/* Tiêu đề Modal Studio Sắp Cờ */}
        <div className="flex items-center justify-between border-b border-amber-500/30 pb-4 mb-4">
          <div className="flex items-center space-x-3">
            <div className="w-10 h-10 rounded-xl bg-amber-500/20 border border-amber-500/50 flex items-center justify-center text-amber-400 font-bold text-xl">
              🧩
            </div>
            <div>
              <h2 className="text-xl font-extrabold text-amber-400 tracking-wide uppercase">
                XiangRust Board Setup Studio
              </h2>
              <p className="text-xs text-slate-400">
                Chế Độ Studio Sắp Cờ Tự Do & Tạo Thế Cờ Tùy Biến
              </p>
            </div>
          </div>
          <button
            onClick={close}
            className="w-8 h-8 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-400 hover:text-white flex items-center justify-center text-lg transition"
          >
            ✕
          </button>
        </div>

        {/* Khung Thao Tác Nhanh & Chọn Lượt Đi */}
        <div className="flex flex-wrap items-center justify-between gap-3 mb-4 bg-slate-950 p-3 rounded-xl border border-amber-500/20">
          <div className="flex items-center space-x-2">
            <span className="text-xs text-slate-400 font-bold uppercase">Lượt Đi:</span>
            <button
              onClick={toggleTurn}
              className={`px-3 py-1.5 rounded-lg text-xs font-bold transition flex items-center gap-1.5 ${
                turn === 'w'
                  ? 'bg-rose-500/20 text-rose-400 border border-rose-500/40'
                  : 'bg-slate-800 text-slate-200 border border-slate-700'
              }`}
            >
              <span className={`w-2.5 h-2.5 rounded-full ${turn === 'w' ? 'bg-rose-500' : 'bg-slate-100'}`}></span>
              {turn === 'w' ? '🔴 Đỏ Đi Trước' : '⚫ Đen Đi Trước'}
            </button>
          </div>

          <div className="flex items-center space-x-2">
            <button
              onClick={clearBoard}
              className="px-3 py-1.5 rounded-lg bg-rose-950/60 hover:bg-rose-900/80 text-rose-300 border border-rose-800/40 text-xs font-bold transition"
            >
              🗑️ Xóa Sạch Bàn Cờ
            </button>
            <button
              onClick={resetBoard}
              className="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 border border-slate-700 text-xs font-bold transition"
            >
              🔄 Bàn Cờ Mặc Định
            </button>
          </div>
        </div>

        {/* Khay Quân Cờ (Piece Palette Bar) */}
        <div className="bg-slate-950 p-3 rounded-xl border border-amber-500/20 mb-4">
          <span className="text-xs font-bold text-amber-400 uppercase tracking-wider block mb-2">
            Khay Quân Cờ (Nhấp Chọn Quân Đặt Vào Bàn Cờ):
          </span>
          <div className="grid grid-cols-2 gap-4">
            {/* Quân Đỏ */}
            <div className="flex items-center space-x-1.5 bg-slate-900/80 p-2 rounded-lg border border-rose-500/20">
              <span className="text-[10px] font-bold text-rose-400 uppercase mr-1">Đỏ:</span>
              {redPieces.map((p) => (
                <button
                  key={p}
                  onClick={() => update({ ...state, pick: state.pick === p ? null : p, select: null })}
                  className={`w-8 h-8 rounded-full border-2 flex items-center justify-center font-bold text-sm transition shadow ${
                    state.pick === p
                      ? 'bg-rose-600 text-white border-yellow-300 scale-110 ring-2 ring-yellow-400'
                      : 'bg-rose-950/80 text-rose-200 border-rose-500/60 hover:border-rose-400'
                  }`}
                >
                  {labels[p]}
                </button>
              ))}
            </div>

            {/* Quân Đen */}
            <div className="flex items-center space-x-1.5 bg-slate-900/80 p-2 rounded-lg border border-slate-700/60">
              <span className="text-[10px] font-bold text-slate-300 uppercase mr-1">Đen:</span>
              {blackPieces.map((p) => (
                <button
                  key={p}
                  onClick={() => update({ ...state, pick: state.pick === p ? null : p, select: null })}
                  className={`w-8 h-8 rounded-full border-2 flex items-center justify-center font-bold text-sm transition shadow ${
                    state.pick === p
                      ? 'bg-slate-100 text-slate-950 border-amber-400 scale-110 ring-2 ring-amber-400'
                      : 'bg-slate-800 text-slate-200 border-slate-600 hover:border-slate-400'
                  }`}
                >
                  {labels[p]}
                </button>
              ))}
            </div>
          </div>
        </div>

        {/* Lưới Bàn Cờ Studio Tương Tác */}
        <div className="bg-amber-950/20 border border-amber-500/30 rounded-xl p-4 mb-4">
          <div className="grid grid-cols-9 gap-1.5 max-w-md mx-auto aspect-[9/10] bg-slate-950 p-3 rounded-lg border border-amber-500/40">
            {Array.from({ length: 90 }).map((_, idx) => {
              // Quy đổi index phẳng [90] sang (rank, file)
              const r = 9 - Math.floor(idx / 9);
              const f = idx % 9;
              const squareIdx = r * 9 + f;
              const piece = board[squareIdx];
              const isSelected = state.select === squareIdx;

              return (
                <div
                  key={squareIdx}
                  onClick={() => handleSquareClick(squareIdx)}
                  className={`relative aspect-square rounded-full border flex items-center justify-center cursor-pointer transition ${
                    isSelected
                      ? 'border-yellow-400 bg-yellow-500/30 scale-105 ring-2 ring-yellow-400'
                      : 'border-slate-800 hover:border-amber-500/50 bg-slate-900/60'
                  }`}
                >
                  {piece !== '.' && (
                    <div
                      className={`w-full h-full rounded-full flex items-center justify-center font-bold text-sm shadow ${
                        piece === piece.toUpperCase()
                          ? 'bg-rose-900 text-rose-100 border border-rose-500'
                          : 'bg-slate-800 text-slate-100 border border-slate-500'
                      }`}
                    >
                      {labels[piece]}
                    </div>
                  )}

                  {isSelected && piece !== '.' && (
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        removePiece(squareIdx);
                      }}
                      className="absolute -top-1 -right-1 w-4 h-4 bg-rose-600 text-white rounded-full text-[10px] flex items-center justify-center font-bold"
                    >
                      ✕
                    </button>
                  )}
                </div>
              );
            })}
          </div>
        </div>

        {/* Chuỗi FEN Tương Ứng & Nút Hoàn Tất */}
        <div className="bg-slate-950 p-3 rounded-xl border border-slate-800 mb-6">
          <span className="text-[11px] font-mono text-slate-400 uppercase">FEN Output:</span>
          <p className="text-xs font-mono text-amber-300 break-all mt-1">{state.fen}</p>
        </div>

        <div className="flex justify-end space-x-3">
          <button
            onClick={close}
            className="py-3 px-6 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 font-bold text-sm uppercase transition"
          >
            Hủy
          </button>
          <button
            onClick={confirmSetup}
            className="py-3 px-6 rounded-xl bg-gradient-to-r from-amber-500 to-yellow-500 hover:from-amber-400 hover:to-yellow-400 text-slate-950 font-black text-sm uppercase transition shadow-lg shadow-amber-900/40"
          >
            ✅ Hoàn Tất Sắp Cờ & Đấu
          </button>
        </div>
      </div>
    </div>
  );
}
