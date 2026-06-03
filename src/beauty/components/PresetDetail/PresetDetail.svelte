<script lang="ts">
  import { fade, scale } from 'svelte/transition';
  import { _ } from 'svelte-i18n';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { presetDetail } from '../../stores/presetDetail';
  import { confirmDialog } from '../../../shared/stores/confirmDialog';
  import { toast } from '../../../shared/stores/toast';
  import { wantedPresets } from '../../stores/wantedPresets';
  import { injectPreset, discardPreset, openUrl } from '../../tauri/album';
  import Button from '../../../shared/components/Button/Button.svelte';

  $: preset = $presetDetail;
  $: images = preset?.image_paths?.map(p => convertFileSrc(p)) ?? [];
  $: activeImage = images[0] ?? '';

  $: if (preset) activeImage = images[0] ?? '';

  $: title    = preset?.title    ?? preset?.name     ?? preset?.preset_id ?? 'Untitled';
  $: creator  = preset?.creator  ?? 'Anonymous';
  $: date = preset?.date
      ?? ((preset as any)?.creation_at
          ? new Date(((preset as any).creation_at as number) * 1000).toLocaleDateString('en-CA')
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
            <Button variant="ghost" class="close-btn" on:click={close}>✕</Button>
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
              <Button
                variant="ghost"
                class="btn-want"
                active={isWanted}
                on:click={() => wantedPresets.toggle(id)}
                title={isWanted ? $_('preset.wished') : $_('preset.wish')}
              >♥</Button>
              <Button variant="icon" class="btn-discard-sm" title={$_('preset.discard')} on:click={discard}>✕</Button>
            </div>
          {/if}
          {#if isPopular}
            <Button variant="ghost" class="btn-garmoth" on:click={openOnGarmoth}>
              <span class="btn-icon">◈</span> {$_('preset.open_garmoth')}
            </Button>
          {:else if preset.download_path}
            <Button variant="primary" class="btn-inject" on:click={inject}>
              <span class="btn-icon">⇪</span> {$_('preset.export_to_bdo')}
            </Button>
          {:else}
            <div class="no-download">{$_('preset.no_binary_linked')}</div>
          {/if}
        </div>

      </div>
    </div>
  </div>
{/if}

<style lang="scss">
  @use './PresetDetail.scss';
</style>
