import { writable } from 'svelte/store';
import * as api from '../tauri/album';
import type { ScrapperProgress } from '../tauri/album';
import { ProgressStatus } from '../tauri/album';
import { toast } from '../../shared/stores/toast';
import { presetDetail } from './presetDetail';
import { scheduleGridRefresh } from './presetGrid';
import { refreshCounts } from './classes';

export const personalRunning = writable(false);
export const personalCurrent = writable(0);
export const personalTotal   = writable(0);
export const personalMsg     = writable('');
export const personalError   = writable('');
export const personalPhase   = writable<'presets' | 'images' | ''>('');

let personDoneCount = 0;
let personTotalExpected = 0;

export function handlePersonalProgress(payload: ScrapperProgress) {
  if (payload.class_name === 'SYNC' && payload.status === ProgressStatus.Done) {
    personalRunning.set(false);
    personalMsg.set('');
    personalPhase.set('');
    personalCurrent.set(0);
    personalTotal.set(0);
    personDoneCount = 0;
    personTotalExpected = 0;
    return;
  }

  if (payload.total > 0) personalTotal.set(payload.total);
  if (payload.current > 0) personalCurrent.set(payload.current);
  if (payload.message) personalMsg.set(payload.message);
  console.log('Personal progress:', payload);
  switch (payload.status) {
    case ProgressStatus.Processing:
      personDoneCount = 0;
      personTotalExpected = payload.total;
      personalRunning.set(true);
      break;
    case ProgressStatus.Metadata:
      personalPhase.set('presets');
      refreshCounts();
      scheduleGridRefresh(payload.class_id);
      break;
    case ProgressStatus.Done:
      personDoneCount++;
      personalPhase.set('images');
      scheduleGridRefresh(payload.class_id);
      if (personDoneCount === personTotalExpected) {
        personalRunning.set(false);
        personalMsg.set('');
        personalPhase.set('');
        personalCurrent.set(0);
        personalTotal.set(0);
      }
      const pid = payload.preset_id;
      const cls = payload.class_name;
      toast.show(`${cls} #${pid} downloaded`, 'success', 5000, async () => {
        const preset = await api.getPresetById(pid);
        if (preset) presetDetail.set(preset);
      });
      break;
  }
}
