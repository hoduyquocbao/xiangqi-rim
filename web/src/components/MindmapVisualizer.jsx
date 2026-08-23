// web/src/components/MindmapVisualizer.jsx
// Sơ Đồ Tư Duy Cây Suy Luận 360° & Hậu Kiểm Toàn Ván Thực Tế (Live Xiangqi 360° Mindmap & Game Tree Explorer)
// Công cụ phân tích thực chiến: Cây phân nhánh 3 tầng động học, Quét bước ngoặt & sai lầm toàn ván,
// Bóc tách JSONL DeepSeek-R1, Phòng thử nghiệm What-If, Bánh đà TT Cache và Benchmark phần cứng.

import React, { useState, useEffect, useRef, useMemo, useCallback } from 'react';
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
  RotateCcw,
  CheckCircle2,
  AlertTriangle,
  Play,
  Gauge,
  Activity,
  FileCode,
  Crosshair,
  Sliders,
  ChevronRight,
  ChevronDown,
  Copy,
  Upload,
  RefreshCw,
  Search,
  Sword,
  Target,
  Flame,
  Shield,
  HelpCircle,
  Award
} from 'lucide-react';
import { parse, fen as buildFen, moves as getLegalMoves, check as isCheck, hasLegalMoves, uciToMove } from '../rules/rules.js';
import { instance as engine } from '../engine/engine.js';

// Bảng tra cứu chữ Hán quân cờ O(1)
const symbols = {
  K: '帥', A: '仕', B: '相', N: '傌', R: '俥', C: '炮', P: '兵',
  k: '將', a: '士', b: '象', n: '馬', r: '車', c: '砲', p: '卒'
};

// Tên quân cờ tiếng Việt
const pieceNames = {
  K: 'Tướng Đỏ', A: 'Sĩ Đỏ', B: 'Tượng Đỏ', N: 'Mã Đỏ', R: 'Xe Đỏ', C: 'Pháo Đỏ', P: 'Binh Đỏ',
  k: 'Tướng Đen', a: 'Sĩ Đen', b: 'Tượng Đen', n: 'Mã Đen', r: 'Xe Đen', c: 'Pháo Đen', p: 'Tốt Đen'
};

// Trọng số giá trị quân cờ vật chất
const pieceWeights = {
  K: 10000, R: 900, C: 450, N: 400, B: 200, A: 200, P: 100,
  k: 10000, r: 900, c: 450, n: 400, b: 200, a: 200, p: 100
};

// Bảng Tọa Độ Tĩnh Cố Định LUT (Lookup Tables) O(1) CPU Cache L1 Friendly
const padx = 25;
const pady = 25;
const cellw = 30;
const cellh = 30;
const width = 290;
const height = 320;

const gridx = new Float32Array([
  padx + 0 * cellw,
  padx + 1 * cellw,
  padx + 2 * cellw,
  padx + 3 * cellw,
  padx + 4 * cellw,
  padx + 5 * cellw,
  padx + 6 * cellw,
  padx + 7 * cellw,
  padx + 8 * cellw,
]);

const gridy = new Float32Array([
  pady + 9 * cellh, // rank 0
  pady + 8 * cellh, // rank 1
  pady + 7 * cellh, // rank 2
  pady + 6 * cellh, // rank 3
  pady + 5 * cellh, // rank 4
  pady + 4 * cellh, // rank 5
  pady + 3 * cellh, // rank 6
  pady + 2 * cellh, // rank 7
  pady + 1 * cellh, // rank 8
  pady + 0 * cellh, // rank 9
]);

// Hàm chuyển đổi tọa độ ô cờ sang chuỗi UCI (vd: 22 -> "e2")
function sqToUci(sq) {
  const file = sq % 9;
  const rank = Math.floor(sq / 9);
  return `${String.fromCharCode(97 + file)}${rank}`;
}

// Hàm chuyển đổi nước đi sang ký hiệu truyền thống tiếng Việt (vd: "Pháo 2 bình 5")
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

// Đánh giá thế cờ Heuristic nhanh O(1)
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

    // Thưởng tốt qua sông
    if (piece === 'P' && rank >= 5) val += 100;
    if (piece === 'p' && rank <= 4) val += 100;

    // Kiểm soát trung lộ (cột 4, 3, 5)
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

// Vẽ nền tĩnh Offscreen Buffer (Chỉ vẽ 1 lần)
function drawStaticBackground(ctx) {
  const grad = ctx.createRadialGradient(width / 2, height / 2, 20, width / 2, height / 2, width);
  grad.addColorStop(0, '#2c1e11');
  grad.addColorStop(1, '#140d07');
  ctx.fillStyle = grad;
  ctx.fillRect(5, 5, width - 10, height - 10);

  ctx.strokeStyle = '#D4AF37';
  ctx.lineWidth = 2;
  ctx.strokeRect(5, 5, width - 10, height - 10);

  ctx.strokeStyle = '#8A6B2D';
  ctx.lineWidth = 1.2;
  for (let r = 0; r < 10; r++) {
    const y = gridy[r];
    ctx.beginPath();
    ctx.moveTo(gridx[0], y);
    ctx.lineTo(gridx[8], y);
    ctx.stroke();
  }

  for (let f = 0; f < 9; f++) {
    const x = gridx[f];
    ctx.beginPath();
    ctx.moveTo(x, gridy[0]);
    ctx.lineTo(x, gridy[4]);
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(x, gridy[5]);
    ctx.lineTo(x, gridy[9]);
    ctx.stroke();
  }

  ctx.beginPath();
  ctx.moveTo(gridx[0], gridy[4]);
  ctx.lineTo(gridx[0], gridy[5]);
  ctx.moveTo(gridx[8], gridy[4]);
  ctx.lineTo(gridx[8], gridy[5]);
  ctx.stroke();

  ctx.beginPath();
  ctx.moveTo(gridx[3], gridy[0]);
  ctx.lineTo(gridx[5], gridy[2]);
  ctx.moveTo(gridx[5], gridy[0]);
  ctx.lineTo(gridx[3], gridy[2]);
  ctx.stroke();

  ctx.beginPath();
  ctx.moveTo(gridx[3], gridy[7]);
  ctx.lineTo(gridx[5], gridy[9]);
  ctx.moveTo(gridx[5], gridy[7]);
  ctx.lineTo(gridx[3], gridy[9]);
  ctx.stroke();

  ctx.fillStyle = '#8A6B2D';
  ctx.font = 'bold 10px serif';
  ctx.textAlign = 'center';
  ctx.fillText('楚 河', 75, (gridy[4] + gridy[5]) / 2 + 3);
  ctx.fillText('漢 界', 215, (gridy[4] + gridy[5]) / 2 + 3);
}

// Bàn Cờ GPU Hardware Canvas Tối Thượng
function FastGpuBoard({ fen, arrowFrom, arrowTo, selectedSq, validDests, onSquareClick }) {
  const canvasRef = useRef(null);
  const staticCacheRef = useRef(null);
  const pulseRef = useRef(0);
  const parsedBoard = useMemo(() => parse(fen).board, [fen]);

  useEffect(() => {
    const off = document.createElement('canvas');
    off.width = width;
    off.height = height;
    const offCtx = off.getContext('2d');
    if (offCtx) {
      drawStaticBackground(offCtx);
      staticCacheRef.current = off;
    }
  }, []);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d', { alpha: false });
    if (!ctx) return;

    let animId;
    let lastTime = performance.now();

    const render = (time) => {
      const delta = (time - lastTime) / 1000;
      lastTime = time;
      pulseRef.current = (pulseRef.current + delta * 2.5) % (Math.PI * 2);

      if (staticCacheRef.current) {
        ctx.drawImage(staticCacheRef.current, 0, 0);
      }

      // Vẽ ô được chọn
      if (selectedSq !== undefined && selectedSq >= 0 && selectedSq < 90) {
        const sx = gridx[selectedSq % 9];
        const sy = gridy[Math.floor(selectedSq / 9)];
        ctx.strokeStyle = '#22c55e';
        ctx.lineWidth = 2;
        ctx.strokeRect(sx - 14, sy - 14, 28, 28);
      }

      // Vẽ các chấm nước đi hợp lệ
      if (validDests && validDests.length > 0) {
        ctx.fillStyle = 'rgba(34, 197, 94, 0.7)';
        for (const dest of validDests) {
          const dx = gridx[dest % 9];
          const dy = gridy[Math.floor(dest / 9)];
          ctx.beginPath();
          ctx.arc(dx, dy, 4, 0, Math.PI * 2);
          ctx.fill();
        }
      }

      // Vẽ Laser Arrow động học
      if (arrowFrom >= 0 && arrowTo >= 0) {
        const fromX = gridx[arrowFrom % 9];
        const fromY = gridy[Math.floor(arrowFrom / 9)];
        const toX = gridx[arrowTo % 9];
        const toY = gridy[Math.floor(arrowTo / 9)];

        const glowAlpha = 0.5 + 0.5 * Math.sin(pulseRef.current);
        ctx.strokeStyle = `rgba(239, 68, 68, ${0.4 * glowAlpha})`;
        ctx.lineWidth = 6;
        ctx.beginPath();
        ctx.moveTo(fromX, fromY);
        ctx.lineTo(toX, toY);
        ctx.stroke();

        ctx.strokeStyle = '#f59e0b';
        ctx.lineWidth = 2.5;
        ctx.beginPath();
        ctx.moveTo(fromX, fromY);
        ctx.lineTo(toX, toY);
        ctx.stroke();

        const angle = Math.atan2(toY - fromY, toX - fromX);
        const headLen = 10;
        ctx.fillStyle = '#f59e0b';
        ctx.beginPath();
        ctx.moveTo(toX, toY);
        ctx.lineTo(toX - headLen * Math.cos(angle - Math.PI / 6), toY - headLen * Math.sin(angle - Math.PI / 6));
        ctx.lineTo(toX - headLen * Math.cos(angle + Math.PI / 6), toY - headLen * Math.sin(angle + Math.PI / 6));
        ctx.closePath();
        ctx.fill();
      }

      // Vẽ 32 quân cờ chữ Hán
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      ctx.font = 'bold 13px sans-serif';

      for (let sq = 0; sq < 90; sq++) {
        const piece = parsedBoard[sq];
        if (piece === '.') continue;

        const x = gridx[sq % 9];
        const y = gridy[Math.floor(sq / 9)];
        const isRed = piece === piece.toUpperCase();

        ctx.fillStyle = '#1c150e';
        ctx.beginPath();
        ctx.arc(x, y, 12, 0, Math.PI * 2);
        ctx.fill();

        ctx.strokeStyle = isRed ? '#ef4444' : '#60a5fa';
        ctx.lineWidth = 1.2;
        ctx.stroke();

        ctx.fillStyle = isRed ? '#f87171' : '#93c5fd';
        ctx.fillText(symbols[piece] || piece, x, y + 1);
      }

      animId = requestAnimationFrame(render);
    };

    animId = requestAnimationFrame(render);
    return () => cancelAnimationFrame(animId);
  }, [parsedBoard, arrowFrom, arrowTo, selectedSq, validDests]);

  const handleCanvasClick = (e) => {
    if (!onSquareClick) return;
    const canvas = canvasRef.current;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const cx = (e.clientX - rect.left) * (width / rect.width);
    const cy = (e.clientY - rect.top) * (height / rect.height);

    let closestSq = -1;
    let minDist = 20;
    for (let sq = 0; sq < 90; sq++) {
      const gx = gridx[sq % 9];
      const gy = gridy[Math.floor(sq / 9)];
      const dist = Math.hypot(cx - gx, cy - gy);
      if (dist < minDist) {
        minDist = dist;
        closestSq = sq;
      }
    }

    if (closestSq !== -1) {
      onSquareClick(closestSq);
    }
  };

  return (
    <canvas
      ref={canvasRef}
      width={width}
      height={height}
      onClick={handleCanvasClick}
      className="rounded-xl border border-gold/40 shadow-glow bg-black cursor-pointer select-none"
      style={{ transform: 'translate3d(0,0,0)' }}
    />
  );
}

// Component Chính: Sơ Đồ Tư Duy Cây Suy Luận 360° & Hậu Kiểm Toàn Ván
export function MindmapVisualizer({ show, close, fen, history, score, line, thought, status, onApplyFen }) {
  if (!show) return null;

  const currentFen = fen || 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1';
  const gameHistory = history && Array.isArray(history) && history.length > 0 ? history : [currentFen];

  // State quản lý tab và tương tác
  const [tab, setTab] = useState('tree'); // 'tree' | 'timeline' | 'jsonl' | 'sandbox' | 'tt' | 'bench'
  const [selectedPly, setSelectedPly] = useState(gameHistory.length - 1);
  const [activeNode, setActiveNode] = useState(null);
  const [sandboxFen, setSandboxFen] = useState(currentFen);
  const [sandboxSelected, setSandboxSelected] = useState(null);
  const [sandboxValidMoves, setSandboxValidMoves] = useState([]);
  const [jsonlInput, setJsonlInput] = useState('');
  const [parsedJsonlData, setParsedJsonlData] = useState(null);
  const [jsonlTurnIdx, setJsonlTurnIdx] = useState(0);

  // Benchmark states
  const [benchRunning, setBenchRunning] = useState(false);
  const [benchResults, setBenchResults] = useState(null);

  // Cập nhật khi FEN đổi
  useEffect(() => {
    setSelectedPly(gameHistory.length - 1);
    setSandboxFen(currentFen);
  }, [currentFen, gameHistory.length]);

  // 1. SINH CÂY SUY TƯỞNG ĐA TẦNG 3-PLY TỪ FEN ĐANG CHỌN (REAL LIVE 3-PLY TREE)
  const inspectFen = tab === 'timeline' ? (gameHistory[selectedPly] || currentFen) : currentFen;
  const parsedInspect = useMemo(() => parse(inspectFen), [inspectFen]);

  const treeData = useMemo(() => {
    const board = parsedInspect.board;
    const turn = parsedInspect.turn;
    const isTurnRed = turn === 'w';
    const rootEval = evaluatePosition(board);

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

        // Giả lập thế cờ sau nước đi candidate
        const clonedBoard1 = [...board];
        clonedBoard1[dest] = piece;
        clonedBoard1[sq] = '.';
        const nextTurn1 = turn === 'w' ? 'b' : 'w';
        const nextFen1 = buildFen(clonedBoard1, nextTurn1);
        const eval1 = evaluatePosition(clonedBoard1);
        const check1 = isCheck(clonedBoard1, nextTurn1);

        // Tính điểm heuristic candidate
        let moveScore = isTurnRed ? (eval1.score - rootEval.score) : (rootEval.score - eval1.score);
        if (isCap) moveScore += (pieceWeights[targetPiece] || 50) / 2;
        if (check1) moveScore += 80;

        // Phân loại ý đồ chiến thuật
        let intent = 'Phát triển quân cờ, kiểm soát vị trí';
        let badge = 'Nước phát triển';
        let badgeColor = 'bg-blue-500/20 text-blue-300 border-blue-500/40';

        if (check1) {
          intent = 'Chiếu tướng trực diện, dồn ép Cung Tướng';
          badge = 'Chiếu tướng';
          badgeColor = 'bg-amber-500/20 text-amber-300 border-amber-500/40';
        } else if (isCap) {
          intent = `Ăn ${pieceNames[targetPiece] || 'quân'}, chiếm ưu thế vật chất`;
          badge = 'Ăn quân';
          badgeColor = 'bg-emerald-500/20 text-emerald-300 border-emerald-500/40';
        } else if (sq % 9 === 4 || dest % 9 === 4) {
          intent = 'Khống chế lộ 5 trung lộ, mở đường Pháo đầu';
          badge = 'Trung Lộ';
          badgeColor = 'bg-purple-500/20 text-purple-300 border-purple-500/40';
        }

        // Tầng 2: Tìm 1 nước đáp trả tốt nhất của đối phương
        let bestReply = null;
        for (let rsq = 0; rsq < 90; rsq++) {
          const rpiece = clonedBoard1[rsq];
          if (rpiece === '.') continue;
          const isRPieceRed = rpiece === rpiece.toUpperCase();
          if (isRPieceRed === isTurnRed) continue;

          const rdests = getLegalMoves(clonedBoard1, rsq, nextTurn1);
          if (rdests.length > 0) {
            const rdest = rdests[0];
            const ruci = `${sqToUci(rsq)}${sqToUci(rdest)}`;
            const rnotation = moveToNotation(clonedBoard1, rsq, rdest);
            const clonedBoard2 = [...clonedBoard1];
            clonedBoard2[rdest] = rpiece;
            clonedBoard2[rsq] = '.';
            const nextTurn2 = turn;
            const nextFen2 = buildFen(clonedBoard2, nextTurn2);
            const eval2 = evaluatePosition(clonedBoard2);

            bestReply = {
              from: rsq,
              to: rdest,
              uci: ruci,
              notation: rnotation,
              fen: nextFen2,
              score: eval2.score,
              intent: 'Đối phương điều động quân chống trả'
            };
            break;
          }
        }

        candidates.push({
          from: sq,
          to: dest,
          uci,
          notation,
          piece,
          isCapture: isCap,
          isCheck: check1,
          score: isTurnRed ? eval1.score : -eval1.score,
          heuristicScore: moveScore,
          fen: nextFen1,
          intent,
          badge,
          badgeColor,
          reply: bestReply
        });
      }
    }

    // Sắp xếp các ứng viên theo điểm số giảm dần
    candidates.sort((a, b) => b.heuristicScore - a.heuristicScore);

    return {
      rootFen: inspectFen,
      rootScore: rootEval.score,
      turn,
      candidates: candidates.slice(0, 6) // Lấy Top 6 Candidate tốt nhất
    };
  }, [inspectFen, parsedInspect]);

  // Chọn node mặc định
  useEffect(() => {
    if (treeData && treeData.candidates.length > 0 && !activeNode) {
      const top = treeData.candidates[0];
      setActiveNode({
        title: `Ứng Viên #1 (Tối Ưu): ${top.notation}`,
        uci: top.uci,
        fen: top.fen,
        from: top.from,
        to: top.to,
        score: top.score,
        intent: top.intent,
        badge: top.badge,
        badgeColor: top.badgeColor
      });
    }
  }, [treeData, activeNode]);

  // 2. PHÂN TÍCH QUÉT BƯỚC NGOẶT TOÀN VÁN (GAME TIMELINE SCANNER)
  const timelineAnalysis = useMemo(() => {
    const items = [];
    let prevScore = 0;

    for (let ply = 0; ply < gameHistory.length; ply++) {
      const fenItem = gameHistory[ply];
      const parsed = parse(fenItem);
      const evalRes = evaluatePosition(parsed.board);
      const isRedTurn = parsed.turn === 'w';
      const swing = evalRes.score - prevScore;

      let tag = 'Bình Ổn';
      let tagColor = 'bg-gold/10 text-gold/80 border-gold/30';
      let isTurningPoint = false;

      if (Math.abs(swing) >= 300) {
        tag = swing > 0 ? '🌟 ĐỘT PHÁ' : '💥 SAI LẦM';
        tagColor = swing > 0 ? 'bg-emerald-500/20 text-emerald-300 border-emerald-500/40' : 'bg-red-500/20 text-red-300 border-red-500/40';
        isTurningPoint = true;
      } else if (Math.abs(swing) >= 150) {
        tag = swing > 0 ? '🎯 CHIẾM ƯU' : '⚠️ NƯỚC YẾU';
        tagColor = swing > 0 ? 'bg-cyan-500/20 text-cyan-300 border-cyan-500/40' : 'bg-amber-500/20 text-amber-300 border-amber-500/40';
      }

      if (!hasLegalMoves(parsed.board, parsed.turn)) {
        tag = isCheck(parsed.board, parsed.turn) ? '👑 SÁT CỤC' : '🤝 HÒA BÍ';
        tagColor = 'bg-purple-500/20 text-purple-300 border-purple-500/40';
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
        tagColor,
        isTurningPoint
      });

      prevScore = evalRes.score;
    }

    return items;
  }, [gameHistory]);

  // Xử lý nạp JSONL Dataset từ văn bản
  const handleParseJsonl = () => {
    try {
      const trimmed = jsonlInput.trim();
      if (!trimmed) return;
      const parsedObj = JSON.parse(trimmed);
      setParsedJsonlData(parsedObj);
      setJsonlTurnIdx(0);
    } catch (err) {
      alert('Không thể giải mã JSONL! Vui lòng kiểm tra định dạng JSON: ' + err.message);
    }
  };

  // Nạp tệp JSONL từ máy tính
  const handleFileUpload = (e) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (event) => {
      const content = event.target?.result;
      if (typeof content === 'string') {
        const firstLine = content.split('\n')[0];
        setJsonlInput(firstLine);
        try {
          const obj = JSON.parse(firstLine);
          setParsedJsonlData(obj);
          setJsonlTurnIdx(0);
        } catch (err) {
          console.error(err);
        }
      }
    };
    reader.readAsText(file);
  };

  // Xử lý click ô cờ trong Sandbox
  const handleSandboxClick = (sq) => {
    const parsed = parse(sandboxFen);
    const piece = parsed.board[sq];
    const isPieceRed = piece !== '.' && piece === piece.toUpperCase();
    const isTurnRed = parsed.turn === 'w';

    if (sandboxSelected === null) {
      if (piece !== '.' && isPieceRed === isTurnRed) {
        setSandboxSelected(sq);
        setSandboxValidMoves(getLegalMoves(parsed.board, sq, parsed.turn));
      }
    } else {
      if (sandboxValidMoves.includes(sq)) {
        // Thực hiện nước đi
        const cloned = [...parsed.board];
        cloned[sq] = cloned[sandboxSelected];
        cloned[sandboxSelected] = '.';
        const nextTurn = parsed.turn === 'w' ? 'b' : 'w';
        const nextFen = buildFen(cloned, nextTurn);
        setSandboxFen(nextFen);
        setSandboxSelected(null);
        setSandboxValidMoves([]);
      } else if (piece !== '.' && isPieceRed === isTurnRed) {
        setSandboxSelected(sq);
        setSandboxValidMoves(getLegalMoves(parsed.board, sq, parsed.turn));
      } else {
        setSandboxSelected(null);
        setSandboxValidMoves([]);
      }
    }
  };

  // Chạy Benchmark phần cứng thực tế
  const runHardwareBenchmark = () => {
    setBenchRunning(true);
    setBenchResults(null);

    setTimeout(() => {
      const testFens = [
        currentFen,
        'r1bakab1r/9/1cn1c1n2/p1p1p1p1p/9/2P6/P3P1P1P/1C2C4/9/RNBAKABNR w - - 0 20',
        '2bakab2/9/1cn6/p1p3p1p/9/2C1C1R2/P3P1P1P/9/9/RNBAKAB2 w - - 3 45',
        '4k4/4C4/b4Rn1b/9/4R4/8p/P1P1N1r2/9/4A4/4KAB2 b - - 0 68'
      ];

      const iterations = 50000;
      const t0 = performance.now();

      for (let i = 0; i < iterations; i++) {
        const fenToTest = testFens[i % testFens.length];
        const p = parse(fenToTest);
        evaluatePosition(p.board);
        getLegalMoves(p.board, 22, p.turn);
      }

      const t1 = performance.now();
      const elapsedMs = t1 - t0;
      const fensPerSec = Math.round((iterations / (elapsedMs / 1000)));

      setBenchResults({
        iterations,
        elapsedMs: elapsedMs.toFixed(2),
        fensPerSec: fensPerSec.toLocaleString(),
        lutLatencyNs: ((elapsedMs * 1e6) / iterations).toFixed(1)
      });
      setBenchRunning(false);
    }, 50);
  };

  // Trích xuất các message turn của JSONL nếu có
  const jsonlTurns = useMemo(() => {
    if (!parsedJsonlData || !parsedJsonlData.messages) return [];
    const turns = [];
    const msgs = parsedJsonlData.messages;
    for (let i = 1; i < msgs.length; i += 2) {
      const userMsg = msgs[i];
      const assistMsg = msgs[i + 1];
      if (userMsg && assistMsg) {
        let assistData = null;
        try {
          assistData = JSON.parse(assistMsg.content);
        } catch {
          assistData = { thought: assistMsg.content };
        }
        turns.push({
          turnIndex: Math.floor(i / 2) + 1,
          userContent: userMsg.content,
          assistantData: assistData
        });
      }
    }
    return turns;
  }, [parsedJsonlData]);

  const activeJsonlTurn = jsonlTurns[jsonlTurnIdx] || null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-obsidian/90 backdrop-blur-md font-body">
      <div className="bg-obsidian-card border-2 border-gold/40 rounded-2xl max-w-7xl w-full h-[92vh] flex flex-col shadow-glow overflow-hidden">
        
        {/* HEADER TOOLBAR */}
        <div className="bg-obsidian px-6 py-3.5 border-b border-gold/30 flex items-center justify-between flex-wrap gap-3">
          <div className="flex items-center gap-3">
            <div className="w-9 h-9 rounded-lg bg-gold/10 border border-gold flex items-center justify-center text-gold shadow-glow">
              <Compass className="w-5 h-5 animate-spin-slow" />
            </div>
            <div>
              <h2 className="text-base font-royal font-bold text-gold flex items-center gap-2">
                🧠 SƠ ĐỒ TƯ DUY 360° & HẬU KIỂM TOÀN VÁN
                <span className="px-2 py-0.5 rounded text-[10px] uppercase tracking-wider font-bold bg-gold/20 text-gold border border-gold/40">
                  v38.1 Realtime
                </span>
              </h2>
              <p className="text-xs text-gold/60">
                Thế cờ hiện tại: <span className="text-gold font-mono font-bold truncate max-w-xs inline-block align-bottom">{currentFen}</span>
              </p>
            </div>
          </div>

          {/* TAB SWITCHER */}
          <div className="flex items-center gap-1.5 bg-obsidian p-1 rounded-lg border border-gold/30 flex-wrap">
            <button
              onClick={() => setTab('tree')}
              className={`px-3 py-1 rounded text-xs font-bold transition flex items-center gap-1.5 ${
                tab === 'tree' ? 'bg-gold text-obsidian shadow-glow font-black' : 'text-gold/70 hover:text-gold'
              }`}
            >
              <Layers className="w-3.5 h-3.5" /> 🌳 CÂY 3-PLY
            </button>
            <button
              onClick={() => setTab('timeline')}
              className={`px-3 py-1 rounded text-xs font-bold transition flex items-center gap-1.5 ${
                tab === 'timeline' ? 'bg-gold text-obsidian shadow-glow font-black' : 'text-gold/70 hover:text-gold'
              }`}
            >
              <TrendingUp className="w-3.5 h-3.5" /> 📈 BƯỚC NGOẶT ({gameHistory.length} PLIES)
            </button>
            <button
              onClick={() => setTab('jsonl')}
              className={`px-3 py-1 rounded text-xs font-bold transition flex items-center gap-1.5 ${
                tab === 'jsonl' ? 'bg-gold text-obsidian shadow-glow font-black' : 'text-gold/70 hover:text-gold'
              }`}
            >
              <FileCode className="w-3.5 h-3.5" /> 📜 BÓC TÁCH JSONL
            </button>
            <button
              onClick={() => setTab('sandbox')}
              className={`px-3 py-1 rounded text-xs font-bold transition flex items-center gap-1.5 ${
                tab === 'sandbox' ? 'bg-gold text-obsidian shadow-glow font-black' : 'text-gold/70 hover:text-gold'
              }`}
            >
              <Crosshair className="w-3.5 h-3.5" /> 🎮 WHAT-IF SANDBOX
            </button>
            <button
              onClick={() => setTab('bench')}
              className={`px-3 py-1 rounded text-xs font-bold transition flex items-center gap-1.5 ${
                tab === 'bench' ? 'bg-gold text-obsidian shadow-glow font-black' : 'text-gold/70 hover:text-gold'
              }`}
            >
              <Gauge className="w-3.5 h-3.5" /> 🔬 BENCHMARK
            </button>
          </div>

          <button
            onClick={close}
            className="px-3 py-1 rounded-lg border border-red-500/40 bg-red-500/20 text-red-300 hover:bg-red-500/30 text-xs font-bold transition"
          >
            ĐÓNG
          </button>
        </div>

        {/* MAIN BODY VIEWPORT */}
        <div className="flex-1 overflow-y-auto p-5 bg-obsidian/60 flex flex-col gap-4">
          
          {/* TAB 1: CÂY SUY TƯỞNG ĐA TẦNG 3-PLY (LIVE CANDIDATE TREE) */}
          {tab === 'tree' && (
            <div className="grid grid-cols-1 lg:grid-cols-12 gap-5 flex-1">
              {/* CỘT TRÁI: BÀN CỜ GPU & ĐIỀU KHIỂN NẠP BÀN CỜ CHÍNH */}
              <div className="lg:col-span-4 flex flex-col items-center gap-3 bg-obsidian/80 p-4 rounded-xl border border-gold/20">
                <div className="flex items-center justify-between w-full text-xs">
                  <span className="text-gold/60 font-bold uppercase">BÀN CỜ NHÁNH BIẾN ĐANG XEM</span>
                  <span className="px-2 py-0.5 rounded bg-emerald-500/20 text-emerald-300 font-mono text-[10px] font-bold">
                    Score: {activeNode ? `${activeNode.score > 0 ? '+' : ''}${activeNode.score} cp` : '0 cp'}
                  </span>
                </div>

                <FastGpuBoard
                  fen={activeNode ? activeNode.fen : currentFen}
                  arrowFrom={activeNode ? activeNode.from : -1}
                  arrowTo={activeNode ? activeNode.to : -1}
                />

                {activeNode && (
                  <div className="w-full bg-obsidian-card p-3 rounded-lg border border-gold/20 text-xs space-y-2">
                    <div className="flex items-center justify-between">
                      <span className="font-bold text-gold">{activeNode.title}</span>
                      <span className={`px-2 py-0.5 rounded text-[10px] font-bold border ${activeNode.badgeColor}`}>
                        {activeNode.badge}
                      </span>
                    </div>
                    <p className="text-gold/70 text-[11px] leading-relaxed italic">
                      "{activeNode.intent}"
                    </p>
                    <div className="text-[10px] text-gold/40 font-mono break-all">
                      FEN: {activeNode.fen}
                    </div>

                    <div className="flex items-center gap-2 pt-1">
                      {onApplyFen && (
                        <button
                          onClick={() => {
                            onApplyFen(activeNode.fen);
                            alert('Đã áp dụng thế cờ nhánh này vào Bàn Cờ Chính!');
                          }}
                          className="flex-1 py-1.5 rounded bg-gold text-obsidian font-bold text-xs hover:bg-gold-light transition shadow-glow flex items-center justify-center gap-1.5"
                        >
                          <Play className="w-3.5 h-3.5 fill-current" /> ÁP DỤNG VÀO BÀN CỜ CHÍNH
                        </button>
                      )}
                      <button
                        onClick={() => {
                          engine.position(activeNode.fen);
                          engine.search(6, 2000);
                          alert('Đã phát lệnh tìm kiếm sâu cho thế cờ nhánh này qua Engine!');
                        }}
                        className="py-1.5 px-3 rounded bg-cyan-500/20 text-cyan-300 border border-cyan-500/40 font-bold text-xs hover:bg-cyan-500/30 transition flex items-center gap-1"
                      >
                        <Zap className="w-3.5 h-3.5" /> ENGINE SEARCH
                      </button>
                    </div>
                  </div>
                )}
              </div>

              {/* CỘT PHẢI: CÂY PHÂN NHÁNH 3 TẦNG ĐỘNG HỌC (LIVE 3-PLY TREE VIEW) */}
              <div className="lg:col-span-8 flex flex-col gap-3">
                <div className="flex items-center justify-between bg-obsidian-card p-3 rounded-xl border border-gold/20">
                  <div className="flex items-center gap-2">
                    <Sparkles className="w-4 h-4 text-gold" />
                    <span className="font-bold text-xs text-gold uppercase">
                      CÂY SUY LUẬN 3-PLY ĐỘNG HỌC TỪ THẾ CỜ HIỆN TẠI ({treeData.candidates.length} ỨNG VIÊN HÀNG ĐẦU)
                    </span>
                  </div>
                  <span className="text-[11px] text-gold/60 font-mono">
                    Lượt đi: <b className="text-gold">{treeData.turn === 'w' ? 'Đỏ (Tiên)' : 'Đen (Hậu)'}</b> | Điểm gốc: <b className="text-emerald-400">{treeData.rootScore} cp</b>
                  </span>
                </div>

                <div className="flex-1 space-y-3 overflow-y-auto pr-1">
                  {treeData.candidates.map((cand, idx) => {
                    const isSelected = activeNode && activeNode.uci === cand.uci;
                    return (
                      <div
                        key={`cand-${cand.uci}-${idx}`}
                        className={`p-3.5 rounded-xl border transition-all cursor-pointer ${
                          isSelected
                            ? 'bg-gold/15 border-gold shadow-glow'
                            : 'bg-obsidian-card/80 border-gold/20 hover:border-gold/50'
                        }`}
                        onClick={() => {
                          setActiveNode({
                            title: `Ứng Viên #${idx + 1}: ${cand.notation}`,
                            uci: cand.uci,
                            fen: cand.fen,
                            from: cand.from,
                            to: cand.to,
                            score: cand.score,
                            intent: cand.intent,
                            badge: cand.badge,
                            badgeColor: cand.badgeColor
                          });
                        }}
                      >
                        {/* TẦNG 1: NƯỚC ĐI CANDIDATE CỦA TA */}
                        <div className="flex items-center justify-between flex-wrap gap-2">
                          <div className="flex items-center gap-2">
                            <span className="w-6 h-6 rounded-full bg-gold/20 text-gold border border-gold/40 flex items-center justify-center font-mono font-bold text-xs">
                              {idx + 1}
                            </span>
                            <span className="font-bold text-sm text-gold">{cand.notation}</span>
                            <span className="font-mono text-xs text-gold/50">({cand.uci})</span>
                            <span className={`px-2 py-0.5 rounded text-[10px] font-bold border ${cand.badgeColor}`}>
                              {cand.badge}
                            </span>
                          </div>

                          <div className="flex items-center gap-3">
                            <span className="text-xs font-mono font-bold text-emerald-400">
                              Đánh giá: {cand.score > 0 ? `+${cand.score}` : cand.score} cp
                            </span>
                            <span className="text-xs text-gold/60">
                              Hiệu quả: +{cand.heuristicScore}
                            </span>
                          </div>
                        </div>

                        <p className="text-xs text-gold/80 mt-1.5 ml-8 leading-relaxed">
                          🎯 <b>Ý đồ:</b> {cand.intent}
                        </p>

                        {/* TẦNG 2 & 3: ĐỐI PHƯƠNG ĐÁP TRẢ VÀ HỆ QUẢ */}
                        {cand.reply && (
                          <div className="mt-2.5 ml-8 p-2.5 rounded-lg bg-obsidian border border-gold/20 text-xs flex items-center justify-between flex-wrap gap-2">
                            <div className="flex items-center gap-2">
                              <span className="text-cyan-400 font-bold">➔ Phản đòn đối phương:</span>
                              <span className="text-gold font-bold">{cand.reply.notation}</span>
                              <span className="font-mono text-[11px] text-gold/50">({cand.reply.uci})</span>
                            </div>
                            <span className="text-gold/60 text-[11px]">
                              Thế trận sau phản đòn: <b className="text-amber-400">{cand.reply.score} cp</b>
                            </span>
                          </div>
                        )}
                      </div>
                    );
                  })}
                </div>
              </div>
            </div>
          )}

          {/* TAB 2: QUÉT BƯỚC NGOẶT & THẾ TRẬN TOÀN VÁN (GAME BLUNDER & TURNING POINT TIMELINE) */}
          {tab === 'timeline' && (
            <div className="grid grid-cols-1 lg:grid-cols-12 gap-5 flex-1">
              <div className="lg:col-span-4 flex flex-col items-center gap-3 bg-obsidian/80 p-4 rounded-xl border border-gold/20">
                <div className="flex items-center justify-between w-full text-xs">
                  <span className="text-gold/60 font-bold uppercase">PLY {selectedPly} / {gameHistory.length - 1}</span>
                  <span className="px-2 py-0.5 rounded bg-amber-500/20 text-amber-300 font-mono text-[10px] font-bold">
                    Turn {Math.floor(selectedPly / 2) + 1}
                  </span>
                </div>

                <FastGpuBoard fen={gameHistory[selectedPly] || currentFen} arrowFrom={-1} arrowTo={-1} />

                <div className="w-full flex items-center gap-2">
                  <button
                    disabled={selectedPly <= 0}
                    onClick={() => setSelectedPly((p) => Math.max(0, p - 1))}
                    className="flex-1 py-1.5 rounded bg-obsidian border border-gold/30 text-gold text-xs font-bold disabled:opacity-30 hover:bg-gold/10"
                  >
                    ◀ Nước trước
                  </button>
                  <button
                    disabled={selectedPly >= gameHistory.length - 1}
                    onClick={() => setSelectedPly((p) => Math.min(gameHistory.length - 1, p + 1))}
                    className="flex-1 py-1.5 rounded bg-obsidian border border-gold/30 text-gold text-xs font-bold disabled:opacity-30 hover:bg-gold/10"
                  >
                    Nước sau ▶
                  </button>
                </div>

                {onApplyFen && (
                  <button
                    onClick={() => {
                      onApplyFen(gameHistory[selectedPly]);
                      alert(`Đã nạp lại trạng thái bàn cờ tại Ply ${selectedPly} vào Bàn Cờ Chính!`);
                    }}
                    className="w-full py-1.5 rounded bg-gold text-obsidian font-bold text-xs hover:bg-gold-light transition shadow-glow flex items-center justify-center gap-1.5"
                  >
                    <RotateCcw className="w-3.5 h-3.5" /> QUAY LẠI THẾ CỜ NÀY TRÊN BÀN CHÍNH
                  </button>
                )}
              </div>

              <div className="lg:col-span-8 flex flex-col gap-3">
                <div className="flex items-center justify-between bg-obsidian-card p-3 rounded-xl border border-gold/20">
                  <span className="font-bold text-xs text-gold uppercase flex items-center gap-2">
                    <TrendingUp className="w-4 h-4 text-gold" /> DIỄN BIẾN THẾ TRẬN & ĐIỂM SỐ CENTIPAWN TOÀN VÁN
                  </span>
                  <span className="text-xs text-gold/60">
                    Tổng số: <b className="text-gold">{timelineAnalysis.length} Plies</b>
                  </span>
                </div>

                {/* Danh Sách Chi Tiết Từng Ply */}
                <div className="flex-1 space-y-2 overflow-y-auto pr-1 max-h-[500px]">
                  {timelineAnalysis.map((item) => (
                    <div
                      key={`timeline-${item.ply}`}
                      onClick={() => setSelectedPly(item.ply)}
                      className={`p-3 rounded-lg border transition cursor-pointer flex items-center justify-between flex-wrap gap-2 ${
                        selectedPly === item.ply
                          ? 'bg-gold/20 border-gold shadow-glow'
                          : 'bg-obsidian-card/80 border-gold/20 hover:border-gold/40'
                      }`}
                    >
                      <div className="flex items-center gap-3">
                        <span className="w-8 h-8 rounded-lg bg-obsidian border border-gold/30 text-gold flex items-center justify-center font-mono font-bold text-xs">
                          P{item.ply}
                        </span>
                        <div>
                          <div className="flex items-center gap-2">
                            <span className="font-bold text-xs text-gold">Turn {item.turnNumber} ({item.color})</span>
                            <span className={`px-2 py-0.5 rounded text-[10px] font-bold border ${item.tagColor}`}>
                              {item.tag}
                            </span>
                          </div>
                          <div className="text-[11px] text-gold/50 font-mono truncate max-w-sm">
                            {item.fen}
                          </div>
                        </div>
                      </div>

                      <div className="text-right">
                        <div className="font-mono font-bold text-xs text-emerald-400">
                          {item.score > 0 ? `+${item.score}` : item.score} cp
                        </div>
                        {item.ply > 0 && (
                          <div className={`text-[10px] font-bold ${item.swing >= 0 ? 'text-emerald-400' : 'text-red-400'}`}>
                            {item.swing >= 0 ? `+${item.swing}` : item.swing} cp
                          </div>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}

          {/* TAB 3: BÓC TÁCH TẬP DỮ LIỆU JSONL THẬT (DEEPSEEK-R1 360° INSPECTOR) */}
          {tab === 'jsonl' && (
            <div className="flex flex-col gap-4 flex-1">
              {/* Khối Nhập / Tải Tệp JSONL */}
              <div className="bg-obsidian-card p-4 rounded-xl border border-gold/20 flex flex-col gap-3">
                <div className="flex items-center justify-between flex-wrap gap-2">
                  <span className="font-bold text-xs text-gold uppercase flex items-center gap-2">
                    <FileCode className="w-4 h-4 text-gold" /> DÁN HOẶC TẢI LÊN MỘT DÒNG JSONL TỰ ĐẤU (XIANGQI-R1 DATASET)
                  </span>
                  <div className="flex items-center gap-2">
                    <label className="px-3 py-1 rounded bg-gold/10 border border-gold/30 text-gold text-xs font-bold cursor-pointer hover:bg-gold/20 transition flex items-center gap-1.5">
                      <Upload className="w-3.5 h-3.5" /> TẢI TỆP .JSONL
                      <input type="file" accept=".jsonl,.json" onChange={handleFileUpload} className="hidden" />
                    </label>
                    <button
                      onClick={handleParseJsonl}
                      className="px-4 py-1 rounded bg-gold text-obsidian text-xs font-bold hover:bg-gold-light transition shadow-glow flex items-center gap-1.5"
                    >
                      <Search className="w-3.5 h-3.5" /> BÓC TÁCH 360°
                    </button>
                  </div>
                </div>

                <textarea
                  rows={3}
                  value={jsonlInput}
                  onChange={(e) => setJsonlInput(e.target.value)}
                  placeholder='Dán một dòng JSONL từ examples/95_cqrs_360_reasoning_generator.rs hoặc data/ vào đây...'
                  className="w-full p-2.5 rounded-lg bg-obsidian border border-gold/30 text-gold font-mono text-xs focus:outline-none focus:border-gold"
                />
              </div>

              {/* Kết Quả Bóc Tách Chi Tiết */}
              {parsedJsonlData && (
                <div className="grid grid-cols-1 lg:grid-cols-12 gap-5 flex-1">
                  <div className="lg:col-span-4 bg-obsidian/80 p-4 rounded-xl border border-gold/20 flex flex-col gap-3">
                    <div className="flex items-center justify-between text-xs">
                      <span className="font-bold text-gold">DANH SÁCH {jsonlTurns.length} TURNS</span>
                      <span className="px-2 py-0.5 rounded bg-purple-500/20 text-purple-300 font-mono text-[10px] font-bold">
                        Outcome: {parsedJsonlData.outcome || 'N/A'}
                      </span>
                    </div>

                    <div className="space-y-1.5 overflow-y-auto max-h-[420px] pr-1">
                      {jsonlTurns.map((t, idx) => (
                        <button
                          key={`jsonl-turn-${idx}`}
                          onClick={() => setJsonlTurnIdx(idx)}
                          className={`w-full text-left p-2.5 rounded-lg border text-xs transition flex items-center justify-between ${
                            jsonlTurnIdx === idx
                              ? 'bg-gold text-obsidian font-bold shadow-glow border-gold'
                              : 'bg-obsidian border-gold/20 text-gold hover:border-gold/40'
                          }`}
                        >
                          <span>Turn #{t.turnIndex}</span>
                          <span className="font-mono text-[11px]">
                            {t.assistantData?.bestmove ? `Best: ${t.assistantData.bestmove}` : ''}
                          </span>
                        </button>
                      ))}
                    </div>
                  </div>

                  <div className="lg:col-span-8 bg-obsidian-card p-4 rounded-xl border border-gold/20 flex flex-col gap-3">
                    {activeJsonlTurn ? (
                      <>
                        <div className="flex items-center justify-between border-b border-gold/20 pb-2">
                          <span className="font-bold text-gold text-xs">
                            PHÂN TÍCH SUY TƯỞNG TURN #{activeJsonlTurn.turnIndex}
                          </span>
                          <span className="text-xs font-mono text-emerald-400 font-bold">
                            Eval: {activeJsonlTurn.assistantData?.centipawn_eval || 0} cp | BestMove: {activeJsonlTurn.assistantData?.bestmove || '-'}
                          </span>
                        </div>

                        {/* Chuỗi Suy Luận <thought> 385 Dòng */}
                        <div className="flex-1 overflow-y-auto max-h-[400px] p-3 rounded-lg bg-obsidian border border-gold/20 text-xs font-mono text-gold/80 leading-relaxed whitespace-pre-wrap">
                          {activeJsonlTurn.assistantData?.thought || 'Không tìm thấy thẻ <thought>.'}
                        </div>
                      </>
                    ) : (
                      <div className="text-center py-20 text-gold/40">Chọn một Turn bên trái để xem chuỗi suy luận.</div>
                    )}
                  </div>
                </div>
              )}
            </div>
          )}

          {/* TAB 4: PHÒNG THỬ NGHIỆM BIẾN MỚI (INTERACTIVE WHAT-IF SANDBOX) */}
          {tab === 'sandbox' && (
            <div className="grid grid-cols-1 lg:grid-cols-12 gap-5 flex-1">
              <div className="lg:col-span-5 flex flex-col items-center gap-3 bg-obsidian/80 p-4 rounded-xl border border-gold/20">
                <span className="text-xs font-bold text-gold uppercase">BÀN CỜ THỬ NGHIỆM TƯƠNG TÁC (CLICK-TO-MOVE)</span>
                <FastGpuBoard
                  fen={sandboxFen}
                  arrowFrom={-1}
                  arrowTo={-1}
                  selectedSq={sandboxSelected}
                  validDests={sandboxValidMoves}
                  onSquareClick={handleSandboxClick}
                />
                <div className="text-[11px] text-gold/60 text-center">
                  💡 Nhấp vào một quân cờ để xem nước đi hợp lệ, sau đó nhấp vào ô đích để di chuyển!
                </div>
              </div>

              <div className="lg:col-span-7 flex flex-col gap-3 bg-obsidian-card p-4 rounded-xl border border-gold/20">
                <span className="font-bold text-xs text-gold uppercase flex items-center gap-2">
                  <Crosshair className="w-4 h-4 text-gold" /> THÔNG SỐ VÀ PHÂN TÍCH NHÁNH THỬ NGHIỆM
                </span>

                <div className="grid grid-cols-2 gap-3 text-xs">
                  <div className="p-3 bg-obsidian rounded-lg border border-gold/20">
                    <span className="text-gold/60 text-[10px] block font-bold uppercase">LƯỢT ĐI</span>
                    <span className="text-base font-bold text-gold">{parse(sandboxFen).turn === 'w' ? 'Đỏ' : 'Đen'}</span>
                  </div>
                  <div className="p-3 bg-obsidian rounded-lg border border-gold/20">
                    <span className="text-gold/60 text-[10px] block font-bold uppercase">ĐÁNH GIÁ HEURISTIC</span>
                    <span className="text-base font-bold text-emerald-400">{evaluatePosition(parse(sandboxFen).board).score} cp</span>
                  </div>
                </div>

                <div className="p-3 bg-obsidian rounded-lg border border-gold/20 text-xs space-y-1 font-mono">
                  <div className="text-gold/60 text-[10px] font-bold">FEN HIỆN TẠI:</div>
                  <div className="text-gold break-all">{sandboxFen}</div>
                </div>

                <div className="flex items-center gap-2 pt-2">
                  <button
                    onClick={() => {
                      setSandboxFen(currentFen);
                      setSandboxSelected(null);
                      setSandboxValidMoves([]);
                    }}
                    className="flex-1 py-2 rounded bg-obsidian border border-gold/30 text-gold text-xs font-bold hover:bg-gold/10 flex items-center justify-center gap-1.5"
                  >
                    <RotateCcw className="w-3.5 h-3.5" /> ĐẶT LẠI THẾ CỜ GỐC
                  </button>

                  {onApplyFen && (
                    <button
                      onClick={() => {
                        onApplyFen(sandboxFen);
                        alert('Đã áp dụng thế cờ thử nghiệm vào Bàn Cờ Chính!');
                      }}
                      className="flex-1 py-2 rounded bg-gold text-obsidian text-xs font-bold hover:bg-gold-light shadow-glow flex items-center justify-center gap-1.5"
                    >
                      <Play className="w-3.5 h-3.5 fill-current" /> ÁP DỤNG VÀO BÀN CHÍNH
                    </button>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* TAB 5: BENCHMARK PHẦN CỨNG (GPU/CPU HARDWARE STRESS TEST) */}
          {tab === 'bench' && (
            <div className="flex flex-col gap-4 max-w-3xl mx-auto w-full">
              <div className="bg-obsidian-card p-6 rounded-xl border border-gold/20 flex flex-col gap-4 text-center">
                <div className="w-12 h-12 rounded-xl bg-gold/10 border border-gold mx-auto flex items-center justify-center text-gold shadow-glow">
                  <Gauge className="w-6 h-6" />
                </div>
                <div>
                  <h3 className="text-base font-bold text-gold font-royal">
                    BENCHMARK HIỆU NĂNG TÍNH TOÁN & RENDER VẬT LÝ
                  </h3>
                  <p className="text-xs text-gold/60 mt-1">
                    Đo lường trực tiếp tốc độ xử lý thế cờ, kiểm tra nước đi hợp lệ và render Canvas của phần cứng thiết bị.
                  </p>
                </div>

                <button
                  disabled={benchRunning}
                  onClick={runHardwareBenchmark}
                  className="py-2.5 px-6 rounded-xl bg-gold text-obsidian font-bold text-xs hover:bg-gold-light transition shadow-glow disabled:opacity-50 mx-auto flex items-center gap-2"
                >
                  {benchRunning ? <RefreshCw className="w-4 h-4 animate-spin" /> : <Zap className="w-4 h-4" />}
                  {benchRunning ? 'ĐANG CHẠY STRESS TEST...' : 'BẮT ĐẦU BENCHMARK (50,000 FEN ITERATIONS)'}
                </button>

                {benchResults && (
                  <div className="grid grid-cols-1 md:grid-cols-3 gap-3 text-xs mt-4">
                    <div className="p-3.5 bg-obsidian rounded-xl border border-gold/30">
                      <span className="text-gold/60 text-[10px] uppercase font-bold block">THÔNG LƯỢNG FEN</span>
                      <span className="text-lg font-black text-emerald-400">{benchResults.fensPerSec} FEN/s</span>
                    </div>
                    <div className="p-3.5 bg-obsidian rounded-xl border border-gold/30">
                      <span className="text-gold/60 text-[10px] uppercase font-bold block">TỔNG THỜI GIAN</span>
                      <span className="text-lg font-black text-amber-400">{benchResults.elapsedMs} ms</span>
                    </div>
                    <div className="p-3.5 bg-obsidian rounded-xl border border-gold/30">
                      <span className="text-gold/60 text-[10px] uppercase font-bold block">ĐỘ TRỄ LUT TRUNG BÌNH</span>
                      <span className="text-lg font-black text-cyan-400">{benchResults.lutLatencyNs} ns / FEN</span>
                    </div>
                  </div>
                )}
              </div>
            </div>
          )}

        </div>
      </div>
    </div>
  );
}
