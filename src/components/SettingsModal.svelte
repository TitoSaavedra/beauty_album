<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import { saveConfig, openLogs } from '../tauri/album';
  import type { AppConfig } from '../tauri/album';

  export let config: AppConfig;

  const dispatch = createEventDispatcher<{ save: AppConfig; close: void }>();

  let bdo_docs_dir = config.bdo_docs_dir;
  let saving = false;
  let error = '';

  $: previewPaths = bdo_docs_dir.trim()
    ? [
        { label: 'Presets', path: bdo_docs_dir.trim() + '\\Beauty Album\\Presets' },
        { label: 'Customization', path: bdo_docs_dir.trim() + '\\Customization' },
        { label: 'Input', path: bdo_docs_dir.trim() + '\\Beauty Album\\to_download' },
        { label: 'Logs', path: bdo_docs_dir.trim() + '\\Beauty Album\\Logs' },
      ]
    : [];

  async function pickFolder() {
    const selected = await openDialog({ directory: true, multiple: false });
    if (typeof selected === 'string') bdo_docs_dir = selected;
  }

  async function save() {
    saving = true;
    error = '';
    try {
      const updated: AppConfig = { bdo_docs_dir: bdo_docs_dir.trim(), cf_clearance: config.cf_clearance ?? '' };
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
      <label class="field-label" for="bdo-docs">Black Desert Documents Directory</label>
      <p class="field-hint">Select the Black Desert Online documents folder — all subdirectories are created automatically.</p>
      <div class="input-row">
        <input
          id="bdo-docs"
          class="field-input"
          type="text"
          bind:value={bdo_docs_dir}
          placeholder="e.g. C:\Users\You\Documents\Black Desert"
        />
        <button class="btn-browse" on:click={pickFolder} title="Browse">...</button>
      </div>
    </div>

    {#if previewPaths.length > 0}
      <div class="derived-paths">
        {#each previewPaths as { label, path }}
          <div class="derived-row">
            <span class="derived-label">{label}</span>
            <span class="derived-path">{path}</span>
          </div>
        {/each}
      </div>
    {/if}

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
    width: 520px;
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

  .derived-paths {
    display: flex;
    flex-direction: column;
    gap: 4px;
    background: rgba(15, 18, 25, 0.7);
    border: 1px solid rgba(100, 110, 130, 0.2);
    border-radius: 5px;
    padding: 10px 12px;
  }

  .derived-row {
    display: flex;
    gap: 10px;
    font-size: 11px;
    font-family: monospace;
    line-height: 1.6;
  }

  .derived-label {
    color: var(--muted);
    opacity: 0.7;
    min-width: 90px;
    flex-shrink: 0;
  }

  .derived-path {
    color: #94a3b8;
    word-break: break-all;
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
