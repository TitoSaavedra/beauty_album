<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { fly, slide } from 'svelte/transition';
  import { flip } from 'svelte/animate';
  import { _ } from 'svelte-i18n';
  import type { ClassEntry, PopularStats } from '../../tauri/album';
  import Button from '../../../../shared/components/Button/Button.svelte';
  import Input from '../../../../shared/components/Input/Input.svelte';
  import Select from '../../../../shared/components/Select/Select.svelte';
  import Toggle from '../../../../shared/components/Toggle/Toggle.svelte';
  import { getClasses, getPopularClasses, getClassFavorites, setClassFavorite } from '../../tauri/album';
  import { classes } from '../../stores/classes';

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

  let loading = false;
  let error = '';
  let favorites: Set<string> = new Set();

  onMount(async () => {
    try {
      const favs = await getClassFavorites();
      favorites = new Set(favs);
    } catch { /* ignore on first launch */ }

    loading = true;
    try {
      const [albumClasses, popularClasses] = await Promise.all([getClasses(), getPopularClasses()]);
      const popularMap = new Map(popularClasses.map(c => [c.class_id, c.preset_count]));
      classes.set(albumClasses.map(c => ({ ...c, popular_count: popularMap.get(c.class_id) ?? 0 })));
      if ($classes[0]) dispatch('select', $classes[0]);
    } catch (e) { error = String(e); }
    finally { loading = false; }
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
    ? $classes.filter(c => c.name.toLowerCase().includes(search.toLowerCase()))
    : $classes;

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
    <Input
      bind:value={search}
      on:input={onSearchInput}
      placeholder="Search classes, presets, creators..."
    />
    <Button
      variant="icon"
      active={filterActive}
      title="Filter & Sort"
      on:click={() => filterOpen = !filterOpen}
    >
      <svg width="15" height="15" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
        <line x1="2" y1="4" x2="14" y2="4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        <line x1="2" y1="8" x2="14" y2="8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        <line x1="2" y1="12" x2="14" y2="12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        <circle cx="5.5" cy="4" r="1.75" fill="var(--color-bg-surface)" stroke="currentColor" stroke-width="1.3"/>
        <circle cx="10.5" cy="8" r="1.75" fill="var(--color-bg-surface)" stroke="currentColor" stroke-width="1.3"/>
        <circle cx="6" cy="12" r="1.75" fill="var(--color-bg-surface)" stroke="currentColor" stroke-width="1.3"/>
      </svg>
      {#if filterActive}
        <span class="filter-dot"></span>
      {/if}
    </Button>
  </div>

  {#if filterOpen}
    <div class="filter-panel" transition:slide={{ duration: 180 }}>
      <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
      <div class="fp-row" on:click={toggleViewMode}>
        <span class="fp-field-label">{$_('sidebar.popular_toggle')}</span>
        <Toggle checked={localViewMode === 'popular'} />
      </div>

      {#if localViewMode === 'popular'}
        <div class="fp-field">
          <span class="fp-field-label">{$_('sidebar.time_label')}</span>
          <Select
            bind:value={localDays}
            on:change={emitFilter}
            options={[
              { value: 0,   label: $_('sidebar.all') },
              { value: 20,  label: $_('sidebar.days_20') },
              { value: 30,  label: $_('sidebar.days_30') },
              { value: 60,  label: $_('sidebar.days_60') },
              { value: 90,  label: $_('sidebar.days_90') },
              { value: 180, label: $_('sidebar.days_180') },
              { value: 365, label: $_('sidebar.days_365') },
            ]}
          />
        </div>
        <div class="fp-field">
          <span class="fp-field-label">{$_('sidebar.region_label')}</span>
          <Select
            bind:value={localRegion}
            on:change={emitFilter}
            options={[
              { value: '', label: $_('sidebar.all_regions') },
              ...regions.map(r => ({ value: r, label: r.toUpperCase() }))
            ]}
          />
        </div>
      {/if}

      <div class="fp-field">
        <span class="fp-field-label">{$_('sidebar.sort_by_label')}</span>
        <Select
          bind:value={localSortBy}
          on:change={setSortBy}
          options={[
            { value: 'downloads', label: $_('sidebar.sort_downloads') },
            { value: 'views',     label: $_('sidebar.sort_views') },
            { value: 'favorites', label: $_('sidebar.sort_favorites') },
          ]}
        />
      </div>
    </div>
  {/if}
</div>

<div class="list custom-scroll">
  {#if loading}
    <p class="status">Loading...</p>
  {:else if error}
    <p class="status error">{error}</p>
  {:else if $classes.length === 0}
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
