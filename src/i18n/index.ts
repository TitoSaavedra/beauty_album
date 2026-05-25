import { register, init, getLocaleFromNavigator } from 'svelte-i18n';

register('en', () => import('./locales/en.json'));
register('es', () => import('./locales/es.json'));
register('pt-BR', () => import('./locales/pt-BR.json'));

let savedLocale: string | null = null;
if (typeof window !== 'undefined' && typeof localStorage !== 'undefined') {
  savedLocale = localStorage.getItem('app-locale');
}

const initialLocale = savedLocale || getLocaleFromNavigator() || 'en';

init({
  fallbackLocale: 'en',
  initialLocale,
});
