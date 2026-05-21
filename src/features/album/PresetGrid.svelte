<script lang="ts">
  import { fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import type { PresetEntry } from '../../tauri/album';
  import PresetCard from './PresetCard.svelte';

  export let presets: PresetEntry[] = [];
  export let selectedClass: string | null = null;
  export let loading = false;
  export let error = '';
</script>

{#if !selectedClass}
  <div class="state-msg">
    <div class="state-hint">Select a class to browse presets</div>
  </div>
{:else if loading}
  <div class="state-msg">
    <div class="state-hint loading">Loading presets...</div>
  </div>
{:else if error}
  <div class="state-msg">
    <div class="state-hint error">{error}</div>
  </div>
{:else if presets.length === 0}
  <div class="state-msg">
    <div class="state-hint">No presets found in this class</div>
  </div>
{:else}
  <div class="grid">
    {#each presets as preset, i (preset.preset_id ?? i)}
      <div in:fly={{ y: 28, duration: 350, delay: Math.min(i * 30, 600), easing: cubicOut }}>
        <PresetCard {preset} />
      </div>
    {/each}
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
</style>
