<script lang="ts">
  import { convertFileSrc } from '@tauri-apps/api/core';
  import type { PresetEntry } from '../../tauri/album';
  import { presetDetail } from '../../stores/presetDetail';

  export let preset: PresetEntry;

  $: firstImage = preset.image_paths?.[0] ? convertFileSrc(preset.image_paths[0]) : null;
  $: title = preset.title ?? preset.name ?? preset.preset_id ?? 'Untitled';
  $: creator = preset.creator ?? '';
  $: downloads = preset.downloads ?? 0;
  $: favorites = preset.favorites ?? 0;

  function onThumbError(e: Event) {
    (e.currentTarget as HTMLImageElement).style.display = 'none';
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div class="preset-card" on:click={() => presetDetail.set(preset)}>
  <div class="thumb-wrap">
    {#if firstImage}
      <img src={firstImage} alt={title} class="thumb" on:error={onThumbError} loading="lazy" />
    {:else}
      <span class="no-media">NO MEDIA</span>
    {/if}
    <div class="gradient-overlay">
      <h3 class="card-title" title={title}>{title}</h3>
      {#if creator}
        <div class="card-creator">@{creator}</div>
      {/if}
    </div>
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
</style>
