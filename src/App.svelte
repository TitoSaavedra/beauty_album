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

  interface ScrapperProgress {
    preset_id: string;
    status: string;
    message: string;
    current: number;
    total: number;
  }

  let config: AppConfig = { album_dir: '', bdo_output_dir: '', album_input_dir: '' };
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

  onMount(async () => {
    try {
      config = await api.getConfig();
      appConfig.set(config);
      if (config.album_dir) await loadClasses();
    } catch { /* config not yet saved */ }
  });

  async function selectClass(name: string) {
    selectedClass = name;
    presets = [];
    presetsError = '';
    loadingPresets = true;
    try {
      presets = await api.getPresets(name);
    } catch (err) {
      presetsError = String(err);
    } finally {
      loadingPresets = false;
    }
  }

  async function loadClasses() {
    loadingClasses = true;
    classesError = '';
    try {
      classes = await api.getClasses();
      if (classes.length > 0) await selectClass(classes[0].name);
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
    if (config.album_dir) await loadClasses();
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
      scrapperCurrent = payload.current;
      scrapperTotal = payload.total;
      scrapperMsg = payload.message;
      if (payload.status === 'error') scrapperError = payload.message;
      if (payload.status === 'cancelled') scrapperMsg = 'Cancelled';
    });

    try {
      await api.runScrapper();
      scrapperMsg = 'Done';
    } catch (e) {
      scrapperMsg = String(e);
      scrapperError = String(e);
    } finally {
      scrapperRunning = false;
      unlisten();
      if (config.album_dir) await loadClasses();
    }
  }

  async function stopScraper() {
    await api.stopScrapper();
  }
</script>

<div class="app">
  <nav class="nav glass-panel">
    <div class="nav-brand">
      <svg class="nav-icon" viewBox="0 0 512 512" xmlns="http://www.w3.org/2000/svg">
        <defs>
          <linearGradient id="ni-gem" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stop-color="#ffe566"/>
            <stop offset="100%" stop-color="#b3861b"/>
          </linearGradient>
        </defs>
        <polygon points="256,60 340,185 300,185 256,160 212,185 172,185" fill="url(#ni-gem)" opacity="0.95"/>
        <polygon points="172,185 212,185 192,290 152,420" fill="#b3861b" opacity="0.8"/>
        <polygon points="340,185 360,420 320,290 300,185" fill="#ffcc4d" opacity="0.75"/>
        <polygon points="212,185 300,185 256,430 " fill="#ffcc4d" opacity="0.6"/>
        <polygon points="256,60 278,145 256,135 234,145" fill="white" opacity="0.25"/>
      </svg>
      <span class="nav-title">Archive // Beauty Album</span>
    </div>
    <div class="global-stats">
      {#if selectedClass && !loadingPresets && presets.length > 0}
        TOTAL PRESETS: {presets.length}
      {/if}
    </div>
    <div class="nav-actions">
      {#if scrapperRunning}
        <button class="btn-stop" on:click={stopScraper} title="Stop scrapping">Stop</button>
      {:else}
        <button class="btn-sync" on:click={startScraper} title="Sync presets from Garmoth">Sync</button>
      {/if}
      <button class="btn-settings" on:click={() => (settingsOpen = true)} title="Settings">⚙</button>
    </div>
  </nav>

  {#if scrapperRunning || scrapperMsg}
    <div class="scrapper-strip" class:has-error={!!scrapperError}>
      <div
        class="scrapper-fill"
        style="width: {scrapperTotal > 0 ? Math.round((scrapperCurrent / scrapperTotal) * 100) : 0}%"
      ></div>
      <span class="scrapper-label">
        {#if scrapperTotal > 0}
          {scrapperCurrent}/{scrapperTotal} —
        {/if}
        {scrapperMsg}
      </span>
    </div>
  {/if}

  <div class="main-layout">
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
</style>
