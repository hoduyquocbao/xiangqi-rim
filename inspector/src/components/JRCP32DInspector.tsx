import React, { useState } from 'react';
import { Parsed32D } from '../types/xiangqi';
import { 
  ShieldAlert, Target, Award, Layers, Zap, BookOpen, 
  Flame, Scale, Compass, Activity, ChevronDown, ChevronUp
} from 'lucide-react';

interface JRCP32DInspectorProps {
  parsed: Parsed32D;
}

export const JRCP32DInspector: React.FC<JRCP32DInspectorProps> = ({ parsed }) => {
  const [activeTab, setActiveTab] = useState<'overview' | 'tactics' | 'strategy' | 'columns' | 'all32'>('overview');
  const [expandedDim, setExpandedDim] = useState<number | null>(null);

  const toggleDim = (dim: number) => {
    setExpandedDim(expandedDim === dim ? null : dim);
  };

  return (
    <div className="bg-[#12161F] border border-[#232A38] rounded-2xl p-5 shadow-xl flex flex-col gap-4">
      {/* Header Tabs */}
      <div className="flex items-center gap-2 border-b border-[#232A38] pb-3 overflow-x-auto">
        <button
          onClick={() => setActiveTab('overview')}
          className={`flex items-center gap-2 px-3 py-1.5 rounded-lg font-mono text-xs font-semibold transition-all ${
            activeTab === 'overview'
              ? 'bg-[#4FD3C4] text-[#0B0E14] shadow-md'
              : 'bg-[#171C27] text-[#8B93A7] hover:text-[#E7E9EE] border border-[#232A38]'
          }`}
        >
          <Award size={14} /> Tổng quan & Candidate
        </button>

        <button
          onClick={() => setActiveTab('tactics')}
          className={`flex items-center gap-2 px-3 py-1.5 rounded-lg font-mono text-xs font-semibold transition-all ${
            activeTab === 'tactics'
              ? 'bg-[#4FD3C4] text-[#0B0E14] shadow-md'
              : 'bg-[#171C27] text-[#8B93A7] hover:text-[#E7E9EE] border border-[#232A38]'
          }`}
        >
          <ShieldAlert size={14} /> An toàn & Chiến thuật
        </button>

        <button
          onClick={() => setActiveTab('strategy')}
          className={`flex items-center gap-2 px-3 py-1.5 rounded-lg font-mono text-xs font-semibold transition-all ${
            activeTab === 'strategy'
              ? 'bg-[#4FD3C4] text-[#0B0E14] shadow-md'
              : 'bg-[#171C27] text-[#8B93A7] hover:text-[#E7E9EE] border border-[#232A38]'
          }`}
        >
          <BookOpen size={14} /> Binh pháp 36 kế
        </button>

        <button
          onClick={() => setActiveTab('columns')}
          className={`flex items-center gap-2 px-3 py-1.5 rounded-lg font-mono text-xs font-semibold transition-all ${
            activeTab === 'columns'
              ? 'bg-[#4FD3C4] text-[#0B0E14] shadow-md'
              : 'bg-[#171C27] text-[#8B93A7] hover:text-[#E7E9EE] border border-[#232A38]'
          }`}
        >
          <Activity size={14} /> 9 Lộ & Mobility
        </button>

        <button
          onClick={() => setActiveTab('all32')}
          className={`flex items-center gap-2 px-3 py-1.5 rounded-lg font-mono text-xs font-semibold transition-all ${
            activeTab === 'all32'
              ? 'bg-[#C89B3C] text-[#0B0E14] shadow-md'
              : 'bg-[#171C27] text-[#8B93A7] hover:text-[#E7E9EE] border border-[#232A38]'
          }`}
        >
          <Layers size={14} /> Tất cả 32D
        </button>
      </div>

      {/* Tab 1: Overview & Candidates */}
      {activeTab === 'overview' && (
        <div className="flex flex-col gap-4">
          {/* Card Centipawn & Bestmove */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
            <div className="bg-[#171C27] border border-[#232A38] rounded-xl p-4 flex flex-col justify-between">
              <span className="font-mono text-[10px] text-[#8B93A7] uppercase tracking-wider">
                [27/32] CENTIPAWN TỔNG HỢP
              </span>
              <div className="text-2xl font-mono font-bold text-[#4FD3C4] mt-1">
                {parsed.centipawnSummary >= 0 ? `+${parsed.centipawnSummary}` : parsed.centipawnSummary} cp
              </div>
              <span className="text-xs font-mono text-[#8B93A7] mt-1">
                {parsed.centipawnSummary === 0 ? 'Cân bằng tuyệt đối' : parsed.centipawnSummary > 0 ? 'Đỏ có ưu thế' : 'Đen có ưu thế'}
              </span>
            </div>

            <div className="bg-[#171C27] border border-[#C89B3C]/40 rounded-xl p-4 flex flex-col justify-between">
              <span className="font-mono text-[10px] text-[#C89B3C] uppercase tracking-wider font-semibold">
                [26/32] NƯỚC ĐI TỐI ƯU (BESTMOVE)
              </span>
              <div className="text-2xl font-mono font-bold text-[#C89B3C] mt-1">
                {parsed.bestMoveSelection.selectedMove || 'N/A'}
              </div>
              <span className="text-xs font-serif text-[#E7E9EE] mt-1 line-clamp-1">
                {parsed.bestMoveSelection.selectedDesc || 'Nước đi xuất sắc'}
              </span>
            </div>

            <div className="bg-[#171C27] border border-[#232A38] rounded-xl p-4 flex flex-col justify-between">
              <span className="font-mono text-[10px] text-[#8B93A7] uppercase tracking-wider">
                [3/32] TƯƠNG QUAN VẬT CHẤT
              </span>
              <div className="text-sm font-mono text-[#E7E9EE] mt-1">
                Đỏ: <span className="text-[#C1392B] font-bold">{parsed.material.redScore}cp</span> | Đen: <span className="text-[#4FD3C4] font-bold">{parsed.material.blackScore}cp</span>
              </div>
              <span className="text-xs font-mono text-[#8B93A7] mt-1">
                Chênh lệch: {parsed.material.diff}cp
              </span>
            </div>
          </div>

          {/* Bảng 3 Ứng viên (Candidates evaluation) */}
          <div className="bg-[#171C27] border border-[#232A38] rounded-xl p-4">
            <h4 className="font-mono text-xs text-[#8B93A7] font-semibold uppercase tracking-wider mb-3 flex items-center gap-2">
              <Target size={14} className="text-[#C89B3C]" /> [25/32] ĐÁNH GIÁ 3 ỨNG VIÊN (CANDIDATES)
            </h4>
            <div className="flex flex-col gap-2">
              {parsed.candidates.length > 0 ? (
                parsed.candidates.map((cand, idx) => (
                  <div
                    key={idx}
                    className={`flex items-center justify-between p-3 rounded-lg border transition-all ${
                      cand.isBest
                        ? 'bg-[#C89B3C]/10 border-[#C89B3C]'
                        : 'bg-[#12161F] border-[#232A38]'
                    }`}
                  >
                    <div className="flex items-center gap-3">
                      <span
                        className={`w-6 h-6 rounded-full flex items-center justify-center font-mono text-xs font-bold ${
                          cand.isBest ? 'bg-[#C89B3C] text-[#0B0E14]' : 'bg-[#232A38] text-[#8B93A7]'
                        }`}
                      >
                        {cand.rank}
                      </span>
                      <span className="font-mono text-sm font-bold text-[#E7E9EE]">
                        {cand.moveStr}
                      </span>
                      <span className="font-serif text-xs text-[#8B93A7]">
                        {cand.description}
                      </span>
                    </div>

                    <div className="flex items-center gap-2">
                      <span className="font-mono text-xs font-semibold text-[#4FD3C4]">
                        {cand.score}cp
                      </span>
                      {cand.isBest && (
                        <span className="px-2 py-0.5 rounded text-[10px] font-mono font-bold bg-[#C89B3C] text-[#0B0E14]">
                          ★ BEST
                        </span>
                      )}
                    </div>
                  </div>
                ))
              ) : (
                <div className="text-xs font-mono text-[#8B93A7] py-2">
                  Chưa trích xuất được danh sách ứng viên từ thought log.
                </div>
              )}
            </div>
          </div>

          {/* Phản đòn & Luật vật lý & Endgame Tablebase */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            <div className="bg-[#171C27] border border-[#232A38] rounded-xl p-3.5">
              <span className="font-mono text-[10px] text-[#8B93A7] uppercase">
                [29/32] NƯỚC PHẢN ĐÒN SẮC BÉN NHẤT
              </span>
              <p className="font-mono text-xs text-[#4FD3C4] font-semibold mt-1">
                {parsed.sharpenedCounter || 'Không có'}
              </p>
            </div>
            <div className="bg-[#171C27] border border-[#232A38] rounded-xl p-3.5">
              <span className="font-mono text-[10px] text-[#8B93A7] uppercase">
                [32/32] TỈ LỆ THẮNG HÒA THUA TẢN CUỘC
              </span>
              <p className="font-mono text-xs text-[#E7E9EE] font-medium mt-1">
                {parsed.endgameTablebaseRatio || 'Chưa kích hoạt Tablebase'}
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Tab 2: Tactics & Safety */}
      {activeTab === 'tactics' && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          {[
            { num: 7, title: 'AN TOÀN TƯỚNG', val: parsed.kingSafety.mySideStatus, icon: ShieldAlert },
            { num: 8, title: 'QUÂN BỊ TẤN CÔNG', val: parsed.attackedPieces, icon: Flame },
            { num: 9, title: 'QUÂN TREO', val: parsed.hangingPieces, icon: Zap },
            { num: 10, title: 'QUÂN BỊ GHIM', val: parsed.pinnedPieces, icon: Scale },
            { num: 11, title: 'ĐÒN KÉP', val: parsed.doubleAttacks, icon: Target },
            { num: 12, title: 'ĐÒN MỞ', val: parsed.discoveredAttacks, icon: Compass },
            { num: 13, title: 'BẪY ĂN QUÂN', val: parsed.tacticalTraps, icon: Flame },
            { num: 14, title: 'CHIẾU BÍ TIỀM ẨN', val: parsed.mateThreats, icon: ShieldAlert },
            { num: 15, title: 'DƯƠNG ĐÔNG KÍCH TÂY', val: parsed.eastWestFeint, icon: Zap },
          ].map((item) => (
            <div key={item.num} className="bg-[#171C27] border border-[#232A38] rounded-xl p-3.5">
              <div className="flex items-center gap-2 font-mono text-[10px] text-[#8B93A7] uppercase">
                <item.icon size={13} className="text-[#4FD3C4]" />
                [{item.num}/32] {item.title}
              </div>
              <p className="font-serif text-xs text-[#E7E9EE] mt-1.5 leading-relaxed">
                {item.val || 'Không có'}
              </p>
            </div>
          ))}
        </div>
      )}

      {/* Tab 3: Strategy & Binh pháp */}
      {activeTab === 'strategy' && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          {[
            { num: 16, title: 'MẪU CHIẾN THUẬT', val: parsed.tacticalPattern },
            { num: 17, title: 'PHỐI HỢP QUÂN', val: parsed.coordination },
            { num: 18, title: 'ĐIỂM YẾU CẤU TRÚC', val: parsed.structuralWeakness },
            { num: 19, title: '36 KẾ BINH PHÁP', val: parsed.thirtySixStratagems },
            { num: 20, title: 'THẾ TRẬN KINH ĐIỂN', val: parsed.classicFormation },
            { num: 21, title: 'GIAI ĐOẠN & CHIẾN LƯỢC', val: parsed.phaseStrategy },
            { num: 22, title: 'TEMPO & SÁNG KIẾN', val: parsed.tempoInitiative },
            { num: 23, title: 'ƯU THẾ TỔNG HỢP', val: parsed.compositeAdvantage },
            { num: 24, title: 'BẤT LỢI TỔNG HỢP', val: parsed.compositeDisadvantage },
          ].map((item) => (
            <div key={item.num} className="bg-[#171C27] border border-[#232A38] rounded-xl p-3.5">
              <div className="font-mono text-[10px] text-[#C89B3C] uppercase font-semibold">
                [{item.num}/32] {item.title}
              </div>
              <p className="font-serif text-xs text-[#E7E9EE] mt-1.5 leading-relaxed">
                {item.val || 'Không có'}
              </p>
            </div>
          ))}
        </div>
      )}

      {/* Tab 4: 9 Columns & Mobility */}
      {activeTab === 'columns' && (
        <div className="flex flex-col gap-3">
          <div className="bg-[#171C27] border border-[#232A38] rounded-xl p-4">
            <span className="font-mono text-[10px] text-[#8B93A7] uppercase font-semibold">
              [4/32] PHÂN TÍCH 9 LỘ
            </span>
            <pre className="font-mono text-xs text-[#4FD3C4] mt-2 whitespace-pre-wrap leading-relaxed">
              {parsed.nineColumns.raw || 'Chưa có thông tin'}
            </pre>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            <div className="bg-[#171C27] border border-[#232A38] rounded-xl p-4">
              <span className="font-mono text-[10px] text-[#8B93A7] uppercase">
                [5/32] MỨC ĐỘ TRIỂN KHAI QUÂN
              </span>
              <p className="font-serif text-xs text-[#E7E9EE] mt-2 whitespace-pre-wrap">
                {parsed.deployment.raw}
              </p>
            </div>

            <div className="bg-[#171C27] border border-[#232A38] rounded-xl p-4">
              <span className="font-mono text-[10px] text-[#8B93A7] uppercase">
                [6/32] ĐỘ LINH HOẠT (MOBILITY)
              </span>
              <p className="font-mono text-xs text-[#E7E9EE] mt-2">
                Đỏ: {parsed.mobility.redMovesCount} nước | Đen: {parsed.mobility.blackMovesCount} nước
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Tab 5: All 32 Dimensions List */}
      {activeTab === 'all32' && (
        <div className="flex flex-col gap-2">
          {Array.from({ length: 32 }, (_, i) => i + 1).map((dimNum) => {
            const isExp = expandedDim === dimNum;
            return (
              <div
                key={dimNum}
                className="bg-[#171C27] border border-[#232A38] rounded-xl overflow-hidden"
              >
                <button
                  onClick={() => toggleDim(dimNum)}
                  className="w-full flex items-center justify-between p-3 text-left hover:bg-[#232A38]/40 transition-colors"
                >
                  <span className="font-mono text-xs font-semibold text-[#4FD3C4]">
                    [{dimNum}/32] Chiều kích #{dimNum}
                  </span>
                  {isExp ? <ChevronUp size={16} className="text-[#8B93A7]" /> : <ChevronDown size={16} className="text-[#8B93A7]" />}
                </button>

                {isExp && (
                  <div className="p-3 border-t border-[#232A38] bg-[#12161F] font-mono text-xs text-[#E7E9EE] whitespace-pre-wrap leading-relaxed">
                    {dimNum === 1 && parsed.inventory.raw}
                    {dimNum === 2 && parsed.board2d.raw}
                    {dimNum === 3 && parsed.material.raw}
                    {dimNum === 4 && parsed.nineColumns.raw}
                    {dimNum === 5 && parsed.deployment.raw}
                    {dimNum === 6 && parsed.mobility.raw}
                    {dimNum === 7 && parsed.kingSafety.raw}
                    {dimNum === 8 && parsed.attackedPieces}
                    {dimNum === 9 && parsed.hangingPieces}
                    {dimNum === 10 && parsed.pinnedPieces}
                    {dimNum === 11 && parsed.doubleAttacks}
                    {dimNum === 12 && parsed.discoveredAttacks}
                    {dimNum === 13 && parsed.tacticalTraps}
                    {dimNum === 14 && parsed.mateThreats}
                    {dimNum === 15 && parsed.eastWestFeint}
                    {dimNum === 16 && parsed.tacticalPattern}
                    {dimNum === 17 && parsed.coordination}
                    {dimNum === 18 && parsed.structuralWeakness}
                    {dimNum === 19 && parsed.thirtySixStratagems}
                    {dimNum === 20 && parsed.classicFormation}
                    {dimNum === 21 && parsed.phaseStrategy}
                    {dimNum === 22 && parsed.tempoInitiative}
                    {dimNum === 23 && parsed.compositeAdvantage}
                    {dimNum === 24 && parsed.compositeDisadvantage}
                    {dimNum === 25 && JSON.stringify(parsed.candidates, null, 2)}
                    {dimNum === 26 && parsed.bestMoveSelection.raw}
                    {dimNum === 27 && `${parsed.centipawnSummary} cp`}
                    {dimNum === 28 && parsed.verification}
                    {dimNum === 29 && parsed.sharpenedCounter}
                    {dimNum === 30 && parsed.physicalRulesConstraint}
                    {dimNum === 31 && parsed.exchangeChain}
                    {dimNum === 32 && parsed.endgameTablebaseRatio}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};
