export const colors = {
  white: '#FFF',
  black: '#0F0F0F98',
  primary: '#FF6000',
  secondary: '#454545',
  button: {
    primary: {
      text: '#454545',
      border: '#FF6000',
      background: '#FF6000',
      hoverBorder: '#FF2000',
      hoverText: '#000',
      hoverBackground: '#FF2000',
      focusText: '',
      focusBorder: '#FF2000',
      focusBackground: '#FF2000',
      active: '',
      disabled: {
        text: '#454545',
        border: '#FF6000',
        background: '#666',
        hoverBorder: '#666',
        hoverText: '#FFF',
        hoverBackground: '',
      },
    },
    secondary: {
      text: '#FF6000',
      border: '#454545',
      background: '#454545',
      hoverBorder: '#FF6000',
      hoverText: '#FFF',
      hoverBackground: '',
      focusText: '',
      focusBorder: '',
      focusBackground: '',
      active: '',
      disabled: {
        text: '#454545',
        border: '#FF6000',
        background: '#FF6000',
        hoverBorder: '#454545',
        hoverText: '#FFF',
        hoverBackground: '',
      },
    },
    transparent: {
      text: '#FF6000',
      border: 'transparent',
      background: 'transparent',
      hoverBorder: 'transparent',
      hoverText: '#FF6000',
      hoverBackground: '#6E6E6E',
      focusText: '#FF6000',
      focusBorder: 'transparent',
      focusBackground: 'transparent',
      active: '#6E6E6E',
      disabled: {
        text: '',
        border: '',
        background: '',
        hoverBorder: '',
        hoverText: '',
        hoverBackground: '',
      },
    },
  },
};

export type ButtonVariant = typeof colors.button;

type Theme = {
  colors: typeof colors;
};

const theme: Theme = {
  colors,
};

export default theme;
