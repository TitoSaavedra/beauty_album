import { writable } from 'svelte/store';

export const syncLoading = writable<string>('Initializing...');
