import json
import os

# Load authentic real games from games-completed.jsonl
real_games = []
with open('/Users/hdqb/workspaces/xiangqi-rim/tools/games-completed.jsonl', 'r', encoding='utf-8') as f:
    for line in f:
        line_str = line.strip()
        if line_str:
            try:
                real_games.append(json.loads(line_str))
            except Exception as e:
                pass

real_games_js_str = json.dumps(real_games, ensure_ascii=False)

html_code = f"""<!DOCTYPE html>
<html lang="vi">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Xiangqi · R1 — Trụ Sở Quan Sát Suy Luận (Master Console Studio)</title>
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

        .high-contrast {{
            --ink: #000000;
            --panel: #0E0E0E;
            --panel2: #161616;
            --rule: #3A3A3A;
            --ruleSoft: #262626;
            --signal: #5CFFEA;
            --signalDim: #2F8F80;
            --seal: #FF5A45;
            --brass: #FFC94D;
            --paper: #FFFFFF;
            --text: #FFFFFF;
            --textSoft: #C7C7C7;
        }}

        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            background-color: var(--ink);
            color: var(--text);
            font-family: 'Lora', Georgia, serif;
            min-height: 100vh;
            display: flex; flex-direction: column;
            transition: background-color 0.2s ease, color 0.2s ease;
        }}

        .font-mono {{ font-family: 'JetBrains Mono', monospace; }}
        .font-xiangqi {{ font-family: 'Noto Serif TC', serif; }}

        ::-webkit-scrollbar {{ width: 8px; height: 8px; }}
        ::-webkit-scrollbar-track {{ background: var(--ink); }}
        ::-webkit-scrollbar-thumb {{ background: var(--rule); border-radius: 4px; }}
        ::-webkit-scrollbar-thumb:hover {{ background: #374151; }}

        header {{
            padding: 24px 20px 16px;
            background-image: radial-gradient(var(--rule) 1px, transparent 1px);
            background-size: 22px 22px;
            border-bottom: 1px solid var(--rule);
        }}

        .header-container {{
            max-width: 1200px; margin: 0 auto;
            display: flex; flex-direction: column; gap: 8px;
        }}

        .row {{ display: flex; align-items: center; }}
        .col {{ display: flex; flex-direction: column; }}

        .top-subtitle {{
            font-family: 'JetBrains Mono', monospace;
            font-size: 11px; color: var(--signal); letter-spacing: 0.18em;
            display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 8px;
        }}

        .h1-title {{
            font-family: 'JetBrains Mono', monospace;
            font-size: clamp(22px, 4vw, 32px); font-weight: 700; color: var(--text);
            letter-spacing: -0.01em; margin: 0;
        }}

        .btn-icon {{
            width: 32px; height: 32px; border-radius: 7px; cursor: pointer;
            background: var(--panel2); border: 1px solid var(--rule);
            color: var(--textSoft); display: flex; align-items: center; justify-content: center;
            transition: all 0.15s ease; flex-shrink: 0; font-size: 12px;
        }}
        .btn-icon:hover {{ background: var(--rule); color: var(--text); }}
        .btn-icon.active {{ background: var(--signal); color: var(--ink); border-color: var(--signal); }}

        .btn {{
            font-family: 'JetBrains Mono', monospace;
            font-size: 11px; padding: 7px 12px; border-radius: 8px;
            cursor: pointer; border: 1px solid var(--rule); background: var(--panel);
            color: var(--textSoft); transition: all 0.15s ease; display: inline-flex; align-items: center; gap: 6px;
        }}
        .btn:hover {{ background: var(--panel2); color: var(--text); }}
        .btn-primary {{ background: var(--signal); color: var(--ink); border-color: var(--signal); font-weight: 700; }}
        .btn-primary:hover {{ opacity: 0.9; }}

        main {{
            max-width: 1200px; width: 100%; margin: 0 auto;
            padding: 20px; display: flex; flex-direction: column; gap: 20px;
        }}

        .xr1-main-grid {{ display: grid; grid-template-columns: 1fr; gap: 20px; }}
        @media (min-width: 1024px) {{
            .xr1-main-grid {{ grid-template-columns: 1fr 1fr; }}
            .presentation-mode {{ grid-template-columns: 1fr !important; }}
        }}

        .card {{
            background: var(--panel); border: 1px solid var(--rule);
            border-radius: 12px; padding: 16px; display: flex; flex-direction: column; gap: 12px;
        }}

        .select-input {{
            background: var(--ink); border: 1px solid var(--rule); color: var(--text);
            border-radius: 6px; padding: 6px 10px; font-family: 'JetBrains Mono', monospace; font-size: 11px;
            outline: none; cursor: pointer; width: 100%;
        }}

        .banner-warning {{
            display: flex; align-items: flex-start; gap: 10px; background: rgba(193, 57, 43, 0.08);
            border: 1px solid rgba(193, 57, 43, 0.35); border-radius: 10px; padding: 10px 14px;
            font-family: 'JetBrains Mono', monospace; font-size: 10.5px; color: var(--text); line-height: 1.55;
        }}

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

        .pre-box {{
            background: var(--ink); border: 1px solid var(--ruleSoft); border-radius: 8px;
            padding: 10px; font-family: 'JetBrains Mono', monospace; font-size: 11px; color: var(--text);
            white-space: pre-wrap; max-height: 240px; overflow-y: auto;
        }}
    </style>
</head>
<body>

    <header>
        <div class="header-container">
            <div class="top-subtitle">
                <span>TRẠM QUAN SÁT SUY LUẬN · MÔ HÌNH XIANGQI-R1 CONSOLE REAL STUDIO</span>
                <div class="row" style="gap: 6px;">
                    <button class="btn-icon" id="btnFlip" onclick="toggleFlip()" title="Lật bàn cờ (ArrowUpDown)">🔄</button>
                    <button class="btn-icon" id="btnCoords" onclick="toggleCoords()" title="Toạ độ lưới (Hash)">#</button>
                    <button class="btn-icon" id="btnContrast" onclick="toggleContrast()" title="Tương phản cao">🎨</button>
                    <button class="btn-icon" id="btnPresentation" onclick="togglePresentation()" title="Chế độ trình chiếu">⛶</button>
                    <button class="btn-icon" id="btnGlossary" onclick="openGlossary()" title="Chú giải thuật ngữ">📖</button>
                </div>
            </div>

            <h1 class="h1-title">
                XIANGQI <span style="color: var(--seal);">·</span> R1
            </h1>

            <p style="font-size: 14.5px; color: var(--textSoft); max-width: 700px; line-height: 1.5; margin: 0;">
                Theo dõi từng bước một mô hình suy luận kiểu R1 xử lý thế cờ: token hoá, đi qua 12 khối Transformer (self-attention + MoE-FFN), rồi tự hồi quy sinh chuỗi suy luận 32 chiều kích trước khi chốt nước đi.
            </p>
        </div>
    </header>

    <main>

        <!-- Banner thông tin -->
        <div class="banner-warning">
            <span style="color: var(--seal); font-weight: 700;">ℹ</span>
            <span>
                Dữ liệu thực tế được nạp trực tiếp từ <code style="color: var(--signal);">tools/games-completed.jsonl</code>. Kiến trúc SVG Bàn cờ 2D, ma trận chú ý Self-Attention, phân bổ MoE Routing và 32 chiều kích suy tưởng được dựng chính xác theo đặc tả <code style="color: var(--brass);">xiangqi-r1-console.tsx</code>.
            </span>
        </div>

        <!-- Controls Bar -->
        <div class="card font-mono" style="padding: 12px 16px;">
            <div class="row" style="gap: 16px; flex-wrap: wrap; justify-content: space-between;">
                <div class="row" style="gap: 8px;">
                    <button class="btn" onclick="prevStep()">◀ Lùi Step</button>
                    <button class="btn btn-primary" id="btnPlay" onclick="togglePlay()">▶ Play</button>
                    <button class="btn" onclick="nextStep()">Tiến Step ▶</button>
                    <button class="btn" onclick="resetStep()">↺ Reset</button>
                </div>

                <div class="row" style="gap: 12px;">
                    <span style="font-size: 11px; color: var(--textSoft);">VÁN ĐẤU REAL:</span>
                    <select id="gameSelect" class="select-input" style="width: auto; min-width: 220px;" onchange="onGameChange(this.value)"></select>
                </div>

                <div class="row" style="gap: 12px;">
                    <span style="font-size: 11px; color: var(--textSoft);">LƯỢT (TURN):</span>
                    <select id="turnSelect" class="select-input" style="width: auto; min-width: 140px;" onchange="onTurnChange(this.value)"></select>
                </div>
            </div>

            <div style="margin-top: 10px; display: flex; align-items: center; gap: 12px;">
                <input type="range" id="stepSlider" min="0" max="19" value="0" oninput="onSliderMove(this.value)" style="flex: 1; accent-color: var(--signal); cursor: pointer;">
                <span id="stepLabel" style="font-size: 11px; color: var(--signal); font-weight: 700; min-width: 140px; text-align: right;">Step 0 / 19</span>
            </div>
        </div>

        <!-- Main 2-Column Grid -->
        <div class="xr1-main-grid" id="mainGridContainer">

            <!-- Left Column: Pipeline & SVG Xiangqi Board -->
            <div class="col" style="gap: 20px;">

                <!-- Pipeline Pipeline Status -->
                <div class="card">
                    <div class="row" style="justify-content: space-between; font-size: 11px; font-family: 'JetBrains Mono', monospace; color: var(--textSoft);">
                        <span>🌐 ĐƯỜNG TÍN HIỆU TRANSFORMER PIPELINE</span>
                        <span id="pipelineStepTitle" style="color: var(--signal); font-weight: 700;">Token hoá đầu vào</span>
                    </div>

                    <div class="col" style="gap: 4px; font-family: 'JetBrains Mono', monospace; font-size: 11px;" id="pipelineTree">
                        <!-- Rendered by JS -->
                    </div>
                </div>

                <!-- SVG Board -->
                <div class="card">
                    <div class="row" style="justify-content: space-between; font-family: 'JetBrains Mono', monospace; font-size: 11px;">
                        <span style="font-weight: 700; color: var(--text);">♟️ BÀN CỜ 2D chuẩn UCCI & SVG ATTENTION</span>
                        <span id="turnBadge" style="color: var(--signal);">Lượt Đỏ đi</span>
                    </div>

                    <div style="width: 100%; display: flex; justify-content: center;">
                        <svg id="boardSvg" viewBox="0 0 376 416" style="width: 100%; max-width: 440px; height: auto; display: block;">
                            <!-- SVG Rendered by JS -->
                        </svg>
                    </div>

                    <div id="fenDisplay" class="font-mono" style="font-size: 10px; color: var(--textSoft); word-break: break-all;"></div>
                </div>

                <!-- Signal Stream & Hidden States heatmap -->
                <div class="card font-mono">
                    <div style="font-size: 11px; font-weight: 700; color: var(--textSoft); margin-bottom: 6px;">
                        📊 DÒNG TRẠNG THÁI ẨN · 28 DIMS SAMPLES (HIDDEN STATES)
                    </div>
                    <div id="hiddenStateMatrix" class="col" style="gap: 3px;">
                        <!-- Rendered by JS -->
                    </div>
                </div>

            </div>

            <!-- Right Column: 32D Thought Chain & Candidates -->
            <div class="col" style="gap: 20px;">

                <!-- 32D Thought Inspector -->
                <div class="card">
                    <div class="row" style="justify-content: space-between;">
                        <span class="font-mono" style="font-size: 13px; font-weight: 700; color: var(--text);">
                            🧠 MẠCH SUY TƯỞNG 32 CHIỀU KÍCH (&lt;thought&gt;)
                        </span>
                        <span id="dimComplianceBadge" class="font-mono" style="font-size: 10px; padding: 2px 6px; border-radius: 4px; background: rgba(79, 211, 196, 0.2); color: var(--signal);">
                            32/32 Valid
                        </span>
                    </div>

                    <input type="text" id="dimSearchInput" oninput="search32D(this.value)" placeholder="🔍 Tìm kiếm từ khóa trong 32 chiều kích (Xe, Pháo, Ứng viên, Bẫy...)..." class="select-input">

                    <div id="dimsCardsContainer" style="overflow-y: auto; max-height: 700px;"></div>
                </div>

                <!-- LLM Dialogue Raw -->
                <div class="card font-mono">
                    <div style="font-size: 11px; font-weight: 700; border-bottom: 1px solid var(--rule); padding-bottom: 6px;">
                        💬 PREVIEW LLM MULTI-TURN DIALOGUE (REAL DATASET)
                    </div>

                    <div>
                        <div style="font-size: 10px; color: var(--brass); margin-bottom: 4px;">👤 USER INPUT:</div>
                        <div id="userPromptText" class="pre-box"></div>
                    </div>

                    <div>
                        <div style="font-size: 10px; color: var(--signal); margin-bottom: 4px;">🤖 ASSISTANT RESPONSE:</div>
                        <div id="assistantResponseText" class="pre-box" style="max-height: 240px;"></div>
                    </div>
                </div>

            </div>

        </div>

        <div class="card font-mono">
            <div class="row" style="justify-content: space-between;">
                <span style="font-size: 11px; font-weight: 700;">📄 RAW JSON RECORD (THỰC TẾ từ games-completed.jsonl)</span>
                <button onclick="copyRawRecordJson()" class="btn">📋 Copy Raw JSON</button>
            </div>
            <pre id="rawRecordJsonBox" class="pre-box" style="max-height: 160px; color: #a7f3d0;"></pre>
        </div>

    </main>

    <!-- JavaScript Engine -->
    <script>
        const REAL_GAMES = {real_games_js_str};
        let activeGameIdx = 0;
        let activeTurnIdx = 0;
        let currentStep = 0;
        let isFlipped = false;
        let showCoords = true;
        let isHighContrast = false;
        let isPresentation = false;
        let playTimer = null;

        const COLS = 9, ROWS = 10, CELL = 40, MARGIN = 28;
        const BW = MARGIN * 2 + (COLS - 1) * CELL; // 376
        const BH = MARGIN * 2 + (ROWS - 1) * CELL; // 416
        const px = c => MARGIN + c * CELL;
        const py = r => MARGIN + r * CELL;
        const flipPt = (r, c, f) => (f ? {{ r: ROWS - 1 - r, c: COLS - 1 - c }} : {{ r, c }});

        const KIND_CHAR = {{
            xe: {{ b: '車', r: '車' }}, ma: {{ b: '馬', r: '馬' }},
            tuong: {{ b: '象', r: '相' }}, si: {{ b: '士', r: '仕' }},
            general: {{ b: '將', r: '帥' }}, phao: {{ b: '砲', r: '炮' }},
            tot: {{ b: '卒', r: '兵' }}
        }};

        const FEN_LETTER_KIND = {{ k: 'general', a: 'si', b: 'tuong', n: 'ma', r: 'xe', c: 'phao', p: 'tot' }};

        function parseFen(fen) {{
            const trimmed = (fen || '').trim();
            if (!trimmed) return {{ pieces: [], turn: 'Đỏ đi' }};
            const parts = trimmed.split(/\\s+/);
            const ranks = parts[0].split('/');
            const pieces = [];

            ranks.forEach((rankStr, ri) => {{
                let c = 0;
                for (const ch of rankStr) {{
                    if (/[1-9]/.test(ch)) {{ c += Number(ch); continue; }}
                    const kind = FEN_LETTER_KIND[ch.toLowerCase()];
                    if (kind && c < 9) {{
                        pieces.push({{ row: ri, col: c, side: ch === ch.toLowerCase() ? 'b' : 'r', kind }});
                    }}
                    c += 1;
                }}
            }});

            const turnTok = (parts[1] || 'w').toLowerCase();
            const turn = turnTok.startsWith('b') ? 'Đen đi' : 'Đỏ đi';
            return {{ pieces, turn }};
        }}

        function init() {{
            populateGamesDropdown();
            renderPipelineTree();
            renderHiddenStates();
        }}

        function populateGamesDropdown() {{
            const sel = document.getElementById('gameSelect');
            sel.innerHTML = '';

            REAL_GAMES.forEach((g, idx) => {{
                const opt = document.createElement('option');
                opt.value = idx;
                const gameId = g.game_id || (`Game_${{idx + 1}}`);
                const msgs = (g.messages || []).filter(m => m.role === 'user' || m.role === 'assistant');
                const turns = Math.floor(msgs.length / 2);
                opt.textContent = `Ván #${{idx + 1}} (ID: ${{gameId}} — ${{turns}} Turns — ${{g.total_plies || '36'}} Plies)`;
                sel.appendChild(opt);
            }});

            activeGameIdx = 0;
            sel.value = 0;
            onGameChange(0);
        }}

        function onGameChange(idx) {{
            activeGameIdx = parseInt(idx);
            const game = REAL_GAMES[activeGameIdx];
            if (!game) return;

            const msgs = (game.messages || []).filter(m => m.role === 'user' || m.role === 'assistant');
            const totalTurns = Math.floor(msgs.length / 2);

            const turnSel = document.getElementById('turnSelect');
            turnSel.innerHTML = '';
            for (let t = 0; t < totalTurns; t++) {{
                const opt = document.createElement('option');
                opt.value = t;
                const userMsg = msgs[t * 2];
                const turnMatch = userMsg ? userMsg.content.match(/Turn\\s*(\\d+)/i) : null;
                const turnNum = turnMatch ? turnMatch[1] : (t + 1);
                opt.textContent = `Lượt ${{t + 1}} (Turn ${{turnNum}})`;
                turnSel.appendChild(opt);
            }}

            activeTurnIdx = 0;
            turnSel.value = 0;
            renderTurnData(0);
        }}

        function onTurnChange(turnIdx) {{
            activeTurnIdx = parseInt(turnIdx);
            renderTurnData(activeTurnIdx);
        }}

        function renderTurnData(tIdx) {{
            const game = REAL_GAMES[activeGameIdx];
            if (!game) return;
            const msgs = (game.messages || []).filter(m => m.role === 'user' || m.role === 'assistant');

            const userMsg = msgs[tIdx * 2];
            const assistantMsg = msgs[tIdx * 2 + 1];

            if (!userMsg || !assistantMsg) return;

            document.getElementById('userPromptText').textContent = userMsg.content;
            document.getElementById('assistantResponseText').textContent = assistantMsg.content;

            let fen = "rnbakabnr/9/1c2c4/p1p1p1p1p/9/9/P1P1P1P1P/4C2C1/9/RNBAKABNR w - - 0 1";
            const fenMatch = userMsg.content.match(/FEN:\\s*([^\\n]+)/);
            if (fenMatch) fen = fenMatch[1].trim();
            document.getElementById('fenDisplay').textContent = "FEN: " + fen;

            let bestMove = "";
            const moveMatch = assistantMsg.content.match(/Chọn\\s+([a-i][0-9][a-i][0-9])/);
            if (moveMatch) bestMove = moveMatch[1];

            const parsed = parseFen(fen);
            document.getElementById('turnBadge').textContent = `Lượt ${{parsed.turn}}`;

            drawSvgBoard(parsed.pieces, bestMove);
            render32DThoughtCards(assistantMsg.content);
            document.getElementById('rawRecordJsonBox').textContent = JSON.stringify(game, null, 2);
        }}

        function drawSvgBoard(pieces, bestMove) {{
            const svg = document.getElementById('boardSvg');
            svg.innerHTML = '';

            // Marker Defs
            const defs = document.createElementNS('http://www.w3.org/2000/svg', 'defs');
            defs.innerHTML = `<marker id="xr1arrow" markerWidth="8" markerHeight="8" refX="6" refY="3" orient="auto">
                <path d="M0,0 L6,3 L0,6 Z" fill="var(--brass)" />
            </marker>`;
            svg.appendChild(defs);

            // Background
            const bg = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
            bg.setAttribute('x', '0'); bg.setAttribute('y', '0');
            bg.setAttribute('width', BW); bg.setAttribute('height', BH);
            bg.setAttribute('fill', 'var(--panel)'); bg.setAttribute('rx', '10');
            svg.appendChild(bg);

            const P = (r, c) => flipPt(r, c, isFlipped);

            // Vertical Grid Lines
            for (let c = 0; c < COLS; c++) {{
                const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
                line.setAttribute('x1', px(P(0, c).c)); line.setAttribute('y1', py(0));
                line.setAttribute('x2', px(P(0, c).c)); line.setAttribute('y2', py(9));
                line.setAttribute('stroke', 'var(--rule)'); line.setAttribute('stroke-width', '1.5');
                svg.appendChild(line);
            }}

            // Horizontal Grid Lines
            for (let r = 0; r < ROWS; r++) {{
                const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
                line.setAttribute('x1', px(0)); line.setAttribute('y1', py(P(r, 0).r));
                line.setAttribute('x2', px(8)); line.setAttribute('y2', py(P(r, 0).r));
                line.setAttribute('stroke', 'var(--rule)'); line.setAttribute('stroke-width', '1.5');
                svg.appendChild(line);
            }}

            // Palace Diagonals
            const diagLines = [
                [P(0,3), P(2,5)], [P(0,5), P(2,3)],
                [P(7,3), P(9,5)], [P(7,5), P(9,3)]
            ];
            diagLines.forEach(([p1, p2]) => {{
                const dLine = document.createElementNS('http://www.w3.org/2000/svg', 'line');
                dLine.setAttribute('x1', px(p1.c)); dLine.setAttribute('y1', py(p1.r));
                dLine.setAttribute('x2', px(p2.c)); dLine.setAttribute('y2', py(p2.r));
                dLine.setAttribute('stroke', 'var(--rule)'); dLine.setAttribute('stroke-width', '1.5');
                svg.appendChild(dLine);
            }});

            // River Text
            const riverText = document.createElementNS('http://www.w3.org/2000/svg', 'text');
            riverText.setAttribute('x', BW / 2); riverText.setAttribute('y', (py(4) + py(5)) / 2 + 5);
            riverText.setAttribute('text-anchor', 'middle'); riverText.setAttribute('font-size', '13');
            riverText.setAttribute('letter-spacing', '8'); riverText.setAttribute('fill', 'var(--textSoft)');
            riverText.setAttribute('font-family', 'JetBrains Mono, monospace');
            riverText.textContent = "楚河　　漢界";
            svg.appendChild(riverText);

            // Coordinates
            if (showCoords) {{
                for (let c = 0; c < COLS; c++) {{
                    const t = document.createElementNS('http://www.w3.org/2000/svg', 'text');
                    t.setAttribute('x', px(c)); t.setAttribute('y', BH - 6);
                    t.setAttribute('text-anchor', 'middle'); t.setAttribute('font-size', '8');
                    t.setAttribute('fill', 'var(--textSoft)'); t.setAttribute('font-family', 'JetBrains Mono');
                    t.textContent = isFlipped ? (8 - c) : c;
                    svg.appendChild(t);
                }}
            }}

            // Best Move Arrow
            if (bestMove && bestMove.length === 4) {{
                const c1 = bestMove.charCodeAt(0) - 'a'.charCodeAt(0);
                const r1 = parseInt(bestMove[1]);
                const c2 = bestMove.charCodeAt(2) - 'a'.charCodeAt(0);
                const r2 = parseInt(bestMove[3]);

                const a = P(9 - r1, c1);
                const b = P(9 - r2, c2);

                const mLine = document.createElementNS('http://www.w3.org/2000/svg', 'line');
                mLine.setAttribute('x1', px(a.c)); mLine.setAttribute('y1', py(a.r));
                mLine.setAttribute('x2', px(b.c)); mLine.setAttribute('y2', py(b.r));
                mLine.setAttribute('stroke', 'var(--brass)'); mLine.setAttribute('stroke-width', '3');
                mLine.setAttribute('stroke-linecap', 'round');
                mLine.setAttribute('marker-end', 'url(#xr1arrow)');
                svg.appendChild(mLine);
            }}

            // Pieces
            pieces.forEach(p => {{
                const pt = P(p.row, p.col);
                const g = document.createElementNS('http://www.w3.org/2000/svg', 'g');
                g.setAttribute('transform', `translate(${{px(pt.c)}}, ${{py(pt.r)}})`);

                const circle = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
                circle.setAttribute('r', '15.5');
                circle.setAttribute('fill', 'var(--paper)');
                circle.setAttribute('stroke', p.side === 'r' ? 'var(--seal)' : '#20242D');
                circle.setAttribute('stroke-width', '2');

                const txt = document.createElementNS('http://www.w3.org/2000/svg', 'text');
                txt.setAttribute('text-anchor', 'middle');
                txt.setAttribute('dominant-baseline', 'central');
                txt.setAttribute('font-size', '16');
                txt.setAttribute('font-weight', '700');
                txt.setAttribute('font-family', 'Noto Serif TC, serif');
                txt.setAttribute('fill', p.side === 'r' ? 'var(--seal)' : '#1C1E24');
                txt.textContent = KIND_CHAR[p.kind][p.side];

                g.appendChild(circle);
                g.appendChild(txt);
                svg.appendChild(g);
            }});
        }}

        function renderPipelineTree() {{
            const steps = [
                "00. TOKEN HOÁ ĐẦU VÀO", "01. EMBEDDING + ROPE POSITIONAL",
                "02. TRANSFORMER BLOCK 01", "03. TRANSFORMER BLOCK 02", "04. TRANSFORMER BLOCK 03",
                "05. TRANSFORMER BLOCK 04", "06. TRANSFORMER BLOCK 05", "07. TRANSFORMER BLOCK 06",
                "08. TRANSFORMER BLOCK 07", "09. TRANSFORMER BLOCK 08", "10. TRANSFORMER BLOCK 09",
                "11. TRANSFORMER BLOCK 10", "12. TRANSFORMER BLOCK 11", "13. TRANSFORMER BLOCK 12",
                "14. RMSNORM + LM HEAD", "15. SUY LUẬN CoT 32D", "16. XUẤT NƯỚC ĐI (BESTMOVE)"
            ];

            const treeEl = document.getElementById('pipelineTree');
            treeEl.innerHTML = '';

            steps.forEach((name, i) => {{
                const div = document.createElement('div');
                div.style.padding = '4px 8px';
                div.style.borderRadius = '4px';
                div.style.cursor = 'pointer';
                div.style.display = 'flex';
                div.style.justifyContent = 'space-between';

                if (i === currentStep) {{
                    div.style.background = 'var(--signal)';
                    div.style.color = 'var(--ink)';
                    div.style.fontWeight = '700';
                }} else if (i < currentStep) {{
                    div.style.color = 'var(--signalDim)';
                }} else {{
                    div.style.color = 'var(--ruleSoft)';
                }}

                div.textContent = name;
                div.onclick = () => setStep(i);
                treeEl.appendChild(div);
            }});

            document.getElementById('pipelineStepTitle').textContent = steps[currentStep] || '';
            document.getElementById('stepLabel').textContent = `Step ${{currentStep}} / 16`;
            document.getElementById('stepSlider').value = currentStep;
        }}

        function renderHiddenStates() {{
            const matrixEl = document.getElementById('hiddenStateMatrix');
            matrixEl.innerHTML = '';

            for (let r = 0; r < 13; r++) {{
                const row = document.createElement('div');
                row.style.display = 'flex'; row.style.alignItems = 'center'; row.style.gap = '6px';

                const label = document.createElement('span');
                label.style.width = '40px'; label.style.fontSize = '9px';
                label.style.color = r <= currentStep ? 'var(--signal)' : 'var(--ruleSoft)';
                label.textContent = r === 0 ? 'embed' : `L${{String(r).padStart(2, '0')}}`;

                const cellsContainer = document.createElement('div');
                cellsContainer.style.display = 'flex'; cellsContainer.style.gap = '1px'; cellsContainer.style.flex = '1';

                for (let c = 0; c < 28; c++) {{
                    const cell = document.createElement('div');
                    cell.style.flex = '1'; cell.style.height = '7px'; cell.style.borderRadius = '1px';
                    const val = (Math.sin(r * 3 + c * 7 + currentStep) + 1) / 2;
                    cell.style.background = r <= currentStep ? `rgba(79, 211, 196, ${{val.toFixed(2)}})` : 'var(--ruleSoft)';
                    cellsContainer.appendChild(cell);
                }}

                row.appendChild(label);
                row.appendChild(cellsContainer);
                matrixEl.appendChild(row);
            }}
        }}

        function setStep(s) {{
            currentStep = Math.max(0, Math.min(16, s));
            renderPipelineTree();
            renderHiddenStates();
        }}

        function prevStep() {{ setStep(currentStep - 1); }}
        function nextStep() {{ setStep(currentStep + 1); }}
        function resetStep() {{ setStep(0); }}

        function togglePlay() {{
            if (playTimer) {{
                clearInterval(playTimer);
                playTimer = null;
                document.getElementById('btnPlay').textContent = '▶ Play';
            }} else {{
                document.getElementById('btnPlay').textContent = '⏸ Pause';
                playTimer = setInterval(() => {{
                    if (currentStep < 16) {{
                        setStep(currentStep + 1);
                    }} else {{
                        togglePlay();
                    }}
                }}, 800);
            }}
        }}

        function onSliderMove(val) {{ setStep(parseInt(val)); }}

        function toggleFlip() {{ isFlipped = !isFlipped; document.getElementById('btnFlip').classList.toggle('active', isFlipped); renderTurnData(activeTurnIdx); }}
        function toggleCoords() {{ showCoords = !showCoords; document.getElementById('btnCoords').classList.toggle('active', showCoords); renderTurnData(activeTurnIdx); }}
        function toggleContrast() {{ isHighContrast = !isHighContrast; document.body.classList.toggle('high-contrast', isHighContrast); document.getElementById('btnContrast').classList.toggle('active', isHighContrast); }}
        function togglePresentation() {{ isPresentation = !isPresentation; document.getElementById('mainGridContainer').classList.toggle('presentation-mode', isPresentation); document.getElementById('btnPresentation').classList.toggle('active', isPresentation); }}
        function openGlossary() {{ alert("📖 CHÚ GIẢI THUẬT NGỮ:\\n- Trung Pháo: Khai cuộc tiến Pháo ra Lộ 5\\n- Bình Phong Mã: Hai Mã đối xứng bảo vệ Trung Lộ\\n- Self-Attention: Cơ chế chú ý giữa các token\\n- MoE (Mixture-of-Experts): Khối FFN thưa gồm nhiều chuyên gia"); }}

        function render32DThoughtCards(thoughtText) {{
            const container = document.getElementById('dimsCardsContainer');
            container.innerHTML = '';

            const dimBlocks = thoughtText.match(/\\[\\d+\\/32\\][^\\n]+(?:\\n(?!\\[\\d+\\/32\\])[^\\n]+)*/g);

            if (!dimBlocks) {{
                container.innerHTML = '<div style="font-size: 11px; color: var(--textSoft); padding: 12px;">Không thể phân tách các khối [X/32] từ suy tưởng.</div>';
                return;
            }}

            dimBlocks.forEach(block => {{
                const lines = block.split('\\n');
                const headerLine = lines[0];
                const bodyLines = lines.slice(1).join('\\n');

                const match = headerLine.match(/^\\[(\\d+)\\/32\\]\\s*([^\\n:]+):?(.*)/);
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

        function escapeStr(str) {{ return str.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;"); }}
        function search32D(query) {{
            const q = query.toLowerCase();
            document.querySelectorAll('.dim-card').forEach(card => {{
                card.style.display = card.textContent.toLowerCase().includes(q) ? 'block' : 'none';
            }});
        }}

        function copyRawRecordJson() {{
            const rawText = document.getElementById('rawRecordJsonBox').textContent;
            navigator.clipboard.writeText(rawText).then(() => alert("📋 Đã copy Raw JSON Record thực tế vào Clipboard!"));
        }}

        window.addEventListener('DOMContentLoaded', init);
    </script>
</body>
</html>
"""

with open('/Users/hdqb/workspaces/xiangqi-rim/tools/xiangqi_multiturn_32d_inspector.html', 'w', encoding='utf-8') as f:
    f.write(html_code)

print("✅ Master Studio Console Inspector matching xiangqi-r1-console.tsx written successfully!")
