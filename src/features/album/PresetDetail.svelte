<script lang="ts">
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { presetDetail } from '../../stores/presetDetail';
  import { confirmDialog } from '../../stores/confirmDialog';
  import { toast } from '../../stores/toast';
  import { injectPreset } from '../../tauri/album';

  $: preset = $presetDetail;
  $: images = preset?.image_paths?.map(p => convertFileSrc(p)) ?? [];
  $: activeImage = images[0] ?? '';

  $: if (preset) activeImage = images[0] ?? '';

  $: title    = preset?.title    ?? preset?.name     ?? preset?.preset_id ?? 'Untitled';
  $: creator  = preset?.creator  ?? 'Anonymous';
  $: date     = preset?.date     ?? '—';
  $: id       = preset?.preset_id ?? '—';
  $: downloads = preset?.downloads ?? 0;
  $: views     = preset?.views     ?? 0;
  $: favorites = preset?.favorites ?? 0;
  $: className = (preset as any)?.class ?? '';

  function close() { presetDetail.set(null); }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
  }

  function inject() {
    if (!preset?.download_path) return;
    confirmDialog.show({
      title: 'Inject Preset',
      text: 'All files in the BDO output directory will be deleted and replaced with this preset. This cannot be undone.',
      confirmLabel: 'Inject',
      onConfirm: async () => {
        await injectPreset(preset!.download_path!);
        presetDetail.set(null);
        toast.show('Preset injected successfully');
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
  <div class="backdrop" on:click={close}>
    <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
    <div class="modal" on:click|stopPropagation>

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
        <div class="info-top">
          <div class="title-row">
            <div>
              {#if className}
                <span class="class-badge">{className}</span>
              {/if}
              <h2 class="preset-title">{title}</h2>
            </div>
            <button class="close-btn" on:click={close}>&times;</button>
          </div>

          <div class="meta-list">
            <div class="meta-row">
              <span class="meta-label">Creator</span>
              <span class="meta-value">{creator}</span>
            </div>
            <div class="meta-row">
              <span class="meta-label">Registry ID</span>
              <span class="meta-value mono">{id}</span>
            </div>
            <div class="meta-row">
              <span class="meta-label">Upload Date</span>
              <span class="meta-value">{date}</span>
            </div>
          </div>

          <div class="stats-grid">
            <div class="stat-box">
              <div class="stat-label">Downloads</div>
              <div class="stat-value">{Number(downloads).toLocaleString()}</div>
            </div>
            <div class="stat-box">
              <div class="stat-label">Views</div>
              <div class="stat-value">{Number(views).toLocaleString()}</div>
            </div>
            <div class="stat-box">
              <div class="stat-label">Favorites</div>
              <div class="stat-value gold">{Number(favorites).toLocaleString()}</div>
            </div>
          </div>
        </div>

        <div class="download-area">
          {#if preset.download_path}
            <button class="btn-download bdo-gold-gradient" on:click={inject}>
              Inject Local Parameters
            </button>
          {:else}
            <div class="no-download">No binary structure linked</div>
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
    background: rgba(0, 0, 0, 0.85);
    z-index: 50;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 16px;
    backdrop-filter: blur(4px);
    transition: opacity 0.3s;
  }

  .modal {
    background: #0a0d12;
    border: 1px solid #222d3a;
    border-radius: 8px;
    max-width: 900px;
    width: 100%;
    max-height: 85vh;
    overflow: hidden;
    display: flex;
    flex-direction: row;
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.8);
  }

  /* LEFT */
  .panel-left {
    flex: 0 0 58%;
    background: #05070a;
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    border-right: 1px solid #1a232c;
  }

  .main-preview {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #090c10;
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
    color: #64748b;
  }

  .thumbs {
    display: flex;
    gap: 8px;
    overflow-x: auto;
    padding-bottom: 4px;
    flex-shrink: 0;
  }

  .thumb {
    width: 64px;
    height: 64px;
    flex-shrink: 0;
    background: #090c10;
    border-radius: 4px;
    border: 1px solid #1a232c;
    cursor: pointer;
    overflow: hidden;
    transition: border-color 0.2s;
  }

  .thumb:hover { border-color: rgba(255, 204, 77, 0.5); }
  .thumb-active { border-color: #ffcc4d !important; }

  .thumb-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  /* RIGHT */
  .panel-right {
    flex: 0 0 42%;
    padding: 32px;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    background: #0d1218;
    overflow-y: auto;
  }

  .info-top {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .title-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 12px;
  }

  .class-badge {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.15em;
    text-transform: uppercase;
    color: #ffcc4d;
    background: rgba(255, 204, 77, 0.1);
    padding: 4px 10px;
    border-radius: 3px;
    border: 1px solid rgba(255, 204, 77, 0.2);
    display: inline-block;
    margin-bottom: 12px;
  }

  .preset-title {
    font-size: 20px;
    font-weight: 700;
    color: #fff;
    letter-spacing: 0.03em;
    line-height: 1.3;
  }

  .close-btn {
    background: none;
    border: none;
    color: #94a3b8;
    font-size: 26px;
    font-weight: 300;
    cursor: pointer;
    line-height: 1;
    flex-shrink: 0;
    transition: color 0.2s;
    padding: 0;
  }

  .close-btn:hover { color: #fff; }

  .meta-list {
    border-top: 1px solid #1a232c;
    padding-top: 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .meta-row {
    display: flex;
    justify-content: space-between;
    font-size: 13px;
  }

  .meta-label { color: #94a3b8; }
  .meta-value { color: #e2e8f0; font-weight: 500; }
  .meta-value.mono { font-family: monospace; }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
    background: #070a0e;
    padding: 16px;
    border-radius: 8px;
    border: 1px solid #1a232c;
    text-align: center;
  }

  .stat-label {
    font-size: 11px;
    color: #94a3b8;
    margin-bottom: 4px;
    letter-spacing: 0.05em;
  }

  .stat-value {
    font-size: 13px;
    font-weight: 700;
    color: #fff;
    font-family: monospace;
  }

  .stat-value.gold { color: #ffcc4d; }

  .download-area {
    margin-top: 32px;
    border-top: 1px solid #1a232c;
    padding-top: 24px;
  }

  .btn-download {
    width: 100%;
    padding: 14px;
    border: none;
    border-radius: 6px;
    color: #000;
    font-weight: 700;
    font-size: 12px;
    letter-spacing: 0.15em;
    text-transform: uppercase;
    cursor: pointer;
    transition: filter 0.2s;
    box-shadow: 0 4px 20px rgba(255, 204, 77, 0.08);
  }

  .btn-download:hover { filter: brightness(1.1); }

  .no-download {
    text-align: center;
    font-size: 12px;
    color: #64748b;
    font-style: italic;
    letter-spacing: 0.05em;
  }
</style>
