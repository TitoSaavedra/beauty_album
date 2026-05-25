import { writable, get } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import * as api from '../tauri/album';
import { toast } from '../../../shared/stores/toast';
import { presetDetail } from './presetDetail';
import { scheduleGridRefresh, viewMode, selectedClassObj } from './presetGrid';

interface ScrapperProgress {
  preset_id: string;
  status: string;
  message: string;
  class_name: string;
  class_id: number;
  current: number;
  total: number;
}

export const scrapperRunning = writable(false);
export const scrapperCurrent = writable(0);
export const scrapperTotal = writable(0);
export const scrapperMsg = writable('');
export const scrapperError = writable('');
export const scrapperPhase = writable<'presets' | 'images' | ''>('');

export async function startScraper() {
  if (get(scrapperRunning)) return;
  scrapperRunning.set(true);
  scrapperCurrent.set(0);
  scrapperTotal.set(0);
  scrapperMsg.set('Starting...');
  scrapperError.set('');
  scrapperPhase.set('');

  const unlisten = await listen<ScrapperProgress>('scrapper_progress', ({ payload }) => {
    if (payload.total > 0) scrapperTotal.set(payload.total);
    if (payload.current > 0) scrapperCurrent.set(payload.current);
    if (payload.message) scrapperMsg.set(payload.message);

    if (payload.status === 'metadata') {
      scrapperPhase.set('presets');
      const vm = get(viewMode);
      const scObj = get(selectedClassObj);
      if (vm === 'presets' && payload.class_id && payload.class_id === scObj?.class_id)
        scheduleGridRefresh(payload.class_id);
    } else if (payload.status === 'done') {
      scrapperPhase.set('images');
      const vm = get(viewMode);
      const scObj = get(selectedClassObj);
      if (vm === 'presets' && payload.class_id && payload.class_id === scObj?.class_id)
        scheduleGridRefresh(payload.class_id);
      const pid = payload.preset_id;
      const cls = payload.class_name;
      toast.show(`${cls} #${pid} downloaded`, 'success', 5000, async () => {
        const list = await api.getPresets(cls);
        const found = list.find(p => p.preset_id === pid);
        if (found) presetDetail.set(found);
      });
    } else if (payload.status === 'error') {
      scrapperError.set(payload.message);
    } else if (payload.status === 'cancelled') {
      scrapperMsg.set('Cancelled');
    }
  });

  let alreadyRunning = false;
  try {
    const doneMsg = await api.runScrapper();
    if (doneMsg) toast.show(doneMsg, 'success', 6000);
  } catch (e) {
    const msg = String(e);
    if (msg.includes('409') || msg.includes('Conflict')) {
      alreadyRunning = true;
    } else {
      scrapperMsg.set(msg);
      scrapperError.set(msg);
    }
  } finally {
    unlisten();
    if (!alreadyRunning) {
      scrapperRunning.set(false);
      scrapperMsg.set('');
      scrapperPhase.set('');
    }
  }
}
