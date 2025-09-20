import { createRoot } from 'react-dom/client';
import App from './App';
import './lib/i18n';

createRoot(document.getElementById('root')!).render(App());

document.addEventListener('DOMContentLoaded', () => {
  document.addEventListener('contextmenu', (e) => {
    e.preventDefault();
  });
});
