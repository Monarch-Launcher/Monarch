export const colors = {
  white: '#FFF',
  black: '#000',
};

type Theme = {
  colors: typeof colors;
};

const theme: Theme = {
  colors,
};

export default theme;
