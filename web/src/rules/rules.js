// Bộ sinh nước đi hợp lệ và kiểm tra luật Cờ Tướng 9x10 client-side 0ms
// Định danh đơn từ tiếng Anh: parse, fen, moves, check, valid, inside, pseudo, face, board, square, turn, piece, color, type, file, rank, x, y, dx, dy, nx, ny, step, screen, leg, eye, target, t, list, king, cloned, parts, row, rows, char, r, c, idx, home, palace, raw, symbol, side, lx, ly, dir, crossed, sx, sy, empty, min, max, red, black, rx, bx, ry, by

// Giải mã chuỗi FEN thành mảng 90 ô cờ và lượt đi ('w' hoặc 'b')
export function parse(fen) {
  if (!fen || typeof fen !== 'string') return { board: new Array(90).fill('.'), turn: 'w' };
  const parts = fen.trim().split(' ');
  const rows = parts[0].split('/');
  if (rows.length !== 10) return { board: new Array(90).fill('.'), turn: 'w' };
  const board = new Array(90).fill('.');
  
  for (let r = 0; r < 10; r++) {
    const rank = 9 - r;
    const row = rows[r];
    let file = 0;
    
    for (let c = 0; c < row.length; c++) {
      const char = row[c];
      if (char >= '1' && char <= '9') {
        file += parseInt(char, 10);
      } else {
        const idx = rank * 9 + file;
        board[idx] = char;
        file += 1;
      }
    }
  }
  
  const turn = parts[1] ? parts[1] : 'w';
  return { board, turn };
}

// Mã hóa mảng 90 ô cờ và lượt đi thành chuỗi FEN
export function fen(board, turn) {
  const rows = [];
  
  for (let r = 0; r < 10; r++) {
    const rank = 9 - r;
    let row = '';
    let empty = 0;
    
    for (let file = 0; file < 9; file++) {
      const idx = rank * 9 + file;
      const piece = board[idx];
      
      if (piece === '.') {
        empty += 1;
      } else {
        if (empty > 0) {
          row += empty.toString();
          empty = 0;
        }
        row += piece;
      }
    }
    
    if (empty > 0) {
      row += empty.toString();
    }
    rows.push(row);
  }
  
  return rows.join('/') + ' ' + turn;
}

// Kiểm tra điểm tọa độ (x, y) có nằm trong bàn cờ 9x10 hay không
function inside(x, y) {
  return x >= 0 && x <= 8 && y >= 0 && y <= 9;
}

// Kiểm tra 2 Tướng có đối mặt trực tiếp trên cùng cột không (Luật Lộ Mặt Tướng)
function face(board) {
  let red = -1;
  let black = -1;
  
  for (let idx = 0; idx < 90; idx++) {
    if (board[idx] === 'K') red = idx;
    if (board[idx] === 'k') black = idx;
  }
  
  if (red === -1 || black === -1) return false;
  
  const rx = red % 9;
  const bx = black % 9;
  
  if (rx !== bx) return false;
  
  const ry = Math.floor(red / 9);
  const by = Math.floor(black / 9);
  const min = Math.min(ry, by);
  const max = Math.max(ry, by);
  
  for (let y = min + 1; y < max; y++) {
    if (board[y * 9 + rx] !== '.') {
      return false;
    }
  }
  
  return true;
}

// Sinh danh sách nước đi giả định (Pseudo-legal moves) cho một ô cờ
function pseudo(board, square) {
  const piece = board[square];
  if (piece === '.') return [];
  
  const color = piece === piece.toUpperCase() ? 'w' : 'b';
  const type = piece.toUpperCase();
  const x = square % 9;
  const y = Math.floor(square / 9);
  const list = [];
  
  if (type === 'K') {
    const dx = [0, 0, 1, -1];
    const dy = [1, -1, 0, 0];
    for (let i = 0; i < 4; i++) {
      const nx = x + dx[i];
      const ny = y + dy[i];
      if (inside(nx, ny)) {
        const palace = nx >= 3 && nx <= 5 && (color === 'w' ? ny >= 0 && ny <= 2 : ny >= 7 && ny <= 9);
        if (palace) {
          const target = ny * 9 + nx;
          const t = board[target];
          if (t === '.' || (t === t.toUpperCase() ? 'w' : 'b') !== color) {
            list.push(target);
          }
        }
      }
    }
  } else if (type === 'A') {
    const dx = [1, 1, -1, -1];
    const dy = [1, -1, 1, -1];
    for (let i = 0; i < 4; i++) {
      const nx = x + dx[i];
      const ny = y + dy[i];
      if (inside(nx, ny)) {
        const palace = nx >= 3 && nx <= 5 && (color === 'w' ? ny >= 0 && ny <= 2 : ny >= 7 && ny <= 9);
        if (palace) {
          const target = ny * 9 + nx;
          const t = board[target];
          if (t === '.' || (t === t.toUpperCase() ? 'w' : 'b') !== color) {
            list.push(target);
          }
        }
      }
    }
  } else if (type === 'B') {
    const dx = [2, 2, -2, -2];
    const dy = [2, -2, 2, -2];
    for (let i = 0; i < 4; i++) {
      const nx = x + dx[i];
      const ny = y + dy[i];
      if (inside(nx, ny)) {
        const home = color === 'w' ? ny <= 4 : ny >= 5;
        if (home) {
          const eye = Math.floor(y + dy[i] / 2) * 9 + Math.floor(x + dx[i] / 2);
          if (board[eye] === '.') {
            const target = ny * 9 + nx;
            const t = board[target];
            if (t === '.' || (t === t.toUpperCase() ? 'w' : 'b') !== color) {
              list.push(target);
            }
          }
        }
      }
    }
  } else if (type === 'N') {
    const dx = [1, 1, -1, -1, 2, 2, -2, -2];
    const dy = [2, -2, 2, -2, 1, -1, 1, -1];
    for (let i = 0; i < 8; i++) {
      const nx = x + dx[i];
      const ny = y + dy[i];
      if (inside(nx, ny)) {
        const lx = Math.abs(dx[i]) === 2 ? x + dx[i] / 2 : x;
        const ly = Math.abs(dy[i]) === 2 ? y + dy[i] / 2 : y;
        const leg = ly * 9 + lx;
        if (board[leg] === '.') {
          const target = ny * 9 + nx;
          const t = board[target];
          if (t === '.' || (t === t.toUpperCase() ? 'w' : 'b') !== color) {
            list.push(target);
          }
        }
      }
    }
  } else if (type === 'R') {
    const dx = [0, 0, 1, -1];
    const dy = [1, -1, 0, 0];
    for (let i = 0; i < 4; i++) {
      for (let step = 1; step < 10; step++) {
        const nx = x + dx[i] * step;
        const ny = y + dy[i] * step;
        if (!inside(nx, ny)) break;
        const target = ny * 9 + nx;
        const t = board[target];
        if (t === '.') {
          list.push(target);
        } else {
          if ((t === t.toUpperCase() ? 'w' : 'b') !== color) {
            list.push(target);
          }
          break;
        }
      }
    }
  } else if (type === 'C') {
    const dx = [0, 0, 1, -1];
    const dy = [1, -1, 0, 0];
    for (let i = 0; i < 4; i++) {
      let screen = 0;
      for (let step = 1; step < 10; step++) {
        const nx = x + dx[i] * step;
        const ny = y + dy[i] * step;
        if (!inside(nx, ny)) break;
        const target = ny * 9 + nx;
        const t = board[target];
        if (screen === 0) {
          if (t === '.') {
            list.push(target);
          } else {
            screen = 1;
          }
        } else {
          if (t !== '.') {
            if ((t === t.toUpperCase() ? 'w' : 'b') !== color) {
              list.push(target);
            }
            break;
          }
        }
      }
    }
  } else if (type === 'P') {
    const dir = color === 'w' ? 1 : -1;
    const nx = x;
    const ny = y + dir;
    if (inside(nx, ny)) {
      const target = ny * 9 + nx;
      const t = board[target];
      if (t === '.' || (t === t.toUpperCase() ? 'w' : 'b') !== color) {
        list.push(target);
      }
    }
    const crossed = color === 'w' ? y >= 5 : y <= 4;
    if (crossed) {
      const side = [x - 1, x + 1];
      for (let i = 0; i < 2; i++) {
        const sx = side[i];
        const sy = y;
        if (inside(sx, sy)) {
          const target = sy * 9 + sx;
          const t = board[target];
          if (t === '.' || (t === t.toUpperCase() ? 'w' : 'b') !== color) {
            list.push(target);
          }
        }
      }
    }
  }
  
  return list;
}

// Kiểm tra bên lượt turn có đang bị chiếu tướng hay không
export function check(board, turn) {
  const symbol = turn === 'w' ? 'K' : 'k';
  let king = -1;
  
  for (let idx = 0; idx < 90; idx++) {
    if (board[idx] === symbol) {
      king = idx;
      break;
    }
  }
  
  if (king === -1) return true;
  if (face(board)) return true;
  
  for (let idx = 0; idx < 90; idx++) {
    const piece = board[idx];
    if (piece !== '.') {
      const color = piece === piece.toUpperCase() ? 'w' : 'b';
      if (color !== turn) {
        const targets = pseudo(board, idx);
        if (targets.includes(king)) {
          return true;
        }
      }
    }
  }
  
  return false;
}

// Sinh danh sách nước đi hoàn toàn hợp lệ (Legal moves) cho ô cờ
export function moves(board, square, turn) {
  const piece = board[square];
  if (piece === '.') return [];
  
  const color = piece === piece.toUpperCase() ? 'w' : 'b';
  if (turn && color !== turn) return [];
  
  const raw = pseudo(board, square);
  const valid = [];
  
  for (let i = 0; i < raw.length; i++) {
    const target = raw[i];
    const cloned = [...board];
    cloned[target] = cloned[square];
    cloned[square] = '.';
    
    if (!check(cloned, color) && !face(cloned)) {
      valid.push(target);
    }
  }
  
  return valid;
}

// Kiểm tra xem bên phe turn có còn bất kỳ nước đi hợp lệ nào không
export function hasLegalMoves(board, turn) {
  for (let square = 0; square < 90; square++) {
    const piece = board[square];
    if (piece !== '.') {
      const color = piece === piece.toUpperCase() ? 'w' : 'b';
      if (color === turn) {
        const valid = moves(board, square, turn);
        if (valid.length > 0) return true;
      }
    }
  }
  return false;
}

// Chuyển đổi nước đi chuẩn UCI (ví dụ 'h7e7') sang chỉ số ô cờ {from, to}
export function uciToMove(uci) {
  if (!uci || typeof uci !== 'string' || uci.length < 4) return null;
  const f1 = uci.charCodeAt(0) - 97; // 'a' = 97
  const r1 = parseInt(uci[1], 10);
  const f2 = uci.charCodeAt(2) - 97;
  const r2 = parseInt(uci[3], 10);
  if (isNaN(f1) || isNaN(r1) || isNaN(f2) || isNaN(r2)) return null;
  if (f1 < 0 || f1 > 8 || f2 < 0 || f2 > 8 || r1 < 0 || r1 > 9 || r2 < 0 || r2 > 9) return null;
  const from = r1 * 9 + f1;
  const to = r2 * 9 + f2;
  if (from < 0 || from >= 90 || to < 0 || to >= 90) return null;
  return { from, to };
}


