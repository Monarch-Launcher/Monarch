import './styles.css';

import { initZoomControls } from '@global/zoomControls';
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
import React from 'react';
import ReactDOM from 'react-dom/client';

import App from './App';

const appWindow = getCurrentWindow();

// Set quicklaunch global shortcuts
// await initShortcuts();

// Set minimum size of window
await appWindow.setMinSize(new LogicalSize(800, 600));

// Initialize global UI zoom controls
initZoomControls();

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
