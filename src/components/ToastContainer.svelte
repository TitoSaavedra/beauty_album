<script lang="ts">
  import { toast } from '../stores/toast';
</script>

<div class="toast-container">
  {#each $toast as t (t.id)}
    <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
    <div class="toast toast-{t.type}" on:click={() => toast.dismiss(t.id)}>
      <span class="toast-bar"></span>
      <span class="toast-text">{t.text}</span>
    </div>
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    bottom: 28px;
    right: 28px;
    z-index: 500;
    display: flex;
    flex-direction: column;
    gap: 8px;
    pointer-events: none;
    align-items: flex-end;
  }

  .toast {
    display: flex;
    align-items: stretch;
    gap: 0;
    border-radius: 4px;
    font-size: 11px;
    font-family: monospace;
    font-weight: 500;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    cursor: pointer;
    pointer-events: all;
    background: #070a0e;
    border: 1px solid #1a232c;
    box-shadow: 0 4px 24px rgba(0, 0, 0, 0.7);
    animation: toast-in 0.2s ease-out;
    max-width: 380px;
    overflow: hidden;
  }

  @keyframes toast-in {
    from { opacity: 0; transform: translateY(12px); }
    to   { opacity: 1; transform: translateY(0); }
  }

  .toast-bar {
    width: 3px;
    flex-shrink: 0;
  }

  .toast-text {
    padding: 12px 16px;
    line-height: 1.4;
  }

  .toast-success .toast-bar  { background: #b3861b; }
  .toast-success .toast-text { color: #c9a020; }
  .toast-success { border-color: rgba(179, 134, 27, 0.3); }

  .toast-error .toast-bar  { background: #7f1d1d; }
  .toast-error .toast-text { color: #f87171; }
  .toast-error { border-color: rgba(239, 68, 68, 0.25); }
</style>
