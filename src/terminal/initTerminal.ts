import { invoke } from '@tauri-apps/api';

const initTerminal = async () => {
  await invoke('init_terminal');
};

export default initTerminal;
