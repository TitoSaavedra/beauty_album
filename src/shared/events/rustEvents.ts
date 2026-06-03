import { listen } from '@tauri-apps/api/event';
import { get } from 'svelte/store';
import { _ } from 'svelte-i18n';
import { appLoading, initStatus } from '../stores/appState';
import { handlePersonalProgress, personalRunning, personalCurrent, personalTotal, personalMsg, personalPhase } from '../../beauty/stores/personalSync';
import { handlePopularProgress, popularRunning, popularCurrent, popularTotal, popularMsg, popularPhase } from '../../beauty/stores/popularSync';
import type { ScrapperProgress } from '../../beauty/tauri/album';
import { ProgressType } from '../../beauty/tauri/album';
import { toast } from '../stores/toast';
import { syncLoading } from '../stores/syncLoading';

const log = (event: string, data: any) => {
  const time = new Date().toLocaleTimeString();
  console.log(`%c[${time}] ${event}`, 'color: #0ea5e9; font-weight: bold;', data);
};

export async function initRustEvents(onDbReady: (ok: boolean) => Promise<void>) {
  console.log('%c✓ Rust Events Listener Ready', 'color: #10b981; font-weight: bold; font-size: 13px;');

  await listen<string>('init_progress', ({ payload }) => {
    log('init_progress', payload);
    initStatus.set(payload);
  });

  await listen<boolean>('db_ready', ({ payload }) => {
    log('db_ready', payload);
    appLoading.set(false);
    onDbReady(payload);
  });

  await listen<string[]>('folder_changed', async ({ payload: files }) => {
    log('folder_changed', files);
    personalRunning.set(false);
    personalCurrent.set(0);
    personalTotal.set(0);
    personalMsg.set('');
    personalPhase.set('');

    popularRunning.set(false);
    popularCurrent.set(0);
    popularTotal.set(0);
    popularMsg.set('');
    popularPhase.set('');

    const count = files.length;
    const t = get(_);
    const label = count === 1 ? files[0] : `${count} ${t('common.new_presets_found')}`;
    toast.show(`${t('common.new_preset_found')}: ${label}`, 'warning', 5000);
  });

  await listen<ScrapperProgress>('scrapper_progress', ({ payload }) => {
    log('scrapper_progress', payload);

    if (payload.message === 'Initializing...') {
      syncLoading.set(payload.message);
      return;
    }

    syncLoading.set('');
    switch (payload.progress_type) {
      case ProgressType.Preset:
        handlePersonalProgress(payload);
        break;
      case ProgressType.Popular:
        handlePopularProgress(payload);
        break;
    }
  });

  await listen<string>('sync_loading', ({ payload }) => {
    log('sync_loading', payload);
    syncLoading.set(payload);
  });

}
