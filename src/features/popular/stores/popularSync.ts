import { writable, get } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import * as api from '../../beauty/tauri/album';
import { toast } from '../../../shared/stores/toast';
import { scheduleGridRefresh, viewMode, selectedClassObj } from '../../beauty/stores/presetGrid';

interface ScrapperProgress {
  preset_id: string;
  status: string;
  message: string;
  class_name: string;
  class_id: number;
  current: number;
  total: number;
}

export const popularRunning = writable(false);
export const popularCurrent = writable(0);
export const popularTotal = writable(0);
export const popularMsg = writable('');
export const popularPhase = writable<'syncing' | 'images' | ''>('');

export async function startPopularSync() {
  if (get(popularRunning)) return;
  popularRunning.set(true);
  popularCurrent.set(0);
  popularTotal.set(0);
  popularMsg.set('Starting...');
  popularPhase.set('syncing');

  const unlisten = await listen<ScrapperProgress>('popular_progress', ({ payload }) => {
    if (payload.total > 0 && get(popularTotal) === 0) popularTotal.set(payload.total);
    if (payload.current > 0) popularCurrent.set(payload.current);
    if (payload.message) popularMsg.set(payload.message);
    if (payload.status === 'metadata' || payload.status === 'done') {
      popularPhase.set('syncing');
      const vm = get(viewMode);
      const scObj = get(selectedClassObj);
      if (vm === 'popular' && payload.class_id && payload.class_id === scObj?.class_id)
        scheduleGridRefresh(payload.class_id);
    } else if (payload.status === 'additional_data') {
      popularPhase.set('images');
    }
  });

  try {
    const doneMsg = await api.syncPopular();
    if (doneMsg) toast.show(doneMsg, 'success', 6000);
  } catch (e) {
    toast.show(String(e), 'error', 6000);
  } finally {
    unlisten();
    popularRunning.set(false);
    popularMsg.set('');
    popularTotal.set(0);
    popularCurrent.set(0);
    popularPhase.set('');
  }
}
