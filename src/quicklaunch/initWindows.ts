import { invoke } from '@tauri-apps/api';
import {
  isRegistered,
  register,
  unregisterAll,
} from '@tauri-apps/api/globalShortcut';
import type { MonarchWindow } from '@global/types';

// TODO: PROPER ERROR HANDLING
const initWindows = async () => {
  // Define global shortcuts for quicklaunch (proto-typing)
  await unregisterAll();

  const quickLaunchRegistered = await isRegistered('Shift+C');
  const closeRegistered = await isRegistered('Esc');

  // Build initial quicklaunch
  let window: MonarchWindow = await invoke('build_quicklaunch');
  
  if (!quickLaunchRegistered) {
    await register('Shift+C', async () => {
      await invoke("show_quicklaunch", { window });
    });
  }

  if (!closeRegistered) {
    await register('Esc', async () => {
      await invoke("hide_quicklaunch", { window });
    });
  }
};

export default initWindows;
