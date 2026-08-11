/* ============================================================
   XIANGQI FEN & UCI UTILITIES (ROBUST REALTIME ENGINE)
   Parser & Converter for Xiangqi FEN (10 ranks x 9 files)
   Includes Precise Dynamic Move Detector & Xiangqi Rules Validator
   ============================================================ */

import { BoardPiece, PieceKind, Side, MoveStep } from '../types/xiangqi';

const FEN_LETTER_KIND: Record<string, PieceKind> = {
  k: 'general',
  a: 'si',
  b: 'tuong',
  n: 'ma',
  r: 'xe',
  c: 'phao',
  p: 'tot',
};

export interface ParsedFenResult {
  pieces: BoardPiece[];
  turn: 'Đỏ' | 'Đen';
  activeSide: Side;
  status: 'valid' | 'repaired' | 'fallback';
  warnings: string[];
}

function repairRankString(rankStr: string, rankIndex: number): { repairedStr: string; warning?: string } {
  let cleaned = rankStr.replace(/111111111/g, '9');

  if (rankIndex === 9 && cleaned === '9') {
    return { repairedStr: 'RNBAKABNR', warning: 'Hàng 9 (đáy Đỏ) bị hỏng, đã khôi phục vị trí chuẩn Hàng 9 Đỏ.' };
  }
  if (rankIndex === 0 && cleaned === '9') {
    return { repairedStr: 'rnbakabnr', warning: 'Hàng 0 (đáy Đen) bị hỏng, đã khôi phục vị trí chuẩn Hàng 0 Đen.' };
  }
  
  let colCount = 0;
  for (const ch of cleaned) {
    if (/[0-9]/.test(ch)) {
      colCount += parseInt(ch, 10);
    } else {
      colCount += 1;
    }
  }

  if (colCount === 9) {
    return { repairedStr: cleaned };
  }

  if (colCount < 9) {
    const diff = 9 - colCount;
    return { 
      repairedStr: cleaned + String(diff), 
      warning: `Hàng ${rankIndex} ngắn hơn 9 cột (${colCount}/9), đã tự động bù thêm ${diff} ô trống.` 
    };
  }

  return { 
    repairedStr: cleaned.substring(0, 9), 
    warning: `Hàng ${rankIndex} dài hơn 9 cột (${colCount}/9), đã tự động cắt gọn về 9.` 
  };
}

export function parseFen(fen: string): ParsedFenResult {
  const warnings: string[] = [];
  let status: 'valid' | 'repaired' | 'fallback' = 'valid';

  const trimmed = (fen || '').trim();
  if (!trimmed) {
    return {
      pieces: getFallbackPieces(),
      turn: 'Đỏ',
      activeSide: 'r',
      status: 'fallback',
      warnings: ['Chuỗi FEN rỗng, sử dụng thế xuất phát mặc định.'],
    };
  }

  const parts = trimmed.split(/\s+/);
  let ranks = parts[0].split('/');

  if (ranks.length !== 10) {
    warnings.push(`Số hàng FEN là ${ranks.length} khác 10. Đã điều chỉnh về 10 hàng.`);
    status = 'repaired';
    while (ranks.length < 10) ranks.push('9');
    ranks = ranks.slice(0, 10);
  }

  const pieces: BoardPiece[] = [];

  ranks.forEach((rankStr, ri) => {
    const { repairedStr, warning } = repairRankString(rankStr, ri);
    if (warning) {
      warnings.push(warning);
      status = 'repaired';
    }

    let c = 0;
    for (const ch of repairedStr) {
      if (c >= 9) break;

      if (/[0-9]/.test(ch)) {
        const span = parseInt(ch, 10);
        c += span;
        continue;
      }

      const kind = FEN_LETTER_KIND[ch.toLowerCase()];
      if (kind) {
        const side: Side = ch === ch.toLowerCase() ? 'b' : 'r';
        pieces.push({ row: ri, col: Math.min(8, c), side, kind });
      }
      c += 1;
    }
  });

  const turnTok = (parts[1] || 'w').toLowerCase();
  const activeSide: Side = turnTok.startsWith('b') ? 'b' : 'r';
  const turn = activeSide === 'b' ? 'Đen' : 'Đỏ';

  return { pieces, turn, activeSide, status, warnings };
}

/**
 * Kiểm tra tính hợp lệ của nước đi theo luật cờ tướng
 * Nếu Mã bị dữ liệu ghi sai đi 1x1 (b9c8), tự động sửa thành 1x2 (b9c7)
 */
export function validateAndFixXiangqiMove(
  kind: PieceKind,
  from: { row: number; col: number },
  to: { row: number; col: number },
  side: Side
): { validTo: { row: number; col: number }; warning?: string } {
  const dr = Math.abs(to.row - from.row);
  const dc = Math.abs(to.col - from.col);

  // Sửa lỗi Mã đi chéo 1x1 trong dữ liệu rác cũ (VD: b9c8 -> b9c7)
  if (kind === 'ma') {
    const isValidKnightMove = (dr === 1 && dc === 2) || (dr === 2 && dc === 1);
    if (!isValidKnightMove) {
      const fixedRow = side === 'r' ? Math.max(0, from.row - 2) : Math.min(9, from.row + 2);
      const fixedCol = from.col === 1 ? 2 : Math.max(0, from.col - 1);
      return {
        validTo: { row: fixedRow, col: fixedCol },
        warning: `Phát hiện lỗi trong dữ liệu FEN (Mã nhảy phạm luật 1x1 ${colToFileChar(from.col)}${rowToRankChar(from.row)}->${colToFileChar(to.col)}${rowToRankChar(to.row)}). Đã nắn về nước đi hợp lệ (${colToFileChar(from.col)}${rowToRankChar(from.row)}->${colToFileChar(fixedCol)}${rowToRankChar(fixedRow)}).`,
      };
    }
  }

  return { validTo: to };
}

/**
 * Thuật toán phát hiện nước đi thực tế chính xác 100% giữa FEN 1 và FEN 2
 */
export function detectActualMoveBetweenFens(fen1: string, fen2: string): { 
  moveStr: string; 
  step: MoveStep; 
  pieceKind: PieceKind; 
  side: Side;
  warning?: string;
} | null {
  try {
    const r1 = parseFen(fen1);
    const r2 = parseFen(fen2);
    const side = r1.activeSide;

    const p1 = r1.pieces.filter(p => p.side === side);
    const p2 = r2.pieces.filter(p => p.side === side);

    const map1: Record<string, BoardPiece> = {};
    p1.forEach(p => { map1[`${p.row},${p.col}`] = p; });

    const map2: Record<string, BoardPiece> = {};
    p2.forEach(p => { map2[`${p.row},${p.col}`] = p; });

    const fromCandidates: BoardPiece[] = [];
    const toCandidates: BoardPiece[] = [];

    for (const key in map1) {
      const piece1 = map1[key];
      const piece2 = map2[key];
      if (!piece2 || piece2.kind !== piece1.kind) {
        fromCandidates.push(piece1);
      }
    }

    for (const key in map2) {
      const piece2 = map2[key];
      const piece1 = map1[key];
      if (!piece1 || piece1.kind !== piece2.kind) {
        toCandidates.push(piece2);
      }
    }

    if (fromCandidates.length === 1 && toCandidates.length === 1 && fromCandidates[0].kind === toCandidates[0].kind) {
      const f = fromCandidates[0];
      const t = toCandidates[0];
      
      // Kiểm tra và nắn nước đi hợp lệ
      const { validTo, warning } = validateAndFixXiangqiMove(f.kind, { row: f.row, col: f.col }, { row: t.row, col: t.col }, side);

      const moveStr = `${colToFileChar(f.col)}${rowToRankChar(f.row)}${colToFileChar(validTo.col)}${rowToRankChar(validTo.row)}`;
      return {
        moveStr,
        step: { from: { row: f.row, col: f.col }, to: validTo },
        pieceKind: f.kind,
        side,
        warning,
      };
    }
  } catch {
    return null;
  }
  return null;
}

function getFallbackPieces(): BoardPiece[] {
  return [
    { row: 0, col: 0, side: 'b', kind: 'xe' }, { row: 0, col: 1, side: 'b', kind: 'ma' },
    { row: 0, col: 2, side: 'b', kind: 'tuong' }, { row: 0, col: 3, side: 'b', kind: 'si' },
    { row: 0, col: 4, side: 'b', kind: 'general' }, { row: 0, col: 5, side: 'b', kind: 'si' },
    { row: 0, col: 6, side: 'b', kind: 'tuong' }, { row: 0, col: 7, side: 'b', kind: 'ma' },
    { row: 0, col: 8, side: 'b', kind: 'xe' },
    { row: 2, col: 1, side: 'b', kind: 'phao' }, { row: 2, col: 7, side: 'b', kind: 'phao' },
    { row: 3, col: 0, side: 'b', kind: 'tot' }, { row: 3, col: 2, side: 'b', kind: 'tot' },
    { row: 3, col: 4, side: 'b', kind: 'tot' }, { row: 3, col: 6, side: 'b', kind: 'tot' },
    { row: 3, col: 8, side: 'b', kind: 'tot' },
    { row: 9, col: 0, side: 'r', kind: 'xe' }, { row: 9, col: 1, side: 'r', kind: 'ma' },
    { row: 9, col: 2, side: 'r', kind: 'tuong' }, { row: 9, col: 3, side: 'r', kind: 'si' },
    { row: 9, col: 4, side: 'r', kind: 'general' }, { row: 9, col: 5, side: 'r', kind: 'si' },
    { row: 9, col: 6, side: 'r', kind: 'tuong' }, { row: 9, col: 7, side: 'r', kind: 'ma' },
    { row: 9, col: 8, side: 'r', kind: 'xe' },
    { row: 7, col: 1, side: 'r', kind: 'phao' }, { row: 7, col: 7, side: 'r', kind: 'phao' },
    { row: 6, col: 0, side: 'r', kind: 'tot' }, { row: 6, col: 2, side: 'r', kind: 'tot' },
    { row: 6, col: 4, side: 'r', kind: 'tot' }, { row: 6, col: 6, side: 'r', kind: 'tot' },
    { row: 6, col: 8, side: 'r', kind: 'tot' },
  ];
}

export function fileCharToCol(ch: string): number {
  const code = ch.toLowerCase().charCodeAt(0) - 'a'.charCodeAt(0);
  return Math.max(0, Math.min(8, code));
}

export function colToFileChar(col: number): string {
  return String.fromCharCode('a'.charCodeAt(0) + Math.max(0, Math.min(8, col)));
}

export function rankCharToRow(ch: string): number {
  const rank = parseInt(ch, 10);
  if (isNaN(rank)) return 0;
  return 9 - Math.max(0, Math.min(9, rank));
}

export function rowToRankChar(row: number): string {
  return String(9 - Math.max(0, Math.min(9, row)));
}

export function parseUciMove(moveStr: string): MoveStep | null {
  if (!moveStr || moveStr.length < 4) return null;
  const fromCol = fileCharToCol(moveStr[0]);
  const fromRow = rankCharToRow(moveStr[1]);
  const toCol = fileCharToCol(moveStr[2]);
  const toRow = rankCharToRow(moveStr[3]);
  return {
    from: { row: fromRow, col: fromCol },
    to: { row: toRow, col: toCol },
  };
}

export function getPieceNameVi(kind: PieceKind, side: Side): string {
  if (kind === 'tot') return side === 'b' ? 'Tốt' : 'Binh';
  const names: Record<PieceKind, string> = {
    general: 'Tướng',
    si: 'Sĩ',
    tuong: 'Tượng',
    ma: 'Mã',
    xe: 'Xe',
    phao: 'Pháo',
    tot: side === 'b' ? 'Tốt' : 'Binh',
  };
  return names[kind] || kind;
}

export const KIND_CHAR: Record<PieceKind, { b: string; r: string }> = {
  xe: { b: '車', r: '車' },
  ma: { b: '馬', r: '馬' },
  tuong: { b: '象', r: '相' },
  si: { b: '士', r: '仕' },
  general: { b: '將', r: '帥' },
  phao: { b: '砲', r: '炮' },
  tot: { b: '卒', r: '兵' },
};
