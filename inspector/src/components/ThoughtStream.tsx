import React, { useState } from 'react';
import { Terminal, Copy, Check, Search, Code } from 'lucide-react';

interface ThoughtStreamProps {
  thoughtText: string;
}

export const ThoughtStream: React.FC<ThoughtStreamProps> = ({ thoughtText }) => {
  const [copied, setCopied] = useState(false);
  const [searchFilter, setSearchFilter] = useState('');
  const [viewMode, setViewMode] = useState<'formatted' | 'raw'>('formatted');

  const handleCopy = () => {
    navigator.clipboard.writeText(thoughtText);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const lines = thoughtText.split('\n');

  const filteredLines = searchFilter
    ? lines.filter(line => line.toLowerCase().includes(searchFilter.toLowerCase()))
    : lines;

  return (
    <div className="bg-[#12161F] border border-[#232A38] rounded-2xl p-5 shadow-xl flex flex-col gap-4">
      {/* Header controls */}
      <div className="flex items-center justify-between flex-wrap gap-3 border-b border-[#232A38] pb-3">
        <div className="flex items-center gap-2 font-mono text-xs font-semibold text-[#E7E9EE]">
          <Terminal size={15} className="text-[#4FD3C4]" />
          THOUGHT LOG STREAM (CoT RAW)
        </div>

        <div className="flex items-center gap-2">
          {/* View mode toggle */}
          <button
            onClick={() => setViewMode(viewMode === 'formatted' ? 'raw' : 'formatted')}
            className="flex items-center gap-1.5 px-2.5 py-1 rounded-lg font-mono text-xs bg-[#171C27] text-[#8B93A7] border border-[#232A38] hover:text-[#E7E9EE] transition-colors"
          >
            <Code size={13} /> {viewMode === 'formatted' ? 'Định dạng 32D' : 'Xem Raw Text'}
          </button>

          {/* Copy button */}
          <button
            onClick={handleCopy}
            className="flex items-center gap-1.5 px-2.5 py-1 rounded-lg font-mono text-xs bg-[#4FD3C4]/10 text-[#4FD3C4] border border-[#4FD3C4]/30 hover:bg-[#4FD3C4]/20 transition-colors"
          >
            {copied ? <Check size={13} /> : <Copy size={13} />}
            {copied ? 'Đã copy' : 'Sao chép'}
          </button>
        </div>
      </div>

      {/* Search Input */}
      <div className="relative">
        <Search size={14} className="absolute left-3 top-2.5 text-[#8B93A7]" />
        <input
          type="text"
          value={searchFilter}
          onChange={(e) => setSearchFilter(e.target.value)}
          placeholder="Lọc từ khóa trong suy luận (ví dụ: e2e6, Bestmove, [25/32]...)"
          className="w-full bg-[#0B0E14] border border-[#232A38] rounded-lg pl-9 pr-3 py-1.5 font-mono text-xs text-[#E7E9EE] placeholder-[#8B93A7] focus:outline-none focus:border-[#4FD3C4]"
        />
      </div>

      {/* Main Content Stream */}
      <div className="bg-[#0B0E14] border border-[#232A38] rounded-xl p-4 max-h-[460px] overflow-y-auto font-mono text-xs leading-relaxed">
        {viewMode === 'formatted' ? (
          <div className="flex flex-col gap-1">
            {filteredLines.map((line, idx) => {
              const isTag = /^\[\d+\/32\]/.test(line.trim());
              const isBest = line.includes('★BEST★');
              const isHeader = line.includes('Bàn cờ Turn') || line.includes('FEN:');

              return (
                <div
                  key={idx}
                  className={`py-0.5 px-1 rounded ${
                    isTag
                      ? 'text-[#4FD3C4] font-bold bg-[#4FD3C4]/5 mt-2'
                      : isBest
                      ? 'text-[#C89B3C] font-semibold bg-[#C89B3C]/10'
                      : isHeader
                      ? 'text-[#C1392B] font-semibold'
                      : 'text-[#8B93A7]'
                  }`}
                >
                  {line}
                </div>
              );
            })}
          </div>
        ) : (
          <pre className="text-[#8B93A7] whitespace-pre-wrap">{thoughtText}</pre>
        )}
      </div>
    </div>
  );
};
