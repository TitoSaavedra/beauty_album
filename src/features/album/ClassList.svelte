<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import type { ClassEntry } from '../../tauri/album';

  export let classes: ClassEntry[] = [];
  export let selectedClass: string | null = null;
  export let loading = false;
  export let error = '';

  const dispatch = createEventDispatcher<{ select: string }>();

  let search = '';
  $: filtered = search.trim()
    ? classes.filter(c => c.name.toLowerCase().includes(search.toLowerCase()))
    : classes;

  function onIconError(e: Event) {
    (e.currentTarget as HTMLElement).style.display = 'none';
  }
</script>

<div class="search-box">
  <input
    class="search-input"
    type="text"
    bind:value={search}
    placeholder="SEARCH CLASS..."
  />
</div>

<div class="list custom-scroll">
  {#if loading}
    <p class="status">Loading...</p>
  {:else if error}
    <p class="status error">{error}</p>
  {:else if classes.length === 0}
    <p class="status">Open ⚙ to set album directory</p>
  {:else if filtered.length === 0}
    <p class="status">No results</p>
  {:else}
    {#each filtered as cls (cls.name)}
      <button
        class="class-btn"
        class:active={cls.name === selectedClass}
        on:click={() => dispatch('select', cls.name)}
      >
        <span class="cls-name">{cls.name}</span>
        <div class="cls-right">
          {#if cls.preset_count > 0}
            <span class="count">{cls.preset_count}</span>
          {/if}
          {#if cls.icon_path}
            <img
              src={convertFileSrc(cls.icon_path)}
              alt=""
              class="cls-icon"
              on:error={onIconError}
            />
          {/if}
        </div>
      </button>
    {/each}
  {/if}
</div>

<style>
  .search-box {
    padding: 16px;
    border-bottom: 1px solid rgba(26, 35, 44, 0.5);
    flex-shrink: 0;
  }

  .search-input {
    width: 100%;
    background: #0f141a;
    border: 1px solid #1a232c;
    border-radius: 4px;
    padding: 8px 12px;
    font-size: 11px;
    font-family: monospace;
    letter-spacing: 0.1em;
    color: #fff;
    outline: none;
    transition: border-color 0.2s;
  }

  .search-input::placeholder { color: #64748b; }

  .search-input:focus { border-color: #ffcc4d; }

  .list {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .status {
    font-size: 11px;
    color: #64748b;
    text-align: center;
    padding: 24px 8px;
    letter-spacing: 0.05em;
  }

  .error { color: #f87171; }

  .class-btn {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 10px 16px;
    text-align: left;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: #94a3b8;
    background: transparent;
    border: none;
    border-left: 3px solid transparent;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    gap: 8px;
  }

  .class-btn:hover {
    color: #fff;
    background: #0f141a;
  }

  .class-btn.active {
    border-left-color: #ffcc4d;
    background: linear-gradient(90deg, rgba(255, 204, 77, 0.08) 0%, transparent 100%);
    color: #ffcc4d;
  }

  .cls-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .cls-right {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .count {
    font-size: 10px;
    font-family: monospace;
    color: #64748b;
  }

  .class-btn.active .count { color: rgba(255, 204, 77, 0.6); }

  .cls-icon {
    width: 20px;
    height: 20px;
    object-fit: contain;
    background: #0a0d12;
    padding: 2px;
    border-radius: 3px;
    border: 1px solid #1a232c;
    flex-shrink: 0;
    transition: border-color 0.2s;
  }

  .class-btn:hover .cls-icon,
  .class-btn.active .cls-icon {
    border-color: rgba(255, 204, 77, 0.4);
  }
</style>
