#!/usr/bin/env python3
"""
============================================================
XIANGQI-R1 32D INSPECTOR — PRODUCTION PYTHON DATA API & SERVER
Reads live games data from tools/games-completed.jsonl
Serves REST API at /api/games and static files with correct MIME types
============================================================
"""

import http.server
import socketserver
import json
import os
import sys
import mimetypes

# Đăng ký MIME types chuẩn xác chống lỗi MIME type "application/octet-stream"
mimetypes.init()
mimetypes.add_type('application/javascript', '.js')
mimetypes.add_type('application/javascript', '.mjs')
mimetypes.add_type('application/javascript', '.ts')
mimetypes.add_type('application/javascript', '.tsx')
mimetypes.add_type('text/css', '.css')
mimetypes.add_type('application/json', '.json')
mimetypes.add_type('image/svg+xml', '.svg')

PORT = 8080
INSPECTOR_DIR = os.path.dirname(os.path.abspath(__file__))
WORKSPACE_DIR = os.path.dirname(INSPECTOR_DIR)
GAMES_JSONL_PATH = os.path.abspath(os.path.join(WORKSPACE_DIR, 'tools', 'games-completed.jsonl'))
DIST_DIR = os.path.join(INSPECTOR_DIR, 'dist')

# Ưu tiên serve từ dist/ nếu đã build, nếu chưa build thì serve inspector/
SERVE_DIR = DIST_DIR if os.path.exists(DIST_DIR) else INSPECTOR_DIR

class InspectorHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=SERVE_DIR, **kwargs)

    def guess_type(self, path):
        # Đảm bảo trả về đúng MIME type cho .js, .mjs, .ts, .tsx, .css
        base, ext = os.path.splitext(path)
        ext = ext.lower()
        if ext in ('.js', '.mjs', '.ts', '.tsx'):
            return 'application/javascript; charset=utf-8'
        if ext == '.css':
            return 'text/css; charset=utf-8'
        if ext == '.json':
            return 'application/json; charset=utf-8'
        if ext == '.svg':
            return 'image/svg+xml'
        return super().guess_type(path)

    def do_GET(self):
        # Normalize path
        req_path = self.path.split('?')[0]

        if req_path == '/api/games':
            self.send_response(200)
            self.send_header('Content-type', 'application/json; charset=utf-8')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()

            games = []
            if os.path.exists(GAMES_JSONL_PATH):
                with open(GAMES_JSONL_PATH, 'r', encoding='utf-8') as f:
                    for line in f:
                        line = line.strip()
                        if line:
                            try:
                                games.append(json.loads(line))
                            except json.JSONDecodeError:
                                pass

            response_data = json.dumps(games, ensure_ascii=False)
            self.wfile.write(response_data.encode('utf-8'))
            return
        
        # Nếu file không tồn tại và không phải API/assets, fallback về index.html (SPA Fallback)
        translated_path = self.translate_path(self.path)
        if not os.path.exists(translated_path) and not req_path.startswith('/api/') and not req_path.startswith('/assets/'):
            self.path = '/index.html'

        return super().do_GET()

def main():
    print(f"🚀 Starting Xiangqi-R1 32D Inspector Server on http://localhost:{PORT}")
    print(f"📁 Serving static directory: {SERVE_DIR}")
    print(f"📄 Reading live data from: {GAMES_JSONL_PATH}")

    socketserver.TCPServer.allow_reuse_address = True
    with socketserver.TCPServer(("", PORT), InspectorHandler) as httpd:
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\nShutting down server.")
            httpd.server_close()

if __name__ == '__main__':
    main()
