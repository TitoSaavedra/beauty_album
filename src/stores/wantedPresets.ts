import { writable } from 'svelte/store';
import { getWanted, setWanted } from '../tauri/album';

function createWantedStore() {
  const { subscribe, set, update } = writable<Set<string>>(new Set());

  return {
    subscribe,
    async load() {
      try {
        const ids = await getWanted();
        set(new Set(ids));
      } catch { /* ignore on first launch */ }
    },
    toggle(id: string) {
      update(s => {
        const next = new Set(s);
        next.has(id) ? next.delete(id) : next.add(id);
        setWanted([...next]);
        return next;
      });
    },
  };
}

export const wantedPresets = createWantedStore();
