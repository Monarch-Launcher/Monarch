import Button from '@_ui/button';
import { NoticeBar, NoticeText } from '@_ui/noticeBar';
import Page from '@_ui/page';
import { useSettings } from '@global/contexts/settingsProvider';
import { Settings } from '@global/types';
import { Input, Switch } from '@mantine/core';
import { invoke } from '@tauri-apps/api/core';
import * as dialog from '@tauri-apps/plugin-dialog';
import * as React from 'react';
import { useForm } from 'react-hook-form';
import { FaFolderOpen, FaLock, FaSave, FaSteam,FaTrash, FaUser } from 'react-icons/fa';
import { SiEpicgames } from 'react-icons/si';
import styled from 'styled-components';

const SectionTitle = styled.h3`
  color: ${({ theme }) => theme.colors.white};
  font-size: 1.5rem;
  font-weight: 600;
  margin-bottom: 1.5rem;
`;

const MonarchSwitch = styled(Switch)`
  input:checked + .mantine-Switch-track {
    background-color: ${({ theme }) => theme.colors.primary};
    border-color: ${({ theme }) => theme.colors.primary};
  }

  .mantine-Switch-track {
    background-color: ${({ theme }) => theme.colors.surface};
    border-color: ${({ theme }) => theme.colors.surface};
    cursor: pointer;
  }

  .mantine-Switch-label {
    color: ${({ theme }) => theme.colors.white};
    font-family: 'IBM Plex Mono', Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 1rem;
    font-weight: 500;
    padding-left: 1rem;
  }
`;

const FormContainer = styled.div`
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  margin-top: 1.5rem;
  width: 100%;
`;

const ButtonContainer = styled.div`
  display: flex;
  justify-content: flex-end;
  margin-top: 1rem;
`;

const CenteredContainer = styled.div`
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: flex-start;
  background: none;
  padding: 2rem 0;
`;

const Card = styled.div`
  background: transparent;
  border-radius: 12px;
  padding: 0;
  margin-bottom: 2rem;
  width: 100%;
  max-width: none;
  box-sizing: border-box;
`;

const Feedback = styled.div`
  color: ${({ theme }) => theme.colors.primary};
  font-size: 0.9rem;
  margin-top: 0.5rem;
  min-height: 1.2em;
  text-align: right;
`;

// Layout for left-side tabs
const SettingsWrapper = styled.div`
  display: grid;
  grid-template-columns: 240px minmax(0, 1fr);
  column-gap: 2rem;
  width: 100%;
  max-width: none;
  margin: 0;
  padding: 0 4rem 0 2rem;
  box-sizing: border-box;

  @media (max-width: 900px) {
    grid-template-columns: 1fr;
    row-gap: 1rem;
    padding: 0 1rem;
  }
`;

const Sidebar = styled.aside`
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  min-width: 220px;
  position: sticky;
  top: 2rem;
  height: fit-content;

  @media (max-width: 900px) {
    flex-direction: row;
    min-width: 0;
    width: 100%;
    position: static;
    overflow-x: auto;
    padding-bottom: 0.5rem;
  }
`;

const TabButton = styled.button<{ $active?: boolean }>`
  all: unset;
  cursor: pointer;
  padding: 0.75rem 1rem;
  border-radius: 8px;
  color: ${({ theme, $active }) => ($active ? theme.colors.white : 'rgba(255,255,255,0.6)')};
  background: ${({ theme, $active }) => ($active ? theme.colors.primary : 'transparent')};
  font-weight: 600;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  gap: 0.75rem;

  &:hover {
    background: ${({ theme, $active }) => ($active ? theme.colors.primary : 'rgba(255,255,255,0.05)')};
    color: ${({ theme }) => theme.colors.white};
  }
`;

const ContentArea = styled.div`
  min-width: 0;
  width: 100%;
`;

const Description = styled.p`
  color: rgba(255, 255, 255, 0.7);
  margin: 0 0 1.5rem 0;
  font-size: 1rem;
  line-height: 1.5;
`;

const Link = styled.a`
  color: ${({ theme }) => theme.colors.primary};
  text-decoration: none;
  font-weight: 600;
  &:hover {
    text-decoration: underline;
  }
`;

type FormValues = {
  settings: Settings;
  username: string;
  password: string;
  secret: string;
  gameFolder: string;
};

const SettingsPage = () => {
  const { register, handleSubmit, reset, setValue } = useForm<FormValues>();
  const {
    settings,
    updateSettings,
    saveCredentials,
    deleteCredentials,
    saveSecret,
    deleteSecret,
  } = useSettings();
  const [activeTab, setActiveTab] = React.useState<'monarch' | 'steam' | 'epic'>('monarch');

  // Feedback states
  const [feedback, setFeedback] = React.useState<string>('');
  const [deleteFeedback, setDeleteFeedback] = React.useState<string>('');
  const [secretFeedback, setSecretFeedback] = React.useState<string>('');
  const [gameFolderFeedback, setGameFolderFeedback] = React.useState<string>('');

  // Installation states
  const [steamcmdInstalled, setSteamcmdInstalled] = React.useState<boolean | null>(null);
  const [steamcmdInstalling, setSteamcmdInstalling] = React.useState<boolean>(false);
  const [legendaryInstalled, setLegendaryInstalled] = React.useState<boolean | null>(null);
  const [legendaryInstalling, setLegendaryInstalling] = React.useState<boolean>(false);

  // Cache states
  const [cacheLoading, setCacheLoading] = React.useState<boolean>(false);
  const [cacheSize, setCacheSize] = React.useState<string | null>(null);

  // --- Monarch Handlers ---

  const toggleQuickLaunch = React.useCallback(
    async (e: React.ChangeEvent<HTMLInputElement>) => {
      const updatedSettings: Settings = {
        ...settings,
        quicklaunch: {
          ...settings.quicklaunch,
          enabled: e.currentTarget.checked,
        },
      };
      await updateSettings(updatedSettings);
    },
    [settings, updateSettings],
  );

  const handleGameFolderBrowse = React.useCallback(async () => {
    try {
      const selected = await dialog.open({
        multiple: false,
        title: 'Choose Game Folder',
        directory: true,
      });

      if (selected && typeof selected === 'string') {
        setValue('gameFolder', selected);
      }
    } catch (err) {
      console.error('Failed to open folder picker:', err);
    }
  }, [setValue]);

  const handleGameFolderSave = React.useCallback(
    async (values: FormValues) => {
      const { gameFolder } = values;
      if (gameFolder.trim()) {
        const updatedSettings: Settings = {
          ...settings,
          monarch: {
            ...settings.monarch,
            game_folder: gameFolder.trim(),
          },
        };
        await updateSettings(updatedSettings);
        setGameFolderFeedback('Game folder saved!');
        setTimeout(() => setGameFolderFeedback(''), 2000);
        reset({ gameFolder: '' });
      }
    },
    [settings, updateSettings, reset],
  );

  const formatBytes = React.useCallback((bytes: number) => {
    if (!Number.isFinite(bytes) || bytes < 0) return 'Unavailable';
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    const value = bytes / (k ** i);
    return `${value.toFixed(value >= 100 ? 0 : value >= 10 ? 1 : 2)} ${sizes[i]}`;
  }, []);

  const fetchCacheSize = React.useCallback(async () => {
    try {
      setCacheLoading(true);
      const size: number = await invoke('get_cache_size');
      setCacheSize(formatBytes(size));
    } catch (_e) {
      setCacheSize(null);
    } finally {
      setCacheLoading(false);
    }
  }, [formatBytes]);

  const handleClearCache = React.useCallback(async () => {
    try {
      setCacheLoading(true);
      await invoke('clear_cached_images');
      await fetchCacheSize();
    } catch (_e) {
      // ignore
    } finally {
      setCacheLoading(false);
    }
  }, [fetchCacheSize]);

  const handleOpenLogs = React.useCallback(async () => {
    try {
      await invoke('open_logs');
    } catch (e) {
      console.error('Failed to open logs:', e);
    }
  }, []);

  // --- Steam Handlers ---

  const checkSteamcmd = React.useCallback(async () => {
    try {
      const installed: boolean = await invoke('steamcmd_is_installed');
      setSteamcmdInstalled(installed);
    } catch (e) {
      console.error('Failed to check SteamCMD installation:', e);
      setSteamcmdInstalled(false);
    }
  }, []);

  const handleInstallSteamcmd = React.useCallback(async () => {
    try {
      setSteamcmdInstalling(true);
      await invoke('install_steamcmd');
      await checkSteamcmd();
    } catch (e) {
      console.error('Failed to install SteamCMD:', e);
    } finally {
      setSteamcmdInstalling(false);
    }
  }, [checkSteamcmd]);

  const toggleSteam = React.useCallback(
    async (e: React.ChangeEvent<HTMLInputElement>) => {
      const updatedSettings: Settings = {
        ...settings,
        steam: {
          ...settings.steam,
          manage: e.currentTarget.checked,
        },
      };
      await updateSettings(updatedSettings);
    },
    [settings, updateSettings],
  );

  const onSubmitSteam = React.useCallback(
    async (values: FormValues) => {
      const { username, password } = values;
      await saveCredentials(username, password, 'steam');
      setFeedback('Credentials saved!');
      setTimeout(() => setFeedback(''), 2000);
      reset({ username: '', password: '' });
    },
    [saveCredentials, reset],
  );

  const handleDeleteSteam = React.useCallback(async () => {
    deleteCredentials('steam');
    setDeleteFeedback('User deleted!');
    setTimeout(() => setDeleteFeedback(''), 2000);
  }, [deleteCredentials]);

  const onSubmitSecret = React.useCallback(
    async (values: FormValues) => {
      const { secret } = values;
      saveSecret(secret, 'steam');
      setSecretFeedback('Shared secret saved!');
      setTimeout(() => setSecretFeedback(''), 2000);
      reset({ secret: '' });
    },
    [reset, saveSecret],
  );

  const handleDeleteSecret = React.useCallback(async () => {
    await deleteSecret('steam');
  }, [deleteSecret]);

  // --- Epic Handlers ---

  const checkLegendary = React.useCallback(async () => {
    try {
      const installed: boolean = await invoke('legendary_is_installed');
      setLegendaryInstalled(installed);
    } catch (e) {
      console.error('Failed to check Legendary installation:', e);
      setLegendaryInstalled(false);
    }
  }, []);

  const handleInstallLegendary = React.useCallback(async () => {
    try {
      setLegendaryInstalling(true);
      await invoke('install_legendary');
      await checkLegendary();
    } catch (e) {
      console.error('Failed to install Legendary:', e);
    } finally {
      setLegendaryInstalling(false);
    }
  }, [checkLegendary]);

  const toggleEpic = React.useCallback(
    async (e: React.ChangeEvent<HTMLInputElement>) => {
      const updatedSettings: Settings = {
        ...settings,
        epic: {
          ...settings.epic,
          manage: e.currentTarget.checked,
        },
      };
      await updateSettings(updatedSettings);
    },
    [settings, updateSettings],
  );

  const onSubmitEpic = React.useCallback(
    async (values: FormValues) => {
      const { username, password } = values;
      await saveCredentials(username, password, 'epic');
      setFeedback('Credentials saved!');
      setTimeout(() => setFeedback(''), 2000);
      reset({ username: '', password: '' });
    },
    [saveCredentials, reset],
  );

  const handleDeleteEpic = React.useCallback(async () => {
    deleteCredentials('epic');
    setDeleteFeedback('User deleted!');
    setTimeout(() => setDeleteFeedback(''), 2000);
  }, [deleteCredentials]);

  // --- Reset Handler (Moved down to access checkSteamcmd/checkLegendary) ---

  const handleResetToDefaultSettings = React.useCallback(async () => {
    try {
      const defaultSettings = await invoke<Settings>('default_settings');
      updateSettings(defaultSettings);
      if (activeTab === 'monarch') fetchCacheSize();
      if (activeTab === 'steam') checkSteamcmd();
      if (activeTab === 'epic') checkLegendary();
    } catch (e) {
      console.error('Failed to reset settings');
      if (e && typeof e === 'object' && 'settings' in e) {
        updateSettings(e.settings as Settings);
      }
    }
  }, [updateSettings, activeTab, fetchCacheSize, checkSteamcmd, checkLegendary]);

  // --- Effects ---

  React.useEffect(() => {
    if (activeTab === 'monarch') fetchCacheSize();
    if (activeTab === 'steam') checkSteamcmd();
    if (activeTab === 'epic') checkLegendary();
  }, [activeTab, fetchCacheSize, checkSteamcmd, checkLegendary]);

  return (
    <Page>
      <CenteredContainer>
        <SettingsWrapper>
          <Sidebar>
            <TabButton $active={activeTab === 'monarch'} onClick={() => setActiveTab('monarch')}>
              Monarch
            </TabButton>
            <TabButton $active={activeTab === 'steam'} onClick={() => setActiveTab('steam')}>
              <FaSteam size={20} /> Steam
            </TabButton>
            <TabButton $active={activeTab === 'epic'} onClick={() => setActiveTab('epic')}>
              <SiEpicgames size={20} /> Epic Games
            </TabButton>
          </Sidebar>

          <ContentArea>
            {activeTab === 'monarch' && (
              <Card>
                <SectionTitle>General</SectionTitle>
                <Description>
                  Configure general behavior and preferences for the Monarch launcher.
                </Description>

                <MonarchSwitch
                  checked={settings.quicklaunch.enabled}
                  onChange={toggleQuickLaunch}
                  size="md"
                  label="Quicklaunch (Requires restart. Shortcut: Ctrl+Enter)"
                  labelPosition="left"
                />

                <div style={{ marginTop: '2.5rem' }}>
                  <SectionTitle>Game Library Folder</SectionTitle>
                  <Description>
                    Set the default folder where Monarch will download new games.
                  </Description>
                  <form onSubmit={handleSubmit(handleGameFolderSave)}>
                    <FormContainer>
                      <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'flex-start' }}>
                        <Input
                          placeholder="Path to game folder"
                          variant="filled"
                          radius="md"
                          style={{ flex: 1 }}
                          {...register('gameFolder')}
                        />
                        <Button
                          type="button"
                          variant="secondary"
                          onClick={handleGameFolderBrowse}
                          leftIcon={FaFolderOpen}
                        >
                          Browse
                        </Button>
                      </div>
                      <ButtonContainer>
                        <Button type="submit" variant="secondary" leftIcon={FaSave}>
                          Save
                        </Button>
                      </ButtonContainer>
                      <div style={{ color: 'rgba(255,255,255,0.7)', fontSize: '0.9rem' }}>
                        Current: {settings.monarch.game_folder || 'Not set'}
                      </div>
                      <Feedback>{gameFolderFeedback}</Feedback>
                    </FormContainer>
                  </form>
                </div>

                <div style={{ marginTop: '2.5rem' }}>
                  <SectionTitle>Storage & Cache</SectionTitle>
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: '1rem' }}>
                    <p style={{ color: 'rgba(255,255,255,0.7)', margin: 0 }}>
                      Cached images: {cacheLoading ? 'Calculating...' : cacheSize ?? 'Unavailable'}
                    </p>
                    <Button type="button" variant="secondary" onClick={handleClearCache} disabled={cacheLoading}>
                      Clear cache
                    </Button>
                  </div>
                </div>

                <div style={{ marginTop: '2.5rem' }}>
                  <SectionTitle>System</SectionTitle>
                  <div style={{ display: 'flex', gap: '1rem', justifyContent: 'flex-start' }}>
                    <Button type="button" variant="secondary" onClick={handleOpenLogs}>
                      Open Logs
                    </Button>
                    <Button
                      type="button"
                      variant="danger"
                      onClick={handleResetToDefaultSettings}
                    >
                      Reset to Defaults
                    </Button>
                  </div>
                </div>
              </Card>
            )}

            {activeTab === 'steam' && (
              <>
                {steamcmdInstalled === false && (
                  <NoticeBar>
                    <NoticeText>
                      SteamCMD is not installed. Required for Steam integration.
                    </NoticeText>
                    <Button
                      type="button"
                      variant="secondary"
                      onClick={handleInstallSteamcmd}
                      loading={steamcmdInstalling}
                    >
                      {steamcmdInstalling ? 'Installing...' : 'Install SteamCMD'}
                    </Button>
                  </NoticeBar>
                )}

                <Card>
                  <SectionTitle>Steam Integration</SectionTitle>
                  <MonarchSwitch
                    checked={settings.steam.manage}
                    onChange={toggleSteam}
                    size="md"
                    label="Allow Monarch to manage Steam games"
                    labelPosition="left"
                  />

                  <div style={{ marginTop: '2rem' }}>
                    <Description>
                      Enter your Steam credentials to enable library synchronization and game downloads.
                      <br />
                      <Link
                        href="https://github.com/Monarch-Launcher/Monarch/blob/development/docs/steam_login.md"
                        target="_blank"
                        rel="noopener noreferrer"
                      >
                        Read about authentication security
                      </Link>
                    </Description>

                    <form onSubmit={handleSubmit(onSubmitSteam)}>
                      <FormContainer>
                        <Input
                          placeholder="Steam Username"
                          variant="filled"
                          icon={<FaUser />}
                          radius="md"
                          {...register('username')}
                        />
                        <Input
                          placeholder="Steam Password"
                          variant="filled"
                          type="password"
                          icon={<FaLock />}
                          radius="md"
                          {...register('password')}
                        />
                        <ButtonContainer>
                          <Button type="submit" variant="secondary" leftIcon={FaSave}>
                            Save Credentials
                          </Button>
                        </ButtonContainer>

                        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginTop: '1rem' }}>
                          <span style={{ color: 'rgba(255,255,255,0.7)' }}>
                            Status: {settings.steam.username ? `Logged in as ${settings.steam.username}` : 'Not logged in'}
                          </span>
                          {settings.steam.username && (
                            <Button type="button" variant="danger" onClick={handleDeleteSteam} leftIcon={FaTrash}>
                              Remove Account
                            </Button>
                          )}
                        </div>
                        <Feedback>{feedback}</Feedback>
                        <Feedback>{deleteFeedback}</Feedback>
                      </FormContainer>
                    </form>
                  </div>

                  <div style={{ marginTop: '2.5rem', borderTop: '1px solid rgba(255,255,255,0.1)', paddingTop: '2rem' }}>
                    <SectionTitle>Steam Guard (2FA)</SectionTitle>
                    <Description>
                      If you use Steam Guard Mobile Authenticator, you can provide your shared secret here.
                      <br />
                      <Link
                        href="https://github.com/Monarch-Launcher/Monarch/blob/development/docs/steam_login.md"
                        target="_blank"
                        rel="noopener noreferrer"
                      >
                        How to find your shared secret
                      </Link>
                    </Description>

                    <form onSubmit={handleSubmit(onSubmitSecret)}>
                      <FormContainer>
                        <Input
                          placeholder="Shared Secret"
                          variant="filled"
                          type="password"
                          icon={<FaLock />}
                          radius="md"
                          {...register('secret')}
                        />
                        <ButtonContainer>
                          <Button type="submit" variant="secondary" leftIcon={FaSave}>
                            Save Secret
                          </Button>
                        </ButtonContainer>

                        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginTop: '1rem' }}>
                          <span style={{ color: 'rgba(255,255,255,0.7)' }}>
                            Status: {settings.steam.twofa ? 'Secret configured' : 'Not configured'}
                          </span>
                          {settings.steam.twofa && (
                            <Button type="button" variant="danger" onClick={handleDeleteSecret} leftIcon={FaTrash}>
                              Remove Secret
                            </Button>
                          )}
                        </div>
                        <Feedback>{secretFeedback}</Feedback>
                      </FormContainer>
                    </form>
                  </div>
                </Card>
              </>
            )}

            {activeTab === 'epic' && (
              <>
                {legendaryInstalled === false && (
                  <NoticeBar>
                    <NoticeText>
                      Legendary is not installed. Required for Epic Games integration.
                    </NoticeText>
                    <Button
                      type="button"
                      variant="secondary"
                      onClick={handleInstallLegendary}
                      loading={legendaryInstalling}
                    >
                      {legendaryInstalling ? 'Installing...' : 'Install Legendary'}
                    </Button>
                  </NoticeBar>
                )}

                <Card>
                  <SectionTitle>Epic Games Integration</SectionTitle>
                  <MonarchSwitch
                    checked={settings.epic.manage}
                    onChange={toggleEpic}
                    size="md"
                    label="Allow Monarch to manage Epic Games"
                    labelPosition="left"
                  />

                  <div style={{ marginTop: '2rem' }}>
                    <Description>
                      Enter your Epic Games credentials to enable library synchronization and game downloads.
                    </Description>

                    <form onSubmit={handleSubmit(onSubmitEpic)}>
                      <FormContainer>
                        <Input
                          placeholder="Epic Username / Email"
                          variant="filled"
                          icon={<FaUser />}
                          radius="md"
                          {...register('username')}
                        />
                        <Input
                          placeholder="Password"
                          variant="filled"
                          type="password"
                          icon={<FaLock />}
                          radius="md"
                          {...register('password')}
                        />
                        <ButtonContainer>
                          <Button type="submit" variant="secondary" leftIcon={FaSave}>
                            Save Credentials
                          </Button>
                        </ButtonContainer>

                        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginTop: '1rem' }}>
                          <span style={{ color: 'rgba(255,255,255,0.7)' }}>
                            Status: {settings.epic.username ? `Logged in as ${settings.epic.username}` : 'Not logged in'}
                          </span>
                          {settings.epic.username && (
                            <Button type="button" variant="danger" onClick={handleDeleteEpic} leftIcon={FaTrash}>
                              Remove Account
                            </Button>
                          )}
                        </div>
                        <Feedback>{feedback}</Feedback>
                        <Feedback>{deleteFeedback}</Feedback>
                      </FormContainer>
                    </form>
                  </div>
                </Card>
              </>
            )}
          </ContentArea>
        </SettingsWrapper>
      </CenteredContainer>
    </Page>
  );
};

export default SettingsPage;
