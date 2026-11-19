import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import { initRendererProfile } from './profile';
import { installTauriBridge } from './platform/tauriBridge';

installTauriBridge();
initRendererProfile();

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
