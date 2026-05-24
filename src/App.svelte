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
  import LogViewer from './features/logs/LogViewer.svelte';
  import { appConfig } from './stores/appConfig';
  import { toast } from './stores/toast';
  import { presetDetail } from './stores/presetDetail';
  import { wantedPresets } from './stores/wantedPresets';
  import { logViewerOpen } from './stores/logViewer';

  interface ScrapperProgress {
    preset_id: string;
    status: string;
    message: string;
    class_name: string;
    class_id: number;
    current: number;
    total: number;
  }

  let config: AppConfig = { bdo_docs_dir: '', cf_clearance: '' };
  let scrapperRunning = false;
  let scrapperCurrent = 0;
  let scrapperTotal = 0;
  let scrapperMsg = '';
  let scrapperError = '';
  let scrapperPhase: 'presets' | 'images' | '' = '';
  let settingsOpen = false;
  let classes: ClassEntry[] = [];
  let selectedClass: string | null = null;
  let presets: PresetEntry[] = [];
  let loadingClasses = false;
  let loadingPresets = false;
  let classesError = '';
  let presetsError = '';
  let viewMode: 'presets' | 'popular' = 'presets';
  let popularRunning = false;
  let popularCurrent = 0;
  let popularTotal = 0;
  let popularMsg = '';
  let popularPhase: 'syncing' | 'images' | '' = '';
  let filterSortBy: 'downloads' | 'views' | 'favorites' = 'downloads';
  let filterSinceTs = 0;
  let searchQuery = '';
  let popularStats: import('./tauri/album').PopularStats | null = null;
  let searchDebounce: ReturnType<typeof setTimeout> | null = null;
  let appLoading = true;
  let appError = '';
  let initStatus = 'Initializing...';

  const PAGE_SIZE = 50;
  let presetOffset = 0;
  let hasMore = false;
  let loadingMore = false;

  let gridRefreshTimer: ReturnType<typeof setTimeout> | null = null;

  $: selectedClassId = classes.find(c => c.name === selectedClass)?.class_id ?? null;

  function scheduleGridRefresh(classId: number) {
    if (gridRefreshTimer !== null) clearTimeout(gridRefreshTimer);
    gridRefreshTimer = setTimeout(async () => {
      gridRefreshTimer = null;
      if (classId !== selectedClassId || !selectedClass || !config.bdo_docs_dir) return;
      try {
        const results = viewMode === 'popular'
          ? await api.getPopularPresets(selectedClass, 0, PAGE_SIZE, filterSortBy, searchQuery, filterSinceTs)
          : await api.getPresets(selectedClass, 0, PAGE_SIZE, filterSortBy, searchQuery);
        presets = results;
        presetOffset = 0;
        hasMore = results.length === PAGE_SIZE;
      } catch { /* non-fatal */ }
    }, 400);
  }


  async function runQueues() {
    const pending = await api.checkPending();
    if (pending > 0) await startScraper();
    await startPopularSync(); 
  }

  async function onDbReady(ok: boolean) {
    if (!ok) {
      appError = 'Database failed to open. Check logs.';
      appLoading = false;
      return;
    }
    if (config.bdo_docs_dir) {
      await wantedPresets.load();
      initStatus = 'Loading classes...';
      await loadClasses();
      appLoading = false;
      runQueues();
    } else {
      appLoading = false;
      settingsOpen = true;
    }
  }

  onMount(async () => {
    await listen<boolean>('db_ready', ({ payload: ok }) => onDbReady(ok));

    listen<string>('init_progress', ({ payload }) => { initStatus = payload; });

    listen('refresh_album', () => {
      if (config.bdo_docs_dir) loadClasses(true);
    });

    listen<ScrapperProgress>('popular_progress', ({ payload }) => {
      if (payload.total > 0 && popularTotal === 0) popularTotal = payload.total;
      if (payload.current > 0) popularCurrent = payload.current;
      if (payload.message) popularMsg = payload.message;
      if (payload.status === 'metadata') {
        popularPhase = 'syncing';
        if (viewMode === 'popular' && payload.class_id && payload.class_id === selectedClassId)
          scheduleGridRefresh(payload.class_id);
      } else if (payload.status === 'done') {
        popularPhase = 'syncing';
        if (viewMode === 'popular' && payload.class_id && payload.class_id === selectedClassId)
          scheduleGridRefresh(payload.class_id);
      } else if (payload.status === 'additional_data') {
        popularPhase = 'images';
      }
    });

    listen<string[]>('folder_changed', ({ payload: files }) => {
      const label = files.length === 1 ? files[0] : `${files.length} new presets`;
      toast.show(`New preset found: ${label}`, 'success', 4000);
      if (!scrapperRunning && !popularRunning) startScraper();
    });

    try {
      initStatus = 'Loading configuration...';
      config = await api.getConfig();
      appConfig.set(config);
      if (!config.bdo_docs_dir) {
        appLoading = false;
        settingsOpen = true;
      } else {
        initStatus = 'Opening database...';
        if (await api.isDbReady()) {
          // Event fired before listener registered — handle now.
          await onDbReady(true);
        }
        // Otherwise stay in skeleton; db_ready listener above handles the rest.
      }
    } catch { appLoading = false; }
  });

  async function selectClass(name: string) {
    selectedClass = name;
    presets = [];
    presetsError = '';
    presetOffset = 0;
    hasMore = false;
    loadingPresets = true;
    try {
      if (viewMode === 'popular') {
        const [results, stats] = await Promise.all([
          api.getPopularPresets(name, 0, PAGE_SIZE, filterSortBy, searchQuery, filterSinceTs),
          api.getPopularStats(name),
        ]);
        presets = results;
        popularStats = stats;
        hasMore = results.length === PAGE_SIZE;
      } else {
        const results = await api.getPresets(name, 0, PAGE_SIZE, filterSortBy, searchQuery);
        presets = results;
        hasMore = results.length === PAGE_SIZE;
      }
    } catch (err) {
      presetsError = String(err);
    } finally {
      loadingPresets = false;
    }
  }

  async function loadMore() {
    if (loadingMore || !hasMore || !selectedClass) return;
    loadingMore = true;
    const nextOffset = presetOffset + PAGE_SIZE;
    try {
      const results = viewMode === 'popular'
        ? await api.getPopularPresets(selectedClass, nextOffset, PAGE_SIZE, filterSortBy, searchQuery, filterSinceTs)
        : await api.getPresets(selectedClass, nextOffset, PAGE_SIZE, filterSortBy, searchQuery);
      presets = [...presets, ...results];
      presetOffset = nextOffset;
      hasMore = results.length === PAGE_SIZE;
    } catch (err) {
      presetsError = String(err);
    } finally {
      loadingMore = false;
    }
  }

  async function loadClasses(keepSelected = false) {
    if (!keepSelected) loadingClasses = true;
    classesError = '';
    try {
      const prev = selectedClass;
      const fresh = viewMode === 'popular'
        ? await api.getPopularClasses(filterSinceTs)
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
      await selectClass(target);
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
    if (!config.bdo_docs_dir) return;
    if (await api.isDbReady()) {
      // DB already open — reload immediately (subsequent settings saves).
      appLoading = true;
      initStatus = 'Reloading...';
      await wantedPresets.load();
      await loadClasses();
      appLoading = false;
      runQueues();
    } else {
      // First-time setup — Rust will open DB and emit db_ready; listener handles the rest.
      appLoading = true;
      initStatus = 'Opening database...';
    }
  }

  function handleSelectClass(e: CustomEvent<string>) {
    selectClass(e.detail);
  }

  async function setMode(mode: 'presets' | 'popular') {
    if (viewMode === mode) return;
    viewMode = mode;
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
        if (viewMode === 'presets' && payload.class_id && payload.class_id === selectedClassId)
          scheduleGridRefresh(payload.class_id);
      } else if (payload.status === 'done') {
        scrapperPhase = 'images';
        if (viewMode === 'presets' && payload.class_id && payload.class_id === selectedClassId)
          scheduleGridRefresh(payload.class_id);
        const pid = payload.preset_id;
        const cls = payload.class_name;
        toast.show(`${cls} #${pid} downloaded`, 'success', 5000, async () => {
          const list = await api.getPresets(cls);
          const found = list.find(p => p.preset_id === pid);
          if (found) presetDetail.set(found);
        });
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

  $: stripPhase = popularRunning ? (popularPhase === 'images' ? 'popular-images' : 'popular')
    : scrapperPhase;

  $: stripLabel = (() => {
    if (popularRunning) {
      const prefix = popularTotal > 0 ? `${popularCurrent}/${popularTotal} — ` : '';
      const phase = popularPhase === 'images' ? 'Downloading Popular Images' : 'Syncing Popular';
      return `${prefix}${phase}${popularMsg ? ' — ' + popularMsg : ''}`;
    }
    const prefix = scrapperTotal > 0 ? `${scrapperCurrent}/${scrapperTotal} — ` : '';
    const label = scrapperPhase === 'presets' ? 'Downloading Presets'
      : scrapperPhase === 'images' ? 'Downloading Images'
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
      <button class="btn-settings" on:click={() => (settingsOpen = true)} title="Settings">⚙</button>
    </div>
  </nav>

  <div class="main-layout">
    {#if appLoading || appError}
      <div class="app-skeleton">
        {#if appError}
          <div class="app-init-error">{appError}</div>
        {/if}
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
          {filterSortBy}
          {viewMode}
          {popularStats}
          on:select={handleSelectClass}
          on:filterChange={e => {
            const prevSinceTs = filterSinceTs;
            filterSortBy = e.detail.sortBy;
            filterSinceTs = e.detail.sinceTs;
            if (filterSinceTs !== prevSinceTs && viewMode === 'popular') {
              loadClasses(true);
            } else if (selectedClass) {
              selectClass(selectedClass);
            }
          }}
          on:searchChange={e => {
            searchQuery = e.detail;
            if (searchDebounce) clearTimeout(searchDebounce);
            searchDebounce = setTimeout(() => {
              searchDebounce = null;
              if (selectedClass) selectClass(selectedClass);
            }, 350);
          }}
          on:modeChange={e => setMode(e.detail)}
        />
      </aside>

      <main
        class="content custom-scroll"
        on:scroll={e => {
          const el = e.currentTarget;
          if (el.scrollHeight - el.scrollTop - el.clientHeight < 300 && hasMore && !loadingMore) loadMore();
        }}
      >
        <PresetGrid
          {presets}
          {selectedClass}
          loading={loadingPresets}
          error={presetsError}
          isPopular={viewMode === 'popular'}
          {hasMore}
          {loadingMore}
          extraLoading={popularRunning && viewMode === 'popular'}
        />
      </main>
    {/if}
  </div>

  {#if (appLoading && !appError) || scrapperRunning || scrapperMsg || popularRunning}
    <div class="status-bar" class:has-error={!!scrapperError} data-phase={appLoading ? '' : stripPhase}>
      <div class="status-bar-track">
        {#if appLoading || (scrapperRunning && scrapperTotal === 0) || (popularRunning && popularTotal === 0)}
          <div class="status-bar-sweep"></div>
        {:else}
          <div
            class="status-bar-fill"
            style="width: {popularRunning
              ? Math.round((popularCurrent / popularTotal) * 100)
              : Math.round((scrapperCurrent / scrapperTotal) * 100)}%"
          ></div>
        {/if}
      </div>
      <span class="status-bar-text">{appLoading ? initStatus : stripLabel}</span>
    </div>
  {/if}
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

{#if $logViewerOpen}
  <LogViewer on:close={() => logViewerOpen.set(false)} />
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
    position: relative;
  }

  .app-init-error {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 12px;
    letter-spacing: 0.12em;
    color: #f87171;
    background: rgba(7, 10, 14, 0.85);
    z-index: 10;
  }

  .status-bar {
    height: 22px;
    min-height: 22px;
    background: #060810;
    border-top: 1px solid #141c24;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 16px;
    flex-shrink: 0;
  }

  .status-bar-track {
    width: 120px;
    height: 2px;
    background: #141c24;
    border-radius: 2px;
    overflow: hidden;
    position: relative;
    flex-shrink: 0;
  }

  .status-bar-sweep {
    position: absolute;
    top: 0;
    height: 100%;
    width: 50%;
    background: linear-gradient(90deg, transparent, rgba(255, 204, 77, 0.7), transparent);
    animation: status-sweep 1.4s ease-in-out infinite;
  }

  .status-bar-fill {
    position: absolute;
    top: 0;
    left: 0;
    height: 100%;
    background: rgba(255, 204, 77, 0.7);
    transition: width 0.3s ease;
    border-radius: 2px;
  }

  .status-bar[data-phase="presets"] .status-bar-fill   { background: rgba(255, 204, 77, 0.75); }
  .status-bar[data-phase="images"] .status-bar-fill    { background: rgba(6, 182, 212, 0.75); }

  .status-bar[data-phase="popular"] .status-bar-fill   { background: rgba(139, 92, 246, 0.75); }
  .status-bar[data-phase="popular-images"] .status-bar-fill { background: rgba(168, 85, 247, 0.75); }
  .status-bar.has-error .status-bar-fill               { background: rgba(220, 60, 60, 0.75); }

  .status-bar-text {
    font-size: 10px;
    font-family: monospace;
    letter-spacing: 0.08em;
    color: #2d3d50;
  }

  @keyframes status-sweep {
    0%   { left: -50%; }
    100% { left: 120%; }
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
