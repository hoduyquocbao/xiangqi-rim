/* ============================================================
   XIANGQI JRCP 2.0 32-DIMENSIONS PARSER (ROBUST & ROBUST FALLBACK)
   Extracts and parses all 32 dimensions from <thought> tags
   without shortcutting or dropping candidates.
   ============================================================ */

import { Parsed32D, CandidateMove } from '../types/xiangqi';
import { parseUciMove } from './fenParser';

function extractSection(thought: string, currentDim: number): string {
  const currentTag = `[${currentDim}/32]`;
  const nextTag = `[${currentDim + 1}/32]`;
  
  const startIdx = thought.indexOf(currentTag);
  if (startIdx === -1) return '';

  const contentStart = startIdx + currentTag.length;
  let endIdx = -1;

  if (currentDim === 32) {
    endIdx = thought.indexOf('</thought>', contentStart);
    if (endIdx === -1) endIdx = thought.length;
  } else {
    endIdx = thought.indexOf(nextTag, contentStart);
    if (endIdx === -1) {
      endIdx = thought.indexOf('</thought>', contentStart);
      if (endIdx === -1) endIdx = thought.length;
    }
  }

  return thought.substring(contentStart, endIdx).trim();
}

function parseCandidates(raw25: string, raw26: string): CandidateMove[] {
  const candidates: CandidateMove[] = [];
  const lines = raw25.split('\n');

  lines.forEach((line) => {
    const trimmed = line.trim();
    if (!trimmed.includes('Ứng viên')) return;

    const rankMatch = trimmed.match(/Ứng viên\s*(\d+)/i);
    const rank = rankMatch ? parseInt(rankMatch[1], 10) : candidates.length + 1;

    const moveMatch = trimmed.match(/([a-i][0-9][a-i][0-9])/);
    const moveStr = moveMatch ? moveMatch[1] : '';

    const isBest = trimmed.includes('★BEST★') || rank === 1;

    const scoreMatch = trimmed.match(/\(([-+]?\d+)cp\)/i);
    const score = scoreMatch ? parseInt(scoreMatch[1], 10) : 0;

    const step = parseUciMove(moveStr);

    if (moveStr && step) {
      candidates.push({
        rank,
        moveStr,
        description: trimmed.replace(/^\+\s*/, ''),
        score,
        isBest,
        from: step.from,
        to: step.to,
      });
    }
  });

  // Nếu trong [25/32] không trích xuất được candidates, thử tìm bestmove từ [26/32]
  if (candidates.length === 0 && raw26) {
    const bestMoveMatch = raw26.match(/([a-i][0-9][a-i][0-9])/);
    if (bestMoveMatch) {
      const moveStr = bestMoveMatch[1];
      const step = parseUciMove(moveStr);
      if (step) {
        candidates.push({
          rank: 1,
          moveStr,
          description: `Bestmove: ${moveStr}`,
          score: 0,
          isBest: true,
          from: step.from,
          to: step.to,
        });
      }
    }
  }

  return candidates;
}

export function parseJRCP32D(thoughtText: string): Parsed32D {
  const raw1 = extractSection(thoughtText, 1);
  const raw2 = extractSection(thoughtText, 2);
  const raw3 = extractSection(thoughtText, 3);
  const raw4 = extractSection(thoughtText, 4);
  const raw5 = extractSection(thoughtText, 5);
  const raw6 = extractSection(thoughtText, 6);
  const raw7 = extractSection(thoughtText, 7);
  const raw8 = extractSection(thoughtText, 8);
  const raw9 = extractSection(thoughtText, 9);
  const raw10 = extractSection(thoughtText, 10);
  const raw11 = extractSection(thoughtText, 11);
  const raw12 = extractSection(thoughtText, 12);
  const raw13 = extractSection(thoughtText, 13);
  const raw14 = extractSection(thoughtText, 14);
  const raw15 = extractSection(thoughtText, 15);
  const raw16 = extractSection(thoughtText, 16);
  const raw17 = extractSection(thoughtText, 17);
  const raw18 = extractSection(thoughtText, 18);
  const raw19 = extractSection(thoughtText, 19);
  const raw20 = extractSection(thoughtText, 20);
  const raw21 = extractSection(thoughtText, 21);
  const raw22 = extractSection(thoughtText, 22);
  const raw23 = extractSection(thoughtText, 23);
  const raw24 = extractSection(thoughtText, 24);
  const raw25 = extractSection(thoughtText, 25);
  const raw26 = extractSection(thoughtText, 26);
  const raw27 = extractSection(thoughtText, 27);
  const raw28 = extractSection(thoughtText, 28);
  const raw29 = extractSection(thoughtText, 29);
  const raw30 = extractSection(thoughtText, 30);
  const raw31 = extractSection(thoughtText, 31);
  const raw32 = extractSection(thoughtText, 32);

  const redMatMatch = raw3.match(/Đỏ:\s*(\d+)cp/i);
  const blackMatMatch = raw3.match(/Đen:\s*(\d+)cp/i);
  const diffMatMatch = raw3.match(/Chênh lệch:\s*([-+]?\d+)cp/i);

  const redMobMatch = raw6.match(/Đỏ:\s*(\d+)\s*nước/i);
  const blackMobMatch = raw6.match(/Đen:\s*(\d+)\s*nước/i);

  const selectedMoveMatch = raw26.match(/Chọn\s+([a-i][0-9][a-i][0-9])/i);
  const cpMatch = raw27.match(/([-+]?\d+)cp/i);

  return {
    inventory: {
      raw: raw1,
      redPieces: raw1.split('\n').filter(l => l.includes('Đỏ:')),
      blackPieces: raw1.split('\n').filter(l => l.includes('Đen:')),
    },
    board2d: {
      raw: raw2,
      gridText: raw2,
    },
    material: {
      raw: raw3,
      redScore: redMatMatch ? parseInt(redMatMatch[1], 10) : 480,
      blackScore: blackMatMatch ? parseInt(blackMatMatch[1], 10) : 480,
      diff: diffMatMatch ? parseInt(diffMatMatch[1], 10) : 0,
    },
    nineColumns: {
      raw: raw4,
      colStatus: {},
    },
    deployment: {
      raw: raw5,
      redDeployed: raw5,
      blackDeployed: raw5,
    },
    mobility: {
      raw: raw6,
      redMovesCount: redMobMatch ? parseInt(redMobMatch[1], 10) : 0,
      blackMovesCount: blackMobMatch ? parseInt(blackMobMatch[1], 10) : 0,
    },
    kingSafety: {
      raw: raw7,
      mySideStatus: raw7,
    },
    attackedPieces: raw8,
    hangingPieces: raw9,
    pinnedPieces: raw10,
    doubleAttacks: raw11,
    discoveredAttacks: raw12,
    tacticalTraps: raw13,
    mateThreats: raw14,
    eastWestFeint: raw15,
    tacticalPattern: raw16,
    coordination: raw17,
    structuralWeakness: raw18,
    thirtySixStratagems: raw19,
    classicFormation: raw20,
    phaseStrategy: raw21,
    tempoInitiative: raw22,
    compositeAdvantage: raw23,
    compositeDisadvantage: raw24,
    candidates: parseCandidates(raw25, raw26),
    bestMoveSelection: {
      raw: raw26,
      selectedMove: selectedMoveMatch ? selectedMoveMatch[1] : '',
      selectedDesc: raw26,
    },
    centipawnSummary: cpMatch ? parseInt(cpMatch[1], 10) : 0,
    verification: raw28,
    sharpenedCounter: raw29,
    physicalRulesConstraint: raw30,
    exchangeChain: raw31,
    endgameTablebaseRatio: raw32,
  };
}
