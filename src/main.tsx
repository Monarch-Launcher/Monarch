import './styles.css';

import { appWindow, LogicalSize } from '@tauri-apps/api/window';
import React from 'react';
import ReactDOM from 'react-dom/client';

import App from './App';

// Set minimum size of window
await appWindow.setMinSize(new LogicalSize(800, 600));

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
