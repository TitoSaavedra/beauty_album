import { invoke } from '@tauri-apps/api/core';

export interface AppConfig {
    bdo_docs_dir: string;
    cf_clearance: string;
}

export const getConfig = (): Promise<AppConfig> =>
    invoke('get_config');

export const saveConfig = (config: AppConfig): Promise<void> =>
    invoke('save_config', { config });

export const isDbReady = (): Promise<boolean> =>
    invoke('is_db_ready');
