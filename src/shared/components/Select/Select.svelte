<script lang="ts">
  export let value: string | number = '';
  export let options: { value: string | number; label: string }[] = [];
</script>

<select class="field-select" bind:value on:change>
  {#each options as opt}
    <option value={opt.value}>{opt.label}</option>
  {/each}
</select>

<style lang="scss">
  @use './Select.scss';
</style>
