import { writable } from 'svelte/store';

export interface ToastMessage {
    id: number;
    text: string;
    type: 'success' | 'error';
}

let nextId = 0;

function createToast() {
    const { subscribe, update } = writable<ToastMessage[]>([]);
    return {
        subscribe,
        show(text: string, type: ToastMessage['type'] = 'success', duration = 3500) {
            const id = ++nextId;
            update(list => [...list, { id, text, type }]);
            setTimeout(() => {
                update(list => list.filter(t => t.id !== id));
            }, duration);
        },
        dismiss(id: number) {
            update(list => list.filter(t => t.id !== id));
        },
    };
}

export const toast = createToast();
