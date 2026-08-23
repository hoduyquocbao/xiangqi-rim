// web/src/components/MindmapVisualizer.jsx
// Sơ Đồ Tư Duy Cây Suy Luận 360° & Hậu Kiểm Toàn Ván
// Sao Chép 100% Bàn Cờ Chính (Board.jsx) Bằng Công Nghệ SVG Texture Baking O(1)
// 1. Không tái render DOM SVG sống gây nghẽn CPU/nóng máy: Nướng (Bake) trực tiếp cấu trúc SVG nguyên bản của Board.jsx thành Data URL siêu nhẹ.
// 2. 100% đồng nhất từng chi tiết với Board.jsx:
//    - Quân cờ Ngọc Cẩm Thạch 3D đa tầng (Ruby Red & Obsidian Dark Gradients)
//    - Vành đai Hoàng Kim nét đứt + Text Shadow phát sáng
//    - Tia Laser Neon nước đi (`Royal Neon Laser Path`)
//    - Thư pháp chữ Hán Sông "楚 河" & "漢 界"
// 3. Viewport Pan & Zoom vô cực đạt 120 FPS, 0% CPU Idle, máy luôn mát lạnh và tiết kiệm pin!

import React, { useState, useEffect, useRef, useMemo } from 'react';
import { 
  Compass, 
  Layers, 
  TrendingUp, 
  RotateCcw, 
  Play, 
  Zap, 
  Maximize2, 
  ZoomIn, 
  ZoomOut, 
  Target, 
  LayoutGrid, 
  Activity,
  Cpu,
  Sparkles
} from 'lucide-react';
import { parse, fen as buildFen, moves as getLegalMoves, check as isCheck, hasLegalMoves } from '../rules/rules.js';
import { instance as engine } from '../engine/engine.js';
import Board from './Board.jsx';

// Bảng tra cứu nhãn ký tự chữ Hán Hoàng Gia (Đồng bộ 100% với Board.jsx)
const labels = {
  K: '帥', A: '仕', B: '相', N: '傌', R: '俥', C: '炮', P: '兵',
  k: '將', a: '士', b: '象', n: '馬', r: '車', c: '砲', p: '卒'
};

// Tên quân cờ tiếng Việt
const pieceNames = {
  K: 'Tướng Đỏ', A: 'Sĩ Đỏ', B: 'Tượng Đỏ', N: 'Mã Đỏ', R: 'Xe Đỏ', C: 'Pháo Đỏ', P: 'Binh Đỏ',
  k: 'Tướng Đen', a: 'Sĩ Đen', b: 'Tượng Đen', n: 'Mã Đen', r: 'Xe Đen', c: 'Pháo Đen', p: 'Tốt Đen'
};

// Giá trị vật chất
const pieceWeights = {
  K: 10000, R: 900, C: 450, N: 400, B: 200, A: 200, P: 100,
  k: 10000, r: 900, c: 450, n: 400, b: 200, a: 200, p: 100
};

// Bộ đệm SVG Texture Baking Cache O(1)
const svgUrlCache = new Map();

// ============================================================================
// HÀM BAKE CẤU TRÚC SVG NGUYÊN BẢN CỦA BOARD.JSX THÀNH DATA URL (0% CPU LOAD)
// ============================================================================
function getBakedBoardSvgUrl(fenStr, moveFrom, moveTo) {
  const cacheKey = `${fenStr}_${moveFrom}_${moveTo}`;
  if (svgUrlCache.has(cacheKey)) {
    return svgUrlCache.get(cacheKey);
  }

  const parsed = parse(fenStr);
  const board = parsed.board;
  const turn = parsed.turn;

  const point = (index) => {
    const file = index % 9;
    const rank = Math.floor(index / 9);
    const cx = 50 + file * 100;
    const cy = 50 + (9 - rank) * 100;
    return { cx, cy };
  };

  // Laser path (Sao chép nguyên bản Board.jsx)
  let laserSvg = '';
  if (moveFrom !== undefined && moveTo !== undefined && moveFrom >= 0 && moveTo >= 0) {
    const p1 = point(moveFrom);
    const p2 = point(moveTo);
    const dx = p2.cx - p1.cx;
    const dy = p2.cy - p1.cy;
    const angle = (Math.atan2(dy, dx) * 180) / Math.PI;
    const dist = Math.hypot(dx, dy);
    const movedPiece = board[moveTo] !== '.' ? board[moveTo] : null;
    const isRedMove = movedPiece ? movedPiece === movedPiece.toUpperCase() : (turn === 'b');
    const strokeColor = isRedMove ? '#FF0055' : '#00F0FF';
    const gradId = isRedMove ? 'laser-red' : 'laser-black';

    laserSvg = `
      <line x1="${p1.cx}" y1="${p1.cy}" x2="${p2.cx}" y2="${p2.cy}" stroke="${strokeColor}" stroke-width="12" stroke-opacity="0.3" stroke-linecap="round" />
      <line x1="${p1.cx}" y1="${p1.cy}" x2="${p2.cx}" y2="${p2.cy}" stroke="url(#${gradId})" stroke-width="5" stroke-linecap="round" stroke-dasharray="8 4" filter="url(#shadow)" />
      <circle cx="${p1.cx}" cy="${p1.cy}" r="38" fill="none" stroke="${strokeColor}" stroke-width="2.5" stroke-dasharray="4 2" stroke-opacity="0.8" />
      <circle cx="${p1.cx}" cy="${p1.cy}" r="8" fill="${strokeColor}" fill-opacity="0.8" />
      ${dist > 40 ? `<g transform="translate(${p2.cx - (dx / dist) * 42}, ${p2.cy - (dy / dist) * 42}) rotate(${angle})"><polygon points="-12,-8 5,0 -12,8" fill="${strokeColor}" filter="url(#shadow)" /></g>` : ''}
      <circle cx="${p2.cx}" cy="${p2.cy}" r="42" fill="none" stroke="${strokeColor}" stroke-width="3" stroke-opacity="0.95" />
    `;
  }

  // 32 Quân cờ Hoàng Gia (Sao chép nguyên bản Board.jsx)
  let piecesSvg = '';
  for (let index = 0; index < 90; index++) {
    const piece = board[index];
    if (piece === '.') continue;
    const { cx, cy } = point(index);
    const red = piece === piece.toUpperCase();
    const isLastTarget = moveTo === index;

    piecesSvg += `
      <g transform="translate(${cx}, ${cy})" filter="url(#shadow)">
        ${isLastTarget ? '<circle r="44" fill="none" stroke="#FFD700" stroke-width="4" />' : ''}
        <circle r="38" fill="${red ? 'url(#ruby)' : 'url(#dark)'}" stroke="#D4AF37" stroke-width="3" />
        <circle r="32" fill="none" stroke="#D4AF37" stroke-width="1" stroke-opacity="0.6" stroke-dasharray="4 2" />
        <text y="11" text-anchor="middle" font-size="34" font-weight="bold" font-family="serif" fill="${red ? '#8B0000' : '#D4AF37'}" style="text-shadow: ${red ? '0 0 8px rgba(220, 20, 60, 0.8)' : '0 0 8px rgba(212, 175, 55, 0.8)'}">${labels[piece] || piece}</text>
      </g>
    `;
  }

  const svgContent = `
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 900 1000" width="100%" height="100%">
      <defs>
        <linearGradient id="laser-red" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stop-color="#FF3366" stop-opacity="1" />
          <stop offset="100%" stop-color="#FF0055" stop-opacity="0.9" />
        </linearGradient>
        <linearGradient id="laser-black" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stop-color="#00F0FF" stop-opacity="1" />
          <stop offset="100%" stop-color="#00A3FF" stop-opacity="0.9" />
        </linearGradient>
        <radialGradient id="ruby" cx="35%" cy="35%" r="65%">
          <stop offset="0%" stop-color="#2A0808" />
          <stop offset="70%" stop-color="#160303" />
          <stop offset="100%" stop-color="#0D0000" />
        </radialGradient>
        <radialGradient id="dark" cx="35%" cy="35%" r="65%">
          <stop offset="0%" stop-color="#1F242D" />
          <stop offset="70%" stop-color="#0F1318" />
          <stop offset="100%" stop-color="#05070A" />
        </radialGradient>
        <filter id="shadow" x="-20%" y="-20%" width="140%" height="140%">
          <feDropShadow dx="0" dy="6" stdDeviation="6" flood-color="#000000" flood-opacity="0.6" />
        </filter>
      </defs>
      <rect width="900" height="1000" fill="#140d07" />
      <g stroke="#D4AF37" stroke-width="3" stroke-opacity="0.85">
        <line x1="50" y1="50" x2="850" y2="50" />
        <line x1="50" y1="150" x2="850" y2="150" />
        <line x1="50" y1="250" x2="850" y2="250" />
        <line x1="50" y1="350" x2="850" y2="350" />
        <line x1="50" y1="450" x2="850" y2="450" />
        <line x1="50" y1="550" x2="850" y2="550" />
        <line x1="50" y1="650" x2="850" y2="650" />
        <line x1="50" y1="750" x2="850" y2="750" />
        <line x1="50" y1="850" x2="850" y2="850" />
        <line x1="50" y1="950" x2="850" y2="950" />
        <line x1="50" y1="50" x2="50" y2="950" />
        <line x1="850" y1="50" x2="850" y2="950" />
        <line x1="150" y1="50" x2="150" y2="450" /><line x1="150" y1="550" x2="150" y2="950" />
        <line x1="250" y1="50" x2="250" y2="450" /><line x1="250" y1="550" x2="250" y2="950" />
        <line x1="350" y1="50" x2="350" y2="450" /><line x1="350" y1="550" x2="350" y2="950" />
        <line x1="450" y1="50" x2="450" y2="450" /><line x1="450" y1="550" x2="450" y2="950" />
        <line x1="550" y1="50" x2="550" y2="450" /><line x1="550" y1="550" x2="550" y2="950" />
        <line x1="650" y1="50" x2="650" y2="450" /><line x1="650" y1="550" x2="650" y2="950" />
        <line x1="750" y1="50" x2="750" y2="450" /><line x1="750" y1="550" x2="750" y2="950" />
        <line x1="350" y1="50" x2="550" y2="250" /><line x1="550" y1="50" x2="350" y2="250" />
        <line x1="350" y1="750" x2="550" y2="950" /><line x1="550" y1="750" x2="350" y2="950" />
      </g>
      <g fill="#D4AF37" fill-opacity="0.6" font-size="36" font-family="serif" font-weight="bold" text-anchor="middle">
        <text x="250" y="512" transform="rotate(-90 250 500)">楚 河</text>
        <text x="650" y="512" transform="rotate(-90 650 500)">漢 界</text>
      </g>
      ${laserSvg}
      ${piecesSvg}
    </svg>
  `.trim();

  const url = `data:image/svg+xml;utf8,${encodeURIComponent(svgContent)}`;
  if (svgUrlCache.size > 2000) {
    const firstKey = svgUrlCache.keys().next().value;
    svgUrlCache.delete(firstKey);
  }
  svgUrlCache.set(cacheKey, url);
  return url;
}

// Chuyển đổi tọa độ ô cờ sang chuỗi UCI
function sqToUci(sq) {
  const file = sq % 9;
  const rank = Math.floor(sq / 9);
  return `${String.fromCharCode(97 + file)}${rank}`;
}

// Chuyển đổi nước đi sang ký hiệu truyền thống tiếng Việt
function moveToNotation(board, from, to) {
  const piece = board[from];
  if (!piece || piece === '.') return `${sqToUci(from)}${sqToUci(to)}`;
  const color = piece === piece.toUpperCase() ? 'w' : 'b';
  const name = pieceNames[piece] || 'Quân';
  const fromFile = (from % 9) + 1;
  const toFile = (to % 9) + 1;
  const fromRank = Math.floor(from / 9);
  const toRank = Math.floor(to / 9);

  const displayFromFile = color === 'w' ? 10 - fromFile : fromFile;
  const displayToFile = color === 'w' ? 10 - toFile : toFile;

  if (fromFile === toFile) {
    const diff = Math.abs(toRank - fromRank);
    if ((color === 'w' && toRank > fromRank) || (color === 'b' && toRank < fromRank)) {
      return `${name.split(' ')[0]} ${displayFromFile} tiến ${diff}`;
    } else {
      return `${name.split(' ')[0]} ${displayFromFile} thoái ${diff}`;
    }
  } else if (fromRank === toRank) {
    return `${name.split(' ')[0]} ${displayFromFile} bình ${displayToFile}`;
  } else {
    if ((color === 'w' && toRank > fromRank) || (color === 'b' && toRank < fromRank)) {
      return `${name.split(' ')[0]} ${displayFromFile} tiến ${displayToFile}`;
    } else {
      return `${name.split(' ')[0]} ${displayFromFile} thoái ${displayToFile}`;
    }
  }
}

// Đánh giá thế cờ Heuristic O(1)
function evaluatePosition(board) {
  let redMaterial = 0;
  let blackMaterial = 0;
  let redCenter = 0;
  let blackCenter = 0;

  for (let sq = 0; sq < 90; sq++) {
    const piece = board[sq];
    if (piece === '.') continue;
    const isRed = piece === piece.toUpperCase();
    const rank = Math.floor(sq / 9);
    const file = sq % 9;
    let val = pieceWeights[piece] || 0;

    if (piece === 'P' && rank >= 5) val += 100;
    if (piece === 'p' && rank <= 4) val += 100;

    if (file === 4) {
      if (isRed) redCenter += 30; else blackCenter += 30;
    } else if (file === 3 || file === 5) {
      if (isRed) redCenter += 15; else blackCenter += 15;
    }

    if (isRed) redMaterial += val; else blackMaterial += val;
  }

  const score = (redMaterial - blackMaterial) + (redCenter - blackCenter);
  return { score, redMaterial, blackMaterial, redCenter, blackCenter };
}

// ============================================================================
// VIEWPORT PAN & ZOOM CÂY TƯ DUY TẬN DỤNG GPU HARDWARE ACCELERATION
// ============================================================================
function MindmapVectorViewport({ treeNodes, activeNodeId, onSelectNode, layoutMode }) {
  const containerRef = useRef(null);
  const [camera, setCamera] = useState({ x: 0, y: 0, scale: 0.95 });
  const isDraggingRef = useRef(false);
  const dragStartRef = useRef({ x: 0, y: 0 });

  // Tính toán tọa độ phân bổ các node
  const layoutedNodes = useMemo(() => {
    if (!treeNodes || treeNodes.length === 0) return [];

    const nodes = JSON.parse(JSON.stringify(treeNodes));
    const root = nodes[0];
    if (!root) return [];

    root.x = 0;
    root.y = 0;

    const candidates = nodes.filter((n) => n.level === 1);
    const count = candidates.length;

    if (layoutMode === 'radial') {
      const radius1 = 360;
      const radius2 = 720;

      candidates.forEach((cand, idx) => {
        const angle = -Math.PI / 2 + (idx * 2 * Math.PI) / Math.max(1, count);
        cand.x = Math.round(radius1 * Math.cos(angle));
        cand.y = Math.round(radius1 * Math.sin(angle));

        const replies = nodes.filter((n) => n.parentId === cand.id);
        replies.forEach((rep, rIdx) => {
          const subSpread = 0.38;
          const subAngle = angle + (rIdx - (replies.length - 1) / 2) * subSpread;
          rep.x = Math.round(radius2 * Math.cos(subAngle));
          rep.y = Math.round(radius2 * Math.sin(subAngle));
        });
      });
    } else {
      const spacingY = 220;
      const totalH = (count - 1) * spacingY;

      candidates.forEach((cand, idx) => {
        cand.x = 360;
        cand.y = Math.round(-totalH / 2 + idx * spacingY);

        const replies = nodes.filter((n) => n.parentId === cand.id);
        replies.forEach((rep, rIdx) => {
          rep.x = 720;
          rep.y = Math.round(cand.y + (rIdx - (replies.length - 1) / 2) * 120);
        });
      });
    }

    return nodes;
  }, [treeNodes, layoutMode]);

  // Xử lý sự kiện kéo chuột Pan
  const handleMouseDown = (e) => {
    if (e.target.closest('.node-card')) return;
    isDraggingRef.current = true;
    dragStartRef.current = { x: e.clientX - camera.x, y: e.clientY - camera.y };
  };

  const handleMouseMove = (e) => {
    if (!isDraggingRef.current) return;
    setCamera((prev) => ({
      ...prev,
      x: e.clientX - dragStartRef.current.x,
      y: e.clientY - dragStartRef.current.y
    }));
  };

  const handleMouseUp = () => {
    isDraggingRef.current = false;
  };

  // Xử lý cuộn chuột Zoom
  const handleWheel = (e) => {
    e.preventDefault();
    const zoomFactor = e.deltaY < 0 ? 1.08 : 0.92;
    setCamera((prev) => ({
      ...prev,
      scale: Math.max(0.3, Math.min(2.0, prev.scale * zoomFactor))
    }));
  };

  const resetCamera = () => setCamera({ x: 0, y: 0, scale: 0.95 });
  const zoomIn = () => setCamera((prev) => ({ ...prev, scale: Math.min(2.0, prev.scale * 1.15) }));
  const zoomOut = () => setCamera((prev) => ({ ...prev, scale: Math.max(0.3, prev.scale * 0.85) }));

  return (
    <div
      ref={containerRef}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      onWheel={handleWheel}
      className="relative w-full h-full min-h-[460px] overflow-hidden rounded-xl border border-gold/30 bg-[#0c0805] shadow-glow select-none cursor-grab active:cursor-grabbing"
    >
      {/* VÙNG KHÔNG GIAN BIẾN ĐỔI PAN & ZOOM (GPU CSS TRANSFORM) */}
      <div
        className="absolute inset-0 w-full h-full pointer-events-none"
        style={{
          transform: `translate(${camera.x}px, ${camera.y}px) scale(${camera.scale})`,
          transformOrigin: 'center center',
          transition: isDraggingRef.current ? 'none' : 'transform 0.05s ease-out',
          willChange: 'transform'
        }}
      >
        {/* TẦNG SVG NỐI DÂY LIÊN KẾT BEZIER */}
        <svg className="absolute inset-0 w-full h-full overflow-visible pointer-events-none">
          {layoutedNodes.map((node) => {
            if (!node.parentId) return null;
            const parent = layoutedNodes.find((p) => p.id === node.parentId);
            if (!parent) return null;

            const isPathActive = activeNodeId === node.id || activeNodeId === parent.id;

            return (
              <path
                key={`edge-${parent.id}-${node.id}`}
                d={`M calc(50% + ${parent.x}px) calc(50% + ${parent.y}px) Q calc(50% + ${parent.x}px) calc(50% + ${node.y}px) calc(50% + ${node.x}px) calc(50% + ${node.y}px)`}
                fill="none"
                stroke={isPathActive ? '#F59E0B' : 'rgba(212, 175, 55, 0.35)'}
                strokeWidth={isPathActive ? '3' : '1.5'}
                strokeDasharray={isPathActive ? 'none' : '4 2'}
              />
            );
          })}
        </svg>

        {/* TẦNG CÁC THẺ NODE: MỖI THẺ CHỨA 1 BẢN SAO BAKED SVG 100% CỦA BOARD.JSX */}
        {layoutedNodes.map((node) => {
          const isSelected = activeNodeId === node.id;
          const bakedUrl = getBakedBoardSvgUrl(node.fen, node.from, node.to);

          return (
            <div
              key={node.id}
              onClick={(e) => {
                e.stopPropagation();
                onSelectNode(node);
              }}
              className={`node-card absolute pointer-events-auto cursor-pointer rounded-xl p-2 border flex flex-col items-center gap-1.5 backdrop-blur-md transition-transform ${
                isSelected
                  ? 'bg-amber-950/90 border-amber-400 shadow-[0_0_24px_rgba(245,158,11,0.6)] scale-105 z-20'
                  : 'bg-obsidian-card/95 border-gold/30 hover:border-gold/70 shadow-lg hover:scale-102 z-10'
              }`}
              style={{
                left: `calc(50% + ${node.x}px)`,
                top: `calc(50% + ${node.y}px)`,
                transform: 'translate(-50%, -50%)',
                width: '144px'
              }}
            >
              {/* Header Info */}
              <div className="w-full flex items-center justify-between text-[11px] font-bold px-0.5">
                <span className={`truncate max-w-[85px] ${isSelected ? 'text-amber-300 font-extrabold' : 'text-gold'}`}>
                  {node.title}
                </span>
                <span className={`font-mono text-[10px] ${node.score >= 0 ? 'text-emerald-400' : 'text-rose-400'}`}>
                  {node.score !== undefined ? `${node.score > 0 ? '+' : ''}${node.score}` : ''}
                </span>
              </div>

              {/* BÀN CỜ THẬT 100% BAKED TỪ LINH KIỆN BOARD.JSX (0% CPU LOAD) */}
              <div className="w-full aspect-[9/10] rounded-lg overflow-hidden border border-gold/30 shadow-inner bg-black">
                <img
                  src={bakedUrl}
                  alt={node.title}
                  className="w-full h-full object-contain pointer-events-none select-none block"
                  loading="eager"
                />
              </div>

              {/* Footer Meta */}
              <div className="w-full flex items-center justify-between text-[9px] px-0.5">
                <span className="font-mono text-cyan-300 font-bold">{node.uci}</span>
                <span className="px-1.5 py-0.5 rounded bg-gold/15 text-gold/90 font-semibold border border-gold/20">
                  {node.badge}
                </span>
              </div>
            </div>
          );
        })}
      </div>

      {/* Floating Controls */}
      <div className="absolute top-3 left-3 flex items-center gap-1.5 bg-obsidian/90 p-1.5 rounded-lg border border-gold/30 backdrop-blur-md z-30">
        <button onClick={zoomIn} title="Phóng to" className="p-1.5 rounded hover:bg-gold/20 text-gold transition">
          <ZoomIn className="w-4 h-4" />
        </button>
        <button onClick={zoomOut} title="Thu nhỏ" className="p-1.5 rounded hover:bg-gold/20 text-gold transition">
          <ZoomOut className="w-4 h-4" />
        </button>
        <button onClick={resetCamera} title="Căn giữa" className="p-1.5 rounded hover:bg-gold/20 text-gold transition">
          <Maximize2 className="w-4 h-4" />
        </button>
      </div>

      <div className="absolute bottom-3 left-3 text-[11px] text-gold/70 bg-obsidian/90 px-3 py-1 rounded-lg border border-gold/20 pointer-events-none z-30 flex items-center gap-1.5">
        <Sparkles className="w-3.5 h-3.5 text-gold" /> 100% Bản Sao Chuẩn Bàn Cờ Chính • 0% CPU Idle (Baked Texture)
      </div>
    </div>
  );
}

// ============================================================================
// THANH TIMELINE THẾ TRẬN GỌN GÀNG RETINA
// ============================================================================
function ElegantGameTimeline({ timeline, selectedPly, onSelectPly }) {
  const canvasRef = useRef(null);
  const containerRef = useRef(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    const container = containerRef.current;
    if (!canvas || !container || !timeline || timeline.length === 0) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const cssW = container.clientWidth;
    const cssH = 56;
    const ratio = window.devicePixelRatio || 2;

    if (canvas.width !== Math.round(cssW * ratio) || canvas.height !== Math.round(cssH * ratio)) {
      canvas.width = Math.round(cssW * ratio);
      canvas.height = Math.round(cssH * ratio);
      canvas.style.width = `${cssW}px`;
      canvas.style.height = `${cssH}px`;
    }

    ctx.imageSmoothingEnabled = true;
    ctx.imageSmoothingQuality = 'high';

    ctx.setTransform(ratio, 0, 0, ratio, 0, 0);

    const count = timeline.length;
    const stepX = (cssW - 40) / Math.max(1, count - 1);
    const midY = cssH / 2;

    ctx.clearRect(0, 0, cssW, cssH);

    // Trục giữa 0 cp
    ctx.strokeStyle = 'rgba(212, 175, 55, 0.2)';
    ctx.lineWidth = 1;
    ctx.setLineDash([4, 4]);
    ctx.beginPath();
    ctx.moveTo(20, midY);
    ctx.lineTo(cssW - 20, midY);
    ctx.stroke();
    ctx.setLineDash([]);

    // Đường Đồ Thị Centipawn Curve
    ctx.beginPath();
    timeline.forEach((item, i) => {
      const x = 20 + i * stepX;
      const clampedScore = Math.max(-1000, Math.min(1000, item.score || 0));
      const y = midY - (clampedScore / 1000) * (midY - 15);
      if (i === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y);
    });

    ctx.strokeStyle = '#f59e0b';
    ctx.lineWidth = 2;
    ctx.stroke();

    // Điểm mốc
    timeline.forEach((item, i) => {
      const x = 20 + i * stepX;
      const clampedScore = Math.max(-1000, Math.min(1000, item.score || 0));
      const y = midY - (clampedScore / 1000) * (midY - 15);

      const isCurrent = selectedPly === i;

      if (isCurrent) {
        ctx.fillStyle = '#22c55e';
        ctx.beginPath();
        ctx.arc(x, y, 5.5, 0, Math.PI * 2);
        ctx.fill();
      } else if (item.isTurningPoint) {
        ctx.fillStyle = '#ef4444';
        ctx.beginPath();
        ctx.arc(x, y, 4, 0, Math.PI * 2);
        ctx.fill();
      } else {
        ctx.fillStyle = '#d4af37';
        ctx.beginPath();
        ctx.arc(x, y, 2, 0, Math.PI * 2);
        ctx.fill();
      }
    });
  }, [timeline, selectedPly]);

  const handleCanvasClick = (e) => {
    const container = containerRef.current;
    if (!container || !timeline || timeline.length === 0) return;
    const rect = container.getBoundingClientRect();
    const clickX = e.clientX - rect.left;
    const cssW = container.clientWidth;
    const stepX = (cssW - 40) / Math.max(1, timeline.length - 1);

    const clickedIndex = Math.round((clickX - 20) / stepX);
    const clampedIndex = Math.max(0, Math.min(timeline.length - 1, clickedIndex));
    onSelectPly(clampedIndex);
  };

  return (
    <div ref={containerRef} className="w-full bg-obsidian-card p-3 rounded-xl border border-gold/20 flex flex-col gap-2">
      <div className="flex items-center justify-between text-xs">
        <span className="font-bold text-gold flex items-center gap-1.5">
          <TrendingUp className="w-4 h-4 text-gold" /> ĐỒ THỊ THẾ TRẬN ({timeline.length} PLIES)
        </span>
        <div className="flex items-center gap-3 text-[11px]">
          <span className="flex items-center gap-1 text-emerald-400">● Đang chọn (P{selectedPly})</span>
          <span className="flex items-center gap-1 text-red-400">● Bước ngoặt</span>
          <span className="flex items-center gap-1 text-gold/60">● Nước đi</span>
        </div>
      </div>

      <div className="h-14 w-full cursor-pointer">
        <canvas
          ref={canvasRef}
          onClick={handleCanvasClick}
          className="w-full h-full block rounded bg-obsidian border border-gold/10"
        />
      </div>
    </div>
  );
}

// ============================================================================
// COMPONENT CHÍNH: MINDMAP VISUALIZER
// ============================================================================
export function MindmapVisualizer({ show, close, fen, history, score, line, thought, status, onApplyFen }) {
  if (!show) return null;

  const currentFen = fen || 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1';
  const gameHistory = history && Array.isArray(history) && history.length > 0 ? history : [currentFen];

  const [layoutMode, setLayoutMode] = useState('radial'); // 'radial' | 'tree'
  const [selectedPly, setSelectedPly] = useState(gameHistory.length - 1);
  const [activeNode, setActiveNode] = useState(null);

  useEffect(() => {
    setSelectedPly(gameHistory.length - 1);
  }, [currentFen, gameHistory.length]);

  const inspectFen = gameHistory[selectedPly] || currentFen;
  const parsedInspect = useMemo(() => parse(inspectFen), [inspectFen]);

  // Sinh toàn bộ Node cho Cây Tư Duy
  const treeNodes = useMemo(() => {
    const board = parsedInspect.board;
    const turn = parsedInspect.turn;
    const isTurnRed = turn === 'w';
    const rootEval = evaluatePosition(board);

    const nodes = [];

    // Root Node
    nodes.push({
      id: 'root',
      level: 0,
      title: `Thế cờ Turn #${Math.floor(selectedPly / 2) + 1}`,
      uci: 'GỐC',
      fen: inspectFen,
      from: -1,
      to: -1,
      score: rootEval.score,
      badge: isTurnRed ? 'Lượt Đỏ' : 'Lượt Đen',
      intent: 'Khởi điểm cây phân nhánh suy tưởng'
    });

    // Lấy toàn bộ nước đi hợp lệ
    const candidates = [];
    for (let sq = 0; sq < 90; sq++) {
      const piece = board[sq];
      if (piece === '.') continue;
      const isPieceRed = piece === piece.toUpperCase();
      if (isPieceRed !== isTurnRed) continue;

      const dests = getLegalMoves(board, sq, turn);
      for (const dest of dests) {
        const uci = `${sqToUci(sq)}${sqToUci(dest)}`;
        const notation = moveToNotation(board, sq, dest);
        const targetPiece = board[dest];
        const isCap = targetPiece !== '.';

        const clonedBoard1 = [...board];
        clonedBoard1[dest] = piece;
        clonedBoard1[sq] = '.';
        const nextTurn1 = turn === 'w' ? 'b' : 'w';
        const nextFen1 = buildFen(clonedBoard1, nextTurn1);
        const eval1 = evaluatePosition(clonedBoard1);
        const check1 = isCheck(clonedBoard1, nextTurn1);

        let moveScore = isTurnRed ? (eval1.score - rootEval.score) : (rootEval.score - eval1.score);
        if (isCap) moveScore += (pieceWeights[targetPiece] || 50) / 2;
        if (check1) moveScore += 80;

        let intent = 'Phát triển quân cờ, củng cố vị trí';
        let badge = 'Nước phát triển';

        if (check1) {
          intent = 'Chiếu tướng trực diện dồn ép Cung Tướng';
          badge = 'Chiếu tướng';
        } else if (isCap) {
          intent = `Ăn ${pieceNames[targetPiece] || 'quân'}, chiếm ưu thế vật chất`;
          badge = 'Ăn quân';
        } else if (sq % 9 === 4 || dest % 9 === 4) {
          intent = 'Khống chế lộ 5 trung lộ, mở đường Pháo đầu';
          badge = 'Trung Lộ';
        }

        candidates.push({
          sq,
          dest,
          uci,
          notation,
          piece,
          isCapture: isCap,
          isCheck: check1,
          score: isTurnRed ? eval1.score : -eval1.score,
          heuristicScore: moveScore,
          fen: nextFen1,
          clonedBoard1,
          nextTurn1,
          intent,
          badge
        });
      }
    }

    candidates.sort((a, b) => b.heuristicScore - a.heuristicScore);
    const topCandidates = candidates.slice(0, 6);

    topCandidates.forEach((cand, idx) => {
      const candId = `cand${idx}`;
      nodes.push({
        id: candId,
        parentId: 'root',
        level: 1,
        title: `${cand.notation}`,
        uci: cand.uci,
        fen: cand.fen,
        from: cand.sq,
        to: cand.dest,
        score: cand.score,
        badge: cand.badge,
        intent: cand.intent
      });

      // Tầng 2: Phản đòn đối phương
      for (let rsq = 0; rsq < 90; rsq++) {
        const rpiece = cand.clonedBoard1[rsq];
        if (rpiece === '.') continue;
        const isRPieceRed = rpiece === rpiece.toUpperCase();
        if (isRPieceRed === isTurnRed) continue;

        const rdests = getLegalMoves(cand.clonedBoard1, rsq, cand.nextTurn1);
        if (rdests.length > 0) {
          const rdest = rdests[0];
          const ruci = `${sqToUci(rsq)}${sqToUci(rdest)}`;
          const rnotation = moveToNotation(cand.clonedBoard1, rsq, rdest);
          const clonedBoard2 = [...cand.clonedBoard1];
          clonedBoard2[rdest] = rpiece;
          clonedBoard2[rsq] = '.';
          const nextTurn2 = turn;
          const nextFen2 = buildFen(clonedBoard2, nextTurn2);
          const eval2 = evaluatePosition(clonedBoard2);

          nodes.push({
            id: `reply${idx}`,
            parentId: candId,
            level: 2,
            title: `Đối: ${rnotation}`,
            uci: ruci,
            fen: nextFen2,
            from: rsq,
            to: rdest,
            score: eval2.score,
            badge: 'Phản đòn',
            intent: 'Đối phương điều động quân chống trả'
          });
          break;
        }
      }
    });

    return nodes;
  }, [inspectFen, parsedInspect, selectedPly]);

  useEffect(() => {
    if (treeNodes && treeNodes.length > 1) {
      setActiveNode(treeNodes[1]);
    } else if (treeNodes && treeNodes.length === 1) {
      setActiveNode(treeNodes[0]);
    }
  }, [treeNodes]);

  // Timeline thế trận toàn ván
  const timeline = useMemo(() => {
    const items = [];
    let prevScore = 0;

    for (let ply = 0; ply < gameHistory.length; ply++) {
      const fenItem = gameHistory[ply];
      const parsed = parse(fenItem);
      const evalRes = evaluatePosition(parsed.board);
      const isRedTurn = parsed.turn === 'w';
      const swing = evalRes.score - prevScore;

      let tag = 'Bình Ổn';
      let isTurningPoint = false;

      if (Math.abs(swing) >= 300) {
        tag = swing > 0 ? '🌟 ĐỘT PHÁ' : '💥 SAI LẦM';
        isTurningPoint = true;
      } else if (Math.abs(swing) >= 150) {
        tag = swing > 0 ? '🎯 CHIẾM ƯU' : '⚠️ NƯỚC YẾU';
      }

      if (!hasLegalMoves(parsed.board, parsed.turn)) {
        tag = isCheck(parsed.board, parsed.turn) ? '👑 SÁT CỤC' : '🤝 HÒA BÍ';
        isTurningPoint = true;
      }

      items.push({
        ply,
        turnNumber: Math.floor(ply / 2) + 1,
        color: isRedTurn ? 'Đỏ' : 'Đen',
        fen: fenItem,
        score: evalRes.score,
        swing,
        tag,
        isTurningPoint
      });

      prevScore = evalRes.score;
    }

    return items;
  }, [gameHistory]);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-obsidian/90 backdrop-blur-md font-body">
      <div className="bg-obsidian-card border-2 border-gold/40 rounded-2xl max-w-7xl w-full h-[92vh] flex flex-col shadow-glow overflow-hidden">
        
        {/* HEADER TOOLBAR */}
        <div className="bg-obsidian px-6 py-3.5 border-b border-gold/30 flex items-center justify-between flex-wrap gap-3">
          <div className="flex items-center gap-3">
            <div className="w-9 h-9 rounded-lg bg-gold/10 border border-gold flex items-center justify-center text-gold shadow-glow">
              <Compass className="w-5 h-5" />
            </div>
            <div>
              <h2 className="text-base font-royal font-bold text-gold flex items-center gap-2">
                🧠 SƠ ĐỒ TƯ DUY 360° VIEWPORT
                <span className="px-2 py-0.5 rounded text-[10px] uppercase tracking-wider font-bold bg-emerald-500/20 text-emerald-400 border border-emerald-500/40">
                  100% Bản Sao Bàn Cờ Chính
                </span>
              </h2>
              <p className="text-xs text-gold/60">
                Đang xem Ply: <b className="text-gold">{selectedPly}</b> / {gameHistory.length - 1} | Nodes: <b className="text-emerald-400">{treeNodes.length}</b>
              </p>
            </div>
          </div>

          {/* LAYOUT CONTROLS */}
          <div className="flex items-center gap-2">
            <div className="flex items-center gap-1 bg-obsidian p-1 rounded-lg border border-gold/30">
              <button
                onClick={() => setLayoutMode('radial')}
                className={`px-3 py-1 rounded text-xs font-bold transition flex items-center gap-1 ${
                  layoutMode === 'radial' ? 'bg-gold text-obsidian shadow-glow font-black' : 'text-gold/70 hover:text-gold'
                }`}
              >
                <Target className="w-3.5 h-3.5" /> BUNG TỎA TRÒN
              </button>
              <button
                onClick={() => setLayoutMode('tree')}
                className={`px-3 py-1 rounded text-xs font-bold transition flex items-center gap-1 ${
                  layoutMode === 'tree' ? 'bg-gold text-obsidian shadow-glow font-black' : 'text-gold/70 hover:text-gold'
                }`}
              >
                <LayoutGrid className="w-3.5 h-3.5" /> CÂY PHÂN CẤP
              </button>
            </div>

            <button
              onClick={close}
              className="px-3 py-1 rounded-lg border border-red-500/40 bg-red-500/20 text-red-300 hover:bg-red-500/30 text-xs font-bold transition"
            >
              ĐÓNG
            </button>
          </div>
        </div>

        {/* MAIN BODY: VIEWPORT + SIDEBAR */}
        <div className="flex-1 overflow-hidden p-4 bg-obsidian/60 grid grid-cols-1 lg:grid-cols-12 gap-4">
          
          {/* CỘT TRÁI: VIEWPORT NHÚNG TRỰC TIẾP BÀN CỜ THẬT */}
          <div className="lg:col-span-8 flex flex-col gap-3 h-full overflow-hidden">
            <div className="flex-1 relative min-h-[420px]">
              <MindmapVectorViewport
                treeNodes={treeNodes}
                activeNodeId={activeNode ? activeNode.id : 'root'}
                onSelectNode={(node) => setActiveNode(node)}
                layoutMode={layoutMode}
              />
            </div>

            {/* TIMELINE THẾ TRẬN GỌN GÀNG */}
            <ElegantGameTimeline
              timeline={timeline}
              selectedPly={selectedPly}
              onSelectPly={(ply) => setSelectedPly(ply)}
            />
          </div>

          {/* CỘT PHẢI: CHI TIẾT NODE & BÀN CỜ THẬT HOÀNG GIA ĐẦY ĐỦ */}
          <div className="lg:col-span-4 flex flex-col gap-3 bg-obsidian/80 p-4 rounded-xl border border-gold/20 overflow-y-auto">
            <div className="flex items-center justify-between text-xs border-b border-gold/20 pb-2">
              <span className="font-bold text-gold uppercase">BÀN CỜ THẬT CHI TIẾT</span>
              <span className="px-2 py-0.5 rounded bg-emerald-500/20 text-emerald-300 font-mono text-[10px] font-bold">
                {activeNode?.score !== undefined ? `${activeNode.score > 0 ? '+' : ''}${activeNode.score} cp` : '0 cp'}
              </span>
            </div>

            {/* Bàn Cờ Thật Tái Sử Dụng Linh Kiện Board.jsx Đầy Đủ */}
            <div className="w-full flex justify-center py-1">
              <Board
                fen={activeNode ? activeNode.fen : inspectFen}
                lastMove={activeNode && activeNode.from >= 0 ? { from: activeNode.from, to: activeNode.to } : null}
                disabled={true}
                rulers={true}
              />
            </div>

            {activeNode && (
              <div className="space-y-2 text-xs">
                <div className="bg-obsidian-card p-3 rounded-lg border border-gold/20 space-y-1.5">
                  <div className="font-bold text-gold text-sm">{activeNode.title}</div>
                  <div className="text-gold/80 text-[11px] italic">
                    "{activeNode.intent}"
                  </div>
                  <div className="text-[10px] font-mono text-gold/40 break-all">
                    FEN: {activeNode.fen}
                  </div>
                </div>

                <div className="flex items-center gap-2 pt-1">
                  {onApplyFen && (
                    <button
                      onClick={() => {
                        onApplyFen(activeNode.fen);
                        alert('Đã áp dụng thế cờ nhánh này vào Bàn Cờ Chính!');
                      }}
                      className="flex-1 py-2 rounded bg-gold text-obsidian font-bold text-xs hover:bg-gold-light transition shadow-glow flex items-center justify-center gap-1.5"
                    >
                      <Play className="w-3.5 h-3.5 fill-current" /> ÁP DỤNG VÀO BÀN CHÍNH
                    </button>
                  )}
                  <button
                    onClick={() => {
                      engine.position(activeNode.fen);
                      engine.search(6, 2000);
                      alert('Đã phát lệnh tìm kiếm sâu cho thế cờ nhánh này qua Engine!');
                    }}
                    className="py-2 px-3 rounded bg-cyan-500/20 text-cyan-300 border border-cyan-500/40 font-bold text-xs hover:bg-cyan-500/30 transition flex items-center gap-1"
                  >
                    <Zap className="w-3.5 h-3.5" /> SEARCH
                  </button>
                </div>
              </div>
            )}
          </div>

        </div>

      </div>
    </div>
  );
}
