import React, { useRef } from 'react';
import { BoardPiece, CandidateMove, MoveStep, PositionSquare } from '../types/xiangqi';
import { KIND_CHAR, getPieceNameVi } from '../utils/fenParser';

interface XiangqiBoardProps {
  pieces: BoardPiece[];
  lastMove?: MoveStep | null;
  candidateMoves?: CandidateMove[];
  flipped?: boolean;
  showCoords?: boolean;
  showMoves?: boolean;
  hoveredMoveIdx?: number | null;
  onHoverMove?: (idx: number | null) => void;
  queryPoint?: PositionSquare | null;
  onSelectQuery?: (row: number, col: number) => void;
}

const COLS = 9;
const ROWS = 10;
const CELL = 44;
const MARGIN = 30;
const BW = MARGIN * 2 + (COLS - 1) * CELL;
const BH = MARGIN * 2 + (ROWS - 1) * CELL;

const px = (c: number) => MARGIN + c * CELL;
const py = (r: number) => MARGIN + r * CELL;

const flipPt = (r: number, c: number, flipped: boolean) =>
  flipped ? { r: ROWS - 1 - r, c: COLS - 1 - c } : { r, c };

export const XiangqiBoard: React.FC<XiangqiBoardProps> = ({
  pieces,
  lastMove,
  candidateMoves = [],
  flipped = false,
  showCoords = true,
  showMoves = true,
  hoveredMoveIdx = null,
  onHoverMove,
  queryPoint = null,
  onSelectQuery,
}) => {
  const svgRef = useRef<SVGSVGElement>(null);
  const vlines = Array.from({ length: COLS }, (_, c) => c);
  const topRows = [0, 1, 2, 3, 4];
  const botRows = [5, 6, 7, 8, 9];

  const P = (r: number, c: number) => flipPt(r, c, flipped);

  return (
    <div className="relative w-full max-w-[560px] mx-auto select-none">
      <svg
        ref={svgRef}
        viewBox={`0 0 ${BW + (showCoords ? 16 : 0)} ${BH + (showCoords ? 16 : 0)}`}
        className="w-full h-auto drop-shadow-2xl rounded-2xl border border-[#232A38] bg-[#12161F] p-2"
        role="img"
        aria-label="Bàn cờ tướng 10x9"
      >
        <defs>
          <marker
            id="bestArrow"
            markerWidth="10"
            markerHeight="10"
            refX="7"
            refY="3.5"
            orient="auto"
          >
            <polygon points="0 0, 8 3.5, 0 7" fill="#C89B3C" />
          </marker>
          <marker
            id="subArrow"
            markerWidth="10"
            markerHeight="10"
            refX="7"
            refY="3.5"
            orient="auto"
          >
            <polygon points="0 0, 8 3.5, 0 7" fill="#4FD3C4" />
          </marker>
        </defs>

        {/* Nền bàn cờ */}
        <rect x="0" y="0" width={BW} height={BH} fill="#12161F" rx="12" />

        {/* Các đường kẻ bàn cờ */}
        {vlines.map((c) => (
          <line
            key={`v${c}`}
            x1={px(c)}
            y1={py(0)}
            x2={px(c)}
            y2={py(9)}
            stroke="#232A38"
            strokeWidth="1.5"
          />
        ))}
        {topRows.map((r) => (
          <line
            key={`ht${r}`}
            x1={px(0)}
            y1={py(r)}
            x2={px(8)}
            y2={py(r)}
            stroke="#232A38"
            strokeWidth="1.5"
          />
        ))}
        {botRows.map((r) => (
          <line
            key={`hb${r}`}
            x1={px(0)}
            y1={py(r)}
            x2={px(8)}
            y2={py(r)}
            stroke="#232A38"
            strokeWidth="1.5"
          />
        ))}

        {/* Cửu cung Đen (hàng 0..2, cột 3..5) */}
        <line x1={px(3)} y1={py(0)} x2={px(5)} y2={py(2)} stroke="#232A38" strokeWidth="1.5" />
        <line x1={px(5)} y1={py(0)} x2={px(3)} y2={py(2)} stroke="#232A38" strokeWidth="1.5" />

        {/* Cửu cung Đỏ (hàng 7..9, cột 3..5) */}
        <line x1={px(3)} y1={py(7)} x2={px(5)} y2={py(9)} stroke="#232A38" strokeWidth="1.5" />
        <line x1={px(5)} y1={py(7)} x2={px(3)} y2={py(9)} stroke="#232A38" strokeWidth="1.5" />

        {/* Sông Hà (Sở Hà Hán Giới) */}
        <text
          x={BW / 2}
          y={(py(4) + py(5)) / 2 + 5}
          textAnchor="middle"
          fontSize="14"
          letterSpacing="10"
          fill="#4B5364"
          className="font-serif font-medium"
        >
          楚 河　　漢 界
        </text>

        {/* Tọa độ cột a..i */}
        {showCoords &&
          vlines.map((c) => (
            <text
              key={`cl${c}`}
              x={px(c)}
              y={BH + 12}
              textAnchor="middle"
              fontSize="9"
              className="font-mono fill-[#8B93A7]"
            >
              {String.fromCharCode('a'.charCodeAt(0) + c)}
            </text>
          ))}

        {/* Tọa độ hàng 0..9 */}
        {showCoords &&
          Array.from({ length: ROWS }, (_, r) => (
            <text
              key={`rl${r}`}
              x={-10}
              y={py(r) + 3}
              textAnchor="middle"
              fontSize="9"
              className="font-mono fill-[#8B93A7]"
            >
              {9 - r}
            </text>
          ))}

        {/* Nước đi vừa qua (Last move highlight) */}
        {lastMove && (() => {
          const a = P(lastMove.from.row, lastMove.from.col);
          const b = P(lastMove.to.row, lastMove.to.col);
          return (
            <g>
              <circle cx={px(a.c)} cy={py(a.r)} r="18" fill="none" stroke="#C1392B" strokeWidth="2" strokeDasharray="4 4" opacity="0.7" />
              <line
                x1={px(a.c)}
                y1={py(a.r)}
                x2={px(b.c)}
                y2={py(b.r)}
                stroke="#C1392B"
                strokeWidth="2.5"
                strokeDasharray="4 4"
                opacity="0.8"
              />
              <circle cx={px(b.c)} cy={py(b.r)} r="18" fill="none" stroke="#C1392B" strokeWidth="2.5" />
            </g>
          );
        })()}

        {/* Mũi tên Nước đi ứng viên (Candidates Moves Visualizer) */}
        {showMoves &&
          candidateMoves.map((m, i) => {
            const a = P(m.from.row, m.from.col);
            const b = P(m.to.row, m.to.col);
            const hovered = hoveredMoveIdx === i;
            const strokeColor = m.isBest ? '#C89B3C' : '#4FD3C4';
            const markerId = m.isBest ? 'url(#bestArrow)' : 'url(#subArrow)';

            return (
              <g key={`cand_${i}`}>
                {/* Vùng tương tác trong suốt */}
                <line
                  x1={px(a.c)}
                  y1={py(a.r)}
                  x2={px(b.c)}
                  y2={py(b.r)}
                  stroke="transparent"
                  strokeWidth="24"
                  strokeLinecap="round"
                  className="cursor-pointer"
                  onMouseEnter={() => onHoverMove && onHoverMove(i)}
                  onMouseLeave={() => onHoverMove && onHoverMove(null)}
                />
                <line
                  x1={px(a.c)}
                  y1={py(a.r)}
                  x2={px(b.c)}
                  y2={py(b.r)}
                  stroke={strokeColor}
                  strokeWidth={hovered ? 3.5 : m.isBest ? 2.5 : 1.8}
                  opacity={hovered ? 1 : m.isBest ? 0.9 : 0.6}
                  strokeLinecap="round"
                  markerEnd={markerId}
                  className="pointer-events-none transition-all duration-200"
                />
                {/* Badge điểm centipawn / BEST */}
                <g transform={`translate(${(px(a.c) + px(b.c)) / 2}, ${(py(a.r) + py(b.r)) / 2})`}>
                  <rect
                    x="-20"
                    y="-10"
                    width="40"
                    height="18"
                    rx="4"
                    fill="#171C27"
                    stroke={strokeColor}
                    strokeWidth="1"
                    opacity="0.9"
                  />
                  <text
                    textAnchor="middle"
                    dominantBaseline="central"
                    fontSize="9"
                    fontWeight="700"
                    fill={strokeColor}
                    className="font-mono pointer-events-none"
                  >
                    {m.isBest ? '★BEST' : `${m.score}cp`}
                  </text>
                </g>
              </g>
            );
          })}

        {/* Danh sách các Quân cờ (Pieces) */}
        {pieces.map((piece, i) => {
          const p = P(piece.row, piece.col);
          const isQuery =
            queryPoint && queryPoint.row === piece.row && queryPoint.col === piece.col;

          return (
            <g
              key={`piece_${i}`}
              transform={`translate(${px(p.c)},${py(p.r)})`}
              onClick={onSelectQuery ? () => onSelectQuery(piece.row, piece.col) : undefined}
              className={onSelectQuery ? 'cursor-pointer group' : ''}
            >
              <title>
                {getPieceNameVi(piece.kind, piece.side)} · {piece.side === 'r' ? 'Đỏ' : 'Đen'}
              </title>

              {/* Vòng sáng Highlight khi chọn */}
              {isQuery && (
                <circle r="21" fill="none" stroke="#C89B3C" strokeWidth="3" className="animate-pulse" />
              )}

              {/* Thân quân cờ */}
              <circle
                r="17"
                fill="#E8E4D9"
                stroke={piece.side === 'r' ? '#C1392B' : '#1C1E24'}
                strokeWidth={isQuery ? 3 : 2}
                className="drop-shadow-md transition-transform duration-150 group-hover:scale-105"
              />
              <circle
                r="14"
                fill="none"
                stroke={piece.side === 'r' ? '#C1392B' : '#1C1E24'}
                strokeWidth="1"
                opacity="0.3"
              />

              {/* Ký tự Hán tự trên quân cờ */}
              <text
                textAnchor="middle"
                dominantBaseline="central"
                fontSize="17"
                fontWeight="700"
                fill={piece.side === 'r' ? '#C1392B' : '#1C1E24'}
                className="font-serif pointer-events-none"
              >
                {KIND_CHAR[piece.kind][piece.side]}
              </text>
            </g>
          );
        })}
      </svg>
    </div>
  );
};
