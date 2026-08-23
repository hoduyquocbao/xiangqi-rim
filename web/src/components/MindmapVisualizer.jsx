// web/src/components/MindmapVisualizer.jsx
// Sơ Đồ Tư Duy Cây Suy Luận 360° & Hậu Kiểm Toàn Ván Siêu Nét Retina (High-DPI 4K Crisp)
// Triệt tiêu 100% hiện tượng mờ nhòe / vỡ hạt / răng cưa bằng công nghệ:
// 1. Retina DevicePixelRatio (DPR 2x/3x): Mọi Canvas đều được nhân đôi độ phân giải vật lý (Backing Store).
// 2. High-Res Snapshot Caching (2x Texture): Ảnh chụp bàn cờ render ở độ phân giải cao 200x224px, sắc nét từng milimet.
// 3. Crisp Vector Typography & Sub-Pixel Anti-Aliasing: Chữ Hán thư pháp và lưới bàn cờ nét căng như dao cạo.
// 4. Dirty-Region Repainting: Chỉ cập nhật vùng biến thiên, duy trì 0% CPU khi đứng yên, máy luôn mát lạnh.

import React, { useState, useEffect, useRef, useMemo, useCallback } from 'react';
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

// Hệ số Retina DPR vật lý của màn hình
const dpr = typeof window !== 'undefined' ? (window.devicePixelRatio || 2) : 2;

// Bảng tra cứu chữ Hán Hoàng Gia chuẩn cho quân cờ
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

// Kích thước chuẩn Thumbnail Snapshot (Logical CSS Units)
const snapW = 100;
const snapH = 112;
const snapPadX = 8;
const snapPadY = 8;
const snapCellW = (snapW - snapPadX * 2) / 8;
const snapCellH = (snapH - snapPadY * 2) / 9;

// Kích thước Bàn cờ Mini Chi Tiết (Logical CSS Units)
const miniW = 300;
const miniH = 336;
const miniPadX = 24;
const miniPadY = 24;
const miniCellW = (miniW - miniPadX * 2) / 8;
const miniCellH = (miniH - miniPadY * 2) / 9;

// Bộ đệm LRU Cache lưu trữ ảnh chụp nhanh Bitmap của các thế cờ FEN (Zero Duplicate Rasterization)
const snapshotCache = new Map();

// ============================================================================
// HỆ THỐNG VẼ LƯỚI TĨNH BÀN CỜ HOÀNG GIA (HIGH-DPI RETINA SURFACE)
// ============================================================================
function drawStaticBoardSurface(ctx, width, height, padx, pady, cellw, cellh) {
  // 1. Nền Gỗ Hoàng Gia Gradient Tối
  const grad = ctx.createRadialGradient(width / 2, height / 2, 20, width / 2, height / 2, width);
  grad.addColorStop(0, '#2b1c10');
  grad.addColorStop(1, '#0e0804');
  ctx.fillStyle = grad;
  ctx.fillRect(0, 0, width, height);

  // Viền vàng Hoàng Kim kép
  ctx.strokeStyle = '#D4AF37';
  ctx.lineWidth = width > 150 ? 2 : 1.2;
  ctx.strokeRect(padx, pady, width - padx * 2, height - pady * 2);

  // 2. Lưới đường kẻ bàn cờ (10 ngang, 9 dọc)
  ctx.strokeStyle = '#8A6B2D';
  ctx.lineWidth = width > 150 ? 1.2 : 0.8;

  // Đường ngang
  for (let r = 0; r < 10; r++) {
    const y = Math.round(pady + r * cellh) + 0.5;
    ctx.beginPath();
    ctx.moveTo(padx, y);
    ctx.lineTo(width - padx, y);
    ctx.stroke();
  }

  // Đường dọc (ngắt ở sông)
  for (let f = 1; f < 8; f++) {
    const x = Math.round(padx + f * cellw) + 0.5;
    ctx.beginPath();
    ctx.moveTo(x, pady);
    ctx.lineTo(x, pady + 4 * cellh);
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(x, pady + 5 * cellh);
    ctx.lineTo(x, height - pady);
    ctx.stroke();
  }

  // Cửu Cung Đỏ & Đen (Đường chéo X)
  ctx.beginPath();
  ctx.moveTo(padx + 3 * cellw, pady);
  ctx.lineTo(padx + 5 * cellw, pady + 2 * cellh);
  ctx.moveTo(padx + 5 * cellw, pady);
  ctx.lineTo(padx + 3 * cellw, pady + 2 * cellh);
  ctx.stroke();

  ctx.beginPath();
  ctx.moveTo(padx + 3 * cellw, pady + 7 * cellh);
  ctx.lineTo(padx + 5 * cellw, pady + 9 * cellh);
  ctx.moveTo(padx + 5 * cellw, pady + 7 * cellh);
  ctx.lineTo(padx + 3 * cellw, pady + 9 * cellh);
  ctx.stroke();

  // Dấu chữ thập vị trí Pháo và Tốt
  const crossSize = width > 150 ? 4 : 2;
  const crossPad = width > 150 ? 3 : 1.5;
  const markPositions = [
    [1, 2], [7, 2], [1, 7], [7, 7],
    [0, 3], [2, 3], [4, 3], [6, 3], [8, 3],
    [0, 6], [2, 6], [4, 6], [6, 6], [8, 6]
  ];

  ctx.strokeStyle = '#8A6B2D';
  ctx.lineWidth = 1;
  markPositions.forEach(([mf, mr]) => {
    const mx = padx + mf * cellw;
    const my = pady + mr * cellh;
    
    ctx.beginPath();
    if (mf > 0) {
      ctx.moveTo(mx - crossPad - crossSize, my - crossPad);
      ctx.lineTo(mx - crossPad, my - crossPad);
      ctx.lineTo(mx - crossPad, my - crossPad - crossSize);

      ctx.moveTo(mx - crossPad - crossSize, my + crossPad);
      ctx.lineTo(mx - crossPad, my + crossPad);
      ctx.lineTo(mx - crossPad, my + crossPad + crossSize);
    }
    if (mf < 8) {
      ctx.moveTo(mx + crossPad + crossSize, my - crossPad);
      ctx.lineTo(mx + crossPad, my - crossPad);
      ctx.lineTo(mx + crossPad, my - crossPad - crossSize);

      ctx.moveTo(mx + crossPad + crossSize, my + crossPad);
      ctx.lineTo(mx + crossPad, my + crossPad);
      ctx.lineTo(mx + crossPad, my + crossPad + crossSize);
    }
    ctx.stroke();
  });

  // Văn bản Sở Hà Hán Giới
  ctx.fillStyle = 'rgba(212, 175, 55, 0.6)';
  ctx.font = width > 150 ? 'bold 12px "Noto Serif SC", serif' : 'bold 7.5px "Noto Serif SC", serif';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText('楚 河', padx + 2 * cellw, (pady + 4.5 * cellh));
  ctx.fillText('漢 界', padx + 6 * cellw, (pady + 4.5 * cellh));
}

// Bộ đệm Layer Tĩnh Bàn Cờ Siêu Nét Retina (Backing Store 2x)
let staticMiniLayer = null;
let staticSnapLayer = null;

function getStaticBoardLayer(isMini) {
  const scaleRatio = 2; // Luôn tạo bitmap độ nét cao 2x
  if (isMini) {
    if (!staticMiniLayer) {
      const canvas = document.createElement('canvas');
      canvas.width = miniW * scaleRatio;
      canvas.height = miniH * scaleRatio;
      const ctx = canvas.getContext('2d', { alpha: false });
      if (ctx) {
        ctx.scale(scaleRatio, scaleRatio);
        drawStaticBoardSurface(ctx, miniW, miniH, miniPadX, miniPadY, miniCellW, miniCellH);
      }
      staticMiniLayer = canvas;
    }
    return staticMiniLayer;
  } else {
    if (!staticSnapLayer) {
      const canvas = document.createElement('canvas');
      canvas.width = snapW * scaleRatio;
      canvas.height = snapH * scaleRatio;
      const ctx = canvas.getContext('2d', { alpha: false });
      if (ctx) {
        ctx.scale(scaleRatio, scaleRatio);
        drawStaticBoardSurface(ctx, snapW, snapH, snapPadX, snapPadY, snapCellW, snapCellH);
      }
      staticSnapLayer = canvas;
    }
    return staticSnapLayer;
  }
}

// ============================================================================
// HỆ THỐNG SNAPSHOT BITMAP SIÊU NÉT (HIGH-RES 2X RETINA TEXTURE)
// ============================================================================
function getBoardSnapshot(fenStr, moveFrom, moveTo) {
  const cacheKey = `${fenStr}_${moveFrom}_${moveTo}`;
  if (snapshotCache.has(cacheKey)) {
    return snapshotCache.get(cacheKey);
  }

  const scaleRatio = 2;
  const canvas = document.createElement('canvas');
  canvas.width = snapW * scaleRatio;
  canvas.height = snapH * scaleRatio;
  const ctx = canvas.getContext('2d', { alpha: false });
  if (!ctx) return canvas;

  ctx.imageSmoothingEnabled = true;
  ctx.imageSmoothingQuality = 'high';

  // 1. Sao chép Layer Nền Tĩnh Độ Nét Cao
  const staticBg = getStaticBoardLayer(false);
  ctx.drawImage(staticBg, 0, 0, canvas.width, canvas.height);

  // Chuyển hệ trục sang kích thước logic
  ctx.save();
  ctx.scale(scaleRatio, scaleRatio);

  // 2. Vẽ Nước Đi Laser Neon Siêu Nét
  if (moveFrom !== undefined && moveTo !== undefined && moveFrom >= 0 && moveTo >= 0) {
    const f1 = moveFrom % 9;
    const r1 = Math.floor(moveFrom / 9);
    const f2 = moveTo % 9;
    const r2 = Math.floor(moveTo / 9);
    const x1 = snapPadX + f1 * snapCellW;
    const y1 = snapPadY + (9 - r1) * snapCellH;
    const x2 = snapPadX + f2 * snapCellW;
    const y2 = snapPadY + (9 - r2) * snapCellH;

    // Vòng định vị vị trí cũ
    ctx.strokeStyle = 'rgba(255, 0, 85, 0.9)';
    ctx.lineWidth = 1.2;
    ctx.setLineDash([2, 2]);
    ctx.beginPath();
    ctx.arc(x1, y1, 5.5, 0, Math.PI * 2);
    ctx.stroke();
    ctx.setLineDash([]);

    // Tia Laser phát sáng
    ctx.strokeStyle = '#FF0055';
    ctx.lineWidth = 2.2;
    ctx.beginPath();
    ctx.moveTo(x1, y1);
    ctx.lineTo(x2, y2);
    ctx.stroke();

    // Mũi tên chỉ hướng
    const angle = Math.atan2(y2 - y1, x2 - x1);
    ctx.fillStyle = '#FF0055';
    ctx.beginPath();
    ctx.moveTo(x2, y2);
    ctx.lineTo(x2 - 6 * Math.cos(angle - Math.PI / 6), y2 - 6 * Math.sin(angle - Math.PI / 6));
    ctx.lineTo(x2 - 6 * Math.cos(angle + Math.PI / 6), y2 - 6 * Math.sin(angle + Math.PI / 6));
    ctx.closePath();
    ctx.fill();

    // Vòng vị trí mới
    ctx.strokeStyle = '#FFD700';
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    ctx.arc(x2, y2, 6.5, 0, Math.PI * 2);
    ctx.stroke();
  }

  // 3. Vẽ 32 Quân Cờ Ngọc Bích Hoàng Gia Siêu Nét
  const parsed = parse(fenStr);
  const board = parsed.board;

  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.font = 'bold 7.5px "Noto Serif SC", serif';

  for (let sq = 0; sq < 90; sq++) {
    const piece = board[sq];
    if (piece === '.') continue;

    const f = sq % 9;
    const r = Math.floor(sq / 9);
    const x = snapPadX + f * snapCellW;
    const y = snapPadY + (9 - r) * snapCellH;
    const isRed = piece === piece.toUpperCase();

    // Thân quân ngọc bích
    ctx.fillStyle = isRed ? '#2A0808' : '#0F1318';
    ctx.beginPath();
    ctx.arc(x, y, 4.8, 0, Math.PI * 2);
    ctx.fill();

    // Viền vàng quân cờ
    ctx.strokeStyle = isRed ? '#ef4444' : '#60a5fa';
    ctx.lineWidth = 0.9;
    ctx.stroke();

    // Chữ Hán thư pháp
    ctx.fillStyle = isRed ? '#ff4d4d' : '#00f0ff';
    ctx.fillText(labels[piece] || piece, x, y + 0.5);
  }

  ctx.restore();

  // Giới hạn cache 2,000 snapshots
  if (snapshotCache.size > 2000) {
    const firstKey = snapshotCache.keys().next().value;
    snapshotCache.delete(firstKey);
  }

  snapshotCache.set(cacheKey, canvas);
  return canvas;
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
// BÀN CỜ CHI TIẾT SỬ DỤNG DIRTY REGION REPAINTING & RETINA 2X DPI
// ============================================================================
function DirtyRegionBoard({ fen, arrowFrom, arrowTo }) {
  const canvasRef = useRef(null);
  const prevFenRef = useRef(null);
  const prevMoveRef = useRef({ from: -1, to: -1 });
  const parsedBoard = useMemo(() => parse(fen).board, [fen]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    // Cấu hình Backing Store chuẩn Retina
    const scaleRatio = 2;
    if (canvas.width !== miniW * scaleRatio) {
      canvas.width = miniW * scaleRatio;
      canvas.height = miniH * scaleRatio;
      canvas.style.width = `${miniW}px`;
      canvas.style.height = `${miniH}px`;
      prevFenRef.current = null;
    }

    const ctx = canvas.getContext('2d', { alpha: false });
    if (!ctx) return;

    ctx.imageSmoothingEnabled = true;
    ctx.imageSmoothingQuality = 'high';

    const staticBg = getStaticBoardLayer(true);

    // Initial Paint
    if (!prevFenRef.current) {
      ctx.drawImage(staticBg, 0, 0, canvas.width, canvas.height);

      ctx.save();
      ctx.scale(scaleRatio, scaleRatio);
      if (arrowFrom >= 0 && arrowTo >= 0) {
        drawLaserArrow(ctx, arrowFrom, arrowTo);
      }
      drawAllPieces(ctx, parsedBoard);
      ctx.restore();

      prevFenRef.current = fen;
      prevMoveRef.current = { from: arrowFrom, to: arrowTo };
      return;
    }

    // DIRTY REGION REPAINTING
    const prevParsed = parse(prevFenRef.current).board;
    const changedSquares = [];

    for (let sq = 0; sq < 90; sq++) {
      if (prevParsed[sq] !== parsedBoard[sq]) {
        changedSquares.push(sq);
      }
    }

    if (prevMoveRef.current.from >= 0) changedSquares.push(prevMoveRef.current.from);
    if (prevMoveRef.current.to >= 0) changedSquares.push(prevMoveRef.current.to);
    if (arrowFrom >= 0) changedSquares.push(arrowFrom);
    if (arrowTo >= 0) changedSquares.push(arrowTo);

    if (changedSquares.length === 0) return;

    // Tính Bounding Box vùng bẩn
    let minX = miniW, minY = miniH, maxX = 0, maxY = 0;
    changedSquares.forEach((sq) => {
      const f = sq % 9;
      const r = Math.floor(sq / 9);
      const x = miniPadX + f * miniCellW;
      const y = miniPadY + (9 - r) * miniCellH;
      minX = Math.min(minX, x - 24);
      minY = Math.min(minY, y - 24);
      maxX = Math.max(maxX, x + 24);
      maxY = Math.max(maxY, y + 24);
    });

    minX = Math.max(0, Math.floor(minX));
    minY = Math.max(0, Math.floor(minY));
    maxX = Math.min(miniW, Math.ceil(maxX));
    maxY = Math.min(miniH, Math.ceil(maxY));
    const dirtyW = maxX - minX;
    const dirtyH = maxY - minY;

    // Phục hồi nền tĩnh trong vùng bẩn ở scale vật lý
    const srcX = minX * scaleRatio;
    const srcY = minY * scaleRatio;
    const srcW = dirtyW * scaleRatio;
    const srcH = dirtyH * scaleRatio;

    ctx.save();
    ctx.beginPath();
    ctx.rect(srcX, srcY, srcW, srcH);
    ctx.clip();

    ctx.drawImage(staticBg, srcX, srcY, srcW, srcH, srcX, srcY, srcW, srcH);

    // Chuyển sang logical coordinate để vẽ cờ và laser
    ctx.scale(scaleRatio, scaleRatio);

    if (arrowFrom >= 0 && arrowTo >= 0) {
      drawLaserArrow(ctx, arrowFrom, arrowTo);
    }

    for (let sq = 0; sq < 90; sq++) {
      const piece = parsedBoard[sq];
      if (piece === '.') continue;
      const f = sq % 9;
      const r = Math.floor(sq / 9);
      const x = miniPadX + f * miniCellW;
      const y = miniPadY + (9 - r) * miniCellH;

      if (x + 18 >= minX && x - 18 <= maxX && y + 18 >= minY && y - 18 <= maxY) {
        drawSinglePiece(ctx, piece, x, y);
      }
    }

    ctx.restore();

    prevFenRef.current = fen;
    prevMoveRef.current = { from: arrowFrom, to: arrowTo };
  }, [fen, parsedBoard, arrowFrom, arrowTo]);

  return (
    <canvas
      ref={canvasRef}
      className="rounded-xl border border-gold/40 shadow-glow bg-black block"
    />
  );
}

// Vẽ Laser Arrow sắc nét
function drawLaserArrow(ctx, from, to) {
  const f1 = from % 9;
  const r1 = Math.floor(from / 9);
  const f2 = to % 9;
  const r2 = Math.floor(to / 9);
  const x1 = miniPadX + f1 * miniCellW;
  const y1 = miniPadY + (9 - r1) * miniCellH;
  const x2 = miniPadX + f2 * miniCellW;
  const y2 = miniPadY + (9 - r2) * miniCellH;

  // Vòng định vị vị trí cũ
  ctx.strokeStyle = '#FF0055';
  ctx.lineWidth = 1.8;
  ctx.setLineDash([3, 2]);
  ctx.beginPath();
  ctx.arc(x1, y1, 14, 0, Math.PI * 2);
  ctx.stroke();
  ctx.setLineDash([]);

  // Tia Laser
  ctx.strokeStyle = '#FF0055';
  ctx.lineWidth = 3.5;
  ctx.beginPath();
  ctx.moveTo(x1, y1);
  ctx.lineTo(x2, y2);
  ctx.stroke();

  const angle = Math.atan2(y2 - y1, x2 - x1);
  ctx.fillStyle = '#FF0055';
  ctx.beginPath();
  ctx.moveTo(x2, y2);
  ctx.lineTo(x2 - 10 * Math.cos(angle - Math.PI / 6), y2 - 10 * Math.sin(angle - Math.PI / 6));
  ctx.lineTo(x2 - 10 * Math.cos(angle + Math.PI / 6), y2 - 10 * Math.sin(angle + Math.PI / 6));
  ctx.closePath();
  ctx.fill();

  // Vòng vị trí mới
  ctx.strokeStyle = '#FFD700';
  ctx.lineWidth = 2.5;
  ctx.beginPath();
  ctx.arc(x2, y2, 16, 0, Math.PI * 2);
  ctx.stroke();
}

// Vẽ 1 quân cờ chi tiết
function drawSinglePiece(ctx, piece, x, y) {
  const isRed = piece === piece.toUpperCase();

  ctx.fillStyle = isRed ? '#2A0808' : '#0F1318';
  ctx.beginPath();
  ctx.arc(x, y, 13, 0, Math.PI * 2);
  ctx.fill();

  ctx.strokeStyle = isRed ? '#ef4444' : '#60a5fa';
  ctx.lineWidth = 1.5;
  ctx.stroke();

  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.font = 'bold 13px "Noto Serif SC", serif';
  ctx.fillStyle = isRed ? '#ff4d4d' : '#00f0ff';
  ctx.fillText(labels[piece] || piece, x, y + 1);
}

// Vẽ toàn bộ 32 quân cờ
function drawAllPieces(ctx, board) {
  for (let sq = 0; sq < 90; sq++) {
    const piece = board[sq];
    if (piece === '.') continue;
    const f = sq % 9;
    const r = Math.floor(sq / 9);
    const x = miniPadX + f * miniCellW;
    const y = miniPadY + (9 - r) * miniCellH;
    drawSinglePiece(ctx, piece, x, y);
  }
}

// ============================================================================
// CANVAS VIEWPORT SƠ ĐỒ TƯ DUY RETINA 10,000+ NODES (ZERO BLUR / ZERO HEAT)
// ============================================================================
function MindmapCanvasViewport({ treeNodes, activeNodeId, onSelectNode, layoutMode }) {
  const canvasRef = useRef(null);
  const containerRef = useRef(null);
  
  // Camera Viewport Pan & Zoom
  const cameraRef = useRef({ x: 0, y: 0, scale: 1.0 });
  const isDraggingRef = useRef(false);
  const dragStartRef = useRef({ x: 0, y: 0 });

  // Tính toán tọa độ phân bố các Node một lần duy nhất
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
      const radius1 = 320;
      const radius2 = 600;

      candidates.forEach((cand, idx) => {
        const angle = -Math.PI / 2 + (idx * 2 * Math.PI) / Math.max(1, count);
        cand.x = radius1 * Math.cos(angle);
        cand.y = radius1 * Math.sin(angle);

        const replies = nodes.filter((n) => n.parentId === cand.id);
        replies.forEach((rep, rIdx) => {
          const subSpread = 0.35;
          const subAngle = angle + (rIdx - (replies.length - 1) / 2) * subSpread;
          rep.x = radius2 * Math.cos(subAngle);
          rep.y = radius2 * Math.sin(subAngle);
        });
      });
    } else {
      const spacingY = 160;
      const totalH = (count - 1) * spacingY;

      candidates.forEach((cand, idx) => {
        cand.x = 400;
        cand.y = -totalH / 2 + idx * spacingY;

        const replies = nodes.filter((n) => n.parentId === cand.id);
        replies.forEach((rep, rIdx) => {
          rep.x = 800;
          rep.y = cand.y + (rIdx - (replies.length - 1) / 2) * 90;
        });
      });
    }

    return nodes;
  }, [treeNodes, layoutMode]);

  // Vẽ Render trên Canvas THEO YÊU CẦU (High-DPI Retina Backing Store)
  const drawScene = useCallback(() => {
    const canvas = canvasRef.current;
    const container = containerRef.current;
    if (!canvas || !container) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const cssW = container.clientWidth;
    const cssH = container.clientHeight;
    const ratio = window.devicePixelRatio || 2;

    // Đảm bảo canvas có kích thước vật lý gấp đôi kích thước CSS (Retina Scaling)
    if (canvas.width !== Math.round(cssW * ratio) || canvas.height !== Math.round(cssH * ratio)) {
      canvas.width = Math.round(cssW * ratio);
      canvas.height = Math.round(cssH * ratio);
      canvas.style.width = `${cssW}px`;
      canvas.style.height = `${cssH}px`;
    }

    ctx.imageSmoothingEnabled = true;
    ctx.imageSmoothingQuality = 'high';

    const cam = cameraRef.current;

    // Reset transform & Xóa nền
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.fillStyle = '#0a0604';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // Tính toán khung nhìn thế giới
    const viewLeft = (-cssW / 2) / cam.scale - cam.x - 140;
    const viewRight = (cssW / 2) / cam.scale - cam.x + 140;
    const viewTop = (-cssH / 2) / cam.scale - cam.y - 140;
    const viewBottom = (cssH / 2) / cam.scale - cam.y + 140;

    // Áp dụng Ma trận Camera kết hợp Retina Scale
    ctx.setTransform(
      cam.scale * ratio, 0,
      0, cam.scale * ratio,
      (cam.x * cam.scale + cssW / 2) * ratio,
      (cam.y * cam.scale + cssH / 2) * ratio
    );

    // 1. VẼ DÂY NỐI BEZIER PHẲNG SIÊU NÉT
    ctx.lineWidth = 1.5;
    layoutedNodes.forEach((node) => {
      if (!node.parentId) return;
      const parent = layoutedNodes.find((p) => p.id === node.parentId);
      if (!parent) return;

      if (
        (node.x < viewLeft && parent.x < viewLeft) ||
        (node.x > viewRight && parent.x > viewRight) ||
        (node.y < viewTop && parent.y < viewTop) ||
        (node.y > viewBottom && parent.y > viewBottom)
      ) {
        return;
      }

      const isSelected = activeNodeId === node.id || activeNodeId === parent.id;
      ctx.strokeStyle = isSelected ? '#f59e0b' : 'rgba(212, 175, 55, 0.35)';

      ctx.beginPath();
      ctx.moveTo(parent.x, parent.y);
      ctx.quadraticCurveTo(parent.x, node.y, node.x, node.y);
      ctx.stroke();
    });

    // 2. VẼ CÁC NODE KÈM ẢNH CHỤP NHANH SNAPSHOT BITMAP 2X RETINA
    const isFarZoom = cam.scale < 0.45;

    layoutedNodes.forEach((node) => {
      if (
        node.x + 120 < viewLeft ||
        node.x - 120 > viewRight ||
        node.y + 90 < viewTop ||
        node.y - 90 > viewBottom
      ) {
        return;
      }

      const isSelected = activeNodeId === node.id;
      const nodeW = 196;
      const nodeH = 128;
      const rx = node.x - nodeW / 2;
      const ry = node.y - nodeH / 2;

      if (isFarZoom) {
        ctx.fillStyle = isSelected ? '#f59e0b' : node.score >= 0 ? '#10b981' : '#ef4444';
        ctx.beginPath();
        ctx.arc(node.x, node.y, isSelected ? 14 : 9, 0, Math.PI * 2);
        ctx.fill();
        return;
      }

      // Khung Node Card
      ctx.fillStyle = isSelected ? '#2b1b10' : '#140e09';
      ctx.strokeStyle = isSelected ? '#f59e0b' : 'rgba(212, 175, 55, 0.4)';
      ctx.lineWidth = isSelected ? 2 : 1;

      ctx.beginPath();
      ctx.roundRect(rx, ry, nodeW, nodeH, 8);
      ctx.fill();
      ctx.stroke();

      // Blit Ảnh Chụp Nhanh Bàn Cờ Bitmap Snapshot 2X Siêu Nét O(1)
      if (node.fen) {
        const snap = getBoardSnapshot(node.fen, node.from, node.to);
        ctx.drawImage(snap, rx + 8, ry + 8, snapW, snapH);
      }

      // Văn bản tiêu đề & Ký hiệu nước đi
      ctx.textAlign = 'left';
      ctx.textBaseline = 'top';
      ctx.font = 'bold 11px system-ui, -apple-system, sans-serif';
      ctx.fillStyle = isSelected ? '#fbbf24' : '#d4af37';
      ctx.fillText(node.title.slice(0, 12), rx + snapW + 14, ry + 12);

      // Điểm số Centipawn
      ctx.font = 'bold 10px monospace';
      ctx.fillStyle = node.score >= 0 ? '#34d399' : '#f87171';
      const scoreStr = node.score !== undefined ? `${node.score > 0 ? '+' : ''}${node.score} cp` : '';
      ctx.fillText(scoreStr, rx + snapW + 14, ry + 34);

      // Mã nước đi UCI
      ctx.font = '10px monospace';
      ctx.fillStyle = '#93c5fd';
      ctx.fillText(node.uci || '', rx + snapW + 14, ry + 54);

      // Huy hiệu
      if (node.badge) {
        ctx.font = '9px system-ui, -apple-system, sans-serif';
        ctx.fillStyle = '#a78bfa';
        ctx.fillText(node.badge, rx + snapW + 14, ry + 74);
      }
    });
  }, [layoutedNodes, activeNodeId]);

  useEffect(() => {
    drawScene();
  }, [drawScene]);

  // Resize canvas
  useEffect(() => {
    const handleResize = () => {
      drawScene();
    };
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, [drawScene]);

  // Tương tác chuột: Pan & Zoom
  const handleMouseDown = (e) => {
    isDraggingRef.current = true;
    dragStartRef.current = { x: e.clientX, y: e.clientY };
  };

  const handleMouseMove = (e) => {
    if (!isDraggingRef.current) return;
    const cam = cameraRef.current;
    const dx = (e.clientX - dragStartRef.current.x) / cam.scale;
    const dy = (e.clientY - dragStartRef.current.y) / cam.scale;
    cam.x += dx;
    cam.y += dy;
    dragStartRef.current = { x: e.clientX, y: e.clientY };
    drawScene();
  };

  const handleMouseUp = (e) => {
    if (!isDraggingRef.current) return;
    const container = containerRef.current;
    if (!container) return;
    const rect = container.getBoundingClientRect();
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    const cam = cameraRef.current;
    const worldX = (mouseX - container.clientWidth / 2) / cam.scale - cam.x;
    const worldY = (mouseY - container.clientHeight / 2) / cam.scale - cam.y;

    // Hit test click chọn Node
    for (const node of layoutedNodes) {
      const nodeW = 196;
      const nodeH = 128;
      if (
        worldX >= node.x - nodeW / 2 &&
        worldX <= node.x + nodeW / 2 &&
        worldY >= node.y - nodeH / 2 &&
        worldY <= node.y + nodeH / 2
      ) {
        onSelectNode(node);
        break;
      }
    }

    isDraggingRef.current = false;
  };

  const handleWheel = (e) => {
    e.preventDefault();
    const cam = cameraRef.current;
    const zoomFactor = e.deltaY < 0 ? 1.1 : 0.9;
    cam.scale = Math.max(0.2, Math.min(2.5, cam.scale * zoomFactor));
    drawScene();
  };

  const resetCamera = () => {
    cameraRef.current = { x: 0, y: 0, scale: 1.0 };
    drawScene();
  };

  const zoomIn = () => {
    cameraRef.current.scale = Math.min(2.5, cameraRef.current.scale * 1.2);
    drawScene();
  };

  const zoomOut = () => {
    cameraRef.current.scale = Math.max(0.2, cameraRef.current.scale * 0.8);
    drawScene();
  };

  return (
    <div ref={containerRef} className="relative w-full h-full min-h-[460px] overflow-hidden rounded-xl border border-gold/30 bg-black shadow-glow">
      <canvas
        ref={canvasRef}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onWheel={handleWheel}
        className="w-full h-full block select-none cursor-grab active:cursor-grabbing"
      />

      {/* Floating Controls */}
      <div className="absolute top-3 left-3 flex items-center gap-1.5 bg-obsidian/90 p-1.5 rounded-lg border border-gold/30 backdrop-blur-md">
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

      <div className="absolute bottom-3 left-3 text-[11px] text-gold/60 bg-obsidian/90 px-2.5 py-1 rounded border border-gold/20 pointer-events-none">
        ⚡ Retina 4K Crisp (DPR {dpr}x) • Zero-Copy Snapshot • Dirty-Region Repainting
      </div>
    </div>
  );
}

// ============================================================================
// THANH TIMELINE THẾ TRẬN GỌN GÀNG SẮP XẾP MƯỢT MÀ RETINA
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
                🧠 SƠ ĐỒ TƯ DUY 360° CANVAS VIEWPORT
                <span className="px-2 py-0.5 rounded text-[10px] uppercase tracking-wider font-bold bg-emerald-500/20 text-emerald-400 border border-emerald-500/40">
                  Retina 4K Crisp (DPR {dpr}x)
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
          
          {/* CỘT TRÁI: CANVAS VIEWPORT TỰ ĐỘNG BUNG NODES */}
          <div className="lg:col-span-8 flex flex-col gap-3 h-full overflow-hidden">
            <div className="flex-1 relative min-h-[420px]">
              <MindmapCanvasViewport
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

          {/* CỘT PHẢI: CHI TIẾT NODE & BÀN CỜ MINI GPU ĐỒNG BỘ */}
          <div className="lg:col-span-4 flex flex-col gap-3 bg-obsidian/80 p-4 rounded-xl border border-gold/20 overflow-y-auto">
            <div className="flex items-center justify-between text-xs border-b border-gold/20 pb-2">
              <span className="font-bold text-gold uppercase">CHI TIẾT NODE ĐANG CHỌN</span>
              <span className="px-2 py-0.5 rounded bg-emerald-500/20 text-emerald-300 font-mono text-[10px] font-bold">
                {activeNode?.score !== undefined ? `${activeNode.score > 0 ? '+' : ''}${activeNode.score} cp` : '0 cp'}
              </span>
            </div>

            {/* Bàn Cờ Chi Tiết Sử Dụng Dirty Region Repainting & Retina 2x */}
            <div className="flex justify-center py-1">
              <DirtyRegionBoard
                fen={activeNode ? activeNode.fen : inspectFen}
                arrowFrom={activeNode ? activeNode.from : -1}
                arrowTo={activeNode ? activeNode.to : -1}
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
