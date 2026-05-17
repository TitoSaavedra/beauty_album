<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import { saveConfig, openLogs } from '../tauri/album';
  import type { AppConfig } from '../tauri/album';

  export let config: AppConfig;

  const dispatch = createEventDispatcher<{ save: AppConfig; close: void }>();

  let albumDir = config.album_dir;
  let bdoOutputDir = config.bdo_output_dir;
  let albumInputDir = config.album_input_dir;
  let saving = false;
  let error = '';

  async function pickFolder(target: 'album' | 'bdo' | 'input') {
    const selected = await openDialog({ directory: true, multiple: false });
    if (typeof selected === 'string') {
      if (target === 'album') albumDir = selected;
      else if (target === 'bdo') bdoOutputDir = selected;
      else albumInputDir = selected;
    }
  }

  async function save() {
    saving = true;
    error = '';
    try {
      const updated: AppConfig = {
        album_dir: albumDir.trim(),
        bdo_output_dir: bdoOutputDir.trim(),
        album_input_dir: albumInputDir.trim(),
      };
      await saveConfig(updated);
      dispatch('save', updated);
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

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
  <div class="modal" on:click|stopPropagation>
    <h2 class="modal-title">Settings</h2>

    <div class="field">
      <label class="field-label" for="album-dir">Album Directory</label>
      <div class="input-row">
        <input
          id="album-dir"
          class="field-input"
          type="text"
          bind:value={albumDir}
          placeholder="Path to beauty_album folder"
        />
        <button class="btn-browse" on:click={() => pickFolder('album')} title="Browse">...</button>
      </div>
    </div>

    <div class="field">
      <label class="field-label" for="bdo-output">BDO Output Directory</label>
      <div class="input-row">
        <input
          id="bdo-output"
          class="field-input"
          type="text"
          bind:value={bdoOutputDir}
          placeholder="Path to BDO output folder"
        />
        <button class="btn-browse" on:click={() => pickFolder('bdo')} title="Browse">...</button>
      </div>
    </div>

    <div class="field">
      <label class="field-label" for="album-input">Album Input Directory</label>
      <p class="field-hint">Folder containing preset files (.pab) to be downloaded from Garmoth</p>
      <div class="input-row">
        <input
          id="album-input"
          class="field-input"
          type="text"
          bind:value={albumInputDir}
          placeholder="Path to preset input folder"
        />
        <button class="btn-browse" on:click={() => pickFolder('input')} title="Browse">...</button>
      </div>
    </div>

    {#if error}
      <p class="error-msg">{error}</p>
    {/if}

    <div class="actions">
      <button class="btn-cancel" on:click={() => openLogs().catch(() => {})}>Open Logs</button>
      <button class="btn-cancel" on:click={close}>Cancel</button>
      <button class="btn-gold" on:click={save} disabled={saving}>
        {saving ? 'Saving…' : 'Save'}
      </button>
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.65);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .modal {
    background: #1a1e28;
    border: 1px solid rgba(184, 134, 11, 0.4);
    border-radius: 10px;
    padding: 28px 32px;
    width: 480px;
    display: flex;
    flex-direction: column;
    gap: 20px;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.7);
  }

  .modal-title {
    font-size: 16px;
    font-weight: bold;
    letter-spacing: 1.5px;
    text-transform: uppercase;
    color: var(--gold);
    margin: 0;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .field-hint {
    font-size: 11px;
    color: var(--muted);
    margin: 0;
    opacity: 0.7;
  }

  .field-label {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.8px;
    text-transform: uppercase;
    color: var(--muted);
  }

  .input-row {
    display: flex;
    gap: 6px;
  }

  .field-input {
    flex: 1;
    padding: 8px 12px;
    background: rgba(30, 35, 45, 0.9);
    border: 1px solid rgba(100, 110, 130, 0.4);
    border-radius: 5px;
    color: #e0e6ed;
    font-size: 13px;
    outline: none;
    transition: border-color 0.2s;
  }

  .field-input:focus {
    border-color: var(--gold-dim);
  }

  .btn-browse {
    padding: 0 12px;
    background: rgba(30, 35, 45, 0.9);
    border: 1px solid rgba(100, 110, 130, 0.4);
    border-radius: 5px;
    color: #e0e6ed;
    font-size: 13px;
    font-weight: 600;
    letter-spacing: 1px;
    cursor: pointer;
    flex-shrink: 0;
    transition: border-color 0.2s, background 0.2s;
  }

  .btn-browse:hover {
    border-color: var(--gold-dim);
    background: rgba(184, 134, 11, 0.1);
  }

  .error-msg {
    font-size: 12px;
    color: #ff6b6b;
    margin: 0;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 4px;
  }

  .btn-cancel {
    padding: 7px 18px;
    background: transparent;
    border: 1px solid rgba(100, 110, 130, 0.4);
    border-radius: 5px;
    color: var(--muted);
    font-size: 13px;
    cursor: pointer;
    transition: border-color 0.2s, color 0.2s;
  }

  .btn-cancel:hover {
    border-color: rgba(100, 110, 130, 0.8);
    color: #e0e6ed;
  }
</style>
