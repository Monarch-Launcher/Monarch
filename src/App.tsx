import { ThemeProvider } from 'styled-components';
import Routes from './global/routes';
import GlobalStyles from './global/theme/globalStyles';
import theme from './global/theme';

const App = () => {
  return (
    <ThemeProvider theme={theme}>
      <GlobalStyles />
      <Routes />
    </ThemeProvider>
  );
};

export default App;
