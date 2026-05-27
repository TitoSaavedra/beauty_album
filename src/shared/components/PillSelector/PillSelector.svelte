<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let value: string | number = '';
  export let options: { value: string | number; label: string; icon?: string }[] = [];

  const dispatch = createEventDispatcher<{ change: string | number }>();

  function select(v: string | number) {
    value = v;
    dispatch('change', v);
  }
</script>

<div class="pill-group">
  {#each options as opt}
    <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
    <div
      class="pill"
      class:pill-active={value === opt.value}
      on:click={() => select(opt.value)}
    >
      {#if opt.icon}<span>{opt.icon}</span>{/if}
      {opt.label}
    </div>
  {/each}
</div>

<style lang="scss">
  @use './PillSelector.scss';
</style>
