import { writable, get } from 'svelte/store';
import * as api from '../../beauty/tauri/album';
import { toast } from '../../../shared/stores/toast';
import { presetDetail } from '../../beauty/stores/presetDetail';
import { scheduleGridRefresh, viewMode, selectedClassObj } from '../../beauty/stores/presetGrid';

export interface PopularProgress {
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
export const popularTotal   = writable(0);
export const popularMsg     = writable('');
export const popularPhase   = writable<'syncing' | 'images' | ''>('');

export function handlePopularProgress(payload: PopularProgress) {
  if (payload.total > 0 && get(popularTotal) === 0) popularTotal.set(payload.total);
  if (payload.current > 0) popularCurrent.set(payload.current);
  if (payload.message) popularMsg.set(payload.message);

  if (payload.status === 'metadata') {
    popularPhase.set('syncing');
    const vm = get(viewMode);
    const scObj = get(selectedClassObj);
    if (vm === 'popular' && payload.class_id && payload.class_id === scObj?.class_id)
      scheduleGridRefresh(payload.class_id);
  } else if (payload.status === 'done') {
    popularPhase.set('syncing');
    const vm = get(viewMode);
    const scObj = get(selectedClassObj);
    if (vm === 'popular') {
      if (payload.class_id && payload.class_id === scObj?.class_id)
        scheduleGridRefresh(payload.class_id);
      const pid = payload.preset_id;
      const cls = payload.class_name;
      toast.show(`${cls} #${pid} synced`, 'success', 5000, async () => {
        const preset = await api.getPopularPresetById(pid);
        if (preset) presetDetail.set(preset);
      });
    }
  } else if (payload.status === 'additional_data') {
    popularPhase.set('images');
  }
}

export async function startPopularSync() {
  if (get(popularRunning)) return;
  popularRunning.set(true);
  popularCurrent.set(0);
  popularTotal.set(0);
  popularMsg.set('Starting...');
  popularPhase.set('syncing');

  try {
    const doneMsg = await api.syncPopular();
    if (doneMsg) toast.show(doneMsg, 'success', 6000);
  } catch (e) {
    toast.show(String(e), 'error', 6000);
  } finally {
    popularRunning.set(false);
    popularMsg.set('');
    popularTotal.set(0);
    popularCurrent.set(0);
    popularPhase.set('');
  }
}
