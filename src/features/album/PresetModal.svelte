<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { PresetEntry } from '../../tauri/album';
  import PresetCard from './PresetCard.svelte';

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

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.78);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 50;
    padding: 24px;
  }

  .modal-panel {
    background: rgba(20, 24, 32, 0.97);
    border: 1px solid rgba(184, 134, 11, 0.5);
    box-shadow:
      inset 0 0 20px rgba(184, 134, 11, 0.08),
      0 24px 64px rgba(0, 0, 0, 0.85);
    border-radius: 8px;
    width: 90vw;
    max-width: 1280px;
    max-height: 88vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 24px;
    border-bottom: 2px solid rgba(184, 134, 11, 0.3);
    flex-shrink: 0;
    background: rgba(184, 134, 11, 0.05);
  }

  .header-left {
    display: flex;
    align-items: baseline;
    gap: 14px;
  }

  .class-title {
    font-size: 20px;
    font-weight: bold;
    color: rgba(218, 165, 32, 1);
    letter-spacing: 2px;
    text-transform: uppercase;
    text-shadow: 0 0 10px rgba(184, 134, 11, 0.3);
    margin: 0;
  }

  .preset-count {
    font-size: 13px;
    color: rgba(160, 168, 184, 0.7);
  }

  .close-btn {
    background: rgba(30, 35, 45, 0.6);
    border: 1px solid rgba(100, 110, 130, 0.3);
    border-radius: 5px;
    color: rgba(160, 168, 184, 0.7);
    font-size: 14px;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: border-color 0.2s, color 0.2s;
    flex-shrink: 0;
  }

  .close-btn:hover {
    border-color: rgba(184, 134, 11, 0.5);
    color: rgba(218, 165, 32, 1);
  }

  .modal-body {
    flex: 1;
    overflow-y: auto;
    padding: 20px 24px;
  }

  .skill-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(110px, 1fr));
    gap: 15px;
    align-content: start;
  }

  .state-msg {
    text-align: center;
    padding: 80px 0;
    color: rgba(160, 168, 184, 0.3);
    font-size: 15px;
  }

  .state-msg.loading {
    color: rgba(184, 134, 11, 0.5);
    animation: pulse 1.5s ease-in-out infinite;
  }

  .state-msg.error {
    color: #ff6b6b;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
</style>
