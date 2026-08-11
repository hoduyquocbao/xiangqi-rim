import React, { useState } from 'react';
import { Activity, Copy, Check, AlertTriangle, CheckCircle2, RefreshCw, Terminal } from 'lucide-react';
import { ParsedFenResult } from '../utils/fenParser';
import { CandidateMove } from '../types/xiangqi';

interface TelemetryLoggerProps {
  currentPly: number;
  totalPlies: number;
  rawFen: string;
  parsedFen: ParsedFenResult;
  candidates: CandidateMove[];
  gameId: string;
}

export const TelemetryLogger: React.FC<TelemetryLoggerProps> = ({
  currentPly,
  totalPlies,
  rawFen,
  parsedFen,
  candidates,
  gameId,
}) => {
  const [copied, setCopied] = useState(false);
  const [showFullLogs, setShowFullLogs] = useState(false);

  const logs: string[] = [
    `[SYS] Game ID: ${gameId} | Current Ply: ${currentPly + 1}/${totalPlies}`,
    `[FEN RAW] ${rawFen}`,
    `[FEN STATUS] ${parsedFen.status.toUpperCase()} | Pieces Extracted: ${parsedFen.pieces.length} | Active Turn: ${parsedFen.turn}`,
  ];

  if (parsedFen.warnings.length > 0) {
    parsedFen.warnings.forEach((w) => logs.push(`[FEN WARN] ${w}`));
  }

  if (candidates.length > 0) {
    logs.push(`[CANDIDATES] Extracted ${candidates.length} candidate moves: ${candidates.map((c) => `${c.moveStr} (${c.score}cp)`).join(', ')}`);
  } else {
    logs.push(`[CANDIDATES WARN] No candidate moves extracted for this turn.`);
  }

  const logTextToCopy = logs.join('\n');

  const handleCopyLogs = () => {
    navigator.clipboard.writeText(logTextToCopy);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="bg-[#12161F] border border-[#232A38] rounded-2xl p-4 flex flex-col gap-3 shadow-xl">
      {/* Telemetry Header */}
      <div className="flex items-center justify-between flex-wrap gap-2 border-b border-[#232A38] pb-3">
        <div className="flex items-center gap-2 font-mono text-xs font-bold text-[#E7E9EE]">
          <Activity size={15} className="text-[#4FD3C4]" />
          TELEMETRY, LOGGER & METRICS PANEL
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={() => setShowFullLogs((s) => !s)}
            className="flex items-center gap-1.5 px-2.5 py-1 rounded-lg font-mono text-xs bg-[#171C27] text-[#8B93A7] border border-[#232A38] hover:text-[#E7E9EE] transition-colors"
          >
            <Terminal size={13} /> {showFullLogs ? 'Ẩn Log Chi Tiết' : 'Xem Log Chi Tiết'}
          </button>

          <button
            onClick={handleCopyLogs}
            className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg font-mono text-xs font-bold bg-[#4FD3C4] text-[#0B0E14] shadow-md hover:bg-[#4FD3C4]/90 transition-all"
          >
            {copied ? <Check size={14} /> : <Copy size={14} />}
            {copied ? 'Đã sao chép Log!' : 'Sao chép Telemetry Log'}
          </button>
        </div>
      </div>

      {/* Metrics Badges */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-2 font-mono text-xs">
        <div className="bg-[#171C27] border border-[#232A38] rounded-xl p-2.5 flex flex-col">
          <span className="text-[10px] text-[#8B93A7] uppercase">TRẠNG THÁI FEN</span>
          <div className="flex items-center gap-1.5 mt-1 font-bold">
            {parsedFen.status === 'valid' && (
              <span className="text-[#4FD3C4] flex items-center gap-1"><CheckCircle2 size={13} /> CHUẨN VALID</span>
            )}
            {parsedFen.status === 'repaired' && (
              <span className="text-[#C89B3C] flex items-center gap-1"><RefreshCw size={13} /> AUTO REPAIRED</span>
            )}
            {parsedFen.status === 'fallback' && (
              <span className="text-[#C1392B] flex items-center gap-1"><AlertTriangle size={13} /> FALLBACK MODE</span>
            )}
          </div>
        </div>

        <div className="bg-[#171C27] border border-[#232A38] rounded-xl p-2.5 flex flex-col">
          <span className="text-[10px] text-[#8B93A7] uppercase">SỐ QUÂN CỜ HIỂN THỊ</span>
          <span className="text-sm font-bold text-[#E7E9EE] mt-0.5">
            {parsedFen.pieces.length} / 32 quân
          </span>
        </div>

        <div className="bg-[#171C27] border border-[#232A38] rounded-xl p-2.5 flex flex-col">
          <span className="text-[10px] text-[#8B93A7] uppercase">MŨI TÊN CANDIDATES</span>
          <span className="text-sm font-bold text-[#4FD3C4] mt-0.5">
            {candidates.length} ứng viên
          </span>
        </div>

        <div className="bg-[#171C27] border border-[#232A38] rounded-xl p-2.5 flex flex-col">
          <span className="text-[10px] text-[#8B93A7] uppercase">PLIES TIẾN TRÌNH</span>
          <span className="text-sm font-bold text-[#C89B3C] mt-0.5">
            {currentPly + 1} / {totalPlies}
          </span>
        </div>
      </div>

      {/* Warnings Banner if any */}
      {parsedFen.warnings.length > 0 && (
        <div className="bg-[#C89B3C]/10 border border-[#C89B3C]/40 rounded-xl p-3 flex flex-col gap-1">
          <div className="flex items-center gap-2 font-mono text-xs font-bold text-[#C89B3C]">
            <AlertTriangle size={14} /> CẢNH BÁO FEN KHÔNG CHUẨN (ĐÃ TỰ ĐỘNG KHÔI PHỤC BÀN CỜ):
          </div>
          {parsedFen.warnings.map((w, idx) => (
            <p key={idx} className="font-mono text-[11px] text-[#E7E9EE] pl-5">
              • {w}
            </p>
          ))}
        </div>
      )}

      {/* Full Console Log View */}
      {showFullLogs && (
        <div className="bg-[#0B0E14] border border-[#232A38] rounded-xl p-3 font-mono text-xs flex flex-col gap-1.5 max-h-48 overflow-y-auto">
          {logs.map((logLine, idx) => (
            <div
              key={idx}
              className={`${
                logLine.includes('[FEN WARN]') || logLine.includes('[CANDIDATES WARN]')
                  ? 'text-[#C89B3C]'
                  : logLine.includes('[SYS]')
                  ? 'text-[#4FD3C4] font-bold'
                  : 'text-[#8B93A7]'
              }`}
            >
              {logLine}
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
