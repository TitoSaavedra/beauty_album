<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { PresetEntry } from '../../tauri/album';
  import PresetCard from '../PresetCard/PresetCard.svelte';

  export let selectedClass: string;
  export let presets: PresetEntry[] = [];
  export let loading = false;
  export let error = '';

  const dispatch = createEventDispatcher<{ close: void }>();

  function close() {
    dispatch('close');
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
  }
</script>

<svelte:window on:keydown={onKeydown} />

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div class="backdrop" on:click={close}>
  <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
  <div class="modal-panel" on:click|stopPropagation>

    <div class="modal-header">
      <div class="header-left">
        <h2 class="class-title">{selectedClass}</h2>
        {#if !loading}
          <span class="preset-count">{presets.length} preset{presets.length !== 1 ? 's' : ''}</span>
        {/if}
      </div>
      <button class="close-btn" on:click={close}>✕</button>
    </div>

    <div class="modal-body">
      {#if loading}
        <div class="state-msg loading">Loading presets...</div>
      {:else if error}
        <div class="state-msg error">{error}</div>
      {:else if presets.length === 0}
        <div class="state-msg">No presets found</div>
      {:else}
        <div class="skill-grid">
          {#each presets as preset (preset.preset_id ?? preset.image_paths?.[0] ?? Math.random())}
            <PresetCard {preset} />
          {/each}
        </div>
      {/if}
    </div>

  </div>
</div>
