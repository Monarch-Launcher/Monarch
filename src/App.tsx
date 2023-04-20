import { ThemeProvider } from 'styled-components';
import Routes from './global/routes';
import GlobalStyles from './global/theme/globalStyles';
import theme from './global/theme';
import SearchGamesProvider from './global/contexts/searchGamesProvider';
import LibraryProvider from './global/contexts/libraryProvider';

const App = () => {
  return (
    <LibraryProvider>
      <SearchGamesProvider>
        <ThemeProvider theme={theme}>
          <GlobalStyles />
          <Routes />
        </ThemeProvider>
      </SearchGamesProvider>
    </LibraryProvider>
  );
};

export default App;
