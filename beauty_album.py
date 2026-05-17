from __future__ import annotations

import asyncio
import json
import re
import shutil
from datetime import datetime, timedelta, timezone
from pathlib import Path

from playwright.async_api import async_playwright, Error as PlaywrightError

from shared.logger import setup_logger
from src.config import BEAUTY_ALBUM_OUTPUT_DIR, GARMOTH_BASE_URL, LOCAL_CUSTOMIZATION_DIR, OK_FILENAME

BEAUTY_ALBUM_OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

log = setup_logger("garmoth_beauty_album")

MANUAL_IDS: list[int] = []


def scan_local_presets() -> dict[str, dict]:
    presets: dict[str, dict] = {}
    if not LOCAL_CUSTOMIZATION_DIR.exists():
        log.warning("La ruta local %s no existe.", LOCAL_CUSTOMIZATION_DIR)
        return presets

    for file in LOCAL_CUSTOMIZATION_DIR.rglob("*"):
        if file.is_file():
            match = re.search(r'^([^_]+)_.*?ID(\d+)', file.name)
            if match:
                class_hint = match.group(1).strip().replace("_", " ")
                preset_id = match.group(2).strip()
                presets[preset_id] = {"class": class_hint, "path": file}

    log.info("Escaneo local completado: %d presets encontrados.", len(presets))
    return presets


def check_integrity(preset_id: str, class_hint: str | None = None) -> tuple[bool, Path | None]:
    dirs_to_check: list[Path] = []
    if class_hint:
        dirs_to_check.append(BEAUTY_ALBUM_OUTPUT_DIR / class_hint.replace("_", " ") / preset_id)
    else:
        for d in BEAUTY_ALBUM_OUTPUT_DIR.iterdir():
            if d.is_dir():
                dirs_to_check.append(d / preset_id)

    two_weeks_ago = datetime.now(timezone.utc) - timedelta(weeks=2)
    for p_dir in dirs_to_check:
        ok_file = p_dir / OK_FILENAME
        if ok_file.exists():
            mtime = datetime.fromtimestamp(ok_file.stat().st_mtime, tz=timezone.utc)
            if mtime < two_weeks_ago:
                continue
            json_path = p_dir / f"{preset_id}.json"
            if json_path.exists():
                try:
                    with open(json_path, "r", encoding="utf-8") as f:
                        data = json.load(f)
                    c_file = data.get("customization_file")
                    if c_file and (p_dir / c_file).exists():
                        images = [
                            x for x in p_dir.iterdir()
                            if x.suffix.lower() in {".png", ".jpg", ".jpeg"} and "icon" not in x.name
                        ]
                        if images:
                            return True, p_dir
                except Exception:
                    pass
    return False, None


def handle_local_file_copy(preset_dir: Path, preset_id: str, local_info: dict | None) -> None:
    if not local_info or "path" not in local_info:
        return
    src_path: Path = local_info["path"]
    dest_path = preset_dir / src_path.name
    if not dest_path.exists():
        shutil.copy2(src_path, dest_path)
        log.info("Copiado archivo binario local: %s", src_path.name)

    json_path = preset_dir / f"{preset_id}.json"
    if json_path.exists():
        try:
            with open(json_path, "r", encoding="utf-8") as f:
                data = json.load(f)
            data["customization_file"] = src_path.name
            with open(json_path, "w", encoding="utf-8") as f:
                json.dump(data, f, ensure_ascii=False, indent=2)
        except Exception as e:
            log.error("Error actualizando JSON: %s", e)


async def process_preset(page, preset_id: str, local_info: dict | None = None) -> None:
    class_hint: str | None = local_info["class"] if local_info else None
    is_valid, preset_dir = check_integrity(preset_id, class_hint)

    class_safe_name = class_hint.replace("_", " ") if class_hint else "Unknown"
    if not preset_dir:
        preset_dir = BEAUTY_ALBUM_OUTPUT_DIR / class_safe_name / preset_id
    preset_dir.mkdir(parents=True, exist_ok=True)

    url = f"{GARMOTH_BASE_URL}/{preset_id}"
    if is_valid:
        log.info("Preset %s OK. Saltando.", preset_id)
        return
    log.info("Abriendo Garmoth para preset %s: %s", preset_id, url)

    captured_responses: dict[str, bytes] = {}

    async def handle_response(response) -> None:
        try:
            r_url = response.url
            if "classes/svg/" in r_url or (not is_valid and "beauty-album/images/" in r_url):
                if response.status == 200:
                    captured_responses[r_url] = await response.body()
        except Exception:
            pass

    page.on("response", handle_response)

    try:
        await page.goto(url, wait_until="domcontentloaded")
        await page.wait_for_selector("h1.text-xl", timeout=15000)
        await page.wait_for_timeout(4000)
    except PlaywrightError:
        log.error("No se pudo cargar la pagina del preset %s", preset_id)
        page.remove_listener("response", handle_response)
        return

    h1_text = await page.locator("h1.text-xl").first.inner_text()
    if "|" in h1_text:
        title, class_name = [part.strip() for part in h1_text.split("|", 1)]
    else:
        title = h1_text.strip()
        class_name = class_hint or "Unknown"

    class_safe_name = re.sub(r"[^A-Za-z0-9]+", " ", class_name).strip()
    preset_dir = BEAUTY_ALBUM_OUTPUT_DIR / class_safe_name / preset_id
    preset_dir.mkdir(parents=True, exist_ok=True)

    creator = ""
    creator_el = page.locator("h2.text-sm.text-100").first
    if await creator_el.count() > 0:
        creator = (await creator_el.inner_text()).strip()

    date_text = ""
    date_el = page.locator(".text-200").first
    if await date_el.count() > 0:
        date_text = (await date_el.inner_text()).strip()

    stats: list[str] = []
    p_elements = page.locator("div.mt-2.flex.items-center p")
    for i in range(await p_elements.count()):
        stats.append((await p_elements.nth(i).inner_text()).strip())

    downloads = stats[0] if len(stats) > 0 else "0"
    views = stats[1] if len(stats) > 1 else "0"
    favorites = stats[2] if len(stats) > 2 else "0"

    icon_img_el = page.locator('img[src*="classes/svg/"]').first
    if await icon_img_el.count() > 0:
        icon_url = await icon_img_el.get_attribute("src")
        if icon_url and icon_url in captured_responses:
            class_icon_path = BEAUTY_ALBUM_OUTPUT_DIR / class_safe_name / "icon.svg"
            if not class_icon_path.exists():
                class_icon_path.parent.mkdir(parents=True, exist_ok=True)
                class_icon_path.write_bytes(captured_responses[icon_url])

    json_path = preset_dir / f"{preset_id}.json"

    if is_valid:
        try:
            with open(json_path, "r", encoding="utf-8") as f:
                data_json = json.load(f)
        except Exception:
            data_json = {}
        data_json.update({
            "class": class_name,
            "title": title,
            "creator": creator,
            "date": date_text,
            "downloads": int(downloads) if downloads.isdigit() else downloads,
            "views": int(views) if views.isdigit() else views,
            "favorites": int(favorites) if favorites.isdigit() else favorites,
        })
        with open(json_path, "w", encoding="utf-8") as f:
            json.dump(data_json, f, ensure_ascii=False, indent=2)
        log.info("Preset %s info actualizada.", preset_id)
        page.remove_listener("response", handle_response)
        return

    image_filenames: list[str] = []
    img_elements = page.locator('img[src*="beauty-album/images/"]')
    for i in range(await img_elements.count()):
        img_url = await img_elements.nth(i).get_attribute("src")
        if img_url and img_url in captured_responses:
            filename = img_url.split("/")[-1]
            (preset_dir / filename).write_bytes(captured_responses[img_url])
            image_filenames.append(filename)

    data_json = {
        "class": class_name,
        "id": int(preset_id),
        "title": title,
        "creator": creator,
        "date": date_text,
        "downloads": int(downloads) if downloads.isdigit() else downloads,
        "views": int(views) if views.isdigit() else views,
        "favorites": int(favorites) if favorites.isdigit() else favorites,
        "images": image_filenames,
    }

    with open(json_path, "w", encoding="utf-8") as f:
        json.dump(data_json, f, ensure_ascii=False, indent=2)

    handle_local_file_copy(preset_dir, preset_id, local_info)
    (preset_dir / OK_FILENAME).touch()
    log.info("Preset %s procesado con exito.", preset_id)
    page.remove_listener("response", handle_response)


async def run_scraper() -> None:
    local_presets = scan_local_presets()
    all_presets: list[tuple[str, dict | None]] = [
        (pid, info) for pid, info in local_presets.items()
    ]
    for pid in MANUAL_IDS:
        pid_str = str(pid)
        if pid_str not in local_presets:
            all_presets.append((pid_str, None))

    if not all_presets:
        log.info("No hay presets para procesar.")
        return

    async with async_playwright() as p:
        browser = await p.firefox.launch(headless=True)
        page = await browser.new_page()
        for preset_id, info in all_presets:
            await process_preset(page, preset_id, info)
            await asyncio.sleep(1.0)
        await browser.close()

    log.info("Proceso completado.")
