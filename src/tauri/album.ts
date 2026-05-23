import { invoke } from '@tauri-apps/api/core';

export interface ClassEntry {
    name: string;
    icon_svg?: string | null;
    preset_count: number;
    is_favorite?: boolean;
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
    is_wanted?: boolean;
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

export const getPresets = (
    className: string,
    offset = 0,
    limit = 50,
    sortBy = 'downloads',
): Promise<PresetEntry[]> =>
    invoke('get_presets', { className, offset, limit, sortBy });

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

export const checkPendingImage2 = (): Promise<number> =>
    invoke('check_pending_image2');

export const isDbReady = (): Promise<boolean> =>
    invoke('is_db_ready');

export const openLogs = (): Promise<void> =>
    invoke('open_logs');

export const syncPopular = (): Promise<string> =>
    invoke('sync_popular');

export const getPopularClasses = (): Promise<ClassEntry[]> =>
    invoke('get_popular_classes');

export const getPopularPresets = (
    className: string,
    offset = 0,
    limit = 50,
    sortBy = 'downloads',
): Promise<PresetEntry[]> =>
    invoke('get_popular_presets', { className, offset, limit, sortBy });

export const openUrl = (url: string): Promise<void> =>
    invoke('open_url', { url });

export const discardPreset = (presetId: string): Promise<void> =>
    invoke('discard_preset', { presetId });

export const getWanted = (): Promise<string[]> =>
    invoke('get_wanted');

export const setWanted = (ids: string[]): Promise<void> =>
    invoke('set_wanted', { ids });

export const toggleWanted = (presetId: string): Promise<boolean> =>
    invoke('toggle_wanted', { presetId });

export const getClassFavorites = (): Promise<string[]> =>
    invoke('get_class_favorites');

export const setClassFavorite = (className: string, isFavorite: boolean): Promise<void> =>
    invoke('set_class_favorite', { className, isFavorite });

export interface LogEntry {
    id: number;
    ts: number;
    tag: string;
    source: string;
    msg: string;
}

export interface LogStats {
    total: number;
    errors: number;
    by_source: { source: string; count: number }[];
    by_tag: { tag: string; count: number }[];
}

export const getLogs = (
    tag?: string,
    source?: string,
    limit = 100,
    offset = 0,
): Promise<LogEntry[]> =>
    invoke('get_logs', { tag, source, limit, offset });

export const getLogStats = (): Promise<LogStats> =>
    invoke('get_log_stats');
