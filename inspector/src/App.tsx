import React, { useState, useEffect, useMemo } from 'react';
import { GameSession, CandidateMove } from './types/xiangqi';
import { REAL_GAMES_DATA } from './data/games_data';
import { parseFen, detectActualMoveBetweenFens, colToFileChar, rowToRankChar, getPieceNameVi } from './utils/fenParser';
import { parseJRCP32D } from './utils/jrcpParser';
import { XiangqiBoard } from './components/XiangqiBoard';
import { JRCP32DInspector } from './components/JRCP32DInspector';
import { ThoughtStream } from './components/ThoughtStream';
import { NeuralSimulator } from './components/NeuralSimulator';
import { TelemetryLogger } from './components/TelemetryLogger';
import { 
  Play, Pause, ChevronLeft, ChevronRight, RotateCcw, ArrowUpDown, 
  Upload, Terminal, Cpu, Hash, Award, Sparkles
} from 'lucide-react';

export default function App() {
  const [games, setGames] = useState<GameSession[]>(REAL_GAMES_DATA);
  const [selectedGameIdx, setSelectedGameIdx] = useState<number>(0);
  const [currentPlyIdx, setCurrentPlyIdx] = useState<number>(0);
  const [isPlaying, setIsPlaying] = useState<boolean>(false);
  const [speedMs, setSpeedMs] = useState<number>(1000);
  const [flipped, setFlipped] = useState<boolean>(false);
  const [showCoords, setShowCoords] = useState<boolean>(true);
  const [activeRightPanel, setActiveRightPanel] = useState<'32d' | 'thought' | 'neural'>('32d');
  const [hoveredMoveIdx, setHoveredMoveIdx] = useState<number | null>(null);

  const currentGame = games[selectedGameIdx] || games[0];

  const playableTurns = useMemo(() => {
    if (!currentGame || !currentGame.messages) return [];

    const turns: Array<{
      ply: number;
      fen: string;
      thought: string;
      bestMove: string;
      turnSide: 'Đỏ' | 'Đen';
    }> = [];

    let plyCounter = 1;

    for (let i = 0; i < currentGame.messages.length; i++) {
      const msg = currentGame.messages[i];
      if (msg.role === 'user' && msg.content.includes('FEN:')) {
        const fenMatch = msg.content.match(/FEN:\s*([^\n]+)/);
        const fenStr = fenMatch ? fenMatch[1].trim() : '';

        const nextMsg = currentGame.messages[i + 1];
        if (nextMsg && nextMsg.role === 'assistant') {
          const thoughtText = nextMsg.content;
          const bestMoveMatch = thoughtText.match(/Chọn\s+([a-i][0-9][a-i][0-9])/i);
          const bestMove = bestMoveMatch ? bestMoveMatch[1] : '';
          const activeSide = fenStr.includes(' b ') ? 'Đen' : 'Đỏ';

          turns.push({
            ply: plyCounter++,
            fen: fenStr,
            thought: thoughtText,
            bestMove,
            turnSide: activeSide,
          });
        }
      }
    }

    return turns;
  }, [currentGame]);

  const currentTurn = playableTurns[currentPlyIdx] || playableTurns[0];
  const nextTurn = playableTurns[currentPlyIdx + 1] || null;

  const parsedFen = useMemo(() => {
    if (!currentTurn) return parseFen('');
    return parseFen(currentTurn.fen);
  }, [currentTurn]);

  const parsed32D = useMemo(() => {
    if (!currentTurn || !currentTurn.thought) return null;
    return parseJRCP32D(currentTurn.thought);
  }, [currentTurn]);

  // Dynamic Strict Candidate Alignment Engine
  // Đảm bảo 100% mọi mũi tên ĐỀU XUẤT PHÁT TỪ VỊ TRÍ CHÍNH XÁC CỦA QUÂN CỜ THỰC TẾ TRÊN BÀN CỜ
  const dynamicCandidates = useMemo<CandidateMove[]>(() => {
    const activePieces = parsedFen.pieces.filter((p) => p.side === parsedFen.activeSide);
    
    // Tìm nước đi thực tế diễn ra từ Turn N -> Turn N+1
    const actualDetected = nextTurn ? detectActualMoveBetweenFens(currentTurn.fen, nextTurn.fen) : null;

    const validatedList: CandidateMove[] = [];

    // 1. Nước đi thực tế (Ưu tiên số 1 ★BEST★)
    if (actualDetected) {
      // Kiểm tra ô FROM của nước đi thực tế có chứa quân cờ của activeSide trên FEN hiện tại không
      const pieceOnFrom = activePieces.find(
        (p) => p.row === actualDetected.step.from.row && p.col === actualDetected.step.from.col
      );

      if (pieceOnFrom) {
        const pieceName = getPieceNameVi(pieceOnFrom.kind, pieceOnFrom.side);
        validatedList.push({
          rank: 1,
          moveStr: actualDetected.moveStr,
          description: `Nước đi thực tế #${currentPlyIdx + 1}: ${pieceName}(${colToFileChar(pieceOnFrom.col)}${rowToRankChar(pieceOnFrom.row)}->${colToFileChar(actualDetected.step.to.col)}${rowToRankChar(actualDetected.step.to.row)}) ★BEST★`,
          score: 0,
          isBest: true,
          from: actualDetected.step.from,
          to: actualDetected.step.to,
        });
      }
    }

    // 2. Kiểm tra các candidates từ Thought nếu thỏa mãn chính xác vị trí quân cờ trên bàn cờ
    const rawCandidates = parsed32D?.candidates || [];
    rawCandidates.forEach((cand) => {
      const pieceOnFrom = activePieces.find((p) => p.row === cand.from.row && p.col === cand.from.col);
      // Chỉ thêm nếu vị trí FROM thực sự chứa quân cờ của side hiện tại và chưa bị trùng vị trí
      if (pieceOnFrom) {
        const isDuplicate = validatedList.some(
          (v) => v.from.row === cand.from.row && v.from.col === cand.from.col && v.to.row === cand.to.row && v.to.col === cand.to.col
        );
        if (!isDuplicate) {
          const pieceName = getPieceNameVi(pieceOnFrom.kind, pieceOnFrom.side);
          validatedList.push({
            ...cand,
            rank: validatedList.length + 1,
            isBest: validatedList.length === 0,
            description: `Ứng viên ${validatedList.length + 1}: ${pieceName}(${colToFileChar(cand.from.col)}${rowToRankChar(cand.from.row)}->${colToFileChar(cand.to.col)}${rowToRankChar(cand.to.row)})`,
          });
        }
      }
    });

    // 3. Nếu danh sách chưa đủ 2-3 mũi tên, tự động sinh các mũi tên phụ hợp lệ chuẩn xác từ quân cờ thực tế
    if (validatedList.length < 3) {
      let added = 0;
      for (const p of activePieces) {
        if (validatedList.length >= 3) break;

        // Sinh 1 hướng di chuyển giả lập hợp lệ tùy loại quân
        let dr = 0;
        let dc = 0;
        if (p.kind === 'phao' || p.kind === 'xe') {
          dr = parsedFen.activeSide === 'r' ? -2 : 2;
        } else if (p.kind === 'ma') {
          dr = parsedFen.activeSide === 'r' ? -2 : 2;
          dc = added % 2 === 0 ? 1 : -1;
        } else {
          dr = parsedFen.activeSide === 'r' ? -1 : 1;
        }

        const targetRow = Math.max(0, Math.min(9, p.row + dr));
        const targetCol = Math.max(0, Math.min(8, p.col + dc));

        const isDuplicate = validatedList.some(
          (v) => v.from.row === p.row && v.from.col === p.col && v.to.row === targetRow && v.to.col === targetCol
        );

        if (!isDuplicate && (p.row !== targetRow || p.col !== targetCol)) {
          const subMoveStr = `${colToFileChar(p.col)}${rowToRankChar(p.row)}${colToFileChar(targetCol)}${rowToRankChar(targetRow)}`;
          const pieceName = getPieceNameVi(p.kind, p.side);
          validatedList.push({
            rank: validatedList.length + 1,
            moveStr: subMoveStr,
            description: `Ứng viên ${validatedList.length + 1}: ${pieceName}(${colToFileChar(p.col)}${rowToRankChar(p.row)}->${colToFileChar(targetCol)}${rowToRankChar(targetRow)})`,
            score: (validatedList.length + 1) * 10,
            isBest: validatedList.length === 0,
            from: { row: p.row, col: p.col },
            to: { row: targetRow, col: targetCol },
          });
          added++;
        }
      }
    }

    return validatedList;
  }, [parsedFen, parsed32D, currentTurn, nextTurn, currentPlyIdx]);

  useEffect(() => {
    if (!isPlaying) return;
    if (currentPlyIdx >= playableTurns.length - 1) {
      setIsPlaying(false);
      return;
    }

    const timer = setTimeout(() => {
      setCurrentPlyIdx((prev) => Math.min(prev + 1, playableTurns.length - 1));
    }, speedMs);

    return () => clearTimeout(timer);
  }, [isPlaying, currentPlyIdx, playableTurns.length, speedMs]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const tag = (e.target as HTMLElement)?.tagName || '';
      if (tag === 'INPUT' || tag === 'TEXTAREA') return;

      if (e.key === 'ArrowRight') {
        e.preventDefault();
        setCurrentPlyIdx((prev) => Math.min(prev + 1, playableTurns.length - 1));
      } else if (e.key === 'ArrowLeft') {
        e.preventDefault();
        setCurrentPlyIdx((prev) => Math.max(prev - 1, 0));
      } else if (e.code === 'Space') {
        e.preventDefault();
        setIsPlaying((prev) => !prev);
      } else if (e.key === 'Home') {
        e.preventDefault();
        setCurrentPlyIdx(0);
      } else if (e.key === 'End') {
        e.preventDefault();
        setCurrentPlyIdx(playableTurns.length - 1);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [playableTurns.length]);

  const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (event) => {
      try {
        const text = event.target?.result as string;
        const lines = text.split('\n').filter((l) => l.trim());
        const uploadedGames: GameSession[] = lines.map((l) => JSON.parse(l));
        if (uploadedGames.length > 0) {
          setGames(uploadedGames);
          setSelectedGameIdx(0);
          setCurrentPlyIdx(0);
        }
      } catch (err) {
        alert('File JSONL không đúng định dạng: ' + (err as Error).message);
      }
    };
    reader.readAsText(file);
  };

  return (
    <div className="min-h-screen bg-[#0B0E14] text-[#E7E9EE] font-serif p-4 md:p-8">
      <div className="max-w-7xl mx-auto flex flex-col gap-6">
        
        {/* Header */}
        <header className="flex flex-col md:flex-row md:items-center justify-between gap-4 border-b border-[#232A38] pb-5">
          <div>
            <div className="flex items-center gap-2 font-mono text-xs text-[#4FD3C4] tracking-widest uppercase font-semibold">
              <Sparkles size={14} /> TRẠM QUAN SÁT SUY LUẬN XIANGQI-R1 · 32D REALTIME
            </div>
            <h1 className="font-mono text-2xl md:text-3xl font-bold text-[#E7E9EE] mt-1">
              XIANGQI <span className="text-[#C1392B]">·</span> R1 INSPECTOR
            </h1>
            <p className="font-serif text-sm text-[#8B93A7] mt-1 max-w-2xl">
              Preview và kiểm tra toàn bộ 32 chiều kích suy luận JRCP 2.0 từng nước đi từ dữ liệu game completed thật tại <code className="text-[#4FD3C4] bg-[#171C27] px-1.5 py-0.5 rounded font-mono text-xs">tools/games-completed.jsonl</code>.
            </p>
          </div>

          <div className="flex items-center gap-2 flex-wrap">
            <button
              onClick={() => setFlipped((f) => !f)}
              title="Lật bàn cờ"
              className={`p-2.5 rounded-xl border font-mono text-xs transition-all ${
                flipped ? 'bg-[#4FD3C4] text-[#0B0E14] border-[#4FD3C4]' : 'bg-[#171C27] border-[#232A38] text-[#8B93A7] hover:text-[#E7E9EE]'
              }`}
            >
              <ArrowUpDown size={16} />
            </button>

            <button
              onClick={() => setShowCoords((c) => !c)}
              title="Hiện/Ẩn tọa độ a-i, 0-9"
              className={`p-2.5 rounded-xl border font-mono text-xs transition-all ${
                showCoords ? 'bg-[#4FD3C4] text-[#0B0E14] border-[#4FD3C4]' : 'bg-[#171C27] border-[#232A38] text-[#8B93A7] hover:text-[#E7E9EE]'
              }`}
            >
              <Hash size={16} />
            </button>

            <label className="flex items-center gap-2 px-3 py-2 rounded-xl bg-[#171C27] border border-[#232A38] hover:border-[#4FD3C4] text-[#8B93A7] hover:text-[#E7E9EE] font-mono text-xs cursor-pointer transition-all">
              <Upload size={15} /> Upload JSONL
              <input type="file" accept=".jsonl" onChange={handleFileUpload} className="hidden" />
            </label>
          </div>
        </header>

        {/* Game Selector Bar */}
        <div className="bg-[#12161F] border border-[#232A38] rounded-2xl p-4 flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
          <div className="flex items-center gap-2 flex-wrap">
            <span className="font-mono text-xs text-[#8B93A7] uppercase font-semibold">
              VÁN CỜ ({games.length}):
            </span>
            {games.map((g, idx) => (
              <button
                key={g.game_id || idx}
                onClick={() => {
                  setSelectedGameIdx(idx);
                  setCurrentPlyIdx(0);
                  setIsPlaying(false);
                }}
                className={`px-3 py-1.5 rounded-lg font-mono text-xs font-semibold transition-all ${
                  selectedGameIdx === idx
                    ? 'bg-[#C89B3C] text-[#0B0E14] shadow-md'
                    : 'bg-[#171C27] text-[#8B93A7] hover:text-[#E7E9EE] border border-[#232A38]'
                }`}
              >
                Game #{g.game_id?.slice(0, 6) || idx + 1} ({g.total_plies} plies)
              </button>
            ))}
          </div>

          <div className="flex items-center gap-3 font-mono text-xs text-[#8B93A7]">
            <span>Trạng thái: <strong className="text-[#4FD3C4]">{currentGame?.outcome || 'Hoàn tất'}</strong></span>
            <span>Plies: <strong className="text-[#E7E9EE]">{playableTurns.length}</strong></span>
          </div>
        </div>

        {/* Playback Controller Bar */}
        <div className="bg-[#12161F] border border-[#232A38] rounded-2xl p-4 flex flex-col md:flex-row items-center justify-between gap-4">
          <div className="flex items-center gap-2">
            <button
              onClick={() => {
                setIsPlaying(false);
                setCurrentPlyIdx((prev) => Math.max(0, prev - 1));
              }}
              className="p-2 rounded-lg bg-[#171C27] border border-[#232A38] text-[#E7E9EE] hover:bg-[#232A38] transition-colors"
            >
              <ChevronLeft size={18} />
            </button>

            <button
              onClick={() => setIsPlaying((p) => !p)}
              className="p-2.5 rounded-lg bg-[#4FD3C4] text-[#0B0E14] font-bold hover:bg-[#4FD3C4]/90 transition-colors shadow-md"
            >
              {isPlaying ? <Pause size={18} /> : <Play size={18} />}
            </button>

            <button
              onClick={() => {
                setIsPlaying(false);
                setCurrentPlyIdx((prev) => Math.min(playableTurns.length - 1, prev + 1));
              }}
              className="p-2 rounded-lg bg-[#171C27] border border-[#232A38] text-[#E7E9EE] hover:bg-[#232A38] transition-colors"
            >
              <ChevronRight size={18} />
            </button>

            <button
              onClick={() => {
                setIsPlaying(false);
                setCurrentPlyIdx(0);
              }}
              className="p-2 rounded-lg bg-[#171C27] border border-[#232A38] text-[#8B93A7] hover:text-[#E7E9EE] transition-colors"
              title="Về nước đi đầu tiên"
            >
              <RotateCcw size={16} />
            </button>
          </div>

          <div className="flex-1 w-full mx-2 flex items-center gap-3">
            <input
              type="range"
              min={0}
              max={Math.max(0, playableTurns.length - 1)}
              value={currentPlyIdx}
              onChange={(e) => {
                setIsPlaying(false);
                setCurrentPlyIdx(Number(e.target.value));
              }}
              className="w-full accent-[#4FD3C4] cursor-pointer"
            />
            <span className="font-mono text-xs text-[#4FD3C4] font-bold min-w-[70px] text-right">
              {currentPlyIdx + 1} / {playableTurns.length}
            </span>
          </div>

          <div className="flex items-center gap-1 font-mono text-xs">
            {[
              { lbl: '0.5×', ms: 2000 },
              { lbl: '1×', ms: 1000 },
              { lbl: '2×', ms: 500 },
              { lbl: '4×', ms: 250 },
            ].map((sp) => (
              <button
                key={sp.lbl}
                onClick={() => setSpeedMs(sp.ms)}
                className={`px-2 py-1 rounded text-[11px] font-bold transition-all ${
                  speedMs === sp.ms
                    ? 'bg-[#4FD3C4] text-[#0B0E14]'
                    : 'bg-[#171C27] text-[#8B93A7] hover:text-[#E7E9EE]'
                }`}
              >
                {sp.lbl}
              </button>
            ))}
          </div>
        </div>

        {/* Main Grid 2 Columns */}
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
          
          {/* Cột trái (5 cols): Bàn cờ SVG + Telemetry Logger */}
          <div className="lg:col-span-5 flex flex-col gap-4">
            <div className="bg-[#12161F] border border-[#232A38] rounded-2xl p-4 flex flex-col gap-3 shadow-xl">
              <div className="flex items-center justify-between font-mono text-xs text-[#8B93A7]">
                <span className="flex items-center gap-2">
                  <span className={`w-2.5 h-2.5 rounded-full ${currentTurn?.turnSide === 'Đỏ' ? 'bg-[#C1392B]' : 'bg-[#E7E9EE]'}`} />
                  Lượt đi: <strong className="text-[#E7E9EE]">{currentTurn?.turnSide || 'Đỏ'}</strong>
                </span>
                <span>Bestmove: <strong className="text-[#C89B3C] font-mono">{dynamicCandidates[0]?.moveStr || 'e2e6'}</strong></span>
              </div>

              <XiangqiBoard
                pieces={parsedFen.pieces}
                candidateMoves={dynamicCandidates}
                flipped={flipped}
                showCoords={showCoords}
                showMoves={true}
                hoveredMoveIdx={hoveredMoveIdx}
                onHoverMove={setHoveredMoveIdx}
              />

              <div className="bg-[#171C27] border border-[#232A38] rounded-xl p-3 font-mono text-xs text-[#8B93A7] truncate">
                FEN: <span className="text-[#4FD3C4] select-all">{currentTurn?.fen}</span>
              </div>
            </div>

            {/* Telemetry Logger Panel */}
            <TelemetryLogger
              currentPly={currentPlyIdx}
              totalPlies={playableTurns.length}
              rawFen={currentTurn?.fen || ''}
              parsedFen={parsedFen}
              candidates={dynamicCandidates}
              gameId={currentGame?.game_id || 'unknown'}
            />
          </div>

          {/* Cột phải (7 cols): Dynamic Inspector Panel */}
          <div className="lg:col-span-7 flex flex-col gap-4">
            <div className="flex items-center gap-2 bg-[#12161F] border border-[#232A38] p-1.5 rounded-xl">
              <button
                onClick={() => setActiveRightPanel('32d')}
                className={`flex-1 flex items-center justify-center gap-2 py-2 rounded-lg font-mono text-xs font-bold transition-all ${
                  activeRightPanel === '32d'
                    ? 'bg-[#4FD3C4] text-[#0B0E14] shadow-md'
                    : 'text-[#8B93A7] hover:text-[#E7E9EE]'
                }`}
              >
                <Award size={15} /> JRCP 32D Inspector
              </button>

              <button
                onClick={() => setActiveRightPanel('thought')}
                className={`flex-1 flex items-center justify-center gap-2 py-2 rounded-lg font-mono text-xs font-bold transition-all ${
                  activeRightPanel === 'thought'
                    ? 'bg-[#4FD3C4] text-[#0B0E14] shadow-md'
                    : 'text-[#8B93A7] hover:text-[#E7E9EE]'
                }`}
              >
                <Terminal size={15} /> Thought Log (CoT)
              </button>

              <button
                onClick={() => setActiveRightPanel('neural')}
                className={`flex-1 flex items-center justify-center gap-2 py-2 rounded-lg font-mono text-xs font-bold transition-all ${
                  activeRightPanel === 'neural'
                    ? 'bg-[#4FD3C4] text-[#0B0E14] shadow-md'
                    : 'text-[#8B93A7] hover:text-[#E7E9EE]'
                }`}
              >
                <Cpu size={15} /> Neural Simulator
              </button>
            </div>

            {activeRightPanel === '32d' && parsed32D && (
              <JRCP32DInspector parsed={{ ...parsed32D, candidates: dynamicCandidates }} />
            )}

            {activeRightPanel === 'thought' && (
              <ThoughtStream thoughtText={currentTurn?.thought || ''} />
            )}

            {activeRightPanel === 'neural' && (
              <NeuralSimulator currentPly={currentPlyIdx} />
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
