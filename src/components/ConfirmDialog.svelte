<script lang="ts">
  import { confirmDialog } from '../stores/confirmDialog';

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
  <div class="swal-backdrop" on:click={cancel}>
    <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
    <div class="swal-popup" on:click|stopPropagation>

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
        <button class="swal-cancel" on:click={cancel} disabled={running}>Cancel</button>
        <button class="swal-confirm" on:click={confirm} disabled={running}>
          {running ? 'Processing…' : (opts.confirmLabel ?? 'Confirm')}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .swal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.75);
    backdrop-filter: blur(6px);
    z-index: 200;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 16px;
    animation: fade-in 0.15s ease;
  }

  @keyframes fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .swal-popup {
    background: #0f141a;
    border: 1px solid #1a232c;
    border-radius: 16px;
    padding: 40px 32px 32px;
    width: 100%;
    max-width: 400px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.7);
    animation: slide-up 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
  }

  @keyframes slide-up {
    from { transform: translateY(20px) scale(0.95); opacity: 0; }
    to { transform: translateY(0) scale(1); opacity: 1; }
  }

  .swal-icon {
    width: 64px;
    height: 64px;
    margin-bottom: 4px;
  }

  .swal-icon svg {
    width: 100%;
    height: 100%;
  }

  .swal-title {
    font-size: 20px;
    font-weight: 700;
    color: #fff;
    text-align: center;
    letter-spacing: 0.01em;
  }

  .swal-text {
    font-size: 14px;
    color: #94a3b8;
    text-align: center;
    line-height: 1.6;
    max-width: 300px;
  }

  .swal-error {
    font-size: 12px;
    color: #f87171;
    text-align: center;
    background: rgba(248, 113, 113, 0.1);
    border: 1px solid rgba(248, 113, 113, 0.2);
    padding: 8px 14px;
    border-radius: 6px;
    width: 100%;
  }

  .swal-actions {
    display: flex;
    gap: 10px;
    margin-top: 8px;
    width: 100%;
  }

  .swal-cancel {
    flex: 1;
    padding: 11px;
    background: transparent;
    border: 1px solid #1a232c;
    border-radius: 8px;
    color: #94a3b8;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: border-color 0.2s, color 0.2s;
  }

  .swal-cancel:hover:not(:disabled) {
    border-color: #94a3b8;
    color: #fff;
  }

  .swal-confirm {
    flex: 1;
    padding: 11px;
    background: linear-gradient(135deg, #ffcc4d 0%, #b3861b 100%);
    border: none;
    border-radius: 8px;
    color: #000;
    font-size: 13px;
    font-weight: 700;
    letter-spacing: 0.05em;
    cursor: pointer;
    transition: filter 0.2s;
  }

  .swal-confirm:hover:not(:disabled) { filter: brightness(1.1); }
  .swal-confirm:disabled, .swal-cancel:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
