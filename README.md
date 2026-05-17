# BDO Beauty Album

A desktop viewer for Black Desert Online beauty presets, built with **Tauri 2.0**, **Rust**, and **Svelte 4**.

Browse preset classes, view images in a lightbox, and download customization files — all from a native desktop app.

---

## Tech Stack

| Layer | Technology |
|---|---|
| Desktop shell | Tauri 2.0 |
| Backend | Rust (2021 edition) |
| Frontend | Svelte 4 + TypeScript |
| Build tool | Vite 5 |
| Styling | Tailwind CSS v3 + CSS custom properties |
| Package manager | pnpm |

---

## Prerequisites

| Tool | Version | Notes |
|---|---|---|
| [Rust](https://rustup.rs) | stable (≥ 1.77) | Install via `rustup` with the MSVC toolchain on Windows |
| [Node.js](https://nodejs.org) | ≥ 18 | LTS recommended |
| [pnpm](https://pnpm.io) | ≥ 9 | `npm install -g pnpm` |
| [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) | — | Pre-installed on Windows 11; required at runtime |

> **Windows users**: make sure the [MSVC Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (C++ workload) are installed before running `rustup`.

---

## Getting Started

### 1. Clone the repository

```bash
git clone https://github.com/your-user/beauty-album.git
cd beauty-album
```

### 2. Install JavaScript dependencies

```bash
pnpm install
```

### 3. Run in development mode

Starts the Vite dev server and the Tauri window with hot-reload:

```bash
pnpm run tauri dev
```

The app opens automatically. Changes to Svelte/TS files reload the UI instantly; Rust changes trigger a recompile.

---

## Build

### Production desktop app

Compiles the frontend, links the Rust binary, and packages both MSI and NSIS installers:

```bash
pnpm run tauri build
```

Output artifacts:

| Format | Path |
|---|---|
| Executable | `src-tauri/target/release/beauty-album.exe` |
| MSI installer | `src-tauri/target/release/bundle/msi/BDO Beauty Album_<version>_x64_en-US.msi` |
| NSIS installer | `src-tauri/target/release/bundle/nsis/BDO Beauty Album_<version>_x64-setup.exe` |

### Frontend only (no Tauri)

```bash
pnpm run build
```

Output goes to `dist/`.

### Preview the production frontend

```bash
pnpm run preview
```

---

## Rust Commands

Run these from inside the `src-tauri/` directory, or prefix with `--manifest-path src-tauri/Cargo.toml`.

```bash
# Type-check without producing a binary (fast)
cargo check --manifest-path src-tauri/Cargo.toml

# Compile in debug mode
cargo build --manifest-path src-tauri/Cargo.toml

# Compile in release mode
cargo build --release --manifest-path src-tauri/Cargo.toml

# Run lints
cargo clippy --manifest-path src-tauri/Cargo.toml

# Auto-format Rust source
cargo fmt --manifest-path src-tauri/Cargo.toml
```

---

## Project Structure

```
beauty-album/
├── src/                        # Svelte frontend
│   ├── index.html
│   ├── main.ts
│   ├── App.svelte
│   ├── components/
│   │   ├── Lightbox.svelte
│   │   └── SettingsModal.svelte
│   ├── features/
│   │   └── album/
│   │       ├── ClassList.svelte
│   │       ├── PresetCard.svelte
│   │       └── PresetGrid.svelte
│   ├── stores/
│   │   └── lightbox.ts
│   ├── styles/
│   │   └── app.css
│   └── tauri/
│       └── album.ts            # Typed IPC wrappers
│
├── src-tauri/                  # Rust / Tauri backend
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   ├── icons/
│   │   └── icon.ico
│   └── src/
│       ├── lib.rs
│       ├── main.rs
│       ├── commands/
│       │   ├── album.rs        # get_classes, get_presets, open_file
│       │   └── config.rs       # get_config, save_config
│       ├── errors/
│       │   └── mod.rs
│       ├── services/
│       │   ├── album_service.rs
│       │   └── config_service.rs
│       └── state/
│           └── mod.rs          # AppConfig, AppState
│
├── vite.config.ts
├── tailwind.config.js
├── postcss.config.js
└── package.json
```

---

## Configuration

On first launch, click the **⚙** button in the top-right corner to open Settings:

| Field | Description |
|---|---|
| **Album Directory** | Path to the `beauty_album` folder containing class subdirectories |
| **BDO Output Directory** | Path to the Black Desert Online output folder |

Settings are saved to `config.json` in the OS app-config directory:

| OS | Path |
|---|---|
| Windows | `%APPDATA%\com.bdo.beauty-album\config.json` |
| macOS | `~/Library/Application Support/com.bdo.beauty-album/config.json` |
| Linux | `~/.config/com.bdo.beauty-album/config.json` |

---

## IPC Commands

| Command | Description |
|---|---|
| `get_config` | Returns the current `AppConfig` |
| `save_config` | Persists `AppConfig` to disk and updates runtime state |
| `get_classes` | Lists class directories in the album folder |
| `get_presets` | Lists presets for a given class, sorted by downloads |
| `open_file` | Opens a file with the system default application |

---

## License

MIT
