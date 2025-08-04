import Button from '@_ui/button';
import Page from '@_ui/page';
import { useSettings } from '@global/contexts/settingsProvider';
import { Settings } from '@global/types';
import { Input, Switch } from '@mantine/core';
import * as React from 'react';
import { useForm } from 'react-hook-form';
import { FaLock, FaSave, FaTrash, FaUser } from 'react-icons/fa';
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
  border-radius: 30px;
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
  width: 100vw;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-start;
  background: none;
  padding: 4rem 0 2rem 0;
  overflow-y: auto;
`;

const Card = styled.div`
  background: rgba(255, 255, 255, 0.1);
  border-radius: 28px;
  border: 2px solid ${({ theme }) => theme.colors.primary};
  padding: 2.5rem 2rem 2rem 2rem;
  margin-bottom: 2.5rem;
  width: 100%;
  max-width: 720px;
  transition: box-shadow 0.3s, transform 0.2s;
  backdrop-filter: blur(6px);
  position: relative;
  z-index: 1;
  &:hover {
    transform: scale(1.025);
  }
`;

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

type FormValues = {
  settings: Settings;
  username: string;
  password: string;
  secret: string;
};

const SettingsPage = () => {
  const { register, handleSubmit, reset } = useForm<FormValues>();
  const {
    settings,
    updateSettings,
    saveCredentials,
    deleteCredentials,
    saveSecret,
    deleteSecret,
  } = useSettings();
  const [feedback, setFeedback] = React.useState<string>('');
  const [deleteFeedback, setDeleteFeedback] = React.useState<string>('');
  const [secretFeedback, setSecretFeedback] = React.useState<string>('');

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

  return (
    <Page>
      <CenteredContainer>
        <Card>
          <SectionTitle>Monarch</SectionTitle>
          <AnimatedSwitch
            checked={settings.quicklaunch.enabled}
            onChange={toggleQuickLaunch}
            size="md"
            label="Quicklaunch (Requires application restart. Shortcut: Ctrl+Enter)"
            labelPosition="left"
          />
        </Card>
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
          <p style={{ color: '#fff', margin: '1rem 0 0.5rem 0', fontSize: '1.05rem', fontWeight: 400 }}>
            We recommend only filling in username to begin with, as you&apos;ll then be prompted by SteamCMD to enter a password
            on the first run. After that SteamCMD should remember you.
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
      </CenteredContainer>
    </Page>
  );
};

export default SettingsPage;
