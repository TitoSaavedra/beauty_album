<script lang="ts">
  import { fade, scale } from 'svelte/transition';
  import { _ } from 'svelte-i18n';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { presetDetail } from '../../stores/presetDetail';
  import { confirmDialog } from '../../../../shared/stores/confirmDialog';
  import { toast } from '../../../../shared/stores/toast';
  import { wantedPresets } from '../../stores/wantedPresets';
  import { injectPreset, discardPreset, openUrl } from '../../tauri/album';

  $: preset = $presetDetail;
  $: images = preset?.image_paths?.map(p => convertFileSrc(p)) ?? [];
  $: activeImage = images[0] ?? '';

  $: if (preset) activeImage = images[0] ?? '';

  $: title    = preset?.title    ?? preset?.name     ?? preset?.preset_id ?? 'Untitled';
  $: creator  = preset?.creator  ?? 'Anonymous';
  $: date = preset?.date
      ?? ((preset as any)?.created_at
          ? new Date(((preset as any).created_at as number) * 1000).toLocaleDateString('en-CA')
          : null);
  $: id       = preset?.preset_id ?? '—';
  $: downloads = preset?.downloads ?? 0;
  $: views     = preset?.views     ?? 0;
  $: favorites = preset?.favorites ?? 0;
  $: className = (preset as any)?.class_display ?? '';
  $: isPopular = !!(preset as any)?.is_popular;
  $: isDownloaded = !!(preset as any)?.is_downloaded;
  $: isWanted = isPopular && $wantedPresets.has(id);
  $: syncedAt = preset?.updated_at
      ? new Date(preset.updated_at * 1000).toLocaleDateString('en-CA')
      : null;

  function close() { presetDetail.set(null); }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
  }

  function inject() {
    if (!preset?.download_path) return;
    confirmDialog.show({
      title: $_('confirm.inject_title'),
      text: $_('confirm.inject_text'),
      confirmLabel: $_('preset.export_to_bdo'),
      onConfirm: async () => {
        await injectPreset(preset!.download_path!);
        presetDetail.set(null);
        toast.show('Preset injected successfully');
      },
    });
  }

  function openOnGarmoth() {
    openUrl(`https://garmoth.com/beauty-album/preset/${id}`);
  }

  function discard() {
    const presetId = id;
    confirmDialog.show({
      title: $_('confirm.discard_title'),
      text: $_('confirm.discard_text'),
      confirmLabel: $_('preset.discard'),
      onConfirm: async () => {
        presetDetail.set(null);
        try { await discardPreset(presetId); } catch { /* non-fatal */ }
      },
    });
  }

  function onImgError(e: Event) {
    (e.currentTarget as HTMLImageElement).style.display = 'none';
  }
</script>

<svelte:window on:keydown={onKeydown} />

{#if preset}
  <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
  <div class="backdrop" on:click={close} transition:fade={{ duration: 200 }}>
    <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
    <div class="modal" on:click|stopPropagation transition:scale={{ duration: 250, start: 0.95 }}>

      <!-- LEFT: image preview -->
      <div class="panel-left">
        <div class="main-preview">
          {#if activeImage}
            <img src={activeImage} alt={title} class="main-img" on:error={onImgError} />
          {:else}
            <span class="no-media">NO MEDIA</span>
          {/if}
        </div>
        {#if images.length > 1}
          <div class="thumbs custom-scroll">
            {#each images as img, i}
              <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
              <div
                class="thumb"
                class:thumb-active={img === activeImage}
                on:click={() => (activeImage = img)}
              >
                <img src={img} alt="" class="thumb-img" on:error={onImgError} />
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- RIGHT: info -->
      <div class="panel-right custom-scroll">

        <div class="panel-header">
          <div class="header-top">
            <div class="header-meta">
              {#if className}
                <span class="class-tag">{className}</span>
              {/if}
              <span class="preset-id">#{id}</span>
            </div>
            <button class="close-btn" on:click={close}>✕</button>
          </div>
          <h2 class="preset-title">{title}</h2>
          <div class="creator-row">
            <span class="creator-at">@</span><span class="creator-name">{creator}</span>
          </div>
        </div>

        <div class="stats-row">
          <div class="stat-item">
            <span class="stat-icon">↓</span>
            <span class="stat-val">{Number(downloads).toLocaleString()}</span>
            <span class="stat-lbl">Downloads</span>
          </div>
          <div class="stat-divider"></div>
          <div class="stat-item">
            <span class="stat-icon">◉</span>
            <span class="stat-val">{Number(views).toLocaleString()}</span>
            <span class="stat-lbl">Views</span>
          </div>
          <div class="stat-divider"></div>
          <div class="stat-item stat-fav">
            <span class="stat-icon">♥</span>
            <span class="stat-val">{Number(favorites).toLocaleString()}</span>
            <span class="stat-lbl">Favorites</span>
          </div>
        </div>

        <div class="meta-grid">
          {#if date}
            <span class="meta-key">Uploaded</span>
            <span class="meta-val">{date}</span>
          {/if}
          {#if syncedAt}
            <span class="meta-key">Synced</span>
            <span class="meta-val">{syncedAt}</span>
          {/if}
          {#if isDownloaded}
            <span class="meta-key">Status</span>
            <span class="meta-val status-downloaded">Downloaded</span>
          {/if}
        </div>

        <div class="actions">
          {#if isPopular && !isDownloaded}
            <div class="want-row">
              <button
                class="btn-want"
                class:btn-want-active={isWanted}
                on:click={() => wantedPresets.toggle(id)}
                title={isWanted ? $_('preset.wished') : $_('preset.wish')}
              >
                <span class="btn-want-icon">♥</span>
              </button>
              <button class="btn-discard-sm" title={$_('preset.discard')} on:click={discard}>✕</button>
            </div>
          {/if}
          {#if isPopular}
            <button class="btn-garmoth" on:click={openOnGarmoth}>
              <span class="btn-icon">◈</span> {$_('preset.open_garmoth')}
            </button>
          {:else if preset.download_path}
            <button class="btn-inject bdo-gold-gradient" on:click={inject}>
              <span class="btn-icon">⇪</span> {$_('preset.export_to_bdo')}
            </button>
          {:else}
            <div class="no-download">{$_('preset.no_binary_linked')}</div>
          {/if}
        </div>

      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.8);
    z-index: 50;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 16px;
    backdrop-filter: blur(6px);
  }

  .modal {
    background: #090c11;
    border: 1px solid #1e2a38;
    border-radius: 10px;
    max-width: 880px;
    width: 100%;
    max-height: 88vh;
    overflow: hidden;
    display: flex;
    flex-direction: row;
    box-shadow: 0 32px 80px rgba(0, 0, 0, 0.85), 0 0 0 1px rgba(255,204,77,0.04);
  }

  .panel-left {
    flex: 0 0 70%;
    background: #05070a;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 20px;
    border-right: 1px solid #1a2332;
  }

  .main-preview {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #070a0e;
    border-radius: 6px;
    overflow: hidden;
    min-height: 0;
  }

  .main-img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    border-radius: 4px;
  }

  .no-media {
    font-size: 11px;
    letter-spacing: 0.15em;
    color: #334155;
    text-transform: uppercase;
  }

  .thumbs {
    display: flex;
    gap: 8px;
    overflow-x: auto;
    padding-bottom: 2px;
    flex-shrink: 0;
  }

  .thumb {
    width: 58px;
    height: 58px;
    flex-shrink: 0;
    background: #090c10;
    border-radius: 4px;
    border: 1px solid #1a2332;
    cursor: pointer;
    overflow: hidden;
    transition: border-color 0.18s;
  }

  .thumb:hover { border-color: rgba(255, 204, 77, 0.4); }
  .thumb-active { border-color: #ffcc4d !important; }

  .thumb-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .panel-right {
    flex: 0 0 30%;
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    background: #0a0d12;
  }

  .panel-header {
    padding: 16px 16px 12px;
    border-bottom: 1px solid #141c27;
  }

  .header-top {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }

  .header-meta {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .class-tag {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: #ffcc4d;
    background: rgba(255, 204, 77, 0.08);
    border: 1px solid rgba(255, 204, 77, 0.18);
    padding: 3px 8px;
    border-radius: 3px;
  }

  .preset-id {
    font-size: 10px;
    font-family: monospace;
    color: #334155;
    letter-spacing: 0.05em;
  }

  .close-btn {
    background: none;
    border: 1px solid #1a2332;
    border-radius: 4px;
    color: #475569;
    font-size: 13px;
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: border-color 0.18s, color 0.18s;
    flex-shrink: 0;
  }

  .close-btn:hover { border-color: #334155; color: #94a3b8; }

  .preset-title {
    font-size: 14px;
    font-weight: 700;
    color: #f1f5f9;
    letter-spacing: 0.02em;
    line-height: 1.3;
    margin-bottom: 6px;
  }

  .creator-row {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .creator-at {
    font-size: 12px;
    color: #ffcc4d;
    font-weight: 700;
    opacity: 0.7;
  }

  .creator-name {
    font-size: 12px;
    color: #94a3b8;
    font-weight: 500;
  }

  .stats-row {
    display: flex;
    align-items: stretch;
    padding: 12px 16px;
    border-bottom: 1px solid #141c27;
    gap: 0;
  }

  .stat-item {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 3px;
  }

  .stat-divider {
    width: 1px;
    background: #141c27;
    margin: 2px 0;
  }

  .stat-icon {
    font-size: 12px;
    color: #334155;
    line-height: 1;
  }

  .stat-val {
    font-size: 16px;
    font-weight: 700;
    font-family: monospace;
    color: #e2e8f0;
    letter-spacing: -0.02em;
  }

  .stat-lbl {
    font-size: 9px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: #334155;
  }

  .stat-fav .stat-icon { color: #7c3f5e; }
  .stat-fav .stat-val { color: #c084a0; }

  .meta-grid {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 6px 12px;
    padding: 12px 16px;
    border-bottom: 1px solid #141c27;
    align-items: center;
  }

  .meta-key {
    font-size: 10px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: #334155;
    white-space: nowrap;
  }

  .meta-val {
    font-size: 12px;
    color: #94a3b8;
    font-weight: 500;
    font-family: monospace;
  }

  .status-downloaded {
    color: #4ade80;
    font-family: inherit;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    font-size: 10px;
  }

  .actions {
    padding: 12px 16px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: auto;
  }

  .want-row {
    display: flex;
    justify-content: flex-end;
    gap: 6px;
  }

  .btn-want {
    flex: 0 0 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 5px;
    height: 28px;
    border-radius: 4px;
    border: 1px solid var(--color-accent-quaternary);
    opacity: 0.4;
    background: rgba(168, 85, 247, 0.04);
    color: var(--color-text-secondary);
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    cursor: pointer;
    transition: all 0.18s;
  }

  .btn-want-icon { font-size: 11px; line-height: 1; }

  .btn-want:hover {
    opacity: 0.8;
    background: transparent;
    color: var(--color-accent-quaternary);
  }

  .btn-want.btn-want-active {
    opacity: 1;
    background: transparent;
    border-color: var(--color-accent-quaternary);
    color: var(--color-accent-quaternary);
  }

  .btn-discard-sm {
    width: 28px;
    height: 28px;
    flex-shrink: 0;
    border-radius: 4px;
    border: 1px solid rgba(220, 60, 60, 0.15);
    background: transparent;
    color: #475569;
    font-size: 11px;
    cursor: pointer;
    transition: background 0.18s, border-color 0.18s, color 0.18s;
  }

  .btn-discard-sm:hover {
    background: rgba(220, 60, 60, 0.1);
    border-color: rgba(220, 60, 60, 0.3);
    color: #f87171;
  }

  .btn-garmoth {
    width: 100%;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 5px;
    border: 1px solid rgba(255, 204, 77, 0.22);
    border-radius: 4px;
    background: transparent;
    color: #ffcc4d;
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    cursor: pointer;
    transition: background 0.18s, border-color 0.18s;
  }

  .btn-garmoth:hover {
    background: rgba(255, 204, 77, 0.07);
    border-color: rgba(255, 204, 77, 0.4);
  }

  .btn-inject {
    width: 100%;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 5px;
    border: none;
    border-radius: 4px;
    color: #0a0c10;
    font-weight: 700;
    font-size: 9px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    cursor: pointer;
    transition: filter 0.18s;
  }

  .btn-inject:hover { filter: brightness(1.1); }

  .btn-icon { font-size: 13px; line-height: 1; }

  .no-download {
    text-align: center;
    font-size: 11px;
    color: #334155;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    padding: 8px 0;
  }
</style>
