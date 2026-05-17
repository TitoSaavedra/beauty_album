<script lang="ts">
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { lightbox } from '../stores/lightbox';

  function onKeydown(e: KeyboardEvent) {
    if (!$lightbox) return;
    if (e.key === 'Escape') lightbox.close();
    if (e.key === 'ArrowLeft') lightbox.prev();
    if (e.key === 'ArrowRight') lightbox.next();
  }
</script>

<svelte:window on:keydown={onKeydown} />

{#if $lightbox}
  <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-noninteractive-element-interactions -->
  <div
    class="overlay"
    role="dialog"
    aria-modal="true"
    on:click|self={lightbox.close}
  >
    <button class="btn-gold close-btn" on:click={lightbox.close}>✕ Close</button>

    {#if $lightbox.images.length > 1}
      <button class="btn-gold nav prev" on:click={lightbox.prev}>‹</button>
    {/if}

    <img
      src={convertFileSrc($lightbox.images[$lightbox.index])}
      alt=""
      class="img"
    />

    {#if $lightbox.images.length > 1}
      <button class="btn-gold nav next" on:click={lightbox.next}>›</button>
      <span class="counter">{$lightbox.index + 1} / {$lightbox.images.length}</span>
    {/if}
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.92);
    z-index: 1000;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .close-btn {
    position: absolute;
    top: 16px;
    right: 16px;
    font-size: 14px;
    padding: 6px 14px;
  }

  .nav {
    position: absolute;
    font-size: 22px;
    padding: 10px 18px;
  }

  .prev { left: 16px; }
  .next { right: 16px; }

  .img {
    max-width: 88vw;
    max-height: 88vh;
    object-fit: contain;
    border-radius: 4px;
    box-shadow: 0 0 40px rgba(0, 0, 0, 0.8);
  }

  .counter {
    position: absolute;
    bottom: 16px;
    color: var(--muted);
    font-size: 13px;
  }
</style>
