from __future__ import annotations

import argparse
import asyncio
import json
import logging
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

import uvicorn
from fastapi import BackgroundTasks, FastAPI, HTTPException
from sse_starlette.sse import EventSourceResponse

# ── CLI args ───────────────────────────────────────────────────────────────────

parser = argparse.ArgumentParser()
parser.add_argument("--album-dir", type=Path, required=True)
parser.add_argument("--input-dir", type=Path, required=True)
parser.add_argument("--log-dir", type=Path, required=True)
args = parser.parse_args()

ALBUM_DIR = args.album_dir
INPUT_DIR = args.input_dir
LOG_DIR = args.log_dir

# ── Logging ────────────────────────────────────────────────────────────────────

LOG_DIR.mkdir(parents=True, exist_ok=True)

_file_handler = logging.FileHandler(LOG_DIR / "server.log", encoding="utf-8")
_file_handler.setFormatter(logging.Formatter("%(asctime)s [%(levelname)-5s] %(name)s — %(message)s"))

logging.root.setLevel(logging.INFO)
logging.root.addHandler(_file_handler)
# Keep uvicorn quiet on stdout (already going to file)
logging.getLogger("uvicorn.access").handlers = [_file_handler]
logging.getLogger("uvicorn.error").handlers = [_file_handler]

log = logging.getLogger("main")

# ── Session state ──────────────────────────────────────────────────────────────

_running = False
_cancel: asyncio.Event = asyncio.Event()
_queue: asyncio.Queue = asyncio.Queue()

# ── FastAPI app ────────────────────────────────────────────────────────────────

app = FastAPI()


@app.post("/scrape", status_code=202)
async def start_scrape(background_tasks: BackgroundTasks):
    global _running, _cancel, _queue

    if _running:
        raise HTTPException(status_code=409, detail="Scrape already in progress")

    _cancel = asyncio.Event()
    _queue = asyncio.Queue()
    _running = True
    log.info("Scrape started — album_dir=%s  input_dir=%s", ALBUM_DIR, INPUT_DIR)

    background_tasks.add_task(_run_scraper_task)
    return {"ok": True}


@app.get("/events")
async def event_stream():
    async def generator():
        while True:
            item = await _queue.get()
            yield {"data": json.dumps(item, ensure_ascii=False)}
            if item.get("type") in ("done", "cancelled"):
                break

    return EventSourceResponse(generator())


@app.post("/stop")
async def stop_scrape():
    _cancel.set()
    log.info("Stop requested")
    return {"ok": True}


@app.get("/pending")
async def pending_count():
    from scrapper import _scan_local_presets, _preset_age_days, RECENT_DAYS
    local = _scan_local_presets(INPUT_DIR)
    count = 0
    for pid in local:
        age = _preset_age_days(ALBUM_DIR, pid)
        if age is None or age >= RECENT_DAYS:
            count += 1
    return {"count": count}


@app.get("/health")
async def health():
    return {"ok": True}


# ── Background task ────────────────────────────────────────────────────────────

async def _run_scraper_task() -> None:
    global _running
    from scrapper import run_scraper
    try:
        await run_scraper(ALBUM_DIR, INPUT_DIR, _queue, _cancel, LOG_DIR)
    except Exception as exc:
        log.error("Scraper crashed: %s", exc)
        await _queue.put({"type": "done", "n_done": 0, "n_skip": 0, "n_error": 1})
    finally:
        _running = False


# ── Entry point ────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    log.info("Server starting on port 8765")
    uvicorn.run(
        app,
        host="127.0.0.1",
        port=8765,
        log_config=None,  # handled manually above
    )
