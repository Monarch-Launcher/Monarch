import { invoke } from '@tauri-apps/api';
import {
  isRegistered,
  register,
  unregisterAll,
} from '@tauri-apps/api/globalShortcut';

// TODO: PROPER ERROR HANDLING
const initShortcuts = async () => {
  // Check if quicklaunch is enabled, else return
  const quicklaunchEnabled = await invoke('quicklaunch_is_enabled');
  if (!quicklaunchEnabled) {
    return;
  }

  // Define global shortcuts for quicklaunch (proto-typing)
  await unregisterAll();

  const quickLaunchRegistered = await isRegistered('CommandOrControl+Space');
  const closeRegistered = await isRegistered('Esc');

  // Build initial quicklaunch
  await invoke('init_quicklaunch');

  console.log("Finished setting up init_quicklaunch!");

  if (!quickLaunchRegistered) {
    await register('CommandOrControl+Space', async () => {
      await invoke("show_quicklaunch");
    });
    console.log("Finished setting up show quicklaunch!");
  }

  if (!closeRegistered) {
    await register('Esc', async () => {
      await invoke("hide_quicklaunch");
    });
    console.log("Finished setting up hide quicklaunch!");
  }

  console.log("Finished setting up initShortcuts!");
};

export default initShortcuts;