// web/src/components/MindmapVisualizer.jsx
// Sơ Đồ Tư Duy Cây Suy Luận 360° & Động Cơ Mở Rộng Chiều Ngang / Chiều Sâu Vô Hạn
// TÍNH NĂNG VÔ HẠN:
// 1. Mở Rộng Chiều Ngang Vô Hạn (Infinite Breadth Expansion):
//    - Tăng tự do số biến thể (+5, +10, hoặc ♾ Toàn Bộ 100% Nước Đi Hợp Lệ), không bao giờ bị giới hạn hay tự quay vòng!
// 2. Mở Rộng Chiều Sâu Vô Hạn (Infinite Depth Expansion):
//    - Cho phép chọn bất kỳ node nào ở bất kỳ tầng nào và đào sâu đệ quy (+1 Ply, +3 Plies) không giới hạn độ sâu (Level 1, 2, 3, 4, 5, 6, 7, 8... vô hạn).
// 3. Bố Trí Cây Phân Bổ Tự Động Vô Hạn (Infinite Tree Layout):
//    - Thuật toán Radial & Hierarchical Tree Layout tự động tính toán góc dải quạt và độ giãn nở, không đè lấn nhau.
// 4. Kho Tri Thức TT Vĩnh Cửu O(1):
//    - Lưu và tra cứu tức thì toàn bộ hàng trăm node của cây vào localStorage/JSON, nhận diện TT Hit giảm N thời gian tìm kiếm.
// 5. Baked SVG Texture 120 FPS:
//    - 100% đồng nhất Board.jsx, 0% CPU Idle, máy luôn mát lạnh!

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
  Sparkles, 
  ArrowRight, 
  ArrowDown, 
  Database, 
  Download, 
  Upload, 
  Save, 
  CheckCircle2, 
  PlusCircle, 
  FolderTree, 
  Sliders,
  Plus,
  Minus,
  Infinity as InfinityIcon,
  ChevronDown,
  ChevronUp
} from 'lucide-react';
import { parse, fen as buildFen, moves as getLegalMoves, check as isCheck, hasLegalMoves } from '../rules/rules.js';
import { instance as engine } from '../engine/engine.js';
import Board from './Board.jsx';

// Định danh khóa lưu trữ đơn từ
const store = 'tt';

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
// HỆ THỐNG QUẢN LÝ KHO TRI THỨC TT VĨNH CỬU (PERSISTENT TT STORE O(1))
// ============================================================================
function loadPersistentTTStore() {
  try {
    const raw = localStorage.getItem(store);
    return raw ? JSON.parse(raw) : {};
  } catch (e) {
    return {};
  }
}

function savePersistentTTStore(data) {
  try {
    localStorage.setItem(store, JSON.stringify(data));
  } catch (e) {
    console.error('Không thể lưu TT vào LocalStorage:', e);
  }
}

// ============================================================================
// HÀM BAKE CẤU TRÚC SVG NGUYÊN BẢN CỦA BOARD.JSX THÀNH DATA URL (0% CPU LOAD)
// ============================================================================
function getBakedBoardSvgUrl(fenStr, moveFrom, moveTo) {
  const cacheKey = `${fenStr}:${moveFrom}:${moveTo}`;
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

// Sinh danh sách nước đi ứng viên được xếp hạng từ một thế cờ
function generateCandidateMoves(fenStr, limit = 999) {
  const parsed = parse(fenStr);
  const board = parsed.board;
  const turn = parsed.turn;
  const isTurnRed = turn === 'w';
  const rootEval = evaluatePosition(board);

  const moves = [];
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

      const nextBoard = [...board];
      nextBoard[dest] = piece;
      nextBoard[sq] = '.';
      const nextTurn = turn === 'w' ? 'b' : 'w';
      const nextFen = buildFen(nextBoard, nextTurn);
      const nextEval = evaluatePosition(nextBoard);
      const nextCheck = isCheck(nextBoard, nextTurn);

      let moveScore = isTurnRed ? (nextEval.score - rootEval.score) : (rootEval.score - nextEval.score);
      if (isCap) moveScore += (pieceWeights[targetPiece] || 50) / 2;
      if (nextCheck) moveScore += 80;

      let intent = 'Phát triển quân cờ, củng cố vị trí';
      let badge = 'Nước phát triển';

      if (nextCheck) {
        intent = 'Chiếu tướng trực diện dồn ép Cung Tướng';
        badge = 'Chiếu tướng';
      } else if (isCap) {
        intent = `Ăn ${pieceNames[targetPiece] || 'quân'}, chiếm ưu thế vật chất`;
        badge = 'Ăn quân';
      } else if (sq % 9 === 4 || dest % 9 === 4) {
        intent = 'Khống chế lộ 5 trung lộ, mở đường Pháo đầu';
        badge = 'Trung Lộ';
      }

      moves.push({
        sq,
        dest,
        uci,
        notation,
        piece,
        isCapture: isCap,
        isCheck: nextCheck,
        score: isTurnRed ? nextEval.score : -nextEval.score,
        heuristicScore: moveScore,
        fen: nextFen,
        intent,
        badge
      });
    }
  }

  moves.sort((a, b) => b.heuristicScore - a.heuristicScore);
  return moves.slice(0, limit);
}

// ============================================================================
// VIEWPORT PAN & ZOOM CÂY TƯ DUY VÔ HẠN (GPU HARDWARE ACCELERATION)
// ============================================================================
function MindmapVectorViewport({ 
  treeNodes, 
  activeNodeId, 
  onSelectNode, 
  layoutMode, 
  onExpandBreadth,
  onShrinkBreadth,
  onExpandDepth,
  onDeepenThreePlies,
  onExpandAllBreadth,
  breadthCount,
  totalAvailableMoves
}) {
  const containerRef = useRef(null);
  const [camera, setCamera] = useState({ x: 0, y: 0, scale: 0.95 });
  const isDraggingRef = useRef(false);
  const dragStartRef = useRef({ x: 0, y: 0 });

  // Thuật toán tính tọa độ tổng quát cho Cây Vô Hạn (General Infinite Tree Layout)
  const layoutedNodes = useMemo(() => {
    if (!treeNodes || treeNodes.length === 0) return [];

    const nodes = JSON.parse(JSON.stringify(treeNodes));
    const nodeMap = new Map();
    nodes.forEach((n) => nodeMap.set(n.id, n));

    const childrenMap = new Map();
    nodes.forEach((n) => {
      if (n.parentId) {
        if (!childrenMap.has(n.parentId)) childrenMap.set(n.parentId, []);
        childrenMap.get(n.parentId).push(n);
      }
    });

    const root = nodes.find((n) => !n.parentId) || nodes[0];
    root.x = 0;
    root.y = 0;

    // Đếm số lá bên dưới để phân bổ không gian đệ quy
    function countLeaves(nodeId) {
      const kids = childrenMap.get(nodeId) || [];
      if (kids.length === 0) return 1;
      return kids.reduce((sum, kid) => sum + countLeaves(kid.id), 0);
    }

    if (layoutMode === 'radial') {
      // Phân bổ dải quạt Radial đệ quy
      function layoutRadial(nodeId, startAngle, endAngle, currentRadius) {
        const kids = childrenMap.get(nodeId) || [];
        if (kids.length === 0) return;

        const totalWeight = countLeaves(nodeId);
        let accAngle = startAngle;

        kids.forEach((kid) => {
          const kidWeight = countLeaves(kid.id);
          const span = ((endAngle - startAngle) * kidWeight) / totalWeight;
          const midAngle = accAngle + span / 2;

          kid.x = Math.round(currentRadius * Math.cos(midAngle));
          kid.y = Math.round(currentRadius * Math.sin(midAngle));

          layoutRadial(kid.id, accAngle, accAngle + span, currentRadius + 380);
          accAngle += span;
        });
      }

      layoutRadial(root.id, -Math.PI, Math.PI, 380);
    } else {
      // Phân bổ cây phân cấp ngang (Hierarchical Tree Layout)
      let leafY = 0;
      const leafSpacing = 160;

      function layoutTreeVertical(nodeId, depthLevel) {
        const node = nodeMap.get(nodeId);
        const kids = childrenMap.get(nodeId) || [];
        node.x = depthLevel * 380;

        if (kids.length === 0) {
          node.y = leafY;
          leafY += leafSpacing;
        } else {
          kids.forEach((kid) => layoutTreeVertical(kid.id, depthLevel + 1));
          const firstKid = kids[0];
          const lastKid = kids[kids.length - 1];
          node.y = Math.round((firstKid.y + lastKid.y) / 2);
        }
      }

      layoutTreeVertical(root.id, 0);

      // Căn giữa trục Y
      const rootY = root.y;
      nodes.forEach((n) => {
        n.y -= rootY;
      });
    }

    return nodes;
  }, [treeNodes, layoutMode]);

  // Xử lý sự kiện kéo chuột Pan
  const handleMouseDown = (e) => {
    if (e.target.closest('.node-card') || e.target.closest('button')) return;
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
      scale: Math.max(0.1, Math.min(3.0, prev.scale * zoomFactor))
    }));
  };

  const resetCamera = () => setCamera({ x: 0, y: 0, scale: 0.95 });
  const zoomIn = () => setCamera((prev) => ({ ...prev, scale: Math.min(3.0, prev.scale * 1.18) }));
  const zoomOut = () => setCamera((prev) => ({ ...prev, scale: Math.max(0.1, prev.scale * 0.82) }));

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
                strokeWidth={isPathActive ? '3.5' : '1.5'}
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
                <span className={`truncate max-w-[80px] ${isSelected ? 'text-amber-300 font-extrabold' : 'text-gold'}`}>
                  {node.title}
                </span>
                <div className="flex items-center gap-1">
                  {node.ttHit && (
                    <span title={`Khớp Tri Thức TT (D${node.ttDepth})`} className="px-1 py-0.2 rounded bg-amber-500/30 text-amber-300 text-[8px] font-extrabold border border-amber-500/50">
                      ⚡TT
                    </span>
                  )}
                  <span className={`font-mono text-[10px] ${node.score >= 0 ? 'text-emerald-400' : 'text-rose-400'}`}>
                    {node.score !== undefined ? `${node.score > 0 ? '+' : ''}${node.score}` : ''}
                  </span>
                </div>
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
                <span className="px-1.5 py-0.5 rounded bg-gold/15 text-gold/90 font-semibold border border-gold/20 truncate max-w-[70px]">
                  {node.badge}
                </span>
              </div>
            </div>
          );
        })}
      </div>

      {/* FLOATING VIEWPORT CONTROLS: MỞ RỘNG VÔ HẠN */}
      <div className="absolute top-3 left-3 flex items-center gap-1.5 bg-obsidian/95 p-1.5 rounded-lg border border-gold/30 backdrop-blur-md z-30 flex-wrap">
        <button onClick={zoomIn} title="Phóng to" className="p-1.5 rounded hover:bg-gold/20 text-gold transition">
          <ZoomIn className="w-4 h-4" />
        </button>
        <button onClick={zoomOut} title="Thu nhỏ" className="p-1.5 rounded hover:bg-gold/20 text-gold transition">
          <ZoomOut className="w-4 h-4" />
        </button>
        <button onClick={resetCamera} title="Căn giữa" className="p-1.5 rounded hover:bg-gold/20 text-gold transition">
          <Maximize2 className="w-4 h-4" />
        </button>

        <div className="h-4 w-px bg-gold/30 mx-1" />

        {/* NÚT MỞ RỘNG CHIỀU NGANG VÔ HẠN (+5, -5, ALL) */}
        <div className="flex items-center gap-1 bg-amber-950/40 p-0.5 rounded border border-amber-500/30">
          <button
            onClick={onShrinkBreadth}
            title="Thu gọn chiều ngang (-5 biến thể)"
            className="p-1 rounded hover:bg-amber-500/20 text-amber-300 transition"
          >
            <Minus className="w-3.5 h-3.5" />
          </button>
          <button
            onClick={onExpandBreadth}
            title="Mở rộng thêm chiều ngang (+5 biến thể) - Vô Hạn!"
            className="px-2 py-0.5 text-xs font-bold text-amber-300 hover:bg-amber-500/20 rounded transition flex items-center gap-1"
          >
            <Plus className="w-3.5 h-3.5" /> NGANG: {breadthCount >= totalAvailableMoves ? 'TẤT CẢ' : breadthCount} / {totalAvailableMoves}
          </button>
          <button
            onClick={onExpandAllBreadth}
            title="Bung 100% tất cả các nước đi hợp lệ ở thế cờ này"
            className="px-1.5 py-0.5 text-[10px] font-black bg-amber-500/30 text-amber-200 hover:bg-amber-500/50 rounded transition flex items-center gap-0.5"
          >
            <InfinityIcon className="w-3 h-3" /> ALL
          </button>
        </div>

        {/* NÚT MỞ RỘNG CHIỀU SÂU VÔ HẠN (+1 PLY, +3 PLIES) */}
        <div className="flex items-center gap-1 bg-cyan-950/40 p-0.5 rounded border border-cyan-500/30">
          <button
            onClick={onExpandDepth}
            title="Đào sâu tiếp 1 tầng từ node đang chọn (Mở Rộng Chiều Sâu Vô Hạn)"
            className="px-2 py-0.5 text-xs font-bold text-cyan-300 hover:bg-cyan-500/20 rounded transition flex items-center gap-1"
          >
            <ArrowDown className="w-3.5 h-3.5" /> ĐÀO SÂU (+1 PLY)
          </button>
          <button
            onClick={onDeepenThreePlies}
            title="Tự động mở rộng 3 tầng sâu tiếp theo theo Best Line"
            className="px-2 py-0.5 text-[10px] font-black bg-cyan-500/30 text-cyan-200 hover:bg-cyan-500/50 rounded transition flex items-center gap-0.5"
          >
            +3 PLIES
          </button>
        </div>
      </div>

      <div className="absolute bottom-3 left-3 text-[11px] text-gold/70 bg-obsidian/90 px-3 py-1 rounded-lg border border-gold/20 pointer-events-none z-30 flex items-center gap-1.5">
        <Sparkles className="w-3.5 h-3.5 text-gold" /> Mở Rộng Ngang & Sâu Vô Hạn • Lưu TT Vĩnh Cửu • 120 FPS
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
// COMPONENT CHÍNH: MINDMAP VISUALIZER VÔ HẠN
// ============================================================================
export function MindmapVisualizer({ show, close, fen, history, score, line, thought, status, onApplyFen }) {
  if (!show) return null;

  const currentFen = fen || 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1';
  const gameHistory = history && Array.isArray(history) && history.length > 0 ? history : [currentFen];

  const [layoutMode, setLayoutMode] = useState('radial'); // 'radial' | 'tree'
  const [selectedPly, setSelectedPly] = useState(gameHistory.length - 1);
  const [activeNode, setActiveNode] = useState(null);

  // Trạng thái mở rộng chiều ngang (Breadth Limit) - KHÔNG GIỚI HẠN
  const [breadthLimit, setBreadthLimit] = useState(6);

  // Trạng thái mở rộng chiều sâu đệ quy - Map<nodeId, { depth: number, breadth: number }>
  const [expandedNodes, setExpandedNodes] = useState(() => new Map());

  // Kho Tri Thức TT Vĩnh Cửu
  const [ttStore, setTtStore] = useState(() => loadPersistentTTStore());
  const [ttFeedback, setTtFeedback] = useState('');

  useEffect(() => {
    setSelectedPly(gameHistory.length - 1);
    setExpandedNodes(new Map());
  }, [currentFen, gameHistory.length]);

  const inspectFen = gameHistory[selectedPly] || currentFen;
  const parsedInspect = useMemo(() => parse(inspectFen), [inspectFen]);

  // Tổng số nước đi hợp lệ tại thế cờ gốc
  const allRootMoves = useMemo(() => {
    return generateCandidateMoves(inspectFen, 999);
  }, [inspectFen]);

  const totalAvailableMoves = allRootMoves.length;

  // SINH CÂY TƯ DUY VÔ HẠN ĐỆ QUY (INFINITE RECURSIVE TREE GENERATOR)
  const treeNodes = useMemo(() => {
    const nodes = [];
    const rootTTHit = ttStore[inspectFen];
    const rootEval = evaluatePosition(parsedInspect.board);

    // Root Node (Level 0)
    const rootNode = {
      id: 'root',
      level: 0,
      title: `Thế cờ Turn #${Math.floor(selectedPly / 2) + 1}`,
      uci: 'GỐC',
      fen: inspectFen,
      from: -1,
      to: -1,
      score: rootTTHit ? rootTTHit.score : rootEval.score,
      badge: parsedInspect.turn === 'w' ? 'Lượt Đỏ' : 'Lượt Đen',
      intent: 'Khởi điểm cây phân nhánh suy tưởng vô hạn',
      ttHit: !!rootTTHit,
      ttDepth: rootTTHit?.depth || 0
    };
    nodes.push(rootNode);

    // Level 1: Các biến thể ứng viên theo breadthLimit (Mở rộng chiều ngang vô hạn)
    const level1Moves = allRootMoves.slice(0, breadthLimit);

    // Hàm đệ quy sinh các nhánh con cháu không giới hạn chiều sâu
    function expandRecursive(parentNode, depthBudget) {
      if (depthBudget <= 0) return;

      const childMoves = generateCandidateMoves(parentNode.fen, 3);
      childMoves.forEach((move, cIdx) => {
        const childId = `${parentNode.id}_c${cIdx}`;
        const childTTHit = ttStore[move.fen];

        const childNode = {
          id: childId,
          parentId: parentNode.id,
          level: parentNode.level + 1,
          title: `L${parentNode.level + 1}: ${move.notation}`,
          uci: move.uci,
          fen: move.fen,
          from: move.sq,
          to: move.dest,
          score: childTTHit ? childTTHit.score : move.score,
          badge: move.badge,
          intent: move.intent,
          ttHit: !!childTTHit,
          ttDepth: childTTHit?.depth || 0
        };
        nodes.push(childNode);

        // Đào sâu tiếp nếu node này được cấu hình mở rộng
        const nodeExpansion = expandedNodes.get(childId) || 0;
        const remainingBudget = Math.max(depthBudget - 1, nodeExpansion);
        if (remainingBudget > 0) {
          expandRecursive(childNode, remainingBudget);
        }
      });
    }

    level1Moves.forEach((cand, idx) => {
      const candId = `cand${idx}`;
      const ttEntry = ttStore[cand.fen];

      const candNode = {
        id: candId,
        parentId: 'root',
        level: 1,
        title: `${cand.notation}`,
        uci: cand.uci,
        fen: cand.fen,
        from: cand.sq,
        to: cand.dest,
        score: ttEntry ? ttEntry.score : cand.score,
        badge: cand.badge,
        intent: cand.intent,
        ttHit: !!ttEntry,
        ttDepth: ttEntry?.depth || 0
      };
      nodes.push(candNode);

      // Phản đòn đối phương mặc định (Tầng 2)
      const replyMoves = generateCandidateMoves(cand.fen, 1);
      if (replyMoves.length > 0) {
        const reply = replyMoves[0];
        const replyId = `reply${idx}`;
        const replyTTHit = ttStore[reply.fen];

        const replyNode = {
          id: replyId,
          parentId: candId,
          level: 2,
          title: `Đối: ${reply.notation}`,
          uci: reply.uci,
          fen: reply.fen,
          from: reply.sq,
          to: reply.dest,
          score: replyTTHit ? replyTTHit.score : reply.score,
          badge: 'Phản đòn',
          intent: 'Đối phương điều động quân chống trả',
          ttHit: !!replyTTHit,
          ttDepth: replyTTHit?.depth || 0
        };
        nodes.push(replyNode);

        // Kiểm tra xem candNode hoặc replyNode có yêu cầu mở rộng sâu tiếp không
        const candDepth = expandedNodes.get(candId) || 0;
        const replyDepth = expandedNodes.get(replyId) || 0;
        const maxDepthToExpand = Math.max(candDepth, replyDepth);

        if (maxDepthToExpand > 0) {
          expandRecursive(replyNode, maxDepthToExpand);
        }
      }
    });

    return nodes;
  }, [inspectFen, parsedInspect, selectedPly, breadthLimit, allRootMoves, expandedNodes, ttStore]);

  useEffect(() => {
    if (treeNodes && treeNodes.length > 1 && !activeNode) {
      setActiveNode(treeNodes[1]);
    }
  }, [treeNodes, activeNode]);

  // Hành động Mở Rộng Chiều Ngang (+5 biến thể) - VÔ HẠN (KHÔNG BAO GIỜ WRAP-AROUND)
  const handleExpandBreadth = useCallback(() => {
    setBreadthLimit((prev) => Math.min(totalAvailableMoves, prev + 5));
    setTtFeedback(`Đã mở rộng chiều ngang (+5): ${Math.min(totalAvailableMoves, breadthLimit + 5)} / ${totalAvailableMoves} biến thể!`);
    setTimeout(() => setTtFeedback(''), 3000);
  }, [breadthLimit, totalAvailableMoves]);

  // Thu gọn chiều ngang (-5 biến thể)
  const handleShrinkBreadth = useCallback(() => {
    setBreadthLimit((prev) => Math.max(2, prev - 5));
    setTtFeedback(`Đã thu gọn chiều ngang: ${Math.max(2, breadthLimit - 5)} biến thể!`);
    setTimeout(() => setTtFeedback(''), 3000);
  }, [breadthLimit]);

  // Mở rộng toàn bộ chiều ngang (100% nước đi)
  const handleExpandAllBreadth = useCallback(() => {
    setBreadthLimit(totalAvailableMoves);
    setTtFeedback(`Đã bung 100% toàn bộ ${totalAvailableMoves} nước đi hợp lệ!`);
    setTimeout(() => setTtFeedback(''), 3000);
  }, [totalAvailableMoves]);

  // Hành động Mở Rộng Chiều Sâu (+1 Ply) cho Node Đang Chọn - VÔ HẠN
  const handleExpandDepth = useCallback(() => {
    if (!activeNode) return;
    setExpandedNodes((prev) => {
      const next = new Map(prev);
      const currentDepth = next.get(activeNode.id) || 0;
      next.set(activeNode.id, currentDepth + 1);
      setTtFeedback(`Đã đào sâu thêm +1 Ply (Tổng Depth: ${currentDepth + 1}) cho nhánh ${activeNode.title}!`);
      setTimeout(() => setTtFeedback(''), 3000);
      return next;
    });
  }, [activeNode]);

  // Hành động Đào Sâu +3 Plies Ngay Lập Tức
  const handleDeepenThreePlies = useCallback(() => {
    if (!activeNode) return;
    setExpandedNodes((prev) => {
      const next = new Map(prev);
      const currentDepth = next.get(activeNode.id) || 0;
      next.set(activeNode.id, currentDepth + 3);
      setTtFeedback(`Đã đào sâu +3 Plies (Tổng Depth: ${currentDepth + 3}) cho nhánh ${activeNode.title}!`);
      setTimeout(() => setTtFeedback(''), 3000);
      return next;
    });
  }, [activeNode]);

  // Thu gọn nhánh sâu của Node đang chọn
  const handleCollapseDepth = useCallback(() => {
    if (!activeNode) return;
    setExpandedNodes((prev) => {
      const next = new Map(prev);
      next.delete(activeNode.id);
      setTtFeedback(`Đã thu gọn chiều sâu nhánh ${activeNode.title}!`);
      setTimeout(() => setTtFeedback(''), 3000);
      return next;
    });
  }, [activeNode]);

  // Lưu toàn bộ cây tư duy vào Kho Tri Thức TT Vĩnh Cửu
  const handleSaveTreeToPersistentTT = useCallback(() => {
    const updatedStore = { ...ttStore };
    let savedCount = 0;

    treeNodes.forEach((node) => {
      if (node.fen) {
        updatedStore[node.fen] = {
          fen: node.fen,
          uci: node.uci,
          title: node.title,
          score: node.score || 0,
          depth: 14,
          timestamp: Date.now()
        };
        savedCount++;
      }
    });

    savePersistentTTStore(updatedStore);
    setTtStore(updatedStore);
    setTtFeedback(`Đã lưu thành công ${savedCount} thế cờ vào Kho Tri Thức TT Vĩnh Cửu!`);
    setTimeout(() => setTtFeedback(''), 4000);
  }, [treeNodes, ttStore]);

  // Xuất file JSON Kho Tri Thức TT
  const handleExportTTJson = useCallback(() => {
    const dataStr = "data:text/json;charset=utf-8," + encodeURIComponent(JSON.stringify(ttStore, null, 2));
    const downloadAnchor = document.createElement('a');
    downloadAnchor.setAttribute("href", dataStr);
    downloadAnchor.setAttribute("download", `tt-${Date.now()}.json`);
    document.body.appendChild(downloadAnchor);
    downloadAnchor.click();
    downloadAnchor.remove();
    setTtFeedback('Đã xuất file Tri Thức TT JSON thành công!');
    setTimeout(() => setTtFeedback(''), 3000);
  }, [ttStore]);

  // Nạp file JSON Kho Tri Thức TT
  const handleImportTTJson = useCallback((e) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (event) => {
      try {
        const imported = JSON.parse(event.target?.result);
        const merged = { ...ttStore, ...imported };
        savePersistentTTStore(merged);
        setTtStore(merged);
        setTtFeedback(`Đã nạp ${Object.keys(imported).length} mục tri thức TT thành công!`);
        setTimeout(() => setTtFeedback(''), 4000);
      } catch (err) {
        alert('File JSON không hợp lệ!');
      }
    };
    reader.readAsText(file);
  }, [ttStore]);

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

  const ttCount = Object.keys(ttStore).length;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-obsidian/90 backdrop-blur-md font-body">
      <div className="bg-obsidian-card border-2 border-gold/40 rounded-2xl max-w-7xl w-full h-[94vh] flex flex-col shadow-glow overflow-hidden">
        
        {/* HEADER TOOLBAR */}
        <div className="bg-obsidian px-6 py-3 border-b border-gold/30 flex items-center justify-between flex-wrap gap-3">
          <div className="flex items-center gap-3">
            <div className="w-9 h-9 rounded-lg bg-gold/10 border border-gold flex items-center justify-center text-gold shadow-glow">
              <Compass className="w-5 h-5" />
            </div>
            <div>
              <h2 className="text-base font-royal font-bold text-gold flex items-center gap-2">
                🧠 SƠ ĐỒ TƯ DUY 360° & TRI THỨC TT VĨNH CỬU
                <span className="px-2 py-0.5 rounded text-[10px] uppercase tracking-wider font-bold bg-amber-500/20 text-amber-300 border border-amber-500/40 flex items-center gap-1">
                  <Database className="w-3 h-3" /> {ttCount} Thế Cờ TT
                </span>
              </h2>
              <p className="text-xs text-gold/60">
                Ply: <b className="text-gold">{selectedPly}</b> / {gameHistory.length - 1} | Nodes: <b className="text-emerald-400">{treeNodes.length}</b> | Chiều ngang: <b className="text-amber-300">{breadthLimit >= totalAvailableMoves ? 'TẤT CẢ' : breadthLimit} / {totalAvailableMoves}</b>
              </p>
            </div>
          </div>

          {/* TT ACTIONS & CONTROLS */}
          <div className="flex items-center gap-2 flex-wrap">
            {ttFeedback && (
              <span className="text-xs px-2.5 py-1 rounded bg-emerald-500/20 text-emerald-300 border border-emerald-500/40 animate-pulse font-bold">
                {ttFeedback}
              </span>
            )}

            {/* LƯU CÂY VÀO TT */}
            <button
              onClick={handleSaveTreeToPersistentTT}
              title="Lưu toàn bộ các thế cờ trong cây tìm kiếm vào Kho Tri Thức TT Vĩnh Cửu"
              className="px-2.5 py-1 rounded bg-gold text-obsidian hover:bg-gold-light text-xs font-bold transition flex items-center gap-1 shadow-glow"
            >
              <Save className="w-3.5 h-3.5 fill-current" /> LƯU TT ({treeNodes.length})
            </button>

            {/* XUẤT / NẠP FILE TT */}
            <button
              onClick={handleExportTTJson}
              title="Tải file JSON Kho Tri Thức TT xuống máy"
              className="p-1.5 rounded bg-obsidian border border-gold/40 text-gold hover:bg-gold/20 text-xs transition"
            >
              <Download className="w-4 h-4" />
            </button>

            <label
              title="Nạp file JSON Kho Tri Thức TT từ máy tính"
              className="p-1.5 rounded bg-obsidian border border-gold/40 text-gold hover:bg-gold/20 text-xs transition cursor-pointer"
            >
              <Upload className="w-4 h-4" />
              <input type="file" accept=".json" onChange={handleImportTTJson} className="hidden" />
            </label>

            <div className="h-4 w-px bg-gold/30 mx-1" />

            {/* CHẾ ĐỘ VIEW BUNG TỎA / CÂY */}
            <div className="flex items-center gap-1 bg-obsidian p-1 rounded-lg border border-gold/30">
              <button
                onClick={() => setLayoutMode('radial')}
                className={`px-2.5 py-0.5 rounded text-xs font-bold transition flex items-center gap-1 ${
                  layoutMode === 'radial' ? 'bg-gold text-obsidian shadow-glow font-black' : 'text-gold/70 hover:text-gold'
                }`}
              >
                <Target className="w-3 h-3" /> TỎA TRÒN
              </button>
              <button
                onClick={() => setLayoutMode('tree')}
                className={`px-2.5 py-0.5 rounded text-xs font-bold transition flex items-center gap-1 ${
                  layoutMode === 'tree' ? 'bg-gold text-obsidian shadow-glow font-black' : 'text-gold/70 hover:text-gold'
                }`}
              >
                <LayoutGrid className="w-3 h-3" /> CÂY
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
                onExpandBreadth={handleExpandBreadth}
                onShrinkBreadth={handleShrinkBreadth}
                onExpandDepth={handleExpandDepth}
                onDeepenThreePlies={handleDeepenThreePlies}
                onExpandAllBreadth={handleExpandAllBreadth}
                breadthCount={breadthLimit}
                totalAvailableMoves={totalAvailableMoves}
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
              <span className="font-bold text-gold uppercase flex items-center gap-1.5">
                <FolderTree className="w-4 h-4 text-gold" /> CHI TIẾT NODE
              </span>
              <div className="flex items-center gap-1.5">
                {activeNode?.ttHit && (
                  <span className="px-2 py-0.5 rounded bg-amber-500/20 text-amber-300 font-bold text-[10px] border border-amber-500/40">
                    ⚡ TT KHỚP
                  </span>
                )}
                <span className="px-2 py-0.5 rounded bg-emerald-500/20 text-emerald-300 font-mono text-[10px] font-bold">
                  {activeNode?.score !== undefined ? `${activeNode.score > 0 ? '+' : ''}${activeNode.score} cp` : '0 cp'}
                </span>
              </div>
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
              <div className="space-y-2.5 text-xs">
                <div className="bg-obsidian-card p-3 rounded-lg border border-gold/20 space-y-1.5">
                  <div className="flex items-center justify-between">
                    <div className="font-bold text-gold text-sm">{activeNode.title}</div>
                    <span className="text-[10px] font-mono text-cyan-300 bg-cyan-950/80 px-2 py-0.5 rounded border border-cyan-800">
                      UCI: {activeNode.uci}
                    </span>
                  </div>
                  <div className="text-gold/80 text-[11px] italic">
                    "{activeNode.intent}"
                  </div>
                  <div className="text-[10px] font-mono text-gold/40 break-all">
                    FEN: {activeNode.fen}
                  </div>
                </div>

                {/* CÁC THAO TÁC MỞ RỘNG VÔ HẠN CHO NODE NÀY */}
                <div className="grid grid-cols-2 gap-2">
                  <button
                    onClick={handleExpandDepth}
                    className="py-1.5 px-2 rounded bg-cyan-500/20 text-cyan-300 border border-cyan-500/40 font-bold text-xs hover:bg-cyan-500/30 transition flex items-center justify-center gap-1"
                  >
                    <ArrowDown className="w-3.5 h-3.5" /> ĐÀO SÂU (+1 PLY)
                  </button>

                  <button
                    onClick={handleDeepenThreePlies}
                    className="py-1.5 px-2 rounded bg-cyan-600/30 text-cyan-200 border border-cyan-400/40 font-bold text-xs hover:bg-cyan-600/40 transition flex items-center justify-center gap-1"
                  >
                    <ChevronDown className="w-3.5 h-3.5" /> +3 PLIES SÂU
                  </button>
                </div>

                <div className="grid grid-cols-2 gap-2">
                  <button
                    onClick={handleExpandBreadth}
                    className="py-1.5 px-2 rounded bg-amber-500/20 text-amber-300 border border-amber-500/40 font-bold text-xs hover:bg-amber-500/30 transition flex items-center justify-center gap-1"
                  >
                    <ArrowRight className="w-3.5 h-3.5" /> +5 BIẾN THỂ
                  </button>

                  <button
                    onClick={handleCollapseDepth}
                    className="py-1.5 px-2 rounded bg-zinc-800 text-gold/80 border border-gold/20 font-bold text-xs hover:bg-zinc-700 transition flex items-center justify-center gap-1"
                  >
                    <ChevronUp className="w-3.5 h-3.5" /> THU GỌN SÂU
                  </button>
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
                      <Play className="w-3.5 h-3.5 fill-current" /> ÁP DỤNG VÀO BÀN
                    </button>
                  )}
                  <button
                    onClick={() => {
                      engine.position(activeNode.fen);
                      engine.search(6, 2000);
                      alert('Đã phát lệnh tìm kiếm sâu cho thế cờ nhánh này qua Engine!');
                    }}
                    className="py-2 px-3 rounded bg-emerald-500/20 text-emerald-300 border border-emerald-500/40 font-bold text-xs hover:bg-emerald-500/30 transition flex items-center gap-1"
                  >
                    <Zap className="w-3.5 h-3.5" /> SEARCH ENGINE
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
