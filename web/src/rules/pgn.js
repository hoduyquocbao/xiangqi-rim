// Bộ trợ giúp mã hóa và giải mã định dạng PGN Cờ Tướng chuẩn
// Định danh đơn từ tiếng Anh: parse, stringify, text, game, moves, headers, event, site, date, red, black, result, line, parts, key, val, item, idx, tag, head, body, row, clean, match, turn, fen, tags, rows, move

// Giải mã chuỗi PGN Cờ Tướng thành đối tượng dữ liệu chứa headers và danh sách nước đi
export function parse(text) {
  const headers = {};
  const moves = [];

  if (!text || typeof text !== 'string') {
    return { headers, moves };
  }

  const lines = text.split('\n');
  let body = '';

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim();
    if (line.startsWith('[')) {
      const match = line.match(/\[(\w+)\s+"([^"]*)"\]/);
      if (match) {
        const key = match[1].toLowerCase();
        const val = match[2];
        headers[key] = val;
      }
    } else if (line.length > 0 && !line.startsWith(';')) {
      body += ' ' + line;
    }
  }

  // Làm sạch phần thân ghi nước đi và tách thành các từ đơn/nước đi
  const clean = body.replace(/\{[^}]*\}/g, '').replace(/(?:^|\s)\d+\.+\s*/g, ' ').trim();
  const parts = clean.split(/\s+/);

  for (let i = 0; i < parts.length; i++) {
    const item = parts[i].trim();
    if (item && item !== '*' && item !== '1-0' && item !== '0-1' && item !== '1/2-1/2') {
      moves.push(item);
    }
  }

  return { headers, moves };
}

// Mã hóa danh sách lịch sử nước đi và thông tin ván đấu thành chuỗi PGN
export function stringify(history, tags = {}) {
  const event = tags.event || 'XiangRust Match';
  const site = tags.site || 'Local Server';
  const date = tags.date || new Date().toISOString().split('T')[0];
  const red = tags.red || 'Player Red';
  const black = tags.black || 'XiangRust AI';
  const result = tags.result || '*';

  const head = [
    `[Event "${event}"]`,
    `[Site "${site}"]`,
    `[Date "${date}"]`,
    `[Red "${red}"]`,
    `[Black "${black}"]`,
    `[Result "${result}"]`
  ].join('\n');

  const rows = [];
  for (let i = 0; i < history.length; i++) {
    const move = history[i];
    if (i % 2 === 0) {
      const idx = Math.floor(i / 2) + 1;
      rows.push(`${idx}. ${move}`);
    } else {
      rows[rows.length - 1] += ` ${move}`;
    }
  }

  const body = rows.join(' ');
  return head + '\n\n' + body + ' ' + result;
}
