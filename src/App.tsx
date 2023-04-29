import { ThemeProvider } from 'styled-components';
import { MantineProvider } from '@mantine/core';
import Routes from '@global/routes';
import GlobalStyles from '@global/theme/globalStyles';
import theme from '@global/theme';
import SearchGamesProvider from '@global/contexts/searchGamesProvider';
import LibraryProvider from '@global/contexts/libraryProvider';

const App = () => {
  return (
    <MantineProvider>
      <ThemeProvider theme={theme}>
        <LibraryProvider>
          <SearchGamesProvider>
            <GlobalStyles />
            <Routes />
          </SearchGamesProvider>
        </LibraryProvider>
      </ThemeProvider>
    </MantineProvider>
  );
};

export default App;
