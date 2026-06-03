import { writable } from 'svelte/store';

interface LightboxState {
    images: string[];
    index: number;
}

function createLightbox() {
    const { subscribe, set, update } = writable<LightboxState | null>(null);

    return {
        subscribe,
        open: (images: string[], index = 0) => set({ images, index }),
        close: () => set(null),
        next: () => update(s => s ? { ...s, index: (s.index + 1) % s.images.length } : s),
        prev: () => update(s => s ? { ...s, index: (s.index - 1 + s.images.length) % s.images.length } : s),
    };
}

export const lightbox = createLightbox();
