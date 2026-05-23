import { writable } from 'svelte/store';
import { getWanted, toggleWanted } from '../tauri/album';

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
    async toggle(id: string) {
      try {
        const isWanted = await toggleWanted(id);
        update(s => {
          const next = new Set(s);
          if (isWanted) { next.add(id); } else { next.delete(id); }
          return next;
        });
      } catch { /* non-fatal */ }
    },
  };
}

export const wantedPresets = createWantedStore();
