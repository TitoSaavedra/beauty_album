<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { fly, slide } from 'svelte/transition';
  import { flip } from 'svelte/animate';
  import { _ } from 'svelte-i18n';
  import { listen } from '@tauri-apps/api/event';
  import type { ClassEntry, PopularStats } from '../../tauri/album';
  import { getClasses, getPopularClasses, getClassFavorites, setClassFavorite } from '../../tauri/album';

  export let selectedClass: string | null = null;
  export let filterSortBy: 'downloads' | 'views' | 'favorites' = 'downloads';
  export let viewMode: 'presets' | 'popular' = 'presets';
  export let popularStats: PopularStats | null = null;
  export let regions: string[] = [];

  const dispatch = createEventDispatcher<{
    select: ClassEntry;
    filterChange: { sortBy: 'downloads' | 'views' | 'favorites'; sinceTs: number; region?: string };
    searchChange: string;
    modeChange: 'presets' | 'popular';
  }>();

  let classes: ClassEntry[] = [];
  let loading = false;
  let error = '';
  let favorites: Set<string> = new Set();

  onMount(async () => {
    const unlisten = await listen<{ class_id: number; count: number; is_popular: boolean }>('class_count_updated', ({ payload }) => {
      classes = classes.map(c =>
        c.class_id === payload.class_id
          ? payload.is_popular
            ? { ...c, popular_count: payload.count }
            : { ...c, preset_count: payload.count }
          : c
      );
    });

    try {
      const favs = await getClassFavorites();
      favorites = new Set(favs);
    } catch { /* ignore on first launch */ }

    loading = true;
    try {
      const [albumClasses, popularClasses] = await Promise.all([getClasses(), getPopularClasses()]);
      const popularMap = new Map(popularClasses.map(c => [c.class_id, c.preset_count]));
      classes = albumClasses.map(c => ({ ...c, popular_count: popularMap.get(c.class_id) ?? 0 }));
      if (classes[0]) dispatch('select', classes[0]);
    } catch (e) { error = String(e); }
    finally { loading = false; }

    return unlisten;
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
  let localRegion = '';

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
    dispatch('filterChange', { sortBy: localSortBy, sinceTs: sinceTs(), region: localRegion });
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
        <span class="fp-field-label">{$_('sidebar.popular_toggle')}</span>
        <div class="toggle-switch" class:toggle-on={localViewMode === 'popular'}>
          <div class="toggle-knob"></div>
        </div>
      </div>

      {#if localViewMode === 'popular'}
        <div class="fp-field">
          <span class="fp-field-label">{$_('sidebar.time_label')}</span>
          <select class="fp-select" bind:value={localDays} on:change={emitFilter}>
            <option value={0}>{$_('sidebar.all')}</option>
            <option value={20}>{$_('sidebar.days_20')}</option>
            <option value={30}>{$_('sidebar.days_30')}</option>
            <option value={60}>{$_('sidebar.days_60')}</option>
            <option value={90}>{$_('sidebar.days_90')}</option>
            <option value={180}>{$_('sidebar.days_180')}</option>
            <option value={365}>{$_('sidebar.days_365')}</option>
          </select>
        </div>
        <div class="fp-field">
          <span class="fp-field-label">{$_('sidebar.region_label')}</span>
          <select class="fp-select" bind:value={localRegion} on:change={emitFilter}>
            <option value="">{$_('sidebar.all_regions')}</option>
            {#each regions as r}
              <option value={r}>{r.toUpperCase()}</option>
            {/each}
          </select>
        </div>
      {/if}

      <div class="fp-field">
        <span class="fp-field-label">{$_('sidebar.sort_by_label')}</span>
        <select class="fp-select" bind:value={localSortBy} on:change={setSortBy}>
          <option value="downloads">{$_('sidebar.sort_downloads')}</option>
          <option value="views">{$_('sidebar.sort_views')}</option>
          <option value="favorites">{$_('sidebar.sort_favorites')}</option>
        </select>
      </div>
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
          on:click={() => dispatch('select', cls)}
        >
          <span class="cls-name">{cls.name}</span>
          <div class="cls-right">
            {#if viewMode === 'popular' ? (cls.popular_count ?? 0) > 0 : cls.preset_count > 0}
              <span class="count">{viewMode === 'popular' ? (cls.popular_count ?? 0) : cls.preset_count}</span>
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

<style lang="scss">
  @use './ClassList.scss';
</style>
