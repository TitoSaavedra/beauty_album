<script lang="ts">
  import { fade, scale } from 'svelte/transition';
  import { _ } from 'svelte-i18n';
  import { confirmDialog } from '../../stores/confirmDialog';

  $: opts = $confirmDialog;

  let running = false;
  let error = '';

  async function confirm() {
    if (!opts) return;
    running = true;
    error = '';
    try {
      await opts.onConfirm();
      confirmDialog.close();
    } catch (e) {
      error = String(e);
    } finally {
      running = false;
    }
  }

  function cancel() {
    confirmDialog.close();
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') cancel();
  }
</script>

<svelte:window on:keydown={onKeydown} />

{#if opts}
  <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
  <div class="swal-backdrop" on:click={cancel} transition:fade={{ duration: 200 }}>
    <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
    <div class="swal-popup" on:click|stopPropagation transition:scale={{ duration: 250, start: 0.95 }}>

      <div class="swal-icon">
        <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M12 2L2 20h20L12 2z" stroke="#ffcc4d" stroke-width="1.5" stroke-linejoin="round" fill="rgba(255,204,77,0.08)"/>
          <line x1="12" y1="9" x2="12" y2="14" stroke="#ffcc4d" stroke-width="1.8" stroke-linecap="round"/>
          <circle cx="12" cy="17" r="0.8" fill="#ffcc4d"/>
        </svg>
      </div>

      <h2 class="swal-title">{opts.title}</h2>
      <p class="swal-text">{opts.text}</p>

      {#if error}
        <p class="swal-error">{error}</p>
      {/if}

      <div class="swal-actions">
        <button class="swal-cancel" on:click={cancel} disabled={running}>{$_('common.cancel')}</button>
        <button class="swal-confirm" on:click={confirm} disabled={running}>
          {running ? $_('confirm.processing') : (opts.confirmLabel ?? $_('common.confirm'))}
        </button>
      </div>
    </div>
  </div>
{/if}
