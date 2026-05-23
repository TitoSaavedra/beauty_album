<script lang="ts">
  import { toast } from '../stores/toast';

  const ICONS = { success: '✓', warning: '⚠', error: '✕' };
</script>

<div class="toast-container">
  {#each $toast as t (t.id)}
    <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
    <div class="toast toast-{t.type}" class:clickable={!!t.onClick} on:click={() => { t.onClick?.(); toast.dismiss(t.id); }}>
      <span class="toast-icon">{ICONS[t.type]}</span>
      <span class="toast-text">{t.text}</span>
    </div>
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    top: 72px;
    right: 20px;
    z-index: 500;
    display: flex;
    flex-direction: column;
    gap: 8px;
    pointer-events: none;
    align-items: flex-end;
    width: 340px;
  }

  .toast {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 12px 14px;
    border-radius: 8px;
    font-size: 11px;
    font-family: monospace;
    letter-spacing: 0.05em;
    pointer-events: all;
    backdrop-filter: blur(16px);
    border: 1px solid rgba(179, 134, 27, 0.30);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.55), 0 1px 0 rgba(255, 255, 255, 0.04) inset;
    animation: toast-in 0.26s cubic-bezier(0.16, 1, 0.3, 1);
    width: 100%;
  }

  @keyframes toast-in {
    from { opacity: 0; transform: translateX(calc(100% + 20px)); }
    to   { opacity: 1; transform: translateX(0); }
  }

  .toast-icon {
    flex-shrink: 0;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 10px;
    font-weight: 700;
    line-height: 1;
    margin-top: 1px;
  }

  .toast-text {
    flex: 1;
    line-height: 1.55;
    word-break: break-word;
    cursor: default;
  }

  .clickable { cursor: pointer; }

  /* ── success — pastel sage/mint ── */
  .toast-success {
    background: rgba(10, 30, 22, 0.93);
  }
  .toast-success .toast-icon {
    background: rgba(110, 231, 183, 0.12);
    color: #6ee7b7;
  }
  .toast-success .toast-text { color: #a7e8d0; }
  .toast-success.clickable:hover {
    border-color: rgba(110, 231, 183, 0.35);
    background: rgba(12, 36, 26, 0.97);
  }
  .toast-success.clickable:hover .toast-text { color: #c3f0e0; }

  /* ── warning — pastel amber ── */
  .toast-warning {
    background: rgba(24, 18, 6, 0.93);
  }
  .toast-warning .toast-icon {
    background: rgba(251, 191, 36, 0.12);
    color: #fbbf24;
  }
  .toast-warning .toast-text { color: #f5d080; }
  .toast-warning.clickable:hover {
    border-color: rgba(251, 191, 36, 0.38);
    background: rgba(30, 22, 6, 0.97);
  }
  .toast-warning.clickable:hover .toast-text { color: #fde68a; }

  /* ── error — pastel rose ── */
  .toast-error {
    background: rgba(28, 8, 8, 0.93);
  }
  .toast-error .toast-icon {
    background: rgba(252, 165, 165, 0.12);
    color: #fca5a5;
  }
  .toast-error .toast-text { color: #fca5a5; }
  .toast-error.clickable:hover {
    border-color: rgba(252, 165, 165, 0.35);
    background: rgba(34, 10, 10, 0.97);
  }
  .toast-error.clickable:hover .toast-text { color: #fecaca; }
</style>
