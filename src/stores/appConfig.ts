import { writable } from 'svelte/store';
import type { AppConfig } from '../tauri/album';

export const appConfig = writable<AppConfig>({ bdo_docs_dir: '' });
