<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import * as api from './tauri/album';
  import type { ClassEntry, PresetEntry, AppConfig } from './tauri/album';
  import ClassList from './features/album/ClassList.svelte';
  import PresetGrid from './features/album/PresetGrid.svelte';
  import PresetDetail from './features/album/PresetDetail.svelte';
  import SettingsModal from './components/SettingsModal.svelte';
  import ConfirmDialog from './components/ConfirmDialog.svelte';
  import ToastContainer from './components/ToastContainer.svelte';
  import { appConfig } from './stores/appConfig';
  import { toast } from './stores/toast';

  interface ScrapperProgress {
    preset_id: string;
    status: string;
    message: string;
    class_name: string;
    current: number;
    total: number;
  }

  let config: AppConfig = { bdo_docs_dir: '', cf_clearance: '' };
  let useTestDir = false;
  let scrapperRunning = false;
  let scrapperCurrent = 0;
  let scrapperTotal = 0;
  let scrapperMsg = '';
  let scrapperError = '';
  let settingsOpen = false;
  let classes: ClassEntry[] = [];
  let selectedClass: string | null = null;
  let presets: PresetEntry[] = [];
  let loadingClasses = false;
  let loadingPresets = false;
  let classesError = '';
  let presetsError = '';
  let classInitVisible = false;
  let classInitCurrent = 0;
  let classInitTotal = 0;
  let classInitName = '';

  onMount(async () => {
    listen('refresh_album', () => {
      if (config.bdo_docs_dir) loadClasses(true);
    });

    listen<{ current: number; total: number; class_name: string }>('class_init_progress', ({ payload }) => {
      classInitVisible = true;
      classInitCurrent = payload.current;
      classInitTotal = payload.total;
      classInitName = payload.class_name;
      toast.show(`Class ready: ${payload.class_name}`, 'success', 2000);
    });

    listen<string[]>('folder_changed', ({ payload: files }) => {
      const label = files.length === 1 ? files[0] : `${files.length} new presets`;
      toast.show(`New preset found: ${label}`, 'success', 4000);
      if (!scrapperRunning) startScraper();
    });

    try {
      config = await api.getConfig();
      appConfig.set(config);
      if (config.bdo_docs_dir) {
        try { await api.initClasses(); } finally { classInitVisible = false; }
        await loadClasses();
        const pending = await api.checkPending();
        if (pending > 0) startScraper();
      } else {
        settingsOpen = true;
      }
    } catch { /* config not yet saved */ }
  });

  async function selectClass(name: string) {
    selectedClass = name;
    presets = [];
    presetsError = '';
    loadingPresets = true;
    try {
      presets = await api.getPresets(name, useTestDir);
    } catch (err) {
      presetsError = String(err);
    } finally {
      loadingPresets = false;
    }
  }

  async function loadClasses(keepSelected = false) {
    loadingClasses = true;
    classesError = '';
    try {
      const prev = selectedClass;
      classes = await api.getClasses(useTestDir);
      const target = keepSelected && prev && classes.some(c => c.name === prev)
        ? prev
        : classes[0]?.name;
      if (!target) return;
      if (keepSelected && target === prev) {
        // Refresh in-place: Svelte only animates truly new preset_ids
        try { presets = await api.getPresets(target, useTestDir); } catch (err) { presetsError = String(err); }
      } else {
        await selectClass(target);
      }
    } catch (e) {
      classesError = String(e);
    } finally {
      loadingClasses = false;
    }
  }

  async function handleSettingsSave(e: CustomEvent<AppConfig>) {
    config = e.detail;
    appConfig.set(config);
    settingsOpen = false;
    selectedClass = null;
    presets = [];
    if (config.bdo_docs_dir) await loadClasses();
  }

  function handleSelectClass(e: CustomEvent<string>) {
    selectClass(e.detail);
  }

  async function startScraper() {
    if (scrapperRunning) return;
    scrapperRunning = true;
    scrapperCurrent = 0;
    scrapperTotal = 0;
    scrapperMsg = 'Starting...';
    scrapperError = '';

    const unlisten = await listen<ScrapperProgress>('scrapper_progress', ({ payload }) => {
      if (payload.total > 0) scrapperTotal = payload.total;
      if (payload.current > 0) scrapperCurrent = payload.current;
      if (payload.message) scrapperMsg = payload.message;

      if (payload.status === 'metadata' || payload.status === 'done') {
        if (config.bdo_docs_dir) loadClasses(true);
        if (payload.status === 'done') {
          toast.show(`${payload.class_name} #${payload.preset_id} downloaded`, 'success', 3000);
        }
      } else if (payload.status === 'error') {
        scrapperError = payload.message;
      } else if (payload.status === 'cancelled') {
        scrapperMsg = 'Cancelled';
      }
    });

    let alreadyRunning = false;
    try {
      const doneMsg = await api.runScrapper(useTestDir);
      if (doneMsg) toast.show(doneMsg, 'success', 6000);
    } catch (e) {
      const msg = String(e);
      if (msg.includes('409') || msg.includes('Conflict')) {
        alreadyRunning = true;
      } else {
        scrapperMsg = msg;
        scrapperError = msg;
      }
    } finally {
      unlisten();
      if (!alreadyRunning) {
        scrapperRunning = false;
        scrapperMsg = '';
        if (config.bdo_docs_dir) await loadClasses(true);
      }
    }
  }

  async function stopScraper() {
    await api.stopScrapper();
  }

  async function toggleDir() {
    useTestDir = !useTestDir;
    selectedClass = null;
    presets = [];
    if (config.bdo_docs_dir) await loadClasses();
  }

</script>

<div class="app">
  <nav class="nav glass-panel">
    <div class="nav-brand">
      <svg class="nav-icon" viewBox="0 0 512 512" xmlns="http://www.w3.org/2000/svg">
        <defs>
          <linearGradient id="ni-gem-top" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stop-color="#ffe566"/>
            <stop offset="100%" stop-color="#ffcc4d"/>
          </linearGradient>
          <linearGradient id="ni-gem-left" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" stop-color="#b3861b"/>
            <stop offset="100%" stop-color="#ffcc4d"/>
          </linearGradient>
          <linearGradient id="ni-gem-right" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" stop-color="#ffdd6b"/>
            <stop offset="100%" stop-color="#996600"/>
          </linearGradient>
        </defs>
        <rect width="512" height="512" rx="96" fill="#0a0c10"/>
        <rect x="3" y="3" width="506" height="506" rx="94" fill="none" stroke="#ffcc4d" stroke-width="1.5" opacity="0.18"/>
        <polygon points="256,78 340,198 300,198 256,168 212,198 172,198" fill="url(#ni-gem-top)" opacity="0.95"/>
        <polygon points="172,198 212,198 192,298 152,418" fill="url(#ni-gem-left)" opacity="0.82"/>
        <polygon points="340,198 360,418 320,298 300,198" fill="url(#ni-gem-right)" opacity="0.78"/>
        <polygon points="212,198 300,198 280,318 256,434 232,318" fill="#ffcc4d" opacity="0.65"/>
        <polygon points="152,418 192,298 232,318 256,434" fill="#b3861b" opacity="0.85"/>
        <polygon points="360,418 256,434 280,318 320,298" fill="#cc9922" opacity="0.80"/>
        <polygon points="256,78 282,155 256,145 230,155" fill="white" opacity="0.22"/>
        <circle cx="364" cy="128" r="3.5" fill="#ffcc4d" opacity="0.45"/>
        <circle cx="148" cy="142" r="5" fill="#ffcc4d" opacity="0.55"/>
      </svg>
      <span class="nav-title">Archive // Beauty Album</span>
    </div>
    <div class="global-stats">
      {#if classes.length > 0}
        TOTAL PRESETS: {classes.reduce((s, c) => s + c.preset_count, 0)}
      {/if}
    </div>
    <div class="nav-actions">
      {#if scrapperRunning && useTestDir}
        <button class="btn-stop" on:click={stopScraper} title="Stop">Stop</button>
      {:else}
        <button class="btn-sync" style={useTestDir ? '' : 'display:none'} on:click={startScraper} title="Sync presets from Garmoth">Sync</button>
        <button class="btn-dir-toggle" class:active={useTestDir} on:click={toggleDir} title="Toggle between Presets and Test directories">
          {useTestDir ? 'Test' : 'Presets'}
        </button>
      {/if}
      <button class="btn-settings" on:click={() => (settingsOpen = true)} title="Settings">⚙</button>
    </div>
  </nav>

  {#if scrapperRunning || scrapperMsg || classInitVisible}
    <div class="scrapper-strip" class:has-error={!!scrapperError}>
      <div
        class="scrapper-fill"
        style="width: {classInitVisible
          ? (classInitTotal > 0 ? Math.round((classInitCurrent / classInitTotal) * 100) : 0)
          : (scrapperTotal > 0 ? Math.round((scrapperCurrent / scrapperTotal) * 100) : 0)}%"
      ></div>
      <span class="scrapper-label">
        {#if classInitVisible}
          {classInitCurrent}/{classInitTotal} — {classInitName}
        {:else}
          {#if scrapperTotal > 0}{scrapperCurrent}/{scrapperTotal} —{/if}
          {scrapperMsg}
        {/if}
      </span>
    </div>
  {/if}

  <div class="main-layout">
    {#if classInitVisible}
      <div class="main-layout-skeleton"></div>
    {:else}
      <aside class="sidebar">
        <ClassList
          {classes}
          {selectedClass}
          loading={loadingClasses}
          error={classesError}
          on:select={handleSelectClass}
        />
      </aside>

      <main class="content custom-scroll">
        <PresetGrid
          {presets}
          {selectedClass}
          loading={loadingPresets}
          error={presetsError}
        />
      </main>
    {/if}
  </div>
</div>

<PresetDetail />
<ConfirmDialog />
<ToastContainer />

{#if settingsOpen}
  <SettingsModal
    {config}
    on:save={handleSettingsSave}
    on:close={() => (settingsOpen = false)}
  />
{/if}

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  .nav {
    height: 64px;
    min-height: 64px;
    border-bottom: 1px solid #1a232c;
    padding: 0 32px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    z-index: 20;
    flex-shrink: 0;
    border-radius: 0;
  }

  .nav-brand {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .nav-icon {
    width: 28px;
    height: 28px;
    flex-shrink: 0;
  }

  .nav-title {
    font-size: 13px;
    font-weight: 700;
    letter-spacing: 0.2em;
    color: #fff;
    text-transform: uppercase;
  }

  .global-stats {
    font-size: 11px;
    font-family: monospace;
    letter-spacing: 0.15em;
    color: #64748b;
    background: #070a0e;
    padding: 6px 16px;
    border-radius: 4px;
    border: 1px solid #1a232c;
    min-width: 160px;
    text-align: center;
    min-height: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .nav-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .btn-sync {
    padding: 0 14px;
    height: 34px;
    background: transparent;
    border: 1px solid rgba(184, 134, 11, 0.4);
    border-radius: 5px;
    color: #b3861b;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    cursor: pointer;
    transition: border-color 0.2s, color 0.2s, background 0.2s;
  }

  .btn-sync:hover:not(:disabled) {
    border-color: #ffcc4d;
    color: #ffcc4d;
    background: rgba(184, 134, 11, 0.08);
  }

  .btn-stop {
    padding: 0 14px;
    height: 34px;
    background: transparent;
    border: 1px solid rgba(220, 60, 60, 0.5);
    border-radius: 5px;
    color: #dc3c3c;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    cursor: pointer;
    transition: border-color 0.2s, color 0.2s, background 0.2s;
  }

  .btn-stop:hover {
    border-color: #ff5555;
    color: #ff5555;
    background: rgba(220, 60, 60, 0.08);
  }

  .scrapper-strip {
    position: relative;
    height: 28px;
    background: #0c1017;
    border-bottom: 1px solid #1a232c;
    display: flex;
    align-items: center;
    overflow: hidden;
    flex-shrink: 0;
  }

  .scrapper-strip.has-error .scrapper-fill {
    background: rgba(220, 60, 60, 0.35);
  }

  .scrapper-fill {
    position: absolute;
    inset: 0;
    background: rgba(184, 134, 11, 0.18);
    transition: width 0.3s ease;
  }

  .scrapper-label {
    position: relative;
    font-size: 10px;
    font-family: monospace;
    letter-spacing: 0.08em;
    color: #64748b;
    padding: 0 16px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .btn-dir-toggle {
    padding: 0 12px;
    height: 34px;
    background: transparent;
    border: 1px solid #1a232c;
    border-radius: 5px;
    color: #64748b;
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    cursor: pointer;
    transition: border-color 0.2s, color 0.2s, background 0.2s;
    min-width: 64px;
  }

  .btn-dir-toggle:hover {
    border-color: #64748b;
    color: #94a3b8;
  }

  .btn-dir-toggle.active {
    border-color: rgba(99, 102, 241, 0.55);
    color: #818cf8;
    background: rgba(99, 102, 241, 0.08);
  }

  .btn-settings {
    background: transparent;
    border: 1px solid #1a232c;
    border-radius: 5px;
    color: #64748b;
    font-size: 16px;
    width: 34px;
    height: 34px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: border-color 0.2s, color 0.2s;
  }

  .btn-settings:hover {
    border-color: #ffcc4d;
    color: #ffcc4d;
  }

  .main-layout {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .sidebar {
    width: 288px;
    border-right: 1px solid #1a232c;
    background: rgba(7, 10, 14, 0.9);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    height: 100%;
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 32px;
    position: relative;
    background-image: radial-gradient(rgba(255, 255, 255, 0.012) 1px, transparent 0);
    background-size: 24px 24px;
  }

  .main-layout-skeleton {
    flex: 1;
    background: linear-gradient(90deg, #070a0e 25%, #0f141a 50%, #070a0e 75%);
    background-size: 200% 100%;
    animation: shimmer 1.4s ease-in-out infinite;
  }

  @keyframes shimmer {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }
</style>
