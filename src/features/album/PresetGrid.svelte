<script lang="ts">
  import { fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import type { PresetEntry } from '../../tauri/album';
  import { discardPreset } from '../../tauri/album';
  import PresetCard from './PresetCard.svelte';

  export let presets: PresetEntry[] = [];
  export let selectedClass: string | null = null;
  export let loading = false;
  export let error = '';
  export let isPopular = false;
  export let filterShowDownloaded = true;
  export let searchQuery = '';
  export let hasMore = false;
  export let loadingMore = false;

  let discarded = new Set<string>();

  $: localPresets = (() => {
    let list = [...presets];

    if (!filterShowDownloaded) {
      list = list.filter(p => !p.is_downloaded);
    }

    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      list = list.filter(p =>
        (p.title ?? p.name ?? '').toLowerCase().includes(q) ||
        (p.creator ?? '').toLowerCase().includes(q)
      );
    }

    if (isPopular) {
      list = list.filter(p => !discarded.has(p.preset_id ?? ''));
      list.sort((a, b) => (b.is_downloaded ? 1 : 0) - (a.is_downloaded ? 1 : 0));
    }

    return list;
  })();

  $: if (presets) discarded = new Set<string>();

  async function handleDiscard(e: CustomEvent<string>) {
    const id = e.detail;
    discarded = new Set([...discarded, id]);
    try {
      await discardPreset(id);
    } catch {
      // non-fatal
    }
  }
</script>

{#if !selectedClass}
  <div class="state-msg">
    <div class="state-hint">Select a class to browse presets</div>
  </div>
{:else if loading}
  <div class="grid">
    {#each Array.from({length: 12}) as _, i}
      <div class="skel-card" style="animation-delay: {i * 50}ms"></div>
    {/each}
  </div>
{:else if error}
  <div class="state-msg">
    <div class="state-hint error">{error}</div>
  </div>
{:else if localPresets.length === 0}
  <div class="state-msg">
    <div class="state-hint">No presets found in this class</div>
  </div>
{:else}
  <div class="grid">
    {#each localPresets as preset, i (preset.preset_id ?? i)}
      <div in:fly={{ y: 20, duration: 220, easing: cubicOut }}>
        <PresetCard {preset} {selectedClass} on:discard={handleDiscard} />
      </div>
    {/each}
    {#if loadingMore}
      {#each Array.from({length: 6}) as _, i}
        <div class="skel-card" style="animation-delay: {i * 60}ms"></div>
      {/each}
    {/if}
  </div>
{/if}

<style>
  .state-msg {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 40vh;
  }

  .state-hint {
    font-size: 12px;
    letter-spacing: 0.15em;
    text-transform: uppercase;
    color: #64748b;
  }

  .state-hint.loading {
    color: rgba(255, 204, 77, 0.4);
    animation: pulse 1.5s ease-in-out infinite;
  }

  .state-hint.error { color: #f87171; }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 24px;
    align-content: start;
  }

  .skel-card {
    aspect-ratio: 4 / 5;
    border-radius: 8px;
    border: 1px solid #1a232c;
    background: linear-gradient(90deg, #0d1219 25%, #141c25 50%, #0d1219 75%);
    background-size: 200% 100%;
    animation: shimmer 1.6s ease-in-out infinite;
  }

  @keyframes shimmer {
    0%   { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }
</style>
