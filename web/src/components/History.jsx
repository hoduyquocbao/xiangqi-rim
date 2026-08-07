// Bảng Nhật ký Lịch sử Ván đấu Hoàng Gia (XiangRust Match History & Replay Modal)
// Định danh đơn từ tiếng Anh: History, modal, list, match, open, close, clear, export, play, item, stamp, mode, depth, result, winner, moves, button, text, flex, card

import React, { useState, useEffect } from 'react';
import * as store from '../storage/store.js';

export default function History({ open, close, loadMatch }) {
  const [list, update] = useState([]);

  const refresh = () => {
    update(store.load());
  };

  useEffect(() => {
    if (open) {
      refresh();
    }
  }, [open]);

  if (!open) return null;

  const handleClear = () => {
    if (window.confirm('Bạn có chắc chắn muốn xóa toàn bộ lịch sử ván đấu đã lưu?')) {
      store.clear();
      refresh();
    }
  };

  const handleExport = (match) => {
    const pgn = store.exportPGN(match.history, match.winner ? (match.winner === 'w' ? '1-0' : '0-1') : '1/2-1/2');
    const blob = new Blob([pgn], { type: 'text/plain;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = match.id + '.pgn';
    link.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-md animate-fade-in p-4">
      <div className="relative w-full max-w-2xl bg-obsidian border-2 border-gold/40 rounded-xl shadow-2xl overflow-hidden flex flex-col max-h-[85vh]">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-gold/20 bg-gold/5">
          <div className="flex items-center gap-3">
            <span className="text-2xl">📜</span>
            <h2 className="text-xl font-bold text-gold tracking-wide">NHẬT KÝ VÁN ĐẤU & KÍNH NGHIỆM AI</h2>
          </div>
          <button
            onClick={close}
            className="text-gold/60 hover:text-gold text-2xl font-bold transition-colors"
          >
            ✕
          </button>
        </div>

        {/* Content Body */}
        <div className="p-6 overflow-y-auto flex-1 space-y-4">
          {list.length === 0 ? (
            <div className="text-center py-12 text-gold/40 text-sm">
              Chưa có ván đấu nào được lưu trữ trong nhật ký. Hãy hoàn thành một ván đấu để tự động ghi vết trí nhớ!
            </div>
          ) : (
            list.map((item, idx) => (
              <div
                key={item.id || idx}
                className="bg-black/40 border border-gold/20 rounded-lg p-4 flex flex-col md:flex-row md:items-center justify-between gap-4 hover:border-gold/50 transition-all"
              >
                <div className="space-y-1">
                  <div className="flex items-center gap-2">
                    <span className={`px-2 py-0.5 text-xs font-bold rounded ${item.winner === 'w' ? 'bg-gold/20 text-gold border border-gold/40' : item.winner === 'b' ? 'bg-red-900/40 text-red-400 border border-red-500/40' : 'bg-gray-800 text-gray-400'}`}>
                      {item.winner === 'w' ? 'HOÀNG KIM THẮNG' : item.winner === 'b' ? 'THÁI THƯỢNG THẮNG' : 'HÒA CỜ'}
                    </span>
                    <span className="text-xs text-gold/40">
                      {new Date(item.stamp).toLocaleString('vi-VN')}
                    </span>
                  </div>
                  <div className="text-xs text-gold/70 flex gap-4">
                    <span>Chế độ: <strong className="text-gold">{item.mode?.toUpperCase()}</strong></span>
                    <span>Độ sâu: <strong className="text-gold">Depth {item.depth}</strong></span>
                    <span>Số nước: <strong className="text-gold">{item.movesCount}</strong></span>
                  </div>
                </div>

                <div className="flex items-center gap-2">
                  {loadMatch && (
                    <button
                      onClick={() => {
                        loadMatch(item);
                        close();
                      }}
                      className="px-3 py-1.5 text-xs font-semibold bg-gold/10 hover:bg-gold/20 text-gold border border-gold/40 rounded transition-colors"
                    >
                      NẠP BÀN CỜ
                    </button>
                  )}
                  <button
                    onClick={() => handleExport(item)}
                    className="px-3 py-1.5 text-xs font-semibold bg-blue-900/20 hover:bg-blue-900/40 text-blue-300 border border-blue-500/40 rounded transition-colors"
                  >
                    XUẤT PGN
                  </button>
                </div>
              </div>
            ))
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between px-6 py-4 border-t border-gold/20 bg-gold/5">
          <button
            onClick={handleClear}
            disabled={list.length === 0}
            className="px-4 py-2 text-xs font-bold text-red-400 hover:text-red-300 bg-red-950/40 hover:bg-red-900/60 border border-red-500/30 rounded transition-colors disabled:opacity-40"
          >
            XÓA TOÀN BỘ LỊCH SỬ
          </button>
          <button
            onClick={close}
            className="px-5 py-2 text-xs font-bold bg-gold text-black hover:bg-gold/90 rounded font-semibold transition-all shadow-md"
          >
            ĐÓNG
          </button>
        </div>
      </div>
    </div>
  );
}
