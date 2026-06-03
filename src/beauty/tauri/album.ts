import { invoke } from '@tauri-apps/api/core';

// ─────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────

export interface ClassEntry {
    class_id: number;
    name: string;
    icon_svg?: string | null;
    preset_count: number;
    popular_count?: number;
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

export interface PopularStats {
    total: number;
    d20: number; d30: number; d60: number;
    d90: number; d180: number; d365: number;
}

export enum ProgressStatus {
    Processing = 'processing',
    Metadata = 'metadata',
    Done = 'done',
    Cancelled = 'cancelled',
}

export enum ProgressType {
    Preset = 'preset',
    Popular = 'popular',
}

export interface ScrapperProgress {
    preset_id: string;
    status: ProgressStatus;
    message: string;
    class_name: string;
    class_id: number;
    current: number;
    total: number;
    progress_type: ProgressType;
}

// ─────────────────────────────────────────────────────────────────
// Database State
// ─────────────────────────────────────────────────────────────────

export const isDbReady = (): Promise<boolean> =>
    invoke('is_db_ready');

// ─────────────────────────────────────────────────────────────────
// Config
// ─────────────────────────────────────────────────────────────────

export const getConfig = (): Promise<AppConfig> =>
    invoke('get_config');

export const saveConfig = (config: AppConfig): Promise<void> =>
    invoke('save_config', { config });

// ─────────────────────────────────────────────────────────────────
// Classes
// ─────────────────────────────────────────────────────────────────

export const getClasses = (): Promise<ClassEntry[]> =>
    invoke('get_classes');

export const getPopularStats = (className: string): Promise<PopularStats> =>
    invoke('get_popular_stats', { className });

export const getPopularRegions = (): Promise<string[]> =>
    invoke('get_popular_regions');

export const getClassFavorites = (): Promise<string[]> =>
    invoke('get_class_favorites');

export const setClassFavorite = (className: string, isFavorite: boolean): Promise<void> =>
    invoke('set_class_favorite', { className, isFavorite });


// ─────────────────────────────────────────────────────────────────
// Presets (Read)
// ─────────────────────────────────────────────────────────────────

export const getPresets = (
    className: string,
    offset = 0,
    limit = 50,
    sortBy = 'downloads',
    search = '',
): Promise<PresetEntry[]> =>
    invoke('get_presets', { className, offset, limit, sortBy, search });

export const getPopularPresets = (
    className: string,
    offset = 0,
    limit = 50,
    sortBy = 'downloads',
    search = '',
    sinceTs = 0,
    region = '',
): Promise<{ presets: PresetEntry[]; wanted: PresetEntry[] }> =>
    invoke('get_popular_presets', { className, offset, limit, sortBy, search, sinceTs, region });

export const getPresetById = (presetId: string): Promise<PresetEntry> =>
    invoke('get_preset_by_id', { presetId });

// ─────────────────────────────────────────────────────────────────
// Presets (Write)
// ─────────────────────────────────────────────────────────────────

export const discardPreset = (presetId: string): Promise<void> =>
    invoke('discard_preset', { presetId });

export const getWanted = (): Promise<string[]> =>
    invoke('get_wanted');

export const setWanted = (ids: string[]): Promise<void> =>
    invoke('set_wanted', { ids });

export const toggleWanted = (presetId: string): Promise<boolean> =>
    invoke('toggle_wanted', { presetId });

// ─────────────────────────────────────────────────────────────────
// File & URL Operations
// ─────────────────────────────────────────────────────────────────

export const injectPreset = (downloadPath: string): Promise<void> =>
    invoke('inject_preset', { downloadPath });

export const openFile = (path: string): Promise<void> =>
    invoke('open_file', { path });

export const openUrl = (url: string): Promise<void> =>
    invoke('open_url', { url });

