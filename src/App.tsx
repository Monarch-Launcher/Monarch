import CollectionsProvider from '@global/contexts/collectionsProvider';
import LibraryProvider from '@global/contexts/libraryProvider';
import SearchGamesProvider from '@global/contexts/searchGamesProvider';
import Routes from '@global/routes';
import theme from '@global/theme';
import GlobalStyles from '@global/theme/globalStyles';
import { MantineProvider } from '@mantine/core';
import { ThemeProvider } from 'styled-components';

const App = () => {
  return (
    <MantineProvider>
      <ThemeProvider theme={theme}>
        <LibraryProvider>
          <CollectionsProvider>
            <SearchGamesProvider>
              <GlobalStyles />
              <Routes />
            </SearchGamesProvider>
          </CollectionsProvider>
        </LibraryProvider>
      </ThemeProvider>
    </MantineProvider>
  );
};

export default App;
