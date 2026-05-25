<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import type { PresetEntry } from '../../tauri/album';
  import { presetDetail } from '../../stores/presetDetail';
  import { confirmDialog } from '../../../../shared/stores/confirmDialog';
  import { wantedPresets } from '../../stores/wantedPresets';

  export let preset: PresetEntry;
  export let selectedClass: string | null = null;

  const dispatch = createEventDispatcher<{ discard: string }>();

  $: firstImage = preset.image_paths?.[0] ? convertFileSrc(preset.image_paths[0]) : null;
  $: isSkeleton = !preset.image_paths?.length;
  $: title = preset.title ?? preset.name ?? preset.preset_id ?? 'Untitled';
  $: creator = preset.creator ?? '';
  $: downloads = preset.downloads ?? 0;
  $: views = preset.views ?? 0;
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
  class:wished={isWanted}
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
      <div class="card-actions">
        <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
        <button
          class="action-want"
          class:want-active={isWanted}
          title={isWanted ? 'Remove from wishlist' : 'Add to wishlist'}
          on:click={toggleWant}
        >
          <span class="action-icon">♥</span>
        </button>
        <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
        <button class="action-discard" title="Discard" on:click={discard}>✕</button>
      </div>
    {/if}
  </div>

  <div class="card-footer">
    <span class="stat" title="Downloads">↓ {Number(downloads).toLocaleString()}</span>
    <span class="stat" title="Views">◉ {Number(views).toLocaleString()}</span>
    <span class="stat stat-fav" title="Favorites">♥ {Number(favorites).toLocaleString()}</span>
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
    transition: border-color 0.25s, box-shadow 0.25s;
  }

  .preset-card:hover {
    border-color: #ffcc4d;
    box-shadow: 0 0 20px rgba(255, 204, 77, 0.1);
  }

  .preset-card.downloaded {
    border-color: #22c55e;
  }

  .preset-card.downloaded:hover {
    border-color: #4ade80;
    box-shadow: 0 0 20px rgba(34, 197, 94, 0.1);
  }

  .preset-card.wished {
    border-color: var(--color-accent-quaternary);
    box-shadow: 0 0 16px rgba(var(--color-accent-quaternary-rgb), 0.25);
  }

  .preset-card.wished:hover {
    border-color: var(--color-accent-quaternary);
    box-shadow: 0 0 24px rgba(var(--color-accent-quaternary-rgb), 0.35);
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
    background: linear-gradient(to top, rgba(0,0,0,0.95) 0%, rgba(0,0,0,0.5) 55%, transparent 100%);
    padding: 36px 12px 10px;
    transform: translateY(6px);
    transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .preset-card:hover .gradient-overlay {
    transform: translateY(0);
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
    font-size: 10px;
    color: #94a3b8;
    margin-top: 2px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .card-actions {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    display: flex;
    justify-content: flex-end;
    gap: 1px;
    opacity: 0;
    transform: translateY(4px);
    transition: opacity 0.2s, transform 0.2s;
    z-index: 6;
    padding: 0 10px 10px;
  }

  .preset-card:hover .card-actions {
    opacity: 1;
    transform: translateY(0);
  }

  .action-want {
    flex: 0 0 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 5px;
    height: 28px;
    border-radius: 5px 0 0 5px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(8, 12, 18, 0.92);
    color: #64748b;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.06em;
    cursor: pointer;
    transition: all 0.18s;
  }

  .action-want:hover {
    background: rgba(255, 204, 77, 0.14);
    color: #ffcc4d;
    border-color: rgba(255, 204, 77, 0.35);
  }

  .action-want.want-active {
    background: transparent;
    color: var(--color-accent-quaternary);
    border-color: var(--color-accent-quaternary);
  }

  .action-icon { font-size: 12px; line-height: 1; }

  .action-discard {
    width: 28px;
    height: 28px;
    border-radius: 0 5px 5px 0;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-left: none;
    background: rgba(8, 12, 18, 0.92);
    color: #64748b;
    font-size: 11px;
    cursor: pointer;
    transition: background 0.18s, color 0.18s, border-color 0.18s;
  }

  .action-discard:hover {
    background: rgba(220, 60, 60, 0.18);
    color: #f87171;
    border-color: rgba(220, 60, 60, 0.3);
  }

  .card-footer {
    padding: 8px 12px;
    background: #0c1015;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-top: 1px solid #141b23;
    gap: 4px;
  }

  .stat {
    font-size: 10px;
    font-family: monospace;
    color: #475569;
    letter-spacing: 0.04em;
  }

  .stat-fav { color: #64405a; }
  .preset-card:hover .stat { color: #64748b; }
  .preset-card:hover .stat-fav { color: #7c4f6d; }
</style>
