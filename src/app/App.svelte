<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from 'svelte-i18n';
  import * as api from '../features/beauty/tauri/album';
  import type { ClassEntry, AppConfig } from '../features/beauty/tauri/album';
  import ClassList from '../features/beauty/components/ClassList/ClassList.svelte';
  import PresetGrid from '../features/beauty/components/PresetGrid/PresetGrid.svelte';
  import PresetDetail from '../features/beauty/components/PresetDetail/PresetDetail.svelte';
  import SettingsModal from '../shared/components/SettingsModal/SettingsModal.svelte';
  import ConfirmDialog from '../shared/components/ConfirmDialog/ConfirmDialog.svelte';
  import ToastContainer from '../shared/components/ToastContainer/ToastContainer.svelte';
  import { appConfig } from '../shared/stores/appConfig';
  import { appLoading, appError, initStatus, settingsOpen } from '../shared/stores/appState';
  import { wantedPresets } from '../features/beauty/stores/wantedPresets';
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
  import { initRustEvents } from '../shared/events/rustEvents';
  import Button from '../shared/components/Button/Button.svelte';

  let config: AppConfig = { bdo_docs_dir: '', cf_clearance: '' };
  let searchDebounce: ReturnType<typeof setTimeout> | null = null;
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
      appError.set('Database failed to open. Check logs.');
      appLoading.set(false);
      return;
    }
    if (config.bdo_docs_dir) {
      await wantedPresets.load();
      appLoading.set(false);
      runQueues();
    } else {
      appLoading.set(false);
      settingsOpen.set(true);
    }
  }

  onMount(async () => {
    await initRustEvents(onDbReady);

    try {
      initStatus.set('Loading configuration...');
      config = await api.getConfig();
      appConfig.set(config);

      if (!config.bdo_docs_dir) {
        appLoading.set(false);
        settingsOpen.set(true);
      } else {
        initStatus.set('Opening database...');
        if (await api.isDbReady()) {
          await onDbReady(true);
        }
      }
    } catch { appLoading.set(false); }
  });

  async function handleSettingsSave(e: CustomEvent<AppConfig>) {
    config = e.detail;
    appConfig.set(config);
    settingsOpen.set(false);
    selectedClass.set(null);
    selectedClassObj.set(null);
    presets.set([]);
    if (!config.bdo_docs_dir) return;
    if (await api.isDbReady()) {
      appLoading.set(true);
      initStatus.set('Reloading...');
      await wantedPresets.load();
      appLoading.set(false);
      runQueues();
    } else {
      appLoading.set(true);
      initStatus.set('Opening database...');
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
    <div class="nav-actions">
      <Button variant="icon" on:click={() => settingsOpen.set(true)} title="Settings">⚙</Button>
    </div>
  </nav>

  <div class="main-layout">
    {#if $appLoading || $appError}
        <div class="app-skeleton">
        {#if $appError}
          <div class="app-init-error">{$appError}</div>
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
  </div>

  {#if ($appLoading && !$appError) || $scrapperRunning || $scrapperMsg || $popularRunning}
    <div class="status-bar" class:has-error={!!$scrapperError} data-phase={$appLoading ? '' : stripPhase}>
      <div class="status-bar-track">
        {#if $appLoading || ($scrapperRunning && $scrapperTotal === 0) || ($popularRunning && $popularTotal === 0)}
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
      <span class="status-bar-text">{$appLoading ? $initStatus : stripLabel}</span>
    </div>
  {/if}
</div>

<PresetDetail />
<ConfirmDialog />
<ToastContainer />

{#if $settingsOpen}
  <SettingsModal
    {config}
    on:save={handleSettingsSave}
    on:close={() => settingsOpen.set(false)}
  />
{/if}

<style lang="scss">
  @use './App.scss';
</style>
