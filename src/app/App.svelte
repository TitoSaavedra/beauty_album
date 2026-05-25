<script lang="ts">
  import { onMount } from 'svelte';
  import { _, locale } from 'svelte-i18n';
  import { listen } from '@tauri-apps/api/event';
  import * as api from '../features/beauty/tauri/album';
  import type { ClassEntry, AppConfig } from '../features/beauty/tauri/album';
  import ClassList from '../features/beauty/components/ClassList/ClassList.svelte';
  import PresetGrid from '../features/beauty/components/PresetGrid/PresetGrid.svelte';
  import PresetDetail from '../features/beauty/components/PresetDetail/PresetDetail.svelte';
  import SettingsModal from '../shared/components/SettingsModal/SettingsModal.svelte';
  import ConfirmDialog from '../shared/components/ConfirmDialog/ConfirmDialog.svelte';
  import ToastContainer from '../shared/components/ToastContainer/ToastContainer.svelte';
  import { appConfig } from '../shared/stores/appConfig';
  import { toast } from '../shared/stores/toast';
  import { wantedPresets } from '../features/beauty/stores/wantedPresets';
  import { theme } from '../stores/theme';
  import {
    selectedClass, selectedClassObj, presets, loadingPresets, presetsError,
    hasMore, loadingMore, popularStats, popularRegions, filterSortBy,
    filterSinceTs, filterRegion, searchQuery, viewMode,
    selectClass, loadMore, setMode,
  } from '../features/beauty/stores/presetGrid';
  import {
    scrapperRunning, scrapperTotal, scrapperCurrent, scrapperMsg, scrapperError, scrapperPhase,
    startScraper,
  } from '../features/beauty/stores/scraper';
  import {
    popularRunning, popularTotal, popularCurrent, popularMsg, popularPhase,
    startPopularSync,
  } from '../features/popular/stores/popularSync';

  let config: AppConfig = { bdo_docs_dir: '', cf_clearance: '' };
  let settingsOpen = false;
  let appLoading = true;
  let appError = '';
  let initStatus = 'Initializing...';
  let searchDebounce: ReturnType<typeof setTimeout> | null = null;
  let activeTab: 'album' | 'test' = 'album';

  $: stripPhase = $popularRunning
    ? ($popularPhase === 'images' ? 'popular-images' : 'popular')
    : $scrapperPhase;

  $: stripLabel = (() => {
    if ($popularRunning) {
      const prefix = $popularTotal > 0 ? `${$popularCurrent}/${$popularTotal} — ` : '';
      const phase = $popularPhase === 'images' ? 'Downloading Popular Images' : 'Syncing Popular';
      return `${prefix}${phase}${$popularMsg ? ' — ' + $popularMsg : ''}`;
    }
    const prefix = $scrapperTotal > 0 ? `${$scrapperCurrent}/${$scrapperTotal} — ` : '';
    const label = $scrapperPhase === 'presets' ? 'Downloading Presets'
      : $scrapperPhase === 'images' ? 'Downloading Images'
      : $scrapperMsg;
    return `${prefix}${label}`;
  })();

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

    listen<string[]>('folder_changed', async ({ payload: files }) => {
      const count = files.length;
      const label = count === 1 ? files[0] : `${count} ${$_('common.new_presets_found')}`;
      toast.show(`${$_('common.new_preset_found')}: ${label}`, 'success', 5000);
      if (!$scrapperRunning) {
        await startScraper();
        startPopularSync();
      }
    });

    try {
      initStatus = 'Loading configuration...';
      config = await api.getConfig();
      appConfig.set(config);

      if (config.locale) {
        locale.set(config.locale);
        if (typeof window !== 'undefined') {
          localStorage.setItem('preferred-locale', config.locale);
        }
      }

      if (config.theme) {
        theme.set(config.theme);
      }

      if (!config.bdo_docs_dir) {
        appLoading = false;
        settingsOpen = true;
      } else {
        initStatus = 'Opening database...';
        if (await api.isDbReady()) {
          await onDbReady(true);
        }
      }
    } catch { appLoading = false; }
  });

  async function handleSettingsSave(e: CustomEvent<AppConfig>) {
    config = e.detail;
    appConfig.set(config);
    settingsOpen = false;
    selectedClass.set(null);
    selectedClassObj.set(null);
    presets.set([]);
    if (!config.bdo_docs_dir) return;
    if (await api.isDbReady()) {
      appLoading = true;
      initStatus = 'Reloading...';
      await wantedPresets.load();
      appLoading = false;
      runQueues();
    } else {
      appLoading = true;
      initStatus = 'Opening database...';
    }
  }

  function handleSelectClass(e: CustomEvent<ClassEntry>) {
    selectedClassObj.set(e.detail);
    selectClass(e.detail.name);
  }
</script>

<div class="app">
  <nav class="nav glass-panel">
    <div class="nav-brand">
      {#if $selectedClassObj}
        <span class="nav-title">
          Album
          <span class="nav-sep">&gt;</span>
          {#if $selectedClassObj.icon_svg}
            <span class="nav-class-icon">{@html $selectedClassObj.icon_svg}</span>
          {/if}
          {$selectedClassObj.name}
          <span class="nav-sep">&gt;</span>
          {$selectedClassObj.preset_count} Presets
        </span>
      {:else}
        <span class="nav-title">Album</span>
      {/if}
    </div>
    <!-- <div class="nav-tabs">
      <button class="nav-tab" class:active={activeTab === 'album'} on:click={() => activeTab = 'album'}>
        Album
      </button>
    </div> -->
    <div class="nav-actions">
      <button class="btn-settings" on:click={() => (settingsOpen = true)} title="Settings">⚙</button>
    </div>
  </nav>

  <div class="main-layout">
    {#if activeTab === 'album'}
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
          selectedClass={$selectedClass}
          filterSortBy={$filterSortBy}
          viewMode={$viewMode}
          popularStats={$popularStats}
          regions={$popularRegions}
          on:select={handleSelectClass}
          on:filterChange={e => {
            filterSortBy.set(e.detail.sortBy);
            filterSinceTs.set(e.detail.sinceTs);
            filterRegion.set(e.detail.region || '');
            if ($selectedClass) selectClass($selectedClass);
          }}
          on:searchChange={e => {
            searchQuery.set(e.detail);
            if (searchDebounce) clearTimeout(searchDebounce);
            searchDebounce = setTimeout(() => {
              searchDebounce = null;
              if ($selectedClass) selectClass($selectedClass);
            }, 350);
          }}
          on:modeChange={e => setMode(e.detail)}
        />
      </aside>

      <main
        class="content custom-scroll"
        on:scroll={e => {
          const el = e.currentTarget;
          if (el.scrollHeight - el.scrollTop - el.clientHeight < 300 && $hasMore && !$loadingMore) loadMore();
        }}
      >
        <PresetGrid
          presets={$presets}
          selectedClass={$selectedClass}
          loading={$loadingPresets}
          error={$presetsError}
          isPopular={$viewMode === 'popular'}
          hasMore={$hasMore}
          loadingMore={$loadingMore}
          extraLoading={$popularRunning && $viewMode === 'popular'}
        />
      </main>
      {/if}
    {:else}
      <div class="test-tab">
        <div class="test-content">Test Tab (blank)</div>
      </div>
    {/if}
  </div>

  {#if (appLoading && !appError) || $scrapperRunning || $scrapperMsg || $popularRunning}
    <div class="status-bar" class:has-error={!!$scrapperError} data-phase={appLoading ? '' : stripPhase}>
      <div class="status-bar-track">
        {#if appLoading || ($scrapperRunning && $scrapperTotal === 0) || ($popularRunning && $popularTotal === 0)}
          <div class="status-bar-sweep"></div>
        {:else}
          <div
            class="status-bar-fill"
            style="width: {$popularRunning
              ? Math.round(($popularCurrent / $popularTotal) * 100)
              : Math.round(($scrapperCurrent / $scrapperTotal) * 100)}%"
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

<style lang="scss">
  @import './App.scss';
</style>
