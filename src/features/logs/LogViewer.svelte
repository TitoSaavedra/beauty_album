<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { getLogs, getLogStats } from '../../tauri/album';
  import type { LogEntry, LogStats } from '../../tauri/album';

  const dispatch = createEventDispatcher();

  const PAGE_SIZE = 100;

  let stats: LogStats | null = null;
  let logs: LogEntry[] = [];
  let loading = false;
  let statsLoading = true;
  let error = '';

  let filterTag = '';
  let filterSource = '';
  let page = 0;
  let hasMore = false;

  const TAG_COLORS: Record<string, string> = {
    ERR:   '#f87171',
    WARN:  '#fbbf24',
    USER:  '#60a5fa',
    POP:   '#2dd4bf',
    SYNC:  '#2dd4bf',
    WATCH: '#2dd4bf',
    INIT:  '#a78bfa',
    INFO:  '#94a3b8',
  };

  function tagColor(tag: string): string {
    return TAG_COLORS[tag] ?? '#475569';
  }

  function fmtTs(ts: number): string {
    return new Date(ts * 1000).toLocaleString();
  }

  async function loadStats() {
    statsLoading = true;
    try { stats = await getLogStats(); } catch { stats = null; }
    finally { statsLoading = false; }
  }

  async function loadLogs(reset = false) {
    loading = true;
    error = '';
    if (reset) { page = 0; logs = []; }
    try {
      const rows = await getLogs(
        filterTag || undefined,
        filterSource || undefined,
        PAGE_SIZE,
        page * PAGE_SIZE,
      );
      logs = reset ? rows : [...logs, ...rows];
      hasMore = rows.length === PAGE_SIZE;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function applyFilters() {
    loadLogs(true);
  }

  function clearFilters() {
    filterTag = '';
    filterSource = '';
    loadLogs(true);
  }

  function nextPage() {
    page += 1;
    loadLogs();
  }

  function prevPage() {
    if (page === 0) return;
    page -= 1;
    logs = [];
    loadLogs();
  }

  onMount(() => {
    loadStats();
    loadLogs(true);
  });
</script>

<div class="lv-overlay" on:click|self={() => dispatch('close')} role="presentation">
  <div class="lv-modal">
    <div class="lv-header">
      <span class="lv-title">Log Viewer</span>
      <button class="lv-close" on:click={() => dispatch('close')}>✕</button>
    </div>

    <!-- Stats -->
    <div class="lv-stats">
      {#if statsLoading}
        <div class="lv-stat-skeleton"></div>
      {:else if stats}
        <div class="lv-stat">
          <span class="lv-stat-val">{stats.total.toLocaleString()}</span>
          <span class="lv-stat-lbl">Total</span>
        </div>
        <div class="lv-stat lv-stat-err">
          <span class="lv-stat-val">{stats.errors.toLocaleString()}</span>
          <span class="lv-stat-lbl">Errors</span>
        </div>
        <div class="lv-chips">
          {#each stats.by_source as s}
            <button
              class="lv-chip"
              class:lv-chip-active={filterSource === s.source}
              on:click={() => { filterSource = filterSource === s.source ? '' : s.source; applyFilters(); }}
            >{s.source} <em>{s.count}</em></button>
          {/each}
        </div>
        <div class="lv-chips">
          {#each stats.by_tag as t}
            <button
              class="lv-chip"
              class:lv-chip-active={filterTag === t.tag}
              style="--chip-color: {tagColor(t.tag)}"
              on:click={() => { filterTag = filterTag === t.tag ? '' : t.tag; applyFilters(); }}
            >{t.tag} <em>{t.count}</em></button>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Filter bar -->
    <div class="lv-filters">
      <select class="lv-select" bind:value={filterTag} on:change={applyFilters}>
        <option value="">All tags</option>
        {#if stats}
          {#each stats.by_tag as t}
            <option value={t.tag}>{t.tag}</option>
          {/each}
        {/if}
      </select>
      <select class="lv-select" bind:value={filterSource} on:change={applyFilters}>
        <option value="">All sources</option>
        {#if stats}
          {#each stats.by_source as s}
            <option value={s.source}>{s.source}</option>
          {/each}
        {/if}
      </select>
      {#if filterTag || filterSource}
        <button class="lv-clear-btn" on:click={clearFilters}>Clear</button>
      {/if}
      <span class="lv-page-info">Page {page + 1}</span>
    </div>

    <!-- Log list -->
    <div class="lv-list custom-scroll">
      {#if error}
        <div class="lv-error">{error}</div>
      {:else if loading && logs.length === 0}
        <div class="lv-empty">Loading...</div>
      {:else if logs.length === 0}
        <div class="lv-empty">No entries</div>
      {:else}
        {#each logs as entry (entry.id)}
          <div class="lv-row">
            <span class="lv-ts">{fmtTs(entry.ts)}</span>
            <span class="lv-tag" style="color: {tagColor(entry.tag)}">{entry.tag}</span>
            <span class="lv-source">{entry.source}</span>
            <span class="lv-msg">{entry.msg}</span>
          </div>
        {/each}
        {#if loading}
          <div class="lv-empty">Loading...</div>
        {/if}
      {/if}
    </div>

    <!-- Pagination -->
    <div class="lv-pagination">
      <button class="lv-page-btn" disabled={page === 0} on:click={prevPage}>← Prev</button>
      <button class="lv-page-btn" disabled={!hasMore} on:click={nextPage}>Next →</button>
    </div>
  </div>
</div>

<style>
  .lv-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.72);
    z-index: 200;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .lv-modal {
    background: #0a0e15;
    border: 1px solid #1a232c;
    border-radius: 10px;
    width: 960px;
    max-width: 96vw;
    height: 80vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .lv-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 20px;
    border-bottom: 1px solid #1a232c;
    flex-shrink: 0;
  }

  .lv-title {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: #fff;
  }

  .lv-close {
    background: transparent;
    border: none;
    color: #475569;
    font-size: 14px;
    cursor: pointer;
    padding: 4px 8px;
    transition: color 0.15s;
  }

  .lv-close:hover { color: #fff; }

  .lv-stats {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 20px;
    border-bottom: 1px solid #0f151d;
    flex-shrink: 0;
    flex-wrap: wrap;
  }

  .lv-stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    background: #0c1117;
    border: 1px solid #1a232c;
    border-radius: 6px;
    padding: 6px 14px;
    min-width: 60px;
  }

  .lv-stat-err .lv-stat-val { color: #f87171; }

  .lv-stat-val {
    font-size: 18px;
    font-weight: 700;
    color: #e2e8f0;
    font-family: monospace;
  }

  .lv-stat-lbl {
    font-size: 9px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: #475569;
  }

  .lv-stat-skeleton {
    height: 48px;
    width: 80px;
    border-radius: 6px;
    background: #0c1117;
    border: 1px solid #1a232c;
  }

  .lv-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .lv-chip {
    background: #0c1117;
    border: 1px solid #1a232c;
    border-radius: 4px;
    color: var(--chip-color, #64748b);
    font-size: 9px;
    font-family: monospace;
    letter-spacing: 0.08em;
    padding: 3px 8px;
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s;
  }

  .lv-chip em {
    font-style: normal;
    color: #334155;
    margin-left: 4px;
  }

  .lv-chip:hover, .lv-chip-active {
    border-color: var(--chip-color, #64748b);
    background: rgba(255,255,255,0.04);
  }

  .lv-filters {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 20px;
    border-bottom: 1px solid #0f151d;
    flex-shrink: 0;
  }

  .lv-select {
    background: #0c1117;
    border: 1px solid #1a232c;
    border-radius: 4px;
    color: #94a3b8;
    font-size: 11px;
    font-family: monospace;
    padding: 4px 8px;
    cursor: pointer;
  }

  .lv-clear-btn {
    background: transparent;
    border: 1px solid #1a232c;
    border-radius: 4px;
    color: #64748b;
    font-size: 10px;
    letter-spacing: 0.1em;
    padding: 4px 10px;
    cursor: pointer;
    transition: color 0.15s, border-color 0.15s;
  }

  .lv-clear-btn:hover {
    color: #f87171;
    border-color: #f87171;
  }

  .lv-page-info {
    font-size: 10px;
    font-family: monospace;
    color: #334155;
    margin-left: auto;
  }

  .lv-list {
    flex: 1;
    overflow-y: auto;
  }

  .lv-row {
    display: grid;
    grid-template-columns: 160px 48px 160px 1fr;
    gap: 12px;
    align-items: baseline;
    padding: 5px 20px;
    border-bottom: 1px solid #0a0e14;
    font-size: 11px;
    font-family: monospace;
  }

  .lv-row:hover { background: rgba(255,255,255,0.02); }

  .lv-ts {
    color: #334155;
    font-size: 10px;
    white-space: nowrap;
  }

  .lv-tag {
    font-weight: 700;
    font-size: 10px;
    letter-spacing: 0.08em;
  }

  .lv-source {
    color: #475569;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .lv-msg {
    color: #94a3b8;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .lv-empty {
    padding: 40px;
    text-align: center;
    font-size: 11px;
    font-family: monospace;
    color: #334155;
  }

  .lv-error {
    padding: 20px;
    color: #f87171;
    font-size: 11px;
    font-family: monospace;
  }

  .lv-pagination {
    display: flex;
    gap: 8px;
    padding: 10px 20px;
    border-top: 1px solid #0f151d;
    flex-shrink: 0;
  }

  .lv-page-btn {
    background: #0c1117;
    border: 1px solid #1a232c;
    border-radius: 4px;
    color: #64748b;
    font-size: 10px;
    font-family: monospace;
    letter-spacing: 0.08em;
    padding: 4px 12px;
    cursor: pointer;
    transition: color 0.15s, border-color 0.15s;
  }

  .lv-page-btn:disabled { opacity: 0.3; cursor: default; }
  .lv-page-btn:not(:disabled):hover { color: #e2e8f0; border-color: #2d3d50; }
</style>
