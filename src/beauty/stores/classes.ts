import { writable } from 'svelte/store';
import type { ClassEntry } from '../tauri/album';
import { getClasses } from '../tauri/album';

export const classes = writable<ClassEntry[]>([]);

export async function refreshCounts() {
  try {
    const updated = await getClasses();
    const map = new Map(updated.map(c => [c.class_id, c]));
    classes.update(list =>
      list.map(c => {
        const fresh = map.get(c.class_id);
        if (!fresh) return c;
        return { ...c, preset_count: fresh.preset_count, popular_count: fresh.popular_count ?? 0 };
      })
    );
  } catch { /* non-fatal */ }
}
