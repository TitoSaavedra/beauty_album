import { writable, get } from 'svelte/store';
import * as api from '../tauri/album';
import type { ScrapperProgress } from '../tauri/album';
import { ProgressStatus } from '../tauri/album';
import { toast } from '../../shared/stores/toast';
import { presetDetail } from './presetDetail';
import { scheduleGridRefresh, viewMode, selectedClassObj } from './presetGrid';
import { refreshCounts } from './classes';

export const popularRunning = writable(false);
export const popularCurrent = writable(0);
export const popularTotal   = writable(0);
export const popularMsg     = writable('');
export const popularPhase   = writable<'syncing' | 'images' | ''>('');

let popDoneCount = 0;
let popTotalExpected = 0;

export function handlePopularProgress(payload: ScrapperProgress) {
  if (payload.class_name === 'SYNC' && payload.status === ProgressStatus.Done) {
    popularRunning.set(false);
    popularMsg.set('');
    popularPhase.set('');
    popularCurrent.set(0);
    popularTotal.set(0);
    popDoneCount = 0;
    popTotalExpected = 0;
    return;
  }

  if (payload.total > 0) popularTotal.set(payload.total);
  if (payload.current > 0) popularCurrent.set(payload.current);
  if (payload.message) popularMsg.set(payload.message);
  console.log('Popular progress:', payload);
  switch (payload.status) {
    case ProgressStatus.Processing:
      popDoneCount = 0;
      popTotalExpected = payload.total;
      popularRunning.set(true);
      break;
    case ProgressStatus.Metadata:
      popularPhase.set('syncing');
      refreshCounts();
      const vm = get(viewMode);
      if (vm === 'popular')
        scheduleGridRefresh(payload.class_id);
      break;
    case ProgressStatus.Done:
      popDoneCount++;
      popularPhase.set('images');
      const vm2 = get(viewMode);
      if (vm2 === 'popular') {
        if (payload.class_id)
          scheduleGridRefresh(payload.class_id);
        const pid = payload.preset_id;
        const cls = payload.class_name;
        toast.show(`${cls} #${pid} synced`, 'success', 5000, async () => {
          const preset = await api.getPresetById(pid);
          if (preset) presetDetail.set(preset);
        });
      }
      if (popDoneCount === popTotalExpected) {
        popularRunning.set(false);
        popularMsg.set('');
        popularTotal.set(0);
        popularCurrent.set(0);
        popularPhase.set('');
      }
      break;
  }
}
