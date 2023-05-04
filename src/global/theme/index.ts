export const colors = {
  white: '#FFF',
  black: '#0F0F0F98',
  lightgreen: '#14CA26',
  darkgreen: '#0E8628',
  primary: '#FF6000',
  secondary: '#454545',
  error: '#FF3333',
  background: '#2F2F2F',
  button: {
    primary: {
      text: '#454545',
      border: '#FF6000',
      background: '#FF6000',
      hoverBorder: '#FF3000',
      hoverText: '#000',
      hoverBackground: '#FF3000',
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
      text: '#FF6000',
      border: '#454545',
      background: '#454545',
      hoverBorder: '#373737',
      hoverText: '#FF6000',
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
