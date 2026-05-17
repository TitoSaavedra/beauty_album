<script lang="ts">
  import { toast } from '../stores/toast';
</script>

<div class="toast-container">
  {#each $toast as t (t.id)}
    <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
    <div class="toast toast-{t.type}" on:click={() => toast.dismiss(t.id)}>
      <span class="toast-icon">{t.type === 'success' ? '✓' : '✕'}</span>
      <span class="toast-text">{t.text}</span>
    </div>
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    top: 20px;
    right: 20px;
    z-index: 500;
    display: flex;
    flex-direction: column;
    gap: 10px;
    pointer-events: none;
  }

  .toast {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 18px;
    border-radius: 8px;
    font-size: 13px;
    font-weight: 600;
    letter-spacing: 0.03em;
    cursor: pointer;
    pointer-events: all;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6);
    animation: toast-in 0.25s cubic-bezier(0.34, 1.56, 0.64, 1);
    max-width: 320px;
  }

  @keyframes toast-in {
    from { opacity: 0; transform: translateX(40px) scale(0.95); }
    to   { opacity: 1; transform: translateX(0)    scale(1); }
  }

  .toast-success {
    background: #0f1f14;
    border: 1px solid #22c55e;
    color: #86efac;
  }

  .toast-error {
    background: #1f0f0f;
    border: 1px solid #ef4444;
    color: #fca5a5;
  }

  .toast-icon {
    font-size: 14px;
    flex-shrink: 0;
  }
</style>
