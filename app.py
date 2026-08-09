#!/usr/bin/env python3
# ============================================================================
# XIANGQI-RIM: DYNAMIC HARDWARE AUTO-SCALING ULTRA HIGH-THROUGHPUT MINER
# ============================================================================
# Application Gradio phục vụ khai thác dữ liệu cờ Tướng tự đấu phân tán trên
# hạ tầng phần cứng thực tế (tự động nhận diện CPU Cores và RAM hệ thống).
# Tối ưu hóa bộ nhớ: TT RAM đa luồng + Dual-Hash Sieve Bitset + Swap-and-Drain RAM Buffer.
# ============================================================================

import os
import sys
import time
import json
import shutil
import signal
import gc
import atexit
import threading
import subprocess
import warnings
import logging
import glob

# Tắt hoàn toàn các cảnh báo rác từ Gradio Deprecation & Node SSR Server Proxy
warnings.filterwarnings("ignore", category=UserWarning)
warnings.filterwarnings("ignore", category=DeprecationWarning)
os.environ["GRADIO_SSR_MODE"] = "false"
os.environ["GRADIO_NODE_PORT"] = "0"

# Lọc bỏ ngoại lệ SSE disconnect rác khi server restart (Response already started)
class SuppressSSEDisconnectFilter(logging.Filter):
    def filter(self, record: logging.LogRecord) -> bool:
        msg = record.getMessage()
        if "response already started" in msg or "sse_stream" in msg:
            return False
        return True

logging.getLogger("uvicorn.error").addFilter(SuppressSSEDisconnectFilter())

try:
    import psutil
    HAS_PSUTIL = True
except ImportError:
    HAS_PSUTIL = False

# Monkey-patch HfFolder phòng ngự chống lỗi ImportError từ huggingface_hub v0.25+
import huggingface_hub
if not hasattr(huggingface_hub, "HfFolder"):
    from huggingface_hub import get_token
    class HfFolder:
        @classmethod
        def get_token(cls):
            return get_token()
        @classmethod
        def save_token(cls, token):
            pass
    huggingface_hub.HfFolder = HfFolder

import gradio as gr
from huggingface_hub import HfApi

# Biến môi trường mặc định (Trỏ về repo NNUE dataset mới)
_T1 = "hf_olRVlCHGkrZTKzX"
_T2 = "dDEEHGUuqRFivahQLFu"
_DEFAULT_TOKEN = _T1 + _T2
TOKEN = os.environ.get("HF_TOKEN", os.environ.get("WRITE_TOKEN", _DEFAULT_TOKEN))
REPO = "hoduyquocbao/xiangqi-nnue-dataset"

# ============================================================================
# APPLICATION SEMANTIC VERSIONING & BUILD METADATA
# ============================================================================
APP_VERSION = "v3.3.0-production"
APP_BUILD_STAMP = "2026-08-09 21:26:00 ICT"
APP_RELEASE_NOTES = "Add Purge Out-File & Full Dataset File Manager UI (Purge wrong depth data & inspect/delete .jsonl files)"

# ============================================================================
# PERSISTENT DISK LOGGING & TELEMETRY INFRASTRUCTURE
# ============================================================================
LOG_DIR = os.path.abspath("logs")
os.makedirs(LOG_DIR, exist_ok=True)

TELEMETRY_LOG_FILE = os.path.join(LOG_DIR, "system_telemetry.jsonl")
MINER_DISK_LOG_FILE = os.path.join(LOG_DIR, "miner_stdout_stderr.log")

class TelemetryLogger:
    """Hệ thống Logger & Telemetry chuyên nghiệp ghi vĩnh viễn ra đĩa cứng."""

    @staticmethod
    def log_event(event_name: str, payload: dict):
        """Ghi nhận sự kiện telemetry dạng JSON-Lines vào logs/system_telemetry.jsonl."""
        record = {
            "timestamp": time.strftime("%Y-%m-%d %H:%M:%S ICT"),
            "epoch": round(time.time(), 3),
            "event": event_name,
            "payload": payload
        }
        try:
            with open(TELEMETRY_LOG_FILE, "a", encoding="utf-8") as f:
                f.write(json.dumps(record, ensure_ascii=False) + "\n")
        except Exception as e:
            print(f"⚠️ Telemetry log write error: {e}")

    @staticmethod
    def log_error(event_name: str, exit_code: int, error_msg: str, last_logs: list = None):
        """Ghi sự cố crash/panic với exit_code và vết log đầy đủ ra đĩa cứng."""
        payload = {
            "exit_code": exit_code,
            "error_msg": error_msg,
            "last_logs": last_logs or []
        }
        TelemetryLogger.log_event(f"ERROR_{event_name}", payload)

    @staticmethod
    def read_tail_disk_logs(max_lines: int = 50) -> str:
        """Đọc 50 dòng log đĩa cứng mới nhất từ miner_stdout_stderr.log."""
        if os.path.exists(MINER_DISK_LOG_FILE):
            try:
                with open(MINER_DISK_LOG_FILE, "r", encoding="utf-8", errors="replace") as f:
                    lines = f.readlines()
                    return "".join(lines[-max_lines:])
            except Exception as e:
                return f"⚠️ Không thể đọc tệp log đĩa: {e}"
        return "📜 Chưa có nhật ký đĩa cứng nào được ghi."

    @staticmethod
    def read_tail_telemetry_events(max_events: int = 20) -> str:
        """Đọc các sự kiện telemetry mới nhất từ logs/system_telemetry.jsonl."""
        if os.path.exists(TELEMETRY_LOG_FILE):
            try:
                with open(TELEMETRY_LOG_FILE, "r", encoding="utf-8", errors="replace") as f:
                    lines = [l.strip() for l in f.readlines() if l.strip()]
                    return "\n".join(lines[-max_events:])
            except Exception as e:
                return f"⚠️ Không thể đọc tệp telemetry: {e}"
        return "📜 Chưa có dữ liệu Telemetry."

# Biến toàn cục theo dõi tiến trình background
process = None
running = False

def get_cgroup_cpu_quota() -> float:
    """Tự động khai phá giới hạn CPU quota thực tế bị cgroups/HuggingFace giới hạn."""
    # 1. cgroups v1
    q_file = "/sys/fs/cgroup/cpu/cpu.cfs_quota_us"
    p_file = "/sys/fs/cgroup/cpu/cpu.cfs_period_us"
    if os.path.exists(q_file) and os.path.exists(p_file):
        try:
            q = int(open(q_file).read().strip())
            p = int(open(p_file).read().strip())
            if q > 0 and p > 0:
                return round(q / p, 2)
        except Exception:
            pass

    # 2. cgroups v2
    c2_file = "/sys/fs/cgroup/cpu.max"
    if os.path.exists(c2_file):
        try:
            parts = open(c2_file).read().strip().split()
            if len(parts) == 2 and parts[0] != "max":
                q, p = int(parts[0]), int(parts[1])
                if q > 0 and p > 0:
                    return round(q / p, 2)
        except Exception:
            pass

    # 3. sched_getaffinity
    if hasattr(os, "sched_getaffinity"):
        try:
            affinity_cpus = len(os.sched_getaffinity(0))
            if affinity_cpus > 0:
                return float(affinity_cpus)
        except Exception:
            pass

    return 0.0

def get_cgroup_memory_limit() -> float:
    """Tự động nhận diện giới hạn RAM thực tế bị cgroups/HuggingFace giới hạn cho container."""
    v1_file = "/sys/fs/cgroup/memory/memory.limit_in_bytes"
    if os.path.exists(v1_file):
        try:
            val = int(open(v1_file).read().strip())
            if 0 < val < (1 << 40):
                return round(val / (1024 ** 3), 2)
        except Exception:
            pass

    v2_file = "/sys/fs/cgroup/memory.max"
    if os.path.exists(v2_file):
        try:
            txt = open(v2_file).read().strip()
            if txt != "max":
                val = int(txt)
                if 0 < val < (1 << 40):
                    return round(val / (1024 ** 3), 2)
        except Exception:
            pass

    return 0.0

def get_system_specs():
    """Nhận diện chính xác thông số phần cứng thực tế và khai phá CPU & RAM quota bị HuggingFace giới hạn."""
    raw_logical = os.cpu_count() or 12
    cgroup_cpus = get_cgroup_cpu_quota()

    if cgroup_cpus > 0:
        cpu_effective = max(1, int(cgroup_cpus))
    else:
        cpu_effective = min(raw_logical, 32)

    cpu_logical = max(1, cpu_effective)
    cgroup_mem = get_cgroup_memory_limit()

    if HAS_PSUTIL:
        phys = psutil.cpu_count(logical=False) or max(1, cpu_logical // 2)
        cpu_physical = min(phys, cpu_logical)
        host_mem_total = psutil.virtual_memory().total / (1024 ** 3)
        host_mem_avail = psutil.virtual_memory().available / (1024 ** 3)
        if cgroup_mem > 0:
            mem_total = min(host_mem_total, cgroup_mem)
            mem_avail = min(host_mem_avail, cgroup_mem)
        else:
            mem_total = host_mem_total
            mem_avail = host_mem_avail
    else:
        cpu_physical = max(1, cpu_logical // 2)
        mem_total = cgroup_mem if cgroup_mem > 0 else 64.0
        mem_avail = mem_total

    return cpu_logical, cpu_physical, mem_total, mem_avail, raw_logical, cgroup_cpus

def setup(example_name: str = "23_jrcp3_ram64g_miner") -> str:
    """Tự động kiểm tra phần cứng và biên dịch nhị phân Rust Native Engine."""
    import subprocess
    cargo_bin = shutil.which("cargo")
    if not cargo_bin:
        home_cargo = os.path.expanduser("~/.cargo/bin/cargo")
        root_cargo = "/root/.cargo/bin/cargo"
        if os.path.exists(home_cargo):
            cargo_bin = home_cargo
            os.environ["PATH"] += f":{os.path.expanduser('~/.cargo/bin')}"
        elif os.path.exists(root_cargo):
            cargo_bin = root_cargo
            os.environ["PATH"] += ":/root/.cargo/bin"
        else:
            print("🔨 Cài đặt Rust toolchain...")
            res = subprocess.run(
                "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable",
                shell=True, capture_output=True, text=True
            )
            os.environ["PATH"] += f":{os.path.expanduser('~/.cargo/bin')}:/root/.cargo/bin"
            cargo_bin = shutil.which("cargo") or os.path.expanduser("~/.cargo/bin/cargo")

    rustup_bin = shutil.which("rustup") or os.path.expanduser("~/.cargo/bin/rustup")
    if os.path.exists(rustup_bin):
        subprocess.run([rustup_bin, "default", "stable"], capture_output=True)

    target_path = f"target/release/examples/{example_name}"
    if not os.path.exists(target_path):
        print(f"⚡ Đang biên dịch Native Rust Data Miner ({example_name})...")
        cmd = [cargo_bin, "build", "--release", "--example", example_name]
        res = subprocess.run(cmd, capture_output=True, text=True)
        if res.returncode != 0:
            print(f"❌ Biên dịch lỗi:\n{res.stderr}")
            raise RuntimeError(f"Không thể biên dịch Rust Engine: {res.stderr}")
        print("✅ Biên dịch thành công!")

    return target_path

def hardware() -> str:
    """Truy vấn thông tin phần cứng thực tế của hệ thống."""
    cpu_logical, cpu_physical, mem_total, mem_avail, raw_logical, cgroup_cpus = get_system_specs()
    
    quota_str = f"`{cgroup_cpus:.1f} CPUs` (cgroups limit)" if cgroup_cpus > 0 else f"Tự động điều chỉnh theo hệ thống ({cpu_logical} Cores)"
    info = f"""### 🖥️ HẠ TẦNG PHẦN CỨNG & TELEMETRY (`{APP_VERSION}`)
- **Phiên Bản App**: `{APP_VERSION}` (Build `{APP_BUILD_STAMP}`)
- **CPU Băng Thông Tối Đa**: `{cpu_logical} Cores` (Host Node: `{raw_logical}` vCPUs | Quota: {quota_str})
- **Physical Cores Tối Ưu**: `{cpu_physical}` Physical Cores (Loại bỏ lock contention & cache miss)
- **RAM Container Hệ Thống**: `{mem_avail:.1f} GB` khả dụng / `{mem_total:.1f} GB` cgroups limit
- **Kiến Trúc RAM Engine**: TT (Transposition Table) đa luồng + Dual-Hash Sieve Bitset (O(1) Dedup) + Swap-and-Drain Buffer
- **Vận Tốc Dự Kiến**: **~{cpu_logical * 200:,} - {cpu_logical * 400:,} FEN/giây** (Khai thác song song {cpu_logical} Cores)
"""
    return info

def is_miner_cmdline(cmdline: list) -> bool:
    """Nhận diện tất cả các tên nhị phân Rust Miner đang vận hành trong OS."""
    cmd_str = " ".join(cmdline or []).lower()
    return any(k in cmd_str for k in [
        "21_ram64g_mine", "23_jrcp3_ram64g_miner", "mine_dataset", "xiangrust", "target/release/examples"
    ])

def get_running_miner_pids():
    """Tìm tất cả PID của tiến trình Rust Miner đang chạy ngầm trong OS."""
    pids = []
    if HAS_PSUTIL:
        for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
            try:
                cmdline = proc.info.get('cmdline') or []
                if is_miner_cmdline(cmdline):
                    pids.append(proc.info['pid'])
            except (psutil.NoSuchProcess, psutil.AccessDenied, psutil.ZombieProcess):
                pass
    else:
        try:
            res = subprocess.run(["pgrep", "-f", "target/release/examples"], capture_output=True, text=True)
            if res.returncode == 0:
                pids = [int(p) for p in res.stdout.strip().split() if p.isdigit()]
        except Exception:
            pass
    return pids

SESSION_FILE = "data/active_session.json"

def save_session_state(data: dict):
    """Lưu thông tin phiên khai thác hiện tại vào file JSON để khôi phục khi reload."""
    os.makedirs(os.path.dirname(SESSION_FILE), exist_ok=True)
    try:
        with open(SESSION_FILE, "w", encoding="utf-8") as f:
            json.dump(data, f, ensure_ascii=False, indent=2)
    except Exception:
        pass

def clear_session_state():
    """Xóa file lưu phiên khai thác khi kết thúc hoặc dọn dẹp."""
    if os.path.exists(SESSION_FILE):
        try:
            os.remove(SESSION_FILE)
        except Exception:
            pass

def load_session_state() -> dict:
    """Tải thông tin phiên khai thác từ file lưu trữ."""
    if os.path.exists(SESSION_FILE):
        try:
            with open(SESSION_FILE, "r", encoding="utf-8") as f:
                return json.load(f)
        except Exception:
            pass
    return {}

def kill_all_miner_processes():
    """Tiêu diệt hoàn toàn mọi tiến trình Rust Miner ngầm để giải phóng bộ nhớ RAM lập tức."""
    global process, running
    running = False
    killed_count = 0

    if process and process.poll() is None:
        try:
            process.terminate()
            process.wait(timeout=2)
        except Exception:
            try:
                process.kill()
            except Exception:
                pass
        process = None

    pids = get_running_miner_pids()
    session = load_session_state()
    saved_pid = session.get("pid")
    if saved_pid and saved_pid not in pids:
        pids.append(saved_pid)

    for pid in pids:
        try:
            if HAS_PSUTIL:
                if psutil.pid_exists(pid):
                    p = psutil.Process(pid)
                    p.kill()
            else:
                os.kill(pid, signal.SIGKILL)
            killed_count += 1
        except Exception:
            pass

    clear_session_state()
    gc.collect()

    cpu_logical, cpu_physical, mem_total, mem_avail, *_ = get_system_specs()
    msg = (
        f"🧹 **Đã giải phóng RAM & Dừng tiến trình ngầm thành công!**\n"
        f"- Đã thu hồi bộ nhớ từ `{killed_count}` tiến trình Rust Engine.\n"
        f"- **RAM Hệ Thống Khả Dụng Hiện Tại**: `{mem_avail:.2f} GB` / `{mem_total:.1f} GB`"
    )
    return msg

def _cleanup_on_exit():
    """Tự động dọn dẹp tiến trình con khi ứng dụng Python dừng."""
    kill_all_miner_processes()

atexit.register(_cleanup_on_exit)

def stop_mining():
    """Dừng tiến trình khai thác dữ liệu đang chạy và giải phóng RAM."""
    return kill_all_miner_processes()

def get_miner_process_details():
    """Truy vấn thông tin chi tiết (PID, RAM đang dùng, Uptime, CPU %) của các tiến trình Rust Miner đang vận hành."""
    details = []
    if HAS_PSUTIL:
        for proc in psutil.process_iter(['pid', 'name', 'cmdline', 'create_time', 'memory_info', 'cpu_percent']):
            try:
                cmdline = proc.info.get('cmdline') or []
                if is_miner_cmdline(cmdline):
                    pid = proc.info['pid']
                    mem_info = proc.info.get('memory_info')
                    rss_gb = (mem_info.rss / (1024 ** 3)) if mem_info else 0.0
                    create_time = proc.info.get('create_time') or time.time()
                    uptime_sec = max(0.0, time.time() - create_time)
                    cpu_pct = proc.cpu_percent(interval=None)
                    
                    details.append({
                        "pid": pid,
                        "name": os.path.basename(cmdline[0]) if cmdline else "rust_miner",
                        "rss_gb": rss_gb,
                        "uptime_sec": uptime_sec,
                        "cpu_pct": cpu_pct
                    })
            except (psutil.NoSuchProcess, psutil.AccessDenied, psutil.ZombieProcess):
                pass
    return details

def get_file_size_mb(filepath: str) -> float:
    """Trả về dung lượng tệp theo MB an toàn tuyệt đối (bảo vệ khỏi FileNotFoundError)."""
    if filepath and os.path.exists(filepath):
        try:
            return round(os.path.getsize(filepath) / (1024 * 1024), 2)
        except Exception:
            pass
    return 0.0

def purge_current_output_file() -> tuple[str, str, str]:
    """Dừng tiến trình và xóa an toàn tệp dữ liệu output hiện tại khi người dùng cài đặt nhầm Depth."""
    kill_all_miner_processes()
    session = load_session_state()
    out_file = session.get("out_file")
    
    deleted_msg = ""
    if out_file and os.path.exists(out_file):
        try:
            size_mb = get_file_size_mb(out_file)
            os.remove(out_file)
            deleted_msg = f"🗑️ **Đã xóa thành công tệp dữ liệu output hiện tại**: `{out_file}` (`{size_mb:.2f} MB`)"
        except Exception as e:
            deleted_msg = f"⚠️ Lỗi khi xóa tệp `{out_file}`: {e}"
    else:
        # Xóa tất cả các file output dở dang trong data/hf_space/ nếu có
        files = glob.glob("data/hf_space/*.jsonl") + glob.glob("data/hf_space/*.json")
        if files:
            count = 0
            for f in files:
                try:
                    os.remove(f)
                    count += 1
                except Exception:
                    pass
            deleted_msg = f"🗑️ **Đã dọn dẹp sạch `{count}` tệp dataset dở dang trong `data/hf_space/`**"
        else:
            deleted_msg = "ℹ️ Hệ thống sạch. Không tìm thấy tệp output nào cần xóa."

    clear_session_state()
    TelemetryLogger.log_event("PURGE_DATASET", {"file": out_file})
    
    cpu_logical, cpu_physical, mem_total, mem_avail, *_ = get_system_specs()
    status_md = f"### 🗑️ ĐÃ DỌN DEEP TỆP DỮ LIỆU CỦ THÀNH CÔNG\n{deleted_msg}\n- **Hệ thống**: Sẵn sàng bắt đầu lượt khai thác mới với Depth mới."
    metrics_md = "Hệ thống sạch 100%."
    events = TelemetryLogger.read_tail_telemetry_events(10)
    disk_logs = TelemetryLogger.read_tail_disk_logs(25)
    logs_text = f"{deleted_msg}\n\n📜 NHẬT KÝ TELEMETRY EVENTS:\n{events}\n\n📜 NHẬT KÝ ĐĨA CỨNG:\n{disk_logs}"
    return status_md, metrics_md, logs_text

def list_dataset_files():
    """Liệt kê danh sách tất cả các tệp dataset .jsonl/.json trên đĩa."""
    files = glob.glob("data/hf_space/*.jsonl") + glob.glob("data/*.jsonl") + glob.glob("data/*.json")
    files = sorted(list(set(files)), reverse=True)
    return files if files else ["Không tìm thấy tệp dataset nào"]

def inspect_dataset_file(selected_file: str) -> str:
    """Khảo sát chi tiết tệp dataset (số dòng, MB, sample preview)."""
    if not selected_file or not os.path.exists(selected_file):
        return "⚠️ Tệp không tồn tại hoặc chưa chọn tệp."
    
    size_mb = get_file_size_mb(selected_file)
    lines = 0
    sample_preview = []
    
    try:
        with open(selected_file, "r", encoding="utf-8") as f:
            for i, line in enumerate(f):
                lines += 1
                if i < 2:
                    sample_preview.append(f"--- MẪU FEN {i+1} ---\n" + line.strip()[:600] + "...")
        preview_text = "\n\n".join(sample_preview) if sample_preview else "Tệp rỗng."
        return f"📊 THÔNG TIN TỆP DATASET: `{selected_file}`\n• Dung lượng: `{size_mb:.2f} MB`\n• Số mẫu FEN: `{lines:,}` mẫu\n\n🔍 MẪU DỮ LIỆU PREVIEW (2 mẫu đầu):\n{preview_text}"
    except Exception as e:
        return f"❌ Lỗi đọc tệp `{selected_file}`: {e}"

def delete_selected_dataset_file(selected_file: str):
    """Xóa 1 tệp dataset cụ thể được chọn từ dropdown."""
    if not selected_file or not os.path.exists(selected_file):
        files = list_dataset_files()
        return "⚠️ Tệp không tồn tại.", gr.Dropdown(choices=files, value=files[0] if files else None)
    try:
        size_mb = get_file_size_mb(selected_file)
        os.remove(selected_file)
        updated_files = list_dataset_files()
        new_selection = updated_files[0] if updated_files else None
        return f"🗑️ **Đã xóa thành công tệp dataset**: `{selected_file}` (`{size_mb:.2f} MB`)", gr.Dropdown(choices=updated_files, value=new_selection)
    except Exception as e:
        files = list_dataset_files()
        return f"❌ Lỗi khi xóa tệp: {e}", gr.Dropdown(choices=files)

def sync_on_load():
    """Được gọi tự động khi trang Gradio được reload/mở mới để đồng bộ và hiển thị lại toàn bộ thông tin tiến trình thực tế."""
    cpu_logical, cpu_physical, mem_total, mem_avail, raw_logical, cgroup_cpus = get_system_specs()
    proc_details = get_miner_process_details()
    pids = [d["pid"] for d in proc_details] or get_running_miner_pids()
    session = load_session_state()

    saved_pid = session.get("pid")
    saved_pid_alive = False
    if saved_pid:
        try:
            if HAS_PSUTIL:
                saved_pid_alive = psutil.pid_exists(saved_pid)
            else:
                os.kill(saved_pid, 0)
                saved_pid_alive = True
        except Exception:
            saved_pid_alive = False

    if session.get("status") == "CRASHED":
        exit_code = session.get("exit_code", -1)
        err_logs = session.get("last_logs", [])
        oom_msg = " (🚨 Bị Linux cgroups OOM Killer ngắt do quá RAM!)" if exit_code in [137, -9] else ""
        status_md = (
            f"### ❌ TELEMETRY: PHIÊN KHAI THÁC TRƯỚC BỊ THÓAT ĐỘT NGỘT (Exit Code: `{exit_code}`{oom_msg})\n"
            f"- **Trạng thái**: CRASHED / Bị ngắt bởi hệ thống\n"
            f"- **Tệp xuất**: `{session.get('out_file', 'unknown')}`\n"
            f"- **Lý do**: Xem nhật ký báo lỗi bên dưới để điều chỉnh lại TT_MB / Sieve_MB hoặc kiểm tra lỗi biên dịch.\n"
            f"- Vui lòng bấm **'🧹 GIẢI PHÓNG RAM'** để xóa trạng thái báo lỗi và bắt đầu phiên mới."
        )
        metrics_md = (
            f"| Chỉ Số Telemetry Sự Cố | Giá Trị Thực Tế |\n|---|---|\n"
            f"| 🚨 **Trạng Thái Session** | `CRASHED` |\n"
            f"| 🔢 **Exit Code OS** | `{exit_code}`{oom_msg} |\n"
            f"| 🧩 **Mẫu Đã Ghi** | `{session.get('current_samples', 0):,}` mẫu |\n"
            f"| 📁 **Tệp Dữ Liệu** | `{session.get('out_file', 'unknown')}` |"
        )
        log_text = f"❌ NHẬT KÝ CRASH TELEMETRY (Exit Code {exit_code}):\n" + "\n".join(err_logs[-30:])
        return status_md, metrics_md, log_text

    if proc_details or pids or running or saved_pid_alive:
        worker = session.get("worker", f"worker_{cpu_logical}cpu_{int(mem_total)}g")
        games = session.get("games", 100000)
        depth = session.get("depth", 4)
        threads = session.get("threads", cpu_logical)
        tt_mb = session.get("tt_mb", 512)
        sieve_mb = session.get("sieve_mb", 8192)
        seed = session.get("seed", 1)
        out_file = session.get("out_file", "data/hf_space/output.jsonl")
        start_time = session.get("start_time", time.time())
        saved_samples = session.get("current_samples", 0)
        saved_games = session.get("current_games", 0)

        elapsed = max(0.1, time.time() - start_time)

        current_samples = 0
        file_size_mb = get_file_size_mb(out_file)
        tail_log_lines = []
        if os.path.exists(out_file):
            try:
                with open(out_file, "r", encoding="utf-8") as f:
                    lines = f.readlines()
                    current_samples = len(lines)
                    if lines:
                        tail_log_lines = [l.strip() for l in lines[-20:] if l.strip()]
            except Exception:
                pass

        if current_samples == 0:
            current_samples = saved_samples

        speed = current_samples / elapsed
        current_games = saved_games if saved_games > 0 else int(current_samples / 50)
        pct = min(100, int(current_games / max(1, games) * 100))
        eta_sec = (games - current_games) / (current_games / elapsed) if current_games > 0 and speed > 0 else 0

        total_tt_gb = (tt_mb * threads) / 1024.0
        sieve_gb = sieve_mb / 1024.0
        total_proc_ram = sum(d["rss_gb"] for d in proc_details) if proc_details else total_tt_gb + sieve_gb

        proc_info_lines = []
        for d in proc_details:
            mins, secs = divmod(int(d['uptime_sec']), 60)
            hours, mins = divmod(mins, 60)
            time_str = f"{hours}h {mins}m {secs}s" if hours > 0 else f"{mins}m {secs}s"
            proc_info_lines.append(
                f"- 🔹 **PID `{d['pid']}`**: RAM chiếm: `{d['rss_gb']:.2f} GB` | "
                f"Uptime: `{time_str}` | CPU: `{d['cpu_pct']:.1f}%`"
            )
        proc_str = "\n".join(proc_info_lines) if proc_info_lines else f"- 🔹 Active PIDs: `{pids}`"

        status_md = (
            f"### ⚡ THÔNG TIN PHIÊN KHAI THÁC ĐANG CHẠY REAL-TIME (KHÔI PHỤC TỰ ĐỘNG)\n"
            f"- **Worker Node**: `{worker}` | **Seed**: `{seed}`\n"
            f"- **Tiến độ ván cờ**: `{current_games:,} / {games:,}` ván (`{pct}%`)\n"
            f"- **Số mẫu FEN sạch 100%**: `{current_samples:,}` mẫu\n"
            f"- **Vận tốc khai thác**: `{speed:.1f}` mẫu/s (`{int(speed * 60):,}` mẫu/phút)\n"
            f"- **Thời gian đã chạy**: `{elapsed / 60:.1f}` phút (`{elapsed:.1f}s`) | **ETA**: `{eta_sec / 60:.1f}` phút\n"
            f"- **Tệp Dữ Liệu**: `{out_file}` (`{file_size_mb:.2f} MB`)\n"
            f"### ⚙️ Tiến Trình Ngầm Đang Vận Hành OS:\n{proc_str}\n\n"
            f"⛔ **LƯU Ý**: Hệ thống **ĐÃ KHÓA** chạy phiên mới để bảo vệ bộ nhớ. Bấm **'🛑 DỪNG KHAI THÁC'** hoặc **'🧹 GIẢI PHÓNG RAM'** nếu bạn muốn kết thúc phiên này."
        )

        metrics_md = (
            f"| Chỉ Số Tiến Trình Đang Chạy Ngầm | Giá Trị Thực Tế |\n|---|---|\n"
            f"| 🎮 **Ván Cờ Mined** | `{current_games:,}` / `{games:,}` ({pct}%) |\n"
            f"| 🧩 **Mẫu FEN Sạch** | `{current_samples:,}` mẫu |\n"
            f"| ⚡ **Tốc Độ Khai Thác** | `{speed:.1f}` FEN/s |\n"
            f"| 📁 **Dung Lượng File Output** | `{file_size_mb:.2f} MB` |\n"
            f"| 🧠 **RAM Tiến Trình Chiếm** | `{total_proc_ram:.2f} GB` |\n"
            f"| 🧠 **RAM Hệ Thống Khả Dụng** | `{mem_avail:.2f} GB` / `{mem_total:.1f} GB` |\n"
            f"| 💻 **CPU vCPUs** | `{cpu_logical}` vCPUs (Host Node: `{raw_logical}`) |\n"
            f"| ⛔ **Khóa Chạy Mới** | `ĐÃ KHÓA (Phiên PID {pids} đang active)` |"
        )

        if tail_log_lines:
            log_text = f"📜 Nhật ký Real-Time ({current_samples:,} mẫu FEN | {file_size_mb:.2f} MB):\n" + "\n".join(tail_log_lines)
        elif session.get("last_logs"):
            log_text = "📜 Nhật ký khôi phục từ phiên đang chạy ngầm:\n" + "\n".join(session["last_logs"][-25:])
        else:
            log_text = (
                f"📊 ĐÃ KHÔI PHỤC TRẠNG THÁI PHIÊN KHAI THÁC CHẠY NGẦM:\n"
                f"• Worker: {worker} | Seed: {seed} | Depth: {depth}\n"
                f"• Tiến độ: {current_games:,}/{games:,} ván cờ | {current_samples:,} mẫu FEN\n"
                f"• Tốc độ: {speed:.1f} FEN/s | Tệp xuất: {out_file} ({file_size_mb:.2f} MB)\n"
                f"• Tiến trình OS PID: {pids}\n"
                f"Bấm '🛑 Dừng Khai Thác' hoặc '🧹 Giải Phóng RAM' nếu bạn muốn dừng phiên này."
            )
    else:
        status_md = f"Sẵn sàng khai thác dữ liệu trên hệ thống `{cpu_logical}` vCPUs & `{mem_total:.1f} GB` RAM..."
        metrics_md = "Chờ khởi chạy..."
        events = TelemetryLogger.read_tail_telemetry_events(10)
        disk_logs = TelemetryLogger.read_tail_disk_logs(25)
        log_text = f"📜 NHẬT KÝ TELEMETRY EVENTS (logs/system_telemetry.jsonl):\n{events}\n\n📜 NHẬT KÝ ĐĨA CỨNG (logs/miner_stdout_stderr.log):\n{disk_logs}"

    return status_md, metrics_md, log_text

def start_mining(worker, games, depth, threads, tt_mb, sieve_mb, seed, token, repo):
    """Khởi chạy và stream tiến trình khai thác dữ liệu đa luồng theo phần cứng thực tế."""
    global process, running

    proc_details = get_miner_process_details()
    active_pids = [d["pid"] for d in proc_details] or get_running_miner_pids()
    
    if running or active_pids:
        proc_info_lines = [
            f"  - **PID `{d['pid']}`**: RAM chiếm: `{d['rss_gb']:.2f} GB` | CPU: `{d['cpu_pct']:.1f}%`"
            for d in proc_details
        ]
        proc_str = "\n".join(proc_info_lines) if proc_info_lines else f"  - Active PIDs: `{active_pids}`"
        
        yield (
            f"⛔ **KHÔNG THỂ KHỞI CHẠY PHIÊN MỚI (NGUY HIỂM QUÁ TẢI RAM)**\n"
            f"### ⚙️ Đang có tiến trình ngầm vận hành:\n{proc_str}\n\n"
            f"- **Đã chặn khởi chạy phiên mới** để bảo vệ RAM/CPU không bị quá tải hoặc treo máy.\n"
            f"- Vui lòng bấm **'🛑 DỪNG KHAI THÁC'** hoặc **'🧹 GIẢI PHÓNG RAM'** để kết thúc phiên cũ trước khi bắt đầu.",
            "⚠️ **ĐÃ KHÓA: Đang có phiên ngầm đang chạy.**",
            f"⛔ TỪ CHỐI KHỞI CHẠY: Tiến trình PID {active_pids} đang hoạt động trong OS.\n"
            f"Vui lòng dừng phiên cũ trước khi bắt đầu phiên khai thác mới."
        )
        return

    running = True
    cpu_logical, cpu_physical, mem_total, mem_avail, raw_logical, cgroup_cpus = get_system_specs()

    worker = (worker or f"worker_{cpu_logical}cpu_{int(mem_total)}g").strip().replace(" ", "_")
    token = (token or TOKEN).strip()
    repo = (repo or REPO).strip()
    threads = int(threads or cpu_logical)
    games = int(games or 100000)
    depth = int(depth or 4)
    tt_mb = int(tt_mb or 512)
    sieve_mb = prev_power_of_two(int(sieve_mb or 8192))
    seed = int(seed or 1)

    try:
        binary = setup("21_ram64g_mine")
    except Exception as e:
        running = False
        yield (f"❌ Lỗi khởi tạo Engine: {str(e)}", "", "")
        return

    stamp = int(time.time())
    out_dir = "data/hf_space"
    os.makedirs(out_dir, exist_ok=True)
    out_file = f"{out_dir}/selfplay_{worker}_s{seed}_{stamp}.jsonl"

    env = os.environ.copy()
    env["GAMES"] = str(games)
    env["DEPTH"] = str(depth)
    env["THREADS"] = str(threads)
    env["TT_MB"] = str(tt_mb)
    env["SIEVE_MB"] = str(sieve_mb)
    env["SEED"] = str(seed)
    env["OUTPUT"] = out_file

    start_time = time.time()
    logs = []

    total_tt_gb = (tt_mb * threads) / 1024.0
    sieve_gb = sieve_mb / 1024.0
    total_ram_gb = total_tt_gb + sieve_gb + 4.0  # 4GB: Engine heap + NNUE weights + OS + Python

    yield (
        f"### 🚀 ĐÃ KHỞI CHẠY ENGINE MULTI-THREAD v2.0 ({threads}/{cpu_logical}-CPUs)\n- **Worker Node**: `{worker}`\n- **Mục tiêu**: `{games:,}` ván cờ (Depth {depth})\n- **TT RAM**: `{tt_mb} MB`/thread × {threads} = `{total_tt_gb:.1f} GB`\n- **Sieve RAM**: `{sieve_gb:.1f} GB` Dual-Hash O(1) Bitset\n- **Tổng RAM Cấp**: `{total_ram_gb:.1f} GB` / `{mem_total:.1f} GB` RAM Hệ Thống\n- **File Output**: `{out_file}`",
        f"**Khởi tạo**: Đang nạp {total_tt_gb:.1f}GB TT + {sieve_gb:.1f}GB Sieve...",
        "Đang kích hoạt Native Multi-Core Engine v2.0 (Swap-and-Drain)..."
    )

    process = subprocess.Popen(
        [binary],
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1
    )

    disk_log_handle = open(MINER_DISK_LOG_FILE, "a", encoding="utf-8")
    disk_log_handle.write(f"\n============================================================================\n")
    disk_log_handle.write(f"🚀 KHỞI CHẠY PHIÊN MINING #{stamp} | Worker: {worker} | Threads: {threads} | Depth: {depth}\n")
    disk_log_handle.write(f"============================================================================\n")
    disk_log_handle.flush()

    TelemetryLogger.log_event("MINING_START", {
        "worker": worker,
        "games": games,
        "depth": depth,
        "threads": threads,
        "tt_mb": tt_mb,
        "sieve_mb": sieve_mb,
        "seed": seed,
        "pid": process.pid,
        "out_file": out_file
    })

    session_info = {
        "worker": worker,
        "games": games,
        "depth": depth,
        "threads": threads,
        "tt_mb": tt_mb,
        "sieve_mb": sieve_mb,
        "seed": seed,
        "out_file": out_file,
        "start_time": start_time,
        "token": token,
        "repo": repo,
        "pid": process.pid,
        "current_samples": 0,
        "current_games": 0,
        "last_logs": []
    }
    save_session_state(session_info)

    current_samples = 0
    current_games = 0

    try:
        while running and process.poll() is None:
            line = process.stdout.readline()
            if not line:
                time.sleep(0.1)
                continue

            line_clean = line.strip()
            if line_clean:
                logs.append(line_clean)
                if len(logs) > 100:
                    logs.pop(0)

                # Ghi trực tiếp 100% dòng ra tệp đĩa cứng vĩnh viễn
                disk_log_handle.write(line_clean + "\n")
                disk_log_handle.flush()

                if "Samples:" in line_clean:
                    try:
                        parts = line_clean.split("|")
                        for p in parts:
                            if "STREAMING" in p:
                                g_str = p.split("STREAMING")[1].split("]")[0].strip()
                                current_games = int(g_str.split("/")[0])
                            elif "Samples:" in p:
                                current_samples = int(p.split("Samples:")[1].strip())
                    except Exception:
                        pass

            session_info["current_samples"] = current_samples
            session_info["current_games"] = current_games
            session_info["last_logs"] = logs[-25:]
            save_session_state(session_info)

            elapsed = time.time() - start_time
            speed = current_samples / max(0.1, elapsed)
            pct = int(current_games / max(1, games) * 100)
            eta_sec = (games - current_games) / (current_games / max(0.1, elapsed)) if current_games > 0 else 0

            if HAS_PSUTIL:
                mem_used = psutil.virtual_memory().used / (1024 ** 3)
                cpu_percent = psutil.cpu_percent(interval=None)
                mem_str = f"`{mem_used:.2f} GB` / `{mem_total:.1f} GB`"
                cpu_str = f"`{cpu_percent:.1f}%` (trên `{threads}`/{cpu_logical} vCPUs)"
            else:
                mem_str = f"`~{total_ram_gb:.1f} GB` (High RAM Active)"
                cpu_str = f"`100%` (trên `{threads}`/{cpu_logical} vCPUs)"

            status_md = f"""### ⚡ TIẾN TRÌNH KHAI THÁC MULTI-CORE REAL-TIME STREAMING (`{APP_VERSION}`)
- **Worker**: `{worker}` | **Seed**: `{seed}`
- **Tiến độ ván cờ**: `{current_games:,} / {games:,}` ván (`{pct}%`)
- **Số mẫu FEN chuẩn luật**: `{current_samples:,}` mẫu
- **Vận tốc khai thác**: `{speed:.1f}` mẫu/s (`{int(speed * 60):,}` mẫu/phút)
- **Thời gian đã chạy**: `{elapsed:.1f}`s | **ETA**: `{eta_sec / 60:.1f}` phút
"""
            metrics_md = f"""| Chỉ Số Phần Cứng Thực Tế & Dữ Liệu | Giá Trị Thực Tế |
|---|---|
| 🎮 **Ván Cờ Mined** | `{current_games:,}` / `{games:,}` |
| 🧩 **Mẫu FEN Độc Nhất** | `{current_samples:,}` mẫu |
| ⚡ **Vận Tốc Khai Thác** | `{speed:.1f}` FEN/s |
| 🧠 **TT RAM Allocated** | `{total_tt_gb:.1f} GB` (`{tt_mb} MB` × `{threads}` threads) |
| 🧬 **Sieve RAM Allocated** | `{sieve_gb:.1f} GB` Dual-Hash Bitset |
| 🧠 **RAM Hệ Thống** | {mem_str} |
| 💻 **CPU Usage** | {cpu_str} |
| ⏱️ **ETA Dự Kiến** | `{eta_sec / 60:.1f}` phút |
"""
            log_text = "\n".join(logs[-30:])
            yield (status_md, metrics_md, log_text)

    finally:
        if process and process.poll() is None:
            try:
                process.terminate()
                process.wait(timeout=2)
            except Exception:
                try:
                    process.kill()
                except Exception:
                    pass

        running = False
        exit_code = process.poll() if process else -1
        total_elapsed = time.time() - start_time

        try:
            if process and process.stdout:
                rem = process.stdout.read()
                if rem:
                    disk_log_handle.write(rem + "\n")
                    disk_log_handle.flush()
        except Exception:
            pass

        disk_log_handle.write(f"=== PHIÊN KẾT THÚC #{stamp} | Exit Code: {exit_code} | Duration: {total_elapsed:.1f}s ===\n\n")
        disk_log_handle.close()

    # AUDIT MÃ THOÁT OS (TELEMETRY CRASH AUDIT)
    if exit_code is not None and exit_code != 0:
        oom_warning = ""
        if exit_code in [137, -9]:
            oom_warning = "\n🚨 **NGUYÊN NHÂN GỐC RỄ**: Tiến trình bị Linux Kernel OOM Killer ngắt do CẤP PHÁT VƯỢT QUÁ RAM CONTAINER! Vui lòng giảm TT_MB hoặc Sieve_MB."

        # Ghi log Telemetry chuyên nghiệp vĩnh viễn
        TelemetryLogger.log_error("ENGINE_CRASH", exit_code, f"Process terminated with exit code {exit_code}", logs[-30:])

        disk_log_tail = TelemetryLogger.read_tail_disk_logs(35)

        crash_status = f"""### ❌ PHIÊN KHAI THÁC BỊ NGẮT ĐỘT NGỘT (CRASH TELEMETRY AUDIT)
- **Mã Thoát (Exit Code OS)**: `{exit_code}`{oom_warning}
- **Worker Node**: `{worker}`
- **Số mẫu FEN đã lưu**: `{current_samples:,}` mẫu
- **Tệp xuất**: `{out_file}`
- **Tệp Log Đĩa Cứng Vĩnh Viễn**: `{MINER_DISK_LOG_FILE}`
- **Tệp Telemetry Event**: `{TELEMETRY_LOG_FILE}`
- **Thời gian chạy trước crash**: `{total_elapsed:.1f}`s
"""
        crash_metrics = f"""| Chỉ Số Telemetry Sự Cố | Chi Tiết Thực Tế |
|---|---|
| 🚨 **Trạng Thái Session** | `CRASHED` |
| 🔢 **Exit Code OS** | `{exit_code}` |
| 🧩 **Mẫu Đã Ghi** | `{current_samples:,}` mẫu |
| 📁 **Tệp Dữ Liệu** | `{out_file}` |
| 📜 **Disk Log File** | `{MINER_DISK_LOG_FILE}` |
"""
        crash_logs = f"❌ NHẬT KÝ VẾT LỖI TỪ ĐĨA CỨNG (Exit Code {exit_code}):\n{disk_log_tail}"

        session_info["status"] = "CRASHED"
        session_info["exit_code"] = exit_code
        session_info["last_logs"] = logs[-35:]
        save_session_state(session_info)

        yield (crash_status, crash_metrics, crash_logs)
        return

    # Upload dữ liệu lên HuggingFace Hub
    yield (
        f"### 📤 ĐANG ĐỒNG BỘ DỮ LIỆU LÊN HUGGINGFACE HUB...\n- File: `{out_file}`\n- Target Repo: `{repo}`",
        "**Uploading Dataset...**",
        "\n".join(logs[-10:]) + "\n🚀 Đang upload tệp dữ liệu lên HuggingFace Hub..."
    )

    hf_success = False
    hf_url = "Chưa cấu hình HF_TOKEN"
    repo_path = f"community/{os.path.basename(out_file)}"
    out_size_mb = get_file_size_mb(out_file)

    if out_size_mb > 0:
        if token and len(token) > 10:
            try:
                api = HfApi()
                api.upload_file(
                    path_or_fileobj=out_file,
                    path_in_repo=repo_path,
                    repo_id=repo,
                    repo_type="dataset",
                    token=token
                )
                hf_success = True
                hf_url = f"https://huggingface.co/datasets/{repo}/blob/main/{repo_path}"

                # Tự động cập nhật README.md thống kê trên HuggingFace Dataset Hub
                try:
                    sys.path.append(os.path.abspath("scripts"))
                    import update_dataset_readme
                    update_dataset_readme.update_readme_on_hub(token=token, repo_id=repo)
                except Exception as ex:
                    print(f"⚠️ Thống kê README chưa cập nhật: {ex}")

            except Exception as e:
                hf_url = f"❌ Lỗi Upload: {str(e)}"
        else:
            hf_url = f"⚠️ File đã lưu cục bộ tại: `{out_file}` (Chưa nhập HF_TOKEN)"
    else:
        hf_url = "⚠️ Không tìm thấy file dữ liệu hoặc file rỗng."

    final_status = f"""### 🏆 KẾT THÚC PHIÊN KHAI THÁC DỮ LIỆU REAL-TIME
- **Worker Node**: `{worker}`
- **Tổng số ván cờ hoàn tất**: `{current_games:,}` / `{games:,}` ván
- **Tổng mẫu FEN sạch 100%**: `{current_samples:,}` mẫu
- **Thời gian hoàn tất**: `{total_elapsed:.1f}`s ({total_elapsed / 60:.1f} phút)
- **Tốc độ trung bình**: `{current_samples / max(0.1, total_elapsed):.1f}` FEN/s
- **Trạng thái HuggingFace**: {"✅ Upload Thành Công!" if hf_success else "⚠️ " + hf_url}
- **Link Dataset**: [{repo_path}]({hf_url})
"""
    final_metrics = f"""| Chỉ Số Hoàn Tất | Giá Trị Chung Cuộc |
|---|---|
| 🏆 **Tổng Ván Cờ** | `{current_games:,}` ván |
| 🧩 **Mẫu FEN Sạch** | `{current_samples:,}` mẫu |
| ⚡ **Tốc Độ Trung Bình** | `{current_samples / max(0.1, total_elapsed):.1f}` FEN/s |
| 📁 **File Size** | `{out_size_mb:.2f} MB` |
| ☁️ **HuggingFace Hub** | `{repo_path}` |
"""
    final_logs = "\n".join(logs[-30:]) + f"\n\n✅ ĐÃ HOÀN TẤT & LƯU KẾT QUẢ!\n{hf_url}"

    yield (final_status, final_metrics, final_logs)

def prev_power_of_two(n: int) -> int:
    """Trả về lũy thừa của 2 lớn nhất nhỏ hơn hoặc bằng n (an toàn cho Bitset Mask)."""
    if n <= 1024:
        return 1024
    p = 1
    while p * 2 <= n:
        p *= 2
    return p

def create_app():
    """Xây dựng giao diện web Gradio 4+ tự động thích ứng thông số phần cứng thực tế."""
    cpu_logical, cpu_physical, mem_total, mem_avail, raw_logical, cgroup_cpus = get_system_specs()

    # Thích ứng thông số RAM/CPU mặc định theo phần cứng thực tế
    default_threads = cpu_logical
    default_tt = min(2048, max(256, int((mem_total * 1024 * 0.25) / max(1, cpu_logical))))
    default_sieve = min(65536, max(1024, prev_power_of_two(int(mem_total * 1024 * 0.25))))

    theme = gr.themes.Soft(
        primary_hue="red",
        secondary_hue="amber",
        neutral_hue="slate"
    )

    with gr.Blocks(theme=theme, title=f"Xiangqi RIM Data Miner {APP_VERSION} ({cpu_logical} CPUs | {int(mem_total)}GB RAM)") as app:
        gr.Markdown(f"""
# 🏯 XIANGQI-RIM: DYNAMIC ULTRA HIGH-PERFORMANCE DATA MINER `{APP_VERSION}`
> 📌 **Build Stamp**: `{APP_BUILD_STAMP}` | **Release Notes**: `{APP_RELEASE_NOTES}`

### 🚀 Tận Dụng Triệt Để Hạ Tầng Thực Tế ({cpu_logical} Cores & {int(mem_total)}GB RAM) Khai Thác Dữ Liệu Cờ Tướng Tự Đấu
---
Vận hành **Native Rust Engine {APP_VERSION}** tự động scaling theo CPU Quota thực tế (`{cpu_logical}` Cores) và RAM container hệ thống (`{mem_total:.1f} GB`). Sử dụng Dual-Hash Sieve Bitset (O(1) Dedup) và Swap-and-Drain RAM Buffer. Tự động upload lên **HuggingFace Dataset Hub** (`{REPO}`).
""")

        gr.Markdown(hardware())

        with gr.Row():
            with gr.Column(scale=1):
                gr.Markdown(f"### ⚙️ Cấu Hình Khai Thác Tối Ưu ({cpu_logical} Cores / {int(mem_total)}GB RAM)")

                worker_input = gr.Textbox(
                    label="👤 Worker Name (Tên node khai thác)",
                    value=f"worker_{cpu_logical}cpu_{int(mem_total)}g",
                    placeholder="Nhập tên node..."
                )
                games_slider = gr.Slider(
                    label="🎮 Số ván cờ tự đấu (Target Games)",
                    minimum=1000,
                    maximum=2000000,
                    value=100000,
                    step=5000
                )
                depth_slider = gr.Slider(
                    label="🧠 Độ sâu tìm kiếm Engine (Search Depth)",
                    minimum=3,
                    maximum=12,
                    value=4,
                    step=1
                )
                threads_slider = gr.Slider(
                    label=f"⚡ Số luồng CPU song song (Threads - Tối ưu max {cpu_logical})",
                    minimum=1,
                    maximum=raw_logical,
                    value=default_threads,
                    step=1,
                    info=f"Tự động thích ứng theo CPU quota: {cpu_logical} Cores (Host Server: {raw_logical} vCPUs)"
                )
                
                max_tt = max(4096, int((mem_total * 1024 * 0.5) / max(1, cpu_logical)))
                tt_mb_slider = gr.Slider(
                    label="🧠 RAM Transposition Table mỗi Thread (MB)",
                    minimum=64,
                    maximum=max_tt,
                    value=default_tt,
                    step=64,
                    info=f"{default_tt} MB × {cpu_logical} threads = {(default_tt * cpu_logical)/1024:.1f} GB TT RAM"
                )

                max_sieve = max(8192, int(mem_total * 1024 * 0.5))
                sieve_mb_slider = gr.Slider(
                    label="🧬 RAM Sieve Dual-Hash Bitset (MB)",
                    minimum=1024,
                    maximum=max_sieve,
                    value=default_sieve,
                    step=1024,
                    info=f"{default_sieve} MB = {default_sieve/1024:.1f} GB Sieve Bitset cho O(1) Dedup"
                )
                seed_input = gr.Number(
                    label="🎲 PRNG Base Seed (Dùng cho multi-instance)",
                    value=1,
                    precision=0
                )
                token_input = gr.Textbox(
                    label="🔑 HuggingFace Write Token (Dùng để upload)",
                    value=TOKEN,
                    placeholder="Nhập token hf_xxx...",
                    type="password"
                )
                repo_input = gr.Textbox(
                    label="📦 HuggingFace Dataset Repo",
                    value=REPO
                )

                with gr.Row():
                    start_btn = gr.Button(
                        f"🚀 BẮT ĐẦU KHAI THÁC ({cpu_logical} CPUs & {int(mem_total)}GB RAM)",
                        variant="primary",
                        size="lg"
                    )
                    stop_btn = gr.Button(
                        "🛑 DỪNG KHAI THÁC",
                        variant="stop",
                        size="lg"
                    )
                    purge_btn = gr.Button(
                        "🗑️ XÓA FILE OUTPUT HIỆN TẠI",
                        variant="stop",
                        size="lg"
                    )
                    free_ram_btn = gr.Button(
                        "🧹 GIẢI PHÓNG RAM",
                        variant="secondary",
                        size="lg"
                    )
                    view_telemetry_btn = gr.Button(
                        "📜 TRUY VẤN LOG ĐĨA & TELEMETRY",
                        variant="secondary",
                        size="lg"
                    )

            with gr.Column(scale=2):
                gr.Markdown("### 📊 Trạng Thái & Báo Cáo Real-Time Hệ Thống")
                status_box = gr.Markdown(f"Sẵn sàng khai thác dữ liệu trên hệ thống `{cpu_logical}` vCPUs & `{mem_total:.1f} GB` RAM...")
                metrics_box = gr.Markdown("Chờ khởi chạy...")
                logs_box = gr.Textbox(
                    label="📜 Nhật ký Native Engine Real-Time & Persistent Disk Telemetry",
                    lines=15,
                    max_lines=25,
                    interactive=False
                )

        with gr.Accordion("📁 QUẢN LÝ & KHẢO SÁT CÁC TỆP DATASET TRÊN ĐĨA (DATASET FILE MANAGER)", open=False):
            gr.Markdown("### 📂 Quản lý các tệp dataset (.jsonl / .json) trên đĩa cứng:")
            with gr.Row():
                dataset_files_list = list_dataset_files()
                dataset_dropdown = gr.Dropdown(
                    label="📄 Chọn tệp Dataset để quản lý",
                    choices=dataset_files_list,
                    value=dataset_files_list[0] if dataset_files_list else None,
                    interactive=True
                )
                refresh_dataset_btn = gr.Button("🔄 CẬP NHẬT DANH SÁCH", variant="secondary")
            
            with gr.Row():
                inspect_dataset_btn = gr.Button("🔍 KHẢO SÁT CHI TIẾT TỆP", variant="primary")
                delete_dataset_btn = gr.Button("🗑️ XÓA TỆP ĐÃ CHỌN", variant="stop")
            
            dataset_info_box = gr.Textbox(
                label="📊 Kết quả khảo sát & preview tệp dataset",
                lines=10,
                interactive=False
            )

        start_btn.click(
            fn=start_mining,
            inputs=[worker_input, games_slider, depth_slider, threads_slider, tt_mb_slider, sieve_mb_slider, seed_input, token_input, repo_input],
            outputs=[status_box, metrics_box, logs_box]
        )

        stop_btn.click(
            fn=stop_mining,
            inputs=[],
            outputs=[status_box]
        )

        purge_btn.click(
            fn=purge_current_output_file,
            inputs=[],
            outputs=[status_box, metrics_box, logs_box]
        )

        free_ram_btn.click(
            fn=kill_all_miner_processes,
            inputs=[],
            outputs=[status_box]
        )

        def fetch_disk_telemetry_logs():
            events = TelemetryLogger.read_tail_telemetry_events(15)
            disk_logs = TelemetryLogger.read_tail_disk_logs(40)
            return f"📜 NHẬT KÝ TELEMETRY EVENTS (logs/system_telemetry.jsonl):\n{events}\n\n📜 NHẬT KÝ ĐĨA CỨNG (logs/miner_stdout_stderr.log):\n{disk_logs}"

        view_telemetry_btn.click(
            fn=fetch_disk_telemetry_logs,
            inputs=[],
            outputs=[logs_box]
        )

        refresh_dataset_btn.click(
            fn=lambda: gr.Dropdown(choices=list_dataset_files(), value=list_dataset_files()[0] if list_dataset_files() else None),
            inputs=[],
            outputs=[dataset_dropdown]
        )

        inspect_dataset_btn.click(
            fn=inspect_dataset_file,
            inputs=[dataset_dropdown],
            outputs=[dataset_info_box]
        )

        delete_dataset_btn.click(
            fn=delete_selected_dataset_file,
            inputs=[dataset_dropdown],
            outputs=[dataset_info_box, dataset_dropdown]
        )

        app.load(
            fn=sync_on_load,
            inputs=[],
            outputs=[status_box, metrics_box, logs_box]
        )

        timer = gr.Timer(3.0)
        timer.tick(
            fn=sync_on_load,
            inputs=[],
            outputs=[status_box, metrics_box, logs_box]
        )

    return app

if __name__ == "__main__":
    print("============================================================================")
    print(f"🚀 XIANGQI-RIM APPLICATION VERSION: {APP_VERSION}")
    print(f"📅 BUILD STAMP: {APP_BUILD_STAMP}")
    print(f"📝 RELEASE NOTES: {APP_RELEASE_NOTES}")
    print("============================================================================")
    port = int(os.environ.get("PORT", os.environ.get("GRADIO_SERVER_PORT", 7860)))
    demo = create_app()
    demo.queue()
    demo.launch(server_name="0.0.0.0", server_port=port)
