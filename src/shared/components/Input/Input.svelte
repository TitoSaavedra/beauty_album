<script lang="ts">
  export let value = '';
  export let placeholder = '';
</script>

<div class="input-wrap" class:has-icon={$$slots.icon}>
  {#if $$slots.icon}
    <span class="input-icon"><slot name="icon" /></span>
  {/if}
  <input
    type="text"
    {placeholder}
    bind:value
    class="field-input"
    on:input
    on:change
    on:keydown
  />
</div>

<style lang="scss">
  @use './Input.scss';
</style>
