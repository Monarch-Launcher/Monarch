import Button from '@_ui/button';
import Page from '@_ui/page';
import { useSettings } from '@global/contexts/settingsProvider';
import { Settings } from '@global/types';
import { Input, Switch } from '@mantine/core';
import { invoke } from '@tauri-apps/api';
import * as React from 'react';
import { useForm } from 'react-hook-form';
import styled from 'styled-components';

const Section = styled.div`
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  margin-bottom: 2rem;
`;

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

const StyledInput = styled(Input)`
  border-radius: 12px;
  padding: 10px 14px;
  background-color: ${({ theme }) => theme.colors.background};
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.1);
  transition: all 0.3s ease;

  &:hover,
  &:focus {
    background-color: ${({ theme }) => theme.colors.secondary};
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  }

  &::placeholder {
    color: ${({ theme }) => theme.colors.black};
  }
`;

const StyledButton = styled(Button)`
  border-radius: 30px;
  padding: 10px 20px;
  font-weight: bold;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  transition: all 0.3s ease;

  &:hover {
    background-color: ${({ theme }) => theme.colors.primary};
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  }
`;

type FormValues = {
  settings: Settings;
  username: string;
  password: string;
  secret: string;
};

const SettingsPage = () => {
  const { register, handleSubmit } = useForm<FormValues>();
  const { settings, updateSettings, saveCredentials } = useSettings();

  const onSubmit = React.useCallback(
    async (values: FormValues) => {
      const { username, password } = values;
      await saveCredentials(username, password);
    },
    [saveCredentials],
  );

  const onSubmitSecret = React.useCallback(async (values: FormValues) => {
    const { secret } = values;
    await invoke('set_secret', {
      platform: 'steam',
      secret,
    });
  }, []);

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
    await invoke('delete_password', {
      platform: 'steam',
    });
  }, []);

  return (
    <Page title="Settings">
      <Section>
        <SectionTitle>Monarch</SectionTitle>
        <MonarchSwitch
          checked={settings.quicklaunch.enabled}
          onChange={toggleQuickLaunch}
          size="md"
          label="Quicklaunch (Windows and MacOS only)"
          labelPosition="left"
        />
      </Section>
      <Section>
        <SectionTitle>Steam</SectionTitle>
        <MonarchSwitch
          checked={settings.steam.manage}
          onChange={toggleSteam}
          size="md"
          label="Allow Monarch to manage Steam games"
          labelPosition="left"
        />

        <form onSubmit={handleSubmit(onSubmit)}>
          <FormContainer>
            <StyledInput
              placeholder="Steam username"
              variant="filled"
              {...register('username')}
            />
            <StyledInput
              placeholder="Steam password"
              variant="filled"
              type="password"
              {...register('password')}
            />
            <ButtonContainer>
              <StyledButton type="submit" variant="primary">
                Save
              </StyledButton>
            </ButtonContainer>
          </FormContainer>
        </form>

        <ButtonContainer>
          <StyledButton type="button" variant="primary" onClick={handleDelete}>
            Delete user
          </StyledButton>
        </ButtonContainer>

        <form onSubmit={handleSubmit(onSubmitSecret)}>
          <FormContainer>
            <StyledInput
              placeholder="Steam shared secret"
              variant="filled"
              type="password"
              {...register('secret')}
            />
            <ButtonContainer>
              <StyledButton type="submit" variant="primary">
                Save
              </StyledButton>
            </ButtonContainer>
          </FormContainer>
        </form>
      </Section>
    </Page>
  );
};

export default SettingsPage;
