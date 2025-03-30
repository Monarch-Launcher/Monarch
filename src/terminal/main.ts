import { Terminal } from '@xterm/xterm';
import '@xterm/xterm/css/xterm.css';
import { FitAddon } from '@xterm/addon-fit';
import { invoke } from '@tauri-apps/api';

var term = new Terminal();
term.open(document.getElementById('terminal') as HTMLElement);
term.onData(writeToPty);

const fitAddon = new FitAddon();
term.loadAddon(fitAddon);
fitAddon.fit();

// Write data from the terminal to the pty
function writeToPty(data: string) {
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
  console.log(data);

  if (data) {
    await writeToTerminal(data);
  }

  window.requestAnimationFrame(readFromPty);
}

window.requestAnimationFrame(readFromPty);
