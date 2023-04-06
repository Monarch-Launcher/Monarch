import * as React from 'react';
import { createGlobalStyle } from 'styled-components';

const MainGlobalStyles = createGlobalStyle`
  html {
    height: 100%;
    box-sizing: border-box;
    font-family: 'Roboto';

    line-height:1.5;
    color: #252F3F;
  }
  body {
    position: relative;
    height: 100%;
    margin: 0;
  }
  #root {
    height: 100%;
    overflow: auto;
  }

`;

const GlobalStyles = React.memo(() => <MainGlobalStyles />);

GlobalStyles.displayName = 'GlobalStyles';

export default GlobalStyles;
