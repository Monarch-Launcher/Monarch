import Button from '@_ui/button';
import Page from '@_ui/page';
import { useSettings } from '@global/contexts/settingsProvider';
import { Settings } from '@global/types';
import { Input, Switch } from '@mantine/core';
import * as React from 'react';
import { useForm } from 'react-hook-form';
import styled from 'styled-components';
import { invoke } from '@tauri-apps/api';

const Section = styled.div`
  display: flex;
  flex-direction: column;
  align-items: flex-start;
`;

const SectionTitle = styled.h3`
  color: ${({ theme }) => theme.colors.primary};
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
`;

const FormContainer = styled.div`
  display: flex;
  flex-direction: column;
  gap: 1rem;
  margin-top: 1rem;
`;

const ButtonContainer = styled.div`
  display: flex;
  flex-direction: row;
  justify-content: space-between;
`;

type FormValues = {
  settings: Settings;
  username: string;
  password: string;
};

const SettingsPage = () => {
  const { register, handleSubmit } = useForm<FormValues>();
  const { settings, updateSettings, saveCredentials } = useSettings();

  const onSubmit = React.useCallback(
    async (values: FormValues) => {
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const { username, password } = values;

      await saveCredentials(username, password);
    },
    [updateSettings, saveCredentials],
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
    [settings],
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
    [settings],
  );

  const handleDelete = React.useCallback(
    async () => {
      await invoke('delete_password', {
        platform: "steam",
      });
    }, []
  );

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
            <Input
              placeholder="Steam username"
              variant="filled"
              {...register('username')}
            />
            <Input
              placeholder="Steam password"
              variant="filled"
              type="password"
              {...register('password')}
            />
            <ButtonContainer>
              <Button type="submit" variant="primary">
                Save
              </Button>
            </ButtonContainer>
          </FormContainer>
        </form>
        <ButtonContainer>
          <Button type="submit" variant="primary" onClick={handleDelete}>
            Delete user
          </Button>
        </ButtonContainer>
      </Section>
    </Page>
  );
};

export default SettingsPage;
