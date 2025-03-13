import { invoke } from '@tauri-apps/api';
import {
  isRegistered,
  register,
  unregisterAll,
} from '@tauri-apps/api/globalShortcut';
import { Settings } from '@global/types';

// TODO: PROPER ERROR HANDLING
const initShortcuts = async () => {
  // Check if quicklaunch is enabled, else return
  const quicklaunchEnabled = await invoke('quicklaunch_is_enabled');
  if (!quicklaunchEnabled) {
    return;
  }

  // Define global shortcuts for quicklaunch (proto-typing)
  await unregisterAll();

  const quicklaunchSettings = await invoke('get_settings') as Settings;

  const quickLaunchRegistered = await isRegistered(quicklaunchSettings.quicklaunch.open_shortcut);
  const closeRegistered = await isRegistered(quicklaunchSettings.quicklaunch.close_shortcut);

  // Build initial quicklaunch
  await invoke('init_quicklaunch');

  if (!quickLaunchRegistered) {
    await register(quicklaunchSettings.quicklaunch.open_shortcut, async () => {
      await invoke("show_quicklaunch");
    });
  }

  if (!closeRegistered) {
    await register(quicklaunchSettings.quicklaunch.close_shortcut, async () => {
      await invoke("hide_quicklaunch");
    });
  }
};

export default initShortcuts;
