# BDO Beauty Album

Desktop viewer for Black Desert Online beauty presets. Browses local preset archives, displays images and metadata, syncs new presets from Garmoth automatically, and injects customization files into the game output directory.

Built with Tauri 2.0, Rust, Svelte 4, and a bundled Python/FastAPI scraper.

---

## Stack

- **Shell**: Tauri 2.0
- **Backend**: Rust 2021
- **Frontend**: Svelte 4 + TypeScript + Vite 5
- **Scraper**: Python 3.11, FastAPI, Playwright (Firefox)
- **Styling**: Tailwind CSS v3

---

## Prerequisites

- [Rust](https://rustup.rs) stable with the MSVC toolchain (`rustup default stable-msvc`)
- [Node.js](https://nodejs.org) ≥ 18 + pnpm (`npm i -g pnpm`)
- [Python](https://www.python.org) 3.11
- [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (pre-installed on Windows 11)
- [MSVC Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the C++ workload

---

## Setup

```bash
# 1. Install JS dependencies
pnpm install

# 2. Install Python dependencies
pip install -r requirements.txt

# 3. Install the Playwright Firefox browser (one-time)
playwright install firefox

# 4. Build the bundled Python server (required before first cargo build/check)
npm run build:python
```

---

## Development

```bash
pnpm tauri dev
```

Starts the Vite dev server and the Tauri window. The Rust backend spawns `src-python/main.py` directly via the system Python interpreter.

---

## Production build

```bash
pnpm tauri build
```

Runs in sequence:
1. `vite build` — compiles the Svelte frontend
2. `npm run build:python` — PyInstaller bundles the Python server into `src-tauri/binaries/server-x86_64-pc-windows-msvc.exe`
3. `cargo tauri build` — compiles Rust, links everything, produces MSI and NSIS installers

Output:

| Artifact | Path |
|---|---|
| Executable | `src-tauri/target/release/beauty-album.exe` |
| MSI | `src-tauri/target/release/bundle/msi/` |
| NSIS | `src-tauri/target/release/bundle/nsis/` |

The installed app is self-contained — no Python or Node runtime required on the target machine. Firefox for Playwright must still be installed once (`playwright install firefox`).

---

## Configuration

On first launch, open Settings (⚙ top-right) and set the **BDO Documents Directory** — the root folder where Black Desert Online writes its user files (e.g. `C:\Users\<user>\Documents\Black Desert`).

All app directories are derived from this single path:

| Directory | Path |
|---|---|
| Preset archive | `<bdo_docs_dir>/Beauty Album/Presets/` |
| Customization output | `<bdo_docs_dir>/Customization/` |
| Preset input (drop zone) | `<bdo_docs_dir>/Beauty Album/to_download/` |
| Logs | `<bdo_docs_dir>/Beauty Album/Logs/` |

Config is saved to `%APPDATA%\com.bdo.beauty-album\config.json`.

---

## How it works

- On startup, Rust checks for pending (unsynced) presets and starts the scraper automatically if any are found.
- A background watcher polls the input directory every 20 seconds for new `.pab` files. When files appear, the scraper starts and a toast notification is shown.
- The scraper communicates with a local FastAPI server (port 8765) via HTTP + SSE. Progress is relayed to the UI in real time — the preset grid updates after each individual preset completes.
- All Rust-side events are logged to `tauri.log` in the Logs directory with structured `[TAG ]` prefixes (`[SYNC ]`, `[WATCH]`, `[USER ]`, `[ERR  ]`). Open them from Settings → Open Logs.

---

## License

MIT
