import { dialog, invoke } from '@tauri-apps/api';
import * as React from 'react';

import type { Settings } from '../types';

type SettingsContextType = {
  settings: Settings;
  error: boolean;
  loading: boolean;
  getSettings: () => Promise<void>;
  updateSettings: (updatedSettings: Settings) => Promise<void>;
  saveCredentials: (username: string, password: string) => Promise<void>;
};

const defaultLauncherSettings = {
  game_folders: [],
  manage: false,
  username: '',
};

const defaultSettings = {
  epic: defaultLauncherSettings,
  steam: defaultLauncherSettings,
  monarch: {
    game_folder: '',
    monarch_home: '',
    run_on_startup: false,
    send_logs: true,
    start_minimized: false,
  },
  quicklaunch: {
    close_shortcut: 'Esc',
    open_shortcut: 'Super+Enter',
    enabled: true,
    size: 'medium',
  },
};

const initialState: SettingsContextType = {
  settings: defaultSettings,
  error: false,
  loading: false,
  getSettings: async () => { },
  updateSettings: async () => { },
  saveCredentials: async () => { },
};

const SettingsContext = React.createContext<SettingsContextType>(initialState);
export const useSettings = () => React.useContext(SettingsContext);

type Props = {
  children: React.ReactNode;
};

const SettingsProvider = ({ children }: Props) => {
  const [settings, setSettings] = React.useState<Settings>(defaultSettings);
  const [error, setError] = React.useState(false);
  const [loading, setLoading] = React.useState(false);

  const getSettings = React.useCallback(async () => {
    try {
      const result: Settings = await invoke('get_settings');
      setSettings(result);
    } catch (err) {
      setError(true);
    } finally {
      setLoading(false);
    }
  }, []);

  const updateSettings = React.useCallback(
    async (updatedSettings: Settings) => {
      try {
        // @ts-expect-error
        // eslint-disable-next-line @typescript-eslint/no-unused-vars
        const result: Settings = await invoke('set_settings', {
          settings: updatedSettings,
        });
      } catch (err) {
        setError(true);
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  const saveCredentials = React.useCallback(
    async (username: string, password: string) => {
      try {
        await invoke('set_password', {
          platform: "steam",
          username: username,
          password: password,
        });
      } catch (err) {
        await dialog.message(`An error has occured: ${err}`, {
          title: 'Error',
          type: 'error',
        });
      } finally {
        setLoading(false);
      }

    }, []
  );

  React.useEffect(() => {
    getSettings();
  }, [getSettings]);

  const value = React.useMemo<SettingsContextType>(() => {
    return {
      settings,
      error,
      loading,
      getSettings,
      updateSettings,
      saveCredentials
    };
  }, [settings, error, loading, getSettings, updateSettings, saveCredentials]);

  return (
    <SettingsContext.Provider value={value}>
      {children}
    </SettingsContext.Provider>
  );
};

export default SettingsProvider;
