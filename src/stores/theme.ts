import { writable } from 'svelte/store';

function createThemeStore() {
  const stored = typeof window !== 'undefined' ? localStorage.getItem('preferred-theme') : null;
  const initial = (stored as 'dark' | 'light') || 'dark';

  const { subscribe, set } = writable<'dark' | 'light'>(initial);

  return {
    subscribe,
    set: (value: 'dark' | 'light') => {
      localStorage.setItem('preferred-theme', value);
      document.documentElement.setAttribute('data-theme', value);
      set(value);
    },
    toggle: () => {
      const newTheme = initial === 'dark' ? 'light' : 'dark';
      localStorage.setItem('preferred-theme', newTheme);
      document.documentElement.setAttribute('data-theme', newTheme);
      set(newTheme);
    }
  };
}

export const theme = createThemeStore();

if (typeof window !== 'undefined') {
  const stored = localStorage.getItem('preferred-theme');
  const themeValue = (stored as 'dark' | 'light') || 'dark';
  document.documentElement.setAttribute('data-theme', themeValue);
}
