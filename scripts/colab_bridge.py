#!/usr/bin/env python3
"""
Colab MCP Bridge Helper Script
Re-establishes Colab MCP WebSocket connection for target notebook files.
"""

import sys
import os
import asyncio
import logging
import webbrowser

# Add site-packages and src paths
uv_src_path = "/Users/hdqb/.cache/uv/git-v0/checkouts/79ec5ff6492e82aa/b9ab389/src"
uv_site_path = "/Users/hdqb/.cache/uv/archive-v0/fKr1k6RWh9O0MO31/lib/python3.14/site-packages"
for p in [uv_src_path, uv_site_path]:
    if os.path.exists(p) and p not in sys.path:
        sys.path.insert(0, p)

try:
    from colab_mcp.websocket_server import ColabWebSocketServer
except ImportError:
    print("Error: colab_mcp package not found in Python path.")
    sys.exit(1)

async def main():
    logging.basicConfig(level=logging.INFO)
    async with ColabWebSocketServer() as wss:
        token = wss.token
        port = wss.port
        print(f"\n==================================================", flush=True)
        print(f"  COLAB MCP WEBSOCKET PROXY READY", flush=True)
        print(f"==================================================", flush=True)
        print(f"Port : {port}", flush=True)
        print(f"Token: {token}", flush=True)
        
        # Target URL anchor
        hash_fragment = f"#mcpProxyToken={token}&mcpProxyPort={port}"
        
        target_notebook = sys.argv[1] if len(sys.argv) > 1 else "empty.ipynb"
        if target_notebook.startswith("http"):
            colab_url = f"{target_notebook}{hash_fragment}"
        else:
            colab_url = f"https://colab.research.google.com/notebooks/empty.ipynb{hash_fragment}"

        print(f"\nCONNECT_URL: {colab_url}\n", flush=True)
        webbrowser.open_new(colab_url)

        print("Waiting for Colab Frontend WebSocket connection...", flush=True)
        await wss.connection_live.wait()
        print("SUCCESS! Colab Frontend is CONNECTED to local MCP server!")
        
        # Keep alive
        while wss.connection_live.is_set():
            await asyncio.sleep(1)

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\nColab MCP Bridge stopped.")
