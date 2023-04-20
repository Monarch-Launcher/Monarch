import React from 'react';
import ReactDOM from 'react-dom/client';
import { appWindow, LogicalSize } from '@tauri-apps/api/window';
import App from './app';
import './styles.css';

// Set minimum size of window
await appWindow.setMinSize(new LogicalSize(800, 500));

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
