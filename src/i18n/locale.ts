import { derived } from 'svelte/store';
import { locale as i18nLocale } from 'svelte-i18n';

export const preferredLocale = derived(i18nLocale, $locale => {
  if (typeof window !== 'undefined') {
    localStorage.setItem('preferred-locale', $locale);
  }
  return $locale;
});

export function setPreferredLocale(lang: string) {
  i18nLocale.set(lang);
}

export function getStoredLocale(): string | null {
  if (typeof window !== 'undefined') {
    return localStorage.getItem('preferred-locale');
  }
  return null;
}
