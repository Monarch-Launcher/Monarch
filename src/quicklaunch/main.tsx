import '../styles.css';
import theme from '@global/theme';
import GlobalStyles from '@global/theme/globalStyles';
import React from 'react';
import ReactDOM from 'react-dom/client';
import { ThemeProvider } from 'styled-components';
import Quicklaunch from './quicklaunch'

ReactDOM.createRoot(
  document.getElementById('quicklaunch-root') as HTMLElement,
).render(
  <React.StrictMode>
    <ThemeProvider theme={theme}>
      <GlobalStyles />
      <Quicklaunch />
    </ThemeProvider>
  </React.StrictMode>,
);
