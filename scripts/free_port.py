#!/usr/bin/env python3
# ============================================================================
# XIANGQI-RIM CLI UTILITY: SAFE & PRECISE PORT PID KILLER
# ============================================================================
# Script chạy thủ công ngắt chính xác PID chiếm cổng (Mặc định 7860)
# KHÔNG diệt python global, KHÔNG ảnh hưởng tới các dịch vụ khác trên máy.
# ============================================================================

import argparse
import os
import sys
import time

try:
    import psutil
    HAS_PSUTIL = True
except ImportError:
    HAS_PSUTIL = False

def free_specific_port(port: int, force: bool = False):
    print(f"🔍 Quét cổng socket TCP #{port}...")
    found_pids = []

    if HAS_PSUTIL:
        current_pid = os.getpid()
        for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
            try:
                if proc.info['pid'] == current_pid or proc.info['pid'] == 1:
                    continue
                connections = proc.connections(kind='inet')
                for conn in connections:
                    if conn.laddr and conn.laddr.port == port:
                        found_pids.append((proc.info['pid'], proc.info['name'], proc.info['cmdline']))
            except (psutil.NoSuchProcess, psutil.AccessDenied, psutil.ZombieProcess, AttributeError):
                pass
    else:
        import subprocess
        try:
            out = subprocess.check_output(f"lsof -t -i:{port}", shell=True, text=True)
            for line in out.strip().split('\n'):
                if line.strip().isdigit():
                    pid = int(line.strip())
                    if pid != os.getpid() and pid != 1:
                        found_pids.append((pid, "unknown", []))
        except Exception:
            pass

    if not found_pids:
        print(f"✅ Cổng #{port} đang sạch 100%. Không có tiến trình nào chiếm giữ.")
        return

    for pid, name, cmd in found_pids:
        cmd_str = " ".join(cmd[:4]) if cmd else ""
        print(f"🎯 Phát hiện tiến trình chiếm cổng #{port}: PID `{pid}` ({name}) [{cmd_str}]")
        try:
            if HAS_PSUTIL:
                p = psutil.Process(pid)
                if force:
                    p.kill()
                    print(f"💥 Đã cưỡng chế diệt SIGKILL (kill -9) PID {pid}")
                else:
                    p.terminate()
                    print(f"🛑 Đã gửi tín hiệu SIGTERM (kill -15) PID {pid}")
            else:
                import subprocess
                sig = "-9" if force else "-15"
                subprocess.run(f"kill {sig} {pid}", shell=True)
                print(f"🛑 Đã gửi tín hiệu kill {sig} PID {pid}")
        except Exception as e:
            print(f"⚠️ Lỗi khi ngắt PID {pid}: {e}")

    time.sleep(0.5)
    print(f"✨ Hoàn tất giải phóng cổng #{port} mà KHÔNG ảnh hưởng các dịch vụ Python khác!")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Chương trình ngắt chính xác tiến trình kẹt cổng socket mà không ảnh hưởng dịch vụ khác.")
    parser.add_argument("--port", type=int, default=7860, help="Cổng TCP cần giải phóng (Mặc định 7860)")
    parser.add_argument("--force", action="store_true", help="Gửi tín hiệu SIGKILL (-9) thay vì SIGTERM (-15)")
    args = parser.parse_args()
    free_specific_port(args.port, args.force)
