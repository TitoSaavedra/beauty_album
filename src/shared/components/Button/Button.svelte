<script lang="ts">
  export let variant: 'primary' | 'ghost' | 'icon' = 'primary';
  export let disabled = false;
  export let active = false;
  export let type: 'button' | 'submit' | 'reset' = 'button';
  export let title = '';
  let className = '';
  export { className as class };
</script>

<button
  {type}
  {disabled}
  {title}
  class="btn btn-{variant} {className}"
  class:active
  on:click
>
  <slot />
</button>

<style lang="scss">
  @use './Button.scss';
</style>
