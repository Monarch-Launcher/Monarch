import { dialog, invoke } from '@tauri-apps/api';
import * as React from 'react';

import type { Settings } from '../types';

type SettingsContextType = {
  settings: Settings;
  error: boolean;
  loading: boolean;
  getSettings: () => Promise<void>;
  updateSettings: (updatedSettings: Settings) => Promise<void>;
  saveCredentials: (username: string, password: string, platform: string) => Promise<void>;
  deleteCredentials: (platform: string) => Promise<void>;
  deleteSecret: (platform: string) => Promise<void>;
  saveSecret: (secret: string, platform: string) => Promise<void>;
};

const defaultLauncherSettings = {
  game_folders: [],
  manage: false,
  username: '',
  twofa: false,
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
  deleteCredentials: async () => { },
  deleteSecret: async () => { },
  saveSecret: async () => { },
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
        const result: Settings = await invoke('set_settings', {
          settings: updatedSettings,
        });
        setSettings(result);
      } catch (err) {
        setError(true);
        getSettings();
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  const saveCredentials = React.useCallback(
    async (username: string, password: string, platform: string) => {
      try {
          const result: Settings = await invoke('set_password', {
            platform: platform,
            username: username,
            password: password,
          });
          setSettings(result);
      } catch (err) {
        await dialog.message(`An error has occured: ${err}`, {
          title: 'Error',
          type: 'error',
        });
        await getSettings();
      } finally {
        setLoading(false);
      }

    }, []
  );

  const deleteCredentials = React.useCallback(
    async (platform: string) => {
      try {
        const result: Settings = await invoke('delete_password', {
          platform: platform,
        });
        setSettings(result);
      } catch (err) {
        await dialog.message(`An error has occured: ${err}`, {
          title: 'Error',
          type: 'error',
        });
        await getSettings();
      } finally {
        setLoading(false);
      }
    }, []
  )

const saveSecret = React.useCallback(
  async (secret: string, platform: string) => {
    try {
      const result: Settings = await invoke('set_secret', {
        platform: platform,
        secret: secret,
      });
      setSettings(result);
    } catch (err) {
      await dialog.message(`An error has occured: ${err}`, {
        title: 'Error',
        type: 'error',
      });
      await getSettings();
    } finally {
      setLoading(false);
    }
  }, []
);

 const deleteSecret = React.useCallback(
    async (platform: string) => {
      try {
        const result: Settings = await invoke('delete_secret', {
          platform: platform,
        });
        setSettings(result);
      } catch (err) {
        await dialog.message(`An error has occured: ${err}`, {
          title: 'Error',
          type: 'error',
        });
        await getSettings();
      } finally {
        setLoading(false);
      }
    }, []
  ) 

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
      saveCredentials,
      deleteCredentials,
      deleteSecret,
      saveSecret
    };
  }, [settings, error, loading, getSettings, updateSettings, saveCredentials, deleteCredentials, deleteSecret, saveSecret]);

  return (
    <SettingsContext.Provider value={value}>
      {children}
    </SettingsContext.Provider>
  );
};

export default SettingsProvider;
