import { invoke } from '@tauri-apps/api';
import { FitAddon } from '@xterm/addon-fit';
import { Terminal } from '@xterm/xterm';

const term = new Terminal();
term.open(document.getElementById('terminal') as HTMLElement);
// eslint-disable-next-line @typescript-eslint/no-use-before-define
term.onData(writeToPty);

const fitAddon = new FitAddon();
term.loadAddon(fitAddon);
fitAddon.fit();

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
