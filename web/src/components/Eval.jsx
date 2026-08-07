// Component Thanh Thế Cờ Động (Dynamic Eval Bar) hiển thị Win Rate % và Điểm Centipawn
// Định danh đơn từ tiếng Anh: Eval, score, rate, bar, fill, gold, dark, text, val, side, sign, calc, pct

import React from 'react';

export default function Eval({ score = 0 }) {
  // Quy đổi điểm centipawns (-1000 đến +1000) sang tỷ lệ thắng Win Rate % (0% đến 100%)
  // Phương trình Sigmoid Win Rate: 1 / (1 + 10^(-score / 400)) * 100%
  const calc = (val) => {
    const pct = 1 / (1 + Math.pow(10, -val / 400));
    return Math.min(Math.max(Math.round(pct * 100), 2), 98);
  };

  const rate = calc(score);

  // Định dạng hiển thị điểm centipawn (ví dụ: +1.25 hoặc -0.80)
  const sign = score > 0 ? `+${(score / 100).toFixed(2)}` : (score / 100).toFixed(2);

  return (
    <div className="glass rounded-xl p-4 border border-gold/20 flex flex-col gap-3 shadow-glow">
      <div className="flex items-center justify-between text-xs font-royal font-bold text-gold">
        <span>EVALUATION</span>
        <span className="text-gold/80 font-mono">{sign}</span>
      </div>

      {/* Thanh hiển thị đồ họa dạng gradient Hoàng Gia */}
      <div className="h-5 bg-obsidian rounded-full overflow-hidden border border-gold/40 relative flex shadow-inner">
        {/* Phần màu Đỏ / Hoàng Kim (Gold) đại diện cho ưu thế Đỏ */}
        <div
          className="h-full bg-gold transition-all duration-500 ease-out flex items-center justify-start px-2 text-[10px] font-bold text-obsidian"
          style={{ width: `${rate}%` }}
        >
          {rate >= 15 && <span>{rate}%</span>}
        </div>

        {/* Phần màu Đen / Sơn Son (Vermilion) đại diện cho ưu thế Đen */}
        <div
          className="h-full bg-vermilion transition-all duration-500 ease-out flex items-center justify-end px-2 text-[10px] font-bold text-gold flex-1"
        >
          {rate <= 85 && <span>{100 - rate}%</span>}
        </div>
      </div>

      <div className="flex justify-between text-[11px] text-gold/60 font-body">
        <span className="flex items-center gap-1">
          <span className="w-2 h-2 rounded-full bg-gold inline-block"></span>
          RED WIN
        </span>
        <span className="flex items-center gap-1">
          BLACK WIN
          <span className="w-2 h-2 rounded-full bg-vermilion inline-block"></span>
        </span>
      </div>
    </div>
  );
}
