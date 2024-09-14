import { invoke } from '@tauri-apps/api';
import {
  isRegistered,
  register,
  unregisterAll,
} from '@tauri-apps/api/globalShortcut';

// TODO: PROPER ERROR HANDLING
const initShortcuts = async () => {
  // Define global shortcuts for quicklaunch (proto-typing)
  await unregisterAll();

  const quickLaunchRegistered = await isRegistered('CommandOrControl+C');
  const closeRegistered = await isRegistered('Esc');

  // Build initial quicklaunch
  await invoke('init_quicklaunch');

  console.log("Finished setting up init_quicklaunch!");

  if (!quickLaunchRegistered) {
    await register('CommandOrControl+C', async () => {
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
