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
    views?: number;
    creator?: string;
    date?: string;
    image_paths: string[];
    download_path: string | null;
    [key: string]: unknown;
}

export interface AppConfig {
    album_dir: string;
    bdo_output_dir: string;
    album_input_dir: string;
}

export const getClasses = (): Promise<ClassEntry[]> =>
    invoke('get_classes');

export const getPresets = (className: string): Promise<PresetEntry[]> =>
    invoke('get_presets', { className });

export const getConfig = (): Promise<AppConfig> =>
    invoke('get_config');

export const saveConfig = (config: AppConfig): Promise<void> =>
    invoke('save_config', { config });

export const injectPreset = (downloadPath: string, outputDir: string): Promise<void> =>
    invoke('inject_preset', { downloadPath, outputDir });

export const openFile = (path: string): Promise<void> =>
    invoke('open_file', { path });

export const runScrapper = (): Promise<void> =>
    invoke('run_scrapper');

export const stopScrapper = (): Promise<void> =>
    invoke('stop_scrapper');
