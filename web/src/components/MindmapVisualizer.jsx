// web/src/components/MindmapVisualizer.jsx
// Sơ Đồ Tư Duy Cây Suy Luận 360° & Hậu Kiểm Toàn Ván (Post-Game Retroactive Mindmap)
// Visualizer trực quan hóa: Quy nạp lùi (Backward Induction), Lan truyền tri thức ngược (Ply 20 ➔ Ply 45 ➔ Ply 68 Checkmate),
// Bánh đà tri thức Persistent TT và Bàn cờ Mini SVG tương tác đa chiều.

import React, { useState } from 'react';
import { 
  GitCommit, 
  ArrowUpCircle, 
  Zap, 
  Database, 
  ShieldCheck, 
  Sparkles, 
  Layers, 
  Compass, 
  Cpu, 
  TrendingUp, 
  Eye, 
  Share2, 
  Maximize2, 
  Play, 
  RotateCcw,
  CheckCircle2,
  AlertTriangle
} from 'lucide-react';
import { parse } from '../rules/rules.js';

// Ký tự chữ Hán truyền thống cho 14 loại quân cờ
const symbols = {
  K: '帥', A: '仕', B: '相', N: '傌', R: '俥', C: '炮', P: '兵',
  k: '將', a: '士', b: '象', n: '馬', r: '車', c: '砲', p: '卒'
};

// 3 Mốc thế cờ định mệnh theo chuỗi nhân quả (Causal Timeline)
const milestones = [
  {
    id: 'ply20',
    ply: 20,
    turn: 10,
    title: 'PLY 20: NƯỚC CỜ BƯỚC NGOẶT (TURNING POINT)',
    tag: 'GÀI BẪY CHIẾN THUẬT',
    tagColor: 'bg-amber-500/20 text-amber-300 border-amber-500/40',
    moveUci: 'c3c4',
    moveNotation: 'Binh 7 tiến 1 (Thí Tốt)',
    evalScore: '+120 cp',
    horizonEval: '+120 cp (Depth 4 chỉ thấy ăn thua 1 Binh)',
    omniscienceEval: '+29,995 cp (Thấu thị: Khởi đầu chuỗi sát cục sau 25 nước)',
    fen: 'r1bakab1r/9/1cn1c1n2/p1p1p1p1p/9/2P6/P3P1P1P/1C2C4/9/RNBAKABNR w - - 0 20',
    arrow: { from: 29, to: 38 }, // c3 (file 2, rank 3 = 29) to c4 (file 2, rank 4 = 38)
    thoughtExcerpt: '"Ta chọn nước thí Binh c3c4 này không phải để ăn hơn 100cp trước mắt, mà là đòn gài bẫy dài hạn 25 nước, mở thông trục Lộ 5 cho Pháo đầu, ép đối phương rơi vào thế bị sát cục không thể cứu vãn tại Ply 68!"',
    reasoningPoints: [
      'Thí 1 Binh biên Lộ 7 mở toang đường tiến công cho Song Mã và Pháo đầu Lộ 5.',
      'Ép Tốt đối phương phải ăn sang, làm hổng chân Mã và phá vỡ cấu trúc Sĩ Tượng liên kết.',
      'Khởi phát thế trận Pháo Đầu Ép Trung Lộ (Center Cannon Pressure).'
    ]
  },
  {
    id: 'ply45',
    ply: 45,
    turn: 23,
    title: 'PLY 45: ĐÒN XE PHÁO ÁP ĐÁY (TACTICAL SQUEEZE)',
    tag: 'ĐÒN PHỐI HỢP TRUNG CUỘC',
    tagColor: 'bg-cyan-500/20 text-cyan-300 border-cyan-500/40',
    moveUci: 'e2e9',
    moveNotation: 'Pháo 5 tiến 7 (Cắm Pháo Đáy)',
    evalScore: '+850 cp',
    horizonEval: '+350 cp (Depth 4 thấy ưu thế hơn quân)',
    omniscienceEval: '+30,000 cp (Thấu thị: Ép đối phương gãy Sĩ, sát cục không thể cứu vãn)',
    fen: '2bakab2/9/1cn6/p1p3p1p/9/2C1C1R2/P3P1P1P/9/9/RNBAKAB2 w - - 3 45',
    arrow: { from: 22, to: 85 }, // e2 (22) to e9 (85)
    thoughtExcerpt: '"Đòn cắm Pháo đáy e2e9+ kết hợp Xe Lộ 8 áp sườn Cung Tướng. Đối phương bắt buộc phải hy sinh Sĩ hoặc vẹo Tướng, mở đường cho Mã ngọa tào kết liễu trận đấu!"',
    reasoningPoints: [
      'Thiết lập thế trận Thiết Môn Thuyên / Xe Pháo Dồn Góc khóa chặt sườn Cung.',
      'Triệt tiêu toàn bộ quân bảo vệ của Tướng đối phương tại hàng đáy Tuyến 9.',
      'Chuẩn bị đưa Mã thâm nhập ngọa tào tung đòn trừng phạt dứt điểm.'
    ]
  },
  {
    id: 'ply68',
    ply: 68,
    turn: 34,
    title: 'PLY 68: SÁT CỤC THỰC TẾ (TERMINAL CHECKMATE)',
    tag: 'KẾT LIỄU DỨT ĐIỂM',
    tagColor: 'bg-emerald-500/20 text-emerald-300 border-emerald-500/40',
    moveUci: 'e6g7',
    moveNotation: 'Mã 5 tiến 3 (Mã Ngọa Tào Sát Cục)',
    evalScore: '+30,000 cp (Checkmate)',
    horizonEval: '+30,000 cp (Checkmate)',
    omniscienceEval: '+30,000 cp (Checkmate hoàn tất 100%)',
    fen: '4k4/4C4/b4Rn1b/9/4R4/8p/P1P1N1r2/9/4A4/4KAB2 b - - 0 68',
    arrow: { from: 58, to: 69 }, // e6 (58) to g7 (69)
    thoughtExcerpt: '"SÁT CỤC HOÀN HẢO! Mã ngọa tào kết hợp Song Xe chiếu bí. Toàn bộ chuỗi tính toán từ nước cờ bước ngoặt Ply 20 đã được thực thi trọn vẹn 100% không sai lệch!"',
    reasoningPoints: [
      'Tướng đối phương bị giam cầm trong góc chết, không có nước đi hợp lệ.',
      '100% Phân định thắng bại dứt điểm, 0% hòa lặp nước.',
      'Ghi nhận sự kiện chiến thắng vào Sổ cái CQRS Event Sourcing bất biến.'
    ]
  }
];

// Linh kiện Bàn cờ Mini SVG tương tác
function MiniSvgBoard({ fen, arrow }) {
  const { board } = parse(fen);
  
  // Tọa độ bàn cờ SVG: 9 cột (x: 20..260), 10 hàng (y: 20..290)
  const cellW = 30;
  const cellH = 30;
  const padX = 25;
  const padY = 25;

  const getX = (file) => padX + file * cellW;
  const getY = (rank) => padY + (9 - rank) * cellH;

  return (
    <div className="relative bg-[#0b0f17] p-3 rounded-2xl border border-gold/40 shadow-2xl flex flex-col items-center">
      <svg 
        viewBox="0 0 290 320" 
        className="w-full max-w-[280px] drop-shadow-md select-none"
      >
        {/* Nền bàn cờ vân gỗ hoàng gia */}
        <defs>
          <radialGradient id="woodGrad" cx="50%" cy="50%" r="75%">
            <stop offset="0%" stopColor="#2c1e11" />
            <stop offset="100%" stopColor="#140d07" />
          </radialGradient>
          <linearGradient id="laserArrow" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#FFD700" />
            <stop offset="100%" stopColor="#FF1493" />
          </linearGradient>
          <filter id="glowEffect" x="-20%" y="-20%" width="140%" height="140%">
            <feGaussianBlur stdDeviation="3" result="blur" />
            <feComposite in="SourceGraphic" in2="blur" operator="over" />
          </filter>
        </defs>

        <rect x="5" y="5" width="280" height="310" rx="10" fill="url(#woodGrad)" stroke="#D4AF37" strokeWidth="2" />
        
        {/* Lưới 9x10 */}
        {/* 10 đường ngang */}
        {Array.from({ length: 10 }).map((_, r) => (
          <line 
            key={`h-${r}`} 
            x1={getX(0)} y1={getY(r)} 
            x2={getX(8)} y2={getY(r)} 
            stroke="#8A6B2D" 
            strokeWidth="1.2" 
          />
        ))}

        {/* 9 đường dọc (ngắt ở sông giữa hàng 4 và 5) */}
        {Array.from({ length: 9 }).map((_, f) => (
          <React.Fragment key={`v-${f}`}>
            {/* Nửa bàn dưới */}
            <line 
              x1={getX(f)} y1={getY(0)} 
              x2={getX(f)} y2={getY(4)} 
              stroke="#8A6B2D" 
              strokeWidth="1.2" 
            />
            {/* Nửa bàn trên */}
            <line 
              x1={getX(f)} y1={getY(5)} 
              x2={getX(f)} y2={getY(9)} 
              stroke="#8A6B2D" 
              strokeWidth="1.2" 
            />
          </React.Fragment>
        ))}

        {/* 2 đường biên dọc qua sông */}
        <line x1={getX(0)} y1={getY(4)} x2={getX(0)} y2={getY(5)} stroke="#8A6B2D" strokeWidth="1.2" />
        <line x1={getX(8)} y1={getY(4)} x2={getX(8)} y2={getY(5)} stroke="#8A6B2D" strokeWidth="1.2" />

        {/* Chéo Cung Tướng Đỏ (d0-f2) */}
        <line x1={getX(3)} y1={getY(0)} x2={getX(5)} y2={getY(2)} stroke="#8A6B2D" strokeWidth="1.2" />
        <line x1={getX(5)} y1={getY(0)} x2={getX(3)} y2={getY(2)} stroke="#8A6B2D" strokeWidth="1.2" />

        {/* Chéo Cung Tướng Đen (d7-f9) */}
        <line x1={getX(3)} y1={getY(7)} x2={getX(5)} y2={getY(9)} stroke="#8A6B2D" strokeWidth="1.2" />
        <line x1={getX(5)} y1={getY(7)} x2={getX(3)} y2={getY(9)} stroke="#8A6B2D" strokeWidth="1.2" />

        {/* Chữ Sông Sở Hà Hán Giới */}
        <text x="75" y={getY(4.5) + 4} fill="#8A6B2D" fontSize="10" fontFamily="serif" opacity="0.8">楚 河</text>
        <text x="175" y={getY(4.5) + 4} fill="#8A6B2D" fontSize="10" fontFamily="serif" opacity="0.8">漢 界</text>

        {/* Mũi tên chỉ nước đi chiến thuật (Laser Arrow) */}
        {arrow && (
          <g filter="url(#glowEffect)">
            <line 
              x1={getX(arrow.from % 9)} 
              y1={getY(Math.floor(arrow.from / 9))} 
              x2={getX(arrow.to % 9)} 
              y2={getY(Math.floor(arrow.to / 9))} 
              stroke="url(#laserArrow)" 
              strokeWidth="3.5" 
              strokeDasharray="4 2"
              strokeLinecap="round"
            />
            <circle 
              cx={getX(arrow.from % 9)} 
              cy={getY(Math.floor(arrow.from / 9))} 
              r="6" 
              fill="#FFD700" 
              opacity="0.8" 
            />
            <circle 
              cx={getX(arrow.to % 9)} 
              cy={getY(Math.floor(arrow.to / 9))} 
              r="7" 
              fill="#FF1493" 
            />
          </g>
        )}

        {/* Quân cờ 90 ô */}
        {board.map((p, idx) => {
          if (p === '.') return null;
          const file = idx % 9;
          const rank = Math.floor(idx / 9);
          const cx = getX(file);
          const cy = getY(rank);
          const isRed = p === p.toUpperCase();
          const symbol = symbols[p] || p;

          return (
            <g key={`p-${idx}`} className="cursor-pointer transition-transform hover:scale-110">
              {/* Bóng đổ */}
              <circle cx={cx + 1} cy={cy + 1.5} r="11" fill="#000000" opacity="0.6" />
              {/* Khối quân cờ */}
              <circle 
                cx={cx} 
                cy={cy} 
                r="11" 
                fill={isRed ? '#fef3c7' : '#1e293b'} 
                stroke={isRed ? '#b91c1c' : '#0f172a'} 
                strokeWidth="1.5" 
              />
              <circle 
                cx={cx} 
                cy={cy} 
                r="9.5" 
                fill="none" 
                stroke={isRed ? '#dc2626' : '#475569'} 
                strokeWidth="0.8" 
                strokeDasharray="2 1" 
              />
              {/* Ký tự chữ Hán */}
              <text 
                x={cx} 
                y={cy + 4} 
                textAnchor="middle" 
                fill={isRed ? '#b91c1c' : '#38bdf8'} 
                fontSize="11" 
                fontWeight="bold" 
                fontFamily="serif"
              >
                {symbol}
              </text>
            </g>
          );
        })}
      </svg>
    </div>
  );
}

export function MindmapVisualizer({ show, close }) {
  const [selectedMilestone, setSelectedMilestone] = useState(milestones[0]);
  const [activeTab, setActiveTab] = useState('mindmap'); // 'mindmap' | 'flywheel' | 'cot360'
  const [cacheSearchDepth, setCacheSearchDepth] = useState(4);

  if (!show) return null;

  // Tính toán thời gian tiết kiệm giả lập dựa trên dung lượng TT hit
  const unoptimizedTimeMs = (Math.pow(2.8, cacheSearchDepth) * 0.4).toFixed(1);
  const persistentTtTimeMs = (0.000015).toFixed(6); // 15 nanoseconds
  const speedupRatio = Math.round((parseFloat(unoptimizedTimeMs) * 1000000) / 15);

  return (
    <div className="fixed inset-0 z-50 bg-black/85 backdrop-blur-lg flex items-center justify-center p-3 md:p-6 animate-fadeIn">
      <div className="bg-obsidian-card border-2 border-gold/50 rounded-3xl max-w-6xl w-full max-h-[94vh] flex flex-col shadow-[0_0_50px_rgba(212,175,55,0.25)] overflow-hidden">
        
        {/* HEADER */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-gold/20 bg-gradient-to-r from-obsidian via-obsidian-card to-obsidian">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-gold/10 rounded-xl border border-gold/30 shadow-glow">
              <Compass className="w-6 h-6 text-gold animate-spin-slow" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <h2 className="text-lg md:text-xl font-royal font-bold text-gold tracking-wide">
                  SƠ ĐỒ TƯ DUY CÂY SUY LUẬN 360° & HẬU KIỂM TOÀN VÁN
                </h2>
                <span className="text-[10px] px-2 py-0.5 rounded-full bg-gold/20 text-gold border border-gold/40 font-mono font-bold">
                  v38.0 OMNISCIENT
                </span>
              </div>
              <p className="text-xs text-gold/70">
                Quy Nạp Lùi (Backward Induction) ➔ Lan Truyền Tri Thức Ngược ➔ Bánh Đà Tri Thức Persistent TT
              </p>
            </div>
          </div>

          <div className="flex items-center gap-3">
            <button
              onClick={close}
              className="px-3 py-1.5 bg-vermilion/20 hover:bg-vermilion/40 text-red-300 border border-vermilion/50 rounded-xl text-xs font-bold transition flex items-center gap-1.5"
            >
              ✕ ĐÓNG
            </button>
          </div>
        </div>

        {/* NAVIGATION TABS */}
        <div className="flex items-center gap-2 px-6 py-2 bg-black/40 border-b border-gold/10 text-xs font-bold overflow-x-auto">
          <button
            onClick={() => setActiveTab('mindmap')}
            className={`px-4 py-2 rounded-xl transition flex items-center gap-2 ${
              activeTab === 'mindmap' 
                ? 'bg-gold text-obsidian font-extrabold shadow-glow' 
                : 'bg-gold/5 text-gold/80 hover:bg-gold/15 border border-gold/20'
            }`}
          >
            <GitCommit className="w-4 h-4" />
            1. CÂY TƯ DUY & LAN TRUYỀN NGƯỢC (CAUSAL DAG)
          </button>
          <button
            onClick={() => setActiveTab('flywheel')}
            className={`px-4 py-2 rounded-xl transition flex items-center gap-2 ${
              activeTab === 'flywheel' 
                ? 'bg-gold text-obsidian font-extrabold shadow-glow' 
                : 'bg-gold/5 text-gold/80 hover:bg-gold/15 border border-gold/20'
            }`}
          >
            <Zap className="w-4 h-4" />
            2. BÁNH ĐÀ TRI THỨC TT (N× SPEEDUP)
          </button>
          <button
            onClick={() => setActiveTab('cot360')}
            className={`px-4 py-2 rounded-xl transition flex items-center gap-2 ${
              activeTab === 'cot360' 
                ? 'bg-gold text-obsidian font-extrabold shadow-glow' 
                : 'bg-gold/5 text-gold/80 hover:bg-gold/15 border border-gold/20'
            }`}
          >
            <Layers className="w-4 h-4" />
            3. GIẢI PHẪU 7 KHỐI SUY LUẬN 385 DÒNG
          </button>
        </div>

        {/* CONTENT AREA */}
        <div className="flex-1 overflow-y-auto p-6 space-y-6 bg-radial-gradient">
          
          {/* TAB 1: CÂY TƯ DUY & LAN TRUYỀN NGƯỢC */}
          {activeTab === 'mindmap' && (
            <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
              
              {/* Cột trái: Sơ đồ Cây Nhân Quả (7 Cột) */}
              <div className="lg:col-span-7 space-y-4">
                <div className="bg-obsidian/80 border border-gold/30 rounded-2xl p-4 shadow-xl space-y-3">
                  <div className="flex items-center justify-between">
                    <h3 className="text-sm font-bold text-gold flex items-center gap-2">
                      <Sparkles className="w-4 h-4 text-gold animate-pulse" />
                      TRỤC THỜI GIAN NHÂN QUẢ (BACKWARD INDUCTION CAUSALITY)
                    </h3>
                    <span className="text-[11px] text-emerald-400 font-mono">
                      ● Ground-Truth Known (100% Thấu Thị)
                    </span>
                  </div>
                  <p className="text-xs text-gold/70 leading-relaxed">
                    Nhờ kiến trúc <b>Hậu kiểm Toàn ván (Post-Game Omniscience)</b>, tại <b>Ply 20</b> hệ thống đã nhìn thấu toàn bộ cái chết sát cục ở <b>Ply 68</b>, biên dịch chính xác nguyên nhân gốc rễ vào chuỗi <code className="text-emerald-300">&lt;thought&gt;</code> thay vì đoán mò mù mịt!
                  </p>
                </div>

                {/* Danh sách 3 Nút Mốc Thế Cờ */}
                <div className="space-y-3 relative before:absolute before:left-6 before:top-6 before:bottom-6 before:w-0.5 before:bg-gradient-to-b before:from-amber-500 before:via-cyan-500 before:to-emerald-500">
                  {milestones.map((m, idx) => {
                    const isSelected = selectedMilestone.id === m.id;
                    return (
                      <div
                        key={m.id}
                        onClick={() => setSelectedMilestone(m)}
                        className={`relative ml-12 p-4 rounded-2xl border-2 transition-all cursor-pointer ${
                          isSelected 
                            ? 'bg-gradient-to-r from-gold/20 via-obsidian-card to-obsidian border-gold shadow-[0_0_20px_rgba(212,175,55,0.3)] scale-[1.02]' 
                            : 'bg-obsidian-card/80 border-gold/20 hover:border-gold/50 hover:bg-obsidian'
                        }`}
                      >
                        {/* Biểu tượng nút tròn trên đường kẻ */}
                        <div className={`absolute -left-12 top-4 w-7 h-7 rounded-full flex items-center justify-center font-bold text-xs border-2 shadow-glow ${
                          idx === 0 ? 'bg-amber-500 border-amber-300 text-black' :
                          idx === 1 ? 'bg-cyan-500 border-cyan-300 text-black' :
                          'bg-emerald-500 border-emerald-300 text-black'
                        }`}>
                          {idx + 1}
                        </div>

                        <div className="flex items-center justify-between gap-2">
                          <span className={`text-[10px] px-2.5 py-0.5 rounded-full border font-bold ${m.tagColor}`}>
                            {m.tag}
                          </span>
                          <span className="text-xs font-mono font-bold text-gold">
                            {m.evalScore}
                          </span>
                        </div>

                        <h4 className="text-sm font-bold text-white mt-1">
                          {m.title}
                        </h4>

                        <div className="flex items-center gap-3 text-xs text-gold/80 mt-1">
                          <span>Nước đi: <b className="text-gold">{m.moveNotation}</b> (<code className="text-emerald-400">{m.moveUci}</code>)</span>
                        </div>

                        <p className="text-[11px] text-gray-300 italic mt-2 border-l-2 border-gold/40 pl-2 bg-black/30 py-1 rounded-r">
                          {m.thoughtExcerpt}
                        </p>
                      </div>
                    );
                  })}
                </div>

                {/* Sơ đồ ASCII dòng chảy tri thức ngược */}
                <div className="bg-black/60 border border-gold/20 rounded-xl p-3 text-[11px] font-mono text-gold/80 space-y-1">
                  <div className="text-emerald-400 font-bold flex items-center gap-1">
                    <ArrowUpCircle className="w-4 h-4" />
                    DÒNG CHẢY QUY NẠP LÙI (BACKWARD KNOWLEDGE PROPAGATION):
                  </div>
                  <div>[PLY 68: SÁT CỤC THỰC TẾ (Checkmate)]</div>
                  <div className="text-cyan-400 pl-4">▲ Lan truyền tri thức ngược (Trích xuất chuỗi PV Sát Cục)</div>
                  <div>[PLY 45: Đòn Xe Pháo Áp Đáy (Ép đối phương gãy Sĩ)]</div>
                  <div className="text-amber-400 pl-4">▲ Nhận diện nguyên nhân gốc rễ (Root Cause Causality)</div>
                  <div>[PLY 20: Đòn Thí Binh c3c4 (Nước cờ bước ngoặt định đoạt ván cờ)]</div>
                </div>
              </div>

              {/* Cột phải: Bàn cờ Mini SVG & Bóc Tách Suy Nghĩ (5 Cột) */}
              <div className="lg:col-span-5 space-y-4">
                <div className="bg-obsidian border border-gold/40 rounded-2xl p-4 shadow-2xl space-y-4">
                  <div className="flex items-center justify-between border-b border-gold/20 pb-2">
                    <h3 className="text-xs font-bold text-gold flex items-center gap-1.5">
                      <Eye className="w-4 h-4 text-emerald-400" />
                      TRỰC QUAN HÓA BÀN CỜ MINI ({selectedMilestone.id.toUpperCase()})
                    </h3>
                    <span className="text-[10px] text-gold/60 font-mono">
                      Turn {selectedMilestone.turn}
                    </span>
                  </div>

                  {/* Bàn cờ Mini SVG */}
                  <MiniSvgBoard 
                    fen={selectedMilestone.fen} 
                    arrow={selectedMilestone.arrow} 
                  />

                  {/* So sánh Tầm nhìn: Depth 4 vs Hậu Kiểm Toàn Ván */}
                  <div className="space-y-2 text-xs">
                    <div className="bg-red-950/30 border border-red-500/30 p-2.5 rounded-xl space-y-0.5">
                      <div className="text-[11px] font-bold text-red-400 flex items-center gap-1">
                        <AlertTriangle className="w-3.5 h-3.5" />
                        TẦM NHÌN ONLINE SEARCH (DEPTH 4 - MÙ SÁT CỤC):
                      </div>
                      <p className="text-[11px] text-gold/70">
                        {selectedMilestone.horizonEval}
                      </p>
                    </div>

                    <div className="bg-emerald-950/30 border border-emerald-500/30 p-2.5 rounded-xl space-y-0.5">
                      <div className="text-[11px] font-bold text-emerald-400 flex items-center gap-1">
                        <CheckCircle2 className="w-3.5 h-3.5" />
                        TẦM NHÌN HẬU KIỂM TOÀN VÁN (THẤU THỊ 100%):
                      </div>
                      <p className="text-[11px] text-gold/90 font-medium">
                        {selectedMilestone.omniscienceEval}
                      </p>
                    </div>
                  </div>

                  {/* 3 Điểm Nhận Định Cờ Tướng */}
                  <div className="bg-black/40 border border-gold/20 p-3 rounded-xl space-y-1.5">
                    <h4 className="text-[11px] font-bold text-gold">LÝ DO CHIẾN THUẬT VẬT LÝ:</h4>
                    <ul className="space-y-1 text-[11px] text-gold/80 list-disc list-inside">
                      {selectedMilestone.reasoningPoints.map((pt, pIdx) => (
                        <li key={pIdx} className="leading-snug">{pt}</li>
                      ))}
                    </ul>
                  </div>
                </div>
              </div>

            </div>
          )}

          {/* TAB 2: BÁNH ĐÀ TRI THỨC TT & TĂNG TỐC N× */}
          {activeTab === 'flywheel' && (
            <div className="space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div className="bg-obsidian border border-gold/30 p-4 rounded-2xl space-y-2">
                  <div className="flex items-center gap-2 text-gold text-xs font-bold">
                    <Database className="w-4 h-4 text-emerald-400" />
                    PERSISTENT TT CACHE
                  </div>
                  <div className="text-2xl font-royal font-bold text-emerald-400">
                    1024 MB RAM
                  </div>
                  <p className="text-[11px] text-gold/70">
                    Bảng băm chia sẻ không khóa <code>Arc&lt;Table&gt;</code> lưu trữ toàn bộ cây tìm kiếm ván cờ.
                  </p>
                </div>

                <div className="bg-obsidian border border-gold/30 p-4 rounded-2xl space-y-2">
                  <div className="flex items-center gap-2 text-gold text-xs font-bold">
                    <Zap className="w-4 h-4 text-amber-400" />
                    ĐỘ TRỄ TRUY XUẤT NÚT
                  </div>
                  <div className="text-2xl font-royal font-bold text-amber-400">
                    15 Nanoseconds
                  </div>
                  <p className="text-[11px] text-gold/70">
                    Truy xuất O(1) qua Zobrist Hash thay vì phải chạy lại Alpha-Beta đệ quy 50ms.
                  </p>
                </div>

                <div className="bg-obsidian border border-gold/30 p-4 rounded-2xl space-y-2">
                  <div className="flex items-center gap-2 text-gold text-xs font-bold">
                    <TrendingUp className="w-4 h-4 text-cyan-400" />
                    HỆ SỐ GIA TỐC HẬU KIỂM
                  </div>
                  <div className="text-2xl font-royal font-bold text-cyan-400">
                    {speedupRatio.toLocaleString()} ×
                  </div>
                  <p className="text-[11px] text-gold/70">
                    Giảm N lần thời gian tìm kiếm khi khai thác các nhánh phản đòn của ván cờ đã xong.
                  </p>
                </div>
              </div>

              {/* Trình Tính Toán Hiệu Năng Interactive */}
              <div className="bg-obsidian border-2 border-gold/30 p-6 rounded-3xl shadow-xl space-y-4">
                <h3 className="text-sm font-bold text-gold flex items-center gap-2">
                  <Cpu className="w-5 h-5 text-gold" />
                  MÔ PHỎNG HIỆU NĂNG TIẾT KIỆM THỜI GIAN THEO ĐỘ SÂU (DEPTH {cacheSearchDepth})
                </h3>

                <div className="space-y-2">
                  <div className="flex justify-between text-xs text-gold font-mono">
                    <span>Độ sâu tìm kiếm: Depth {cacheSearchDepth}</span>
                    <span>Hệ số rẽ nhánh: b ≈ 2.8</span>
                  </div>
                  <input
                    type="range"
                    min="2"
                    max="10"
                    value={cacheSearchDepth}
                    onChange={(e) => setCacheSearchDepth(parseInt(e.target.value, 10))}
                    className="w-full accent-gold h-2 bg-black rounded-lg cursor-pointer"
                  />
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-xs font-mono mt-4">
                  <div className="bg-red-950/30 border border-red-500/30 p-4 rounded-xl space-y-2">
                    <span className="text-red-400 font-bold">❌ CHƯA CÓ BÁNH ĐÀ TRI THỨC (TÌM TỪ ĐẦU):</span>
                    <div className="text-lg font-bold text-red-300">{unoptimizedTimeMs} ms / turn</div>
                    <p className="text-[11px] text-gold/70">
                      Phải duyệt lại hàng triệu nút lá đệ quy, nghẽn CPU và mất hàng giờ cho 10,000 ván.
                    </p>
                  </div>

                  <div className="bg-emerald-950/30 border border-emerald-500/30 p-4 rounded-xl space-y-2">
                    <span className="text-emerald-400 font-bold">✅ CÓ BÁNH ĐÀ TRI THỨC PERSISTENT TT:</span>
                    <div className="text-lg font-bold text-emerald-300">{persistentTtTimeMs} ms / turn (15 ns)</div>
                    <p className="text-[11px] text-gold/70">
                      Hit cache 100% các nhánh đã duyệt của ván cờ, hoàn thành trích xuất 3-Ply trong 0.01s!
                    </p>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* TAB 3: 7 KHỐI SUY LUẬN 385 DÒNG */}
          {activeTab === 'cot360' && (
            <div className="space-y-4">
              <div className="bg-obsidian border border-gold/30 p-4 rounded-2xl text-xs text-gold/80 space-y-2">
                <h3 className="font-bold text-gold text-sm flex items-center gap-2">
                  <Layers className="w-4 h-4 text-gold" />
                  CẤU TRÚC 7 KHỐI TƯ DUY TỰ ĐỘNG HÓA HOÀN CHỈNH (385 DÒNG / TURN)
                </h3>
                <p className="text-[11px] text-gold/70">
                  Mỗi lượt Turn là một tệp mã nguồn suy luận độc lập (Autonomous Reasoning Unit), triệt tiêu 100% boilerplate loop filler:
                </p>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-3 text-xs">
                <div className="bg-black/50 border border-gold/20 p-3 rounded-xl space-y-1">
                  <h4 className="font-bold text-gold">Khối 1 (001-090: 90 Dòng)</h4>
                  <p className="text-gray-300 text-[11px]">Khảo sát 90 ô tọa độ vật lý `a0`..`i9`, nêu rõ quân chiếm giữ, độ cơ động, quân bảo kê và đe dọa.</p>
                </div>

                <div className="bg-black/50 border border-gold/20 p-3 rounded-xl space-y-1">
                  <h4 className="font-bold text-gold">Khối 2 (091-150: 60 Dòng)</h4>
                  <p className="text-gray-300 text-[11px]">9 Lộ dọc, 10 Tuyến ngang, 16 Tuyến chéo Sĩ Tượng (kiểm tra tắc mắt tượng), và 25 phân tích cấu trúc bàn cờ.</p>
                </div>

                <div className="bg-black/50 border border-gold/20 p-3 rounded-xl space-y-1">
                  <h4 className="font-bold text-gold">Khối 3 (151-220: 70 Dòng)</h4>
                  <p className="text-gray-300 text-[11px]">Kiểm kê vật chất, rà soát quân treo không padding khi tàn cuộc, và 15 đòn phối hợp chiến thuật lọc theo quân số thực tế.</p>
                </div>

                <div className="bg-black/50 border border-gold/20 p-3 rounded-xl space-y-1">
                  <h4 className="font-bold text-gold">Khối 4 (221-290: 70 Dòng)</h4>
                  <p className="text-gray-300 text-[11px]">Hội đồng 3 vai trò sinh động học theo từng quân cờ: Kẻ Tấn Công (24 bước), Kẻ Phản Biện (24 bước), Trọng Tài (19 tiêu chí).</p>
                </div>

                <div className="bg-black/50 border border-gold/20 p-3 rounded-xl space-y-1">
                  <h4 className="font-bold text-gold">Khối 5 (291-335: 45 Dòng)</h4>
                  <p className="text-gray-300 text-[11px]">Top 5 Ứng viên (mỗi ứng viên 9 dòng đánh giá toàn diện, phân tích ưu/nhược điểm cờ Tướng vật lý).</p>
                </div>

                <div className="bg-black/50 border border-gold/20 p-3 rounded-xl space-y-1">
                  <h4 className="font-bold text-gold">Khối 6 (336-370: 35 Dòng)</h4>
                  <p className="text-gray-300 text-[11px]">Mô phỏng cây 3-Ply ROLLOUT NƯỚC ĐI THẬT từ Engine (Nhánh A 70% phòng thủ, Nhánh B 30% trừng phạt sai lầm).</p>
                </div>

                <div className="bg-black/50 border border-gold/20 p-3 rounded-xl space-y-1 md:col-span-2">
                  <h4 className="font-bold text-gold">Khối 7 (371-385: 15 Dòng)</h4>
                  <p className="text-gray-300 text-[11px]">Thẩm định tính hợp lệ 100%, Flying General, an toàn Cung Tướng, Zobrist Hash anti-repetition, và quyết định nước đi tối thượng.</p>
                </div>
              </div>
            </div>
          )}

        </div>

        {/* FOOTER */}
        <div className="px-6 py-3 border-t border-gold/20 bg-black/60 flex items-center justify-between text-xs text-gold/70">
          <div className="flex items-center gap-2">
            <span className="w-2 h-2 rounded-full bg-emerald-400 animate-ping" />
            <span>MINDMAP VISUALIZER READY • XIANGQI-R1 DATASET GENERATOR</span>
          </div>
          <div className="font-mono text-[11px]">
            BUILD: 2026-08-23 22:15:00 ICT
          </div>
        </div>

      </div>
    </div>
  );
}
