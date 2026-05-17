import { writable } from 'svelte/store';
import type { AppConfig } from '../tauri/album';

export const appConfig = writable<AppConfig>({ album_dir: '', bdo_output_dir: '', album_input_dir: '' });
