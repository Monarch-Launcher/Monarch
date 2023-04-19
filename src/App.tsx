import { ThemeProvider } from 'styled-components';
import Routes from './global/routes';
import GlobalStyles from './global/theme/globalStyles';
import theme from './global/theme';
import GamesProvider from './global/contexts/gamesProvider';

const App = () => {
  return (
    <GamesProvider>
      <ThemeProvider theme={theme}>
        <GlobalStyles />
        <Routes />
      </ThemeProvider>
    </GamesProvider>
  );
};

export default App;
