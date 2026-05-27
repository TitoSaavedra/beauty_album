import { writable } from 'svelte/store';
import type { PresetEntry } from '../tauri/album';

export const presetDetail = writable<PresetEntry | null>(null);
