/* ============================================================
   XIANGQI · R1 — TYPE SYSTEM SPECIFICATION (JRCP 2.0 32D)
   100% Type-safe TypeScript types for 32 Reasoning Dimensions
   ============================================================ */

export type Side = 'r' | 'b'; // 'r' = Red (Đỏ), 'b' = Black (Đen)

export type PieceKind = 
  | 'general'  // Tướng / 將 / 帥
  | 'si'       // Sĩ / 士 / 仕
  | 'tuong'    // Tượng / 象 / 相
  | 'ma'       // Mã / 馬 / 馬
  | 'xe'       // Xe / 車 / 車
  | 'phao'     // Pháo / 砲 / 炮
  | 'tot';     // Tốt / Binh / 卒 / 兵

export interface BoardPiece {
  row: number; // 0..9 (0 là hàng Đen xuất phát, 9 là hàng Đỏ xuất phát)
  col: number; // 0..8 (a..i)
  side: Side;
  kind: PieceKind;
}

export interface PositionSquare {
  row: number;
  col: number;
}

export interface MoveStep {
  from: PositionSquare;
  to: PositionSquare;
}

export interface CandidateMove {
  rank: number;       // 1, 2, 3
  moveStr: string;    // e.g. "e2e6"
  description: string;// e.g. "Pháo(e2->e6) ★BEST★"
  score: number;      // e.g. 0 (centipawns)
  isBest: boolean;
  from: PositionSquare;
  to: PositionSquare;
}

export interface Parsed32D {
  // [1/32] Kiểm kê quân cờ
  inventory: {
    raw: string;
    redPieces: string[];
    blackPieces: string[];
  };
  // [2/32] Bàn cờ 2D
  board2d: {
    raw: string;
    gridText: string;
  };
  // [3/32] Tương quan vật chất chi tiết
  material: {
    raw: string;
    redScore: number;
    blackScore: number;
    diff: number;
  };
  // [4/32] Phân tích 9 lộ
  nineColumns: {
    raw: string;
    colStatus: Record<string, string>;
  };
  // [5/32] Mức độ triển khai quân
  deployment: {
    raw: string;
    redDeployed: string;
    blackDeployed: string;
  };
  // [6/32] Độ linh hoạt (Mobility)
  mobility: {
    raw: string;
    redMovesCount: number;
    blackMovesCount: number;
  };
  // [7/32] An toàn Tướng
  kingSafety: {
    raw: string;
    mySideStatus: string;
  };
  // [8/32] Quân bị tấn công
  attackedPieces: string;
  // [9/32] Quân treo
  hangingPieces: string;
  // [10/32] Quân bị ghim
  pinnedPieces: string;
  // [11/32] Đòn kép
  doubleAttacks: string;
  // [12/32] Đòn mở
  discoveredAttacks: string;
  // [13/32] Bẫy ăn quân
  tacticalTraps: string;
  // [14/32] Chiếu bí tiềm ẩn
  mateThreats: string;
  // [15/32] Dương đông kích tây
  eastWestFeint: string;
  // [16/32] Mẫu chiến thuật
  tacticalPattern: string;
  // [17/32] Phối hợp quân
  coordination: string;
  // [18/32] Điểm yếu cấu trúc
  structuralWeakness: string;
  // [19/32] 36 kế binh pháp
  thirtySixStratagems: string;
  // [20/32] Thế trận kinh điển
  classicFormation: string;
  // [21/32] Giai đoạn & Chiến lược
  phaseStrategy: string;
  // [22/32] Tempo & Sáng kiến
  tempoInitiative: string;
  // [23/32] Ưu thế tổng hợp
  compositeAdvantage: string;
  // [24/32] Bất lợi tổng hợp
  compositeDisadvantage: string;
  // [25/32] Đánh giá Candidates (3 ứng viên)
  candidates: CandidateMove[];
  // [26/32] So sánh & Chọn Bestmove
  bestMoveSelection: {
    raw: string;
    selectedMove: string;
    selectedDesc: string;
  };
  // [27/32] Centipawn tổng hợp
  centipawnSummary: number;
  // [28/32] Xác minh
  verification: string;
  // [29/32] Nước phản đòn sắc bén nhất
  sharpenedCounter: string;
  // [30/32] Giới hạn luật cấm vật lý
  physicalRulesConstraint: string;
  // [31/32] Chuỗi đổi quân
  exchangeChain: string;
  // [32/32] Tỉ lệ thắng hòa thua tản cuộc
  endgameTablebaseRatio: string;
}

export interface TurnMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
  fen?: string;
  turnNumber?: number;
  turnSide?: 'Đỏ' | 'Đen';
  thoughtRaw?: string;
  parsed32D?: Parsed32D;
  bestMove?: string;
}

export interface GameSession {
  game_id: string;
  total_plies: number;
  outcome: 'red_win' | 'black_win' | 'draw' | 'in_progress';
  stamp: number;
  messages: TurnMessage[];
}
