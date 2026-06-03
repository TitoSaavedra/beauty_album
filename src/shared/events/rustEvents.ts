import { listen } from '@tauri-apps/api/event';
import { handlePersonalProgress } from '../../beauty/stores/personalSync';
import { handlePopularProgress } from '../../beauty/stores/popularSync';
import type { ScrapperProgress } from '../../beauty/tauri/album';
import { ProgressType } from '../../beauty/tauri/album';
import { syncLoading } from '../stores/syncLoading';
import * as api from '../../beauty/tauri/album';

const log = (event: string, data: any) => {
  const time = new Date().toLocaleTimeString();
  console.log(`%c[${time}] ${event}`, 'color: #0ea5e9; font-weight: bold;', data);
};

export async function initRustEvents(onDbReady: (ok: boolean) => Promise<void>) {
  console.log('%c✓ Rust Events Listener Ready', 'color: #10b981; font-weight: bold; font-size: 13px;');

  // Check DB status immediately
  const dbReady = await api.isDbReady();
  await onDbReady(dbReady);

  await listen<ScrapperProgress>('scrapper_progress', ({ payload }) => {
    log('scrapper_progress', payload);
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

  await listen<string>('init_progress', ({ payload }) => {
    log('init_progress', payload);
  });
}
