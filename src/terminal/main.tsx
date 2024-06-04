import '../styles.css';
import LibraryProvider from '@global/contexts/libraryProvider';
import theme from '@global/theme';
import GlobalStyles from '@global/theme/globalStyles';
import React from 'react';
import ReactDOM from 'react-dom/client';
import { ThemeProvider } from 'styled-components';
import QuickLaunch from './_components/quicklaunch';

ReactDOM.createRoot(
  document.getElementById('terminal-root') as HTMLElement,
).render(
  <React.StrictMode>
    <ThemeProvider theme={theme}>
      <GlobalStyles />
      <Terminal />
    </ThemeProvider>
  </React.StrictMode>,
);
