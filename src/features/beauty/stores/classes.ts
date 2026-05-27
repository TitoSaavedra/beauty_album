import { writable } from 'svelte/store';
import type { ClassEntry } from '../tauri/album';

export const classes = writable<ClassEntry[]>([]);
