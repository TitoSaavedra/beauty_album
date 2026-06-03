import { writable } from 'svelte/store';
import type { AppConfig } from '../tauri/config';

export const appConfig = writable<AppConfig>({ bdo_docs_dir: '', cf_clearance: '' });
