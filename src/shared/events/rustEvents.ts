import { listen } from '@tauri-apps/api/event';
import { get } from 'svelte/store';
import { _ } from 'svelte-i18n';
import { appLoading, initStatus } from '../stores/appState';
import { classes } from '../../features/beauty/stores/classes';
import { scrapperRunning, handleScrapperProgress, startScraper, type ScrapperProgress } from '../../features/beauty/stores/scraper';
import { handlePopularProgress, startPopularSync, type PopularProgress } from '../../features/popular/stores/popularSync';
import { toast } from '../stores/toast';

interface ClassCountPayload {
  class_id: number;
  count: number;
  is_popular: boolean;
}

export async function initRustEvents(onDbReady: (ok: boolean) => Promise<void>) {
  await listen<boolean>('db_ready', ({ payload }) => {
    appLoading.set(false);
    onDbReady(payload);
  });

  await listen<string>('init_progress', ({ payload }) => {
    initStatus.set(payload);
  });

  await listen<string[]>('folder_changed', async ({ payload: files }) => {
    const count = files.length;
    const t = get(_);
    const label = count === 1 ? files[0] : `${count} ${t('common.new_presets_found')}`;
    toast.show(`${t('common.new_preset_found')}: ${label}`, 'success', 5000);
    if (!get(scrapperRunning)) {
      await startScraper();
      startPopularSync();
    }
  });

  await listen<ScrapperProgress>('scrapper_progress', ({ payload }) => {
    handleScrapperProgress(payload);
  });

  await listen<PopularProgress>('popular_progress', ({ payload }) => {
    handlePopularProgress(payload);
  });

  await listen<ClassCountPayload>('class_count_updated', ({ payload }) => {
    classes.update(list =>
      list.map(c =>
        c.class_id === payload.class_id
          ? payload.is_popular
            ? { ...c, popular_count: payload.count }
            : { ...c, preset_count: payload.count }
          : c
      )
    );
  });

  await listen('log_entry', () => { /* futuro: store de logs en tiempo real */ });
}
