import Button from '@_ui/button';
import { NoticeBar, NoticeText } from '@_ui/noticeBar';
import { invoke } from '@tauri-apps/api/core';
import Page from '@_ui/page';
import { useSettings } from '@global/contexts/settingsProvider';
import { Settings } from '@global/types';
import { Input, Switch } from '@mantine/core';
import * as dialog from '@tauri-apps/plugin-dialog';
import * as React from 'react';
import { useForm } from 'react-hook-form';
import { FaFolderOpen, FaLock, FaSave, FaTrash, FaUser } from 'react-icons/fa';
import styled from 'styled-components';

const SectionTitle = styled.h3`
  color: ${({ theme }) => theme.colors.primary};
  font-size: 1.5rem;
  font-weight: 600;
  margin-bottom: 1rem;
`;

const MonarchSwitch = styled(Switch)`
  input:checked + .mantine-Switch-track {
    background-color: ${({ theme }) => theme.colors.primary};
    border-color: ${({ theme }) => theme.colors.primary};
  }

  .mantine-Switch-track {
    background-color: ${({ theme }) => theme.colors.secondary};
    border-color: ${({ theme }) => theme.colors.secondary};
  }

  .mantine-Switch-label {
    color: ${({ theme }) => theme.colors.white};
    font-family: 'IBM Plex Mono', Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 1rem;
    font-weight: 500;
  }

  &:hover {
    opacity: 0.9;
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

const StyledButton = styled(Button)`
  border-radius: 10px;
  padding: 10px 20px;
  font-weight: bold;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  transition: all 0.3s ease;
  border: none !important;

  &:hover {
    background-color: ${({ theme }) => theme.colors.primary};
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  }
`;

const CenteredContainer = styled.div`
  height: 100vh;
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: flex-start;
  background: none;
  padding: 3.5rem 0 2rem 0;
  overflow-y: auto;
  overflow-x: hidden;
`;

const Card = styled.div`
  background: rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  border: 2px solid ${({ theme }) => theme.colors.primary};
  padding: 2.5rem 2rem 2rem 2rem;
  margin-bottom: 2.5rem;
  width: 100%;
  max-width: none;
  transition: box-shadow 0.3s, transform 0.2s;
  backdrop-filter: blur(6px);
  position: relative;
  z-index: 1;
  box-sizing: border-box;
  overflow: hidden;
  &:hover {
    transform: none;
  }
`;

// NoticeBar and NoticeText are imported from '@_ui/noticeBar'

const AnimatedButton = styled(StyledButton)<{ $danger?: boolean }>`
  display: flex;
  align-items: center;
  gap: 0.5rem;
  transition: background 0.2s, transform 0.2s, box-shadow 0.2s, border 0.2s;
  ${({ $danger }) =>
    $danger && `
      background: #e74c3c;
      color: white;
      border: none !important;
      &:hover { background: #c0392b; }
    `}
  &:hover {
    background: linear-gradient(
      90deg,
      ${({ theme }) => theme.colors.primary} 60%,
      ${({ theme }) => theme.colors.secondary} 100%
    );
    transform: translateY(-3px) scale(1.04);
    box-shadow: 0 6px 24px rgba(0, 0, 0, 0.18);
  }
`;

const AnimatedSwitch = styled(MonarchSwitch)`
  transition: box-shadow 0.2s;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  .mantine-Switch-label {
    font-family: 'IBM Plex Mono', Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 1rem;
    font-weight: 500;
  }
  &:hover {
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
  }
`;

const Feedback = styled.div`
  color: ${({ theme }) => theme.colors.primary};
  font-size: 1rem;
  margin-top: 0.5rem;
  min-height: 1.2em;
`;

// Layout for left-side tabs
const SettingsWrapper = styled.div`
  display: grid;
  grid-template-columns: 240px minmax(0, 1fr);
  column-gap: 1.75rem;
  width: 100%;
  max-width: none;
  padding-left: 1rem;
  padding-right: 4rem;
  box-sizing: border-box;
  overflow-x: hidden;

  @media (max-width: 900px) {
    grid-template-columns: 1fr;
    row-gap: 1rem;
    padding-left: 1rem;
    padding-right: 2rem;
  }
`;

const Sidebar = styled.aside`
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  min-width: 220px;
  position: sticky;
  top: 3.5rem; /* match top padding of container */
  left: 0;

  @media (max-width: 900px) {
    flex-direction: row;
    min-width: 0;
    width: 100%;
    position: static;
  }
`;

const TabButton = styled.button<{ $active?: boolean }>`
  all: unset;
  cursor: pointer;
  padding: 0.75rem 1rem;
  border-radius: 10px;
  border: 2px solid
    ${({ theme, $active }) => ($active ? theme.colors.primary : 'rgba(255,255,255,0.15)')};
  color: ${({ theme }) => theme.colors.white};
  background: ${({ $active }) => ($active ? 'rgba(255,255,255,0.08)' : 'transparent')};
  font-weight: 600;
  transition: background 0.2s, transform 0.2s, box-shadow 0.2s, border 0.2s;
  &:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  }
  
  @media (max-width: 900px) {
    flex: 1;
    text-align: center;
  }
`;

const ContentArea = styled.div`
  min-width: 0;
  width: 100%;
  padding-right: 4rem;
  overflow-x: hidden;

  @media (max-width: 900px) {
    padding-right: 2rem;
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
  const [activeTab, setActiveTab] = React.useState<'monarch' | 'steam'>('monarch');
  const [feedback, setFeedback] = React.useState<string>('');
  const [deleteFeedback, setDeleteFeedback] = React.useState<string>('');
  const [secretFeedback, setSecretFeedback] = React.useState<string>('');
  const [gameFolderFeedback, setGameFolderFeedback] = React.useState<string>('');
  const [steamcmdInstalled, setSteamcmdInstalled] = React.useState<boolean | null>(null);
  const [steamcmdInstalling, setSteamcmdInstalling] = React.useState<boolean>(false);

  const onSubmit = React.useCallback(
    async (values: FormValues) => {
      const { username, password } = values;
      await saveCredentials(username, password, 'steam');
      setFeedback('Credentials saved!');
      setTimeout(() => setFeedback(''), 2000);
      reset({ username: '', password: '' });
    },
    [saveCredentials, reset],
  );

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

  const handleDelete = React.useCallback(async () => {
    deleteCredentials('steam');
    setDeleteFeedback('User deleted!');
    setTimeout(() => setDeleteFeedback(''), 2000);
  }, []);

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

  const checkSteamcmd = React.useCallback(async () => {
    try {
      const installed: boolean = await invoke('steamcmd_is_installed');
      setSteamcmdInstalled(installed);
    } catch (e) {
      console.error('Failed to check SteamCMD installation:', e);
      setSteamcmdInstalled(false);
    }
  }, []);

  React.useEffect(() => {
    if (activeTab === 'steam') {
      checkSteamcmd();
    }
  }, [activeTab, checkSteamcmd]);

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

  return (
    <Page>
      <CenteredContainer>
        <SettingsWrapper>
          <Sidebar>
            <TabButton $active={activeTab === 'monarch'} onClick={() => setActiveTab('monarch')}>
              Monarch
            </TabButton>
            <TabButton $active={activeTab === 'steam'} onClick={() => setActiveTab('steam')}>
              Steam
            </TabButton>
          </Sidebar>
          <ContentArea>
            {activeTab === 'monarch' && (
              <Card>
                <SectionTitle>Monarch</SectionTitle>
                <p
                  style={{
                    color: '#fff',
                    margin: '1rem 0 0.5rem 0',
                    fontSize: '1.05rem',
                    fontWeight: 400,
                  }}
                >
                  Our start-menu like launcher.
                  Enables quicker access to launching games by not requiring you to navigate
                  through the main application.
                  Stability issues under Linux running Wayland.
                  This is due to how Wayland handles global shortcuts.
                </p>
                <AnimatedSwitch
                  checked={settings.quicklaunch.enabled}
                  onChange={toggleQuickLaunch}
                  size="md"
                  label={(
                    <>
                      Quicklaunch (Requires application restart. Shortcut: Ctrl+Enter)
                    </>
                  )}
                  labelPosition="left"
                />

                {/* Game Folder Section */}
                <div style={{ marginTop: '2rem' }}>
                  <p style={{ color: '#fff', margin: '0 0 1rem 0', fontSize: '1rem', fontWeight: 400 }}>
                    Set the default folder where Monarch will download new games to.
                  </p>
                  <p
                    style={{ color: '#fff', margin: '0 0 1rem 0', fontSize: '1rem', fontWeight: 400 }}
                  >
                    (Currently disabled due to weird SteamCMD behaviour.
                    Games will instead be installed in your default steam library location.)
                  </p>
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
                        <AnimatedButton
                          type="button"
                          variant="primary"
                          onClick={handleGameFolderBrowse}
                        >
                          <FaFolderOpen /> Browse
                        </AnimatedButton>
                      </div>
                      <ButtonContainer>
                        <AnimatedButton type="submit" variant="primary">
                          <FaSave /> Save Game Folder
                        </AnimatedButton>
                      </ButtonContainer>
                      <div style={{ color: '#fff', fontWeight: 500 }}>
                        Current folder: {settings.monarch.game_folder || 'Not set'}
                      </div>
                      <Feedback>{gameFolderFeedback}</Feedback>
                    </FormContainer>
                  </form>
                </div>
              </Card>
            )}

            {activeTab === 'steam' && steamcmdInstalled === false && (
              <NoticeBar>
                <NoticeText>
                  SteamCMD is not installed. Some features may require it.
                </NoticeText>
                <AnimatedButton
                  type="button"
                  variant="primary"
                  onClick={handleInstallSteamcmd}
                  disabled={steamcmdInstalling}
                >
                  {steamcmdInstalling ? 'Downloading...' : 'Download SteamCMD'}
                </AnimatedButton>
              </NoticeBar>
            )}

            {activeTab === 'steam' && (
              <Card>
                <SectionTitle>Steam</SectionTitle>
                <AnimatedSwitch
                  checked={settings.steam.manage}
                  onChange={toggleSteam}
                  size="md"
                  label="Allow Monarch to manage Steam games"
                  labelPosition="left"
                />
                {/* Info paragraph about Steam login */}
                <p
                  style={{
                    color: '#fff',
                    margin: '1rem 0 0.5rem 0',
                    fontSize: '1.05rem',
                    fontWeight: 400,
                  }}
                >
                  We recommend only filling in username to begin with,
                  as you&apos;ll then be prompted by SteamCMD to enter a password on the first run.
                  After that SteamCMD should remember you.
                </p>
                <a
                  href="https://github.com/Monarch-Launcher/Monarch/blob/development/docs/steam_login.md"
                  target="_blank"
                  rel="noopener noreferrer"
                  style={{ color: '#fa5002', textDecoration: 'underline', fontWeight: 600 }}
                >
                  How Monarch handles user authentication
                </a>
                <form onSubmit={handleSubmit(onSubmit)}>
                  <FormContainer>
                    <Input
                      placeholder="Steam username"
                      variant="filled"
                      icon={<FaUser />}
                      radius="md"
                      {...register('username')}
                    />
                    <Input
                      placeholder="Steam password"
                      variant="filled"
                      type="password"
                      icon={<FaLock />}
                      radius="md"
                      {...register('password')}
                    />
                    <ButtonContainer>
                      <AnimatedButton type="submit" variant="primary">
                        <FaSave /> Save
                      </AnimatedButton>
                    </ButtonContainer>
                    <ButtonContainer>
                      <div
                        style={{
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'space-between',
                          width: '100%',
                        }}
                      >
                        <span style={{ color: '#fff', fontWeight: 500 }}>
                          Steam login: {settings.steam.username ? settings.steam.username : 'no login'}
                        </span>
                        <AnimatedButton
                          type="button"
                          variant="primary"
                          onClick={handleDelete}
                          $danger
                        >
                          <FaTrash /> Delete user
                        </AnimatedButton>
                      </div>
                    </ButtonContainer>
                    <Feedback>{feedback}</Feedback>
                  </FormContainer>
                </form>
                <form onSubmit={handleSubmit(onSubmitSecret)}>
                  <FormContainer>
                    {/* Instructional text and tutorial link for shared secret */}
                    <div style={{ color: '#fff', marginBottom: '0.5rem', fontWeight: 500 }}>
                      Insert your Steam shared secret, if using a 3rd party 2FA. (Advanced) <br />
                      <a
                        href="https://github.com/Monarch-Launcher/Monarch/blob/development/docs/steam_login.md"
                        target="_blank"
                        rel="noopener noreferrer"
                        style={{ color: '#fa5002', textDecoration: 'underline', fontWeight: 600 }}
                      >
                        How to set up your Steam shared secret (guide)
                      </a>
                    </div>
                    <Input
                      placeholder="Steam shared secret"
                      variant="filled"
                      type="password"
                      icon={<FaLock />}
                      radius="md"
                      {...register('secret')}
                    />
                    <ButtonContainer>
                      <AnimatedButton type="submit" variant="primary">
                        <FaSave /> Save
                      </AnimatedButton>
                    </ButtonContainer>
                    <ButtonContainer>
                      <div
                        style={{
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'space-between',
                          width: '100%',
                        }}
                      >
                        <span style={{ color: '#fff', fontWeight: 500 }}>
                          {settings.steam.twofa ? 'Secret set' : 'Secret not set'}
                        </span>
                        <AnimatedButton
                          type="button"
                          variant="primary"
                          onClick={handleDeleteSecret}
                          $danger
                        >
                          <FaTrash /> Delete secret
                        </AnimatedButton>
                      </div>
                    </ButtonContainer>
                    <Feedback>{secretFeedback}</Feedback>
                  </FormContainer>
                </form>
                <Feedback>{deleteFeedback}</Feedback>
              </Card>
            )}
          </ContentArea>
        </SettingsWrapper>
      </CenteredContainer>
    </Page>
  );
};

export default SettingsPage;
