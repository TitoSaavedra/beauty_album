<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from 'svelte-i18n';
  import * as api from '../beauty/tauri/album';
  import type { ClassEntry, AppConfig } from '../beauty/tauri/album';
  import ClassList from '../beauty/components/ClassList/ClassList.svelte';
  import PresetGrid from '../beauty/components/PresetGrid/PresetGrid.svelte';
  import PresetDetail from '../beauty/components/PresetDetail/PresetDetail.svelte';
  import SettingsModal from './SettingsModal/SettingsModal.svelte';
  import ConfirmDialog from '../shared/components/ConfirmDialog/ConfirmDialog.svelte';
  import ToastContainer from '../shared/components/ToastContainer/ToastContainer.svelte';
  import { appConfig } from '../shared/stores/appConfig';
  import { appLoading, appError, settingsOpen } from '../shared/stores/appState';
  import { wantedPresets } from '../beauty/stores/wantedPresets';
  import {
    selectedClass, selectedClassObj, presets, loadingPresets, presetsError,
    hasMore, loadingMore, popularStats, popularRegions, filterSortBy,
    filterSinceTs, filterRegion, searchQuery, viewMode,
    selectClass, loadMore, setMode,
  } from '../beauty/stores/presetGrid';
  import {
    personalRunning, personalTotal, personalCurrent, personalMsg, personalError, personalPhase,
  } from '../beauty/stores/personalSync';
  import {
    popularRunning, popularTotal, popularCurrent, popularMsg, popularPhase,
  } from '../beauty/stores/popularSync';
  import { initRustEvents } from '../shared/events/rustEvents';
  import { syncLoading } from '../shared/stores/syncLoading';
  import Button from '../shared/components/Button/Button.svelte';

  let config: AppConfig = { bdo_docs_dir: '', cf_clearance: '' };
  let searchDebounce: ReturnType<typeof setTimeout> | null = null;
  $: stripPhase = $popularRunning
    ? ($popularPhase === 'images' ? 'popular-images' : 'popular')
    : $personalPhase;

  $: stripLabel = (() => {
    if ($popularRunning) {
      const prefix = $popularTotal > 0 ? `${$popularCurrent}/${$popularTotal} — ` : '';
      const phase = $popularPhase === 'images' ? 'Downloading Popular Images' : 'Syncing Popular';
      return `${prefix}${phase}${$popularMsg ? ' — ' + $popularMsg : ''}`;
    }
    const prefix = $personalTotal > 0 ? `${$personalCurrent}/${$personalTotal} — ` : '';
    const label = $personalPhase === 'images' ? 'Downloading Images' : $personalMsg;
    return `${prefix}${label}`;
  })();

  async function onDbReady(ok: boolean) {
    if (!ok) {
      appError.set('Database failed to open. Check logs.');
      appLoading.set(false);
      return;
    }
    if (config.bdo_docs_dir) {
      // Fire wanted presets load in background without awaiting
      wantedPresets.load().catch(() => {});
      appLoading.set(false);
    } else {
      appLoading.set(false);
      settingsOpen.set(true);
    }
  }

  onMount(async () => {
    // Fire all initialization tasks in background without awaiting
    initRustEvents(onDbReady).catch((e) => {
      console.error('initRustEvents error:', e);
    });

    (async () => {
      try {
        config = await api.getConfig();
        appConfig.set(config);

        if (!config.bdo_docs_dir) {
          settingsOpen.set(true);
          return;
        }

        const dbReady = await api.isDbReady();
        await onDbReady(dbReady);
      } catch (e) {
        console.error('App init error:', e);
        appError.set(String(e));
      }
    })();
  });

  async function handleSettingsSave(e: CustomEvent<AppConfig>) {
    config = e.detail;
    appConfig.set(config);
    settingsOpen.set(false);
    selectedClass.set(null);
    selectedClassObj.set(null);
    presets.set([]);
    if (!config.bdo_docs_dir) return;

    // Fire reload tasks in background without awaiting
    (async () => {
      if (await api.isDbReady()) {
        await wantedPresets.load().catch(() => {});
      }
    })();
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
    {#if $appError}
      <div class="app-init-error">{$appError}</div>
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

  {#if $syncLoading || $personalRunning || $popularRunning}
    <div class="status-bar" class:has-error={!!$personalError} data-phase={stripPhase}>
      <div class="status-bar-track">
        {#if ($personalTotal === 0 && $popularTotal === 0) || (!$personalRunning && !$popularRunning)}
          <div class="status-bar-sweep"></div>
        {:else}
          <div
            class="status-bar-fill"
            style="width: {$popularRunning
              ? Math.round(($popularCurrent / $popularTotal) * 100)
              : Math.round(($personalCurrent / $personalTotal) * 100)}%"
          ></div>
        {/if}
      </div>
      <span class="status-bar-text">{$syncLoading || stripLabel}</span>
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
