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
  import { presetDetail } from './stores/presetDetail';
  import { wantedPresets } from './stores/wantedPresets';

  interface ScrapperProgress {
    preset_id: string;
    status: string;
    message: string;
    class_name: string;
    current: number;
    total: number;
  }

  let config: AppConfig = { bdo_docs_dir: '', cf_clearance: '' };
  let scrapperRunning = false;
  let scrapperCurrent = 0;
  let scrapperTotal = 0;
  let scrapperMsg = '';
  let scrapperError = '';
  let scrapperPhase: 'presets' | 'images' | 'image2' | '' = '';
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
  let viewMode: 'presets' | 'popular' = 'presets';
  let popularRunning = false;
  let popularCurrent = 0;
  let popularTotal = 0;
  let popularMsg = '';
  let popularPhase: 'syncing' | 'images' | '' = '';
  let filterShowDownloaded = true;
  let filterSortBy: 'downloads' | 'views' | 'favorites' = 'downloads';
  let searchQuery = '';
  let appLoading = true;

  onMount(async () => {
    listen('refresh_album', () => {
      if (config.bdo_docs_dir) loadClasses(true);
    });

    listen<ScrapperProgress>('popular_progress', ({ payload }) => {
      if (payload.total > 0) popularTotal = payload.total;
      if (payload.current > 0) popularCurrent = payload.current;
      if (payload.message) popularMsg = payload.message;
      if (payload.status === 'metadata' || payload.status === 'done') {
        popularPhase = 'syncing';
        if (config.bdo_docs_dir) loadClasses(true);
      } else if (payload.status === 'additional_data') {
        popularPhase = 'images';
      }
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
      if (!scrapperRunning && !popularRunning) startScraper();
    });

    try {
      config = await api.getConfig();
      appConfig.set(config);
      if (config.bdo_docs_dir) {
        try { await api.initClasses(); } finally { classInitVisible = false; }
        await wantedPresets.load();
        await loadClasses();
        appLoading = false;
        const pending = await api.checkPending();
        if (pending > 0) await startScraper();
        await startPopularSync();
      } else {
        appLoading = false;
        settingsOpen = true;
      }
    } catch { appLoading = false; /* config not yet saved */ }
  });

  async function selectClass(name: string) {
    selectedClass = name;
    presets = [];
    presetsError = '';
    loadingPresets = true;
    try {
      presets = viewMode === 'popular'
        ? await api.getPopularPresets(name)
        : await api.getPresets(name);
    } catch (err) {
      presetsError = String(err);
    } finally {
      loadingPresets = false;
    }
  }

  async function loadClasses(keepSelected = false) {
    if (!keepSelected) loadingClasses = true;
    classesError = '';
    try {
      const prev = selectedClass;
      const fresh = viewMode === 'popular'
        ? await api.getPopularClasses()
        : await api.getClasses();
      if (!keepSelected || classes.length === 0) {
        classes = fresh;
      } else {
        const freshMap = new Map(fresh.map(c => [c.name, c]));
        classes = classes.map(c => freshMap.get(c.name) ?? c);
        const newOnes = fresh.filter(c => !classes.some(e => e.name === c.name));
        if (newOnes.length > 0) classes = [...classes, ...newOnes];
      }
      const target = keepSelected && prev && classes.some(c => c.name === prev)
        ? prev
        : classes[0]?.name;
      if (!target) return;
      if (keepSelected && target === prev) {
        try {
          presets = viewMode === 'popular'
            ? await api.getPopularPresets(target)
            : await api.getPresets(target, false);
        } catch (err) { presetsError = String(err); }
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

  async function toggleMode() {
    viewMode = viewMode === 'presets' ? 'popular' : 'presets';
    selectedClass = null;
    presets = [];
    if (config.bdo_docs_dir) await loadClasses();
  }

  async function startPopularSync() {
    if (popularRunning) return;
    popularRunning = true;
    popularCurrent = 0;
    popularTotal = 0;
    popularMsg = 'Starting...';
    popularPhase = 'syncing';
    try {
      const doneMsg = await api.syncPopular();
      if (doneMsg) toast.show(doneMsg, 'success', 6000);
      if (config.bdo_docs_dir) await loadClasses(true);
    } catch (e) {
      toast.show(String(e), 'error', 6000);
    } finally {
      popularRunning = false;
      popularMsg = '';
      popularTotal = 0;
      popularCurrent = 0;
      popularPhase = '';
    }
  }

  async function startScraper() {
    if (scrapperRunning) return;
    scrapperRunning = true;
    scrapperCurrent = 0;
    scrapperTotal = 0;
    scrapperMsg = 'Starting...';
    scrapperError = '';
    scrapperPhase = '';

    const unlisten = await listen<ScrapperProgress>('scrapper_progress', ({ payload }) => {
      if (payload.total > 0) scrapperTotal = payload.total;
      if (payload.current > 0) scrapperCurrent = payload.current;
      if (payload.message) scrapperMsg = payload.message;

      if (payload.status === 'metadata') {
        scrapperPhase = 'presets';
        if (config.bdo_docs_dir) loadClasses(true);
      } else if (payload.status === 'done') {
        scrapperPhase = 'images';
        if (config.bdo_docs_dir) loadClasses(true);
        const pid = payload.preset_id;
        const cls = payload.class_name;
        toast.show(`${cls} #${pid} downloaded`, 'success', 5000, async () => {
          const list = await api.getPresets(cls);
          const found = list.find(p => p.preset_id === pid);
          if (found) presetDetail.set(found);
        });
      } else if (payload.status === 'additional_data') {
        scrapperPhase = 'image2';
      } else if (payload.status === 'error') {
        scrapperError = payload.message;
      } else if (payload.status === 'cancelled') {
        scrapperMsg = 'Cancelled';
      }
    });

    let alreadyRunning = false;
    try {
      const doneMsg = await api.runScrapper();
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
        scrapperPhase = '';
        if (config.bdo_docs_dir) await loadClasses(true);
      }
    }
  }

  $: stripPhase = classInitVisible ? 'init'
    : popularRunning ? (popularPhase === 'images' ? 'popular-images' : 'popular')
    : scrapperPhase;

  $: stripLabel = (() => {
    if (classInitVisible) return `${classInitCurrent}/${classInitTotal} — ${classInitName}`;
    if (popularRunning) {
      const prefix = popularTotal > 0 ? `${popularCurrent}/${popularTotal} — ` : '';
      const phase = popularPhase === 'images' ? 'Downloading Popular Images' : 'Syncing Popular';
      return `${prefix}${phase}${popularMsg ? ' — ' + popularMsg : ''}`;
    }
    const prefix = scrapperTotal > 0 ? `${scrapperCurrent}/${scrapperTotal} — ` : '';
    const label = scrapperPhase === 'presets' ? 'Downloading Presets'
      : scrapperPhase === 'images' ? 'Downloading Images'
      : scrapperPhase === 'image2' ? 'Additional Data'
      : scrapperMsg;
    return `${prefix}${label}`;
  })();

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
      {#if popularRunning}
        <button class="btn-stop-popular" on:click={() => api.stopScrapper()}>Stop</button>
      {/if}
      <button class="btn-mode" class:btn-mode-active={viewMode === 'popular'} on:click={toggleMode}>
        {viewMode === 'popular' ? 'Popular' : 'Presets'}
      </button>
      <button class="btn-settings" on:click={() => (settingsOpen = true)} title="Settings">⚙</button>
    </div>
  </nav>

  {#if scrapperRunning || scrapperMsg || classInitVisible || popularRunning}
    <div
      class="scrapper-strip"
      class:has-error={!!scrapperError}
      data-phase={stripPhase}
    >
      <div
        class="scrapper-fill"
        style="width: {classInitVisible
          ? (classInitTotal > 0 ? Math.round((classInitCurrent / classInitTotal) * 100) : 0)
          : popularRunning
            ? (popularTotal > 0 ? Math.round((popularCurrent / popularTotal) * 100) : 0)
            : (scrapperTotal > 0 ? Math.round((scrapperCurrent / scrapperTotal) * 100) : 0)}%"
      ></div>
      <span class="scrapper-label">{stripLabel}</span>
    </div>
  {/if}

  <div class="main-layout">
    {#if appLoading || classInitVisible}
      <div class="app-skeleton">
        <!-- sidebar skeleton -->
        <div class="skel-sidebar">
          <div class="skel-search"></div>
          {#each Array(9) as _, i}
            <div class="skel-class-row" style="opacity: {1 - i * 0.08}"></div>
          {/each}
        </div>
        <!-- grid skeleton -->
        <div class="skel-grid">
          {#each Array(12) as _, i}
            <div class="skel-card" style="animation-delay: {i * 60}ms"></div>
          {/each}
        </div>
      </div>
    {:else}
      <aside class="sidebar">
        <ClassList
          {classes}
          {selectedClass}
          loading={loadingClasses}
          error={classesError}
          {filterShowDownloaded}
          {filterSortBy}
          on:select={handleSelectClass}
          on:filterChange={e => { filterShowDownloaded = e.detail.showDownloaded; filterSortBy = e.detail.sortBy; }}
          on:searchChange={e => { searchQuery = e.detail; }}
        />
      </aside>

      <main class="content custom-scroll">
        <PresetGrid
          {presets}
          {selectedClass}
          loading={loadingPresets}
          error={presetsError}
          isPopular={viewMode === 'popular'}
          {filterShowDownloaded}
          {filterSortBy}
          {searchQuery}
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
    background: rgba(220, 60, 60, 0.35) !important;
  }

  .scrapper-fill {
    position: absolute;
    inset: 0;
    background: rgba(184, 134, 11, 0.18);
    transition: width 0.3s ease, background 0.4s ease;
  }

  .scrapper-strip[data-phase="presets"] .scrapper-fill   { background: rgba(184, 134, 11, 0.22); }
  .scrapper-strip[data-phase="images"] .scrapper-fill    { background: rgba(6, 182, 212, 0.22); }
  .scrapper-strip[data-phase="image2"] .scrapper-fill    { background: rgba(59, 130, 246, 0.20); }
  .scrapper-strip[data-phase="popular"] .scrapper-fill   { background: rgba(139, 92, 246, 0.22); }
  .scrapper-strip[data-phase="popular-images"] .scrapper-fill { background: rgba(168, 85, 247, 0.18); }
  .scrapper-strip[data-phase="init"] .scrapper-fill      { background: rgba(184, 134, 11, 0.22); }

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

  .btn-mode {
    background: transparent;
    border: 1px solid #1a232c;
    border-radius: 5px;
    color: #64748b;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    padding: 0 14px;
    height: 34px;
    cursor: pointer;
    transition: border-color 0.2s, color 0.2s;
  }

  .btn-mode:hover, .btn-mode-active {
    border-color: #ffcc4d;
    color: #ffcc4d;
  }

  .btn-stop-popular {
    background: rgba(220, 60, 60, 0.08);
    border: 1px solid rgba(220, 60, 60, 0.25);
    border-radius: 5px;
    color: #f87171;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    padding: 0 14px;
    height: 34px;
    cursor: pointer;
    transition: background 0.2s, border-color 0.2s;
  }

  .btn-stop-popular:hover {
    background: rgba(220, 60, 60, 0.15);
    border-color: rgba(220, 60, 60, 0.5);
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

  .app-skeleton {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .skel-sidebar {
    width: 288px;
    flex-shrink: 0;
    border-right: 1px solid #1a232c;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .skel-search {
    height: 34px;
    border-radius: 4px;
    background: linear-gradient(90deg, #0f141a 25%, #141c25 50%, #0f141a 75%);
    background-size: 200% 100%;
    animation: shimmer 1.6s ease-in-out infinite;
    margin-bottom: 8px;
  }

  .skel-class-row {
    height: 36px;
    border-radius: 4px;
    background: linear-gradient(90deg, #0d1219 25%, #131b24 50%, #0d1219 75%);
    background-size: 200% 100%;
    animation: shimmer 1.6s ease-in-out infinite;
  }

  .skel-grid {
    flex: 1;
    padding: 32px;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 24px;
    align-content: start;
  }

  .skel-card {
    aspect-ratio: 4 / 5;
    border-radius: 8px;
    background: linear-gradient(90deg, #0d1219 25%, #141c25 50%, #0d1219 75%);
    background-size: 200% 100%;
    animation: shimmer 1.6s ease-in-out infinite;
    border: 1px solid #1a232c;
  }

  @keyframes shimmer {
    0%   { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }
</style>
