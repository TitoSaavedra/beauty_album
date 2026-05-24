<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { fly, slide } from 'svelte/transition';
  import { flip } from 'svelte/animate';
  import type { ClassEntry, PopularStats } from '../../tauri/album';
  import { getClassFavorites, setClassFavorite } from '../../tauri/album';

  export let classes: ClassEntry[] = [];
  export let selectedClass: string | null = null;
  export let loading = false;
  export let error = '';
  export let filterSortBy: 'downloads' | 'views' | 'favorites' = 'downloads';
  export let viewMode: 'presets' | 'popular' = 'presets';
  export let popularStats: PopularStats | null = null;

  const dispatch = createEventDispatcher<{
    select: string;
    filterChange: { sortBy: 'downloads' | 'views' | 'favorites'; sinceTs: number };
    searchChange: string;
    modeChange: 'presets' | 'popular';
  }>();

  let favorites: Set<string> = new Set();

  onMount(async () => {
    try {
      const favs = await getClassFavorites();
      favorites = new Set(favs);
    } catch { /* ignore on first launch */ }
  });

  async function toggleFavorite(name: string, e: MouseEvent) {
    e.stopPropagation();
    const isFav = favorites.has(name);
    if (isFav) { favorites.delete(name); } else { favorites.add(name); }
    favorites = new Set(favorites);
    try {
      await setClassFavorite(name, !isFav);
    } catch { /* non-fatal */ }
  }

  let search = '';
  let filterOpen = false;
  let localSortBy = filterSortBy;
  let localViewMode = viewMode;
  let localDays = 0; // 0 = ever

  $: localSortBy = filterSortBy;
  $: localViewMode = viewMode;

  $: filtered = search.trim()
    ? classes.filter(c => c.name.toLowerCase().includes(search.toLowerCase()))
    : classes;

  $: sorted = [...filtered].sort((a, b) => {
    const aFav = favorites.has(a.name);
    const bFav = favorites.has(b.name);
    if (aFav !== bFav) return aFav ? -1 : 1;
    return b.preset_count - a.preset_count;
  });

  $: filterActive = localSortBy !== 'downloads' || localViewMode === 'popular' || localDays !== 0;

  function onSearchInput() {
    dispatch('searchChange', search);
  }

  function setSortBy() {
    emitFilter();
  }

  $: statsCount = (() => {
    if (!popularStats) return null;
    if (localDays === 0)   return popularStats.total;
    if (localDays === 20)  return popularStats.d20;
    if (localDays === 30)  return popularStats.d30;
    if (localDays === 60)  return popularStats.d60;
    if (localDays === 90)  return popularStats.d90;
    if (localDays === 180) return popularStats.d180;
    if (localDays === 365) return popularStats.d365;
    return null;
  })();

  function sinceTs(): number {
    if (localDays === 0) return 0;
    return Math.floor(Date.now() / 1000) - localDays * 86400;
  }

  function emitFilter() {
    dispatch('filterChange', { sortBy: localSortBy, sinceTs: sinceTs() });
  }

  function toggleViewMode() {
    localViewMode = localViewMode === 'presets' ? 'popular' : 'presets';
    dispatch('modeChange', localViewMode);
  }

</script>

<div class="search-box">
  <div class="search-row">
    <input
      class="search-input"
      type="text"
      bind:value={search}
      on:input={onSearchInput}
      placeholder="Search classes, presets, creators..."
    />
    <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
    <button
      class="filter-btn"
      class:filter-active={filterActive}
      title="Filter & Sort"
      on:click={() => filterOpen = !filterOpen}
    >
      <svg width="15" height="15" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
        <line x1="2" y1="4" x2="14" y2="4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        <line x1="2" y1="8" x2="14" y2="8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        <line x1="2" y1="12" x2="14" y2="12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        <circle cx="5.5" cy="4" r="1.75" fill="#0f141a" stroke="currentColor" stroke-width="1.3"/>
        <circle cx="10.5" cy="8" r="1.75" fill="#0f141a" stroke="currentColor" stroke-width="1.3"/>
        <circle cx="6" cy="12" r="1.75" fill="#0f141a" stroke="currentColor" stroke-width="1.3"/>
      </svg>
      {#if filterActive}
        <span class="filter-dot"></span>
      {/if}
    </button>
  </div>

  {#if filterOpen}
    <div class="filter-panel" transition:slide={{ duration: 180 }}>
      <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
      <div class="fp-row" on:click={toggleViewMode}>
        <span class="fp-label">Popular</span>
        <div class="toggle-switch" class:toggle-on={localViewMode === 'popular'}>
          <div class="toggle-knob"></div>
        </div>
      </div>

      <div class="fp-selects">
        {#if localViewMode === 'popular'}
          <select class="fp-select" bind:value={localDays} on:change={emitFilter}>
            <option value={0}>Ever</option>
            <option value={20}>20 days</option>
            <option value={30}>30 days</option>
            <option value={60}>60 days</option>
            <option value={90}>90 days</option>
            <option value={180}>180 days</option>
            <option value={365}>365 days</option>
          </select>
        {/if}
        <div class="fp-sort-wrap">
          <span class="fp-sort-label">Sort</span>
          <select class="fp-select" bind:value={localSortBy} on:change={setSortBy}>
            <option value="downloads">↓ Downloads</option>
            <option value="views">◉ Views</option>
            <option value="favorites">♥ Favorites</option>
          </select>
        </div>
      </div>

      {#if localViewMode === 'popular' && statsCount !== null}
        <div class="fp-count">{statsCount.toLocaleString()} presets</div>
      {/if}
    </div>
  {/if}
</div>

<div class="list custom-scroll">
  {#if loading}
    <p class="status">Loading...</p>
  {:else if error}
    <p class="status error">{error}</p>
  {:else if classes.length === 0}
    <p class="status">Open ⚙ to set album directory</p>
  {:else if sorted.length === 0}
    <p class="status">No results</p>
  {:else}
    {#each sorted as cls (cls.name)}
      <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
      <div
        animate:flip={{ duration: 220 }}
        in:fly={{ y: 8, duration: 180 }}
        class="class-row"
      >
        <button
          class="class-btn"
          class:active={cls.name === selectedClass}
          on:click={() => dispatch('select', cls.name)}
        >
          <span class="cls-name">{cls.name}</span>
          <div class="cls-right">
            {#if cls.preset_count > 0}
              <span class="count">{cls.preset_count}</span>
            {/if}
            <span
              class="heart"
              class:active={favorites.has(cls.name)}
              on:click={(e) => toggleFavorite(cls.name, e)}
              title="Pin to top"
            >♥</span>
            {#if cls.icon_svg}
              <span class="cls-icon">{@html cls.icon_svg}</span>
            {/if}
          </div>
        </button>
      </div>
    {/each}
  {/if}
</div>

<style>
  .search-box {
    padding: 12px 16px;
    border-bottom: 1px solid rgba(26, 35, 44, 0.5);
    flex-shrink: 0;
  }

  .search-row {
    display: flex;
    gap: 6px;
    align-items: center;
  }

  .search-input {
    flex: 1;
    background: #0f141a;
    border: 1px solid #1a232c;
    border-radius: 4px;
    padding: 8px 12px;
    font-size: 11px;
    font-family: monospace;
    letter-spacing: 0.1em;
    color: #fff;
    outline: none;
    transition: border-color 0.2s;
    min-width: 0;
  }

  .search-input::placeholder { color: #64748b; }
  .search-input:focus { border-color: #ffcc4d; }

  .filter-btn {
    position: relative;
    width: 34px;
    height: 34px;
    flex-shrink: 0;
    background: #0f141a;
    border: 1px solid #1a232c;
    border-radius: 6px;
    color: #475569;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: border-color 0.2s, color 0.2s, background 0.2s;
  }

  .filter-btn:hover {
    border-color: #334155;
    color: #94a3b8;
    background: #141c27;
  }

  .filter-btn.filter-active {
    border-color: rgba(255, 204, 77, 0.5);
    color: #ffcc4d;
    background: rgba(255, 204, 77, 0.06);
  }

  .filter-dot {
    position: absolute;
    top: 5px;
    right: 5px;
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: #ffcc4d;
    box-shadow: 0 0 6px rgba(255, 204, 77, 0.7);
  }

  .filter-panel {
    margin-top: 8px;
    background: #080d14;
    border: 1px solid #1a2535;
    border-radius: 6px;
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    overflow: hidden;
  }

  .fp-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    cursor: pointer;
  }

  .fp-label {
    font-size: 11px;
    color: #64748b;
    letter-spacing: 0.06em;
  }

  .toggle-switch {
    width: 28px;
    height: 15px;
    border-radius: 8px;
    background: #1a2535;
    border: 1px solid #243040;
    position: relative;
    transition: background 0.2s, border-color 0.2s;
    flex-shrink: 0;
  }

  .toggle-switch.toggle-on {
    background: rgba(255, 204, 77, 0.2);
    border-color: rgba(255, 204, 77, 0.4);
  }

  .toggle-knob {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: #475569;
    transition: transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1), background 0.2s;
  }

  .toggle-on .toggle-knob {
    transform: translateX(13px);
    background: #ffcc4d;
  }

  .fp-selects {
    display: flex;
    gap: 5px;
    align-items: center;
  }

  .fp-sort-wrap {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 5px;
    min-width: 0;
  }

  .fp-sort-label {
    font-size: 9px;
    font-family: monospace;
    letter-spacing: 0.1em;
    color: #334155;
    text-transform: uppercase;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .fp-select {
    flex: 1;
    min-width: 0;
    width: 100%;
    background: #0a0d14;
    border: 1px solid #1a2535;
    border-radius: 4px;
    color: #64748b;
    font-size: 10px;
    font-family: monospace;
    letter-spacing: 0.04em;
    padding: 5px 6px;
    cursor: pointer;
    outline: none;
    appearance: none;
    transition: border-color 0.15s, color 0.15s;
  }

  .fp-select:hover,
  .fp-select:focus {
    border-color: rgba(255, 204, 77, 0.3);
    color: #94a3b8;
  }

  .fp-select option {
    background: #0a0d14;
    color: #94a3b8;
  }

  .fp-count {
    font-size: 10px;
    font-family: monospace;
    color: #2d3f52;
    letter-spacing: 0.06em;
    text-align: right;
  }


  .list {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .status {
    font-size: 11px;
    color: #64748b;
    text-align: center;
    padding: 24px 8px;
    letter-spacing: 0.05em;
  }

  .error { color: #f87171; }

  .class-row {
    width: 100%;
  }

  .class-btn {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 10px 16px;
    text-align: left;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: #94a3b8;
    background: transparent;
    border: none;
    border-left: 3px solid transparent;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    gap: 8px;
  }

  .class-btn:hover {
    color: #fff;
    background: #0f141a;
  }

  .class-btn.active {
    border-left-color: #ffcc4d;
    background: linear-gradient(90deg, rgba(255, 204, 77, 0.08) 0%, transparent 100%);
    color: #ffcc4d;
  }

  .cls-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .cls-right {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .count {
    font-size: 10px;
    font-family: monospace;
    color: #64748b;
  }

  .class-btn.active .count { color: rgba(255, 204, 77, 0.6); }

  .heart {
    font-size: 11px;
    color: #1a232c;
    cursor: pointer;
    transition: color 0.15s, transform 0.15s;
    line-height: 1;
    padding: 2px;
    user-select: none;
  }

  .heart:hover { color: rgba(255, 100, 100, 0.5); transform: scale(1.2); }
  .heart.active { color: #e05252; }
  .heart.active:hover { color: #ff6b6b; }

  .cls-icon {
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #0a0d12;
    padding: 2px;
    border-radius: 3px;
    border: 1px solid #1a232c;
    flex-shrink: 0;
    transition: border-color 0.2s;
    overflow: hidden;
  }

  .cls-icon :global(svg) {
    width: 100%;
    height: 100%;
  }

  .class-btn:hover .cls-icon,
  .class-btn.active .cls-icon {
    border-color: rgba(255, 204, 77, 0.4);
  }
</style>
