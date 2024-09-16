import '../styles.css';
import React from 'react';
import ReactDOM from 'react-dom/client';
import CollectionsProvider from '@global/contexts/collectionsProvider';
import LibraryProvider from '@global/contexts/libraryProvider';
import SearchGamesProvider from '@global/contexts/searchGamesProvider';
import theme from '@global/theme';
import GlobalStyles from '@global/theme/globalStyles';
import { MantineProvider } from '@mantine/core';
import { ThemeProvider } from 'styled-components';

import Quicklaunch from './quicklaunch'

ReactDOM.createRoot(
  document.getElementById('quicklaunch-root') as HTMLElement,
).render(
  <React.StrictMode>
    <ThemeProvider theme={theme}>
      <MantineProvider>
        <LibraryProvider>
          <CollectionsProvider>
            <SearchGamesProvider>
              <GlobalStyles />
              <Quicklaunch />
            </SearchGamesProvider>
          </CollectionsProvider>
        </LibraryProvider>
      </MantineProvider>  
    </ThemeProvider>
  </React.StrictMode>,
);
