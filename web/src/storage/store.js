// Trình lưu trữ lịch sử trí nhớ ván đấu và kinh nghiệm AI (XiangRust Local Storage Engine)
// Định danh đơn từ tiếng Anh: Store, key, item, list, load, save, clear, match, history, record, parse, stringify

const KEY = 'history';
const LIMIT = 50;

/**
 * Đọc toàn bộ danh sách ván đấu đã lưu từ LocalStorage
 */
export function load() {
  try {
    const raw = localStorage.getItem(KEY);
    return raw ? JSON.parse(raw) : [];
  } catch (err) {
    console.error('Không thể nạp lịch sử ván đấu:', err);
    return [];
  }
}

/**
 * Ghi nhận một ván đấu mới hoặc cập nhật ván đấu hiện tại vào LocalStorage
 */
export function save(match) {
  try {
    const list = load();
    const existingIdx = list.findIndex((m) => m.id === match.id);
    const record = {
      id: match.id || 'game' + Date.now(),
      stamp: match.stamp || new Date().toISOString(),
      mode: match.mode || 'wasm',
      depth: match.depth || 6,
      fen: match.fen,
      history: match.history || [match.fen],
      movesCount: match.history ? match.history.length - 1 : 0,
      over: match.over || false,
      winner: match.winner || null,
      reason: match.reason || null
    };

    if (existingIdx >= 0) {
      list[existingIdx] = record;
    } else {
      list.unshift(record);
    }

    // Giới hạn lưu giữ tối đa 50 ván đấu gần nhất
    if (list.length > LIMIT) {
      list.length = LIMIT;
    }

    localStorage.setItem(KEY, JSON.stringify(list));
    return record;
  } catch (err) {
    console.error('Không thể lưu ván đấu:', err);
    return null;
  }
}

/**
 * Xóa toàn bộ nhật ký lịch sử ván đấu khỏi bộ nhớ
 */
export function clear() {
  try {
    localStorage.removeItem(KEY);
    return true;
  } catch (err) {
    console.error('Không thể xóa lịch sử:', err);
    return false;
  }
}

/**
 * Xuất chuỗi PGN chuẩn từ danh sách các nước đi FEN
 */
export function exportPGN(history, result = '*') {
  let pgn = '[Event "XiangRust Match"]\n';
  pgn += `[Date "${new Date().toISOString().split('T')[0]}"]\n`;
  pgn += `[Result "${result}"]\n\n`;

  for (let i = 1; i < history.length; i++) {
    const moveNum = Math.floor((i + 1) / 2);
    if (i % 2 === 1) {
      pgn += `${moveNum}. ${history[i]} `;
    } else {
      pgn += `${history[i]} `;
    }
  }

  return pgn.trim();
}

import { saveExperienceDb, loadExperienceDb } from './db.js';

const dataset = 'dataset';

/**
 * Nạp toàn bộ kho trí nhớ kinh nghiệm AI từ LocalStorage & IndexedDB
 */
export function loadExperience() {
  try {
    const raw = localStorage.getItem(dataset);
    const list = raw ? JSON.parse(raw) : [];
    loadExperienceDb().then(() => {}).catch(() => {});
    return list;
  } catch (err) {
    console.error('Không thể nạp kinh nghiệm AI:', err);
    return [];
  }
}

/**
 * Lưu mẫu kinh nghiệm AI vào IndexedDB (không giới hạn) và LocalStorage (chống tràn 5MB)
 */
export function saveExperience(sample) {
  try {
    saveExperienceDb(sample).catch(() => {});
    const list = loadExperience();
    list.push(sample);
    if (list.length > 1000) {
      list.shift(); // Giới hạn mảng LocalStorage ở mức 1000 phần tử an toàn
    }
    localStorage.setItem(dataset, JSON.stringify(list));
    return list.length;
  } catch (err) {
    console.error('Không thể lưu kinh nghiệm AI:', err);
    return 0;
  }
}
