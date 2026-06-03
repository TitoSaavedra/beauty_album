import { writable } from 'svelte/store';

export const appLoading   = writable(true);
export const appError     = writable('');
export const initStatus   = writable('Initializing...');
export const settingsOpen = writable(false);
