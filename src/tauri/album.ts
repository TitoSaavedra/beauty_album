import { invoke } from '@tauri-apps/api/core';

export interface ClassEntry {
    name: string;
    icon_path: string | null;
    preset_count: number;
}

export interface PresetEntry {
    name?: string;
    title?: string;
    preset_id?: string;
    downloads?: number;
    favorites?: number;
    likes?: number;
    views?: number;
    creator?: string;
    date?: string;
    updated_at?: number;
    is_downloaded?: boolean;
    is_popular?: boolean;
    image_paths: string[];
    download_path: string | null;
    [key: string]: unknown;
}

export interface AppConfig {
    bdo_docs_dir: string;
    cf_clearance: string;
}

export const getClasses = (): Promise<ClassEntry[]> =>
    invoke('get_classes');

export const getPresets = (className: string): Promise<PresetEntry[]> =>
    invoke('get_presets', { className });

export const getConfig = (): Promise<AppConfig> =>
    invoke('get_config');

export const saveConfig = (config: AppConfig): Promise<void> =>
    invoke('save_config', { config });

export const injectPreset = (downloadPath: string): Promise<void> =>
    invoke('inject_preset', { downloadPath });

export const openFile = (path: string): Promise<void> =>
    invoke('open_file', { path });

export const runScrapper = (): Promise<string> =>
    invoke('run_scrapper');

export const stopScrapper = (): Promise<void> =>
    invoke('stop_scrapper');

export const checkPending = (): Promise<number> =>
    invoke('check_pending');

export const openLogs = (): Promise<void> =>
    invoke('open_logs');

export const initClasses = (): Promise<void> =>
    invoke('init_classes');

export const syncPopular = (): Promise<string> =>
    invoke('sync_popular');

export const getPopularClasses = (): Promise<ClassEntry[]> =>
    invoke('get_popular_classes');

export const getPopularPresets = (className: string): Promise<PresetEntry[]> =>
    invoke('get_popular_presets', { className });

export const openUrl = (url: string): Promise<void> =>
    invoke('open_url', { url });

export const discardPreset = (presetId: string): Promise<void> =>
    invoke('discard_preset', { presetId });

export const getWanted = (): Promise<string[]> =>
    invoke('get_wanted');

export const setWanted = (ids: string[]): Promise<void> =>
    invoke('set_wanted', { ids });

