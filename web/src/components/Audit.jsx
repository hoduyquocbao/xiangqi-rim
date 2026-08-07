// Component Modal Audit Diagnostic & Dark Pattern Vulnerability Scanner - XiangRust
// Định danh đơn từ tiếng Anh: Audit, open, close, fen, unguarded, overloaded, exposure, horizon, penalty, status, fetch, type, JSON, state, update, load, view, item, info, badge, apiFetch

import React, { useState, useEffect } from 'react';

// Hàm helper apiFetch chuyển vùng tự động giữa relative path và host hiện tại / Cloudflare Tunnel
const apiFetch = async (path, options) => {
  try {
    const res = await fetch(path, options);
    if (res && res.ok) return res;
  } catch (_) {}
  const host = typeof window !== 'undefined' && window.location.hostname ? window.location.hostname : '127.0.0.1';
  try {
    return await fetch(`http://${host}:8888${path}`, options);
  } catch (err) {
    return null;
  }
};

export default function Audit({ open, close, fen }) {
  const [report, update] = useState({
    unguarded: 0,
    overloaded: 0,
    exposure: 0,
    horizon: 0,
    penalty: 0,
    status: 'idle'
  });

  // Tự động quét chẩn đoán khi Modal mở hoặc FEN bàn cờ thay đổi
  useEffect(() => {
    if (!open || !fen) return;

    const scan = async () => {
      try {
        const res = await apiFetch(`/api/v1/audit?fen=${encodeURIComponent(fen)}`);
        if (res && res.ok) {
          const data = await res.json();
          update({
            unguarded: data.unguarded || 0,
            overloaded: data.overloaded || 0,
            exposure: data.exposure || 0,
            horizon: data.horizon || 0,
            penalty: data.penalty || 0,
            status: 'scanned'
          });
        }
      } catch (err) {
        update((prev) => ({ ...prev, status: 'error' }));
      }
    };

    scan();
  }, [open, fen]);

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm p-4 animate-fade-in">
      <div className="relative w-full max-w-xl bg-slate-900 border-2 border-indigo-500/40 rounded-2xl p-6 shadow-2xl text-slate-100 font-sans">
        {/* Tiêu đề Modal Audit Scanner */}
        <div className="flex items-center justify-between border-b border-indigo-500/30 pb-4 mb-6">
          <div className="flex items-center space-x-3">
            <div className="w-10 h-10 rounded-xl bg-indigo-500/20 border border-indigo-500/50 flex items-center justify-center text-indigo-400 font-bold text-xl">
              🛡️
            </div>
            <div>
              <h2 className="text-xl font-extrabold text-indigo-400 tracking-wide uppercase">
                XiangRust Vulnerability Audit
              </h2>
              <p className="text-xs text-slate-400">
                Rà Soát & Chẩn Đoán Ngây Thơ, Rủi Ro Tiềm Ẩn & Lỗ Hổng Phòng Thủ
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

        {/* Bảng Chỉ Số Chẩn Đoán Rủi Ro */}
        <div className="grid grid-cols-2 gap-4 mb-6">
          {/* Unguarded Major Pieces */}
          <div className="bg-slate-800/80 border border-indigo-500/20 rounded-xl p-4">
            <span className="text-xs font-semibold text-slate-400 uppercase tracking-wider">
              Quân Chủ Lực Hổng Vệ Sĩ
            </span>
            <p
              className={`text-2xl font-black mt-1 ${
                report.unguarded > 0 ? 'text-rose-400' : 'text-emerald-400'
              }`}
            >
              {report.unguarded} quân
            </p>
            <p className="text-[11px] text-slate-400 mt-1">
              {report.unguarded > 0
                ? 'Phát hiện Xe/Pháo/Mã không có vệ sĩ bảo vệ!'
                : 'Hệ thống phòng thủ quân chủ lực an toàn.'}
            </p>
          </div>

          {/* Overloaded Defenders */}
          <div className="bg-slate-800/80 border border-indigo-500/20 rounded-xl p-4">
            <span className="text-xs font-semibold text-slate-400 uppercase tracking-wider">
              Quân Phòng Thủ Quá Tải
            </span>
            <p
              className={`text-2xl font-black mt-1 ${
                report.overloaded > 0 ? 'text-amber-400' : 'text-emerald-400'
              }`}
            >
              {report.overloaded} quân
            </p>
            <p className="text-[11px] text-slate-400 mt-1">
              {report.overloaded > 0
                ? 'Phát hiện quân cờ gánh giữ 2 mục tiêu cùng lúc!'
                : 'Không có quân cờ bị quá tải nhiệm vụ.'}
            </p>
          </div>

          {/* Palace Exposure */}
          <div className="bg-slate-800/80 border border-indigo-500/20 rounded-xl p-4">
            <span className="text-xs font-semibold text-slate-400 uppercase tracking-wider">
              Chỉ Số Nguy Cơ Hở Cung
            </span>
            <p
              className={`text-2xl font-black mt-1 ${
                report.exposure > 0 ? 'text-rose-400' : 'text-emerald-400'
              }`}
            >
              {report.exposure} điểm
            </p>
            <p className="text-[11px] text-slate-400 mt-1">
              {report.exposure > 0
                ? 'Lỗ hổng Cung Tướng / khuyết Sĩ Tượng bị đe dọa!'
                : 'Sĩ Tượng vững chắc bảo vệ Cung Tướng.'}
            </p>
          </div>

          {/* Total Centipawn Penalty */}
          <div className="bg-slate-800/80 border border-indigo-500/20 rounded-xl p-4">
            <span className="text-xs font-semibold text-slate-400 uppercase tracking-wider">
              Tổng Điểm Phạt Rủi Ro
            </span>
            <p className="text-2xl font-black text-rose-400 mt-1">
              -{report.penalty} cp
            </p>
            <p className="text-[11px] text-slate-400 mt-1">
              Chỉ số rủi ro chân trời: {report.horizon}
            </p>
          </div>
        </div>

        {/* Nút Đóng Modal */}
        <div className="flex justify-end">
          <button
            onClick={close}
            className="py-3 px-6 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white font-bold text-sm uppercase transition shadow-lg shadow-indigo-900/40"
          >
            Đóng Báo Cáo Chẩn Đoán
          </button>
        </div>
      </div>
    </div>
  );
}
