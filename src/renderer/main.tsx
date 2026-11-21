import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import ControlCenterApp from './ControlCenterApp';
import { initRendererProfile } from './profile';
import { installTauriBridge } from './platform/tauriBridge';
import { isControlCenterWindow } from './platform/windowKind';

installTauriBridge();
initRendererProfile();

const Root = isControlCenterWindow() ? ControlCenterApp : App;

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <Root />
  </React.StrictMode>
);
