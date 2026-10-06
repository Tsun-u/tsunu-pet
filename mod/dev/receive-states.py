"""開發用的總機收件匣：收 mod 送來的狀態，印出 session 名稱與狀態，並寫進 states.log。

用法：python receive-states.py
"""
import json
import pathlib
from datetime import datetime
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

PORT = 47321
SESSIONS_DIR = pathlib.Path.home() / ".claude" / "sessions"
LOG_FILE = pathlib.Path(__file__).with_name("states.log")


def session_name(session_id: str) -> str:
    """Claude Code 把每個 session 的名字寫在 sessions/<pid>.json。"""
    for path in SESSIONS_DIR.glob("*.json"):
        try:
            info = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, ValueError):
            continue
        if info.get("sessionId") == session_id:
            return info.get("name") or session_id[:8]
    return session_id[:8]


class StateHandler(BaseHTTPRequestHandler):
    def do_POST(self):
        body = self.rfile.read(int(self.headers.get("content-length", 0)))
        report = json.loads(body)
        name = session_name(report.get("sessionId", ""))
        line = f"{datetime.now():%H:%M:%S} {name:<16} {report.get('state'):<9} {report.get('detail', '')}"
        print(line, flush=True)
        with LOG_FILE.open("a", encoding="utf-8") as log:
            log.write(line + "\n")
        self.send_response(204)
        self.end_headers()

    def log_message(self, *args):
        pass


if __name__ == "__main__":
    print(f"總機收件匣在 127.0.0.1:{PORT} 等狀態…", flush=True)
    ThreadingHTTPServer(("127.0.0.1", PORT), StateHandler).serve_forever()
