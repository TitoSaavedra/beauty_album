<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import type { PresetEntry } from '../../tauri/album';
  import { presetDetail } from '../../stores/presetDetail';
  import { confirmDialog } from '../../stores/confirmDialog';
  import { wantedPresets } from '../../stores/wantedPresets';

  export let preset: PresetEntry;
  export let selectedClass: string | null = null;

  const dispatch = createEventDispatcher<{ discard: string }>();

  $: firstImage = preset.image_paths?.[0] ? convertFileSrc(preset.image_paths[0]) : null;
  $: isSkeleton = !preset.image_paths?.length;
  $: title = preset.title ?? preset.name ?? preset.preset_id ?? 'Untitled';
  $: creator = preset.creator ?? '';
  $: downloads = preset.downloads ?? 0;
  $: favorites = preset.favorites ?? 0;
  $: id = preset.preset_id ?? '';
  $: isPopular = !!preset.is_popular;
  $: isDownloaded = !!preset.is_downloaded;
  $: isWanted = $wantedPresets.has(id);

  function onThumbError(e: Event) {
    (e.currentTarget as HTMLImageElement).style.display = 'none';
  }

  function openDetail() {
    presetDetail.set({ ...preset, class_display: selectedClass ?? '' } as any);
  }

  function toggleWant(e: MouseEvent) {
    e.stopPropagation();
    wantedPresets.toggle(id);
  }

  function discard(e: MouseEvent) {
    e.stopPropagation();
    confirmDialog.show({
      title: 'Discard Preset',
      text: 'This preset will be hidden from your popular list.',
      confirmLabel: 'Discard',
      onConfirm: () => { dispatch('discard', id); },
    });
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div
  class="preset-card"
  class:downloaded={isDownloaded}
  on:click={openDetail}
>
  <div class="thumb-wrap">
    {#if firstImage}
      <img src={firstImage} alt={title} class="thumb" on:error={onThumbError} loading="lazy" />
    {:else if isSkeleton}
      <div class="skeleton-thumb"></div>
    {:else}
      <span class="no-media">NO MEDIA</span>
    {/if}
    <div class="gradient-overlay">
      <h3 class="card-title" title={title}>{title}</h3>
      {#if creator}
        <div class="card-creator">@{creator}</div>
      {/if}
    </div>

    {#if isPopular && !isDownloaded}
      <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
      <button
        class="card-action-btn want-btn"
        class:want-active={isWanted}
        title={isWanted ? 'Remove from download list' : 'Mark to download'}
        on:click={toggleWant}
      >⬇</button>
      <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
      <button
        class="card-action-btn discard-btn"
        title="Discard"
        on:click={discard}
      >✕</button>
    {/if}
  </div>

  <div class="card-footer">
    <span class="stat">📥 {Number(downloads).toLocaleString()}</span>
    <span class="stat">❤️ {Number(favorites).toLocaleString()}</span>
  </div>
</div>

<style>
  .preset-card {
    background: #0f141a;
    border: 1px solid #1a232c;
    border-radius: 8px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    cursor: pointer;
    position: relative;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .preset-card:hover {
    border-color: #ffcc4d;
    box-shadow: 0 0 24px rgba(255, 204, 77, 0.12);
  }

  .preset-card.downloaded {
    border-color: #22c55e;
  }

  .preset-card.downloaded:hover {
    border-color: #4ade80;
    box-shadow: 0 0 24px rgba(34, 197, 94, 0.12);
  }

  .thumb-wrap {
    aspect-ratio: 4 / 5;
    background: #070a0e;
    overflow: hidden;
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .thumb {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 0.5s ease;
  }

  .preset-card:hover .thumb {
    transform: scale(1.05);
  }

  .skeleton-thumb {
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, #0f141a 25%, #1a232c 50%, #0f141a 75%);
    background-size: 200% 100%;
    animation: shimmer 1.4s ease-in-out infinite;
  }

  @keyframes shimmer {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }

  .no-media {
    font-size: 10px;
    letter-spacing: 0.15em;
    color: #64748b;
    text-transform: uppercase;
  }

  .gradient-overlay {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    background: linear-gradient(to top, rgba(0,0,0,0.92) 0%, rgba(0,0,0,0.45) 50%, transparent 100%);
    padding: 32px 14px 12px;
  }

  .card-title {
    font-size: 13px;
    font-weight: 700;
    color: #fff;
    letter-spacing: 0.03em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .card-creator {
    font-size: 11px;
    color: #94a3b8;
    margin-top: 2px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .card-footer {
    padding: 10px 14px;
    background: #0c1015;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-top: 1px solid #141b23;
  }

  .stat {
    font-size: 11px;
    font-family: monospace;
    color: #94a3b8;
  }

  .card-action-btn {
    position: absolute;
    width: 28px;
    height: 28px;
    border-radius: 4px;
    border: none;
    font-size: 13px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s;
    z-index: 5;
  }

  .want-btn {
    bottom: 10px;
    right: 10px;
    background: rgba(10, 12, 18, 0.85);
    color: #64748b;
  }

  .want-btn:hover, .want-btn.want-active {
    background: rgba(255, 204, 77, 0.15);
    color: #ffcc4d;
    border: 1px solid rgba(255, 204, 77, 0.3);
  }

  .discard-btn {
    top: 8px;
    right: 8px;
    background: rgba(10, 12, 18, 0.85);
    color: #64748b;
  }

  .discard-btn:hover {
    background: rgba(220, 60, 60, 0.2);
    color: #f87171;
  }
</style>
