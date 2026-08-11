import React, { useState } from 'react';
import { Network, Activity, Cpu } from 'lucide-react';

interface NeuralSimulatorProps {
  currentPly: number;
}

export const NeuralSimulator: React.FC<NeuralSimulatorProps> = ({ currentPly }) => {
  const [selectedLayer, setSelectedLayer] = useState<number>(5);
  const [topK] = useState<number>(2);
  const [numExperts] = useState<number>(8);

  const N_LAYERS = 12;
  const expertsList = [
    'Cấu trúc Tốt/Binh', 'An toàn Tướng', 'Phối hợp Xe', 'Thế trận Pháo',
    'Triển khai Mã', 'Kiểm soát Trung lộ', 'Tấn công cánh', 'Phòng thủ cung Tướng'
  ];

  // Giả lập routing chuyên gia cho layer được chọn
  const activeExperts = [(selectedLayer * 2 + currentPly) % numExperts, (selectedLayer * 3 + currentPly + 1) % numExperts];

  // Giả lập Cosine similarity giữa các lớp
  const cosineSeries = Array.from({ length: N_LAYERS - 1 }, (_, i) => 
    Math.max(0.45, Math.min(0.98, 0.85 - (i === 4 ? 0.35 : 0) + (i % 3) * 0.05))
  );

  return (
    <div className="bg-[#12161F] border border-[#232A38] rounded-2xl p-5 shadow-xl flex flex-col gap-4">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-[#232A38] pb-3">
        <div className="flex items-center gap-2 font-mono text-xs font-semibold text-[#E7E9EE]">
          <Cpu size={15} className="text-[#4FD3C4]" />
          MÔ PHỎNG KIẾN TRÚC MẠNG NƠ-RON (12 LAYERS & MoE)
        </div>
        <span className="font-mono text-[10px] text-[#4FD3C4] bg-[#4FD3C4]/10 border border-[#4FD3C4]/30 px-2 py-0.5 rounded">
          Xiangqi-R1 Core Architecture
        </span>
      </div>

      {/* Layer selector */}
      <div className="flex flex-col gap-2">
        <span className="font-mono text-[10px] text-[#8B93A7] uppercase">
          CHỌN KHỐI TRANSFORMER (1-{N_LAYERS})
        </span>
        <div className="grid grid-cols-6 md:grid-cols-12 gap-1.5">
          {Array.from({ length: N_LAYERS }, (_, i) => (
            <button
              key={i}
              onClick={() => setSelectedLayer(i)}
              className={`py-1.5 rounded-lg font-mono text-xs font-bold transition-all ${
                selectedLayer === i
                  ? 'bg-[#4FD3C4] text-[#0B0E14] shadow-md'
                  : 'bg-[#171C27] text-[#8B93A7] hover:text-[#E7E9EE] border border-[#232A38]'
              }`}
            >
              L{i + 1}
            </button>
          ))}
        </div>
      </div>

      {/* Layer detail: MoE Routing */}
      <div className="bg-[#171C27] border border-[#232A38] rounded-xl p-4 flex flex-col gap-3">
        <div className="flex items-center justify-between">
          <span className="font-mono text-xs font-semibold text-[#C89B3C] flex items-center gap-2">
            <Network size={14} /> MoE ROUTING · LỚP #{selectedLayer + 1} (TOP-{topK}/{numExperts})
          </span>
          <span className="font-mono text-[10px] text-[#8B93A7]">
            +1 Shared Expert
          </span>
        </div>

        {/* Chuyên gia bar chart */}
        <div className="grid grid-cols-4 md:grid-cols-8 gap-2 items-end h-20 pt-2">
          {Array.from({ length: numExperts }, (_, i) => {
            const isActive = activeExperts.includes(i);
            const heightPct = isActive ? 85 + (i * 5) % 15 : 20 + (i * 7) % 25;

            return (
              <div key={i} className="flex flex-col items-center gap-1 h-full justify-end">
                <div
                  style={{ height: `${heightPct}%` }}
                  className={`w-full rounded-t transition-all duration-300 ${
                    isActive ? 'bg-[#C89B3C] shadow-lg shadow-[#C89B3C]/20' : 'bg-[#232A38]'
                  }`}
                />
                <span className={`font-mono text-[9px] ${isActive ? 'text-[#C89B3C] font-bold' : 'text-[#8B93A7]'}`}>
                  E{i}
                </span>
              </div>
            );
          })}
        </div>

        <div className="text-[11px] font-serif text-[#8B93A7] bg-[#12161F] p-2.5 rounded-lg border border-[#232A38]">
          Chuyên gia được chọn lượt này: <span className="text-[#C89B3C] font-semibold">{expertsList[activeExperts[0]]}</span> và <span className="text-[#C89B3C] font-semibold">{expertsList[activeExperts[1]]}</span>.
        </div>
      </div>

      {/* Cosine similarity giữa các lớp */}
      <div className="bg-[#171C27] border border-[#232A38] rounded-xl p-4 flex flex-col gap-2">
        <div className="flex items-center justify-between">
          <span className="font-mono text-xs font-semibold text-[#E7E9EE] flex items-center gap-2">
            <Activity size={14} className="text-[#4FD3C4]" /> COSINE SIMILARITY GIỮA CÁC LỚP LIÊN TIẾP
          </span>
          <span className="font-mono text-[10px] text-[#8B93A7]">
            1.0 = giữ nguyên · thấp = sụt giảm thông tin
          </span>
        </div>

        <div className="flex items-end gap-1.5 h-12 pt-2">
          {cosineSeries.map((sim, idx) => {
            const isDrop = sim < 0.55;
            return (
              <div
                key={idx}
                title={`Lớp ${idx + 1} -> ${idx + 2}: ${sim.toFixed(2)}`}
                style={{ height: `${sim * 100}%` }}
                className={`flex-1 rounded-t transition-all ${
                  isDrop ? 'bg-[#C1392B]' : 'bg-[#4FD3C4]/60 hover:bg-[#4FD3C4]'
                }`}
              />
            );
          })}
        </div>
      </div>
    </div>
  );
};
