import './styles/app.css';
import { waitLocale } from 'svelte-i18n';
import './i18n';

waitLocale().then(() => {
  import('./app/App.svelte').then(({ default: App }) => {
    new App({ target: document.getElementById('app')! });
  });
});
