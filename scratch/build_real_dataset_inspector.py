import json
import os

# 1. READ REAL DATASET FROM /Users/hdqb/workspaces/xiangqi-rim/tools/games-completed.jsonl
real_games = []
with open('/Users/hdqb/workspaces/xiangqi-rim/tools/games-completed.jsonl', 'r', encoding='utf-8') as f:
    for line in f:
        line_str = line.strip()
        if line_str:
            try:
                real_games.append(json.loads(line_str))
            except Exception as e:
                print("Error loading real line:", e)

print(f"Loaded {len(real_games)} REAL game records from games-completed.jsonl!")

# Convert real games to JSON string for JS
real_games_json_str = json.dumps(real_games, ensure_ascii=False)

# Build React / TSX inspired Standalone Studio Web App HTML
html_content = f"""<!DOCTYPE html>
<html lang="vi">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Xiangqi-R1 Studio · Trạm Quan Sát Suy Luận 32D (Bản Chính Thức)</title>
    <!-- Fonts -->
    <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600;700&family=Lora:ital,wght@0,400;0,500;0,600;1,400&family=Noto+Serif+TC:wght@600;700;900&display=swap" rel="stylesheet">
    <style>
        :root {{
            --ink: #0B0E14;
            --panel: #12161F;
            --panel2: #171C27;
            --rule: #232A38;
            --ruleSoft: #1A2029;
            --signal: #4FD3C4;
            --signalDim: #2B7A70;
            --seal: #C1392B;
            --brass: #C89B3C;
            --paper: #E8E4D9;
            --text: #E7E9EE;
            --textSoft: #8B93A7;
        }}

        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            background-color: var(--ink);
            color: var(--text);
            font-family: 'Lora', Georgia, serif;
            min-height: 100vh;
            display: flex; flex-direction: column;
        }}

        .font-mono {{ font-family: 'JetBrains Mono', monospace; }}
        .font-xiangqi {{ font-family: 'Noto Serif TC', serif; }}

        ::-webkit-scrollbar {{ width: 8px; height: 8px; }}
        ::-webkit-scrollbar-track {{ background: var(--ink); }}
        ::-webkit-scrollbar-thumb {{ background: var(--rule); border-radius: 4px; }}
        ::-webkit-scrollbar-thumb:hover {{ background: #374151; }}

        header {{
            padding: 20px 24px 16px;
            background-image: radial-gradient(var(--rule) 1px, transparent 1px);
            background-size: 22px 22px;
            border-bottom: 1px solid var(--rule);
        }}

        .header-container {{
            max-width: 1400px; margin: 0 auto;
            display: flex; flex-direction: column; gap: 8px;
        }}

        .top-subtitle {{
            font-family: 'JetBrains Mono', monospace;
            font-size: 11px; color: var(--signal); letter-spacing: 0.18em;
            display: flex; align-items: center; justify-content: space-between;
        }}

        .h1-title {{
            font-family: 'JetBrains Mono', monospace;
            font-size: 28px; font-weight: 700; color: var(--text);
            letter-spacing: -0.01em; margin: 0;
            display: flex; align-items: center; gap: 8px;
        }}

        .dot-seal {{ color: var(--seal); }}

        .subtitle-desc {{
            font-size: 14px; color: var(--textSoft); max-width: 800px; line-height: 1.5;
        }}

        /* Buttons */
        .btn {{
            font-family: 'JetBrains Mono', monospace;
            font-size: 11px; padding: 6px 12px; border-radius: 7px;
            cursor: pointer; border: 1px solid transparent; transition: all 0.15s ease;
            display: inline-flex; align-items: center; gap: 6px; user-select: none;
        }}
        .btn-secondary {{ background: var(--panel2); color: var(--textSoft); border-color: var(--rule); }}
        .btn-secondary:hover {{ background: var(--rule); color: var(--text); }}
        .btn-primary {{ background: var(--signal); color: var(--ink); border-color: var(--signal); font-weight: 700; }}
        .btn-primary:hover {{ opacity: 0.9; }}
        .btn-brass {{ background: var(--brass); color: var(--ink); border-color: var(--brass); font-weight: 700; }}

        main {{
            max-width: 1400px; width: 100%; margin: 0 auto;
            padding: 20px 24px; display: flex; flex-direction: column; gap: 20px;
        }}

        .layout-grid {{
            display: grid; grid-template-columns: 1fr; gap: 20px;
        }}
        @media (min-width: 1024px) {{
            .layout-grid {{ grid-template-columns: 5fr 7fr; }}
        }}

        .card {{
            background: var(--panel); border: 1px solid var(--rule);
            border-radius: 12px; padding: 16px; display: flex; flex-direction: column; gap: 12px;
        }}

        /* Board Geometry */
        .xq-grid {{
            display: grid; grid-template-columns: repeat(9, 1fr); grid-template-rows: repeat(10, 1fr);
            aspect-ratio: 9 / 10; background: var(--panel2); border: 2px solid var(--rule);
            border-radius: 10px; overflow: hidden; position: relative;
        }}
        .xq-cell {{
            border: 1px solid var(--ruleSoft); display: flex; align-items: center; justify-content: center; position: relative;
        }}
        .piece {{
            width: 84%; height: 84%; border-radius: 50%; display: flex; align-items: center; justify-content: center;
            font-size: 1.25rem; font-weight: 900; background: var(--paper); box-shadow: 0 4px 6px rgba(0,0,0,0.5);
            z-index: 10;
        }}
        .piece-red {{ color: var(--seal); border: 2px solid var(--seal); }}
        .piece-black {{ color: #1C1E24; border: 2px solid #20242D; }}
        .bg-src {{ background-color: rgba(200, 155, 60, 0.25) !important; }}
        .bg-dst {{ background-color: rgba(79, 211, 196, 0.25) !important; }}
        .piece-src {{ box-shadow: 0 0 0 3px var(--brass); }}
        .piece-dst {{ box-shadow: 0 0 0 3px var(--signal); }}

        /* 32D Cards */
        .dim-card {{
            background: var(--panel2); border: 1px solid var(--ruleSoft);
            border-left: 4px solid var(--signal); border-radius: 8px; padding: 10px 12px;
            margin-bottom: 8px; font-family: 'JetBrains Mono', monospace; font-size: 11px;
        }}
        .dim-grp-1 {{ border-left-color: var(--signal); }}
        .dim-grp-2 {{ border-left-color: var(--seal); }}
        .dim-grp-3 {{ border-left-color: var(--brass); }}
        .dim-grp-4 {{ border-left-color: #3b82f6; }}
        .dim-grp-5 {{ border-left-color: #a855f7; }}
        .dim-grp-6 {{ border-left-color: #ec4899; }}

        .dim-header {{ display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px; }}
        .dim-num {{ color: var(--signal); font-weight: 700; }}
        .dim-title {{ color: var(--text); font-weight: 600; margin-left: 4px; }}
        .dim-body {{ color: var(--textSoft); white-space: pre-wrap; line-height: 1.5; padding-left: 6px; border-left: 1px solid var(--rule); margin-top: 4px; }}

        .select-input {{
            background: var(--ink); border: 1px solid var(--rule); color: var(--text);
            border-radius: 6px; padding: 6px 10px; font-family: 'JetBrains Mono', monospace; font-size: 11px;
            outline: none; cursor: pointer; width: 100%;
        }}

        .pre-box {{
            background: var(--ink); border: 1px solid var(--ruleSoft); border-radius: 8px;
            padding: 10px; font-family: 'JetBrains Mono', monospace; font-size: 11px; color: var(--text);
            white-space: pre-wrap; max-height: 240px; overflow-y: auto;
        }}

        /* Modal */
        .modal-backdrop {{
            position: fixed; inset: 0; background: rgba(0,0,0,0.8); backdrop-filter: blur(6px);
            display: none; align-items: center; justify-content: center; z-index: 100; padding: 16px;
        }}
        .modal-content {{
            background: var(--panel); border: 1px solid var(--rule); border-radius: 14px;
            width: 100%; max-width: 680px; padding: 20px; display: flex; flex-direction: column; gap: 12px;
        }}
    </style>
</head>
<body>

    <header>
        <div class="header-container">
            <div class="top-subtitle">
                <span>TRẠM QUAN SÁT SUY LUẬN · XIANGQI-R1 CONSOLE REAL DATASET INSPECTOR</span>
                <span style="color: var(--signal);">● REAL DATASET LOADED ({len(real_games)} GAMES)</span>
            </div>
            <h1 class="h1-title">
                XIANGQI <span class="dot-seal">·</span> R1 <span style="font-size: 14px; color: var(--textSoft); font-weight: 400;">v5.0 Master Real Dataset Visualizer</span>
            </h1>
            <p class="subtitle-desc">
                Hệ thống soi dữ liệu cờ Tướng suy luận 32 chiều kích từ file thực tế <code class="font-mono" style="color: var(--signal);">tools/games-completed.jsonl</code>.
                Tải trực tiếp mạch hội thoại multi-turn thực tế, hiển thị bàn cờ 2D chuẩn UCCI, trích xuất 32D thought chain và ma trận ứng viên.
            </p>
        </div>
    </header>

    <main>

        <!-- Top Control Status Bar -->
        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(320px, 1fr)); gap: 16px;">
            <div class="card">
                <div style="display: flex; justify-content: space-between; align-items: center;">
                    <span class="font-mono" style="font-size: 11px; color: var(--textSoft); font-weight: 700;">🎮 VÁN ĐẤU THỰC TẾ (REAL GAMES)</span>
                    <span id="realBadge" class="font-mono" style="font-size: 9px; padding: 2px 6px; border-radius: 4px; background: rgba(79, 211, 196, 0.2); color: var(--signal);">
                        100% Real File Data
                    </span>
                </div>
                <select id="gameSelect" class="select-input" onchange="onGameChange(this.value)">
                </select>
                <div style="display: flex; justify-content: space-between; font-family: 'JetBrains Mono', monospace; font-size: 11px;">
                    <span style="color: var(--textSoft);">Game ID: <strong id="metaGameId" style="color: var(--brass);">-</strong></span>
                    <span style="color: var(--textSoft);">Tổng nước: <strong id="metaPlies" style="color: var(--signal);">-</strong></span>
                </div>
            </div>

            <div class="card" style="justify-content: space-between;">
                <div style="display: flex; justify-content: space-between; align-items: center;">
                    <span class="font-mono" style="font-size: 11px; color: var(--textSoft); font-weight: 700;">♟️ LƯỢT ĐẤU MULTI-TURN (REAL PLIES)</span>
                    <span id="turnCounter" class="font-mono" style="font-size: 13px; font-weight: 700; color: var(--signal);">Turn 1 / 1</span>
                </div>

                <div style="display: flex; align-items: center; gap: 8px; margin-top: 8px;">
                    <button onclick="prevTurnStep()" class="btn btn-secondary">◀ Nước Trước</button>
                    <input type="range" id="turnSlider" min="1" max="1" value="1" oninput="onSliderMove(this.value)" style="flex: 1; accent-color: var(--signal); cursor: pointer;">
                    <button onclick="nextTurnStep()" class="btn btn-primary">Nước Sau ▶</button>
                </div>
            </div>
        </div>

        <!-- Main Layout -->
        <div class="layout-grid">

            <!-- Left Column: Board & Markdown Dialogue -->
            <div style="display: flex; flex-direction: column; gap: 16px;">
                <div class="card">
                    <div style="display: flex; justify-content: space-between; align-items: center;">
                        <span class="font-mono" style="font-size: 12px; font-weight: 700;">♟️ BÀN CỜ 2D TRỰC QUAN</span>
                        <span id="turnSideBadge" class="font-mono" style="font-size: 10px; padding: 2px 8px; border-radius: 999px; background: rgba(193, 57, 43, 0.2); color: #fca5a5; border: 1px solid rgba(193, 57, 43, 0.3);">
                            Lượt Đỏ
                        </span>
                    </div>

                    <div id="xiangqiBoardGrid" class="xq-grid"></div>
                    <div id="fenDisplay" class="font-mono" style="font-size: 10px; color: var(--textSoft); word-break: break-all;"></div>
                </div>

                <div class="card">
                    <span class="font-mono" style="font-size: 12px; font-weight: 700; border-bottom: 1px solid var(--rule); padding-bottom: 6px;">
                        💬 PREVIEW LLM DIALOGUE (THỰC TẾ)
                    </span>

                    <div>
                        <div class="font-mono" style="font-size: 10px; color: var(--brass); margin-bottom: 4px;">👤 USER INPUT:</div>
                        <div id="userPromptText" class="pre-box"></div>
                    </div>

                    <div>
                        <div class="font-mono" style="font-size: 10px; color: var(--signal); margin-bottom: 4px;">🤖 ASSISTANT RESPONSE:</div>
                        <div id="assistantResponseText" class="pre-box" style="max-height: 280px;"></div>
                    </div>
                </div>
            </div>

            <!-- Right Column: 32D Thought Inspector -->
            <div class="card">
                <div style="display: flex; justify-content: space-between; align-items: center;">
                    <span class="font-mono" style="font-size: 13px; font-weight: 700; color: var(--text);">
                        🧠 MẠCH SUY TƯỞNG 32 CHIỀU KÍCH (&lt;thought&gt;)
                    </span>
                    <span id="dimComplianceBadge" class="font-mono" style="font-size: 10px; padding: 2px 6px; border-radius: 4px; background: rgba(79, 211, 196, 0.2); color: var(--signal);">
                        32/32 Valid
                    </span>
                </div>

                <input type="text" id="dimSearchInput" oninput="search32D(this.value)" placeholder="🔍 Tìm kiếm từ khóa trong 32 chiều kích (Xe, Pháo, Ứng viên, Bẫy...)..." class="select-input">

                <div id="dimsCardsContainer" style="overflow-y: auto; max-height: 750px;"></div>
            </div>

        </div>

        <div class="card">
            <div style="display: flex; justify-content: space-between; align-items: center;">
                <span class="font-mono" style="font-size: 12px; font-weight: 700;">📄 RAW JSON RECORD (THỰC TẾ từ games-completed.jsonl)</span>
                <button onclick="copyRawRecordJson()" class="btn btn-secondary">📋 Copy Raw JSON</button>
            </div>
            <pre id="rawRecordJsonBox" class="pre-box" style="max-height: 180px; color: #a7f3d0;"></pre>
        </div>

    </main>

    <script>
        const REAL_GAMES_LIST = {real_games_json_str};
        let activeGameIndex = 0;
        let activeTurnIndex = 0;

        const PIECES_MAP = {{
            'r': {{ name: '車', side: 'black' }}, 'n': {{ name: '馬', side: 'black' }}, 'b': {{ name: '象', side: 'black' }},
            'a': {{ name: '士', side: 'black' }}, 'k': {{ name: '將', side: 'black' }}, 'c': {{ name: '砲', side: 'black' }},
            'p': {{ name: '卒', side: 'black' }},
            'R': {{ name: '車', side: 'red' }}, 'N': {{ name: '馬', side: 'red' }}, 'B': {{ name: '相', side: 'red' }},
            'A': {{ name: '仕', side: 'red' }}, 'K': {{ name: '帥', side: 'red' }}, 'C': {{ name: '炮', side: 'red' }},
            'P': {{ name: '兵', side: 'red' }}
        }};

        function initApp() {{
            populateGameSelector();
        }}

        function populateGameSelector() {{
            const selectEl = document.getElementById('gameSelect');
            selectEl.innerHTML = '';

            REAL_GAMES_LIST.forEach((game, idx) => {{
                const opt = document.createElement('option');
                opt.value = idx;
                const gameId = game.game_id || ("Game_" + (idx + 1));
                const convMsgs = (game.messages || []).filter(m => m.role === 'user' || m.role === 'assistant');
                const turns = Math.floor(convMsgs.length / 2);
                opt.textContent = "Ván #" + (idx + 1) + " (Game ID: " + gameId + " — " + turns + " Lượt Hội Thoại Thật — Plies " + (game.total_plies || '36') + ")";
                selectEl.appendChild(opt);
            }});

            activeGameIndex = 0;
            selectEl.value = 0;
            loadActiveGame(0);
        }}

        function onGameChange(val) {{
            activeGameIndex = parseInt(val);
            loadActiveGame(activeGameIndex);
        }}

        function loadActiveGame(index) {{
            const game = REAL_GAMES_LIST[index];
            if (!game) return;

            const convMsgs = (game.messages || []).filter(m => m.role === 'user' || m.role === 'assistant');
            const totalTurns = Math.floor(convMsgs.length / 2);

            document.getElementById('metaGameId').textContent = game.game_id || ("Game_" + (index + 1));
            document.getElementById('metaPlies').textContent = (game.total_plies || (totalTurns * 2)) + ' plies';

            const slider = document.getElementById('turnSlider');
            slider.min = 1;
            slider.max = Math.max(1, totalTurns);
            slider.value = 1;

            activeTurnIndex = 0;
            renderTurnStep(0);
        }}

        function onSliderMove(val) {{
            activeTurnIndex = parseInt(val) - 1;
            renderTurnStep(activeTurnIndex);
        }}

        function prevTurnStep() {{
            if (activeTurnIndex > 0) {{
                activeTurnIndex--;
                document.getElementById('turnSlider').value = activeTurnIndex + 1;
                renderTurnStep(activeTurnIndex);
            }}
        }}

        function nextTurnStep() {{
            const game = REAL_GAMES_LIST[activeGameIndex];
            if (!game) return;
            const convMsgs = (game.messages || []).filter(m => m.role === 'user' || m.role === 'assistant');
            const totalTurns = Math.floor(convMsgs.length / 2);

            if (activeTurnIndex < totalTurns - 1) {{
                activeTurnIndex++;
                document.getElementById('turnSlider').value = activeTurnIndex + 1;
                renderTurnStep(activeTurnIndex);
            }}
        }}

        function renderTurnStep(turnIdx) {{
            const game = REAL_GAMES_LIST[activeGameIndex];
            if (!game) return;

            const convMsgs = (game.messages || []).filter(m => m.role === 'user' || m.role === 'assistant');
            const totalTurns = Math.floor(convMsgs.length / 2);

            document.getElementById('turnCounter').textContent = "Lượt " + (turnIdx + 1) + " / " + totalTurns;

            const userMsg = convMsgs[turnIdx * 2];
            const assistantMsg = convMsgs[turnIdx * 2 + 1];

            if (!userMsg || !assistantMsg) return;

            document.getElementById('userPromptText').textContent = userMsg.content;
            document.getElementById('assistantResponseText').textContent = assistantMsg.content;

            let fen = "rnbakabnr/9/1c2c4/p1p1p1p1p/9/9/P1P1P1P1P/4C2C1/9/RNBAKABNR w - - 0 1";
            const fenMatch = userMsg.content.match(new RegExp('FEN:\\\\s*([^\\\\n]+)'));
            if (fenMatch) fen = fenMatch[1].trim();
            document.getElementById('fenDisplay').textContent = "FEN: " + fen;

            const sideMatch = userMsg.content.match(new RegExp('Lượt\\\\s*(Đỏ|Đen)\\\\s*đi', 'i'));
            const sideText = sideMatch ? sideMatch[1] : (fen.includes(' w ') ? 'Đỏ' : 'Đen');
            const badge = document.getElementById('turnSideBadge');
            badge.textContent = "Lượt " + sideText;
            if (sideText === 'Đỏ') {{
                badge.style.background = "rgba(193, 57, 43, 0.2)";
                badge.style.color = "#fca5a5";
            }} else {{
                badge.style.background = "rgba(79, 211, 196, 0.2)";
                badge.style.color = "#4FD3C4";
            }}

            let bestMove = "";
            const moveMatch = assistantMsg.content.match(new RegExp('Chọn\\\\s+([a-i][0-9][a-i][0-9])'));
            if (moveMatch) bestMove = moveMatch[1];

            drawXiangqiBoard(fen, bestMove);
            render32DThoughtCards(assistantMsg.content);
            document.getElementById('rawRecordJsonBox').textContent = JSON.stringify(game, null, 2);
        }}

        function drawXiangqiBoard(fen, bestMove) {{
            const gridEl = document.getElementById('xiangqiBoardGrid');
            gridEl.innerHTML = '';

            const fenParts = fen.split(' ');
            const rows = fenParts[0].split('/');

            let moveSrc = -1, moveDst = -1;
            if (bestMove && bestMove.length === 4) {{
                const c1 = bestMove.charCodeAt(0) - 'a'.charCodeAt(0);
                const r1 = parseInt(bestMove[1]);
                const c2 = bestMove.charCodeAt(2) - 'a'.charCodeAt(0);
                const r2 = parseInt(bestMove[3]);
                moveSrc = (9 - r1) * 9 + c1;
                moveDst = (9 - r2) * 9 + c2;
            }}

            let cellIndex = 0;
            for (let r = 0; r < 10; r++) {{
                const rowStr = rows[r] || "9";

                for (let i = 0; i < rowStr.length; i++) {{
                    const char = rowStr[i];
                    if (!isNaN(char)) {{
                        const count = parseInt(char);
                        for (let e = 0; e < count; e++) {{
                            gridEl.appendChild(makeCell(cellIndex === moveSrc, cellIndex === moveDst, null));
                            cellIndex++;
                        }}
                    }} else {{
                        const pInfo = PIECES_MAP[char];
                        gridEl.appendChild(makeCell(cellIndex === moveSrc, cellIndex === moveDst, pInfo));
                        cellIndex++;
                    }}
                }}
            }}
        }}

        function makeCell(isSrc, isDst, pieceInfo) {{
            const cell = document.createElement('div');
            cell.className = 'xq-cell';
            if (isSrc) cell.classList.add('bg-src');
            if (isDst) cell.classList.add('bg-dst');

            if (pieceInfo) {{
                const pEl = document.createElement('div');
                pEl.className = "piece " + (pieceInfo.side === 'red' ? 'piece-red' : 'piece-black') + " font-xiangqi";
                pEl.textContent = pieceInfo.name;
                if (isSrc) pEl.classList.add('piece-src');
                if (isDst) pEl.classList.add('piece-dst');
                cell.appendChild(pEl);
            }}
            return cell;
        }}

        function render32DThoughtCards(thoughtText) {{
            const container = document.getElementById('dimsCardsContainer');
            container.innerHTML = '';

            const dimReg = new RegExp('\\\\[\\\\d+/32\\\\][^\\\\n]+(?:\\\\n(?!\\\\[\\\\d+/32\\\\])[^\\\\n]+)*', 'g');
            const dimBlocks = thoughtText.match(dimReg);

            if (!dimBlocks) {{
                container.innerHTML = '<div style="font-size: 11px; color: var(--textSoft); padding: 12px;">Không thể phân tách các khối [X/32] từ suy tưởng.</div>';
                return;
            }}

            document.getElementById('dimComplianceBadge').textContent = dimBlocks.length + '/32 Complete';

            const headerReg = new RegExp('^\\\\[(\\\\d+)/32\\\\]\\\\s*([^\\\\n:]+):?(.*)');

            dimBlocks.forEach(block => {{
                const lines = block.split('\\n');
                const headerLine = lines[0];
                const bodyLines = lines.slice(1).join('\\n');

                const match = headerLine.match(headerReg);
                if (!match) return;

                const dimNum = parseInt(match[1]);
                const dimTitle = match[2].trim();
                const headerExtra = match[3].trim();
                const bodyText = (headerExtra ? headerExtra + '\\n' : '') + bodyLines;

                let grpClass = 'dim-grp-1';
                let grpTag = 'Nhận thức';
                if (dimNum >= 7 && dimNum <= 12) {{ grpClass = 'dim-grp-2'; grpTag = 'Đe dọa'; }}
                else if (dimNum >= 13 && dimNum <= 18) {{ grpClass = 'dim-grp-3'; grpTag = 'Chiến thuật'; }}
                else if (dimNum >= 19 && dimNum <= 22) {{ grpClass = 'dim-grp-4'; grpTag = 'Binh pháp'; }}
                else if (dimNum >= 23 && dimNum <= 28) {{ grpClass = 'dim-grp-5'; grpTag = 'Quyết định'; }}
                else if (dimNum >= 29) {{ grpClass = 'dim-grp-6'; grpTag = 'Luật đấu'; }}

                const card = document.createElement('div');
                card.className = "dim-card " + grpClass;

                card.innerHTML = `
                    <div class="dim-header">
                        <div>
                            <span class="dim-num">[${{dimNum}}/32]</span>
                            <span class="dim-title">${{escapeStr(dimTitle)}}</span>
                        </div>
                        <span style="font-size: 9px; padding: 2px 6px; background: var(--rule); border-radius: 4px; color: var(--textSoft);">${{grpTag}}</span>
                    </div>
                    ${{bodyText.trim() ? `<div class="dim-body">${{escapeStr(bodyText.trim())}}</div>` : ''}}
                `;

                container.appendChild(card);
            }});
        }}

        function escapeStr(str) {{
            return str.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
        }}

        function search32D(query) {{
            const q = query.toLowerCase();
            document.querySelectorAll('.dim-card').forEach(card => {{
                const text = card.textContent.toLowerCase();
                card.style.display = text.includes(q) ? 'block' : 'none';
            }});
        }}

        function copyRawRecordJson() {{
            const rawText = document.getElementById('rawRecordJsonBox').textContent;
            navigator.clipboard.writeText(rawText).then(() => {{
                alert("📋 Đã copy Raw JSON Record thực tế vào Clipboard!");
            }});
        }}

        window.addEventListener('DOMContentLoaded', initApp);
    </script>
</body>
</html>
"""

with open('/Users/hdqb/workspaces/xiangqi-rim/tools/xiangqi_multiturn_32d_inspector.html', 'w', encoding='utf-8') as f:
    f.write(html_content)

print("✅ Perfect Inspector generated with REAL DATASET from games-completed.jsonl!")
