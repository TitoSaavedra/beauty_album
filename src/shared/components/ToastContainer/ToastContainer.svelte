<script lang="ts">
  import { toast } from '../../stores/toast';

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

<style lang="scss">
  @use './ToastContainer.scss';
</style>
