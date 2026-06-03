import { writable } from 'svelte/store';

export interface ConfirmOptions {
    title: string;
    text: string;
    confirmLabel?: string;
    onConfirm: () => void | Promise<void>;
}

function createConfirmDialog() {
    const { subscribe, set } = writable<ConfirmOptions | null>(null);
    return {
        subscribe,
        show: (opts: ConfirmOptions) => set(opts),
        close: () => set(null),
    };
}

export const confirmDialog = createConfirmDialog();
