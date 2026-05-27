<script lang="ts">
  export let checked = false;
</script>

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div class="toggle" class:toggle-on={checked} on:click>
  <span class="knob" />
</div>

<style lang="scss">
  @use './Toggle.scss';
</style>
