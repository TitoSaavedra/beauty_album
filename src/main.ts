import './styles/app.css';
import { waitLocale } from 'svelte-i18n';
import App from './app/App.svelte';
import './i18n';

waitLocale().then(() => {
  new App({ target: document.getElementById('app')! });
});
