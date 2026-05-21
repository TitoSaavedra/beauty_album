# BDO Beauty Album

Desktop viewer for Black Desert Online beauty presets. Browses local preset archives, displays images and metadata, syncs new presets from Garmoth automatically, and injects customization files into the game output directory.

Built with Tauri 2.0, Rust, and Svelte 4. No Python or external runtime required.

---

## Stack

- **Shell**: Tauri 2.0
- **Backend**: Rust 2021
- **Frontend**: Svelte 4 + TypeScript + Vite 5
- **Scraper**: Rust + playwright-rs (Chromium, headless)
- **Styling**: Tailwind CSS v3

---

## Prerequisites

- [Rust](https://rustup.rs) stable with the MSVC toolchain (`rustup default stable-msvc`)
- [Node.js](https://nodejs.org) ≥ 18 + pnpm (`npm i -g pnpm`)
- [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (pre-installed on Windows 11)
- [MSVC Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the C++ workload

---

## Setup

```bash
pnpm install
```

Playwright-rs downloads Chromium automatically on first run. No manual browser install needed.

---

## Development

```bash
pnpm tauri dev
```

Starts the Vite dev server and the Tauri window.

---

## Production build

```bash
pnpm tauri build
# or launch immediately after build:
pnpm run prod
```

Runs in sequence:
1. `vite build` — compiles the Svelte frontend
2. `cargo tauri build` — compiles Rust, produces MSI and NSIS installers

Output:

| Artifact | Path |
|---|---|
| Executable | `src-tauri/target/release/beauty-album.exe` |
| MSI | `src-tauri/target/release/bundle/msi/` |
| NSIS | `src-tauri/target/release/bundle/nsis/` |

The installed app is fully self-contained — no Python, Node, or external runtime required on the target machine.

---

## Configuration

On first launch, open Settings (⚙ top-right) and set the **BDO Documents Directory** — the root folder where Black Desert Online writes its user files (e.g. `C:\Users\<user>\Documents\Black Desert`).

All app directories are created automatically and derived from this single path:

| Directory | Path |
|---|---|
| Preset archive | `<bdo_docs_dir>/Beauty Album/Presets/` |
| Customization output | `<bdo_docs_dir>/Customization/` |
| Preset input (drop zone) | `<bdo_docs_dir>/Beauty Album/to_download/` |
| Class data | `<bdo_docs_dir>/Beauty Album/Classes/` |
| Logs | `<bdo_docs_dir>/Beauty Album/Logs/` |

Config is saved to `%APPDATA%\com.bdo.beauty-album\config.json`.

---

## How it works

**Sync pipeline** — triggered automatically on startup or when new `.pab` files are dropped into the input directory:

1. **Metadata phase** — all preset JSONs are fetched concurrently from the Garmoth API via `reqwest`. Skeleton cards appear in the grid immediately as each fetch completes.
2. **Image phase** — `image_1` for each preset is downloaded sequentially through a headless Chromium session (bypasses Cloudflare fingerprinting). Cards flip from skeleton to real thumbnail as each image arrives.
3. **image_2 pass** — after all `image_1` downloads complete, a silent background pass fetches secondary images without updating the progress bar.

**Class initialization** — on first launch, Rust downloads Garmoth's JS bundle through Chromium, parses all class definitions, and downloads class icons. This runs once and is skipped on subsequent launches.

**Directory watcher** — polls the input directory every 20 seconds for new `.pab` files and starts the sync automatically when files appear.

**Cloudflare bypass** — all asset and API requests that require browser-level TLS fingerprinting go through playwright-rs (Chromium with BoringSSL). Plain `reqwest` is used only for the Garmoth JSON API where fingerprinting is not enforced.

All events are logged to `tauri.log` in the Logs directory with structured `[TAG ]` prefixes (`[SYNC ]`, `[META ]`, `[CLASS]`, `[WATCH]`, `[USER ]`, `[ERR  ]`). Open from Settings → Open Logs.

---

## License

MIT
