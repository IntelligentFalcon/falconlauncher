import { createRoot } from 'react-dom/client';
import './lib/i18n';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import React from 'react';
import App from './App';

createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);

document.addEventListener('DOMContentLoaded', () => {
  document.addEventListener('contextmenu', (e) => {
    e.preventDefault();
  });
});
