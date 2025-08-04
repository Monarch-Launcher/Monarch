import '@xterm/xterm/css/xterm.css';
import { invoke } from '@tauri-apps/api/core';
import { FitAddon } from '@xterm/addon-fit';
import { Terminal } from '@xterm/xterm';

const term = new Terminal({
  theme: {
    background: '#181818', // Monarch dark background
    foreground: '#FF6000', // Monarch orange
    cursor: '#FF6000',
    black: '#181818',
    red: '#FF3333',
    green: '#14CA26',
    yellow: '#FF6000',
    blue: '#2F2F2F',
    magenta: '#FF6000',
    cyan: '#14CA26',
    white: '#f6f6f6',
    brightBlack: '#454545',
    brightRed: '#FF3333',
    brightGreen: '#14CA26',
    brightYellow: '#FF6000',
    brightBlue: '#2F2F2F',
    brightMagenta: '#FF6000',
    brightCyan: '#14CA26',
    brightWhite: '#f6f6f6',
  },
  fontFamily: 'IBM Plex Mono, Courier New, Courier, monospace',
  fontSize: 16,
  cursorBlink: true,
});
term.open(document.getElementById('terminal') as HTMLElement);
// eslint-disable-next-line @typescript-eslint/no-use-before-define
term.onData(writeToPty);

const fitAddon = new FitAddon();
term.loadAddon(fitAddon);
fitAddon.fit();

// Ensure terminal resizes with window
window.addEventListener('resize', () => {
  fitAddon.fit();
});

// Write data from the terminal to the pty
function writeToPty(data: string) {
  // eslint-disable-next-line no-void
  void invoke('async_write_to_pty', {
    data,
  });
}

// Write data from pty into the terminal
function writeToTerminal(data: string) {
  return new Promise<void>((r) => {
    term.write(data, () => r());
  });
}

async function readFromPty() {
  const data = await invoke<string>('async_read_from_pty');

  if (data) {
    await writeToTerminal(data);
  }

  window.requestAnimationFrame(readFromPty);
}

window.requestAnimationFrame(readFromPty);
