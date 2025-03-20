import { Terminal } from '@xterm/xterm';
import "@xterm/xterm/css/xterm.css";
import { invoke } from '@tauri-apps/api';



var term = new Terminal();
term.open(document.getElementById('terminal') as HTMLElement);
term.write('Hello from \x1B[1;3;31mxterm.js\x1B[0m $ ')

// Write data from pty into the terminal
function writeToTerminal(data: string) {
  return new Promise<void>((r) => {
    term.write(data, () => r());
  });
}

async function readFromPty() {
  const data = await invoke<string>("async_read_from_pty");

  if (data) {
    await writeToTerminal(data);
  }
  
  window.requestAnimationFrame(readFromPty);
}
  
window.requestAnimationFrame(readFromPty);