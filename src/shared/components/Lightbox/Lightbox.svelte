<script lang="ts">
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { lightbox } from '../../stores/lightbox';

  function onKeydown(e: KeyboardEvent) {
    if (!$lightbox) return;
    if (e.key === 'Escape') lightbox.close();
    if (e.key === 'ArrowLeft') lightbox.prev();
    if (e.key === 'ArrowRight') lightbox.next();
  }
</script>

<svelte:window on:keydown={onKeydown} />

{#if $lightbox}
  <!-- svelte-ignore a11y-click-events-have-key-events a11y-noninteractive-element-interactions -->
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
