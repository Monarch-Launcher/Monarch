export const colors = {
  white: '#fff',
  black: '#181818',
  lightgreen: '#14CA26',
  darkgreen: '#0E8628',
  primary: '#FA5002',
  secondary: '#454545',
  error: '#FF3333',
  background: '#101014', // very dark blue-purple background
  button: {
    primary: {
      text: '#454545',
      border: '#FA5002',
      background: '#FA5002',
      hoverBorder: '#FF6000',
      hoverText: '#000',
      hoverBackground: '#FF6000',
      focusText: '',
      focusBorder: '#FF3000',
      focusBackground: '#FF3000',
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
      text: '#FA5002',
      border: '#454545',
      background: '#454545',
      hoverBorder: '#373737',
      hoverText: '#FA5002',
      hoverBackground: '#373737',
      focusText: '',
      focusBorder: '#373737',
      focusBackground: '#373737',
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
    menu: {
      text: '#FA5002',
      border: 'transparent',
      background: 'transparent',
      hoverBorder: 'transparent',
      hoverText: '#FA5002',
      hoverBackground: '#6E6E6E',
      focusText: '#FA5002',
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
    icon: {
      text: '',
      border: 'transparent',
      background: 'transparent',
      hoverBorder: 'transparent',
      hoverText: 'transparent',
      hoverBackground: 'transparent',
      focusText: 'transparent',
      focusBorder: 'transparent',
      focusBackground: 'transparent',
      active: 'transparent',
      disabled: {
        text: '',
        border: '',
        background: '',
        hoverBorder: '',
        hoverText: '',
        hoverBackground: '',
      },
    },
    danger: {
      text: '#FFF',
      border: '#FF3333',
      background: '#FF3333',
      hoverBorder: '#FD1111',
      hoverText: '#FFF',
      hoverBackground: '#FD1111',
      focusText: '#FFF',
      focusBorder: '#FD1111',
      focusBackground: '#FD1111',
      active: '#FD1111',
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
