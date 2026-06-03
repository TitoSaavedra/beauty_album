import { writable, get } from 'svelte/store';
import * as api from '../tauri/album';
import type { ClassEntry, PresetEntry, PopularStats } from '../tauri/album';

const PAGE_SIZE = 50;

export const viewMode = writable<'presets' | 'popular'>('presets');
export const filterSortBy = writable<'downloads' | 'views' | 'favorites'>('downloads');
export const filterSinceTs = writable(0);
export const filterRegion = writable('');
export const searchQuery = writable('');
export const popularRegions = writable<string[]>([]);
export const selectedClass = writable<string | null>(null);
export const selectedClassObj = writable<ClassEntry | null>(null);
export const presets = writable<PresetEntry[]>([]);
export const loadingPresets = writable(false);
export const presetsError = writable('');
export const popularStats = writable<PopularStats | null>(null);
export const presetOffset = writable(0);
export const hasMore = writable(false);
export const loadingMore = writable(false);

let gridRefreshTimer: ReturnType<typeof setTimeout> | null = null;

export function scheduleGridRefresh(classId: number) {
  if (gridRefreshTimer !== null) clearTimeout(gridRefreshTimer);
  gridRefreshTimer = setTimeout(async () => {
    gridRefreshTimer = null;
    const sc = get(selectedClass);
    const scObj = get(selectedClassObj);
    if (classId !== scObj?.class_id || !sc) return;
    const vm = get(viewMode);
    const sortBy = get(filterSortBy);
    const sq = get(searchQuery);
    const sinceTs = get(filterSinceTs);
    const region = get(filterRegion);
    try {
      if (vm === 'popular') {
        const { presets: results, wanted } = await api.getPopularPresets(sc, 0, PAGE_SIZE, sortBy, sq, sinceTs, region);
        const wantedIds = new Set(wanted.map(p => p.preset_id));
        presets.set([...wanted, ...results.filter(p => !wantedIds.has(p.preset_id))]);
        presetOffset.set(0);
        hasMore.set(results.length === PAGE_SIZE);
      } else {
        const results = await api.getPresets(sc, 0, PAGE_SIZE, sortBy, sq);
        presets.set(results);
        presetOffset.set(0);
        hasMore.set(results.length === PAGE_SIZE);
      }
    } catch { /* non-fatal */ }
  }, 400);
}

export async function selectClass(name: string) {
  selectedClass.set(name);
  presets.set([]);
  presetsError.set('');
  presetOffset.set(0);
  hasMore.set(false);
  loadingPresets.set(true);
  const vm = get(viewMode);
  const sortBy = get(filterSortBy);
  const sq = get(searchQuery);
  const sinceTs = get(filterSinceTs);
  const region = get(filterRegion);
  try {
    if (vm === 'popular') {
      const [{ presets: results, wanted }, stats] = await Promise.all([
        api.getPopularPresets(name, 0, PAGE_SIZE, sortBy, sq, sinceTs, region),
        api.getPopularStats(name),
      ]);
      const wantedIds = new Set(wanted.map(p => p.preset_id));
      presets.set([...wanted, ...results.filter(p => !wantedIds.has(p.preset_id))]);
      popularStats.set(stats);
      hasMore.set(results.length === PAGE_SIZE);
    } else {
      const results = await api.getPresets(name, 0, PAGE_SIZE, sortBy, sq);
      presets.set(results);
      hasMore.set(results.length === PAGE_SIZE);
    }
  } catch (err) {
    presetsError.set(String(err));
  } finally {
    loadingPresets.set(false);
  }
}

export async function loadMore() {
  if (get(loadingMore) || !get(hasMore)) return;
  const sc = get(selectedClass);
  if (!sc) return;
  loadingMore.set(true);
  const vm = get(viewMode);
  const sortBy = get(filterSortBy);
  const sq = get(searchQuery);
  const sinceTs = get(filterSinceTs);
  const region = get(filterRegion);
  const nextOffset = get(presetOffset) + PAGE_SIZE;
  try {
    if (vm === 'popular') {
      const { presets: results, wanted } = await api.getPopularPresets(sc, nextOffset, PAGE_SIZE, sortBy, sq, sinceTs, region);
      presets.update(p => {
        const existingIds = new Set(p.map(x => x.preset_id));
        return [...p, ...wanted.filter(x => !existingIds.has(x.preset_id)), ...results.filter(x => !existingIds.has(x.preset_id))];
      });
      presetOffset.set(nextOffset);
      hasMore.set(results.length === PAGE_SIZE);
    } else {
      const results = await api.getPresets(sc, nextOffset, PAGE_SIZE, sortBy, sq);
      presets.update(p => [...p, ...results]);
      presetOffset.set(nextOffset);
      hasMore.set(results.length === PAGE_SIZE);
    }
  } catch (err) {
    presetsError.set(String(err));
  } finally {
    loadingMore.set(false);
  }
}

export async function setMode(mode: 'presets' | 'popular') {
  if (get(viewMode) === mode) return;
  viewMode.set(mode);
  presets.set([]);
  filterSortBy.set('downloads');
  filterRegion.set('');
  if (mode === 'popular') {
    try {
      popularRegions.set(await api.getPopularRegions());
    } catch { /* non-fatal */ }
  }
  const sc = get(selectedClass);
  if (sc) selectClass(sc);
}
