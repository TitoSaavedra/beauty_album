<script lang="ts">
  import { fade, scale } from 'svelte/transition';
  import { _, locale } from 'svelte-i18n';
  import { createEventDispatcher } from 'svelte';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import { saveConfig } from '../../tauri/config';
  import type { AppConfig } from '../../tauri/config';
  import { theme } from '../../../stores/theme';
  import Button from '../Button/Button.svelte';
  import Input from '../Input/Input.svelte';
  import PillSelector from '../PillSelector/PillSelector.svelte';

  export let config: AppConfig;

  const dispatch = createEventDispatcher<{ save: AppConfig; close: void }>();

  let bdo_docs_dir = config.bdo_docs_dir;
  let saving = false;
  let error = '';
  let selectedLocale = $locale;
  let selectedTheme = $theme;


  async function pickFolder() {
    const selected = await openDialog({ directory: true, multiple: false });
    if (typeof selected === 'string') bdo_docs_dir = selected;
  }

  function changeLocale(newLocale: string) {
    selectedLocale = newLocale;
    locale.set(newLocale);
    localStorage.setItem('preferred-locale', newLocale);
  }

  function changeTheme(newTheme: string) {
    selectedTheme = newTheme;
    theme.set(newTheme);
  }

  async function save() {
    saving = true;
    error = '';
    try {
      const updated: AppConfig = {
        bdo_docs_dir: bdo_docs_dir.trim(),
        cf_clearance: config.cf_clearance ?? '',
      };
      await saveConfig(updated);
      dispatch('save', updated);
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  function close() {
    dispatch('close');
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
  }
</script>

<style lang="scss">
  @use './SettingsModal.scss' as *;
</style>

<svelte:window on:keydown={onKeydown} />

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div class="modal-backdrop" on:click={close} transition:fade={{ duration: 200 }}>
  <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
  <div class="modal-container" on:click|stopPropagation transition:scale={{ duration: 250, start: 0.95 }}>
    <div class="modal-header">
      <h2 class="modal-title">{$_('settings.title')}</h2>
      <button class="modal-close" on:click={close} title={$_('common.close')}>✕</button>
    </div>

    <div class="modal-content">
      <div class="settings-row">
        <span class="row-label">{$_('settings.language')}</span>
        <PillSelector
          bind:value={selectedLocale}
          on:change={e => changeLocale(String(e.detail))}
          options={[
            { value: 'en',    label: $_('settings.english'),    icon: '🇬🇧' },
            { value: 'es',    label: $_('settings.spanish'),    icon: '🇪🇸' },
            { value: 'pt-BR', label: $_('settings.portuguese'), icon: '🇧🇷' },
          ]}
        />
      </div>

      <div class="settings-row">
        <span class="row-label">{$_('settings.theme')}</span>
        <PillSelector
          bind:value={selectedTheme}
          on:change={e => changeTheme(e.detail)}
          options={[
            { value: 'dark',  label: $_('settings.dark'),  icon: '🌙' },
            { value: 'light', label: $_('settings.light'), icon: '☀️' },
          ]}
        />
      </div>

      <div class="settings-row settings-row--dir">
        <span class="row-label">{$_('settings.docs_directory')}</span>
        <div class="input-row">
          <Input bind:value={bdo_docs_dir} placeholder="C:\Users\You\Documents\Black Desert">
            <svelte:fragment slot="icon">
              <svg viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
                <path d="M2 6a2 2 0 012-2h4l2 2h6a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z"/>
              </svg>
            </svelte:fragment>
          </Input>
          <button class="btn-browse" on:click={pickFolder} title="Browse">
            <svg viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
              <path d="M2 6a2 2 0 012-2h4l2 2h6a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z"/>
              <path fill-rule="evenodd" d="M10 9a1 1 0 011 1v3.586l1.293-1.293a1 1 0 111.414 1.414l-3 3a1 1 0 01-1.414 0l-3-3a1 1 0 111.414-1.414L9 13.586V10a1 1 0 011-1z" clip-rule="evenodd"/>
            </svg>
          </button>
        </div>
      </div>

      {#if error}
        <div class="error-msg">{error}</div>
      {/if}
    </div>

    <div class="modal-actions">
      <Button variant="ghost" on:click={close}>{$_('common.cancel')}</Button>
      <Button variant="primary" on:click={save} disabled={saving}>
        {saving ? $_('settings.saving') : $_('settings.save')}
      </Button>
    </div>
  </div>
</div>
