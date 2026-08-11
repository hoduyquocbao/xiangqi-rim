import os

html_content = '''<!DOCTYPE html>
<html lang="vi">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Xiangqi-R1 Multi-Turn 32D Dataset Inspector & Markdown Previewer</title>
    <!-- Google Fonts & TailwindCSS CDN -->
    <script src="https://cdn.tailwindcss.com"></script>
    <link href="https://fonts.googleapis.com/css2?family=Fira+Code:wght@400;500;600;700&family=Inter:wght@300;400;500;600;700;800&family=Noto+Serif+TC:wght@600;700;900&display=swap" rel="stylesheet">
    <script src="https://cdn.jsdelivr.net/npm/marked/marked.min.js"></script>
    <style>
        body { font-family: 'Inter', sans-serif; }
        .font-mono { font-family: 'Fira Code', monospace; }
        .font-xiangqi { font-family: 'Noto Serif TC', serif; }

        /* Custom Scrollbars */
        ::-webkit-scrollbar { width: 8px; height: 8px; }
        ::-webkit-scrollbar-track { background: #0f172a; }
        ::-webkit-scrollbar-thumb { background: #334155; border-radius: 4px; }
        ::-webkit-scrollbar-thumb:hover { background: #475569; }

        /* Xiangqi Board CSS Grid */
        .xq-grid {
            display: grid;
            grid-template-columns: repeat(9, 1fr);
            grid-template-rows: repeat(10, 1fr);
            aspect-ratio: 9 / 10;
            background: #1e293b;
            border: 2px solid #475569;
            box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.5), 0 8px 10px -6px rgba(0, 0, 0, 0.5);
            position: relative;
        }

        .xq-cell {
            border: 1px solid #334155;
            display: flex;
            align-items: center;
            justify-content: center;
            position: relative;
            user-select: none;
        }

        .piece {
            width: 86%;
            height: 86%;
            border-radius: 50%;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 1.25rem;
            font-weight: 900;
            box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.4), inset 0 2px 4px rgba(255, 255, 255, 0.2);
            transition: all 0.2s ease;
            z-index: 10;
        }

        .piece-red {
            background: linear-gradient(135deg, #ef4444 0%, #b91c1c 100%);
            color: #fef2f2;
            border: 2px solid #fca5a5;
        }

        .piece-black {
            background: linear-gradient(135deg, #334155 0%, #0f172a 100%);
            color: #38bdf8;
            border: 2px solid #7dd3fc;
        }

        .piece-move-src { ring: 3px solid #eab308; }
        .piece-move-dst { ring: 3px solid #22c55e; animation: pulse 1.5s infinite; }

        .river-row {
            grid-column: 1 / span 9;
            background: rgba(14, 165, 233, 0.08);
            border-top: 2px solid #0284c7;
            border-bottom: 2px solid #0284c7;
            display: flex;
            align-items: center;
            justify-content: space-around;
            color: #38bdf8;
            font-weight: 700;
            letter-spacing: 0.2em;
            font-size: 0.9rem;
        }

        .dim-card {
            background: rgba(30, 41, 59, 0.7);
            backdrop-filter: blur(12px);
            border: 1px solid rgba(255, 255, 255, 0.08);
            border-left-width: 4px;
            transition: all 0.2s ease;
        }

        .dim-card:hover {
            transform: translateY(-1px);
            box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.3);
        }

        /* Dimension Group Left Border Colors */
        .dim-grp-1 { border-left-color: #10b981; } /* Nhận thức bàn cờ */
        .dim-grp-2 { border-left-color: #ef4444; } /* Phân tích đe dọa */
        .dim-grp-3 { border-left-color: #f59e0b; } /* Chiến thuật & bẫy */
        .dim-grp-4 { border-left-color: #3b82f6; } /* 36 Kế & thế trận */
        .dim-grp-5 { border-left-color: #a855f7; } /* Đánh giá & quyết định */
        .dim-grp-6 { border-left-color: #ec4899; } /* Luật đấu & phản đòn */
    </style>
</head>
<body class="bg-slate-950 text-slate-100 min-h-screen flex flex-col">

    <!-- Header / Navbar -->
    <header class="border-b border-slate-800 bg-slate-900/80 backdrop-blur sticky top-0 z-50">
        <div class="max-w-7xl mx-auto px-4 py-3 flex flex-wrap items-center justify-between gap-4">
            <div class="flex items-center space-x-3">
                <div class="w-10 h-10 rounded-xl bg-gradient-to-tr from-amber-500 to-red-600 flex items-center justify-center text-white text-xl font-black shadow-lg shadow-red-950/50 font-xiangqi">
                    帥
                </div>
                <div>
                    <h1 class="text-lg font-bold bg-gradient-to-r from-amber-200 via-emerald-300 to-cyan-300 bg-clip-text text-transparent">
                        Xiangqi-R1 32D Multi-Turn Dataset Inspector
                    </h1>
                    <p class="text-xs text-slate-4.00 flex items-center gap-2">
                        <span>v17.5 JRCP 5.0 Standard</span>
                        <span class="inline-block w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                        <span class="text-emerald-400 font-mono">100% Physical Xiangqi Rules</span>
                    </p>
                </div>
            </div>

            <!-- Controls & Actions -->
            <div class="flex items-center space-x-3">
                <button onclick="loadSampleData()" class="px-3 py-1.5 text-xs font-semibold rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700 transition flex items-center gap-1.5">
                    <span>⚡ Load Sample 20-Turn Game</span>
                </button>
                <label class="px-3 py-1.5 text-xs font-semibold rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white shadow-lg shadow-emerald-950/30 cursor-pointer transition flex items-center gap-1.5">
                    <span>📂 Open JSONL File</span>
                    <input type="file" id="fileInput" accept=".jsonl,.json,.txt" class="hidden" onchange="handleFileSelect(event)">
                </label>
            </div>
        </div>
    </header>

    <!-- Main Container -->
    <main class="flex-1 max-w-7xl w-full mx-auto p-4 space-y-6">

        <!-- Top Status Bar & Controls -->
        <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div class="bg-slate-900/60 border border-slate-800 rounded-xl p-4 flex flex-col justify-between">
                <span class="text-xs font-medium text-slate-400 uppercase tracking-wider">Game Metadata</span>
                <div class="mt-2 space-y-1 font-mono text-xs">
                    <div class="flex justify-between"><span class="text-slate-500">Game ID:</span> <span id="metaGameId" class="text-amber-400 font-bold">-</span></div>
                    <div class="flex justify-between"><span class="text-slate-500">Total Plies:</span> <span id="metaPlies" class="text-emerald-400 font-bold">-</span></div>
                    <div class="flex justify-between"><span class="text-slate-500">Outcome:</span> <span id="metaOutcome" class="text-cyan-400 font-bold">-</span></div>
                </div>
            </div>

            <!-- Turn Navigation Player -->
            <div class="md:col-span-3 bg-slate-900/60 border border-slate-800 rounded-xl p-4 flex flex-col justify-between">
                <div class="flex items-center justify-between">
                    <span class="text-xs font-medium text-slate-400 uppercase tracking-wider">Interactive Trajectory Step Player</span>
                    <span id="turnCounter" class="text-sm font-bold font-mono text-emerald-400">Turn 0 / 0</span>
                </div>
                <div class="mt-3 flex items-center space-x-3">
                    <button onclick="prevTurn()" class="px-4 py-2 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 font-semibold text-xs border border-slate-700 transition flex items-center gap-1">
                        <span>◀ Previous Turn</span>
                    </button>
                    <input type="range" id="turnSlider" min="1" max="1" value="1" oninput="onSliderChange(this.value)" class="flex-1 accent-emerald-500 h-2 bg-slate-800 rounded-lg cursor-pointer">
                    <button onclick="nextTurn()" class="px-4 py-2 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white font-semibold text-xs shadow-md shadow-emerald-950/50 transition flex items-center gap-1">
                        <span>Next Turn ▶</span>
                    </button>
                </div>
            </div>
        </div>

        <!-- Workspace Layout: Left 2D Board & Markdown, Right 32D Inspector -->
        <div class="grid grid-cols-1 lg:grid-cols-12 gap-6">

            <!-- Left Column: 2D Board & Dialogue Preview (5 cols) -->
            <div class="lg:col-span-5 space-y-6">
                <!-- Graphical 2D Xiangqi Board Card -->
                <div class="bg-slate-900/60 border border-slate-800 rounded-2xl p-5 shadow-xl">
                    <div class="flex items-center justify-between mb-4">
                        <h2 class="text-sm font-bold text-slate-200 flex items-center gap-2">
                            <span>♟️ Bàn Cờ 2D Trực Quan</span>
                            <span id="activeTurnSide" class="text-xs px-2 py-0.5 rounded-full bg-red-950/60 border border-red-500/30 text-red-400 font-medium">Lượt Đỏ</span>
                        </h2>
                        <span id="activeFenDisplay" class="text-[10px] font-mono text-slate-500 truncate max-w-[200px]" title="FEN string"></span>
                    </div>

                    <!-- Xiangqi Grid -->
                    <div id="xiangqiBoard" class="xq-grid rounded-xl overflow-hidden">
                        <!-- Cells rendered dynamically by JS -->
                    </div>
                </div>

                <!-- Markdown Conversation Preview Card -->
                <div class="bg-slate-900/60 border border-slate-800 rounded-2xl p-5 shadow-xl space-y-4">
                    <div class="flex items-center justify-between border-b border-slate-800 pb-3">
                        <h3 class="text-sm font-bold text-slate-200 flex items-center gap-2">
                            <span>💬 Preview Markdown Hội Thoại (LLM Input/Output)</span>
                        </h3>
                        <span class="text-xs text-slate-500 font-mono">Format LLM Training</span>
                    </div>

                    <!-- User Message Box -->
                    <div class="bg-slate-950/80 border border-slate-800 rounded-xl p-3.5 space-y-2">
                        <div class="flex items-center justify-between">
                            <span class="text-xs font-bold text-amber-400 font-mono">👤 USER REQUEST:</span>
                            <span class="text-[10px] text-slate-500">Input Prompt</span>
                        </div>
                        <div id="userMsgPreview" class="text-xs font-mono text-slate-300 whitespace-pre-wrap leading-relaxed"></div>
                    </div>

                    <!-- Assistant Message Box -->
                    <div class="bg-slate-950/80 border border-slate-800 rounded-xl p-3.5 space-y-2">
                        <div class="flex items-center justify-between">
                            <span class="text-xs font-bold text-emerald-400 font-mono">🤖 ASSISTANT RESPONSE:</span>
                            <span class="text-[10px] text-slate-500">Output Thought + Move</span>
                        </div>
                        <div id="assistantMsgPreview" class="text-xs font-mono text-slate-300 whitespace-pre-wrap leading-relaxed max-h-60 overflow-y-auto pr-2"></div>
                    </div>
                </div>
            </div>

            <!-- Right Column: Full 32-Dimensional Thought Breakdown (7 cols) -->
            <div class="lg:col-span-7 space-y-6">
                <div class="bg-slate-900/60 border border-slate-800 rounded-2xl p-5 shadow-xl flex flex-col h-full">
                    
                    <!-- Title & Filter Tabs -->
                    <div class="flex flex-wrap items-center justify-between gap-3 border-b border-slate-800 pb-4 mb-4">
                        <div>
                            <h2 class="text-base font-bold text-slate-100 flex items-center gap-2">
                                <span>🧠 Mạch Suy Tưởng 32 Chiều Kích (<thought> Block)</span>
                            </h2>
                            <p class="text-xs text-slate-400 mt-0.5">Phân tích chi tiết 32 chiều kích dưới bối cảnh JRCP 5.0</p>
                        </div>

                        <!-- Filter Chips -->
                        <div class="flex flex-wrap gap-1.5 text-xs">
                            <button onclick="filterDims('all')" class="dim-filter-btn active px-2.5 py-1 rounded-md bg-emerald-900/40 text-emerald-300 border border-emerald-500/40 font-medium">Tất cả (32)</button>
                            <button onclick="filterDims('grp1')" class="dim-filter-btn px-2.5 py-1 rounded-md bg-slate-800 text-slate-300 hover:bg-slate-700">Nhận thức (1-6)</button>
                            <button onclick="filterDims('grp2')" class="dim-filter-btn px-2.5 py-1 rounded-md bg-slate-800 text-slate-300 hover:bg-slate-700">Đe dọa (7-12)</button>
                            <button onclick="filterDims('grp3')" class="dim-filter-btn px-2.5 py-1 rounded-md bg-slate-800 text-slate-300 hover:bg-slate-700">Chiến thuật (13-18)</button>
                            <button onclick="filterDims('grp4')" class="dim-filter-btn px-2.5 py-1 rounded-md bg-slate-800 text-slate-300 hover:bg-slate-700">Binh pháp (19-22)</button>
                        </div>
                    </div>

                    <!-- Search Input -->
                    <div class="mb-4">
                        <input type="text" id="dimSearch" oninput="searchDimensions(this.value)" placeholder="🔍 Tìm kiếm từ khóa trong 32 chiều kích (ví dụ: 'Pháo', 'Sĩ', 'Phong thủ', 'Candidates')..." class="w-full bg-slate-950 border border-slate-800 rounded-lg px-3.5 py-2 text-xs text-slate-200 placeholder-slate-500 focus:outline-none focus:border-emerald-500 transition">
                    </div>

                    <!-- 32D Cards Container -->
                    <div id="dimsContainer" class="space-y-3 overflow-y-auto max-h-[750px] pr-2">
                        <!-- Rendered by JS -->
                    </div>
                </div>
            </div>

        </div>

        <!-- Raw JSON Inspector Modal / Footer Drawer -->
        <div class="bg-slate-900/60 border border-slate-800 rounded-2xl p-5 shadow-xl space-y-3">
            <div class="flex items-center justify-between">
                <h3 class="text-sm font-bold text-slate-200 flex items-center gap-2">
                    <span>📄 Raw JSONL Conversation Object</span>
                </h3>
                <button onclick="copyRawJson()" class="text-xs px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 font-mono transition">
                    📋 Copy Raw JSON
                </button>
            </div>
            <pre id="rawJsonDisplay" class="bg-slate-950 p-4 rounded-xl font-mono text-xs text-emerald-400/90 overflow-x-auto max-h-48 border border-slate-800/80"></pre>
        </div>

    </main>

    <!-- Embedded Sample Data & JavaScript Engine -->
    <script>
        // Sample 20-ply game dataset provided by user request
        let currentRecord = null;
        let currentPlyIndex = 0; // 0-indexed conversation turn

        // Piece symbol mapping
        const PIECE_NAMES = {
            'r': { name: '車', side: 'black' }, 'n': { name: '馬', side: 'black' }, 'b': { name: '象', side: 'black' },
            'a': { name: '士', side: 'black' }, 'k': { name: '將', side: 'black' }, 'c': { name: '砲', side: 'black' },
            'p': { name: '卒', side: 'black' },
            'R': { name: '車', side: 'red' }, 'N': { name: '馬', side: 'red' }, 'B': { name: '相', side: 'red' },
            'A': { name: '仕', side: 'red' }, 'K': { name: '帥', side: 'red' }, 'C': { name: '炮', side: 'red' },
            'P': { name: '兵', side: 'red' }
        };

        const SAMPLE_GAME = {
            "messages": [
                {
                    "role": "system",
                    "content": "Bạn là Xiangqi-R1 Master v5.0 — mô hình suy luận cờ Tướng siêu việt được huấn luyện phân tích chiều sâu chiến thuật 32 chiều kích.\\nBạn phải phân tích bàn cờ qua 32 chiều kích suy tưởng <thought> chi tiết trước khi xuất kết quả JSON JRCP 5.0.\\n32 chiều kích gồm 6 nhóm: Nhận thức Bàn cờ (1-6), Phân tích Đe dọa (7-12), Chiến thuật & Bẫy (13-18), 36 Kế Binh Pháp & Thế Trận (19-22), Đánh giá & Quyết định (23-28), Luật Đấu & Phản Đòn Tối Ưu (29-32).\\nMỗi chiều kích phải cung cấp thông tin cụ thể, chi tiết đến mức agent kém thông minh nhất cũng nhìn rõ hiện trạng bàn cờ."
                },
                {
                    "role": "user",
                    "content": "Bàn cờ Turn 1:\\nFEN: rnbakabnr/9/1c2c4/p1p1p1p1p/9/9/P1P1P1P1P/4C2C1/9/RNBAKABNR w - - 0 1\\nLượt Đỏ đi."
                },
                {
                    "role": "assistant",
                    "content": "<thought>\\n[1/32] KIỂM KÊ QUÂN CỜ:\\n  Đỏ: Xe (a0), Mã (b0), Tượng (c0), Sĩ (d0), Tướng (e0), Sĩ (f0), Tượng (g0), Mã (h0), Xe (i0), Pháo (e2), Pháo (h2), Tốt (a3), Tốt (c3), Tốt (e3), Tốt (g3), Tốt (i3)\\n  Đen: Tốt (a6), Tốt (c6), Tốt (e6), Tốt (g6), Tốt (i6), Pháo (b7), Pháo (e7), Xe (a9), Mã (b9), Tượng (c9), Sĩ (d9), Tướng (e9), Sĩ (f9), Tượng (g9), Mã (h9), Xe (i9)\\n[2/32] BÀN CỜ 2D:\\n    a    b    c    d    e    f    g    h    i\\n  ┌────┬────┬────┬────┬────┬────┬────┬────┬────┐\\n9 │ 車 │ 馬 │ 象 │ 士 │ 將 │ 士 │ 象 │ 馬 │ 車 │\\n  ├────┼────┼────┼────┼────┼────┼────┼────┼────┤\\n8 │ ． │ ． │ ． │ ． │ ． │ ． │ ． │ ． │ ． │\\n  ├────┼────┼────┼────┼────┼────┼────┼────┼────┤\\n7 │ ． │ 砲 │ ． │ ． │ 砲 │ ． │ ． │ ． │ ． │\\n  ├────┼────┼────┼────┼────┼────┼────┼────┼────┤\\n6 │ 卒 │ ． │ 卒 │ ． │ 卒 │ ． │ 卒 │ ． │ 卒 │\\n  ├────┼────┼────┼────┼────┼────┼────┼────┼────┤\\n5 │ ． │ ． │ ． │ ． │ ． │ ． │ ． │ ． │ ． │\\n  ├────┼────┼────┼────┼────┼────┼────┼────┼────┤  ═══ Sông Ngân Hà ═══\\n4 │ ． │ ． │ ． │ ． │ ． │ ． │ ． │ ． │ ． │\\n  ├────┼────┼────┼────┼────┼────┼────┼────┼────┤\\n3 │ 兵 │ ． │ 兵 │ ． │ 兵 │ ． │ 兵 │ ． │ 兵 │\\n  ├────┼────┼────┼────┼────┼────┼────┼────┼────┤\\n2 │ ． │ ． │ ． │ ． │ 炮 │ ． │ ． │ 炮 │ ． │\\n  ├────┼────┼────┼────┼────┼────┼────┼────┼────┤\\n1 │ ． │ ． │ ． │ ． │ ． │ ． │ ． │ ． │ ． │\\n  ├────┼────┼────┼────┼────┼────┼────┼────┼────┤\\n0 │ 車 │ 馬 │ 相 │ 仕 │ 帥 │ 仕 │ 相 │ 馬 │ 車 │\\n  └────┴────┴────┴────┴────┴────┴────┴────┴────┘\\n    a    b    c    d    e    f    g    h    i\\n[3/32] TƯƠNG QUAN VẬT CHẤT CHI TIẾT:\\n  Đỏ: 480cp | Đen: 480cp | Chênh lệch: 0cp\\n  (Xe=90, Pháo=45, Mã=40, Sĩ=20, Tượng=20, Tốt=10, Tướng=0)\\n[4/32] PHÂN TÍCH 9 LỘ:\\n  Lộ 1 (a): KHÓA | Lộ 2 (b): MỞ (Đen chiếm) | Lộ 3 (c): KHÓA | Lộ 4 (d): MỞ | Lộ 5 (e): KHÓA | Lộ 6 (f): MỞ | Lộ 7 (g): KHÓA | Lộ 8 (h): MỞ (Đỏ chiếm) | Lộ 9 (i): KHÓA\\n[5/32] MỨC ĐỘ TRIỂN KHAI QUÂN:\\n  Đỏ: 1/16 quân đã triển khai. Chưa triển khai: Xe(a0), Xe(i0), Mã(b0), Mã(h0), Tượng(c0), Tượng(g0), Sĩ(d0), Sĩ(f0), Tướng(e0), Pháo(h2), Tốt(a3), Tốt(c3), Tốt(e3), Tốt(g3), Tốt(i3)\\n  Đen: 1/16 quân đã triển khai. Chưa triển khai: Xe(a9), Xe(i9), Mã(b9), Mã(h9), Tượng(c9), Tượng(g9), Sĩ(d9), Sĩ(f9), Tướng(e9), Pháo(b7), Tốt(a6), Tốt(c6), Tốt(e6), Tốt(g6), Tốt(i6)\\n[6/32] ĐỘ LINH HOẠT (MOBILITY):\\n  Đỏ: 36 nước đi hợp lệ | Đen: 36 nước đi hợp lệ | Chênh lệch: 0\\n[7/32] AN TOÀN TƯỚNG:\\n  Bên ta (Đỏ): Tướng Đỏ an toàn — Sĩ: 2/2, Tượng: 2/2. Cung Tướng kiên cố.\\n  Đối phương (Đen): Tướng Đen an toàn — Sĩ: 2/2, Tượng: 2/2. Cung Tướng kiên cố.\\n[8/32] QUÂN BỊ TẤN CÔNG:\\n  Bên ta: Quân Đỏ bị tấn công: Tốt(e3, 10cp) bị tấn công bởi Pháo(e7)\\n  Đối phương: Quân Đen bị tấn công: Tốt(e6, 10cp) bị tấn công bởi Pháo(e2)\\n[9/32] QUÂN TREO (HANGING — ĂN MIỄN PHÍ):\\n  Bên ta: Tốt(e3, 10cp) TREO — không có quân bảo vệ, bị Pháo(e7) nhắm tới\\n  Đối phương: Tốt(e6, 10cp) TREO — không có quân bảo vệ, bị Pháo(e2) nhắm tới\\n[10/32] QUÂN BỊ GHIM (PIN):\\n  Bên ta: Không có quân Đỏ nào bị ghim.\\n  Đối phương: Không có quân Đen nào bị ghim.\\n[11/32] ĐÒN KÉP (FORK):\\n  Không phát hiện đòn kép nào trên bàn cờ.\\n[12/32] ĐÒN MỞ (DISCOVERED ATTACK):\\n  Không phát hiện đòn mở nào có thể thực hiện ngay.\\n[13/32] BẪY ĂN QUÂN:\\n  Không phát hiện bẫy ăn quân nào cho Đỏ.\\n[14/32] CHIẾU BÍ TIỀM ẨN:\\n  Không phát hiện chiếu bí tiềm ẩn trong 1 nước.\\n[15/32] DƯƠNG ĐÔNG KÍCH TÂY:\\n  Nước đi tập trung cục bộ (cánh trung tâm), không có dấu hiệu nghi binh.\\n[16/32] MẪU CHIẾN THUẬT:\\n    Đỏ Pháo Đầu Lộ 5 — đe dọa trực tiếp trung lộ\\n    Đen Pháo Đầu Lộ 5 — kiểm soát trung tâm\\n    Đỏ Song Xe lực chiến — sức mạnh tấn công tối đa\\n    Đen Song Xe lực chiến — sức mạnh tấn công tối đa\\n[17/32] PHỐI HỢP QUÂN:\\n  Đỏ Song Xe trùng hàng 0 — kiểm soát toàn bộ hàng ngang; Đen Song Xe trùng hàng 9 — kiểm soát toàn bộ hàng ngang; Đen Mã-Pháo phối hợp gần (b9,b7) — đe dọa chiếu đôi\\n[18/32] ĐIỂM YẾU CẤU TRÚC:\\n  Bên ta: Đỏ: Tốt cô lập trên lộ a; Tốt cô lập trên lộ c; Tốt cô lập trên lộ e; Tốt cô lập trên lộ g; Tốt cô lập trên lộ i\\n  Đối phương: Đen: Tốt cô lập trên lộ a; Tốt cô lập trên lộ c; Tốt cô lập trên lộ e; Tốt cô lập trên lộ g; Tốt cô lập trên lộ i\\n[19/32] 36 KẾ BINH PHÁP ÁP DỤNG:\\n    Kế 3: Tá Đao Sát Nhân — Dùng quân đối phương làm đòn bẩy — Pháo sử dụng quân đối phương làm ngòi để tấn công\\n[20/32] THẾ TRẬN KINH ĐIỂN:\\n  Đỏ: Pháo Đầu (中炮) — Pháo chiếm Trung Lộ 5, tấn công trực diện cung Tướng đối phương; Đen: Pháo Đầu (中炮) — Pháo chiếm Trung Lộ 5, tấn công trực diện cung Tướng đối phương; Đỏ: Tam Tử Kinh (三子经) — Ba lớp phòng thủ Sĩ-Tượng bao quanh Tướng, cung Tướng kiên cố nhất; Đen: Tam Tử Kinh (三子经) — Ba lớp phòng thủ Sĩ-Tượng bao quanh Tướng, cung Tướng kiên cố nhất\\n[21/32] GIAI ĐOẠN & CHIẾN LƯỢC:\\n  Giai đoạn: Khai cuộc (nước thứ 0) — Đỏ đi.\\n[22/32] TEMPO & SÁNG KIẾN:\\n  Thế trận cân bằng — Mobility: Đỏ 36 vs Đen 36. Chưa bên nào nắm rõ sáng kiến.\\n[23/32] ƯU THẾ TỔNG HỢP:\\n  Tương quan vật chất cân bằng (chênh lệch 0cp). Đỏ: 480cp, Đen: 480cp.\\n[24/32] BẤT LỢI TỔNG HỢP:\\n  Cả hai bên duy trì thế trận giằng co — ưu thế thuộc về bên nào triển khai quân tốt hơn.\\n[25/32] ĐÁNH GIÁ CANDIDATES (36 ứng viên, hiển thị top 5):\\n    + Ứng viên 1: a0a1 — Xe(a0->a1)\\n    + Ứng viên 2: a0a2 — Xe(a0->a2)\\n    + Ứng viên 3: b0a2 — Mã(b0->a2)\\n    + Ứng viên 4: b0c2 — Mã(b0->c2)\\n    + Ứng viên 5: c0a2 — Tượng(c0->a2)\\n[26/32] SO SÁNH & CHỌN BESTMOVE:\\n  Chọn e2e6 — Pháo(e2 -> e6), ăn Tốt(e6) (0cp).\\n  Lý do: Tối ưu hóa Centipawn, vị trí quân cờ, và chiến thuật phù hợp giai đoạn Khai cuộc.\\n[27/32] CENTIPAWN TỔNG HỢP: 0cp\\n[28/32] XÁC MINH: e2e6 khớp regex ^[a-i][0-9][a-i][0-9]$ ✓ | Nước đi hợp lệ trong danh sách 36 ứng viên ✓\\n[29/32] NƯỚC PHẢN ĐÒN SẮC BÉN NHẤT CỦA ĐỐI PHƯƠNG:\\n  Nước phản đòn mạnh nhất của Đen: e7e3 (Pháo ăn Tốt(e3)) — buộc ta phải chuẩn bị phương án đối phó.\\n[30/32] GIỚI HẠN LUẬT CẤM VẬT LÝ:\\n  Hợp lệ tuyệt đối — Không vi phạm bất kỳ luật cấm vật lý nào (Chưa đủ chuỗi lặp nước).\\n[31/32] CHUỖI ĐỔI QUÂN TIỀM ẨN:\\n  Ăn quân đơn phương: Đỏ ăn Tốt (10cp) mà không bị phản đòn.\\n[32/32] TỈ LỆ THẮNG HÒA THUA TẢN CUỘC:\\n  Trạng thái trung/tàn cuộc (32 quân) — Chưa đủ điều kiện kích hoạt Tablebase 5 quân.\\n</thought>"
                }
            ],
            "game_id": "055eecf3",
            "total_plies": 20,
            "outcome": "in_progress",
            "stamp": 1786353475
        };

        function loadSampleData() {
            currentRecord = SAMPLE_GAME;
            setupGame();
        }

        function setupGame() {
            if (!currentRecord || !currentRecord.messages) return;

            // Filter out system message to get conversation pairs
            const convMessages = currentRecord.messages.filter(m => m.role === 'user' || m.role === 'assistant');
            const totalTurns = Math.floor(convMessages.length / 2);

            document.getElementById('metaGameId').textContent = currentRecord.game_id || '055eecf3';
            document.getElementById('metaPlies').textContent = (currentRecord.total_plies || convMessages.length) + ' plies';
            document.getElementById('metaOutcome').textContent = currentRecord.outcome || 'in_progress';

            const slider = document.getElementById('turnSlider');
            slider.min = 1;
            slider.max = Math.max(1, totalTurns);
            slider.value = 1;

            currentPlyIndex = 0;
            renderTurn(0);
        }

        function onSliderChange(val) {
            currentPlyIndex = parseInt(val) - 1;
            renderTurn(currentPlyIndex);
        }

        function prevTurn() {
            if (currentPlyIndex > 0) {
                currentPlyIndex--;
                document.getElementById('turnSlider').value = currentPlyIndex + 1;
                renderTurn(currentPlyIndex);
            }
        }

        function nextTurn() {
            const convMessages = currentRecord.messages.filter(m => m.role === 'user' || m.role === 'assistant');
            const totalTurns = Math.floor(convMessages.length / 2);
            if (currentPlyIndex < totalTurns - 1) {
                currentPlyIndex++;
                document.getElementById('turnSlider').value = currentPlyIndex + 1;
                renderTurn(currentPlyIndex);
            }
        }

        function renderTurn(index) {
            const convMessages = currentRecord.messages.filter(m => m.role === 'user' || m.role === 'assistant');
            const userMsg = convMessages[index * 2];
            const assistantMsg = convMessages[index * 2 + 1];

            const totalTurns = Math.floor(convMessages.length / 2);
            document.getElementById('turnCounter').textContent = `Turn ${index + 1} / ${totalTurns}`;

            if (!userMsg || !assistantMsg) return;

            // Render User & Assistant Text Preview
            document.getElementById('userMsgPreview').textContent = userMsg.content;
            document.getElementById('assistantMsgPreview').textContent = assistantMsg.content;

            // Extract FEN string
            const fenMatch = userMsg.content.match(/FEN:\s*([^\n]+)/);
            const fen = fenMatch ? fenMatch[1].strip() : "rnbakabnr/9/1c2c4/p1p1p1p1p/9/9/P1P1P1P1P/4C2C1/9/RNBAKABNR w - - 0 1";
            document.getElementById('activeFenDisplay').textContent = fen;

            const turnSideMatch = userMsg.content.match(/Lượt\s*(Đỏ|Đen)\s*đi/i);
            const turnSide = turnSideMatch ? turnSideMatch[1] : (fen.includes(' w ') ? 'Đỏ' : 'Đen');
            document.getElementById('activeTurnSide').textContent = `Lượt ${turnSide}`;
            document.getElementById('activeTurnSide').className = turnSide === 'Đỏ'
                ? "text-xs px-2 py-0.5 rounded-full bg-red-950/60 border border-red-500/30 text-red-400 font-medium"
                : "text-xs px-2 py-0.5 rounded-full bg-sky-950/60 border border-sky-500/30 text-sky-400 font-medium";

            // Extract Move from [26/32] SO SÁNH & CHỌN BESTMOVE or content
            let moveBest = "";
            const moveMatch = assistantMsg.content.match(/Chọn\s+([a-i][0-9][a-i][0-9])/);
            if (moveMatch) moveBest = moveMatch[1];

            // Render Board 2D
            renderXiangqiBoard(fen, moveBest);

            // Render 32D Cards
            render32DCards(assistantMsg.content);

            // Raw JSON update
            document.getElementById('rawJsonDisplay').textContent = JSON.stringify({
                user: userMsg,
                assistant: assistantMsg
            }, null, 2);
        }

        function renderXiangqiBoard(fen, moveBest) {
            const boardEl = document.getElementById('xiangqiBoard');
            boardEl.innerHTML = '';

            const fenParts = fen.split(' ');
            const rows = fenParts[0].split('/');

            let moveSrc = -1, moveDst = -1;
            if (moveBest && moveBest.length === 4) {
                const c1 = moveBest.charCodeAt(0) - 'a'.charCodeAt(0);
                const r1 = parseInt(moveBest[1]);
                const c2 = moveBest.charCodeAt(2) - 'a'.charCodeAt(0);
                const r2 = parseInt(moveBest[3]);
                moveSrc = (9 - r1) * 9 + c1;
                moveDst = (9 - r2) * 9 + c2;
            }

            let cellIndex = 0;
            for (let r = 0; r < 10; r++) {
                const rowStr = rows[r] || "9";
                let colIdx = 0;

                for (let i = 0; i < rowStr.length; i++) {
                    const char = rowStr[i];
                    if (!isNaN(char)) {
                        const emptyCount = parseInt(char);
                        for (let e = 0; e < emptyCount; e++) {
                            const cell = createCell(r, colIdx, null, cellIndex === moveSrc, cellIndex === moveDst);
                            boardEl.appendChild(cell);
                            colIdx++;
                            cellIndex++;
                        }
                    } else {
                        const pInfo = PIECE_NAMES[char];
                        const cell = createCell(r, colIdx, pInfo, cellIndex === moveSrc, cellIndex === moveDst);
                        boardEl.appendChild(cell);
                        colIdx++;
                        cellIndex++;
                    }
                }
            }
        }

        function createCell(r, c, pieceInfo, isSrc, isDst) {
            const cell = document.createElement('div');
            cell.className = 'xq-cell';

            if (isSrc) cell.classList.add('bg-amber-500/20');
            if (isDst) cell.classList.add('bg-emerald-500/20');

            if (pieceInfo) {
                const pEl = document.createElement('div');
                pEl.className = `piece ${pieceInfo.side === 'red' ? 'piece-red' : 'piece-black'} font-xiangqi`;
                pEl.textContent = pieceInfo.name;
                if (isSrc) pEl.classList.add('piece-move-src');
                if (isDst) pEl.classList.add('piece-move-dst');
                cell.appendChild(pEl);
            }
            return cell;
        }

        function render32DCards(thoughtText) {
            const container = document.getElementById('dimsContainer');
            container.innerHTML = '';

            const dimMatches = thoughtText.match(/\[\d+\/32\][^\n]+(?:\n(?!\[\d+\/32\])[^\n]+)*/g);

            if (!dimMatches) {
                container.innerHTML = '<div class="text-xs text-slate-500 p-4">Không thể phân tích khối &lt;thought&gt; 32D.</div>';
                return;
            }

            dimMatches.forEach((dimBlock) => {
                const headerMatch = dimBlock.match(/^\[(\d+)\/32\]\s*([^\n:]+):?(.*)/s);
                if (!headerMatch) return;

                const dimNum = parseInt(headerMatch[1]);
                const dimTitle = headerMatch[2].trim();
                const dimBody = headerMatch[3].trim();

                let grpClass = 'dim-grp-1';
                let grpTag = 'Nhận thức';
                if (dimNum >= 7 && dimNum <= 12) { grpClass = 'dim-grp-2'; grpTag = 'Đe dọa'; }
                else if (dimNum >= 13 && dimNum <= 18) { grpClass = 'dim-grp-3'; grpTag = 'Chiến thuật'; }
                else if (dimNum >= 19 && dimNum <= 22) { grpClass = 'dim-grp-4'; grpTag = 'Binh pháp'; }
                else if (dimNum >= 23 && dimNum <= 28) { grpClass = 'dim-grp-5'; grpTag = 'Quyết định'; }
                else if (dimNum >= 29) { grpClass = 'dim-grp-6'; grpTag = 'Luật đấu'; }

                const card = document.createElement('div');
                card.className = `dim-card ${grpClass} rounded-xl p-3.5 space-y-1.5`;
                card.dataset.num = dimNum;

                card.innerHTML = `
                    <div class="flex items-center justify-between">
                        <div class="flex items-center space-x-2">
                            <span class="text-xs font-bold font-mono text-emerald-400">[${dimNum}/32]</span>
                            <span class="text-xs font-bold text-slate-200">${dimTitle}</span>
                        </div>
                        <span class="text-[10px] px-2 py-0.5 rounded bg-slate-800 text-slate-400 font-medium">${grpTag}</span>
                    </div>
                    ${dimBody ? `<div class="text-xs font-mono text-slate-300/90 whitespace-pre-wrap leading-relaxed pl-2 border-l border-slate-700/50 mt-1">${escapeHtml(dimBody)}</div>` : ''}
                `;

                container.appendChild(card);
            });
        }

        function escapeHtml(str) {
            return str.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
        }

        function filterDims(type) {
            document.querySelectorAll('.dim-filter-btn').forEach(btn => {
                btn.classList.remove('bg-emerald-900/40', 'text-emerald-300', 'border-emerald-500/40');
                btn.classList.add('bg-slate-800', 'text-slate-300');
            });
            event.target.classList.add('bg-emerald-900/40', 'text-emerald-300', 'border-emerald-500/40');

            document.querySelectorAll('.dim-card').forEach(card => {
                const num = parseInt(card.dataset.num);
                if (type === 'all') card.style.display = 'block';
                else if (type === 'grp1' && num >= 1 && num <= 6) card.style.display = 'block';
                else if (type === 'grp2' && num >= 7 && num <= 12) card.style.display = 'block';
                else if (type === 'grp3' && num >= 13 && num <= 18) card.style.display = 'block';
                else if (type === 'grp4' && num >= 19 && num <= 22) card.style.display = 'block';
                else card.style.display = 'none';
            });
        }

        function searchDimensions(query) {
            const q = query.toLowerCase();
            document.querySelectorAll('.dim-card').forEach(card => {
                const text = card.textContent.toLowerCase();
                card.style.display = text.includes(q) ? 'block' : 'none';
            });
        }

        function handleFileSelect(event) {
            const file = event.target.files[0];
            if (!file) return;

            const reader = new FileReader();
            reader.onload = function(e) {
                try {
                    const text = e.target.result;
                    const firstLine = text.trim().split('\\n')[0];
                    currentRecord = JSON.parse(firstLine);
                    setupGame();
                    alert(`✅ Đã tải file: ${file.name}`);
                } catch (err) {
                    alert("❌ Lỗi định dạng JSONL: " + err.message);
                }
            };
            reader.readAsText(file);
        }

        function copyRawJson() {
            const rawText = document.getElementById('rawJsonDisplay').textContent;
            navigator.clipboard.writeText(rawText).then(() => {
                alert("📋 Đã copy Raw JSON vào Clipboard!");
            });
        }

        // Initialize on page load
        window.addEventListener('DOMContentLoaded', () => {
            loadSampleData();
        });
    </script>
</body>
</html>
'''

os.makedirs('/Users/hdqb/workspaces/xiangqi-rim/tools', exist_ok=True)
with open('/Users/hdqb/workspaces/xiangqi-rim/tools/xiangqi_multiturn_32d_inspector.html', 'w', encoding='utf-8') as f:
    f.write(html_content)

print('✅ Created /Users/hdqb/workspaces/xiangqi-rim/tools/xiangqi_multiturn_32d_inspector.html successfully!')
