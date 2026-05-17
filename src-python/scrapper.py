from __future__ import annotations

import asyncio
import json
import logging
import os
import re
import shutil
from datetime import datetime, timezone
from pathlib import Path

from playwright.async_api import async_playwright, Error as PlaywrightError

GARMOTH_BASE_URL = "https://garmoth.com/beauty-album/preset"
OK_FILENAME = ".ok"
RECENT_DAYS = 14

log = logging.getLogger("scrapper")


def _setup_log(album_dir: Path) -> None:
    if log.handlers:
        return
    album_dir.mkdir(parents=True, exist_ok=True)
    handler = logging.FileHandler(album_dir / "scrapper.log", encoding="utf-8")
    handler.setFormatter(logging.Formatter("%(asctime)s [%(levelname)-5s] %(message)s"))
    log.addHandler(handler)
    log.setLevel(logging.INFO)


# ── Helpers ────────────────────────────────────────────────────────────────────

def _scan_local_presets(input_dir: Path) -> dict[str, dict]:
    presets: dict[str, dict] = {}
    if not input_dir.exists():
        return presets
    for file in input_dir.rglob("*"):
        if file.is_file():
            m = re.search(r"^([^_]+)_.*?ID(\d+)", file.name)
            if m:
                presets[m.group(2).strip()] = {
                    "class": m.group(1).strip().replace("_", " "),
                    "path": file,
                }
    return presets


def _preset_age_days(album_dir: Path, preset_id: str) -> int | None:
    if not album_dir.exists():
        return None
    for class_dir in album_dir.iterdir():
        if not class_dir.is_dir():
            continue
        json_path = class_dir / preset_id / f"{preset_id}.json"
        if not json_path.exists():
            continue
        try:
            data = json.loads(json_path.read_text(encoding="utf-8"))
            ts = data.get("updated_at")
            if not ts:
                continue
            dt = datetime.fromisoformat(ts.replace("Z", "+00:00"))
            return (datetime.now(timezone.utc) - dt).days
        except Exception:
            continue
    return None


def _check_integrity(album_dir: Path, preset_id: str, class_hint: str | None) -> tuple[bool, Path | None]:
    dirs: list[Path] = []
    if class_hint:
        dirs.append(album_dir / class_hint.replace("_", " ") / preset_id)
    elif album_dir.exists():
        dirs = [d / preset_id for d in album_dir.iterdir() if d.is_dir()]

    for p_dir in dirs:
        json_path = p_dir / f"{preset_id}.json"
        if not json_path.exists():
            continue
        try:
            data = json.loads(json_path.read_text(encoding="utf-8"))
            cf = data.get("customization_file")
            if not (cf and (p_dir / cf).exists()):
                continue
            if any(
                x.suffix.lower() in {".png", ".jpg", ".jpeg"} and "icon" not in x.name
                for x in p_dir.iterdir()
            ):
                return True, p_dir
        except Exception:
            pass
    return False, None


def _link_local_file(preset_dir: Path, preset_id: str, local_info: dict | None) -> None:
    if not local_info or "path" not in local_info:
        return
    src: Path = local_info["path"]
    dest = preset_dir / src.name
    if not dest.exists():
        shutil.copy2(src, dest)
    json_path = preset_dir / f"{preset_id}.json"
    if json_path.exists():
        try:
            data = json.loads(json_path.read_text(encoding="utf-8"))
            data["customization_file"] = src.name
            json_path.write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8")
        except Exception:
            pass


# ── Preset processing ──────────────────────────────────────────────────────────

async def _process_preset(page, album_dir: Path, preset_id: str, local_info: dict | None) -> None:
    class_hint: str | None = local_info["class"] if local_info else None
    is_valid, preset_dir = _check_integrity(album_dir, preset_id, class_hint)

    if not preset_dir:
        safe = (class_hint or "Unknown").replace("_", " ")
        preset_dir = album_dir / safe / preset_id
    preset_dir.mkdir(parents=True, exist_ok=True)

    captured: dict[str, bytes] = {}

    async def on_response(response) -> None:
        try:
            url = response.url
            if "classes/svg/" in url or (not is_valid and "beauty-album/images/" in url):
                if response.status == 200:
                    captured[url] = await response.body()
        except Exception:
            pass

    page.on("response", on_response)

    try:
        await page.goto(f"{GARMOTH_BASE_URL}/{preset_id}", wait_until="domcontentloaded")
        await page.wait_for_selector("h1.text-xl", timeout=15000)
        try:
            await page.wait_for_selector("img[src*='beauty-album/images/']", timeout=15000)
        except PlaywrightError:
            pass
        await page.wait_for_timeout(4000)
    except PlaywrightError as exc:
        page.remove_listener("response", on_response)
        raise RuntimeError(f"Page load failed: {exc}") from exc

    h1 = await page.locator("h1.text-xl").first.inner_text()
    if "|" in h1:
        title, class_name = (p.strip() for p in h1.split("|", 1))
    else:
        title, class_name = h1.strip(), class_hint or "Unknown"

    class_safe = re.sub(r"[^A-Za-z0-9 ]+", " ", class_name).strip()
    preset_dir = album_dir / class_safe / preset_id
    preset_dir.mkdir(parents=True, exist_ok=True)

    creator = ""
    el = page.locator("h2.text-sm.text-100").first
    if await el.count() > 0:
        creator = (await el.inner_text()).strip()

    date_text = ""
    el = page.locator("div.flex.gap-2.text-sm > div.text-200").first
    if await el.count() > 0:
        date_text = (await el.inner_text()).strip()

    stats_els = page.locator("div.mt-2.flex.items-center p")
    stats = [(await stats_els.nth(i).inner_text()).strip() for i in range(await stats_els.count())]

    def to_int(s: str) -> int:
        return int(s.replace(",", "")) if s.replace(",", "").isdigit() else 0

    downloads = to_int(stats[0]) if len(stats) > 0 else 0
    views     = to_int(stats[1]) if len(stats) > 1 else 0
    favorites = to_int(stats[2]) if len(stats) > 2 else 0

    el = page.locator('img[src*="classes/svg/"]').first
    if await el.count() > 0:
        icon_url = await el.get_attribute("src")
        if icon_url and icon_url in captured:
            icon_path = album_dir / class_safe / "icon.svg"
            if not icon_path.exists():
                icon_path.parent.mkdir(parents=True, exist_ok=True)
                icon_path.write_bytes(captured[icon_url])

    updated_at = datetime.now(timezone.utc).isoformat()
    json_path = preset_dir / f"{preset_id}.json"

    if is_valid:
        try:
            data = json.loads(json_path.read_text(encoding="utf-8"))
        except Exception:
            data = {}
        data.update({
            "class": class_name, "title": title, "creator": creator,
            "date": date_text, "downloads": downloads, "views": views,
            "favorites": favorites, "updated_at": updated_at,
        })
        json_path.write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8")
        (preset_dir / OK_FILENAME).touch()
        page.remove_listener("response", on_response)
        return

    images: list[str] = []
    img_els = page.locator('img[src*="beauty-album/images/"]')
    for i in range(await img_els.count()):
        img_url = await img_els.nth(i).get_attribute("src")
        if img_url and img_url in captured:
            fname = img_url.split("/")[-1].split("?")[0]
            (preset_dir / fname).write_bytes(captured[img_url])
            images.append(fname)

    data = {
        "class": class_name, "id": int(preset_id), "title": title,
        "creator": creator, "date": date_text, "downloads": downloads,
        "views": views, "favorites": favorites, "images": images,
        "updated_at": updated_at,
    }
    json_path.write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8")
    _link_local_file(preset_dir, preset_id, local_info)
    (preset_dir / OK_FILENAME).touch()
    page.remove_listener("response", on_response)


# ── Public entry point ─────────────────────────────────────────────────────────

async def run_scraper(
    album_dir: Path,
    input_dir: Path,
    queue: asyncio.Queue,
    cancel: asyncio.Event,
) -> None:
    _setup_log(album_dir)

    local_presets = _scan_local_presets(input_dir)
    candidates = list(local_presets.items())

    n_pre_skip = 0
    to_process: list[tuple[str, dict | None]] = []
    for preset_id, info in candidates:
        age = _preset_age_days(album_dir, preset_id)
        if age is not None and age < RECENT_DAYS:
            log.info("[%s] Updated %d day(s) ago — skipping", preset_id, age)
            await queue.put({"type": "progress", "preset_id": preset_id, "status": "skip",
                             "message": f"Updated {age} day(s) ago — skipping",
                             "current": 0, "total": 0})
            n_pre_skip += 1
        else:
            to_process.append((preset_id, info))

    total = len(to_process)
    log.info("%d to process, %d pre-filtered", total, n_pre_skip)

    if not to_process:
        await queue.put({"type": "done", "n_done": 0, "n_skip": n_pre_skip, "n_error": 0})
        return

    n_done = n_error = 0

    local_appdata = os.environ.get("LOCALAPPDATA", str(Path.home() / "AppData" / "Local"))
    firefox_exe = Path(local_appdata) / "ms-playwright" / "firefox-1447" / "firefox" / "firefox.exe"

    async with async_playwright() as p:
        browser = await p.firefox.launch(
            headless=True,
            executable_path=str(firefox_exe) if firefox_exe.exists() else None,
        )
        page = await browser.new_page()

        for idx, (preset_id, info) in enumerate(to_process):
            if cancel.is_set():
                log.info("Cancelled by client")
                await queue.put({"type": "progress", "preset_id": "", "status": "cancelled",
                                 "message": "Scrapping cancelled", "current": idx, "total": total})
                break

            current = idx + 1
            log.info("[%s] Processing (%d/%d)", preset_id, current, total)
            await queue.put({"type": "progress", "preset_id": preset_id, "status": "processing",
                             "message": f"[{current}/{total}] Processing {preset_id}",
                             "current": current, "total": total})
            try:
                await _process_preset(page, album_dir, preset_id, info)
                n_done += 1
                log.info("[%s] Done", preset_id)
                await queue.put({"type": "progress", "preset_id": preset_id, "status": "done",
                                 "message": f"Preset {preset_id} complete",
                                 "current": current, "total": total})
            except Exception as exc:
                n_error += 1
                log.error("[%s] Error: %s", preset_id, exc)
                await queue.put({"type": "progress", "preset_id": preset_id, "status": "error",
                                 "message": str(exc), "current": current, "total": total})

            await asyncio.sleep(1.0)

        await browser.close()

    await queue.put({"type": "done", "n_done": n_done, "n_skip": n_pre_skip, "n_error": n_error})
    log.info("Finished — %d done  %d skipped  %d error(s)", n_done, n_pre_skip, n_error)
