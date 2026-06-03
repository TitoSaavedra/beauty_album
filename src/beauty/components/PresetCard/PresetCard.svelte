<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import type { PresetEntry } from '../../tauri/album';
  import { presetDetail } from '../../stores/presetDetail';
  import { confirmDialog } from '../../../shared/stores/confirmDialog';
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

<style lang="scss">
  @use './PresetCard.scss';
</style>
